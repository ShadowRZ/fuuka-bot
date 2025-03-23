use std::io::Cursor;
use std::sync::{LazyLock, Mutex};

use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache, Weight, Wrap};
use image::imageops::FilterType;
use image::{DynamicImage, ImageReader};
use matrix_sdk::ruma::UInt;
use matrix_sdk::ruma::events::room::message::TextMessageEventContent;
use matrix_sdk::{
    media::{MediaFormat, MediaThumbnailSettings},
    room::RoomMember,
    ruma::media::Method,
};
use tiny_skia::{
    FillRule, IntSize, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Rect, Transform,
};

use crate::RoomMemberExt;

static FONT_SYSTEM: LazyLock<Mutex<FontSystem>> = LazyLock::new(|| Mutex::new(FontSystem::new()));
static SWASH_CACHE: LazyLock<Mutex<SwashCache>> = LazyLock::new(|| Mutex::new(SwashCache::new()));
static MASK: LazyLock<Mask> = LazyLock::new(|| {
    let mut mask = Mask::new(48, 48).unwrap();
    let path = PathBuilder::from_circle(24.0, 24.0, 24.0).unwrap();
    mask.fill_path(&path, FillRule::Winding, true, Transform::identity());

    mask
});

pub async fn render(
    content: TextMessageEventContent,
    room_member: &RoomMember,
) -> anyhow::Result<DynamicImage> {
    let avatar = room_member
        .avatar(MediaFormat::Thumbnail(MediaThumbnailSettings {
            method: Method::Scale,
            width: UInt::from(48u32),
            height: UInt::from(48u32),
            animated: false,
        }))
        .await?;

    let avatar_pixmap = match avatar {
        Some(data) => {
            let mut bg = Pixmap::new(48, 48).unwrap();
            let image = ImageReader::new(Cursor::new(data))
                .with_guessed_format()?
                .decode()?;
            let image = image.resize(48, 48, FilterType::Lanczos3);
            let size = IntSize::from_wh(48, 48).unwrap();
            let data = image.to_rgba8().into_vec();
            let pix = Pixmap::from_vec(data, size).unwrap();
            bg.draw_pixmap(
                0,
                0,
                pix.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                Some(&MASK),
            );

            bg
        }
        None => Pixmap::new(48, 48).unwrap(),
    };

    let pixmap = {
        use cosmic_text::Color;

        let textbox = {
            let metrics = Metrics::new(16.0, 20.0);
            let mut font_system = FONT_SYSTEM.lock().expect("Mutex posioned!");
            let mut swash_cache = SWASH_CACHE.lock().expect("Mutex posioned!");
            let mut buffer = Buffer::new(&mut font_system, metrics);
            let mut buffer = buffer.borrow_with(&mut font_system);
            buffer.set_size(Some(500.0), None);
            buffer.set_wrap(Wrap::WordOrGlyph);

            {
                let name = room_member.name_or_id();
                let body = content.body;
                let attrs = Attrs::new();
                let name_attrs = attrs
                    .color(cosmic_text::Color::rgb(0x1f, 0x47, 0x88))
                    .metrics(Metrics::new(18.0, 20.0))
                    .weight(Weight::BOLD);

                buffer.set_rich_text(
                    [
                        (format!("{name}\n").as_str(), name_attrs),
                        (body.as_str(), attrs),
                    ],
                    attrs,
                    Shaping::Advanced,
                    None,
                );
            }
            buffer.shape_until_scroll(true);

            let mut layout_runs = buffer.layout_runs();

            let size = layout_runs
                .by_ref()
                .map(|layout_run| {
                    (
                        layout_run.line_w.ceil() as u32,
                        layout_run.line_height.ceil() as u32,
                    )
                })
                .reduce(|(acc_w, acc_h), (w, h)| (u32::max(acc_w, w), acc_h + h));

            let (width, height) = size.unzip();
            let mut pixmap = Pixmap::new(width.unwrap_or(20), height.unwrap_or(1)).unwrap();
            let mut paint = Paint {
                anti_alias: false,
                ..Default::default()
            };

            buffer.draw(
                &mut swash_cache,
                Color::rgb(0, 0, 0),
                |x, y, w, h, color| {
                    // Note: due to softbuffer and tiny_skia having incompatible internal color representations we swap
                    // the red and blue channels here
                    paint.set_color_rgba8(color.b(), color.g(), color.r(), color.a());
                    pixmap.fill_rect(
                        Rect::from_xywh(x as f32, y as f32, w as f32, h as f32).unwrap(),
                        &paint,
                        Transform::identity(),
                        None,
                    );
                },
            );

            pixmap
        };

        let mut result = Pixmap::new(
            8 * 5 + textbox.width() + avatar_pixmap.width(),
            8 * 4 + u32::max(48, textbox.height()),
        )
        .unwrap();
        result.draw_pixmap(
            8,
            8,
            avatar_pixmap.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );

        let mut paint = Paint::default();
        paint.set_color_rgba8(192, 229, 245, 255);
        result.draw_pixmap(
            64,
            8,
            rounded_rectangle_pixmap(textbox.width() + 16, textbox.height() + 16, 16.0)
                .expect("Build rounded rectangle failed!")
                .as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
        result.draw_pixmap(
            64 + 8,
            8 + 8,
            textbox.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );

        result
    };

    let data = pixmap.encode_png()?;
    let image = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;

    Ok(image)
}

fn rounded_rectangle_pixmap(w: u32, h: u32, r: f32) -> Option<Pixmap> {
    #[allow(clippy::all)]
    fn rounded_rectangle_path(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        mut r: f32,
    ) -> Option<tiny_skia::Path> {
        if h > w {
            if r > 0.24 * w {
                r = 0.24 * w
            }
        } else if h < w {
            if r > 0.24 * h {
                r = 0.24 * h
            }
        } else if h == w {
            if r > 0.24 * w {
                r = 0.24 * w
            }
        }

        let mut pb = PathBuilder::new();
        pb.move_to(x + r, y);
        pb.line_to(w - r, y);
        pb.cubic_to(w - (r / 2 as f32), y, w, y + (r / 2 as f32), w, y + r);
        pb.line_to(w, h - r);
        pb.cubic_to(w, h - (r / 2 as f32), w - (r / 2 as f32), h, w - r, h);
        pb.line_to(x + r, h);
        pb.cubic_to(x + (r / 2 as f32), h, x, h - (r / 2 as f32), x, h - r);
        pb.line_to(x, y + r);
        pb.cubic_to(x, y + (r / 2 as f32), x + (r / 2 as f32), y, x + r, y);
        pb.close();

        pb.finish()
    }

    let mut paint = Paint::default();
    paint.set_color_rgba8(192, 229, 245, 255);
    paint.anti_alias = true;

    let mut pixmap = Pixmap::new(w, h)?;
    pixmap.fill_path(
        &rounded_rectangle_path(0.0, 0.0, w as f32, h as f32, r)?,
        &paint,
        FillRule::EvenOdd,
        Transform::identity(),
        None,
    );
    Some(pixmap)
}
