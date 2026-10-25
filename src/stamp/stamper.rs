use crate::content::color::Color;
use crate::content::stream::{AppendMode, PDPageContentStream};
use crate::error::PdfResult;
use crate::font::type1::PDType1Font;
use crate::image::PDImageXObject;
use crate::pd::document::PDDocument;
use crate::stamp::options::{PageRange, StampLayer, StampPosition};

#[derive(Debug, Clone)]
enum StampContent {
    Watermark {
        text: String,
        font: PDType1Font,
        font_size: f64,
        color: Color,
        rotation: f64,
        opacity: f64,
    },
    RubberStamp {
        title: String,
        subtitle: Option<String>,
        border_color: Color,
        fill_color: Option<Color>,
        text_color: Color,
        rotation: f64,
        opacity: f64,
    },
    HeaderFooter {
        text: String,
        position: StampPosition,
        font: PDType1Font,
        font_size: f64,
        color: Color,
        opacity: f64,
    },
    Image {
        image: PDImageXObject,
        width: f64,
        height: f64,
        rotation: f64,
        opacity: f64,
    },
}

/// Context-oriented Stamper builder for applying watermarks, rubber stamps, headers/footers, and images.
#[derive(Debug, Clone)]
pub struct Stamper {
    pages: PageRange,
    layer: StampLayer,
    position: StampPosition,
    content: Option<StampContent>,
}

impl Default for Stamper {
    fn default() -> Self {
        Self::new()
    }
}

impl Stamper {
    pub fn new() -> Self {
        Self {
            pages: PageRange::All,
            layer: StampLayer::UnderContent,
            position: StampPosition::Center,
            content: None,
        }
    }

    /// Selects pages to apply the stamp on.
    pub fn pages(mut self, range: PageRange) -> Self {
        self.pages = range;
        self
    }

    /// Sets whether stamp is placed under or over page content.
    pub fn layer(mut self, layer: StampLayer) -> Self {
        self.layer = layer;
        self
    }

    /// Sets the position on the page.
    pub fn position(mut self, pos: StampPosition) -> Self {
        self.position = pos;
        self
    }

    /// Configures a semi-transparent text watermark.
    pub fn watermark(
        mut self,
        text: impl Into<String>,
        font_size: f64,
        color: Color,
        rotation: f64,
        opacity: f64,
    ) -> Self {
        self.layer = StampLayer::UnderContent;
        self.position = StampPosition::Center;
        self.content = Some(StampContent::Watermark {
            text: text.into(),
            font: PDType1Font::HelveticaBold,
            font_size,
            color,
            rotation,
            opacity,
        });
        self
    }

    /// Configures a simple centered diagonal watermark with default styling.
    pub fn simple_watermark(self, text: impl Into<String>) -> Self {
        self.watermark(text, 54.0, Color::rgb(0.85, 0.1, 0.1), 45.0, 0.25)
    }

    /// Configures a boxed rubber stamp (e.g. "APPROVED", "DRAFT", "CONFIDENTIAL").
    pub fn rubber_stamp(
        mut self,
        title: impl Into<String>,
        subtitle: Option<String>,
        color: Color,
        rotation: f64,
    ) -> Self {
        self.layer = StampLayer::OverContent;
        self.content = Some(StampContent::RubberStamp {
            title: title.into(),
            subtitle,
            border_color: color,
            fill_color: Some(Color::rgb(1.0, 1.0, 1.0)),
            text_color: color,
            rotation,
            opacity: 0.9,
        });
        self
    }

    /// Configures a header stamp with placeholder support: `{page}`, `{total}`.
    pub fn header(
        mut self,
        text: impl Into<String>,
        position: StampPosition,
        font_size: f64,
        color: Color,
    ) -> Self {
        self.layer = StampLayer::OverContent;
        self.position = position;
        self.content = Some(StampContent::HeaderFooter {
            text: text.into(),
            position,
            font: PDType1Font::Helvetica,
            font_size,
            color,
            opacity: 1.0,
        });
        self
    }

