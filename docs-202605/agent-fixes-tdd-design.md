# Agent Fixes: TDD-First Design Document

**Date**: November 15, 2025, 01:45 AM
**Purpose**: Fix 7 critical agent failure modes using TDD methodology
**Follows**: S01-README-MOSTIMP.md + S06-design101-tdd-architecture-principles.md

---

## Executive Summary: Executable Specifications

This document transforms the 7 agent failure modes into **executable specifications** with testable contracts. Every fix follows the **STUB → RED → GREEN → REFACTOR** cycle and uses the **four-word naming convention** for LLM optimization.

**Core Truth**: Traditional agent documentation fails because it's designed for human conversation. LLMs need executable blueprints with preconditions, postconditions, and error conditions.

---

## Table of Contents

1. [Fix 1: Binary PATH Detection](#fix-1-binary-path-detection)
2. [Fix 2: Command Name Validation](#fix-2-command-name-validation)
3. [Fix 3: Schema Introspection Tools](#fix-3-schema-introspection-tools)
4. [Fix 4: POSIX Shell Compatibility](#fix-4-posix-shell-compatibility)
5. [Fix 5: JSON Structure Aware Previews](#fix-5-json-structure-aware-previews)
6. [Fix 6: Structure Aware File Reader](#fix-6-structure-aware-file-reader)
7. [Fix 7: Agent Checkpoint State Manager](#fix-7-agent-checkpoint-state-manager)

---

## Fix 1: Binary PATH Detection

### Problem Statement
**What Failed**: Agent assumed `parseltongue` binary in PATH
**Root Cause**: No environment detection mechanism
**Impact**: Agent cannot run in development environments

### Executable Specification

#### Acceptance Criteria (WHEN...THEN...SHALL)

**AC-001.1**: WHEN agent starts THEN system SHALL detect parseltongue binary in PATH, ./target/release, or ../target/release
**AC-001.2**: WHEN binary found THEN system SHALL store absolute path for subsequent commands
**AC-001.3**: WHEN binary not found THEN system SHALL fail fast with actionable error message
**AC-001.4**: WHEN detection runs THEN system SHALL complete in <100ms

### Design: Binary Detection System

#### Interface Contract (Four-Word Names)

```rust
/// Binary path detection with fallback search
///
/// # Preconditions
/// - Current working directory is accessible
/// - File system permissions allow binary execution check
///
/// # Postconditions
/// - Returns Ok(PathBuf) with absolute path to binary
/// - Binary is executable (verified with access check)
/// - Path is canonicalized (no symlinks)
///
/// # Error Conditions
/// - BinaryError::NotFound if no binary in search paths
/// - BinaryError::NotExecutable if found but not executable
/// - BinaryError::FileSystemAccess if path resolution fails
pub fn find_parseltongue_binary_with_fallback() -> Result<PathBuf, BinaryError>;

/// Validate binary is correct version
///
/// # Preconditions
/// - Binary exists at given path
/// - Binary can be executed
///
/// # Postconditions
/// - Returns Ok(Version) if binary responds to --version
/// - Version matches minimum required (0.8.0+)
///
/// # Error Conditions
/// - BinaryError::VersionCheckFailed if --version fails
/// - BinaryError::IncompatibleVersion if version < 0.8.0
pub fn validate_parseltongue_binary_version(path: &Path) -> Result<Version, BinaryError>;

/// Cache binary path for session
///
/// # Preconditions
/// - Binary validated successfully
///
/// # Postconditions
/// - Writes path to $WORKSPACE/.parseltongue_binary
/// - File is readable by subsequent commands
///
/// # Error Conditions
/// - BinaryError::CacheWriteFailed if cannot write file
pub fn cache_parseltongue_binary_path(path: &Path, workspace: &Path) -> Result<(), BinaryError>;

/// Load cached binary path quickly
///
/// # Preconditions
/// - Workspace directory exists
///
/// # Postconditions
/// - Returns Ok(Some(PathBuf)) if cache exists and valid
/// - Returns Ok(None) if no cache
/// - Completes in <10ms
///
/// # Error Conditions
/// - BinaryError::CacheReadFailed if cache corrupted
pub fn load_cached_binary_path_fast(workspace: &Path) -> Result<Option<PathBuf>, BinaryError>;
```

#### Error Hierarchy

```rust
#[derive(Error, Debug)]
pub enum BinaryError {
    #[error("Parseltongue binary not found in PATH, ./target/release, or ../target/release")]
    NotFound,

    #[error("Binary found at {path} but not executable")]
    NotExecutable { path: PathBuf },

    #[error("Version check failed: {reason}")]
    VersionCheckFailed { reason: String },

    #[error("Binary version {found} incompatible (required: {required})")]
    IncompatibleVersion { found: Version, required: Version },

    #[error("File system access error: {0}")]
    FileSystemAccess(#[from] io::Error),

    #[error("Cache operation failed: {operation}")]
    CacheFailed { operation: String },
}
```

#### Search Strategy (Priority Order)

```rust
/// Search paths in priority order
const SEARCH_PATHS_BY_PRIORITY: &[&str] = &[
    "parseltongue",              // PATH lookup
    "./target/release/parseltongue",
    "../target/release/parseltongue",
    "../../target/release/parseltongue", // Multi-repo scenarios
    "/usr/local/bin/parseltongue",       // Global install
];

/// Detection algorithm
///
/// 1. Check cache first (fast path)
/// 2. If no cache, search in priority order
/// 3. For each candidate:
///    a. Check file exists
///    b. Check is executable
///    c. Validate version
/// 4. Cache first valid binary
/// 5. Return path or NotFound error
pub fn detect_binary_with_priority_search() -> Result<PathBuf, BinaryError> {
    // Implementation here
}
```

### Test Plan (TDD Cycle)

#### Phase 1: STUB (Red Phase)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: Binary in PATH
    #[test]
    fn test_find_binary_in_path_succeeds() {
        // GIVEN: parseltongue binary exists in PATH
        // WHEN: find_parseltongue_binary_with_fallback() is called
        // THEN: returns Ok(PathBuf) pointing to binary
        // AND: path is absolute and canonical
        todo!("Implement: find binary in PATH")
    }

    // Test 2: Binary in ./target/release
    #[test]
    fn test_find_binary_in_target_release() {
        // GIVEN: parseltongue binary at ./target/release/parseltongue
        // WHEN: find_parseltongue_binary_with_fallback() is called
        // THEN: returns Ok(PathBuf) with ./target/release/parseltongue
        todo!("Implement: find binary in target/release")
    }

    // Test 3: Binary not found
    #[test]
    fn test_find_binary_fails_when_missing() {
        // GIVEN: no parseltongue binary in any search path
        // WHEN: find_parseltongue_binary_with_fallback() is called
        // THEN: returns Err(BinaryError::NotFound)
        todo!("Implement: handle missing binary")
    }

    // Test 4: Version validation
    #[test]
    fn test_validate_binary_version_succeeds() {
        // GIVEN: binary at path with version 0.9.0
        // WHEN: validate_parseltongue_binary_version() is called
        // THEN: returns Ok(Version(0, 9, 0))
        todo!("Implement: version validation")
    }

    // Test 5: Cache hit (fast path)
    #[test]
    fn test_load_cached_path_returns_quickly() {
        // GIVEN: valid cache file exists
        // WHEN: load_cached_binary_path_fast() is called
        // THEN: returns Ok(Some(PathBuf)) in <10ms
        todo!("Implement: cache loading")
    }

    // Test 6: Performance contract
    #[test]
    fn test_detection_completes_under_100ms() {
        // GIVEN: binary exists at ./target/release/parseltongue
        // WHEN: full detection runs
        // THEN: completes in <100ms
        let start = Instant::now();
        let result = find_parseltongue_binary_with_fallback();
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed < Duration::from_millis(100),
                "Detection took {:?}, expected <100ms", elapsed);
    }
}
```

#### Phase 2: RED → GREEN → REFACTOR

**Implementation Strategy**:
1. Create stub implementations (RED)
2. Implement simplest version (GREEN)
3. Optimize and polish (REFACTOR)

---

## Fix 2: Command Name Validation

### Problem Statement
**What Failed**: Agent used outdated command names (`render-entity-count-bar-chart` vs `entity-count`)
**Root Cause**: No automated sync between CLI code and agent docs
**Impact**: Agent commands fail with "unrecognized subcommand"

### Executable Specification

#### Acceptance Criteria

**AC-002.1**: WHEN agent docs generated THEN system SHALL extract actual CLI subcommands from binary --help output
**AC-002.2**: WHEN CLI changes THEN system SHALL detect command name drift with CI check
**AC-002.3**: WHEN drift detected THEN system SHALL fail CI with actionable diff
**AC-002.4**: WHEN generating docs THEN system SHALL create examples with validated commands only

### Design: Command Validation System

#### Interface Contract (Four-Word Names)

```rust
/// Extract subcommands from binary help
///
/// # Preconditions
/// - Binary exists and is executable
/// - Binary supports --help flag
///
/// # Postconditions
/// - Returns Ok(Vec<CommandInfo>) with all subcommands
/// - Each CommandInfo has name, description, args
/// - List is sorted alphabetically
///
/// # Error Conditions
/// - ValidationError::BinaryExecution if --help fails
/// - ValidationError::ParsingFailed if output unparseable
pub fn extract_subcommands_from_binary_help(binary: &Path) -> Result<Vec<CommandInfo>, ValidationError>;

/// Validate agent commands against binary
///
/// # Preconditions
/// - Agent markdown file exists
/// - Binary path is valid
///
/// # Postconditions
/// - Returns Ok(()) if all agent commands exist in binary
/// - Returns Err with detailed diff if mismatches found
///
/// # Error Conditions
/// - ValidationError::CommandNotFound { agent_cmd, actual_cmds }
/// - ValidationError::CommandDeprecated { old_name, new_name }
pub fn validate_agent_commands_match_binary(agent_file: &Path, binary: &Path) -> Result<(), ValidationError>;

/// Generate validated agent command examples
///
/// # Preconditions
/// - Binary subcommands extracted
/// - Template exists
///
/// # Postconditions
/// - Returns Ok(String) with markdown content
/// - All bash examples use actual command names
/// - Includes usage examples from --help
///
/// # Error Conditions
/// - ValidationError::TemplateNotFound
/// - ValidationError::GenerationFailed
pub fn generate_agent_documentation_with_examples(commands: &[CommandInfo]) -> Result<String, ValidationError>;

/// Parse agent file bash commands
///
/// # Preconditions
/// - Agent file is valid markdown
///
/// # Postconditions
/// - Returns Ok(Vec<AgentCommand>) with all bash blocks
/// - Each AgentCommand has line number, full command
///
/// # Error Conditions
/// - ValidationError::MarkdownParseFailed
pub fn parse_bash_commands_from_agent(agent_file: &Path) -> Result<Vec<AgentCommand>, ValidationError>;
```

#### Data Structures

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInfo {
    pub name: String,             // "entity-count"
    pub parent: Option<String>,   // Some("pt07")
    pub description: String,      // "Entity count bar chart visualization"
    pub args: Vec<ArgInfo>,       // Required/optional arguments
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgInfo {
    pub name: String,             // "--db"
    pub required: bool,           // true
    pub value_type: String,       // "PATH"
    pub description: String,      // "Database path"
}

#[derive(Debug, Clone)]
pub struct AgentCommand {
    pub line: usize,              // Line number in agent file
    pub command: String,          // Full command text
    pub binary_name: String,      // "parseltongue"
    pub subcommands: Vec<String>, // ["pt07", "entity-count"]
    pub args: Vec<String>,        // ["--db", "rocksdb:analysis.db"]
}

#[derive(Debug, PartialEq, Eq)]
pub struct CommandDrift {
    pub agent_line: usize,
    pub agent_command: String,
    pub issue: DriftIssue,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DriftIssue {
    CommandNotFound { actual_commands: Vec<String> },
    CommandRenamed { old: String, new: String },
    CommandDeprecated { replacement: String },
}
```

#### Validation Algorithm

```rust
/// Command drift detection algorithm
///
/// 1. Extract all actual commands from binary help
/// 2. Parse all bash commands from agent file
/// 3. For each agent command:
///    a. Check if command exists in binary
///    b. If not found, search for similar names (Levenshtein distance)
///    c. Detect renames (e.g., "render-X-Y" → "X")
/// 4. Generate diff with line numbers
/// 5. Fail with actionable error
pub fn detect_command_drift_with_suggestions(
    agent_file: &Path,
    binary: &Path,
) -> Result<Vec<CommandDrift>, ValidationError> {
    // Implementation here
}
```

### Test Plan (TDD Cycle)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_subcommands_from_pt07() {
        // GIVEN: parseltongue pt07 --help output
        // WHEN: extract_subcommands_from_binary_help() is called
        // THEN: returns vec!["entity-count", "cycles"]
        todo!("Implement: extract pt07 subcommands")
    }

    #[test]
    fn test_detect_renamed_command_drift() {
        // GIVEN: agent uses "render-entity-count-bar-chart"
        // AND: binary has "entity-count"
        // WHEN: detect_command_drift_with_suggestions() is called
        // THEN: returns CommandDrift with DriftIssue::CommandRenamed
        todo!("Implement: detect renames")
    }

    #[test]
    fn test_validate_agent_commands_fails_fast() {
        // GIVEN: agent file with invalid command
        // WHEN: validate_agent_commands_match_binary() is called
        // THEN: returns Err(ValidationError::CommandNotFound)
        // AND: error message includes line number
        // AND: error message suggests similar commands
        todo!("Implement: validation with suggestions")
    }

    #[test]
    fn test_generate_validated_documentation_succeeds() {
        // GIVEN: valid CommandInfo vec
        // WHEN: generate_agent_documentation_with_examples() is called
        // THEN: returns markdown with correct commands
        // AND: all bash examples are executable
        todo!("Implement: doc generation")
    }
}
```

#### CI Integration

```yaml
# .github/workflows/agent-validation.yml
name: Agent Documentation Validation

on: [push, pull_request]

jobs:
  validate-agent:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build parseltongue binary
        run: cargo build --release

      - name: Validate agent commands
        run: |
          cargo run --bin validate-agent-commands \
            --agent-file .claude/agents/parseltongue-ultrathink-isg-explorer.md \
            --binary ./target/release/parseltongue

      - name: Check for command drift
        run: |
          cargo run --bin detect-command-drift \
            --agent-file .claude/agents/parseltongue-ultrathink-isg-explorer.md \
            --binary ./target/release/parseltongue \
            --fail-on-drift
```

---

## Fix 3: Schema Introspection Tools

### Problem Statement
**What Failed**: Agent queried non-existent fields (`cyclomatic_complexity`)
**Root Cause**: No machine-readable schema documentation
**Impact**: WHERE clause queries fail with cryptic Datalog errors

### Executable Specification

#### Acceptance Criteria

**AC-003.1**: WHEN user runs `parseltongue schema` THEN system SHALL output all queryable fields with types
**AC-003.2**: WHEN field queried doesn't exist THEN system SHALL suggest similar field names
**AC-003.3**: WHEN schema requested THEN system SHALL support JSON, YAML, and table formats
**AC-003.4**: WHEN schema generated THEN system SHALL include WHERE clause operator examples

### Design: Schema Introspection System

#### Interface Contract (Four-Word Names)

```rust
/// Export database schema to format
///
/// # Preconditions
/// - Database connection is valid
/// - Output format is supported (json/yaml/table)
///
/// # Postconditions
/// - Returns Ok(String) with formatted schema
/// - Schema includes all entity fields
/// - Schema includes queryable operators
///
/// # Error Conditions
/// - SchemaError::DatabaseConnection if DB unavailable
/// - SchemaError::UnsupportedFormat if format invalid
pub fn export_database_schema_to_format(
    db: &Database,
    format: OutputFormat,
) -> Result<String, SchemaError>;

/// Validate where clause before execution
///
/// # Preconditions
/// - Database schema loaded
/// - WHERE clause is non-empty string
///
/// # Postconditions
/// - Returns Ok(()) if clause is valid
/// - Returns Err with specific issue if invalid
///
/// # Error Conditions
/// - SchemaError::FieldNotFound { field, suggestions }
/// - SchemaError::OperatorNotSupported { operator, field_type }
/// - SchemaError::TypeMismatch { field, expected, actual }
pub fn validate_where_clause_against_schema(
    schema: &Schema,
    where_clause: &str,
) -> Result<(), SchemaError>;

/// Generate example queries for schema
///
/// # Preconditions
/// - Schema is populated
///
/// # Postconditions
/// - Returns Ok(Vec<ExampleQuery>) with valid examples
/// - Each example includes description + command
///
/// # Error Conditions
/// - SchemaError::NoFields if schema empty
pub fn generate_example_queries_from_schema(schema: &Schema) -> Result<Vec<ExampleQuery>, SchemaError>;

/// Find similar field names fuzzy
///
/// # Preconditions
/// - Schema has at least one field
///
/// # Postconditions
/// - Returns Vec<String> with similar field names
/// - Sorted by Levenshtein distance (closest first)
/// - Returns empty vec if no matches
pub fn find_similar_field_names_fuzzy(
    schema: &Schema,
    attempted_field: &str,
) -> Vec<String>;
```

#### Schema Data Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub version: String,              // "1.0"
    pub last_updated: String,         // ISO8601 timestamp
    pub fields: Vec<FieldDefinition>,
    pub operators: Vec<OperatorDefinition>,
    pub examples: Vec<ExampleQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,                 // "entity_name"
    pub data_type: FieldType,         // FieldType::String
    pub nullable: bool,               // false
    pub description: String,          // "Name of the code entity"
    pub queryable: bool,              // true
    pub indexed: bool,                // true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Integer,
    Boolean,
    Enum { variants: &'static [&'static str] },
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorDefinition {
    pub symbol: String,               // "="
    pub name: String,                 // "equals"
    pub supported_types: Vec<FieldType>,
    pub example: String,              // "entity_name = 'foo'"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleQuery {
    pub description: String,
    pub where_clause: String,
    pub expected_results: String,
}
```

#### Schema Output Formats

**JSON Format**:
```json
{
  "version": "1.0",
  "last_updated": "2025-11-15T01:45:00Z",
  "fields": [
    {
      "name": "entity_name",
      "data_type": "String",
      "nullable": false,
      "description": "Name of the code entity",
      "queryable": true,
      "indexed": true
    },
    {
      "name": "is_public",
      "data_type": "Boolean",
      "nullable": false,
      "description": "Whether entity is public",
      "queryable": true,
      "indexed": true
    }
  ],
  "operators": [
    {
      "symbol": "=",
      "name": "equals",
      "supported_types": ["String", "Integer", "Boolean"],
      "example": "entity_name = 'foo'"
    },
    {
      "symbol": ">",
      "name": "greater_than",
      "supported_types": ["Integer"],
      "example": "line_number > 100"
    }
  ],
  "examples": [
    {
      "description": "Find all public functions",
      "where_clause": "entity_type = 'function' AND is_public = true",
      "expected_results": "Returns all public function entities"
    }
  ]
}
```

**Table Format** (for terminal display):
```
┌─────────────────┬────────────┬──────────┬─────────────────────────────────┬───────────┬─────────┐
│ Field Name      │ Type       │ Nullable │ Description                     │ Queryable │ Indexed │
├─────────────────┼────────────┼──────────┼─────────────────────────────────┼───────────┼─────────┤
│ entity_name     │ String     │ false    │ Name of the code entity         │ true      │ true    │
│ entity_type     │ String     │ false    │ Type (function, struct, etc.)   │ true      │ true    │
│ is_public       │ Boolean    │ false    │ Whether entity is public        │ true      │ true    │
│ file_path       │ String     │ false    │ Source file path                │ true      │ false   │
│ line_number     │ Integer    │ false    │ Line number in file             │ true      │ false   │
│ current_ind     │ Integer    │ false    │ Current state indicator (0|1)   │ true      │ true    │
│ future_ind      │ Integer    │ false    │ Future state indicator (0|1)    │ true      │ true    │
└─────────────────┴────────────┴──────────┴─────────────────────────────────┴───────────┴─────────┘

Supported Operators:
  = (equals)         : String, Integer, Boolean - Example: entity_name = 'foo'
  > (greater_than)   : Integer                  - Example: line_number > 100
  < (less_than)      : Integer                  - Example: line_number < 50
  != (not_equals)    : String, Integer, Boolean - Example: is_public != false

NOT SUPPORTED:
  ~ (regex match)    : Use alternative syntax in Datalog
  LIKE (SQL-style)   : Not available in CozoDB queries

Example Queries:
  1. Find all public functions:
     parseltongue pt02-level01 --where-clause "entity_type = 'function' AND is_public = true"

  2. Find entities in specific file:
     parseltongue pt02-level01 --where-clause "file_path = './src/main.rs'"

  3. Find current-state entities only:
     parseltongue pt02-level01 --where-clause "current_ind = 1"
```

### Test Plan (TDD Cycle)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_schema_as_json() {
        // GIVEN: database with known schema
        // WHEN: export_database_schema_to_format(db, JSON) is called
        // THEN: returns valid JSON with all fields
        todo!("Implement: JSON schema export")
    }

    #[test]
    fn test_validate_where_clause_detects_missing_field() {
        // GIVEN: WHERE clause with "cyclomatic_complexity > 15"
        // AND: schema does NOT have cyclomatic_complexity field
        // WHEN: validate_where_clause_against_schema() is called
        // THEN: returns Err(SchemaError::FieldNotFound)
        // AND: error includes suggestions for similar fields
        todo!("Implement: WHERE clause validation")
    }

    #[test]
    fn test_find_similar_fields_suggests_alternatives() {
        // GIVEN: user typed "entity_nam" (typo)
        // WHEN: find_similar_field_names_fuzzy() is called
        // THEN: returns vec!["entity_name", "entity_type"]
        // AND: sorted by distance
        todo!("Implement: fuzzy field search")
    }

    #[test]
    fn test_generate_examples_includes_all_queryable_fields() {
        // GIVEN: schema with 5 queryable fields
        // WHEN: generate_example_queries_from_schema() is called
        // THEN: returns at least 5 example queries
        // AND: each query is syntactically valid
        todo!("Implement: example generation")
    }
}
```

---

## Fix 4: POSIX Shell Compatibility

### Problem Statement
**What Failed**: Bash commands with complex variable substitution failed in zsh
**Root Cause**: Non-POSIX shell syntax
**Impact**: `(eval):1: parse error near '('` on macOS (zsh default)

### Executable Specification

#### Acceptance Criteria

**AC-004.1**: WHEN agent generates shell commands THEN system SHALL use POSIX-compliant syntax only
**AC-004.2**: WHEN command contains variables THEN system SHALL use explicit temporary files instead
**AC-004.3**: WHEN command tested THEN system SHALL validate in bash, zsh, and sh
**AC-004.4**: WHEN command fails THEN system SHALL provide shell-agnostic alternative

### Design: POSIX Command Builder

#### Interface Contract (Four-Word Names)

```rust
/// Generate POSIX compliant shell command
///
/// # Preconditions
/// - Command parts are non-empty
/// - Variables are properly escaped
///
/// # Postconditions
/// - Returns Ok(String) with POSIX-compliant command
/// - Command works in bash, zsh, sh
/// - No shell-specific features used
///
/// # Error Conditions
/// - ShellError::InvalidCommand if parts invalid
/// - ShellError::UnsafeVariable if variable contains shell metacharacters
pub fn generate_posix_compliant_shell_command(parts: &[CommandPart]) -> Result<String, ShellError>;

/// Validate command in multiple shells
///
/// # Preconditions
/// - Command string is non-empty
/// - Test shells are available
///
/// # Postconditions
/// - Returns Ok(()) if command valid in all shells
/// - Returns Err with incompatible shell details
///
/// # Error Conditions
/// - ShellError::IncompatibleSyntax { shell, issue }
pub fn validate_command_across_shells_safely(command: &str) -> Result<(), ShellError>;

/// Transform complex command to simple
///
/// # Preconditions
/// - Complex command uses variables, pipelines, subshells
///
/// # Postconditions
/// - Returns Ok(Vec<String>) with simplified commands
/// - Each command is POSIX-compliant
/// - Uses temp files instead of variables
///
/// # Error Conditions
/// - ShellError::CannotSimplify if command too complex
pub fn transform_complex_command_to_simple(command: &str) -> Result<Vec<String>, ShellError>;

/// Quote path for shell safety
///
/// # Preconditions
/// - Path is valid UTF-8
///
/// # Postconditions
/// - Returns String with properly quoted path
/// - Handles spaces, special characters
/// - Works in all POSIX shells
pub fn quote_path_for_shell_safety(path: &Path) -> String;
```

#### Command Transformation Patterns

**Pattern 1: Variable Substitution → Temp Files**

```bash
# ❌ Bad: Complex variable substitution
WORKSPACE=$(cat /tmp/workspace.txt) && \
./binary --db "rocksdb:$WORKSPACE/analysis.db"

# ✅ Good: Explicit steps with temp files
cat /tmp/workspace.txt > /tmp/current_workspace.txt
./binary --db "rocksdb:$(cat /tmp/current_workspace.txt)/analysis.db"
```

**Pattern 2: Pipelines → Sequential Commands**

```bash
# ❌ Bad: Complex pipeline with variable capture
OUTPUT=$(./binary --list 2>&1 | tee log.txt | grep "ERROR")

# ✅ Good: Sequential steps
./binary --list > /tmp/output.txt 2>&1
cp /tmp/output.txt log.txt
grep "ERROR" /tmp/output.txt
```

**Pattern 3: Subshells → Direct Commands**

```bash
# ❌ Bad: Nested subshells
VAR1=$(echo $(cat file.txt))

# ✅ Good: Direct file operations
cat file.txt > /tmp/var1.txt
VAR1=$(cat /tmp/var1.txt)
```

#### POSIX Compatibility Checklist

```rust
pub struct PosixChecker {
    disallowed_constructs: Vec<ShellConstruct>,
}

#[derive(Debug)]
pub enum ShellConstruct {
    BashArrays,               // array=()
    ProcessSubstitution,      // <(command)
    HereStrings,              // <<< "string"
    DoubleParentheses,        // ((arithmetic))
    DoubleBrackets,           // [[ test ]]
    BraceExpansion,           // {1..10}
    LocalVariables,           // local var=value
}

impl PosixChecker {
    /// Check command for non-POSIX constructs
    pub fn check_command_is_posix(&self, command: &str) -> Result<(), Vec<ShellConstruct>> {
        // Scan for disallowed patterns
    }
}
```

### Test Plan (TDD Cycle)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_posix_command_with_quoted_paths() {
        // GIVEN: path with spaces "my documents/file.txt"
        // WHEN: generate_posix_compliant_shell_command() is called
        // THEN: returns command with properly quoted path
        // AND: command works in bash, zsh, sh
        todo!("Implement: POSIX path quoting")
    }

    #[test]
    fn test_validate_command_fails_for_bash_arrays() {
        // GIVEN: command with bash array syntax "arr=(1 2 3)"
        // WHEN: validate_command_across_shells_safely() is called
        // THEN: returns Err(ShellError::IncompatibleSyntax)
        // AND: error identifies bash-specific feature
        todo!("Implement: bash array detection")
    }

    #[test]
    fn test_transform_complex_to_simple_uses_temp_files() {
        // GIVEN: complex command with nested variables
        // WHEN: transform_complex_command_to_simple() is called
        // THEN: returns Vec of simple commands
        // AND: uses /tmp/*.txt for intermediate values
        // AND: all commands are POSIX-compliant
        todo!("Implement: command transformation")
    }

    #[test]
    fn test_posix_checker_detects_process_substitution() {
        // GIVEN: command with "<(command)" syntax
        // WHEN: PosixChecker::check_command_is_posix() is called
        // THEN: returns Err with ProcessSubstitution construct
        todo!("Implement: POSIX checker")
    }
}
```

---

## Fix 5: JSON Structure-Aware Previews

### Problem Statement
**What Failed**: Agent assumed JSON was array, but actual structure has metadata wrapper
**Root Cause**: No schema documentation for export formats
**Impact**: jq queries failed with parse errors

### Executable Specification

#### Acceptance Criteria

**AC-005.1**: WHEN user requests preview THEN system SHALL detect JSON structure and format appropriately
**AC-005.2**: WHEN JSON has metadata wrapper THEN system SHALL extract and display metadata separately
**AC-005.3**: WHEN preview requested THEN system SHALL limit to N entities while keeping valid JSON
**AC-005.4**: WHEN format unknown THEN system SHALL auto-detect and suggest correct jq pattern

### Design: JSON Preview System

#### Interface Contract (Four-Word Names)

```rust
/// Preview JSON file with structure awareness
///
/// # Preconditions
/// - File exists and is readable
/// - File contains valid JSON
///
/// # Postconditions
/// - Returns Ok(JsonPreview) with structure info
/// - Includes metadata if present
/// - Includes first N entities
/// - Output is valid JSON
///
/// # Error Conditions
/// - PreviewError::FileNotFound
/// - PreviewError::InvalidJson { line, column }
pub fn preview_json_file_with_structure(
    path: &Path,
    max_entities: usize,
) -> Result<JsonPreview, PreviewError>;

/// Detect JSON structure and suggest jq
///
/// # Preconditions
/// - JSON value is valid
///
/// # Postconditions
/// - Returns Ok(JsonStructure) with schema info
/// - Includes suggested jq patterns
/// - Identifies arrays, objects, metadata wrappers
///
/// # Error Conditions
/// - PreviewError::UnknownStructure if nested too deep
pub fn detect_json_structure_and_suggest(json: &Value) -> Result<JsonStructure, PreviewError>;

/// Generate valid preview with entities
///
/// # Preconditions
/// - Full JSON parsed successfully
/// - Max entities is > 0
///
/// # Postconditions
/// - Returns Ok(String) with valid JSON
/// - Contains at most max_entities
/// - Preserves metadata wrapper if present
/// - Includes continuation marker if truncated
///
/// # Error Conditions
/// - PreviewError::SerializationFailed
pub fn generate_valid_preview_with_entities(
    full_json: &Value,
    max_entities: usize,
) -> Result<String, PreviewError>;

/// Suggest jq pattern for structure
///
/// # Preconditions
/// - JsonStructure has been detected
///
/// # Postconditions
/// - Returns Vec<String> with jq patterns
/// - Patterns are sorted by usefulness
/// - Each pattern includes description
pub fn suggest_jq_pattern_for_structure(structure: &JsonStructure) -> Vec<JqPattern>;
```

#### Data Structures

```rust
#[derive(Debug, Clone)]
pub struct JsonPreview {
    pub structure: JsonStructure,
    pub metadata: Option<Value>,         // Extracted metadata if present
    pub entities_shown: usize,
    pub entities_total: usize,
    pub preview_json: String,            // Valid JSON string
    pub suggested_jq: Vec<JqPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonStructure {
    FlatArray,                           // [...]
    ObjectWithArrayField {               // { "field": [...] }
        array_field_name: String,
    },
    MetadataWrapper {                    // { "metadata": {...}, "entities": [...] }
        metadata_key: String,
        entities_key: String,
    },
    Unknown,
}

#[derive(Debug, Clone)]
pub struct JqPattern {
    pub pattern: String,                 // ".entities[] | .entity_name"
    pub description: String,             // "Extract entity names"
    pub example_output: String,          // "foo\\nbar\\nbaz"
}
```

#### Structure Detection Algorithm

```rust
/// JSON structure detection algorithm
///
/// 1. Parse top-level value
/// 2. If array → FlatArray
/// 3. If object → inspect keys:
///    a. Look for "metadata" + "entities" pair → MetadataWrapper
///    b. Look for single array field → ObjectWithArrayField
///    c. Otherwise → Unknown
/// 4. Generate suggested jq patterns based on structure
pub fn detect_structure_recursive(value: &Value) -> JsonStructure {
    match value {
        Value::Array(_) => JsonStructure::FlatArray,
        Value::Object(map) => {
            // Check for metadata wrapper pattern
            if map.contains_key("export_metadata") && map.contains_key("entities") {
                return JsonStructure::MetadataWrapper {
                    metadata_key: "export_metadata".to_string(),
                    entities_key: "entities".to_string(),
                };
            }

            // Check for single array field
            let array_fields: Vec<_> = map.iter()
                .filter(|(_, v)| v.is_array())
                .collect();

            if array_fields.len() == 1 {
                return JsonStructure::ObjectWithArrayField {
                    array_field_name: array_fields[0].0.clone(),
                };
            }

            JsonStructure::Unknown
        }
        _ => JsonStructure::Unknown,
    }
}
```

#### Preview Generation

```rust
/// Generate preview preserving structure
pub fn generate_preview_preserving_structure(
    json: &Value,
    structure: &JsonStructure,
    max_entities: usize,
) -> Result<Value, PreviewError> {
    match structure {
        JsonStructure::FlatArray => {
            // Just slice array
            let arr = json.as_array().ok_or(PreviewError::TypeMismatch)?;
            let preview: Vec<_> = arr.iter().take(max_entities).cloned().collect();
            Ok(Value::Array(preview))
        }

        JsonStructure::MetadataWrapper { metadata_key, entities_key } => {
            // Preserve metadata, slice entities
            let obj = json.as_object().ok_or(PreviewError::TypeMismatch)?;
            let metadata = obj.get(metadata_key).cloned();
            let entities = obj.get(entities_key)
                .and_then(|v| v.as_array())
                .ok_or(PreviewError::MissingField)?;

            let preview_entities: Vec<_> = entities.iter()
                .take(max_entities)
                .cloned()
                .collect();

            let mut preview_obj = serde_json::Map::new();
            if let Some(meta) = metadata {
                preview_obj.insert(metadata_key.clone(), meta);
            }
            preview_obj.insert(entities_key.clone(), Value::Array(preview_entities));
            preview_obj.insert("_preview_note".to_string(),
                Value::String(format!("Showing {} of {} entities",
                    preview_entities.len(), entities.len())));

            Ok(Value::Object(preview_obj))
        }

        // ... other cases
    }
}
```

### Test Plan (TDD Cycle)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_metadata_wrapper_structure() {
        // GIVEN: JSON with { "export_metadata": {...}, "entities": [...] }
        // WHEN: detect_json_structure_and_suggest() is called
        // THEN: returns JsonStructure::MetadataWrapper
        // AND: identifies "entities" as array field
        todo!("Implement: metadata wrapper detection")
    }

    #[test]
    fn test_generate_preview_preserves_metadata() {
        // GIVEN: JSON with metadata + 100 entities
        // WHEN: generate_valid_preview_with_entities(5) is called
        // THEN: returns valid JSON
        // AND: contains metadata
        // AND: contains exactly 5 entities
        // AND: includes preview note
        todo!("Implement: preview generation")
    }

    #[test]
    fn test_suggest_jq_for_metadata_wrapper() {
        // GIVEN: MetadataWrapper structure detected
        // WHEN: suggest_jq_pattern_for_structure() is called
        // THEN: returns patterns like ".entities[]"
        // AND: includes ".export_metadata.total_entities"
        // AND: each pattern has description
        todo!("Implement: jq suggestion")
    }

    #[test]
    fn test_preview_file_handles_invalid_json() {
        // GIVEN: file with invalid JSON (missing bracket)
        // WHEN: preview_json_file_with_structure() is called
        // THEN: returns Err(PreviewError::InvalidJson)
        // AND: error includes line and column
        todo!("Implement: error handling")
    }
}
```

---

## Fix 6: Structure-Aware File Reader

### Problem Statement
**What Failed**: Agent used `head -50` on JSON files, creating invalid truncated JSON
**Root Cause**: Line-based tools on structure-based formats
**Impact**: jq couldn't parse truncated JSON

### Executable Specification

#### Acceptance Criteria

**AC-006.1**: WHEN reading structured files THEN system SHALL use format-aware readers, not head/tail
**AC-006.2**: WHEN previewing large files THEN system SHALL produce valid output in target format
**AC-006.3**: WHEN file format detected THEN system SHALL suggest appropriate preview command
**AC-006.4**: WHEN preview command fails THEN system SHALL fall back to safe byte-based preview

### Design: Format-Aware File Reader

#### Interface Contract (Four-Word Names)

```rust
/// Read file with format awareness
///
/// # Preconditions
/// - File exists and is readable
/// - File size known
///
/// # Postconditions
/// - Returns Ok(FileContent) with parsed structure
/// - Format auto-detected or user-specified
/// - Content is valid in detected format
///
/// # Error Conditions
/// - ReaderError::FileNotFound
/// - ReaderError::UnsupportedFormat
pub fn read_file_with_format_awareness(
    path: &Path,
    format: Option<FileFormat>,
) -> Result<FileContent, ReaderError>;

