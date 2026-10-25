use crate::cos::document::COSDocument;
use crate::cos::object::{COSDictionary, COSName, COSStream, ObjectId};
use crate::error::{PdfError, PdfResult};
use crate::pd::page::PDPage;
use image::{GenericImageView, ImageFormat};

/// High-level Image XObject (/Subtype /Image).
#[derive(Debug, Clone)]
pub struct PDImageXObject {
    pub width: u32,
    pub height: u32,
    pub image_id: ObjectId,
}

impl PDImageXObject {
    /// Creates an Image XObject from JPEG bytes using native DCTDecode passthrough.
    pub fn create_from_jpeg(cos_doc: &mut COSDocument, jpeg_bytes: &[u8]) -> PdfResult<Self> {
        let dyn_img = image::load_from_memory_with_format(jpeg_bytes, ImageFormat::Jpeg)
            .map_err(|e| PdfError::Image(format!("Failed parsing JPEG: {e}")))?;
        let (width, height) = dyn_img.dimensions();

        let mut dict = COSDictionary::new();
        dict.set("Type", COSName::new("XObject"));
        dict.set("Subtype", COSName::new("Image"));
        dict.set("Width", width as i64);
        dict.set("Height", height as i64);
        dict.set("ColorSpace", COSName::new("DeviceRGB"));
        dict.set("BitsPerComponent", 8);
        dict.set("Filter", COSName::new("DCTDecode"));

        let stream = COSStream::new(dict, jpeg_bytes.to_vec());
        let image_id = cos_doc.add_object(stream);

        Ok(Self {
            width,
            height,
            image_id,
        })
    }

    /// Creates an Image XObject from PNG or raw image bytes with transparency / alpha mask support.
    pub fn create_from_image_bytes(cos_doc: &mut COSDocument, img_bytes: &[u8]) -> PdfResult<Self> {
        let dyn_img = image::load_from_memory(img_bytes)
            .map_err(|e| PdfError::Image(format!("Failed parsing image bytes: {e}")))?;

        let (width, height) = dyn_img.dimensions();
        let rgba = dyn_img.to_rgba8();

        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
        let mut alpha_data = Vec::with_capacity((width * height) as usize);
        let mut has_transparency = false;

        for pixel in rgba.pixels() {
            rgb_data.push(pixel[0]);
            rgb_data.push(pixel[1]);
            rgb_data.push(pixel[2]);
            let a = pixel[3];
            alpha_data.push(a);
            if a < 255 {
                has_transparency = true;
            }
        }

        let mut dict = COSDictionary::new();
        dict.set("Type", COSName::new("XObject"));
        dict.set("Subtype", COSName::new("Image"));
        dict.set("Width", width as i64);
        dict.set("Height", height as i64);
        dict.set("ColorSpace", COSName::new("DeviceRGB"));
        dict.set("BitsPerComponent", 8);

        // If image has alpha channel, create Soft Mask (/SMask)
        if has_transparency {
            let mut smask_dict = COSDictionary::new();
            smask_dict.set("Type", COSName::new("XObject"));
            smask_dict.set("Subtype", COSName::new("Image"));
            smask_dict.set("Width", width as i64);
            smask_dict.set("Height", height as i64);
            smask_dict.set("ColorSpace", COSName::new("DeviceGray"));
            smask_dict.set("BitsPerComponent", 8);

            let mut smask_stream = COSStream::new(smask_dict, alpha_data);
            let _ = smask_stream.encode_flate();
            let smask_id = cos_doc.add_object(smask_stream);
            dict.set("SMask", smask_id);
        }

        let mut stream = COSStream::new(dict, rgb_data);
        let _ = stream.encode_flate();
        let image_id = cos_doc.add_object(stream);

        Ok(Self {
            width,
            height,
            image_id,
        })
    }

    /// Registers this image with a page and returns its resource name (e.g. "Im1").
    pub fn register_with_page(&self, page: &mut PDPage) -> String {
        page.resources.add_xobject("Im", self.image_id)
    }
}