    /// Configures a footer stamp with placeholder support: `{page}`, `{total}`.
    pub fn footer(
        mut self,
        text: impl Into<String>,
        position: StampPosition,
        font_size: f64,
        color: Color,
    ) -> Self {
        self.layer = StampLayer::OverContent;
        self.position = position;
        self.content = Some(StampContent::HeaderFooter {
            text: text.into(),
            position,
            font: PDType1Font::Helvetica,
            font_size,
            color,
            opacity: 1.0,
        });
        self
    }

    /// Configures an image stamp.
    pub fn image(mut self, img: PDImageXObject, width: f64, height: f64, opacity: f64) -> Self {
        self.layer = StampLayer::OverContent;
        self.content = Some(StampContent::Image {
            image: img,
            width,
            height,
            rotation: 0.0,
            opacity,
        });
        self
    }

    /// Applies the configured stamp across all matching pages in the document.
    pub fn apply(&self, doc: &mut PDDocument) -> PdfResult<()> {
        let Some(content) = &self.content else {
            return Ok(());
        };

        let total_pages = doc.page_count();
        let append_mode = match self.layer {
            StampLayer::UnderContent => AppendMode::Prepend,
            StampLayer::OverContent => AppendMode::Append,
        };

        for page_idx in 0..total_pages {
            if !self.pages.matches(page_idx, total_pages) {
                continue;
            }

            let (page_w, page_h) = {
                let page = doc.get_page(page_idx)?;
                let mb = page.media_box();
                (mb.width(), mb.height())
            };

            let mut stream = PDPageContentStream::new(append_mode);
            stream.save_graphics_state();

            match content {
                StampContent::Watermark {
                    text,
                    font,
                    font_size,
                    color,
                    rotation,
                    opacity,
                } => {
                    let page_ref = doc.get_page_mut(page_idx)?;
                    stream.set_opacity_on_page(page_ref, *opacity, *opacity);

                    let (cx, cy) = (page_w / 2.0, page_h / 2.0);
                    let text_w = font.get_string_width(text, *font_size);

                    stream.save_graphics_state();
                    stream.rotate(*rotation, cx, cy);

                    stream.begin_text();
                    stream.set_font_type1(doc, page_idx, *font, *font_size);
                    stream.set_fill_color(*color);
                    stream.show_text_at(text, cx - (text_w / 2.0), cy - (*font_size / 3.0));
                    stream.end_text();

                    stream.restore_graphics_state();
                }

                StampContent::RubberStamp {
                    title,
                    subtitle,
                    border_color,
                    fill_color,
                    text_color,
                    rotation,
                    opacity,
                } => {
                    let page_ref = doc.get_page_mut(page_idx)?;
                    stream.set_opacity_on_page(page_ref, *opacity, *opacity);

                    let font = PDType1Font::HelveticaBold;
                    let title_font_size = 20.0;
                    let title_w = font.get_string_width(title, title_font_size);

                    let sub_font_size = 9.0;
                    let sub_w = subtitle.as_ref().map(|s| font.get_string_width(s, sub_font_size)).unwrap_or(0.0);

                    let stamp_w = title_w.max(sub_w) + 36.0;
                    let stamp_h = if subtitle.is_some() { 56.0 } else { 38.0 };

                    let (x, y) = self.calculate_xy(self.position, page_w, page_h, stamp_w, stamp_h);
                    let (cx, cy) = (x + stamp_w / 2.0, y + stamp_h / 2.0);

                    stream.save_graphics_state();
                    if *rotation != 0.0 {
                        stream.rotate(*rotation, cx, cy);
                    }

                    // Background fill
                    if let Some(fc) = fill_color {
                        stream.set_fill_color(*fc);
                        stream.round_rect(x, y, stamp_w, stamp_h, 6.0);
                        stream.fill();
                    }

                    // Border
                    stream.set_stroke_color(*border_color);
                    stream.set_line_width(2.5);
                    stream.round_rect(x, y, stamp_w, stamp_h, 6.0);
                    stream.stroke();

                    // Title
                    stream.begin_text();
                    stream.set_font_type1(doc, page_idx, font, title_font_size);
                    stream.set_fill_color(*text_color);
                    let title_y = if subtitle.is_some() { y + 26.0 } else { y + 12.0 };
                    stream.show_text_at(title, x + (stamp_w - title_w) / 2.0, title_y);
                    stream.end_text();

                    // Subtitle
                    if let Some(sub) = subtitle {
                        stream.begin_text();
                        stream.set_font_type1(doc, page_idx, PDType1Font::Helvetica, sub_font_size);
                        stream.set_fill_color(*text_color);
                        stream.show_text_at(sub, x + (stamp_w - sub_w) / 2.0, y + 10.0);
                        stream.end_text();
                    }

                    stream.restore_graphics_state();
                }

                StampContent::HeaderFooter {
                    text,
                    position,
                    font,
                    font_size,
                    color,
                    opacity,
                } => {
                    let page_ref = doc.get_page_mut(page_idx)?;
                    stream.set_opacity_on_page(page_ref, *opacity, *opacity);

                    let formatted_text = text
                        .replace("{page}", &(page_idx + 1).to_string())
                        .replace("{total}", &total_pages.to_string());

                    let text_w = font.get_string_width(&formatted_text, *font_size);
                    let (x, y) = self.calculate_xy(*position, page_w, page_h, text_w, *font_size);

                    stream.begin_text();
                    stream.set_font_type1(doc, page_idx, *font, *font_size);
                    stream.set_fill_color(*color);
                    stream.show_text_at(&formatted_text, x, y);
                    stream.end_text();
                }

                StampContent::Image {
                    image,
                    width,
                    height,
                    rotation,
                    opacity,
                } => {
                    let res_name = {
                        let page_ref = doc.get_page_mut(page_idx)?;
                        stream.set_opacity_on_page(page_ref, *opacity, *opacity);
                        image.register_with_page(page_ref)
                    };

                    let (x, y) = self.calculate_xy(self.position, page_w, page_h, *width, *height);
                    let (cx, cy) = (x + width / 2.0, y + height / 2.0);

                    stream.save_graphics_state();
                    if *rotation != 0.0 {
                        stream.rotate(*rotation, cx, cy);
                    }
                    stream.draw_image(&res_name, x, y, *width, *height);
                    stream.restore_graphics_state();
                }
            }

            stream.restore_graphics_state();
            stream.finish(doc, page_idx)?;
        }

        Ok(())
    }

    fn calculate_xy(
        &self,
        pos: StampPosition,
        page_w: f64,
        page_h: f64,
        element_w: f64,
        element_h: f64,
    ) -> (f64, f64) {
        match pos {
            StampPosition::Center => (
                (page_w - element_w) / 2.0,
                (page_h - element_h) / 2.0,
            ),
            StampPosition::TopLeft { margin_x, margin_y } => (
                margin_x,
                page_h - margin_y - element_h,
            ),
            StampPosition::TopCenter { margin_y } => (
                (page_w - element_w) / 2.0,
                page_h - margin_y - element_h,
            ),
            StampPosition::TopRight { margin_x, margin_y } => (
                page_w - margin_x - element_w,
                page_h - margin_y - element_h,
            ),
            StampPosition::BottomLeft { margin_x, margin_y } => (
                margin_x,
                margin_y,
            ),
            StampPosition::BottomCenter { margin_y } => (
                (page_w - element_w) / 2.0,
                margin_y,
            ),
            StampPosition::BottomRight { margin_x, margin_y } => (
                page_w - margin_x - element_w,
                margin_y,
            ),
            StampPosition::Custom { x, y } => (x, y),
        }
    }
}
