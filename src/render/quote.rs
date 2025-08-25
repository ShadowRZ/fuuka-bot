use image::imageops::FilterType;
use image::{DynamicImage, ImageReader};
use matrix_sdk::ruma::UInt;
use matrix_sdk::ruma::events::room::message::TextMessageEventContent;
use matrix_sdk::{
    media::{MediaFormat, MediaThumbnailSettings},
    room::RoomMember,
    ruma::media::Method,
};
use parking_lot::Mutex;
use ruma::html::{Children, Html, NodeData};
use std::io::Cursor;
use std::sync::LazyLock;
use tiny_skia::{FillRule, IntSize, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Transform};

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

    let name = room_member.name_or_id().to_string();

    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::task::spawn_blocking(move || {
        let res = render_inner(content, name, avatar);
        let _ = tx.send(res);
    });

    rx.await?
}

fn render_inner(
    content: TextMessageEventContent,
    name: String,
    avatar: Option<Vec<u8>>,
) -> anyhow::Result<DynamicImage> {
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
        let textbox = {
            let doc = match content.formatted {
                Some(body) => Document::from_html(ruma::html::Html::parse(&body.body)),
                None => Document::from_text(content.body),
            };
            let image = doc.render(&name);

            let (width, height) = (image.width(), image.height());
            let mut pixmap = Pixmap::new(width, height).unwrap();

            let data = image
                .pixels()
                .map(|Rgba(color)| {
                    let color =
                        tiny_skia::ColorU8::from_rgba(color[0], color[1], color[2], color[3]);
                    color.premultiply()
                })
                .collect::<Vec<_>>();

            pixmap.pixels_mut().copy_from_slice(&data);

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

use image::{GenericImage, Pixel, Rgba, RgbaImage};
use parley::{
    Alignment, AlignmentOptions, FontContext, FontFamily, FontStack, FontStyle, FontWeight,
    GenericFamily, Glyph, GlyphRun, LayoutContext, LineHeight, PositionedLayoutItem, StyleProperty,
    TextStyle,
};
use swash::{
    FontRef,
    scale::{Render, ScaleContext, Scaler, Source, StrikeWith, image::Content},
    zeno::{Format, Vector},
};

use crate::RoomMemberExt;

static FONT_CONTEXT: LazyLock<Mutex<FontContext>> = LazyLock::new(|| FontContext::new().into());

static LAYOUT_CONTEXT: LazyLock<Mutex<LayoutContext<Brush>>> =
    LazyLock::new(|| LayoutContext::new().into());

static SCALE_CONTEXT: LazyLock<Mutex<ScaleContext>> = LazyLock::new(|| ScaleContext::new().into());

static ROOT_STYLE: LazyLock<TextStyle<'static, Brush>> = LazyLock::new(|| TextStyle {
    brush: Brush::default(),
    font_stack: FontStack::Single(FontFamily::Generic(GenericFamily::SansSerif)),
    line_height: LineHeight::Absolute(1.3),
    font_size: 16.0,
    ..Default::default()
});

static BOLD: LazyLock<StyleProperty<'static, Brush>> =
    LazyLock::new(|| StyleProperty::FontWeight(FontWeight::new(600.0)));
static UNDERLINE: LazyLock<StyleProperty<'static, Brush>> =
    LazyLock::new(|| StyleProperty::Underline(true));
static ITALIC: LazyLock<StyleProperty<'static, Brush>> =
    LazyLock::new(|| StyleProperty::FontStyle(FontStyle::Italic));
static STRIKE: LazyLock<StyleProperty<'static, Brush>> =
    LazyLock::new(|| StyleProperty::Strikethrough(true));

