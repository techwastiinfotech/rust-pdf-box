use crate::cos::object::{COSDictionary, COSObject, COSString, ObjectId};
use crate::error::{PdfError, PdfResult};
use crate::pd::document::PDDocument;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Button, // Checkbox or Pushbutton
    Choice,
    Signature,
    Unknown(String),
}

/// Represents an AcroForm interactive form field.
#[derive(Debug, Clone)]
pub struct PDFormField {
    pub name: String,
    pub field_type: FieldType,
    pub value: Option<String>,
    pub field_id: ObjectId,
}

/// Interactive AcroForm management and form flattening.
#[derive(Debug, Clone)]
pub struct PDAcroForm {
    pub acro_form_dict: COSDictionary,
}

impl PDAcroForm {
    /// Loads the AcroForm from the document catalog if present.
    pub fn from_doc(doc: &PDDocument) -> Option<Self> {
        let catalog = doc.cos_doc.get_catalog()?;
        let acro_form = catalog.get_dict("AcroForm")?.clone();
        Some(Self {
            acro_form_dict: acro_form,
        })
    }

    /// Lists all form fields in the document.
    pub fn fields(&self, doc: &PDDocument) -> Vec<PDFormField> {
        let mut result = Vec::new();
        if let Some(fields_arr) = self.acro_form_dict.get_array("Fields") {
            for item in fields_arr.iter() {
                if let Some(id) = item.as_reference() {
                    if let Some(COSObject::Dictionary(d)) = doc.cos_doc.get_object(id) {
                        let name = d.get_string("T")
                            .map(|s| s.to_string_lossy())
                            .unwrap_or_default();

                        let ft_str = d.get_name("FT").unwrap_or("");
                        let field_type = match ft_str {
                            "Tx" => FieldType::Text,
                            "Btn" => FieldType::Button,
                            "Ch" => FieldType::Choice,
                            "Sig" => FieldType::Signature,
                            other => FieldType::Unknown(other.to_string()),
                        };

                        let value = d.get("V").map(|v| match v {
                            COSObject::String(s) => s.to_string_lossy(),
                            COSObject::Name(n) => n.as_str().to_string(),
                            _ => String::new(),
                        });

                        result.push(PDFormField {
                            name,
                            field_type,
                            value,
                            field_id: id,
                        });
                    }
                }
            }
        }
        result
    }

    /// Sets the value of a field by its fully-qualified name (/T).
    pub fn set_field_value(
        &mut self,
        doc: &mut PDDocument,
        name: &str,
        value: &str,
    ) -> PdfResult<()> {
        let fields = self.fields(doc);
        for f in fields {
            if f.name == name {
                if let Some(obj) = doc.cos_doc.get_object_mut(f.field_id) {
                    if let Some(dict) = obj.as_dict_mut() {
                        dict.set("V", COSString::literal(value.as_bytes().to_vec()));
                        return Ok(());
                    }
                }
            }
        }
        Err(PdfError::MissingEntry(format!("Form field '{name}' not found")))
    }

    /// Flattens all form fields into page content streams and removes interactive fields.
    pub fn flatten(&mut self, doc: &mut PDDocument) -> PdfResult<()> {
        let total_pages = doc.page_count();
        for i in 0..total_pages {
            let mut widget_appearance_streams = Vec::new();
            if let Ok(page) = doc.get_page(i) {
                if let Some(annots) = page.dict.get_array("Annots") {
                    for item in annots.iter() {
                        if let Some(id) = item.as_reference() {
                            if let Some(COSObject::Dictionary(d)) = doc.cos_doc.get_object(id) {
                                if d.get_name("Subtype") == Some("Widget") {
                                    if let Some(ap_dict) = d.get_dict("AP") {
                                        if let Some(n_id) = ap_dict.get_reference("N") {
                                            widget_appearance_streams.push(n_id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Append widget appearance streams to page contents
            if !widget_appearance_streams.is_empty() {
                let page = doc.get_page_mut(i)?;
                let mut current_ids = page.content_stream_ids();
                current_ids.extend(widget_appearance_streams);
                page.set_contents_array(current_ids);
            }

            // Remove widget annotations from page
            let page = doc.get_page_mut(i)?;
            page.dict.remove("Annots");
        }

        // Remove /AcroForm from Document Catalog
        if let Some(cat) = doc.cos_doc.get_catalog_mut() {
            cat.remove("AcroForm");
        }

        doc.sync();
        Ok(())
    }
}
