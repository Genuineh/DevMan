//! Output parsing for quality check validation.
//!
//! This module provides parsers for extracting and validating command output
//! using various strategies: regex, JSON path, line contains, and custom scripts.

use devman_core::{OutputParser, MetricExtractor, Metric};
use regex::Regex;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Result of parsing command output.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Extracted values (name -> value)
    pub values: HashMap<String, String>,
    /// Whether parsing was successful
    pub success: bool,
    /// Error message if parsing failed
    pub error: Option<String>,
}

impl ParseResult {
    /// Create a successful parse result.
    pub fn success(values: HashMap<String, String>) -> Self {
        Self {
            values,
            success: true,
            error: None,
        }
    }

    /// Create a failed parse result.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            values: HashMap::new(),
            success: false,
            error: Some(error.into()),
        }
    }

    /// Get a value by name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(|v| v.as_str())
    }

    /// Get a value as a boolean.
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(|v| match v.to_lowercase().as_str() {
            "true" | "1" | "yes" | "pass" => Some(true),
            "false" | "0" | "no" | "fail" => Some(false),
            _ => None,
        })
    }

    /// Get a value as a float.
    pub fn get_float(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(|v| v.parse().ok())
    }

    /// Get a value as an integer.
    pub fn get_int(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(|v| v.parse().ok())
    }
}

/// Parse output using the specified parser.
pub fn parse_output(output: &str, parser: &OutputParser) -> ParseResult {
    match parser {
        OutputParser::LineContains { text } => parse_line_contains(output, text),
        OutputParser::Regex { pattern } => parse_regex(output, pattern),
        OutputParser::JsonPath { path } => parse_jsonpath(output, path),
        OutputParser::Custom { script: _ } => {
            ParseResult::failure("Custom script parsing not yet implemented")
        }
    }
}

/// Parse output checking if it contains a specific line.
fn parse_line_contains(output: &str, text: &str) -> ParseResult {
    let mut values = HashMap::new();
    let contains = output.lines().any(|line| line.contains(text));

    values.insert("contains".to_string(), contains.to_string());

    if contains {
        ParseResult::success(values)
    } else {
        ParseResult::failure(format!("Output does not contain '{}'", text))
    }
}

/// Parse output using regex.
fn parse_regex(output: &str, pattern: &str) -> ParseResult {
    match Regex::new(pattern) {
        Ok(re) => {
            let mut values = HashMap::new();

            // Check if the pattern has any named capture groups
            let has_named_captures = re.capture_names().flatten().next().is_some();

            if has_named_captures {
                // Try to find named capture groups
                if let Some(captures) = re.captures(output) {
                    for name in re.capture_names().flatten() {
                        if let Some(value) = captures.name(name) {
                            values.insert(name.to_string(), value.as_str().to_string());
                        }
                    }
                    ParseResult::success(values)
                } else {
                    ParseResult::failure(format!("Output does not match pattern '{}'", pattern))
                }
            } else {
                // Check if pattern matches at all (for simple validation)
                if re.is_match(output) {
                    values.insert("match".to_string(), "true".to_string());
                    ParseResult::success(values)
                } else {
                    ParseResult::failure(format!("Output does not match pattern '{}'", pattern))
                }
            }
        }
        Err(e) => ParseResult::failure(format!("Invalid regex pattern: {}", e)),
    }
}

/// Parse JSON output using JSONPath-like syntax.
///
/// Supports simple dot notation and array indexing:
/// - `field` - root field
/// - `field.nested` - nested field
/// - `array[0]` - array element
/// - `field.array[0].nested` - combined
fn parse_jsonpath(output: &str, path: &str) -> ParseResult {
    // First try to parse the output as JSON
    let json: JsonValue = match serde_json::from_str(output) {
        Ok(json) => json,
        Err(e) => {
            return ParseResult::failure(format!("Output is not valid JSON: {}", e));
        }
    };

    // Navigate the path
    let result = navigate_jsonpath(&json, path);

    match result {
        Some(value) => {
            let mut values = HashMap::new();
            let value_str = json_value_to_string(&value);
            values.insert("value".to_string(), value_str.clone());
            values.insert(path.to_string(), value_str);
            ParseResult::success(values)
        }
        None => ParseResult::failure(format!("Path '{}' not found in JSON output", path)),
    }
}