static MONOSPACE: LazyLock<StyleProperty<'static, Brush>> = LazyLock::new(|| {
    StyleProperty::FontStack(FontStack::Single(FontFamily::Generic(
        GenericFamily::Monospace,
    )))
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brush {
    pub foreground: Rgba<u8>,
    pub background: Rgba<u8>,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            foreground: Rgba([0, 0, 0, 255]),
            background: Rgba([0, 0, 0, 0]),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum LayoutOps {
    Text(String),
    InlineBox,
    Bold,
    Italic,
    Underline,
    Strike,
    #[allow(dead_code)] // TODO
    ChangeBrush(Brush),
    MonospaceFont,
    PopStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BlockType {
    Normal,
    Blockquote,
}

impl Default for BlockType {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Default, Debug, Clone)]
struct Block {
    inner: Vec<LayoutOps>,
    type_: BlockType,
}

impl Block {
    pub fn push_layout_op(&mut self, layout_op: LayoutOps) {
        self.inner.push(layout_op);
    }
}

#[derive(Debug)]
struct Document {
    inner: Vec<Block>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            inner: vec![Default::default()],
        }
    }

    pub fn from_html(html: Html) -> Self {
        let mut res = Self::new();
        res.update(html.children());

        res
    }

    pub fn from_text(text: String) -> Self {
        Self {
            inner: vec![Block {
                inner: vec![LayoutOps::Text(text)],
                type_: BlockType::Normal,
            }],
        }
    }

    pub fn push_layout_op(&mut self, layout_op: LayoutOps) {
        self.inner
            .last_mut()
            .expect("A document should have at least one block")
            .push_layout_op(layout_op);
    }

    pub fn push_block(&mut self, type_: BlockType) {
        self.inner.push(Block {
            type_,
            ..Default::default()
        });
    }

    pub fn render(mut self, name: &str) -> RgbaImage {
        let mut images = self
            .inner
            .iter_mut()
            .filter(|block| !block.inner.is_empty())
            .map(Self::render_single_block)
            .collect::<Vec<_>>();

        let namebox = Self::render_name_block(name);
        images.insert(0, namebox);

        let (width, height) = images.iter().fold((0, 0), |(width, height), image| {
            let (new_w, new_h) = (image.width(), image.height());
            (u32::max(width, new_w), height + new_h)
        });

        let mut res = RgbaImage::new(width, height);

        let mut y = 0;
        for img in images.into_iter() {
            use image::GenericImage;
            res.copy_from(&img, 0, y).unwrap();
            y += img.height();
        }

        res
    }

    fn render_name_block(name: &str) -> RgbaImage {
        let mut font_context = FONT_CONTEXT.lock();
        let mut layout = LAYOUT_CONTEXT.lock();
        let mut builder = layout.tree_builder(&mut font_context, 1.0, true, &ROOT_STYLE);

        let max_advance = Some(500.0);
        builder.push_style_span(TextStyle {
            font_size: 20.0,
            font_weight: FontWeight::BOLD,
            brush: Brush {
                foreground: Rgba([0x1f, 0x47, 0x88, 0xFF]),
                ..Default::default()
            },
            ..Default::default()
        });
        builder.push_text(name);

        let (mut layout, _text) = builder.build();
        layout.break_all_lines(max_advance);
        layout.align(max_advance, Alignment::Start, AlignmentOptions::default());

        // Create image to render into
        let width = layout.width().ceil() as u32;
        let height = layout.height().ceil() as u32;
        let mut img = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));

        // Iterate over laid out lines
        for line in layout.lines() {
            // Iterate over GlyphRun's within each line
            for item in line.items() {
                match item {
                    PositionedLayoutItem::GlyphRun(glyph_run) => {
                        render_glyph_run(&mut SCALE_CONTEXT.lock(), &glyph_run, &mut img);
                    }
                    PositionedLayoutItem::InlineBox(_) => {}
                }
            }
        }

        img
    }

    fn render_single_block(block: &mut Block) -> RgbaImage {
        let mut font_context = FONT_CONTEXT.lock();
        let mut layout = LAYOUT_CONTEXT.lock();
        let mut builder = layout.tree_builder(&mut font_context, 1.0, true, &ROOT_STYLE);
        let ops = &mut block.inner;
        if ops
            .last()
            .map(|op| *op == LayoutOps::Text("\n".to_string()))
            .unwrap_or_default()
        {
            ops.pop();
        }

        let max_advance = Some(match block.type_ {
            BlockType::Normal => 500.0,
            BlockType::Blockquote => 500.0 - 2.0 - 4.0,
        });

        for op in ops {
            match op {
                LayoutOps::Text(text) => builder.push_text(text),
                LayoutOps::InlineBox => {}
                LayoutOps::Bold => builder.push_style_modification_span(&[BOLD.clone()]),
                LayoutOps::Italic => builder.push_style_modification_span(&[ITALIC.clone()]),
                LayoutOps::Underline => builder.push_style_modification_span(&[UNDERLINE.clone()]),
                LayoutOps::Strike => builder.push_style_modification_span(&[STRIKE.clone()]),
                LayoutOps::ChangeBrush(_) => {}
                LayoutOps::MonospaceFont => {
                    builder.push_style_modification_span(&[MONOSPACE.clone()])
                }
                LayoutOps::PopStyle => builder.pop_style_span(),
            }
        }

        let (mut layout, _text) = builder.build();
        layout.break_all_lines(max_advance);
        layout.align(max_advance, Alignment::Start, AlignmentOptions::default());

        // Create image to render into
        let width = layout.width().ceil() as u32;
        let height = layout.height().ceil() as u32;
        let mut img = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));

        // Iterate over laid out lines
        for line in layout.lines() {
            // Iterate over GlyphRun's within each line
            for item in line.items() {
                match item {
                    PositionedLayoutItem::GlyphRun(glyph_run) => {
                        render_glyph_run(&mut SCALE_CONTEXT.lock(), &glyph_run, &mut img);
                    }
                    PositionedLayoutItem::InlineBox(_) => {}
                }
            }
        }

        match block.type_ {
            BlockType::Normal => img,
            BlockType::Blockquote => {
                use imageproc::rect::Rect;
                let new_img = RgbaImage::new(width + 6, height);
                let mut new_img = imageproc::drawing::draw_filled_rect(
                    &new_img,
                    Rect::at(0, 0).of_size(2, height),
                    Rgba([0, 0, 0, 255]),
                );
                new_img.copy_from(&img, 2 + 4, 0).unwrap();
                new_img
            }
        }
    }

    fn update(&mut self, children: Children) {
        for node in children {
            match node.data() {
                NodeData::Document => continue,
                NodeData::Text(text) => {
                    self.push_layout_op(LayoutOps::Text(text.borrow().to_string()));
                }
                NodeData::Element(element) => {
                    use ruma::html::matrix::MatrixElement;
                    let matrix = element.to_matrix();

                    match matrix.element {
                        MatrixElement::Del => {
                            self.push_layout_op(LayoutOps::Strike);
                            self.update(node.children());
                            self.push_layout_op(LayoutOps::PopStyle);
                        }
                        // TODO: Heading support
                        MatrixElement::H(_) => {
                            self.push_block(BlockType::Normal);
                            self.update(node.children());
                        }
                        // TODO: Blockquote
                        MatrixElement::Blockquote => {
                            self.push_block(BlockType::Blockquote);
                            self.update(node.children());
                        }
                        MatrixElement::P => {
                            self.push_block(BlockType::Normal);
                            self.update(node.children());
                        }
                        // TODO: Anchor
                        MatrixElement::A(_) => {
                            self.update(node.children());
                        }
                        // TODO: Unordered List
                        MatrixElement::Ul => {
                            self.push_block(BlockType::Normal);
                            self.update(node.children());
                        }
                        MatrixElement::Ol(_) => {
                            self.push_block(BlockType::Normal);
                            self.update(node.children());
                        }
                        MatrixElement::Sup => continue, // TODO
                        MatrixElement::Sub => continue, // TODO
                        MatrixElement::Li => continue,  // TODO
                        MatrixElement::B => {
                            self.push_layout_op(LayoutOps::Bold);
                            self.update(node.children());
                            self.push_layout_op(LayoutOps::PopStyle);
                        }
                        MatrixElement::I => {
                            self.push_layout_op(LayoutOps::Italic);
                            self.update(node.children());
                            self.push_layout_op(LayoutOps::PopStyle);
                        }
                        MatrixElement::U => {
                            self.push_layout_op(LayoutOps::Underline);
                            self.update(node.children());
                            self.push_layout_op(LayoutOps::PopStyle);
                        }
                        MatrixElement::Strong => {
                            self.push_layout_op(LayoutOps::Bold);
                            self.update(node.children());
                            self.push_layout_op(LayoutOps::PopStyle);
                        }
                        MatrixElement::Em => {
                            self.push_layout_op(LayoutOps::Italic);
                            self.update(node.children());
                            self.push_layout_op(LayoutOps::PopStyle);
                        }
                        MatrixElement::S => {
                            self.push_layout_op(LayoutOps::Strike);
                            self.update(node.children());
                            self.push_layout_op(LayoutOps::PopStyle);
                        }
                        MatrixElement::Code(_) => {
                            self.push_layout_op(LayoutOps::MonospaceFont);
                            self.update(node.children());
                            self.push_layout_op(LayoutOps::PopStyle);
                        }
                        MatrixElement::Hr => continue, // TODO
                        MatrixElement::Br => self.push_layout_op(LayoutOps::Text("\n".to_string())),
                        MatrixElement::Div(_) => {
                            self.push_block(BlockType::Normal);
                            self.update(node.children());
                        }
                        MatrixElement::Table => continue,
                        MatrixElement::Thead => continue,
                        MatrixElement::Tbody => continue,
                        MatrixElement::Tr => continue,
                        MatrixElement::Th => continue,
                        MatrixElement::Td => continue,
                        MatrixElement::Caption => continue,
                        MatrixElement::Pre => {
                            self.push_block(BlockType::Normal);
                            self.push_layout_op(LayoutOps::MonospaceFont);
                            self.update(node.children());
                            self.push_layout_op(LayoutOps::PopStyle);
                        }
                        // TODO: Span Style
                        MatrixElement::Span(_) => {
                            self.update(node.children());
                        }
                        MatrixElement::Img(_) => {
                            self.push_layout_op(LayoutOps::InlineBox);
                        }
                        MatrixElement::Details => continue,
                        MatrixElement::Summary => continue,
                        // Ignore any existing <mx-reply>
                        MatrixElement::MatrixReply => continue,
                        MatrixElement::Other(_) => {
                            // Just assume it's a block element
                            self.push_block(BlockType::Normal);
                            self.update(node.children());
                        }
                        _ => continue,
                    }
                }
                NodeData::Other => continue,
            }
        }
    }
}

