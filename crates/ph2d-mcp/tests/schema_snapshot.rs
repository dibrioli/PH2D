//! Snapshot tests for `ph2d_mcp::Server` default tool catalog.
//!
//! `insta` é o framework canônico de snapshot testing em Rust. Cada
//! `assert_yaml_snapshot!` compara a saída atual contra um arquivo
//! `tests/snapshots/<name>.snap`. Se a estrutura mudar, o test falha
//! e o dev escolhe entre regenerar (`cargo insta accept`) ou ajustar.
//!
//! Esses snapshots formam a CONTRACT pública do MCP server. Mudança no
//! schema = breaking change = bump major.

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