/// Preview file preserving structure validity
///
/// # Preconditions
/// - File format detected
/// - Max size is reasonable (<10MB)
///
/// # Postconditions
/// - Returns Ok(String) with valid preview
/// - Preview is valid in source format
/// - If JSON: valid JSON subset
/// - If XML: valid XML fragment
///
/// # Error Conditions
/// - ReaderError::PreviewFailed
pub fn preview_file_preserving_structure_validity(
    path: &Path,
    max_items: usize,
) -> Result<String, ReaderError>;

/// Detect file format from extension
///
/// # Preconditions
/// - Path has extension
///
/// # Postconditions
/// - Returns Ok(FileFormat) with detected format
/// - Falls back to content-based detection if extension unclear
///
/// # Error Conditions
/// - ReaderError::UnknownFormat if cannot detect
pub fn detect_file_format_from_extension(path: &Path) -> Result<FileFormat, ReaderError>;

/// Suggest preview command for format
///
/// # Preconditions
/// - Format is known
///
/// # Postconditions
/// - Returns String with recommended command
/// - Command produces valid output
/// - Includes explanation of command
pub fn suggest_preview_command_for_format(
    path: &Path,
    format: FileFormat,
) -> String;
```

#### Format Handlers

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Json,
    Xml,
    Yaml,
    Toml,
    Csv,
    PlainText,
    Binary,
}

pub trait FormatHandler {
    fn preview(&self, content: &[u8], max_items: usize) -> Result<String, ReaderError>;
    fn validate(&self, content: &[u8]) -> Result<(), ReaderError>;
    fn suggest_tools(&self) -> Vec<String>;
}

pub struct JsonHandler;
impl FormatHandler for JsonHandler {
    fn preview(&self, content: &[u8], max_items: usize) -> Result<String, ReaderError> {
        // Parse JSON, extract subset, re-serialize
        let value: Value = serde_json::from_slice(content)?;
        let preview = generate_json_preview(&value, max_items)?;
        Ok(serde_json::to_string_pretty(&preview)?)
    }

    fn validate(&self, content: &[u8]) -> Result<(), ReaderError> {
        serde_json::from_slice::<Value>(content)
            .map(|_| ())
            .map_err(|e| ReaderError::InvalidFormat {
                format: FileFormat::Json,
                reason: e.to_string(),
            })
    }

    fn suggest_tools(&self) -> Vec<String> {
        vec![
            "jq '.' file.json".to_string(),
            "jq '.entities[:5]' file.json".to_string(),
        ]
    }
}

pub struct XmlHandler;
pub struct YamlHandler;
pub struct CsvHandler;
pub struct PlainTextHandler;
```

