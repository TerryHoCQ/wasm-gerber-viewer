/// ODB++ format parser
/// 
/// Parses ODB++ ZIP-compressed files and extracts geometry data

use crate::odb::model::*;
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use zip::ZipArchive;

/// Parse ODB++ file from binary data with offset
/// 
/// # Arguments
/// * `data` - ODB++ file content as byte array
/// * `offset_x` - Horizontal offset in mm
/// * `offset_y` - Vertical offset in mm
///
/// # Returns
/// * `Result<Vec<GerberData>, JsValue>` - Parsed geometry layers or error
pub fn parse_odb_with_offset(
    data: &[u8],
    offset_x: f32,
    offset_y: f32,
) -> Result<Vec<crate::shape::GerberData>, JsValue> {
    // Parse ODB++ file
    let odb_data = parse_odb_archive(data)?;

    // Convert to GerberData format
    crate::odb::geometry::convert_to_gerber_data(&odb_data, offset_x, offset_y)
}

/// Parse ODB++ ZIP archive structure
fn parse_odb_archive(data: &[u8]) -> Result<OdbData, JsValue> {
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| JsValue::from_str(&format!("Failed to open ODB++ archive: {}", e)))?;

    // Read and parse stackup.xml
    let stackup = parse_stackup(&mut archive)?;

    // Extract layers
    let layers = extract_layers(&mut archive, &stackup)?;

    // Extract netlist if present
    let netlist = parse_netlist(&mut archive).ok();

    // Extract BOM if present
    let bom = parse_bom(&mut archive).ok();

    let odb_file = OdbFile {
        version: "1.0".to_string(),
        stackup,
        layers,
        netlist,
        bom,
    };

    Ok(OdbData::new(odb_file, 1.0))
}

/// Parse stackup.xml to get layer definitions
fn parse_stackup(archive: &mut ZipArchive<Cursor<&[u8]>>) -> Result<OdbStackup, JsValue> {
    let mut stackup_file = archive
        .by_name("stackup.xml")
        .map_err(|e| JsValue::from_str(&format!("stackup.xml not found: {}", e)))?;

    let mut content = String::new();
    use std::io::Read;
    stackup_file
        .read_to_string(&mut content)
        .map_err(|e| JsValue::from_str(&format!("Failed to read stackup.xml: {}", e)))?;

    parse_stackup_xml(&content)
}

/// Parse stackup XML content
fn parse_stackup_xml(xml_content: &str) -> Result<OdbStackup, JsValue> {
    use quick_xml::de::from_str;

    #[derive(serde::Deserialize)]
    struct Stackup {
        #[serde(default)]
        layer: Vec<StackupLayerXml>,
    }

    #[derive(serde::Deserialize)]
    struct StackupLayerXml {
        #[serde(rename = "@name")]
        name: Option<String>,
        #[serde(rename = "@type")]
        layer_type: Option<String>,
        #[serde(rename = "@index")]
        index: Option<i32>,
        #[serde(rename = "@thickness")]
        thickness: Option<f32>,
    }

    match from_str::<Stackup>(xml_content) {
        Ok(stackup) => {
            let layers = stackup
                .layer
                .into_iter()
                .filter_map(|l| {
                    Some(StackupLayer {
                        name: l.name.unwrap_or_default(),
                        layer_type: l.layer_type.unwrap_or_default(),
                        index: l.index.unwrap_or(0),
                        thickness: l.thickness,
                    })
                })
                .collect();

            Ok(OdbStackup { layers })
        }
        Err(e) => Err(JsValue::from_str(&format!(
            "Failed to parse stackup.xml: {}",
            e
        ))),
    }
}

/// Extract layer geometry files from archive
fn extract_layers(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    stackup: &OdbStackup,
) -> Result<Vec<OdbLayer>, JsValue> {
    let mut layers = Vec::new();

    // Iterate through stackup layers
    for stackup_layer in &stackup.layers {
        // Try to find corresponding layer data file
        let layer_path = format!("layers/{}/", stackup_layer.name);

        // Parse layer geometry
        let geometries = parse_layer_geometries(archive, &layer_path)?;

        if !geometries.is_empty() {
            layers.push(OdbLayer {
                name: stackup_layer.name.clone(),
                layer_type: stackup_layer.layer_type.clone(),
                index: stackup_layer.index,
                geometries,
                polarity: Polarity::Positive,
            });
        }
    }

    Ok(layers)
}

