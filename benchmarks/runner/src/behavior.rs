use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;

use crate::agent::AgentAction;

/// Deserialize f64 that may be null (from JSON infinity → null round-trip).
fn deserialize_f64_or_null<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<f64>::deserialize(deserializer).map(|opt| opt.unwrap_or(f64::INFINITY))
}

/// Behavior metrics derived from an agent's action sequence during a benchmark run.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BehaviorMetrics {
    // Navigation efficiency
    pub actions_before_first_edit: u32,
    #[serde(default, deserialize_with = "deserialize_f64_or_null")]
    pub navigation_to_edit_ratio: f64,
    pub unique_files_read: u32,
    pub redundant_reads: u32,
    pub total_actions: u32,

    // Scope usage
    pub scope_commands_total: u32,
    pub scope_commands_before_first_edit: u32,
    pub scope_then_read_same_file: u32,
    pub scope_command_sequence: Vec<String>,

    // Tool overlap
    pub grep_after_scope_find: u32,
    pub callers_and_refs_same_symbol: bool,
}

/// Aggregated comparison of behavior metrics between with-scope and without-scope runs.
#[derive(Debug, Serialize)]
pub struct BehaviorComparison {
    pub mean_actions_before_edit_with: f64,
    pub mean_actions_before_edit_without: f64,
    pub mean_nav_ratio_with: f64,
    pub mean_nav_ratio_without: f64,
    pub mean_unique_reads_with: f64,
    pub mean_unique_reads_without: f64,
    pub mean_redundant_reads_with: f64,
    pub mean_redundant_reads_without: f64,
    pub mean_scope_commands: f64,
    pub scope_anti_patterns: ScopeAntiPatterns,
}

/// Counts of detected anti-patterns in scope command usage.
#[derive(Debug, Serialize)]
pub struct ScopeAntiPatterns {
    pub sketch_then_read_count: u32,
    pub grep_after_find_count: u32,
    pub callers_and_refs_count: u32,
}

/// Compute behavior metrics from a sequence of agent actions.
///
/// Analyzes the action log to derive navigation efficiency, scope usage patterns,
/// and tool overlap metrics.
pub fn compute_behavior_metrics(actions: &[AgentAction]) -> BehaviorMetrics {
    let total_actions = actions.len() as u32;

    // Find first edit index
    let first_edit_idx = actions.iter().position(|a| a.is_edit);

    // actions_before_first_edit
    let actions_before_first_edit = match first_edit_idx {
        Some(idx) => idx as u32,
        None => total_actions,
    };

    // navigation_to_edit_ratio
    let nav_count = actions.iter().filter(|a| a.is_navigation).count() as f64;
    let edit_count = actions.iter().filter(|a| a.is_edit).count() as f64;
    let navigation_to_edit_ratio = if edit_count == 0.0 {
        f64::INFINITY
    } else {
        nav_count / edit_count
    };

    // unique_files_read and redundant_reads
    let read_files: Vec<&str> = actions
        .iter()
        .filter(|a| a.tool_name == "Read")
        .map(|a| a.arguments_summary.as_str())
        .collect();
    let unique_read_set: HashSet<&str> = read_files.iter().copied().collect();
    let unique_files_read = unique_read_set.len() as u32;
    let redundant_reads = read_files.len() as u32 - unique_files_read;

    // scope_commands_total
    let scope_commands_total = actions.iter().filter(|a| a.is_scope_command).count() as u32;

    // scope_commands_before_first_edit
    let scope_commands_before_first_edit = match first_edit_idx {
        Some(idx) => actions[..idx].iter().filter(|a| a.is_scope_command).count() as u32,
        None => scope_commands_total,
    };

    // scope_then_read_same_file: detect anti-pattern where a scope command is followed
    // by a Read of a file mentioned in the scope command's arguments
    let scope_then_read_same_file = compute_scope_then_read(actions);

    // scope_command_sequence: extract the scope subcommand from each scope action
    let scope_command_sequence = actions
        .iter()
        .filter(|a| a.is_scope_command)
        .filter_map(|a| extract_scope_subcommand(&a.arguments_summary))
        .collect();

    // grep_after_scope_find
    let grep_after_scope_find = compute_grep_after_scope_find(actions);

    // callers_and_refs_same_symbol
    let callers_and_refs_same_symbol = compute_callers_and_refs_overlap(actions);

    BehaviorMetrics {
        actions_before_first_edit,
        navigation_to_edit_ratio,
        unique_files_read,
        redundant_reads,
        total_actions,
        scope_commands_total,
        scope_commands_before_first_edit,
        scope_then_read_same_file,
        scope_command_sequence,
        grep_after_scope_find,
        callers_and_refs_same_symbol,
    }
}