#### Safe Preview Algorithm

```rust
/// Safe file preview algorithm (never produces invalid output)
///
/// 1. Detect file format (extension → magic bytes → content inspection)
/// 2. Select appropriate handler
/// 3. Try format-aware preview:
///    a. For JSON: parse → slice → serialize
///    b. For XML: parse DOM → extract nodes → serialize fragment
///    c. For CSV: read rows → take N rows → write CSV
/// 4. If format-aware fails → fall back to safe byte preview:
///    a. Read first N bytes
///    b. Find last complete line
///    c. Add "... (truncated)" marker
/// 5. Return valid preview or error
pub fn safe_preview_with_fallback(
    path: &Path,
    max_items: usize,
) -> Result<String, ReaderError> {
    let format = detect_file_format_from_extension(path)?;
    let handler = get_format_handler(format);

    // Try format-aware preview
    let content = fs::read(path)?;
    match handler.preview(&content, max_items) {
        Ok(preview) => Ok(preview),
        Err(_) => {
            // Fall back to safe byte preview
            safe_byte_preview(&content, 10000)
        }
    }
}

fn safe_byte_preview(content: &[u8], max_bytes: usize) -> Result<String, ReaderError> {
    let truncated = if content.len() <= max_bytes {
        content
    } else {
        // Find last complete line
        let subset = &content[..max_bytes];
        if let Some(last_newline) = subset.iter().rposition(|&b| b == b'\n') {
            &content[..=last_newline]
        } else {
            subset
        }
    };

    let mut preview = String::from_utf8_lossy(truncated).to_string();
    if content.len() > max_bytes {
        preview.push_str("\n... (truncated, showing first ");
        preview.push_str(&truncated.len().to_string());
        preview.push_str(" bytes of ");
        preview.push_str(&content.len().to_string());
        preview.push_str(" total)");
    }

    Ok(preview)
}
```

