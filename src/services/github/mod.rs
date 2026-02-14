use graphql_client::GraphQLQuery;
use octocrab::{AuthState, Octocrab, OctocrabBuilder, service::middleware::base_uri::BaseUriLayer};
use secrecy::SecretString;

pub mod nixpkgs_pr;

static GRAPHQL_ENDPOINT: &str = "https://api.github.com/graphql";

pub fn octocrab(client: &reqwest::Client, token: SecretString) -> Octocrab {
    let service = tower::ServiceBuilder::new()
        .layer(crate::layer::ReqwestLayer)
        .service(client.clone());
    OctocrabBuilder::new_empty()
        .with_service(service)
        .with_layer(&BaseUriLayer::new(http::Uri::from_static(
            "https://api.github.com",
        )))
        .with_auth(AuthState::AccessToken { token })
        .build()
        .unwrap()
}

async fn post_github_graphql<Q: GraphQLQuery>(
    client: &reqwest::Client,
    token: &str,
    vars: Q::Variables,
) -> Result<graphql_client::Response<Q::ResponseData>, reqwest::Error> {
    let body = Q::build_query(vars);
    let reqwest_response = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?;

    reqwest_response.json().await
}

#[derive(Debug)]
pub struct Error(Vec<graphql_client::Error>);

// Based on https://github.com/cli/go-gh/blob/e1048dfe671b9aee9367a5e3e720831f4d64b33a/pkg/api/errors.go#L58-L69
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GraphQL: ")?;
        let Self(error) = self;
        let len = error.len();
        for (i, e) in error.iter().enumerate() {
            write!(f, "{}", e.message)?;
            if let Some(paths) = &e.path {
                write!(f, " (")?;
                for (i, path) in paths.iter().enumerate() {
                    use graphql_client::PathFragment;
                    if i > 0 {
                        write!(f, ".")?;
                    }
                    match path {
                        PathFragment::Key(k) => {
                            write!(f, "{}", k)?;
                        }
                        PathFragment::Index(idx) => {
                            write!(f, "[{}]", idx)?;
                        }
                    }
                }
                write!(f, ")")?;
            }
            if i < len - 1 {
                write!(f, ", ")?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for Error {}

mod tests {
    #[test]
    pub fn test_github_graphql_error_format() {
        use pretty_assertions::assert_eq;

        let error: Vec<graphql_client::Error> = serde_json::from_value(serde_json::json!(
            [
                { "message": "OH NO" },
                { "message": "this is fine" },
            ]
        ))
        .unwrap();

        let error = super::Error(error);

        let result = format!("{error}");

        assert_eq!(result, "GraphQL: OH NO, this is fine");
    }
    #[test]
    pub fn test_github_graphql_error_format_with_path() {
        use pretty_assertions::assert_eq;

        let error: Vec<graphql_client::Error> = serde_json::from_value(serde_json::json!(
            [
                {
                    "message": "OH NO",
                    "path": ["repository", "issue"]
                },
                {
                    "message": "this is fine",
                    "path": ["repository", "issues", 0, "comments"]
                }
            ]
        ))
        .unwrap();

        let error = super::Error(error);

        let result = format!("{error}");

        assert_eq!(
            result,
            "GraphQL: OH NO (repository.issue), this is fine (repository.issues.[0].comments)"
        );
    }
}
