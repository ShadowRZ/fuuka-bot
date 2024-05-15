use url::{Host, Url};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LinkType {
    Crates(CrateLinkType),
    Pixiv(PixivLinkType),
    Generic,
    CannotBeABase,
}

impl LinkType {
    fn parse_crates_io(url: &Url) -> LinkType {
        let Some(mut paths) = url.path_segments() else {
            return LinkType::CannotBeABase;
        };

        if paths.next() != Some("crates") {
            return LinkType::Generic;
        }

        let name = paths.next().map(ToString::to_string);
        let version = paths.next().map(ToString::to_string);

        match name {
            Some(name) => LinkType::Crates(CrateLinkType::CrateInfo { name, version }),
            None => LinkType::Generic,
        }
    }

    fn parse_pixiv(url: &Url) -> Result<LinkType, crate::Error> {
        let Some(mut paths) = url.path_segments() else {
            return Ok(LinkType::CannotBeABase);
        };

        let p01 = paths.next();

        if !p01
            .map(|p01| ["artworks", "i"].into_iter().any(|allowed| allowed == p01))
            .unwrap_or_default()
        {
            return Ok(LinkType::Generic);
        }

        let artwork_id = paths
            .next()
            .map(|i| i.parse::<i32>())
            .transpose()
            .map_err(|e| crate::Error::InvaildArgument {
                arg: "Artwork ID",
                source: e.into(),
            })?;

        match artwork_id {
            Some(artwork_id) => Ok(LinkType::Pixiv(PixivLinkType::Artwork(artwork_id))),
            None => Ok(LinkType::Generic),
        }
    }
}

impl TryFrom<Url> for LinkType {
    type Error = crate::Error;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        if value.cannot_be_a_base() {
            Ok(LinkType::CannotBeABase)
        } else {
            match value.host() {
                Some(Host::Domain("crates.io")) => Ok(Self::parse_crates_io(&value)),
                Some(Host::Domain("www.pixiv.net")) => Self::parse_pixiv(&value),
                Some(Host::Domain("pixiv.net")) => Self::parse_pixiv(&value),
                _ => Ok(LinkType::Generic),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CrateLinkType {
    CrateInfo {
        name: String,
        version: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PixivLinkType {
    Artwork(i32),
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use url::Url;

    use crate::message::nahida::link_type::{CrateLinkType, PixivLinkType};

    use super::LinkType;

    #[test]
    fn parse_crates_io_ok() {
        let url = Url::parse("https://crates.io/crates/syn").unwrap();
        let result: LinkType = url.try_into().unwrap();
        let expected = LinkType::Crates(CrateLinkType::CrateInfo {
            name: "syn".to_string(),
            version: None,
        });

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_crates_io_with_version_ok() {
        let url = Url::parse("https://crates.io/crates/syn/1").unwrap();
        let result: LinkType = url.try_into().unwrap();
        let expected = LinkType::Crates(CrateLinkType::CrateInfo {
            name: "syn".to_string(),
            version: Some("1".to_string()),
        });

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_pixiv_illust_ok() {
        let url = Url::parse("https://www.pixiv.net/artworks/73396560").unwrap();
        let result: LinkType = url.try_into().unwrap();
        let expected = LinkType::Pixiv(PixivLinkType::Artwork(73396560));

        assert_eq!(expected, result);
    }
}
