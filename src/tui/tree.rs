use serde_json::Value;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub depth: usize,
    pub key: Option<String>,
    pub kind: NodeKind,
    pub collapsed: bool,
    pub child_count: usize,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Object,
    Array,
    Scalar(String, ScalarType),
}

#[derive(Debug, Clone)]
pub enum ScalarType {
    String,
    Number,
    Bool,
    Null,
}

pub fn build_tree(root: &Value, collapsed: &std::collections::HashSet<String>) -> Vec<(String, TreeNode)> {
    let mut result = Vec::new();
    visit_value(root, None, "", 0, collapsed, &mut result);
    result
}

fn visit_value(
    v: &Value,
    key: Option<String>,
    path: &str,
    depth: usize,
    collapsed: &std::collections::HashSet<String>,
    result: &mut Vec<(String, TreeNode)>,
) {
    match v {
        Value::Object(map) => {
            let is_collapsed = collapsed.contains(path);
            result.push((path.to_string(), TreeNode {
                depth,
                key: key.clone(),
                kind: NodeKind::Object,
                collapsed: is_collapsed,
                child_count: map.len(),
            }));
            if !is_collapsed {
                for (k, val) in map {
                    let child_path = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", path, k)
                    };
                    visit_value(val, Some(k.clone()), &child_path, depth + 1, collapsed, result);
                }
            }
        }
        Value::Array(arr) => {
            let is_collapsed = collapsed.contains(path);
            result.push((path.to_string(), TreeNode {
                depth,
                key: key.clone(),
                kind: NodeKind::Array,
                collapsed: is_collapsed,
                child_count: arr.len(),
            }));
            if !is_collapsed {
                for (i, val) in arr.iter().enumerate() {
                    let child_path = format!("{}[{}]", path, i);
                    visit_value(val, Some(i.to_string()), &child_path, depth + 1, collapsed, result);
                }
            }
        }
        Value::Null => {
            result.push((path.to_string(), TreeNode {
                depth,
                key,
                kind: NodeKind::Scalar("null".to_string(), ScalarType::Null),
                collapsed: false,
                child_count: 0,
            }));
        }
        Value::Bool(b) => {
            result.push((path.to_string(), TreeNode {
                depth,
                key,
                kind: NodeKind::Scalar(b.to_string(), ScalarType::Bool),
                collapsed: false,
                child_count: 0,
            }));
        }
        Value::Number(n) => {
            result.push((path.to_string(), TreeNode {
                depth,
                key,
                kind: NodeKind::Scalar(n.to_string(), ScalarType::Number),
                collapsed: false,
                child_count: 0,
            }));
        }
        Value::String(s) => {
            result.push((path.to_string(), TreeNode {
                depth,
                key,
                kind: NodeKind::Scalar(s.clone(), ScalarType::String),
                collapsed: false,
                child_count: 0,
            }));
        }
    }
}
