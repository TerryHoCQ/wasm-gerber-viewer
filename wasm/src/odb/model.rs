/// ODB++ data model structures
/// 
/// Represents the structure and content of ODB++ format files

use serde::{Deserialize, Serialize};

/// Top-level ODB++ file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OdbFile {
    pub version: String,
    pub stackup: OdbStackup,
    pub layers: Vec<OdbLayer>,
    pub netlist: Option<OdbNetlist>,
    pub bom: Option<OdbBom>,
}

/// Stackup definition from stackup.xml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OdbStackup {
    pub layers: Vec<StackupLayer>,
}

/// Individual stackup layer definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackupLayer {
    pub name: String,
    pub layer_type: String, // "signal", "plane", "dielectric", "soldermask", etc.
    pub index: i32,
    pub thickness: Option<f32>, // in mm
}

/// ODB++ layer containing geometry data
#[derive(Debug, Clone)]
pub struct OdbLayer {
    pub name: String,
    pub layer_type: String,
    pub index: i32,
    pub geometries: Vec<OdbGeometry>,
    pub polarity: Polarity,
}

/// Polarity of layer (positive or negative)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Positive,
    Negative,
}

/// Basic geometry primitives in ODB++ format
#[derive(Debug, Clone)]
pub enum OdbGeometry {
    /// Line from (x1, y1) to (x2, y2) with width
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
    },
    /// Circle at (cx, cy) with radius
    Circle {
        cx: f32,
        cy: f32,
        radius: f32,
    },
    /// Polygon defined by vertices
    Polygon {
        vertices: Vec<(f32, f32)>,
    },
    /// Rectangle at (x, y) with width and height
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// Arc from (x1, y1) to (x2, y2) with center (cx, cy)
    Arc {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        cx: f32,
        cy: f32,
        radius: f32,
        clockwise: bool,
    },
}

/// Netlist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OdbNetlist {
    pub nets: Vec<Net>,
}

/// Individual net
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Net {
    pub name: String,
    pub connections: Vec<String>,
}

/// Bill of Materials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OdbBom {
    pub components: Vec<Component>,
}

/// Individual component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub reference: String,
    pub value: String,
    pub footprint: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

/// Parsed ODB++ data ready for conversion
#[derive(Debug)]
pub struct OdbData {
    pub file: OdbFile,
    pub unit_multiplier: f32, // Convert from ODB units to mm
}

impl OdbData {
    /// Create new ODB data with specified unit conversion
    pub fn new(file: OdbFile, unit_multiplier: f32) -> Self {
        OdbData {
            file,
            unit_multiplier,
        }
    }

    /// Get all layer names
    pub fn layer_names(&self) -> Vec<String> {
        self.file.layers.iter().map(|l| l.name.clone()).collect()
    }

    /// Find layer by name
    pub fn find_layer(&self, name: &str) -> Option<&OdbLayer> {
        self.file.layers.iter().find(|l| l.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polarity_equality() {
        assert_eq!(Polarity::Positive, Polarity::Positive);
        assert_ne!(Polarity::Positive, Polarity::Negative);
    }

    #[test]
    fn test_odb_data_layer_lookup() {
        let file = OdbFile {
            version: "1.0".to_string(),
            stackup: OdbStackup {
                layers: vec![],
            },
            layers: vec![OdbLayer {
                name: "Layer1".to_string(),
                layer_type: "signal".to_string(),
                index: 0,
                geometries: vec![],
                polarity: Polarity::Positive,
            }],
            netlist: None,
            bom: None,
        };
        let data = OdbData::new(file, 1.0);
        assert!(data.find_layer("Layer1").is_some());
        assert!(data.find_layer("NonExistent").is_none());
    }
}
