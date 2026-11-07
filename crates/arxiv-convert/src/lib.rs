//! Paper conversion pipelines.

pub mod html;
pub mod pdf;

pub use html::{html_to_markdown, ConvertError, CONVERTER_VERSION};
