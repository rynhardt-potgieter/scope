/// Integration tests for the scope-mcp binary.
///
/// These tests drive the MCP server over stdin/stdout using JSON-RPC
/// messages, verifying the protocol handshake, tool listing, and tool
/// execution.
use assert_cmd::Command;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Stdio};
use std::time::Duration;

/// Helper: start scope-mcp and return the child process.
fn start_mcp() -> Child {
    let bin = assert_cmd::cargo::cargo_bin("scope-mcp");
    std::process::Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start scope-mcp")
}

/// Send a JSON-RPC message and read one response line.
fn send_recv(child: &mut Child, msg: &Value) -> Option<Value> {
    let stdin = child.stdin.as_mut().unwrap();
    writeln!(stdin, "{}", serde_json::to_string(msg).unwrap()).unwrap();
    stdin.flush().unwrap();

    let stdout = child.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    // Read with a timeout-ish approach: try to read, give up after short wait
    // (BufRead is blocking, but scope commands are fast)
    reader.read_line(&mut line).ok()?;

    if line.trim().is_empty() {
        return None;
    }
    serde_json::from_str(line.trim()).ok()
}

/// Send a notification (no response expected).
fn send_notif(child: &mut Child, msg: &Value) {
    let stdin = child.stdin.as_mut().unwrap();
    writeln!(stdin, "{}", serde_json::to_string(msg).unwrap()).unwrap();
    stdin.flush().unwrap();
    std::thread::sleep(Duration::from_millis(100));
}

fn initialize(child: &mut Child) -> Value {
    let resp = send_recv(
        child,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "0.1"}
            }
        }),
    )
    .expect("initialize should return a response");

    send_notif(
        child,
        &serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    );

    resp
}

#[test]
fn test_initialize() {
    let mut child = start_mcp();
    let resp = initialize(&mut child);

    assert!(resp["result"]["capabilities"]["tools"].is_object());
    assert!(resp["result"]["instructions"].is_string());

    child.kill().ok();
}

#[test]
fn test_tools_list_returns_12() {
    let mut child = start_mcp();
    initialize(&mut child);

    let resp = send_recv(
        &mut child,
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

    // Every tool should have a description and inputSchema
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

    child.kill().ok();
}

#[test]
fn test_tool_call_status() {
    let mut child = start_mcp();
    initialize(&mut child);

    let resp = send_recv(
        &mut child,
        &serde_json::json!({
            "jsonrpc": "2.0", "id": 3,
            "method": "tools/call",
            "params": {"name": "scope_status", "arguments": {}}
        }),
    )
    .expect("scope_status should return a response");

    let result = &resp["result"];
    assert_eq!(result["isError"], false);
    let text = result["content"][0]["text"].as_str().unwrap();
    let data: Value = serde_json::from_str(text).expect("content should be valid JSON");
    assert!(data["symbol_count"].is_number());
    assert!(data["file_count"].is_number());

    child.kill().ok();
}

#[test]
fn test_tool_call_unknown_symbol() {
    let mut child = start_mcp();
    initialize(&mut child);

    let resp = send_recv(
        &mut child,
        &serde_json::json!({
            "jsonrpc": "2.0", "id": 4,
            "method": "tools/call",
            "params": {"name": "scope_source", "arguments": {"symbol": "NonExistent12345"}}
        }),
    )
    .expect("should return a response");

    assert_eq!(resp["result"]["isError"], true);
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    // Might be "symbol not found" or "No index found" depending on whether
    // the test CWD has a scope index.
    let lower = text.to_lowercase();
    assert!(
        lower.contains("not found") || lower.contains("no index"),
        "error should mention 'not found' or 'no index': {text}"
    );

    child.kill().ok();
}

#[test]
fn test_tool_call_unknown_tool() {
    let mut child = start_mcp();
    initialize(&mut child);

    let resp = send_recv(
        &mut child,
        &serde_json::json!({
            "jsonrpc": "2.0", "id": 5,
            "method": "tools/call",
            "params": {"name": "totally_fake_tool", "arguments": {}}
        }),
    )
    .expect("should return a response");

    // Unknown tool should return a JSON-RPC error
    assert!(
        resp["error"].is_object(),
        "expected JSON-RPC error for unknown tool"
    );

    child.kill().ok();
}
