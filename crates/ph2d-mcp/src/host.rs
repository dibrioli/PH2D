//! Backend trait + reference in-memory implementation.

use serde_json::Value;
use std::collections::HashMap;

/// Bridge entre o MCP server e o estado real (World/MessageBus/AssetDb).
/// Real backends (S2/S3) implementarão sobre bevy_ecs World direto. O
/// `MemoryHost` abaixo cobre testes e o flow C6/C15 doc.
pub trait McpHost {
    fn spawn_entity(&mut self) -> u64;
    fn add_component(&mut self, entity: u64, name: &str, data: Value);
    fn get_component(&self, entity: u64, name: &str) -> Option<Value>;
    fn send_message(&mut self, target: u64, message: &str, payload: Value);
    /// Returns true if entity existed and was removed.
    fn delete_entity(&mut self, entity: u64) -> bool;
}

#[derive(Default)]
pub struct MemoryHost {
    next_entity: u64,
    components: HashMap<u64, HashMap<String, Value>>,
    message_log: Vec<(u64, String, Value)>,
}

impl MemoryHost {
    pub fn message_log(&self) -> &[(u64, String, Value)] {
        &self.message_log
    }
}

impl McpHost for MemoryHost {
    fn spawn_entity(&mut self) -> u64 {
        self.next_entity += 1;
        self.components.insert(self.next_entity, HashMap::new());
        self.next_entity
    }

    fn add_component(&mut self, entity: u64, name: &str, data: Value) {
        self.components.entry(entity).or_default().insert(name.to_owned(), data);
    }

    fn get_component(&self, entity: u64, name: &str) -> Option<Value> {
        self.components.get(&entity).and_then(|c| c.get(name).cloned())
    }

    fn send_message(&mut self, target: u64, message: &str, payload: Value) {
        self.message_log.push((target, message.to_owned(), payload));
    }

    fn delete_entity(&mut self, entity: u64) -> bool {
        self.components.remove(&entity).is_some()
    }
}
