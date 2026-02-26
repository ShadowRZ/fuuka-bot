//! A generic extractor for any URLs.
//!
//! Returns the page title or Content-Type.

use std::str::FromStr;

use anyhow::Context;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use mime::Mime;
use url::Url;

#[tracing::instrument(name = "generic", skip_all, err)]
pub async fn extract(
    client: &reqwest::Client,
    url: Url,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let resp = client
        .get(url)
        .send()
        .await?
        .error_for_status()
        .context("Server reported failure")?;
    let headers = resp.headers();
    let content_type = headers.get(reqwest::header::CONTENT_TYPE);

    match content_type {
        Some(content_type) => {
            let content_type = Mime::from_str(content_type.to_str()?)?;
            if (content_type.type_(), content_type.subtype()) == (mime::TEXT, mime::HTML) {
                parse_html_title(&resp.text().await?).map(|ok| {
                    ok.map(|str| {
                        RoomMessageEventContent::text_html(
                            format!("[Generic] Page Title: {str}"),
                            format!("<b>[Generic]</b> Page Title: {str}"),
                        )
                    })
                })
            } else {
                Ok(Some(RoomMessageEventContent::text_html(
                    format!("[Generic] Content Type: {content_type}"),
                    format!("<b>[Generic]</b> Content Type: {content_type}"),
                )))
            }
        }
        None => parse_html_title(&resp.text().await?).map(|ok| {
            ok.map(|str| {
                RoomMessageEventContent::text_html(
                    format!("[Generic] Page Title: {str}"),
                    format!("<b>[Generic]</b> Page Title: {str}"),
                )
            })
        }),
    }
}

fn parse_html_title(input: &str) -> anyhow::Result<Option<String>> {
    let dom = tl::parse(input, tl::ParserOptions::default())?;
    let parser = dom.parser();
    let title_element = dom.query_selector("title");
    match title_element {
        Some(mut title_element) => {
            let Some(elem) = title_element.next() else {
                return Ok(None);
            };

            let title = elem.get(parser).map(|node| node.inner_text(parser));
            Ok(title.as_deref().map(ToString::to_string))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_html_title;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_simple_html_title() {
        let str = r#"<html><head><title>Title</title></head></html>"#;
        let res = parse_html_title(str).unwrap();
        let req = Some("Title".to_string());

        assert_eq!(res, req);
    }
}