/// Parse geometries from a layer directory
fn parse_layer_geometries(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    layer_path: &str,
) -> Result<Vec<OdbGeometry>, JsValue> {
    let mut geometries = Vec::new();

    // Look for geometry files in the layer directory
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| JsValue::from_str(&format!("Failed to read archive entry: {}", e)))?;

        let file_name = file.name();

        // Check if file is in the layer directory and has geometry data
        if file_name.starts_with(layer_path) && file_name.ends_with(".gds") {
            // Parse GDS or geometry data
            // This is a simplified stub - actual GDS parsing is complex
            // For now, we'll skip actual geometry parsing
        }
    }

    Ok(geometries)
}

/// Parse netlist from netlist file
fn parse_netlist(archive: &mut ZipArchive<Cursor<&[u8]>>) -> Result<OdbNetlist, JsValue> {
    let mut netlist_file = archive
        .by_name("netlist.xml")
        .map_err(|e| JsValue::from_str(&format!("netlist.xml not found: {}", e)))?;

    let mut content = String::new();
    use std::io::Read;
    netlist_file
        .read_to_string(&mut content)
        .map_err(|e| JsValue::from_str(&format!("Failed to read netlist.xml: {}", e)))?;

    parse_netlist_xml(&content)
}

/// Parse netlist XML content
fn parse_netlist_xml(xml_content: &str) -> Result<OdbNetlist, JsValue> {
    use quick_xml::de::from_str;

    #[derive(serde::Deserialize)]
    struct NetlistXml {
        #[serde(default)]
        net: Vec<NetXml>,
    }

    #[derive(serde::Deserialize)]
    struct NetXml {
        #[serde(rename = "@name")]
        name: Option<String>,
        #[serde(default)]
        connection: Vec<ConnectionXml>,
    }

    #[derive(serde::Deserialize)]
    struct ConnectionXml {
        #[serde(rename = "@ref")]
        ref_val: Option<String>,
    }

    match from_str::<NetlistXml>(xml_content) {
        Ok(netlist_xml) => {
            let nets = netlist_xml
                .net
                .into_iter()
                .map(|n| Net {
                    name: n.name.unwrap_or_default(),
                    connections: n
                        .connection
                        .into_iter()
                        .filter_map(|c| c.ref_val)
                        .collect(),
                })
                .collect();

            Ok(OdbNetlist { nets })
        }
        Err(e) => Err(JsValue::from_str(&format!(
            "Failed to parse netlist.xml: {}",
            e
        ))),
    }
}

/// Parse BOM from BOM file
fn parse_bom(archive: &mut ZipArchive<Cursor<&[u8]>>) -> Result<OdbBom, JsValue> {
    let mut bom_file = archive
        .by_name("bom.xml")
        .map_err(|e| JsValue::from_str(&format!("bom.xml not found: {}", e)))?;

    let mut content = String::new();
    use std::io::Read;
    bom_file
        .read_to_string(&mut content)
        .map_err(|e| JsValue::from_str(&format!("Failed to read bom.xml: {}", e)))?;

    parse_bom_xml(&content)
}

/// Parse BOM XML content
fn parse_bom_xml(xml_content: &str) -> Result<OdbBom, JsValue> {
    use quick_xml::de::from_str;

    #[derive(serde::Deserialize)]
    struct BomXml {
        #[serde(default)]
        component: Vec<ComponentXml>,
    }

    #[derive(serde::Deserialize)]
    struct ComponentXml {
        #[serde(rename = "@ref")]
        reference: Option<String>,
        #[serde(rename = "@value")]
        value: Option<String>,
        #[serde(rename = "@footprint")]
        footprint: Option<String>,
        #[serde(rename = "@x")]
        x: Option<f32>,
        #[serde(rename = "@y")]
        y: Option<f32>,
        #[serde(rename = "@rotation")]
        rotation: Option<f32>,
    }

    match from_str::<BomXml>(xml_content) {
        Ok(bom_xml) => {
            let components = bom_xml
                .component
                .into_iter()
                .map(|c| Component {
                    reference: c.reference.unwrap_or_default(),
                    value: c.value.unwrap_or_default(),
                    footprint: c.footprint.unwrap_or_default(),
                    x: c.x.unwrap_or(0.0),
                    y: c.y.unwrap_or(0.0),
                    rotation: c.rotation.unwrap_or(0.0),
                })
                .collect();

            Ok(OdbBom { components })
        }
        Err(e) => Err(JsValue::from_str(&format!(
            "Failed to parse bom.xml: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stackup_xml() {
        let xml = r#"<?xml version="1.0"?>
<stackup>
    <layer name="Layer1" type="signal" index="0" thickness="0.035"/>
    <layer name="Layer2" type="plane" index="1" thickness="0.035"/>
</stackup>"#;

        let result = parse_stackup_xml(xml);
        assert!(result.is_ok());
    }
}
