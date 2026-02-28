#[derive(serde::Serialize)]
pub struct Context<'a> {
    pub id: u64,
    pub title: &'a str,
    pub tags: &'a [Tag<'a>],
    pub author: Author<'a>,
    pub triggers: &'a [&'a str],
}

#[derive(serde::Serialize)]
pub struct Tag<'a> {
    pub original: &'a str,
    pub translated: Option<&'a str>,
}

#[derive(serde::Serialize)]
pub struct Author<'a> {
    pub id: u64,
    pub name: &'a str,
}

pub mod text {
    use super::Context;
    use minijinja::Environment;

    pub fn format<'a>(env: &Environment, context: &'a Context<'a>) -> anyhow::Result<String> {
        let text = env.get_template("[fuuka-bot]/templates/pixiv/illust.txt")?;
        Ok(text.render(context)?)
    }

    pub fn default() -> &'static str {
        include_str!("illust.text.jinja")
    }
}

pub mod html {
    use super::Context;
    use minijinja::Environment;

    pub fn format<'a>(env: &Environment, context: &'a Context<'a>) -> anyhow::Result<String> {
        let text = env.get_template("[fuuka-bot]/templates/pixiv/illust.html")?;
        Ok(text.render(context)?)
    }

    pub fn default() -> &'static str {
        include_str!("illust.html.jinja")
    }
}

#[cfg(test)]
mod tests {
    use super::{Author, Context, Tag};
    use minijinja::Environment;
    use std::sync::LazyLock;

    static ENVIRONMENT: LazyLock<Environment> = LazyLock::new(|| {
        let mut env = Environment::new();
        env.add_template(
            "[fuuka-bot]/templates/pixiv/illust.txt",
            super::text::default(),
        )
        .unwrap();
        env.add_template(
            "[fuuka-bot]/templates/pixiv/illust.html",
            super::html::default(),
        )
        .unwrap();
        env
    });

    #[test]
    fn format_html_test() {
        use html_compare_rs::assert_html_eq;

        let context: Context<'static> = Context {
            id: 132235564,
            title: "新衣装ホタルちゃん",
            tags: &[
                Tag {
                    original: "ホタル(スターレイル)",
                    translated: Some("流萤（星穹铁道）"),
                },
                Tag {
                    original: "崩壊スターレイル",
                    translated: Some("崩坏：星穹铁道"),
                },
                Tag {
                    original: "Firefly",
                    translated: None,
                },
                Tag {
                    original: "HonkaiStarRail",
                    translated: None,
                },
                Tag {
                    original: "崩壊:スターレイル",
                    translated: Some("Honkai: Star Rail"),
                },
                Tag {
                    original: "女の子",
                    translated: Some("女孩子"),
                },
                Tag {
                    original: "pixivGlowEffect",
                    translated: None,
                },
                Tag {
                    original: "ホタル「春の贈り物」",
                    translated: Some("流萤「春日手信」"),
                },
                Tag {
                    original: "セーラー服",
                    translated: Some("水手服"),
                },
                Tag {
                    original: "崩壊:スターレイル500users入り",
                    translated: Some("崩坏：星穹铁道500收藏"),
                },
            ],
            author: Author {
                id: 78951133,
                name: "どどうさこ",
            },
            triggers: &[],
        };

        let result = super::html::format(&ENVIRONMENT, &context).unwrap();