/// Navigate a JSON structure using a JSONPath-like expression.
fn navigate_jsonpath(json: &JsonValue, path: &str) -> Option<JsonValue> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;

    for part in parts {
        if part.is_empty() {
            continue;
        }

        // Handle array indexing (e.g., "items[0]")
        if let Some(bracket_pos) = part.find('[') {
            let key = &part[..bracket_pos];
            let index_part = &part[bracket_pos + 1..part.len() - 1]; // Remove [ and ]

            if !key.is_empty() {
                // First navigate to the object field
                current = current.get(key)?;
            }

            // Then navigate to the array index
            let index: usize = index_part.parse().ok()?;
            current = current.get(index)?;
        } else {
            // Simple field access
            current = current.get(part)?;
        }
    }

    Some(current.clone())
}

/// Convert a JSON value to string.
fn json_value_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::String(s) => s.clone(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Null => "null".to_string(),
        JsonValue::Array(_) | JsonValue::Object(_) => value.to_string(),
    }
}

/// Evaluate a pass condition against parsed values.
///
/// Supports simple expressions:
/// - `true` - always pass
/// - `false` - always fail
/// - `value == "expected"` - string equality
/// - `value != "expected"` - string inequality
/// - `value > 10` - numeric comparison
/// - `value >= 10`, `<`, `<=` - more comparisons
/// - `contains == "true"` - check if contains was true
/// - `match == "true"` - check if regex matched
pub fn evaluate_pass_condition(condition: &str, values: &ParseResult) -> bool {
    let condition = condition.trim();

    // Handle simple boolean literals
    match condition {
        "true" => return true,
        "false" => return false,
        _ => {}
    }

    // Find the operator (check for two-char operators first)
    let (var, op, expected_value) = if let Some(idx) = condition.find("==") {
        (&condition[..idx], "==", condition[idx + 2..].trim())
    } else if let Some(idx) = condition.find("!=") {
        (&condition[..idx], "!=", condition[idx + 2..].trim())
    } else if let Some(idx) = condition.find(">=") {
        (&condition[..idx], ">=", condition[idx + 2..].trim())
    } else if let Some(idx) = condition.find("<=") {
        (&condition[..idx], "<=", condition[idx + 2..].trim())
    } else if let Some(idx) = condition.find('>') {
        (&condition[..idx], ">", condition[idx + 1..].trim())
    } else if let Some(idx) = condition.find('<') {
        (&condition[..idx], "<", condition[idx + 1..].trim())
    } else {
        // No operator found - check if condition is a single variable name
        return if let Some(value) = values.get(condition) {
            match value.to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                _ => !value.is_empty(),
            }
        } else {
            false
        };
    };

    let var = var.trim();

    // Get the actual value
    let actual_str = values.get(var).map(|s| s.to_string()).unwrap_or_default();
    let actual = actual_str.as_str();

    match op {
        "==" => actual == expected_value,
        "!=" => actual != expected_value,
        ">" => {
            if let (Some(a), Some(b)) = (actual.parse::<f64>().ok(), expected_value.parse::<f64>().ok()) {
                a > b
            } else {
                actual > expected_value
            }
        }
        ">=" => {
            if let (Some(a), Some(b)) = (actual.parse::<f64>().ok(), expected_value.parse::<f64>().ok()) {
                a >= b
            } else {
                actual >= expected_value
            }
        }
        "<" => {
            if let (Some(a), Some(b)) = (actual.parse::<f64>().ok(), expected_value.parse::<f64>().ok()) {
                a < b
            } else {
                actual < expected_value
            }
        }
        "<=" => {
            if let (Some(a), Some(b)) = (actual.parse::<f64>().ok(), expected_value.parse::<f64>().ok()) {
                a <= b
            } else {
                actual <= expected_value
            }
        }
        _ => false,
    }
}

