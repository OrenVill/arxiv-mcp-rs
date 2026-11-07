//! Paper conversion pipelines.

pub mod html;
pub mod pdf;

pub use html::{html_to_markdown, ConvertError, CONVERTER_VERSION};
pub use pdf::{pdf_to_markdown, PdfConvertError, DEFAULT_MAX_PDF_PAGES};
