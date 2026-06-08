#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    pub name: String,
    pub success: bool,
    pub output: String,
    pub cost_cycles: u64,
}

impl ToolResult {
    pub fn ok(name: &str, output: &str, cost: u64) -> Self {
        ToolResult {
            name: name.to_string(),
            success: true,
            output: output.to_string(),
            cost_cycles: cost,
        }
    }

    pub fn err(name: &str, error: &str, cost: u64) -> Self {
        ToolResult {
            name: name.to_string(),
            success: false,
            output: error.to_string(),
            cost_cycles: cost,
        }
    }
}
