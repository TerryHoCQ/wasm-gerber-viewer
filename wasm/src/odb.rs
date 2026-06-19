/// ODB++ format support module
/// 
/// This module provides parsing and conversion of ODB++ (Open Database++) format files
/// to the viewer's internal geometry representation (GerberData).
/// 
/// ODB++ is a compressed, layer-based PCB design format with:
/// - ZIP-compressed file structure
/// - XML-based layer definitions (stackup.xml)
/// - Binary layer data files
/// - Netlist and BOM information

pub mod model;
pub mod parser;
pub mod geometry;

pub use model::{OdbData, OdbLayer, OdbFile};
pub use parser::parse_odb_with_offset;
pub use geometry::convert_to_gerber_data;

use wasm_bindgen::prelude::*;

/// Parse ODB++ file from binary data with offset
/// 
/// # Arguments
/// * `data` - ODB++ file content as byte array
/// * `offset_x` - Horizontal offset in mm
/// * `offset_y` - Vertical offset in mm
///
/// # Returns
/// * `Result<Vec<GerberData>, JsValue>` - Parsed geometry layers or error
pub fn parse_odb_layer_impl(
    data: &[u8],
    offset_x: f32,
    offset_y: f32,
) -> Result<Vec<crate::shape::GerberData>, JsValue> {
    parser::parse_odb_with_offset(data, offset_x, offset_y)
}
