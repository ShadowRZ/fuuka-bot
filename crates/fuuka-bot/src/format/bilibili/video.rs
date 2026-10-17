#[derive(serde::Serialize)]
pub struct Context<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub tags: &'a [&'a str],
    pub author: Author<'a>,
    pub counts: Counts,
}

#[derive(serde::Serialize)]
pub struct Author<'a> {
    pub id: u64,
    pub name: &'a str,
}

#[derive(serde::Serialize)]
pub struct Counts {
    pub view: u64,
    pub like: u64,
    pub coin: u64,
    pub favorite: u64,
    pub danmaku: u64,
    pub reply: u64,
    pub share: u64,
}
pub mod text {
    use super::Context;
    use minijinja::{Environment, value::Serde};

    pub fn format<'a>(env: &Environment, context: &'a Context<'a>) -> anyhow::Result<String> {
        let text = env.get_template("[fuuka-bot]/templates/bilibili/video.txt")?;
        Ok(text.render(Serde(context))?)
    }

    pub fn default() -> &'static str {
        include_str!("video.text.jinja")
    }
}

pub mod html {
    use super::Context;
    use minijinja::{Environment, value::Serde};

    pub fn format<'a>(env: &Environment, context: &'a Context<'a>) -> anyhow::Result<String> {
        let text = env.get_template("[fuuka-bot]/templates/bilibili/video.html")?;
        Ok(text.render(Serde(context))?)
    }

    pub fn default() -> &'static str {
        include_str!("video.html.jinja")
    }
}

#[cfg(test)]
mod tests {
    use super::{Author, Context, Counts};

    use crate::format::ENVIRONMENT;

    #[test]
    fn format_html_test() {
        use html_compare_rs::assert_html_eq;

        let context: Context<'static> = Context {
            id: "BV1GJ411x7h7",
            title: "【官方 MV】Never Gonna Give You Up - Rick Astley",
            description: None,
            tags: &[
                "Never Gonna Give You Up",
                "Rick Astley",
                "欧美MV",
                "流行音乐",
                "欧美音乐",
                "MV",
            ],
            author: Author {
                id: 486906719,
                name: "索尼音乐中国",
            },
            counts: Counts {
                view: 94702144,
                danmaku: 132903,
                reply: 176755,
                favorite: 1346712,
                coin: 1114961,
                share: 420758,
                like: 2573660,
            },
        };

        let result = super::html::format(&ENVIRONMENT, &context).unwrap();