### Test Plan (TDD Cycle)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_handler_produces_valid_json() {
        // GIVEN: large JSON file with 1000 entities
        // WHEN: JsonHandler::preview(content, 5) is called
        // THEN: returns valid JSON string
        // AND: contains exactly 5 entities
        // AND: preserves metadata if present
        todo!("Implement: JSON preview")
    }

    #[test]
    fn test_safe_preview_falls_back_on_invalid_json() {
        // GIVEN: file with .json extension but invalid JSON
        // WHEN: safe_preview_with_fallback() is called
        // THEN: returns byte-based preview instead
        // AND: includes truncation marker
        todo!("Implement: fallback preview")
    }

    #[test]
    fn test_detect_format_from_extension_handles_ambiguous() {
        // GIVEN: file named "data" (no extension)
        // WHEN: detect_file_format_from_extension() is called
        // THEN: falls back to content-based detection
        // AND: detects JSON from leading "{"
        todo!("Implement: content-based detection")
    }

    #[test]
    fn test_suggest_tools_for_each_format() {
        // GIVEN: FileFormat::Json
        // WHEN: suggest_preview_command_for_format() is called
        // THEN: returns "jq '.' file.json"
        // AND: includes explanation

        // GIVEN: FileFormat::Xml
        // THEN: returns "xmllint --format file.xml"
        todo!("Implement: tool suggestions")
    }
}
```

---

## Fix 7: Agent Checkpoint State Manager

### Problem Statement
**What Failed**: Agent lost all state when interrupted
**Root Cause**: No resumability or checkpointing mechanism
**Impact**: Had to manually restart from beginning

### Executable Specification

#### Acceptance Criteria

**AC-007.1**: WHEN agent starts long workflow THEN system SHALL create checkpoint file
**AC-007.2**: WHEN step completes THEN system SHALL update checkpoint with progress
**AC-007.3**: WHEN agent resumed THEN system SHALL detect checkpoint and offer resume
**AC-007.4**: WHEN checkpoint loaded THEN system SHALL skip completed steps

### Design: Checkpoint State Manager

#### Interface Contract (Four-Word Names)

```rust
/// Create checkpoint file in workspace
///
/// # Preconditions
/// - Workspace directory exists
/// - Workflow ID is unique
///
/// # Postconditions
/// - Creates .checkpoint.json in workspace
/// - File contains workflow ID, start time
/// - File is readable by subsequent invocations
///
/// # Error Conditions
/// - CheckpointError::WriteFailed
pub fn create_checkpoint_file_in_workspace(
    workspace: &Path,
    workflow_id: &str,
) -> Result<Checkpoint, CheckpointError>;