fn render_glyph_run(
    context: &mut ScaleContext,
    glyph_run: &GlyphRun<'_, Brush>,
    img: &mut RgbaImage,
) {
    // Resolve properties of the GlyphRun
    let mut run_x = glyph_run.offset();
    let run_y = glyph_run.baseline();
    let style = glyph_run.style();
    let color = style.brush;

    // Get the "Run" from the "GlyphRun"
    let run = glyph_run.run();

    // Resolve properties of the Run
    let font = run.font();
    let font_size = run.font_size();
    let normalized_coords = run.normalized_coords();

    // Convert from parley::Font to swash::FontRef
    let font_ref = FontRef::from_index(font.data.as_ref(), font.index as usize).unwrap();

    // Build a scaler. As the font properties are constant across an entire run of glyphs
    // we can build one scaler for the run and reuse it for each glyph.
    let mut scaler = context
        .builder(font_ref)
        .size(font_size)
        .hint(true)
        .normalized_coords(normalized_coords)
        .build();

    // Iterates over the glyphs in the GlyphRun
    for glyph in glyph_run.glyphs() {
        let glyph_x = run_x + glyph.x;
        let glyph_y = run_y - glyph.y;
        run_x += glyph.advance;

        render_glyph(img, &mut scaler, color, glyph, glyph_x, glyph_y);
    }

    // Draw decorations: underline & strikethrough
    let style = glyph_run.style();
    let run_metrics = run.metrics();
    if let Some(decoration) = &style.underline {
        let offset = decoration.offset.unwrap_or(run_metrics.underline_offset);
        let size = decoration.size.unwrap_or(run_metrics.underline_size);
        render_decoration(img, glyph_run, decoration.brush, offset, size);
    }
    if let Some(decoration) = &style.strikethrough {
        let offset = decoration
            .offset
            .unwrap_or(run_metrics.strikethrough_offset);
        let size = decoration.size.unwrap_or(run_metrics.strikethrough_size);
        render_decoration(img, glyph_run, decoration.brush, offset, size);
    }
}

