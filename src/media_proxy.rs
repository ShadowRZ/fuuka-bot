use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use bytes::{BufMut, Bytes, BytesMut};
use core::str;
use file_format::FileFormat;
use hmac::{Hmac, Mac};
use jose_jwk::{
    Jwk,
    jose_b64::{base64ct::Encoding, serde::Secret},
    jose_jwa::{Algorithm, Signing},
};
use matrix_sdk::{
    media::{MediaFormat, MediaRequestParameters},
    ruma::{MilliSecondsSinceUnixEpoch, MxcUri, OwnedMxcUri, events::room::MediaSource},
};
use serde_json::{Value, json};
use sha2::Sha512;
use std::sync::Arc;
use time::{Duration, ext::NumericalDuration};
use url::Url;
// Node.js Buffer emits unpadded URL safe Base64
use jose_jwk::jose_b64::base64ct::Base64UrlUnpadded;

type HmacSha512 = Hmac<Sha512>;
type Result<T> = std::result::Result<T, MediaProxyError>;

#[derive(Clone)]
pub struct MediaProxy {
    state: Arc<MediaProxyState>,
}

struct MediaProxyState {
    hmac_key: Secret,
    client: matrix_sdk::Client,
}

impl MediaProxy {
    pub fn new(jwk: Jwk, client: &matrix_sdk::Client) -> anyhow::Result<Self> {
        let client = client.clone();

        let Some(hmac_key) = extract_jwk_hmac_key(jwk) else {
            anyhow::bail!("No valid HMAC-SHA512 JWK token provided!");
        };
        let state = Arc::new(MediaProxyState { hmac_key, client });

        Ok(Self { state })
    }

    pub fn router(&self) -> axum::Router {
        axum::Router::new()
            .route("/health", get(Self::health))
            .route("/v1/media/download/{token}", get(Self::get_media))
            .with_state(self.state.clone())
    }

    pub fn create_media_url(
        &self,
        public_url: &Url,
        mxc: &MxcUri,
        ttl_seconds: u32,
    ) -> anyhow::Result<Url> {
        let mut end = MilliSecondsSinceUnixEpoch::now();
        end.0 += (ttl_seconds * 1000).into();
        let token = create_media_token(&self.state.hmac_key, mxc, end)?;

        let mut public_url = public_url.clone();
        public_url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("URL is cannot-be-a-base!"))?
            .pop_if_empty()
            .extend(&["v1", "media", "download", &token]);
        Ok(public_url)
    }

    async fn health() -> Json<Value> {
        Json(json!({ "ok": true }))
    }

    async fn get_media(
        Path(token): Path<String>,
        State(state): State<Arc<MediaProxyState>>,
    ) -> self::Result<(HeaderMap, Bytes)> {
        let client = &state.client;
        let hmac_key = &state.hmac_key;

        let (mxc, expiry) = verify_media_token(hmac_key, &token)?;
        let MilliSecondsSinceUnixEpoch(now) = MilliSecondsSinceUnixEpoch::now();
        let now = Duration::milliseconds(now.into());

        if now > expiry {
            return Err(MediaProxyError::TokenExpired);
        }

        let request = MediaRequestParameters {
            source: MediaSource::Plain(mxc),
            format: MediaFormat::File,
        };
        let data: Bytes = client
            .media()
            .get_media_content(&request, false)
            .await?
            .into();
        let mut sent_headers = HeaderMap::new();
        let format = FileFormat::from_bytes(&data);
        let content_type = format.media_type();

        sent_headers.append(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_str(content_type)?,
        );
        sent_headers.append(
            reqwest::header::CONTENT_LENGTH,
            HeaderValue::from_str(&data.len().to_string())?,
        );

        Ok((sent_headers, data))
    }
}

fn extract_jwk_hmac_key(jwk: Jwk) -> Option<Secret> {
    use jose_jwk::{Key, Oct};
    if !is_hmac_sha512_jwk(&jwk) {
        return None;
    }
    let Key::Oct(Oct { k }) = jwk.key else {
        return None;
    };

    Some(k)
}

fn is_hmac_sha512_jwk(jwk: &Jwk) -> bool {
    jwk.prm.alg == Some(Algorithm::Signing(Signing::Hs512))
}

fn create_media_token(
    hmac_key: &[u8],
    mxc: &MxcUri,
    end: MilliSecondsSinceUnixEpoch,
) -> anyhow::Result<String> {
    let MilliSecondsSinceUnixEpoch(end) = end;
    let mut buf = BytesMut::new();
    buf.put_u8(1);
    let mut signed = BytesMut::new();
    signed.put_f64(end.into());
    signed.put_slice(mxc.as_bytes());

    let mut hmac = HmacSha512::new_from_slice(hmac_key)?;

    hmac.update(&signed);
    let result = hmac.finalize();
    let code_bytes = result.into_bytes();

    buf.put_slice(&code_bytes);
    buf.put(signed);

    let token = Base64UrlUnpadded::encode_string(&buf);

    Ok(token)
}