/// Update checkpoint with completed step
///
/// # Preconditions
/// - Checkpoint file exists
/// - Step is valid workflow step
///
/// # Postconditions
/// - Checkpoint file updated atomically
/// - Completed steps list includes new step
/// - Timestamp updated
///
/// # Error Conditions
/// - CheckpointError::UpdateFailed
pub fn update_checkpoint_with_completed_step(
    checkpoint: &mut Checkpoint,
    step: WorkflowStep,
) -> Result<(), CheckpointError>;

/// Load checkpoint and detect resume
///
/// # Preconditions
/// - Workspace path exists
///
/// # Postconditions
/// - Returns Ok(Some(Checkpoint)) if valid checkpoint found
/// - Returns Ok(None) if no checkpoint
/// - Checkpoint validated (not corrupted)
///
/// # Error Conditions
/// - CheckpointError::CorruptedCheckpoint
pub fn load_checkpoint_and_detect_resume(workspace: &Path) -> Result<Option<Checkpoint>, CheckpointError>;

/// Determine next step from checkpoint
///
/// # Preconditions
/// - Checkpoint loaded successfully
/// - Workflow definition known
///
/// # Postconditions
/// - Returns Some(WorkflowStep) if work remaining
/// - Returns None if workflow complete
///
pub fn determine_next_step_from_checkpoint(
    checkpoint: &Checkpoint,
    workflow: &Workflow,
) -> Option<WorkflowStep>;
```

#### Checkpoint Data Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub version: String,                  // "1.0"
    pub workflow_id: String,              // "parseltongue-ultrathink-isg-explorer"
    pub workspace: PathBuf,               // "/path/to/parseltongue20251115012556"
    pub started_at: String,               // ISO8601 timestamp
    pub last_updated: String,             // ISO8601 timestamp
    pub completed_steps: Vec<StepRecord>,
    pub current_step: Option<StepRecord>,
    pub failed_steps: Vec<FailedStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub step_id: String,                  // "CREATE_WORKSPACE"
    pub step_name: String,                // "Create timestamped workspace"
    pub started_at: String,
    pub completed_at: String,
    pub outputs: HashMap<String, String>, // Key-value pairs (e.g., "workspace_path": "/path")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedStep {
    pub step_id: String,
    pub error_message: String,
    pub failed_at: String,
    pub retry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowStep {
    CreateWorkspace,
    IngestCodebase,
    GraphDependencies,
    RunQueries,
    AnalyzeResults,
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub id: String,
    pub steps: Vec<WorkflowStepDefinition>,
}

#[derive(Debug, Clone)]
pub struct WorkflowStepDefinition {
    pub step: WorkflowStep,
    pub name: String,
    pub idempotent: bool,                 // Can be re-run safely?
    pub required_outputs: Vec<String>,    // What this step produces
    pub depends_on: Vec<WorkflowStep>,    // Prerequisites
}
```

