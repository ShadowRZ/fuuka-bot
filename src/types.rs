//! Types for external API.

use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateMetadata {
    #[serde(rename = "crate")]
    pub crate_info: CrateInfo,
    pub versions: Vec<CrateVersion>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateInfo {
    pub description: Option<String>,
    pub name: String,
    pub max_stable_version: String,
    pub max_version: String,
    pub downloads: u32,
    pub documentation: Option<String>,
    pub repository: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateVersion {
    pub license: Option<String>,
    pub num: String,
    pub rust_version: Option<String>,
    pub yanked: bool,
}

/// Error types.
#[derive(Error, Debug)]
pub enum Error {
    /// This command requires replying to an event.
    #[error("Replying to a event is required for this command")]
    RequiresReply,
    /// This command is missing an argument.
    #[error("Missing an argument: {0}")]
    MissingArgument(&'static str),
    /// Invaild argument passed into an argument.
    #[error("Invaild argument passed for {arg}: {source}")]
    InvaildArgument {
        /// The argument that is invaild.
        arg: &'static str,
        #[source]
        /// The source error that caused it to happen.
        source: anyhow::Error,
    },
    /// An unexpected error happened.
    #[error("{0}")]
    UnexpectedError(&'static str),
    /// An unknown command was passed.
    #[error("Unrecognized command {0}")]
    UnknownCommand(String),
    /// A GraphQL error occured.
    #[error("Error response from {service}: {error:?}")]
    GraphQLError {
        service: &'static str,
        error: Vec<graphql_client::Error>,
    },
}
