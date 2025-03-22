use graphql_client::GraphQLQuery;

pub mod nixpkgs_pr;

static GRAPHQL_ENDPOINT: &str = "https://api.github.com/graphql";

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