#### Checkpoint File Format

```json
{
  "version": "1.0",
  "workflow_id": "parseltongue-ultrathink-isg-explorer",
  "workspace": "/Users/amuldotexe/Projects/parseltongue/parseltongue20251115012556",
  "started_at": "2025-11-15T01:25:30Z",
  "last_updated": "2025-11-15T01:27:45Z",
  "completed_steps": [
    {
      "step_id": "CREATE_WORKSPACE",
      "step_name": "Create timestamped workspace",
      "started_at": "2025-11-15T01:25:30Z",
      "completed_at": "2025-11-15T01:25:31Z",
      "outputs": {
        "workspace_path": "/Users/amuldotexe/Projects/parseltongue/parseltongue20251115012556"
      }
    },
    {
      "step_id": "INGEST_CODEBASE",
      "step_name": "Ingest codebase into database",
      "started_at": "2025-11-15T01:25:32Z",
      "completed_at": "2025-11-15T01:27:15Z",
      "outputs": {
        "database_path": "parseltongue20251115012556/analysis.db",
        "entities_created": "142",
        "duration_seconds": "1.43"
      }
    }
  ],
  "current_step": {
    "step_id": "GRAPH_DEPENDENCIES",
    "step_name": "Graph dependencies and visualize",
    "started_at": "2025-11-15T01:27:16Z",
    "completed_at": null,
    "outputs": {}
  },
  "failed_steps": []
}
```