/// Aggregate behavior metrics from with-scope and without-scope runs into a comparison.
pub fn aggregate_behavior(
    with_scope: &[BehaviorMetrics],
    without_scope: &[BehaviorMetrics],
) -> BehaviorComparison {
    let mean_actions_before_edit_with = mean_u32(with_scope, |m| m.actions_before_first_edit);
    let mean_actions_before_edit_without = mean_u32(without_scope, |m| m.actions_before_first_edit);

    let mean_nav_ratio_with = mean_f64_finite(with_scope, |m| m.navigation_to_edit_ratio);
    let mean_nav_ratio_without = mean_f64_finite(without_scope, |m| m.navigation_to_edit_ratio);

    let mean_unique_reads_with = mean_u32(with_scope, |m| m.unique_files_read);
    let mean_unique_reads_without = mean_u32(without_scope, |m| m.unique_files_read);

    let mean_redundant_reads_with = mean_u32(with_scope, |m| m.redundant_reads);
    let mean_redundant_reads_without = mean_u32(without_scope, |m| m.redundant_reads);

    let mean_scope_commands = mean_u32(with_scope, |m| m.scope_commands_total);

    let sketch_then_read_count: u32 = with_scope.iter().map(|m| m.scope_then_read_same_file).sum();
    let grep_after_find_count: u32 = with_scope.iter().map(|m| m.grep_after_scope_find).sum();
    let callers_and_refs_count: u32 = with_scope
        .iter()
        .filter(|m| m.callers_and_refs_same_symbol)
        .count() as u32;

    BehaviorComparison {
        mean_actions_before_edit_with,
        mean_actions_before_edit_without,
        mean_nav_ratio_with,
        mean_nav_ratio_without,
        mean_unique_reads_with,
        mean_unique_reads_without,
        mean_redundant_reads_with,
        mean_redundant_reads_without,
        mean_scope_commands,
        scope_anti_patterns: ScopeAntiPatterns {
            sketch_then_read_count,
            grep_after_find_count,
            callers_and_refs_count,
        },
    }
}

/// Generate a markdown section summarizing the behavior comparison.
pub fn format_behavior_markdown(comparison: &BehaviorComparison) -> String {
    let mut out = String::new();

    out.push_str("### Agent Behavior Analysis\n\n");

    out.push_str("#### Navigation Efficiency\n");
    out.push_str("| Metric | With Scope | Without Scope |\n");
    out.push_str("|--------|-----------|---------------|\n");
    out.push_str(&format!(
        "| Actions before first edit | {:.1} | {:.1} |\n",
        comparison.mean_actions_before_edit_with, comparison.mean_actions_before_edit_without
    ));
    out.push_str(&format!(
        "| Navigation:edit ratio | {:.1} | {:.1} |\n",
        comparison.mean_nav_ratio_with, comparison.mean_nav_ratio_without
    ));
    out.push_str(&format!(
        "| Unique files read | {:.1} | {:.1} |\n",
        comparison.mean_unique_reads_with, comparison.mean_unique_reads_without
    ));
    out.push_str(&format!(
        "| Redundant file reads | {:.1} | {:.1} |\n",
        comparison.mean_redundant_reads_with, comparison.mean_redundant_reads_without
    ));

    out.push_str("\n#### Scope Anti-Patterns Detected\n");
    out.push_str("| Pattern | Count |\n");
    out.push_str("|---------|-------|\n");
    out.push_str(&format!(
        "| Sketch then read same file | {} |\n",
        comparison.scope_anti_patterns.sketch_then_read_count
    ));
    out.push_str(&format!(
        "| Grep after scope find | {} |\n",
        comparison.scope_anti_patterns.grep_after_find_count
    ));
    out.push_str(&format!(
        "| callers + refs same symbol | {} |\n",
        comparison.scope_anti_patterns.callers_and_refs_count
    ));

    out.push_str(&format!(
        "\n#### Scope Command Usage\n- Mean scope commands per task: {:.1}\n",
        comparison.mean_scope_commands
    ));

    out
}