/// Extract metrics from output using metric extractors.
pub fn extract_metrics(
    output: &str,
    extractors: &[MetricExtractor],
) -> Vec<Metric> {
    let mut metrics = Vec::new();

    for extractor in extractors {
        let result = parse_output(output, &extractor.extractor);

        if let Some(value) = result.get("value").or_else(|| result.get(&extractor.name)) {
            if let Some(float_val) = value.parse::<f64>().ok() {
                metrics.push(Metric {
                    name: extractor.name.clone(),
                    value: float_val,
                    unit: extractor.unit.clone(),
                });
            }
        }
    }

    metrics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line_contains_success() {
        let output = "Build succeeded\nAll tests passed";
        let result = parse_line_contains(output, "succeeded");
        assert!(result.success);
        assert_eq!(result.get_bool("contains"), Some(true));
    }

    #[test]
    fn test_parse_line_contains_failure() {
        let output = "Build succeeded\nAll tests passed";
        let result = parse_line_contains(output, "failed");
        assert!(!result.success);
    }

    #[test]
    fn test_parse_regex_with_named_captures() {
        let output = "Coverage: 85.5%";
        let result = parse_regex(output, r"Coverage: (?P<coverage>[0-9.]+)%");
        assert!(result.success);
        assert_eq!(result.get("coverage"), Some("85.5"));
    }

    #[test]
    fn test_parse_regex_simple_match() {
        let output = "Build succeeded";
        let result = parse_regex(output, r"succeeded");
        assert!(result.success);
        assert_eq!(result.get("match"), Some("true"));
    }

    #[test]
    fn test_parse_regex_no_match() {
        let output = "Build succeeded";
        let result = parse_regex(output, r"failed");
        assert!(!result.success);
    }

    #[test]
    fn test_parse_jsonpath_root_field() {
        let output = r#"{"status": "passed", "coverage": 85.5}"#;
        let result = parse_jsonpath(output, "status");
        assert!(result.success);
        assert_eq!(result.get("value"), Some("passed"));
    }

    #[test]
    fn test_parse_jsonpath_nested() {
        let output = r#"{"result": {"status": "passed", "coverage": 85.5}}"#;
        let result = parse_jsonpath(output, "result.status");
        assert!(result.success);
        assert_eq!(result.get("value"), Some("passed"));
    }

    #[test]
    fn test_parse_jsonpath_array() {
        let output = r#"{"items": [{"name": "first"}, {"name": "second"}]}"#;
        let result = parse_jsonpath(output, "items[0].name");
        assert!(result.success);
        assert_eq!(result.get("value"), Some("first"));
    }

    #[test]
    fn test_parse_jsonpath_not_found() {
        let output = r#"{"status": "passed"}"#;
        let result = parse_jsonpath(output, "nonexistent");
        assert!(!result.success);
    }

    #[test]
    fn test_parse_jsonpath_invalid_json() {
        let output = "not valid json";
        let result = parse_jsonpath(output, "status");
        assert!(!result.success);
    }

    #[test]
    fn test_evaluate_pass_condition_true() {
        let values = ParseResult::success(HashMap::new());
        assert!(evaluate_pass_condition("true", &values));
    }

    #[test]
    fn test_evaluate_pass_condition_false() {
        let values = ParseResult::success(HashMap::new());
        assert!(!evaluate_pass_condition("false", &values));
    }

    #[test]
    fn test_evaluate_pass_condition_equality() {
        let mut values = HashMap::new();
        values.insert("status".to_string(), "passed".to_string());
        let parse_result = ParseResult::success(values);

        assert!(evaluate_pass_condition("status == passed", &parse_result));
        assert!(!evaluate_pass_condition("status == failed", &parse_result));
    }

    #[test]
    fn test_evaluate_pass_condition_numeric() {
        let mut values = HashMap::new();
        values.insert("coverage".to_string(), "85.5".to_string());
        let parse_result = ParseResult::success(values);

        assert!(evaluate_pass_condition("coverage >= 80", &parse_result));
        assert!(evaluate_pass_condition("coverage > 85", &parse_result));
        assert!(!evaluate_pass_condition("coverage >= 90", &parse_result));
    }

    #[test]
    fn test_evaluate_pass_condition_variable_truthy() {
        let mut values = HashMap::new();
        values.insert("match".to_string(), "true".to_string());
        let parse_result = ParseResult::success(values);

        assert!(evaluate_pass_condition("match", &parse_result));
    }

    #[test]
    fn test_extract_metrics_from_regex() {
        let output = "Coverage: 85.5%";
        let extractors = vec![
            MetricExtractor {
                name: "coverage".to_string(),
                extractor: OutputParser::Regex {
                    pattern: r"(?P<value>[0-9.]+)".to_string(),
                },
                unit: Some("%".to_string()),
            },
        ];

        let metrics = extract_metrics(output, &extractors);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "coverage");
        assert!((metrics[0].value - 85.5).abs() < 0.01);
        assert_eq!(metrics[0].unit, Some("%".to_string()));
    }

    #[test]
    fn test_extract_metrics_from_jsonpath() {
        let output = r#"{"coverage": 85.5}"#;
        let extractors = vec![
            MetricExtractor {
                name: "coverage".to_string(),
                extractor: OutputParser::JsonPath {
                    path: "coverage".to_string(),
                },
                unit: Some("%".to_string()),
            },
        ];

        let metrics = extract_metrics(output, &extractors);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "coverage");
        assert!((metrics[0].value - 85.5).abs() < 0.01);
    }

    #[test]
    fn test_regex_multiline_with_captures() {
        let output = "Tests: 100 passed, 5 failed\nCoverage: 85.5%";
        let result = parse_regex(
            output,
            r"Tests: (?P<passed>\d+) passed, (?P<failed>\d+) failed"
        );
        assert!(result.success);
        assert_eq!(result.get("passed"), Some("100"));
        assert_eq!(result.get("failed"), Some("5"));
    }

    #[test]
    fn test_regex_multiline_dot_matches_newline() {
        let output = "Line 1\nLine 2\nLine 3";
        let result = parse_regex(output, r"(?m)Line (?P<num>\d)");
        assert!(result.success);
        // Note: (?m) makes ^ and $ match line boundaries, but . still doesn't match \n
        // The regex should match each line separately
    }

    #[test]
    fn test_jsonpath_number_value() {
        let output = r#"{"count": 42}"#;
        let result = parse_jsonpath(output, "count");
        assert!(result.success);
        assert_eq!(result.get("value"), Some("42"));
        assert_eq!(result.get_int("value"), Some(42));
    }

    #[test]
    fn test_jsonpath_boolean_value() {
        let output = r#"{"enabled": true}"#;
        let result = parse_jsonpath(output, "enabled");
        assert!(result.success);
        assert_eq!(result.get("value"), Some("true"));
        assert_eq!(result.get_bool("value"), Some(true));
    }

    #[test]
    fn test_jsonpath_nested_array() {
        let output = r#"{"data": {"items": [1, 2, 3]}}"#;
        let result = parse_jsonpath(output, "data.items[1]");
        assert!(result.success);
        assert_eq!(result.get("value"), Some("2")); // JSON values are converted to string
    }

    #[test]
    fn test_parse_result_getters() {
        let mut values = HashMap::new();
        values.insert("bool_val".to_string(), "yes".to_string());
        values.insert("int_val".to_string(), "42".to_string());
        values.insert("float_val".to_string(), "3.14".to_string());

        let result = ParseResult::success(values);

        assert_eq!(result.get_bool("bool_val"), Some(true));
        assert_eq!(result.get_int("int_val"), Some(42));
        assert_eq!(result.get_float("float_val"), Some(3.14));
        assert_eq!(result.get("nonexistent"), None);
    }
}