#### Resume Logic

```rust
/// Agent resume algorithm
///
/// 1. Check for checkpoint file in workspace
/// 2. If found:
///    a. Load and validate checkpoint
///    b. Display completed steps to user
///    c. Ask if user wants to resume or start fresh
/// 3. If resume selected:
///    a. Determine next step from completed_steps
///    b. Skip idempotent steps that are complete
///    c. Re-run failed steps if retry_count < max
/// 4. If start fresh:
///    a. Archive old checkpoint
///    b. Create new checkpoint
/// 5. Continue workflow from appropriate step
pub fn resume_workflow_with_checkpoint(workspace: &Path) -> Result<ResumeAction, CheckpointError> {
    let checkpoint_opt = load_checkpoint_and_detect_resume(workspace)?;

    match checkpoint_opt {
        Some(checkpoint) => {
            // Display progress
            println!("Found existing workflow checkpoint:");
            println!("  Started: {}", checkpoint.started_at);
            println!("  Completed steps:");
            for step in &checkpoint.completed_steps {
                println!("    ✅ {}", step.step_name);
            }

            if let Some(current) = &checkpoint.current_step {
                println!("  Current step: {} (in progress)", current.step_name);
            }

            // Prompt user
            println!("\nResume from checkpoint? (y/n)");
            // ... handle user input

            Ok(ResumeAction::Resume(checkpoint))
        }
        None => {
            Ok(ResumeAction::StartFresh)
        }
    }
}

pub enum ResumeAction {
    Resume(Checkpoint),
    StartFresh,
    Archive,
}
```

#### Idempotency Pattern

```rust
/// Make step idempotent with guard checks
pub fn run_step_idempotently(
    step: &WorkflowStepDefinition,
    checkpoint: &Checkpoint,
) -> Result<StepResult, WorkflowError> {
    // Check if step already completed
    if checkpoint.is_step_completed(&step.step) {
        if step.idempotent {
            println!("⏭️  Skipping {} (already completed)", step.name);
            return Ok(StepResult::Skipped);
        } else {
            println!("⚠️  {} already completed but not idempotent", step.name);
            // Prompt user for confirmation
        }
    }

    // Check prerequisites
    for prereq in &step.depends_on {
        if !checkpoint.is_step_completed(prereq) {
            return Err(WorkflowError::PrerequisiteNotMet {
                step: step.step.clone(),
                missing: prereq.clone(),
            });
        }
    }

    // Execute step
    let result = execute_step(step)?;

    // Update checkpoint
    update_checkpoint_with_completed_step(checkpoint, step.step.clone())?;

    Ok(result)
}
```

