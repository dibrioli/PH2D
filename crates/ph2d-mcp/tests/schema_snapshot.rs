//! Snapshot tests for `ph2d_mcp::Server` default tool catalog.
//!
//! `insta` is the canonical snapshot-testing framework for Rust. Each
//! `assert_yaml_snapshot!` compares the current output against a file
//! at `tests/snapshots/<name>.snap`. If the structure changes, the test
//! fails and the dev chooses between regenerating (`cargo insta accept`)
//! or fixing.
//!
//! These snapshots form the PUBLIC CONTRACT of the MCP server. A schema
//! change = breaking change = major version bump.

use ph2d_mcp::{Server, ToolSchema};

#[test]
fn default_tools_schema_snapshot() {
    let mut server = Server::new();
    server.register_default_tools();
    let mut tools: Vec<&ToolSchema> = server.list_tools();
    // Sort by name so snapshot is stable across HashMap iteration order.
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    insta::assert_yaml_snapshot!("default_tools", tools);
}

#[test]
fn destructive_tools_count_snapshot() {
    let mut server = Server::new();
    server.register_default_tools();
    let destructive: Vec<&str> = server
        .list_tools()
        .iter()
        .filter(|t| t.destructive)
        .map(|t| t.name.as_str())
        .collect();
    let mut sorted = destructive.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("destructive_tools", sorted);
}
