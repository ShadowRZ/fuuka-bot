//! Quote images.

use std::process::Stdio;

use tokio::{io::AsyncWriteExt, process::Command};

/// Create a quote image.
pub async fn quote(avatar: Option<String>, text: &str) -> anyhow::Result<Vec<u8>> {
    let mut cmd = Command::new("bash");
    let cmd = cmd
        .arg("-s")
        .arg("-")
        .arg(avatar.unwrap_or_default())
        .arg(text);
    cmd.stdout(Stdio::piped());
    cmd.stdin(Stdio::piped());
    let mut child = cmd.spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or(anyhow::anyhow!("Failed to get stdin!"))?;

    if let Err(e) = stdin.write(include_bytes!("scripts/mkquote.sh")).await {
        child.kill().await?;
        return Err(e.into());
    }

    drop(stdin);

    let out = child.wait_with_output().await?;
    if !out.status.success() {
        anyhow::bail!("Command failed!");
    }
    Ok(out.stdout)
}

/// Convert HTML to Pango markup.
pub fn html2pango(input: &str) -> anyhow::Result<String> {
    use lol_html::{element, rewrite_str, RewriteStrSettings};
    let element_content_handlers = vec![
        element!("a", |el| {
            el.set_tag_name("span")?;
            if el.has_attribute("href") {
                el.set_attribute("underline", "single")?;
                el.set_attribute("underline_color", "blue")?;
            }

            Ok(())
        }),
        element!("strong", |el| {
            el.set_tag_name("b")?;
            Ok(())
        }),
        element!("del", |el| {
            el.set_tag_name("s")?;
            Ok(())
        }),
        element!("blockquote", |el| {
            el.set_tag_name("span")?;
            el.set_attribute("background", "#D3D3D380")?;
            Ok(())
        }),
        element!("font", |el| {
            el.set_tag_name("span")?;
            if let Some(color) = el.get_attribute("color") {
                el.set_attribute("color", &color)?;
            }
            Ok(())
        }),
        element!("span", |el| {
            if el.has_attribute("data-mx-spoiler") {
                el.set_attribute("color", "black")?;
                el.set_attribute("background", "black")?;
            }
            Ok(())
        }),
        element!("br", |el| {
            el.remove();
            Ok(())
        }),
        element!("hr", |el| {
            el.remove();
            Ok(())
        }),
    ];
    let result = rewrite_str(
        input,
        RewriteStrSettings {
            element_content_handlers,
            ..RewriteStrSettings::default()
        },
    )?;

    Ok(result)
}