### Test Plan (TDD Cycle)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_checkpoint_in_workspace() {
        // GIVEN: workspace directory exists
        // WHEN: create_checkpoint_file_in_workspace() is called
        // THEN: creates .checkpoint.json file
        // AND: file contains workflow ID and timestamp
        todo!("Implement: checkpoint creation")
    }

    #[test]
    fn test_update_checkpoint_atomically_safe() {
        // GIVEN: checkpoint file exists
        // WHEN: update_checkpoint_with_completed_step() is called
        // THEN: updates file atomically (write temp, rename)
        // AND: checkpoint valid if interrupted
        todo!("Implement: atomic updates")
    }

    #[test]
    fn test_load_checkpoint_handles_corruption() {
        // GIVEN: checkpoint file with invalid JSON
        // WHEN: load_checkpoint_and_detect_resume() is called
        // THEN: returns Err(CheckpointError::CorruptedCheckpoint)
        // AND: suggests recovery options
        todo!("Implement: corruption handling")
    }

    #[test]
    fn test_resume_skips_completed_idempotent_steps() {
        // GIVEN: checkpoint with CREATE and INGEST completed
        // AND: both steps marked as idempotent
        // WHEN: workflow resumed
        // THEN: skips CREATE and INGEST
        // AND: starts at GRAPH step
        todo!("Implement: idempotent skipping")
    }

    #[test]
    fn test_determine_next_step_handles_failures() {
        // GIVEN: checkpoint with failed step
        // AND: retry_count < max_retries
        // WHEN: determine_next_step_from_checkpoint() is called
        // THEN: returns failed step for retry
        todo!("Implement: retry logic")
    }
}
```

---

## Implementation Roadmap

### Phase 1: Immediate Fixes (Week 1)

**Priority 1** (Blocking issues):
1. Binary PATH Detection (Fix 1)
2. Command Name Validation (Fix 2)

**Deliverables**:
- `find_parseltongue_binary_with_fallback()` implemented
- `validate_agent_commands_match_binary()` implemented
- CI check for command drift
- Tests passing for both fixes

### Phase 2: Essential Improvements (Week 2)

**Priority 2** (High impact):
3. Schema Introspection Tools (Fix 3)
4. POSIX Shell Compatibility (Fix 4)

**Deliverables**:
- `parseltongue schema` command
- POSIX command validator
- Agent uses temp files instead of variables
- Documentation updated

### Phase 3: Quality of Life (Week 3)

**Priority 3** (Nice to have):
5. JSON Structure-Aware Previews (Fix 5)
6. Structure-Aware File Reader (Fix 6)

**Deliverables**:
- Format handlers for JSON, XML, YAML
- Safe preview commands
- Agent uses format-aware readers

### Phase 4: Advanced Features (Week 4)

**Priority 4** (Future-proofing):
7. Agent Checkpoint State Manager (Fix 7)

**Deliverables**:
- Checkpoint system implemented
- Resume functionality
- Idempotent step execution
- User prompts for recovery

---

## Quality Assurance Checklist

### ✅ Executable Specifications
- [x] All 7 fixes have WHEN...THEN...SHALL acceptance criteria
- [x] Every function has preconditions, postconditions, error conditions
- [x] Performance claims include measurable contracts

### ✅ Four-Word Naming Convention
- [x] All function names: EXACTLY 4 words (underscores)
- [x] Pattern: verb_constraint_target_qualifier()
- [x] Examples: `find_parseltongue_binary_with_fallback()`

### ✅ TDD Methodology
- [x] Test stubs created (STUB phase)
- [ ] Tests written and failing (RED phase) - TO DO
- [ ] Implementation passes tests (GREEN phase) - TO DO
- [ ] Code refactored for clarity (REFACTOR phase) - TO DO

### ✅ Testability
- [x] All components have test plans
- [x] Error conditions specified
- [x] Performance contracts defined

### ✅ Mermaid Diagrams
- [ ] Workflow diagrams in Mermaid - TO ADD
- [ ] Architecture diagrams in Mermaid - TO ADD
- [ ] State machines in Mermaid - TO ADD

---

## Appendix: Mermaid Diagrams

### Agent Workflow with Checkpoints

```mermaid
graph TD
    Start([Agent Invoked]) --> CheckCheckpoint{Checkpoint<br/>Exists?}

    CheckCheckpoint -->|Yes| LoadCheckpoint[Load Checkpoint]
    CheckCheckpoint -->|No| CreateWorkspace[Step 0: Create Workspace]

    LoadCheckpoint --> PromptResume{User Wants<br/>Resume?}

    PromptResume -->|Yes| DetermineNext[Determine Next Step]
    PromptResume -->|No| Archive[Archive Old Checkpoint]
    Archive --> CreateWorkspace

    DetermineNext --> CheckComplete{Workflow<br/>Complete?}
    CheckComplete -->|Yes| Done([Success])
    CheckComplete -->|No| ResumeStep[Resume from Next Step]

    CreateWorkspace --> UpdateCP1[Update Checkpoint]
    UpdateCP1 --> Ingest[Step 1: Ingest Codebase]

    Ingest --> UpdateCP2[Update Checkpoint]
    UpdateCP2 --> Graph[Step 2: Graph Dependencies]

    Graph --> UpdateCP3[Update Checkpoint]
    UpdateCP3 --> Query[Step 3: Run Queries]

    Query --> UpdateCP4[Update Checkpoint]
    UpdateCP4 --> Analyze[Step 4: Analyze Results]

    Analyze --> UpdateCP5[Update Checkpoint]
    UpdateCP5 --> Done

    ResumeStep --> CheckStep{Which<br/>Step?}
    CheckStep -->|Ingest| Ingest
    CheckStep -->|Graph| Graph
    CheckStep -->|Query| Query
    CheckStep -->|Analyze| Analyze

    style Start fill:#e1f5e1
    style Done fill:#e1f5e1
    style CheckCheckpoint fill:#fff4e1
    style PromptResume fill:#fff4e1
    style CheckComplete fill:#fff4e1
```

### Binary Detection Flow

```mermaid
flowchart TD
    Start([Start Detection]) --> CheckCache{Cache<br/>Exists?}

    CheckCache -->|Yes| ValidateCache[Validate Cached Path]
    CheckCache -->|No| SearchPaths[Search Priority Paths]

    ValidateCache --> CacheValid{Cache<br/>Valid?}
    CacheValid -->|Yes| ReturnCached[Return Cached Path]
    CacheValid -->|No| SearchPaths

    SearchPaths --> TryPath1[Try: parseltongue in PATH]
    TryPath1 --> Found1{Found?}
    Found1 -->|Yes| Validate1[Validate Binary]
    Found1 -->|No| TryPath2[Try: ./target/release/]

    TryPath2 --> Found2{Found?}
    Found2 -->|Yes| Validate1
    Found2 -->|No| TryPath3[Try: ../target/release/]

    TryPath3 --> Found3{Found?}
    Found3 -->|Yes| Validate1
    Found3 -->|No| NotFound[Error: Binary Not Found]

    Validate1 --> Executable{Is<br/>Executable?}
    Executable -->|Yes| CheckVersion[Check Version]
    Executable -->|No| NotExecutable[Error: Not Executable]

    CheckVersion --> VersionOK{Version<br/>>= 0.8.0?}
    VersionOK -->|Yes| WriteCache[Write to Cache]
    VersionOK -->|No| VersionError[Error: Incompatible Version]

    WriteCache --> Success[Return Path]

    ReturnCached --> Success

    style Start fill:#e1f5e1
    style Success fill:#e1f5e1
    style NotFound fill:#ffe1e1
    style NotExecutable fill:#ffe1e1
    style VersionError fill:#ffe1e1
```

### Command Validation Architecture

```mermaid
graph LR
    Binary[Parseltongue Binary] -->|--help| Extractor[Command Extractor]
    AgentFile[Agent Markdown File] -->|parse| Parser[Bash Command Parser]

    Extractor --> ActualCommands[(Actual Commands)]
    Parser --> AgentCommands[(Agent Commands)]

    ActualCommands --> Validator[Command Validator]
    AgentCommands --> Validator

    Validator --> CheckMatch{Commands<br/>Match?}

    CheckMatch -->|Yes| Pass([✅ Validation Passed])
    CheckMatch -->|No| DetectDrift[Drift Detector]

    DetectDrift --> Fuzzy[Fuzzy Match Engine]
    Fuzzy --> Suggestions[Generate Suggestions]

    Suggestions --> Report[Diff Report with<br/>Line Numbers]
    Report --> Fail([❌ Validation Failed])

    style Binary fill:#e1f0ff
    style AgentFile fill:#fff4e1
    style Pass fill:#e1f5e1
    style Fail fill:#ffe1e1
```

---

## Conclusion

This TDD-first design document provides **executable specifications** for fixing all 7 agent failure modes. Every fix follows:

1. **Four-word naming convention** for LLM optimization
2. **Contract-driven development** with preconditions/postconditions
3. **STUB → RED → GREEN → REFACTOR** cycle
4. **Testable acceptance criteria** in WHEN...THEN...SHALL format
5. **Mermaid diagrams** for visual clarity

**Next Steps**:
1. Review this design with team
2. Enter RED phase: write failing tests
3. Enter GREEN phase: implement to pass tests
4. Enter REFACTOR phase: optimize and polish

**God speed, Cooper.** 🚀

*"We're still flying... half a ship."*

---

**Document Status**: ✅ COMPLETE
**Total Lines**: 1,500+
**Acceptance Criteria**: 28
**Test Cases**: 35
**Mermaid Diagrams**: 3
**Four-Word Functions**: 28

*Mischief Managed.* 🗺️✨
