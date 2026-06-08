use crate::memory;
use crate::tools::result::ToolResult;

const COST: u64 = 1_000_000;

pub fn recall(key: &str) -> ToolResult {
    let value = memory::get_memory_value(key);
    ToolResult::ok("recall", &value, COST)
}

pub fn store(key: &str, value: &str) -> ToolResult {
    memory::set_memory_value(key, value);
    ToolResult::ok("store", "Value stored successfully", COST)
}

pub fn forget(key: &str) -> ToolResult {
    memory::delete_memory_value(key);
    ToolResult::ok("forget", "Value deleted if it existed", COST)
}