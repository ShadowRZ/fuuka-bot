use std::{env::VarError, path::PathBuf};

use matrix_sdk::authentication::matrix::MatrixSession;

mod inner {
    use std::{
        env::VarError,
        path::{Path, PathBuf},
    };

    pub(super) fn config<P: AsRef<Path>>(path: P) -> Result<PathBuf, VarError> {
        static ENV_FUUKA_BOT_CONFIGURATION_DIRECTORY: &str = "FUUKA_BOT_CONFIGURATION_DIRECTORY";
        static ENV_CONFIGURATION_DIRECTORY: &str = "CONFIGURATION_DIRECTORY";

        match std::env::var(ENV_FUUKA_BOT_CONFIGURATION_DIRECTORY).map(PathBuf::from) {
            Ok(mut dir) => {
                dir.push(path);
                Ok(dir)
            }
            Err(VarError::NotPresent) => {
                match std::env::var(ENV_CONFIGURATION_DIRECTORY).map(PathBuf::from) {
                    Ok(mut dir) => {
                        dir.push(path);
                        Ok(dir)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub(super) fn state<P: AsRef<Path>>(path: P) -> Result<PathBuf, VarError> {
        static ENV_FUUKA_BOT_STATE_DIRECTORY: &str = "FUUKA_BOT_STATE_DIRECTORY";
        static ENV_STATE_DIRECTORY: &str = "STATE_DIRECTORY";

        match std::env::var(ENV_FUUKA_BOT_STATE_DIRECTORY).map(PathBuf::from) {
            Ok(mut dir) => {
                dir.push(path);
                Ok(dir)
            }
            Err(VarError::NotPresent) => {
                match std::env::var(ENV_STATE_DIRECTORY).map(PathBuf::from) {
                    Ok(mut dir) => {
                        dir.push(path);
                        Ok(dir)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

static CREDENTIALS_FILE: &str = "credentials.json";
static CONFIG_FILE: &str = "fuuka-bot.toml";
static JWK_TOKEN_FILE: &str = "fuuka-bot.jwk.json";

pub(super) fn config() -> anyhow::Result<crate::Config> {
    let file = self::inner::config(CONFIG_FILE)?;

    let contents = std::fs::read_to_string(file)?;
    let config = toml::from_str::<crate::Config>(&contents)?;
    Ok(config)
}

pub(super) fn store() -> Result<PathBuf, VarError> {
    self::inner::state("store")
}

pub(super) fn jwk_token() -> anyhow::Result<jose_jwk::Jwk> {
    let file = self::inner::config(JWK_TOKEN_FILE)?;

    let contents = std::fs::read_to_string(file)?;
    let jwk = serde_json::from_str::<jose_jwk::Jwk>(&contents)?;
    Ok(jwk)
}

pub(super) fn credentials() -> anyhow::Result<MatrixSession> {
    let file = self::inner::config(CREDENTIALS_FILE)?;

    let contents = std::fs::read_to_string(file)?;
    let session = serde_json::from_str::<MatrixSession>(&contents)?;
    Ok(session)
}
