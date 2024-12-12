use matrix_sdk::ruma::OwnedUserId;
use ruma::UserId;

/// A type of events.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum RequestType {
    Profile {
        category: self::profile::Category,
        response_type: self::profile::ResponseType,
    },
    Ping,
    Divergence,
    Hitokoto,
    Sticker,
    Ignore,
    Unignore(OwnedUserId),
    Pixiv(self::pixiv::PixivCommand),
    Nixpkgs {
        pr_number: i32,
        track: bool,
    },
    Info,
    Help,
}

pub(super) fn from_args(
    mut args: impl Iterator<Item = String>,
) -> anyhow::Result<Option<RequestType>> {
    let Some(command) = args.next() else {
        return Ok(None);
    };

    match command.as_str() {
        "profile" => Ok(
            self::profile::from_args(args)?.map(|(category, response_type)| RequestType::Profile {
                category,
                response_type,
            }),
        ),
        "ping" => Ok(Some(RequestType::Ping)),
        "divergence" => Ok(Some(RequestType::Divergence)),
        "hitokoto" => Ok(Some(RequestType::Hitokoto)),
        "sticker" => Ok(Some(RequestType::Sticker)),
        "ignore" => Ok(Some(RequestType::Ignore)),
        "unignore" => Ok(Some(RequestType::Unignore(UserId::parse(
            args.next()
                .ok_or(crate::Error::MissingArgument("user_id"))?,
        )?))),
        "pixiv" => Ok(Some(RequestType::Pixiv(self::pixiv::from_args(args)?))),
        "nixpkgs" => self::nixpkgs::from_args(args)
            .map(|(pr_number, track)| Some(RequestType::Nixpkgs { pr_number, track })),
        "info" => Ok(Some(RequestType::Info)),
        "help" => Ok(Some(RequestType::Help)),
        _ => Result::Err(crate::Error::UnknownCommand(command).into()),
    }
}

pub mod profile {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
    pub enum Category {
        Name,
        Avatar,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
    pub enum ResponseType {
        Current,
        History,
    }

    impl ResponseType {
        fn from_args(mut args: impl Iterator<Item = String>) -> anyhow::Result<Self> {
            let Some(command) = args.next() else {
                return Ok(Self::Current);
            };

            match command.as_str() {
                "history" => Ok(Self::History),
                _ => Result::Err(crate::Error::UnknownCommand(command).into()),
            }
        }
    }

    pub(super) fn from_args(
        mut args: impl Iterator<Item = String>,
    ) -> anyhow::Result<Option<(Category, ResponseType)>> {
        let Some(command) = args.next() else {
            return Ok(None);
        };

        match command.as_str() {
            "name" => Ok(Some((Category::Name, ResponseType::from_args(args)?))),
            "avatar" => Ok(Some((Category::Avatar, ResponseType::from_args(args)?))),
            _ => Result::Err(crate::Error::UnknownCommand(command).into()),
        }
    }
}

pub mod pixiv {
    use std::str::FromStr;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
    pub enum PixivCommand {
        Ranking,
        IllustInfo(i32),
    }

    pub(super) fn from_args(
        mut args: impl Iterator<Item = String>,
    ) -> anyhow::Result<PixivCommand> {
        Ok(match args.next() {
            Some(id) => PixivCommand::IllustInfo(<i32 as FromStr>::from_str(&id)?),
            None => PixivCommand::Ranking,
        })
    }
}

pub mod nixpkgs {
    pub(super) fn from_args(mut args: impl Iterator<Item = String>) -> anyhow::Result<(i32, bool)> {
        let pr_number: i32 = args
            .next()
            .ok_or(crate::Error::MissingArgument("pr_number"))?
            .parse()?;
        let track = args
            .next()
            .map(|arg| &arg == "track")
            .unwrap_or_default();

        Ok((pr_number, track))
    }
}
