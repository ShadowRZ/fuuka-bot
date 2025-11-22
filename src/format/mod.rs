use std::sync::LazyLock;

use minijinja::Environment;

pub mod bilibili;
pub mod pixiv;

// TODO: Use a bot created environment
pub static ENVIRONMENT: LazyLock<Environment> = LazyLock::new(|| {
    let mut env = Environment::new();
    env.add_template(
        "[fuuka-bot]/templates/pixiv/illust.txt",
        self::pixiv::illust::text::default(),
    )
    .unwrap();
    env.add_template(
        "[fuuka-bot]/templates/pixiv/illust.html",
        self::pixiv::illust::html::default(),
    )
    .unwrap();
    env.add_template(
        "[fuuka-bot]/templates/bilibili/video.txt",
        self::bilibili::video::text::default(),
    )
    .unwrap();
    env.add_template(
        "[fuuka-bot]/templates/bilibili/video.html",
        self::bilibili::video::html::default(),
    )
    .unwrap();
    env.add_filter("to_html", self::filter::to_html);
    env.add_filter("quote", self::filter::quote);
    env
});

pub(self) mod filter {
    pub fn to_html(text: &str) -> String {
        text.replace("\n", "<br/>")
    }

    pub fn quote(text: &str) -> String {
        let quoted: Vec<_> = text.lines().map(|s| format!("> {s}")).collect();

        quoted.join("\n")
    }
}