/// Generate data-driven CLI recommendations based on behavior analysis.
pub fn generate_recommendations(
    comparison: &BehaviorComparison,
    scope_command_sequences: &[Vec<String>],
) -> String {
    let mut recommendations: Vec<String> = Vec::new();

    // Check if scope refs is never used but scope callers is
    let has_callers = scope_command_sequences
        .iter()
        .flatten()
        .any(|cmd| cmd.starts_with("scope callers"));
    let has_refs = scope_command_sequences
        .iter()
        .flatten()
        .any(|cmd| cmd.starts_with("scope refs"));

    if has_callers && !has_refs {
        recommendations.push(
            "**`scope refs` never used** \u{2014} agents use `scope callers` exclusively. \
             Consider merging refs into callers or improving refs discoverability."
                .to_string(),
        );
    }

    if comparison.scope_anti_patterns.sketch_then_read_count > 0 {
        recommendations.push(format!(
            "**Sketch-then-read anti-pattern detected {} time(s)** \u{2014} \
             agents read files already summarized by sketch. \
             Strengthen guidance that sketch output replaces full file reads.",
            comparison.scope_anti_patterns.sketch_then_read_count
        ));
    }

    if comparison.scope_anti_patterns.grep_after_find_count > 0 {
        recommendations.push(format!(
            "**Grep after scope find detected {} time(s)** \u{2014} \
             agents grep for the same thing scope find already answered. \
             Improve scope find output to include enough context.",
            comparison.scope_anti_patterns.grep_after_find_count
        ));
    }

    if comparison.mean_scope_commands > 5.0 {
        recommendations.push(format!(
            "**High scope command usage ({:.1} per task)** \u{2014} \
             agents may be over-relying on scope. \
             Consider enforcing a command limit or improving per-command output density.",
            comparison.mean_scope_commands
        ));
    }

    if comparison.mean_actions_before_edit_with > comparison.mean_actions_before_edit_without {
        recommendations.push(format!(
            "**Over-navigation with scope ({:.1} vs {:.1} actions before first edit)** \u{2014} \
             scope may be encouraging exploration over action. \
             Review whether scope output is actionable enough.",
            comparison.mean_actions_before_edit_with, comparison.mean_actions_before_edit_without
        ));
    }

    let mut out = String::new();
    out.push_str("### CLI Recommendations\n\n");

    if recommendations.is_empty() {
        out.push_str("No actionable recommendations from current data.\n");
    } else {
        out.push_str("Based on agent behavior data:\n\n");
        for (i, rec) in recommendations.iter().enumerate() {
            out.push_str(&format!("{}. {}\n", i + 1, rec));
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Extract the scope subcommand from an arguments_summary string.
///
/// E.g. "scope sketch PaymentService" -> "scope sketch"
fn extract_scope_subcommand(arguments_summary: &str) -> Option<String> {
    let trimmed = arguments_summary.trim();
    // Find "scope " anywhere in the command (handles "cd /path && scope find ...")
    let scope_idx = trimmed.find("scope ")?;
    let from_scope = &trimmed[scope_idx..];
    let parts: Vec<&str> = from_scope.splitn(3, ' ').collect();
    if parts.len() >= 2 {
        Some(format!("{} {}", parts[0], parts[1]))
    } else {
        Some(from_scope.to_string())
    }
}

/// Detect the anti-pattern where a scope command is followed by a Read of a file
/// mentioned in the scope command's arguments_summary.
fn compute_scope_then_read(actions: &[AgentAction]) -> u32 {
    let mut count = 0u32;
    for (i, action) in actions.iter().enumerate() {
        if !action.is_scope_command {
            continue;
        }
        // Check subsequent Read actions for file paths appearing in this scope command's args
        for later in actions.iter().skip(i + 1) {
            if later.tool_name == "Read"
                && action.arguments_summary.contains(&later.arguments_summary)
            {
                count += 1;
                break;
            }
            // Only look at the immediate following actions (stop at the next non-Read, non-nav)
            if later.is_edit {
                break;
            }
        }
    }
    count
}

/// Count Grep actions that follow a "scope find" action within 3 actions.
fn compute_grep_after_scope_find(actions: &[AgentAction]) -> u32 {
    let mut count = 0u32;
    for (i, action) in actions.iter().enumerate() {
        if !action.is_scope_command {
            continue;
        }
        if !action.arguments_summary.contains("scope find") {
            continue;
        }
        // Look at the next 3 actions for a Grep
        let end = (i + 4).min(actions.len());
        for later in &actions[i + 1..end] {
            if later.tool_name == "Grep" {
                count += 1;
                break;
            }
        }
    }
    count
}

/// Check if both "scope callers X" and "scope refs X" appear for the same symbol X.
fn compute_callers_and_refs_overlap(actions: &[AgentAction]) -> bool {
    let mut callers_symbols: HashSet<String> = HashSet::new();
    let mut refs_symbols: HashSet<String> = HashSet::new();

    for action in actions {
        if !action.is_scope_command {
            continue;
        }
        let summary = action.arguments_summary.trim();
        if let Some(symbol) = summary.strip_prefix("scope callers ") {
            callers_symbols.insert(symbol.trim().to_string());
        } else if let Some(symbol) = summary.strip_prefix("scope refs ") {
            refs_symbols.insert(symbol.trim().to_string());
        }
    }

    callers_symbols.intersection(&refs_symbols).next().is_some()
}

/// Compute the mean of a u32 field across a slice of BehaviorMetrics.
fn mean_u32(metrics: &[BehaviorMetrics], f: fn(&BehaviorMetrics) -> u32) -> f64 {
    if metrics.is_empty() {
        return 0.0;
    }
    let total: u64 = metrics.iter().map(|m| f(m) as u64).sum();
    total as f64 / metrics.len() as f64
}

/// Compute the mean of an f64 field, treating INFINITY values as the max finite value
/// seen in the set (to avoid poisoning the mean).
fn mean_f64_finite(metrics: &[BehaviorMetrics], f: fn(&BehaviorMetrics) -> f64) -> f64 {
    if metrics.is_empty() {
        return 0.0;
    }
    let values: Vec<f64> = metrics.iter().map(f).collect();
    let finite_values: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();

    if finite_values.is_empty() {
        return f64::INFINITY;
    }

    // Replace INFINITY with the max finite value for a meaningful mean
    let max_finite = finite_values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let adjusted: Vec<f64> = values
        .iter()
        .map(|v| if v.is_finite() { *v } else { max_finite })
        .collect();

    adjusted.iter().sum::<f64>() / adjusted.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentAction;

    fn make_action(
        sequence: u32,
        tool_name: &str,
        arguments_summary: &str,
        is_navigation: bool,
        is_scope_command: bool,
        is_edit: bool,
    ) -> AgentAction {
        AgentAction {
            sequence,
            tool_name: tool_name.to_string(),
            arguments_summary: arguments_summary.to_string(),
            is_navigation,
            is_scope_command,
            is_edit,
        }
    }

    #[test]
    fn test_empty_actions() {
        let metrics = compute_behavior_metrics(&[]);
        assert_eq!(metrics.total_actions, 0);
        assert_eq!(metrics.actions_before_first_edit, 0);
        assert!(metrics.navigation_to_edit_ratio.is_infinite());
        assert_eq!(metrics.unique_files_read, 0);
        assert_eq!(metrics.redundant_reads, 0);
    }

    #[test]
    fn test_actions_before_first_edit() {
        let actions = vec![
            make_action(1, "Read", "src/main.rs", true, false, false),
            make_action(2, "Bash", "scope sketch Foo", true, true, false),
            make_action(3, "Read", "src/foo.rs", true, false, false),
            make_action(4, "Edit", "src/foo.rs", false, false, true),
        ];
        let metrics = compute_behavior_metrics(&actions);
        assert_eq!(metrics.actions_before_first_edit, 3);
        assert_eq!(metrics.total_actions, 4);
    }

    #[test]
    fn test_no_edits() {
        let actions = vec![
            make_action(1, "Read", "src/main.rs", true, false, false),
            make_action(2, "Read", "src/lib.rs", true, false, false),
        ];
        let metrics = compute_behavior_metrics(&actions);
        assert_eq!(metrics.actions_before_first_edit, 2);
        assert!(metrics.navigation_to_edit_ratio.is_infinite());
    }

    #[test]
    fn test_navigation_to_edit_ratio() {
        let actions = vec![
            make_action(1, "Read", "a.rs", true, false, false),
            make_action(2, "Read", "b.rs", true, false, false),
            make_action(3, "Edit", "a.rs", false, false, true),
        ];
        let metrics = compute_behavior_metrics(&actions);
        assert!((metrics.navigation_to_edit_ratio - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_unique_and_redundant_reads() {
        let actions = vec![
            make_action(1, "Read", "src/a.rs", true, false, false),
            make_action(2, "Read", "src/b.rs", true, false, false),
            make_action(3, "Read", "src/a.rs", true, false, false),
            make_action(4, "Read", "src/a.rs", true, false, false),
        ];
        let metrics = compute_behavior_metrics(&actions);
        assert_eq!(metrics.unique_files_read, 2);
        assert_eq!(metrics.redundant_reads, 2);
    }

    #[test]
    fn test_scope_command_sequence() {
        let actions = vec![
            make_action(1, "Bash", "scope sketch PaymentService", true, true, false),
            make_action(2, "Bash", "scope refs processPayment", true, true, false),
            make_action(3, "Bash", "scope find \"retry logic\"", true, true, false),
        ];
        let metrics = compute_behavior_metrics(&actions);
        assert_eq!(metrics.scope_commands_total, 3);
        assert_eq!(
            metrics.scope_command_sequence,
            vec!["scope sketch", "scope refs", "scope find"]
        );
    }

    #[test]
    fn test_grep_after_scope_find() {
        let actions = vec![
            make_action(1, "Bash", "scope find \"retry\"", true, true, false),
            make_action(2, "Read", "src/a.rs", true, false, false),
            make_action(3, "Grep", "retry", true, false, false),
        ];
        let metrics = compute_behavior_metrics(&actions);
        assert_eq!(metrics.grep_after_scope_find, 1);
    }

    #[test]
    fn test_grep_after_scope_find_too_far() {
        let actions = vec![
            make_action(1, "Bash", "scope find \"retry\"", true, true, false),
            make_action(2, "Read", "a.rs", true, false, false),
            make_action(3, "Read", "b.rs", true, false, false),
            make_action(4, "Read", "c.rs", true, false, false),
            make_action(5, "Grep", "retry", true, false, false),
        ];
        let metrics = compute_behavior_metrics(&actions);
        assert_eq!(metrics.grep_after_scope_find, 0);
    }

    #[test]
    fn test_callers_and_refs_same_symbol() {
        let actions = vec![
            make_action(1, "Bash", "scope callers PaymentService", true, true, false),
            make_action(2, "Bash", "scope refs PaymentService", true, true, false),
        ];
        let metrics = compute_behavior_metrics(&actions);
        assert!(metrics.callers_and_refs_same_symbol);
    }

    #[test]
    fn test_callers_and_refs_different_symbols() {
        let actions = vec![
            make_action(1, "Bash", "scope callers PaymentService", true, true, false),
            make_action(2, "Bash", "scope refs OrderService", true, true, false),
        ];
        let metrics = compute_behavior_metrics(&actions);
        assert!(!metrics.callers_and_refs_same_symbol);
    }

    #[test]
    fn test_scope_then_read_same_file() {
        let actions = vec![
            make_action(
                1,
                "Bash",
                "scope sketch PaymentService src/payment.rs",
                true,
                true,
                false,
            ),
            make_action(2, "Read", "src/payment.rs", true, false, false),
        ];
        let metrics = compute_behavior_metrics(&actions);
        assert_eq!(metrics.scope_then_read_same_file, 1);
    }

    #[test]
    fn test_aggregate_behavior() {
        let with = vec![
            BehaviorMetrics {
                actions_before_first_edit: 4,
                navigation_to_edit_ratio: 2.0,
                unique_files_read: 3,
                redundant_reads: 1,
                total_actions: 6,
                scope_commands_total: 2,
                scope_commands_before_first_edit: 2,
                scope_then_read_same_file: 1,
                scope_command_sequence: vec!["scope sketch".to_string()],
                grep_after_scope_find: 0,
                callers_and_refs_same_symbol: false,
            },
            BehaviorMetrics {
                actions_before_first_edit: 6,
                navigation_to_edit_ratio: 4.0,
                unique_files_read: 5,
                redundant_reads: 3,
                total_actions: 10,
                scope_commands_total: 4,
                scope_commands_before_first_edit: 3,
                scope_then_read_same_file: 0,
                scope_command_sequence: vec!["scope refs".to_string()],
                grep_after_scope_find: 1,
                callers_and_refs_same_symbol: true,
            },
        ];

        let without = vec![BehaviorMetrics {
            actions_before_first_edit: 8,
            navigation_to_edit_ratio: 6.0,
            unique_files_read: 7,
            redundant_reads: 4,
            total_actions: 12,
            scope_commands_total: 0,
            scope_commands_before_first_edit: 0,
            scope_then_read_same_file: 0,
            scope_command_sequence: vec![],
            grep_after_scope_find: 0,
            callers_and_refs_same_symbol: false,
        }];

        let comparison = aggregate_behavior(&with, &without);
        assert!((comparison.mean_actions_before_edit_with - 5.0).abs() < f64::EPSILON);
        assert!((comparison.mean_actions_before_edit_without - 8.0).abs() < f64::EPSILON);
        assert!((comparison.mean_nav_ratio_with - 3.0).abs() < f64::EPSILON);
        assert!((comparison.mean_scope_commands - 3.0).abs() < f64::EPSILON);
        assert_eq!(comparison.scope_anti_patterns.sketch_then_read_count, 1);
        assert_eq!(comparison.scope_anti_patterns.grep_after_find_count, 1);
        assert_eq!(comparison.scope_anti_patterns.callers_and_refs_count, 1);
    }

    #[test]
    fn test_format_behavior_markdown_contains_sections() {
        let comparison = BehaviorComparison {
            mean_actions_before_edit_with: 5.0,
            mean_actions_before_edit_without: 8.0,
            mean_nav_ratio_with: 3.0,
            mean_nav_ratio_without: 6.0,
            mean_unique_reads_with: 4.0,
            mean_unique_reads_without: 7.0,
            mean_redundant_reads_with: 1.0,
            mean_redundant_reads_without: 4.0,
            mean_scope_commands: 3.0,
            scope_anti_patterns: ScopeAntiPatterns {
                sketch_then_read_count: 1,
                grep_after_find_count: 0,
                callers_and_refs_count: 0,
            },
        };

        let md = format_behavior_markdown(&comparison);
        assert!(md.contains("### Agent Behavior Analysis"));
        assert!(md.contains("#### Navigation Efficiency"));
        assert!(md.contains("#### Scope Anti-Patterns Detected"));
        assert!(md.contains("#### Scope Command Usage"));
        assert!(md.contains("5.0"));
        assert!(md.contains("8.0"));
    }

    #[test]
    fn test_generate_recommendations_over_navigation() {
        let comparison = BehaviorComparison {
            mean_actions_before_edit_with: 10.0,
            mean_actions_before_edit_without: 5.0,
            mean_nav_ratio_with: 3.0,
            mean_nav_ratio_without: 2.0,
            mean_unique_reads_with: 4.0,
            mean_unique_reads_without: 4.0,
            mean_redundant_reads_with: 1.0,
            mean_redundant_reads_without: 1.0,
            mean_scope_commands: 3.0,
            scope_anti_patterns: ScopeAntiPatterns {
                sketch_then_read_count: 0,
                grep_after_find_count: 0,
                callers_and_refs_count: 0,
            },
        };

        let recs = generate_recommendations(&comparison, &[]);
        assert!(recs.contains("Over-navigation"));
    }

    #[test]
    fn test_generate_recommendations_no_issues() {
        let comparison = BehaviorComparison {
            mean_actions_before_edit_with: 3.0,
            mean_actions_before_edit_without: 5.0,
            mean_nav_ratio_with: 2.0,
            mean_nav_ratio_without: 4.0,
            mean_unique_reads_with: 3.0,
            mean_unique_reads_without: 5.0,
            mean_redundant_reads_with: 0.0,
            mean_redundant_reads_without: 2.0,
            mean_scope_commands: 2.0,
            scope_anti_patterns: ScopeAntiPatterns {
                sketch_then_read_count: 0,
                grep_after_find_count: 0,
                callers_and_refs_count: 0,
            },
        };

        let recs = generate_recommendations(&comparison, &[]);
        assert!(recs.contains("No actionable recommendations"));
    }

    #[test]
    fn test_extract_scope_subcommand() {
        assert_eq!(
            extract_scope_subcommand("scope sketch PaymentService"),
            Some("scope sketch".to_string())
        );
        assert_eq!(
            extract_scope_subcommand("scope refs processPayment --json"),
            Some("scope refs".to_string())
        );
        assert_eq!(
            extract_scope_subcommand("scope find \"retry logic\""),
            Some("scope find".to_string())
        );
        assert_eq!(extract_scope_subcommand("grep something"), None);
        // cd && scope pattern (Windows agents prefix with cd to temp dir)
        assert_eq!(
            extract_scope_subcommand("cd \"C:\\Users\\tmp\\.tmp123\" && scope find \"payment decline\""),
            Some("scope find".to_string())
        );
        assert_eq!(
            extract_scope_subcommand("cd /tmp/work && scope sketch PaymentService"),
            Some("scope sketch".to_string())
        );
    }
}