        assert_html_eq!(
            result,
            concat!(
                "<p>",
                "<a href=\"https://www.bilibili.com/video/BV1GJ411x7h7\">【官方 MV】Never Gonna Give You Up - Rick Astley</a>",
                " | ",
                "<a href=\"https://space.bilibili.com/486906719\">@索尼音乐中国</a>",
                "</p>\n",
                "<p>",
                "▶️ 94702144 · 👍 2573660 · 🪙 1114961 · 🌟 1346712 · 🪧 132903 · 💬 176755 · ↗️ 420758",
                "</p>\n",
                "<p>",
                "<font color=\"#3771bb\">#Never Gonna Give You Up#</font> ",
                "<font color=\"#3771bb\">#Rick Astley#</font> ",
                "<font color=\"#3771bb\">#欧美MV#</font> ",
                "<font color=\"#3771bb\">#流行音乐#</font> ",
                "<font color=\"#3771bb\">#欧美音乐#</font> ",
                "<font color=\"#3771bb\">#MV#</font>",
                "</p>",
            )
        );
    }

    #[test]
    fn format_text_test() {
        use pretty_assertions::assert_str_eq;

        let context: Context<'static> = Context {
            id: "BV1GJ411x7h7",
            title: "【官方 MV】Never Gonna Give You Up - Rick Astley",
            description: None,
            tags: &[
                "Never Gonna Give You Up",
                "Rick Astley",
                "欧美MV",
                "流行音乐",
                "欧美音乐",
                "MV",
            ],
            author: Author {
                id: 486906719,
                name: "索尼音乐中国",
            },
            counts: Counts {
                view: 94702144,
                danmaku: 132903,
                reply: 176755,
                favorite: 1346712,
                coin: 1114961,
                share: 420758,
                like: 2573660,
            },
        };

        let result = super::text::format(&ENVIRONMENT, &context).unwrap();

        assert_str_eq!(
            result,
            concat!(
                "【官方 MV】Never Gonna Give You Up - Rick Astley https://www.bilibili.com/video/BV1GJ411x7h7",
                " | ",
                "@索尼音乐中国 https://space.bilibili.com/486906719",
                "\n",
                "▶️ 94702144 · 👍 2573660 · 🪙 1114961 · 🌟 1346712 · 🪧 132903 · 💬 176755 · ↗️ 420758",
                "\n",
                "#Never Gonna Give You Up# ",
                "#Rick Astley# ",
                "#欧美MV# ",
                "#流行音乐# ",
                "#欧美音乐# ",
                "#MV#",
            )
        );
    }

    #[test]
    fn format_html_test_with_description() {
        use html_compare_rs::assert_html_eq;

        let context: Context<'static> = Context {
            id: "BV1o44y1v7Bx",
            title: "厨 房 好 搭 档",
            description: Some(
                "第一次做这种台词比较多的鬼畜，做的不是很好，希望喜欢这个视频的小伙伴可以给个三连支持一下！！！",
            ),
            tags: &[
                "鬼畜星探企划第三期",
                "鬼畜",
                "鬼畜调教",
                "特效",
                "搞笑",
                "沙雕",
                "胡闹厨房",
                "青莲地心火",
                "沙雕广告",
                "鬼畜剧场",
            ],
            author: Author {
                id: 341243751,
                name: "To-Go玩家阳",
            },
            counts: Counts {
                view: 1518364,
                danmaku: 1416,
                reply: 659,
                favorite: 29925,
                coin: 12573,
                share: 7444,
                like: 75281,
            },
        };

        let result = super::html::format(&ENVIRONMENT, &context).unwrap();

        assert_html_eq!(
            result,
            concat!(
                "<p>",
                "<a href=\"https://www.bilibili.com/video/BV1o44y1v7Bx\">厨 房 好 搭 档</a>",
                " | ",
                "<a href=\"https://space.bilibili.com/341243751\">@To-Go玩家阳</a>",
                "</p>\n",
                "<p>",
                "▶️ 1518364 · 👍 75281 · 🪙 12573 · 🌟 29925 · 🪧 1416 · 💬 659 · ↗️ 7444",
                "</p>\n",
                "<details><summary>Description</summary><blockquote>第一次做这种台词比较多的鬼畜，做的不是很好，希望喜欢这个视频的小伙伴可以给个三连支持一下！！！</blockquote></details>",
                "<p>",
                "<font color=\"#3771bb\">#鬼畜星探企划第三期#</font> ",
                "<font color=\"#3771bb\">#鬼畜#</font> ",
                "<font color=\"#3771bb\">#鬼畜调教#</font> ",
                "<font color=\"#3771bb\">#特效#</font> ",
                "<font color=\"#3771bb\">#搞笑#</font> ",
                "<font color=\"#3771bb\">#沙雕#</font> ",
                "<font color=\"#3771bb\">#胡闹厨房#</font> ",
                "<font color=\"#3771bb\">#青莲地心火#</font> ",
                "<font color=\"#3771bb\">#沙雕广告#</font> ",
                "<font color=\"#3771bb\">#鬼畜剧场#</font>",
                "</p>",
            )
        );
    }

    #[test]
    fn format_text_test_with_description() {
        use pretty_assertions::assert_str_eq;

        let context: Context<'static> = Context {
            id: "BV1o44y1v7Bx",
            title: "厨 房 好 搭 档",
            description: Some(
                "第一次做这种台词比较多的鬼畜，做的不是很好，希望喜欢这个视频的小伙伴可以给个三连支持一下！！！",
            ),
            tags: &[
                "鬼畜星探企划第三期",
                "鬼畜",
                "鬼畜调教",
                "特效",
                "搞笑",
                "沙雕",
                "胡闹厨房",
                "青莲地心火",
                "沙雕广告",
                "鬼畜剧场",
            ],
            author: Author {
                id: 341243751,
                name: "To-Go玩家阳",
            },
            counts: Counts {
                view: 1518364,
                danmaku: 1416,
                reply: 659,
                favorite: 29925,
                coin: 12573,
                share: 7444,
                like: 75281,
            },
        };

        let result = super::text::format(&ENVIRONMENT, &context).unwrap();

        assert_str_eq!(
            result,
            concat!(
                "厨 房 好 搭 档 https://www.bilibili.com/video/BV1o44y1v7Bx",
                " | ",
                "@To-Go玩家阳 https://space.bilibili.com/341243751",
                "\n",
                "▶️ 1518364 · 👍 75281 · 🪙 12573 · 🌟 29925 · 🪧 1416 · 💬 659 · ↗️ 7444",
                "\n",
                "#鬼畜星探企划第三期# ",
                "#鬼畜# ",
                "#鬼畜调教# ",
                "#特效# ",
                "#搞笑# ",
                "#沙雕# ",
                "#胡闹厨房# ",
                "#青莲地心火# ",
                "#沙雕广告# ",
                "#鬼畜剧场#",
                "\n",
                "> 第一次做这种台词比较多的鬼畜，做的不是很好，希望喜欢这个视频的小伙伴可以给个三连支持一下！！！"
            )
        );
    }

    #[test]
    fn format_html_test_with_multiline_description() {
        use html_compare_rs::assert_html_eq;

        let context: Context<'static> = Context {
            id: "BV13yJ1zUEmH",
            title: "魔女审判混进了奇怪的人",
            description: Some(
                "粉色小奶狗是对的！！！\n太可爱了艾呀玛...\n\n\n咱上大学了，要苦逼上早晚八\n所以之后更新就随缘喽\n（不过本来好像就是随缘）",
            ),
            tags: &[
                "魔法少女的魔女审判",
                "逆转裁判",
                "弹丸论破",
                "简易长矛",
                "樱羽艾玛",
                "粉色小奶狗是对的！！！",
                "电棍",
                "碧蓝档案",
                "边狱巴士",
                "丰川祥子",
            ],
            author: Author {
                id: 29484733,
                name: "Chaos-GofG",
            },
            counts: Counts {
                view: 221938,
                danmaku: 1196,
                reply: 690,
                favorite: 6169,
                coin: 2320,
                share: 5748,
                like: 11831,
            },
        };

        let result = super::html::format(&ENVIRONMENT, &context).unwrap();

        assert_html_eq!(
            result,
            concat!(
                "<p>",
                "<a href=\"https://www.bilibili.com/video/BV13yJ1zUEmH\">魔女审判混进了奇怪的人</a>",
                " | ",
                "<a href=\"https://space.bilibili.com/29484733\">@Chaos-GofG</a>",
                "</p>\n",
                "<p>",
                "▶️ 221938 · 👍 11831 · 🪙 2320 · 🌟 6169 · 🪧 1196 · 💬 690 · ↗️ 5748",
                "</p>\n",
                "<details><summary>Description</summary><blockquote>粉色小奶狗是对的！！！<br/>太可爱了艾呀玛...<br/><br/><br/>咱上大学了，要苦逼上早晚八<br/>所以之后更新就随缘喽<br/>（不过本来好像就是随缘）</blockquote></details>",
                "<p>",
                "<font color=\"#3771bb\">#魔法少女的魔女审判#</font> ",
                "<font color=\"#3771bb\">#逆转裁判#</font> ",
                "<font color=\"#3771bb\">#弹丸论破#</font> ",
                "<font color=\"#3771bb\">#简易长矛#</font> ",
                "<font color=\"#3771bb\">#樱羽艾玛#</font> ",
                "<font color=\"#3771bb\">#粉色小奶狗是对的！！！#</font> ",
                "<font color=\"#3771bb\">#电棍#</font> ",
                "<font color=\"#3771bb\">#碧蓝档案#</font> ",
                "<font color=\"#3771bb\">#边狱巴士#</font> ",
                "<font color=\"#3771bb\">#丰川祥子#</font>",
                "</p>",
            )
        );
    }

    #[test]
    fn format_text_test_with_multiline_description() {
        use pretty_assertions::assert_str_eq;

        let context: Context<'static> = Context {
            id: "BV13yJ1zUEmH",
            title: "魔女审判混进了奇怪的人",
            description: Some(
                "粉色小奶狗是对的！！！\n太可爱了艾呀玛...\n\n\n咱上大学了，要苦逼上早晚八\n所以之后更新就随缘喽\n（不过本来好像就是随缘）",
            ),
            tags: &[
                "魔法少女的魔女审判",
                "逆转裁判",
                "弹丸论破",
                "简易长矛",
                "樱羽艾玛",
                "粉色小奶狗是对的！！！",
                "电棍",
                "碧蓝档案",
                "边狱巴士",
                "丰川祥子",
            ],
            author: Author {
                id: 29484733,
                name: "Chaos-GofG",
            },
            counts: Counts {
                view: 221938,
                danmaku: 1196,
                reply: 690,
                favorite: 6169,
                coin: 2320,
                share: 5748,
                like: 11831,
            },
        };

        let result = super::text::format(&ENVIRONMENT, &context).unwrap();

        assert_str_eq!(
            result,
            concat!(
                "魔女审判混进了奇怪的人 https://www.bilibili.com/video/BV13yJ1zUEmH",
                " | ",
                "@Chaos-GofG https://space.bilibili.com/29484733",
                "\n",
                "▶️ 221938 · 👍 11831 · 🪙 2320 · 🌟 6169 · 🪧 1196 · 💬 690 · ↗️ 5748",
                "\n",
                "#魔法少女的魔女审判# ",
                "#逆转裁判# ",
                "#弹丸论破# ",
                "#简易长矛# ",
                "#樱羽艾玛# ",
                "#粉色小奶狗是对的！！！# ",
                "#电棍# ",
                "#碧蓝档案# ",
                "#边狱巴士# ",
                "#丰川祥子#",
                "\n",
                "> 粉色小奶狗是对的！！！\n> 太可爱了艾呀玛...\n> \n> \n> 咱上大学了，要苦逼上早晚八\n> 所以之后更新就随缘喽\n> （不过本来好像就是随缘）"
            )
        );
    }
}
