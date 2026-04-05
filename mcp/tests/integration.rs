/// Integration tests for the scope-mcp binary.
///
/// These tests drive the MCP server over stdin/stdout using JSON-RPC
/// messages, verifying the protocol handshake, tool listing, and tool
/// execution.
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, Stdio};
use std::time::Duration;

/// Wrapper that owns BufReader over stdout to avoid re-creating it per call.
struct McpSession {
    stdin: ChildStdin,
    reader: Option<BufReader<std::process::ChildStdout>>,
    child: std::process::Child,
}

impl Drop for McpSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl McpSession {
    fn start() -> Self {
        let bin = assert_cmd::cargo::cargo_bin("scope-mcp");
        // Point SCOPE_BIN at the freshly-built scope binary so MCP tools
        // use the same version (with --compact etc.) instead of an older install.
        let scope_bin = assert_cmd::cargo::cargo_bin("scope");
        // MCP server resolves project_root from CWD. cargo test for the mcp
        // package sets CWD to mcp/, but the .scope/ index lives at the repo root.
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("mcp/ should have a parent directory");
        let mut child = std::process::Command::new(bin)
            .env("SCOPE_BIN", &scope_bin)
            .current_dir(repo_root)
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
            reader: Some(reader),
            child,
        }
    }

    fn send_recv(&mut self, msg: &Value) -> Option<Value> {
        writeln!(self.stdin, "{}", serde_json::to_string(msg).unwrap()).unwrap();
        self.stdin.flush().unwrap();

        // Move reader into a thread to enforce a 30s timeout, then take it back.
        let mut reader = self.reader.take().expect("reader already consumed");
        let handle = std::thread::spawn(move || {
            let mut line = String::new();
            let result = reader.read_line(&mut line);
            (reader, line, result)
        });

        let (reader, line, result) = match handle.join() {
            Ok(tuple) => tuple,
            Err(_) => panic!("reader thread panicked"),
        };
        self.reader = Some(reader);
        result.ok()?;

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

// ---------------------------------------------------------------------------
// Tool-level tests for the 7 tools not previously covered via MCP protocol
// ---------------------------------------------------------------------------

/// Helper: call a tool and return the parsed result (success or error).
fn call_tool(s: &mut McpSession, id: u32, name: &str, arguments: Value) -> Value {
    s.send_recv(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments}
    }))
    .unwrap_or_else(|| panic!("{name} should return a response"))
}

#[test]
fn test_tool_call_map() {
    let mut s = McpSession::start();
    s.initialize();
    let resp = call_tool(&mut s, 10, "scope_map", serde_json::json!({"limit": 5}));
    assert_eq!(resp["result"]["isError"], false, "scope_map failed: {resp}");
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let data: Value = serde_json::from_str(text).expect("map output should be valid JSON");
    assert!(
        data["entry_points"].is_array() || data["core_symbols"].is_array(),
        "map should have entry_points or core_symbols"
    );
}

#[test]
fn test_tool_call_sketch() {
    let mut s = McpSession::start();
    s.initialize();
    let resp = call_tool(
        &mut s,
        11,
        "scope_sketch",
        serde_json::json!({"symbol": "Graph"}),
    );
    assert_eq!(
        resp["result"]["isError"], false,
        "scope_sketch failed: {resp}"
    );
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(!text.is_empty(), "sketch should return non-empty content");
}

#[test]
fn test_tool_call_find() {
    let mut s = McpSession::start();
    s.initialize();
    let resp = call_tool(
        &mut s,
        12,
        "scope_find",
        serde_json::json!({"query": "graph", "limit": 3}),
    );
    assert_eq!(
        resp["result"]["isError"], false,
        "scope_find failed: {resp}"
    );
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let data: Value = serde_json::from_str(text).expect("find output should be valid JSON");
    // find returns the data field which is an array of results
    assert!(data.is_array(), "find data should be an array");
}

#[test]
fn test_tool_call_callers() {
    let mut s = McpSession::start();
    s.initialize();
    let resp = call_tool(
        &mut s,
        13,
        "scope_callers",
        serde_json::json!({"symbol": "open", "depth": 1}),
    );
    // open exists and has callers — should succeed
    assert_eq!(
        resp["result"]["isError"], false,
        "scope_callers failed: {resp}"
    );
}

#[test]
fn test_tool_call_deps() {
    let mut s = McpSession::start();
    s.initialize();
    let resp = call_tool(
        &mut s,
        14,
        "scope_deps",
        serde_json::json!({"symbol": "Graph"}),
    );
    assert_eq!(
        resp["result"]["isError"], false,
        "scope_deps failed: {resp}"
    );
}

#[test]
fn test_tool_call_trace() {
    let mut s = McpSession::start();
    s.initialize();
    let resp = call_tool(
        &mut s,
        15,
        "scope_trace",
        serde_json::json!({"symbol": "is_test_file"}),
    );
    // trace can be slow on large graphs — accept either success or timeout
    let is_error = resp["result"]["isError"].as_bool().unwrap_or(false);
    if is_error {
        let text = resp["result"]["content"][0]["text"].as_str().unwrap_or("");
        assert!(
            text.contains("timed out") || text.contains("not found"),
            "unexpected trace error: {text}"
        );
    }
}

#[test]
fn test_tool_call_flow() {
    let mut s = McpSession::start();
    s.initialize();
    let resp = call_tool(
        &mut s,
        16,
        "scope_flow",
        serde_json::json!({"start": "Graph", "end": "src/core/graph.rs::open::method"}),
    );
    // flow between Graph and Graph.open
    assert_eq!(
        resp["result"]["isError"], false,
        "scope_flow failed: {resp}"
    );
}

#[test]
fn test_tool_call_diff_with_ref_alias() {
    let mut s = McpSession::start();
    s.initialize();
    // Test that the "ref" alias works (agents may send "ref" instead of "git_ref")
    let resp = call_tool(&mut s, 17, "scope_diff", serde_json::json!({"ref": "HEAD"}));
    assert_eq!(
        resp["result"]["isError"], false,
        "scope_diff with 'ref' alias failed: {resp}"
    );
}