        assert_html_eq!(
            result,
            concat!(
                "<p>",
                "<a href=\"https://www.pixiv.net/artworks/132235564\">新衣装ホタルちゃん</a>",
                " | ",
                "<a href=\"https://www.pixiv.net/u/78951133\">@どどうさこ</a>",
                "</p>\n",
                "<p>",
                "<font color=\"#3771bb\">#ホタル(スターレイル)</font> (流萤（星穹铁道）) ",
                "<font color=\"#3771bb\">#崩壊スターレイル</font> (崩坏：星穹铁道) ",
                "<font color=\"#3771bb\">#Firefly</font> ",
                "<font color=\"#3771bb\">#HonkaiStarRail</font> ",
                "<font color=\"#3771bb\">#崩壊:スターレイル</font> (Honkai: Star Rail) ",
                "<font color=\"#3771bb\">#女の子</font> (女孩子) ",
                "<font color=\"#3771bb\">#pixivGlowEffect</font>",
                "<font color=\"#3771bb\">#ホタル「春の贈り物」</font> (流萤「春日手信」) ",
                "<font color=\"#3771bb\">#セーラー服</font> (水手服) ",
                "<font color=\"#3771bb\">#崩壊:スターレイル500users入り</font> (崩坏：星穹铁道500收藏)",
                "</p>",
            )
        );
    }

    #[test]
    fn format_text_test() {
        use pretty_assertions::assert_str_eq;

        let context: Context<'static> = Context {
            id: 132235564,
            title: "新衣装ホタルちゃん",
            tags: &[
                Tag {
                    original: "ホタル(スターレイル)",
                    translated: Some("流萤（星穹铁道）"),
                },
                Tag {
                    original: "崩壊スターレイル",
                    translated: Some("崩坏：星穹铁道"),
                },
                Tag {
                    original: "Firefly",
                    translated: None,
                },
                Tag {
                    original: "HonkaiStarRail",
                    translated: None,
                },
                Tag {
                    original: "崩壊:スターレイル",
                    translated: Some("Honkai: Star Rail"),
                },
                Tag {
                    original: "女の子",
                    translated: Some("女孩子"),
                },
                Tag {
                    original: "pixivGlowEffect",
                    translated: None,
                },
                Tag {
                    original: "ホタル「春の贈り物」",
                    translated: Some("流萤「春日手信」"),
                },
                Tag {
                    original: "セーラー服",
                    translated: Some("水手服"),
                },
                Tag {
                    original: "崩壊:スターレイル500users入り",
                    translated: Some("崩坏：星穹铁道500收藏"),
                },
            ],
            author: Author {
                id: 78951133,
                name: "どどうさこ",
            },
            triggers: &[],
        };

        let result = super::text::format(&ENVIRONMENT, &context).unwrap();

        assert_str_eq!(
            result,
            concat!(
                "新衣装ホタルちゃん https://www.pixiv.net/artworks/132235564",
                " | ",
                "@どどうさこ https://www.pixiv.net/u/78951133",
                "\n",
                "#ホタル(スターレイル) (流萤（星穹铁道）) ",
                "#崩壊スターレイル (崩坏：星穹铁道) ",
                "#Firefly ",
                "#HonkaiStarRail ",
                "#崩壊:スターレイル (Honkai: Star Rail) ",
                "#女の子 (女孩子) ",
                "#pixivGlowEffect ",
                "#ホタル「春の贈り物」 (流萤「春日手信」) ",
                "#セーラー服 (水手服) ",
                "#崩壊:スターレイル500users入り (崩坏：星穹铁道500收藏)",
            )
        );
    }

    #[test]
    fn format_html_test_with_triggers() {
        use html_compare_rs::assert_html_eq;

        let context: Context<'static> = Context {
            id: 132235564,
            title: "新衣装ホタルちゃん",
            tags: &[
                Tag {
                    original: "ホタル(スターレイル)",
                    translated: Some("流萤（星穹铁道）"),
                },
                Tag {
                    original: "崩壊スターレイル",
                    translated: Some("崩坏：星穹铁道"),
                },
                Tag {
                    original: "Firefly",
                    translated: None,
                },
                Tag {
                    original: "HonkaiStarRail",
                    translated: None,
                },
                Tag {
                    original: "崩壊:スターレイル",
                    translated: Some("Honkai: Star Rail"),
                },
                Tag {
                    original: "女の子",
                    translated: Some("女孩子"),
                },
                Tag {
                    original: "pixivGlowEffect",
                    translated: None,
                },
                Tag {
                    original: "ホタル「春の贈り物」",
                    translated: Some("流萤「春日手信」"),
                },
                Tag {
                    original: "セーラー服",
                    translated: Some("水手服"),
                },
                Tag {
                    original: "崩壊:スターレイル500users入り",
                    translated: Some("崩坏：星穹铁道500收藏"),
                },
            ],
            author: Author {
                id: 78951133,
                name: "どどうさこ",
            },
            triggers: &["流萤", "星核猎手"],
        };

        let result = super::html::format(&ENVIRONMENT, &context).unwrap();

        assert_html_eq!(
            result,
            concat!(
                "<p>",
                "<a href=\"https://www.pixiv.net/artworks/132235564\">新衣装ホタルちゃん</a>",
                " | ",
                "<a href=\"https://www.pixiv.net/u/78951133\">@どどうさこ</a>",
                "</p>\n",
                "<p>",
                "<font color=\"#3771bb\">#ホタル(スターレイル)</font> (流萤（星穹铁道）) ",
                "<font color=\"#3771bb\">#崩壊スターレイル</font> (崩坏：星穹铁道) ",
                "<font color=\"#3771bb\">#Firefly</font> ",
                "<font color=\"#3771bb\">#HonkaiStarRail</font> ",
                "<font color=\"#3771bb\">#崩壊:スターレイル</font> (Honkai: Star Rail) ",
                "<font color=\"#3771bb\">#女の子</font> (女孩子) ",
                "<font color=\"#3771bb\">#pixivGlowEffect</font>",
                "<font color=\"#3771bb\">#ホタル「春の贈り物」</font> (流萤「春日手信」) ",
                "<font color=\"#3771bb\">#セーラー服</font> (水手服) ",
                "<font color=\"#3771bb\">#崩壊:スターレイル500users入り</font> (崩坏：星穹铁道500收藏)",
                "</p>\n",
                "<p>",
                "<font color=\"#d72b6d\"><b>#流萤诱捕器</b></font> ",
                "<font color=\"#d72b6d\"><b>#星核猎手诱捕器</b></font>",
            )
        );
    }

    #[test]
    fn format_text_test_with_triggers() {
        use pretty_assertions::assert_str_eq;

        let context: Context<'static> = Context {
            id: 132235564,
            title: "新衣装ホタルちゃん",
            tags: &[
                Tag {
                    original: "ホタル(スターレイル)",
                    translated: Some("流萤（星穹铁道）"),
                },
                Tag {
                    original: "崩壊スターレイル",
                    translated: Some("崩坏：星穹铁道"),
                },
                Tag {
                    original: "Firefly",
                    translated: None,
                },
                Tag {
                    original: "HonkaiStarRail",
                    translated: None,
                },
                Tag {
                    original: "崩壊:スターレイル",
                    translated: Some("Honkai: Star Rail"),
                },
                Tag {
                    original: "女の子",
                    translated: Some("女孩子"),
                },
                Tag {
                    original: "pixivGlowEffect",
                    translated: None,
                },
                Tag {
                    original: "ホタル「春の贈り物」",
                    translated: Some("流萤「春日手信」"),
                },
                Tag {
                    original: "セーラー服",
                    translated: Some("水手服"),
                },
                Tag {
                    original: "崩壊:スターレイル500users入り",
                    translated: Some("崩坏：星穹铁道500收藏"),
                },
            ],
            author: Author {
                id: 78951133,
                name: "どどうさこ",
            },
            triggers: &["流萤", "星核猎手"],
        };

        let result = super::text::format(&ENVIRONMENT, &context).unwrap();

        assert_str_eq!(
            result,
            concat!(
                "新衣装ホタルちゃん https://www.pixiv.net/artworks/132235564",
                " | ",
                "@どどうさこ https://www.pixiv.net/u/78951133",
                "\n",
                "#ホタル(スターレイル) (流萤（星穹铁道）) ",
                "#崩壊スターレイル (崩坏：星穹铁道) ",
                "#Firefly ",
                "#HonkaiStarRail ",
                "#崩壊:スターレイル (Honkai: Star Rail) ",
                "#女の子 (女孩子) ",
                "#pixivGlowEffect ",
                "#ホタル「春の贈り物」 (流萤「春日手信」) ",
                "#セーラー服 (水手服) ",
                "#崩壊:スターレイル500users入り (崩坏：星穹铁道500收藏)",
                "\n",
                "#流萤诱捕器 #星核猎手诱捕器"
            )
        );
    }
}