fn verify_media_token(hmac_key: &[u8], token: &str) -> self::Result<(OwnedMxcUri, Duration)> {
    use bytes::Buf;
    let mut hmac = HmacSha512::new_from_slice(hmac_key)?;

    let data = Base64UrlUnpadded::decode_vec(token).map_err(|_| MediaProxyError::InvalidToken)?;
    let mut data = data.as_slice();
    let version = data.get_u8();

    if version != 1 {
        return Err(MediaProxyError::UnknownTokenVersion(version));
    }

    let sig = data.copy_to_bytes(64);
    let mut ex_data = data.chunk();
    hmac.update(ex_data);
    hmac.verify_slice(&sig)
        .map_err(|_| MediaProxyError::BrokenSignature)?;

    let expiry = ex_data.get_f64().milliseconds();
    let mxc = ex_data.chunk();
    let mxc: &MxcUri = str::from_utf8(mxc)?.into();

    Ok((mxc.into(), expiry))
}

#[derive(Debug)]
enum MediaProxyError {
    InvalidToken,
    TokenExpired,
    BrokenSignature,
    UnknownTokenVersion(u8),
    Other(anyhow::Error),
}

impl IntoResponse for MediaProxyError {
    fn into_response(self) -> Response {
        match self {
            MediaProxyError::InvalidToken => {
                (StatusCode::BAD_REQUEST, "Token is invalid".to_string())
            }
            MediaProxyError::TokenExpired => {
                (StatusCode::NOT_FOUND, "Media token expired".to_string())
            }
            MediaProxyError::BrokenSignature => {
                (StatusCode::BAD_REQUEST, "Signature is broken".to_string())
            }
            MediaProxyError::UnknownTokenVersion(version) => (
                StatusCode::BAD_REQUEST,
                format!("Unrecognized version of media token (${version})"),
            ),
            MediaProxyError::Other(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something wrong happened: {e:#}"),
            ),
        }
        .into_response()
    }
}

impl<E> From<E> for MediaProxyError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Other(err.into())
    }
}

mod tests {

    #[test]
    pub fn test_encode() {
        use jose_jwk::Jwk;
        use matrix_sdk::ruma::{MilliSecondsSinceUnixEpoch, UInt, mxc_uri};
        use pretty_assertions::assert_eq;

        let data = serde_json::json!({
            "key_ops": [
                "sign",
                "verify"
            ],
            "ext": true,
            "kty": "oct",
            "k": "NhQyY_ybKwtm-np_fPgIq_808a5NsLuxxkHFqQaRvJbX_Jl5DLVo_cwf3ZWawvG1GE7iziexYNgPNzYOk9Ndc7nxV7xkw0URyVFOWXDKvR_f4HoQxhHYx7tlTML7oqiU-zG4s2vh1U3vCq93v7PLWy3sqdahyOX7JBo2BHEQlog",
            "alg": "HS512"
        });
        let jwk: Jwk = serde_json::from_value(data).unwrap();
        let secret = super::extract_jwk_hmac_key(jwk).unwrap();

        let token = super::create_media_token(
            &secret,
            mxc_uri!("mxc://example.org/abc123"),
            MilliSecondsSinceUnixEpoch(UInt::from(50u32)),
        )
        .unwrap();

        assert_eq!(
            token,
            "ASk3EMAJGCGQYt0Z6tTslBWDulxqCBiUi8A7W8BwQ32tfRdHxTkNIQrV6iNqCHvltNTlJlUgOgmT2qbdIi_icPxASQAAAAAAAG14YzovL2V4YW1wbGUub3JnL2FiYzEyMw"
        )
    }

    #[test]
    pub fn test_decode() {
        use jose_jwk::Jwk;
        use matrix_sdk::ruma::mxc_uri;
        use pretty_assertions::assert_eq;

        let data = serde_json::json!({
            "key_ops": [
                "sign",
                "verify"
            ],
            "ext": true,
            "kty": "oct",
            "k": "NhQyY_ybKwtm-np_fPgIq_808a5NsLuxxkHFqQaRvJbX_Jl5DLVo_cwf3ZWawvG1GE7iziexYNgPNzYOk9Ndc7nxV7xkw0URyVFOWXDKvR_f4HoQxhHYx7tlTML7oqiU-zG4s2vh1U3vCq93v7PLWy3sqdahyOX7JBo2BHEQlog",
            "alg": "HS512"
        });
        let jwk: Jwk = serde_json::from_value(data).unwrap();
        let secret = super::extract_jwk_hmac_key(jwk).unwrap();

        let token = "ASk3EMAJGCGQYt0Z6tTslBWDulxqCBiUi8A7W8BwQ32tfRdHxTkNIQrV6iNqCHvltNTlJlUgOgmT2qbdIi_icPxASQAAAAAAAG14YzovL2V4YW1wbGUub3JnL2FiYzEyMw";

        let (mxc, expiry) = super::verify_media_token(&secret, token).unwrap();

        assert_eq!(mxc, mxc_uri!("mxc://example.org/abc123"));
        assert_eq!(expiry.whole_milliseconds(), 50);
    }
}
