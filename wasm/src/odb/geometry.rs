/// ODB++ to GerberData geometry conversion
/// 
/// Converts ODB++ geometry primitives to the internal GerberData representation
/// compatible with the existing renderer

use crate::odb::model::*;
use crate::shape::*;
use wasm_bindgen::prelude::*;

/// Convert ODB++ data to GerberData layers
pub fn convert_to_gerber_data(
    odb_data: &OdbData,
    offset_x: f32,
    offset_y: f32,
) -> Result<Vec<GerberData>, JsValue> {
    let mut gerber_layers = Vec::new();

    for odb_layer in &odb_data.file.layers {
        match convert_odb_layer_to_gerber(odb_layer, odb_data.unit_multiplier, offset_x, offset_y) {
            Ok(mut gerber_data) => {
                if gerber_data.has_geometry() {
                    gerber_layers.push(gerber_data);
                }
            }
            Err(e) => {
                // Log warning but continue processing other layers
                web_sys::console::warn_1(&format!("Failed to convert layer {}: {}", odb_layer.name, e).into());
            }
        }
    }

    if gerber_layers.is_empty() {
        return Err(JsValue::from_str(
            "ODB++ file contains no valid geometry data",
        ));
    }

    Ok(gerber_layers)
}

/// Convert individual ODB++ layer to GerberData
fn convert_odb_layer_to_gerber(
    odb_layer: &OdbLayer,
    unit_multiplier: f32,
    offset_x: f32,
    offset_y: f32,
) -> Result<GerberData, String> {
    let mut lines = Vec::new();
    let mut circles = Vec::new();
    let mut arcs = Vec::new();
    let mut boundary = Boundary::default();

    // Convert each geometry primitive
    for geometry in &odb_layer.geometries {
        match geometry {
            OdbGeometry::Line {
                x1,
                y1,
                x2,
                y2,
                width,
            } => {
                let x1_mm = (x1 * unit_multiplier) + offset_x;
                let y1_mm = (y1 * unit_multiplier) + offset_y;
                let x2_mm = (x2 * unit_multiplier) + offset_x;
                let y2_mm = (y2 * unit_multiplier) + offset_y;
                let width_mm = width * unit_multiplier;

                lines.push(Line {
                    x1: x1_mm,
                    y1: y1_mm,
                    x2: x2_mm,
                    y2: y2_mm,
                    width: width_mm,
                });

                // Update boundary
                boundary.expand(x1_mm, y1_mm);
                boundary.expand(x2_mm, y2_mm);
            }
            OdbGeometry::Circle {
                cx,
                cy,
                radius,
            } => {
                let cx_mm = (cx * unit_multiplier) + offset_x;
                let cy_mm = (cy * unit_multiplier) + offset_y;
                let radius_mm = radius * unit_multiplier;

                circles.push(Circle {
                    cx: cx_mm,
                    cy: cy_mm,
                    radius: radius_mm,
                });

                // Update boundary
                boundary.expand(cx_mm - radius_mm, cy_mm - radius_mm);
                boundary.expand(cx_mm + radius_mm, cy_mm + radius_mm);
            }
            OdbGeometry::Arc {
                x1,
                y1,
                x2,
                y2,
                cx,
                cy,
                radius,
                clockwise,
            } => {
                let x1_mm = (x1 * unit_multiplier) + offset_x;
                let y1_mm = (y1 * unit_multiplier) + offset_y;
                let x2_mm = (x2 * unit_multiplier) + offset_x;
                let y2_mm = (y2 * unit_multiplier) + offset_y;
                let cx_mm = (cx * unit_multiplier) + offset_x;
                let cy_mm = (cy * unit_multiplier) + offset_y;
                let radius_mm = radius * unit_multiplier;

                arcs.push(Arc {
                    x1: x1_mm,
                    y1: y1_mm,
                    x2: x2_mm,
                    y2: y2_mm,
                    cx: cx_mm,
                    cy: cy_mm,
                    radius: radius_mm,
                    clockwise: *clockwise,
                });

                // Update boundary
                boundary.expand(cx_mm - radius_mm, cy_mm - radius_mm);
                boundary.expand(cx_mm + radius_mm, cy_mm + radius_mm);
            }
            OdbGeometry::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                let x_mm = (x * unit_multiplier) + offset_x;
                let y_mm = (y * unit_multiplier) + offset_y;
                let w_mm = width * unit_multiplier;
                let h_mm = height * unit_multiplier;

                // Convert rectangle to polygon (for simplicity)
                let vertices = vec![
                    (x_mm, y_mm),
                    (x_mm + w_mm, y_mm),
                    (x_mm + w_mm, y_mm + h_mm),
                    (x_mm, y_mm + h_mm),
                ];

                // Update boundary
                boundary.expand(x_mm, y_mm);
                boundary.expand(x_mm + w_mm, y_mm + h_mm);
            }
            OdbGeometry::Polygon { vertices } => {
                for (px, py) in vertices {
                    let px_mm = (px * unit_multiplier) + offset_x;
                    let py_mm = (py * unit_multiplier) + offset_y;
                    boundary.expand(px_mm, py_mm);
                }
            }
        }
    }

    // Determine polarity for rendering
    let polarity_value = match odb_layer.polarity {
        Polarity::Positive => 1,
        Polarity::Negative => 0,
    };

    Ok(GerberData {
        lines,
        circles,
        arcs,
        thermals: Vec::new(),
        triangle_template_instances: Vec::new(),
        path_regions: Vec::new(),
        polarity: polarity_value,
        boundary,
    })
}

/// Helper structures for converted geometry
#[derive(Debug, Clone)]
pub struct Line {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub width: f32,
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct Arc {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
    pub clockwise: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_line_geometry() {
        let odb_geom = OdbGeometry::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            width: 0.5,
        };

        let odb_layer = OdbLayer {
            name: "TestLayer".to_string(),
            layer_type: "signal".to_string(),
            index: 0,
            geometries: vec![odb_geom],
            polarity: Polarity::Positive,
        };

        let odb_file = OdbFile {
            version: "1.0".to_string(),
            stackup: OdbStackup {
                layers: vec![],
            },
            layers: vec![odb_layer],
            netlist: None,
            bom: None,
        };

        let odb_data = OdbData::new(odb_file, 1.0);
        let result = convert_to_gerber_data(&odb_data, 0.0, 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_with_offset() {
        let odb_geom = OdbGeometry::Circle {
            cx: 5.0,
            cy: 5.0,
            radius: 2.0,
        };

        let odb_layer = OdbLayer {
            name: "TestLayer".to_string(),
            layer_type: "signal".to_string(),
            index: 0,
            geometries: vec![odb_geom],
            polarity: Polarity::Positive,
        };

        let odb_file = OdbFile {
            version: "1.0".to_string(),
            stackup: OdbStackup {
                layers: vec![],
            },
            layers: vec![odb_layer],
            netlist: None,
            bom: None,
        };

        let odb_data = OdbData::new(odb_file, 1.0);
        let result = convert_to_gerber_data(&odb_data, 10.0, 10.0);
        assert!(result.is_ok());
    }
}