fn render_decoration(
    img: &mut RgbaImage,
    glyph_run: &GlyphRun<'_, Brush>,
    brush: Brush,
    offset: f32,
    width: f32,
) {
    let y = glyph_run.baseline() - offset;
    for pixel_y in y as u32..(y + width) as u32 {
        for pixel_x in glyph_run.offset() as u32..(glyph_run.offset() + glyph_run.advance()) as u32
        {
            if let Some(pixel) = img.get_pixel_mut_checked(pixel_x, pixel_y) {
                pixel.blend(&brush.foreground)
            }
        }
    }
}

fn render_glyph(
    img: &mut RgbaImage,
    scaler: &mut Scaler<'_>,
    brush: Brush,
    glyph: Glyph,
    glyph_x: f32,
    glyph_y: f32,
) {
    // Compute the fractional offset
    // You'll likely want to quantize this in a real renderer
    let offset = Vector::new(glyph_x.fract(), glyph_y.fract());

    // Render the glyph using swash
    let rendered_glyph = Render::new(
        // Select our source order
        &[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
        ],
    )
    // Select the simple alpha (non-subpixel) format
    .format(Format::Alpha)
    // Apply the fractional offset
    .offset(offset)
    // Render the image
    .render(scaler, glyph.id)
    .unwrap();

    let glyph_width = rendered_glyph.placement.width;
    let glyph_height = rendered_glyph.placement.height;
    let glyph_x = glyph_x.floor() as i32 + rendered_glyph.placement.left;
    let glyph_x = if glyph_x <= 0 { 0 } else { glyph_x as u32 };
    let glyph_y = glyph_y.floor() as i32 - rendered_glyph.placement.top;
    let glyph_y = if glyph_y <= 0 { 0 } else { glyph_y as u32 };

    match rendered_glyph.content {
        Content::Mask => {
            let mut i = 0;
            let bc = brush.foreground;
            for pixel_y in 0..glyph_height {
                for pixel_x in 0..glyph_width {
                    let x = glyph_x + pixel_x;
                    let y = glyph_y + pixel_y;
                    let alpha = rendered_glyph.data[i];
                    let color = Rgba([bc[0], bc[1], bc[2], alpha]);
                    if let Some(pixel) = img.get_pixel_mut_checked(x, y) {
                        pixel.blend(&color)
                    }
                    i += 1;
                }
            }
        }
        Content::SubpixelMask => {}
        Content::Color => {
            let row_size = glyph_width as usize * 4;
            for (pixel_y, row) in rendered_glyph.data.chunks_exact(row_size).enumerate() {
                for (pixel_x, pixel) in row.chunks_exact(4).enumerate() {
                    let x = glyph_x + pixel_x as u32;
                    let y = glyph_y + pixel_y as u32;
                    let color = Rgba(pixel.try_into().expect("Not RGBA"));
                    if let Some(pixel) = img.get_pixel_mut_checked(x, y) {
                        pixel.blend(&color)
                    }
                }
            }
        }
    }
}
