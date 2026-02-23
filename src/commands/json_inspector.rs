use serde_json::Value;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum FileKind {
    Json,
    GeoJson,
}

pub struct JsonInspector {
    pub root: Value,
    pub kind: FileKind,
}

impl JsonInspector {
    pub fn new(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let root: Value = serde_json::from_str(&content)?;
        let kind = detect_kind(path, &root);
        Ok(Self { root, kind })
    }

    pub fn geojson_summary(&self) -> (usize, Vec<String>, Option<(f64, f64, f64, f64)>) {
        let features = match self.root.get("features").and_then(|f| f.as_array()) {
            Some(f) => f,
            None => return (0, vec![], None),
        };
        let count = features.len();
        let mut geom_types: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let mut min_lon = f64::MAX;
        let mut min_lat = f64::MAX;
        let mut max_lon = f64::MIN;
        let mut max_lat = f64::MIN;
        let mut has_coords = false;
        for feature in features {
            if let Some(geom) = feature.get("geometry") {
                if let Some(t) = geom.get("type").and_then(|t| t.as_str()) {
                    geom_types.insert(t.to_string());
                }
                collect_bbox(geom, &mut min_lon, &mut min_lat, &mut max_lon, &mut max_lat, &mut has_coords);
            }
        }
        let bbox = if has_coords { Some((min_lon, min_lat, max_lon, max_lat)) } else { None };
        (count, geom_types.into_iter().collect(), bbox)
    }

    pub fn features_table(&self) -> (Vec<String>, Vec<Vec<String>>) {
        let features = match self.root.get("features").and_then(|f| f.as_array()) {
            Some(f) => f,
            None => return (vec![], vec![]),
        };
        let mut keys: Vec<String> = vec![];
        for feature in features {
            if let Some(props) = feature.get("properties").and_then(|p| p.as_object()) {
                for k in props.keys() {
                    if !keys.contains(k) {
                        keys.push(k.clone());
                    }
                }
            }
        }
        let rows: Vec<Vec<String>> = features
            .iter()
            .map(|f| {
                keys.iter()
                    .map(|k| {
                        f.get("properties")
                            .and_then(|p| p.get(k))
                            .map(|v| value_to_display(v))
                            .unwrap_or_else(|| "null".to_string())
                    })
                    .collect()
            })
            .collect();
        (keys, rows)
    }
}

fn detect_kind(path: &Path, root: &Value) -> FileKind {
    if path.extension().and_then(|e| e.to_str()) == Some("geojson") {
        return FileKind::GeoJson;
    }
    if let Some(t) = root.get("type").and_then(|t| t.as_str()) {
        if t == "FeatureCollection" || t == "Feature" {
            return FileKind::GeoJson;
        }
    }
    FileKind::Json
}

fn collect_bbox(geom: &Value, min_lon: &mut f64, min_lat: &mut f64, max_lon: &mut f64, max_lat: &mut f64, has_coords: &mut bool) {
    if let Some(coords) = geom.get("coordinates") {
        visit_coords(coords, min_lon, min_lat, max_lon, max_lat, has_coords);
    }
}

fn visit_coords(v: &Value, min_lon: &mut f64, min_lat: &mut f64, max_lon: &mut f64, max_lat: &mut f64, has_coords: &mut bool) {
    match v {
        Value::Array(arr) => {
            if arr.len() >= 2 && arr[0].is_number() && arr[1].is_number() {
                if let (Some(lon), Some(lat)) = (arr[0].as_f64(), arr[1].as_f64()) {
                    *has_coords = true;
                    if lon < *min_lon { *min_lon = lon; }
                    if lat < *min_lat { *min_lat = lat; }
                    if lon > *max_lon { *max_lon = lon; }
                    if lat > *max_lat { *max_lat = lat; }
                }
            } else {
                for item in arr {
                    visit_coords(item, min_lon, min_lat, max_lon, max_lat, has_coords);
                }
            }
        }
        _ => {}
    }
}

pub fn value_to_display(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(a) => format!("[{}]", a.len()),
        Value::Object(o) => format!("{{{}}}", o.len()),
    }
}
