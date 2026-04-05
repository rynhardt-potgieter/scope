/// Integration tests for the scope-mcp binary.
///
/// These tests drive the MCP server over stdin/stdout using JSON-RPC
/// messages, verifying the protocol handshake, tool listing, and tool
/// execution.
use assert_cmd::Command;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, Stdio};
use std::time::Duration;

/// Wrapper that owns BufReader over stdout to avoid re-creating it per call.
struct McpSession {
    stdin: ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    _child: std::process::Child,
}

impl McpSession {
    fn start() -> Self {
        let bin = assert_cmd::cargo::cargo_bin("scope-mcp");
        let mut child = std::process::Command::new(bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to start scope-mcp");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        Self {
            stdin,
            reader,
            _child: child,
        }
    }

    fn send_recv(&mut self, msg: &Value) -> Option<Value> {
        writeln!(self.stdin, "{}", serde_json::to_string(msg).unwrap()).unwrap();
        self.stdin.flush().unwrap();

        let mut line = String::new();
        self.reader.read_line(&mut line).ok()?;

        if line.trim().is_empty() {
            return None;
        }
        serde_json::from_str(line.trim()).ok()
    }

    fn send_notif(&mut self, msg: &Value) {
        writeln!(self.stdin, "{}", serde_json::to_string(msg).unwrap()).unwrap();
        self.stdin.flush().unwrap();
        std::thread::sleep(Duration::from_millis(100));
    }

    fn initialize(&mut self) -> Value {
        let resp = self
            .send_recv(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": {"name": "test", "version": "0.1"}
                }
            }))
            .expect("initialize should return a response");

        self.send_notif(
            &serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        );

        resp
    }
}

#[test]
fn test_initialize() {
    let mut s = McpSession::start();
    let resp = s.initialize();

    assert!(resp["result"]["capabilities"]["tools"].is_object());
    assert!(resp["result"]["instructions"].is_string());
}

#[test]
fn test_tools_list_returns_12() {
    let mut s = McpSession::start();
    s.initialize();

    let resp = s
        .send_recv(
            &serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        )
        .expect("tools/list should return a response");

    let tools = resp["result"]["tools"]
        .as_array()
        .expect("tools should be an array");
    assert_eq!(tools.len(), 12, "expected 12 tools, got {}", tools.len());

    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for expected in &[
        "scope_status",
        "scope_map",
        "scope_sketch",
        "scope_summary",
        "scope_source",
        "scope_find",
        "scope_refs",
        "scope_callers",
        "scope_deps",
        "scope_diff",
        "scope_trace",
        "scope_flow",
    ] {
        assert!(names.contains(expected), "missing tool: {expected}");
    }

    for tool in tools {
        assert!(
            tool["description"].is_string(),
            "{} missing description",
            tool["name"]
        );
        assert_eq!(
            tool["inputSchema"]["type"], "object",
            "{} inputSchema.type != object",
            tool["name"]
        );
    }
}

#[test]
fn test_tool_call_status() {
    let mut s = McpSession::start();
    s.initialize();

    let resp = s
        .send_recv(&serde_json::json!({
            "jsonrpc": "2.0", "id": 3,
            "method": "tools/call",
            "params": {"name": "scope_status", "arguments": {}}
        }))
        .expect("scope_status should return a response");

    let result = &resp["result"];
    assert_eq!(result["isError"], false);
    let text = result["content"][0]["text"].as_str().unwrap();
    let data: Value = serde_json::from_str(text).expect("content should be valid JSON");
    assert!(data["symbol_count"].is_number());
    assert!(data["file_count"].is_number());
}

#[test]
fn test_tool_call_unknown_symbol() {
    let mut s = McpSession::start();
    s.initialize();

    let resp = s
        .send_recv(&serde_json::json!({
            "jsonrpc": "2.0", "id": 4,
            "method": "tools/call",
            "params": {"name": "scope_source", "arguments": {"symbol": "NonExistent12345"}}
        }))
        .expect("should return a response");

    assert_eq!(resp["result"]["isError"], true);
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("not found") || lower.contains("no index"),
        "error should mention 'not found' or 'no index': {text}"
    );
}

#[test]
fn test_tool_call_unknown_tool() {
    let mut s = McpSession::start();
    s.initialize();

    let resp = s
        .send_recv(&serde_json::json!({
            "jsonrpc": "2.0", "id": 5,
            "method": "tools/call",
            "params": {"name": "totally_fake_tool", "arguments": {}}
        }))
        .expect("should return a response");

    assert!(
        resp["error"].is_object(),
        "expected JSON-RPC error for unknown tool"
    );
}
