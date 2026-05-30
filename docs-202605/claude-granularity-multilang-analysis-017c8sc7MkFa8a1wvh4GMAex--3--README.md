# parseltongue-05: LLM-cozoDB-to-diff-writer

**Tool 5 in the Parseltongue 6-tool pipeline**

## Purpose

Ultra-minimalist CodeDiff.json generator that reads from CozoDB and produces structured diff context for LLM consumption.

**Key Principle**: Tool 5 generates JSON. LLM reads and applies changes to files.

## Ultra-Minimalist Design Principles

- **NO FILE WRITING**: Tool outputs JSON only, LLM applies changes
- **NO BACKUPS**: Single source of truth in CozoDB
- **NO CONFIGURATION**: Single JSON output operation
- **NO COMPLEXITY**: Query → Transform → JSON

## What It Does

1. Queries CozoDB for entities with `Future_Action != None`
2. Transforms entities into structured diff format
3. Generates CodeDiff.json with:
   - ISGL1 key
   - File path (extracted from key)
   - Operation (Create/Edit/Delete)
   - Future code content
   - Interface signature

## What It Does NOT Do

- ❌ Does NOT write files directly (LLM does that)
- ❌ Does NOT create backups
- ❌ Does NOT validate code (Tool 4 handles syntax, cargo handles types)

## Usage

```bash
# Generate CodeDiff.json from CozoDB
llm-cozodb-to-diff-writer --database ./parseltongue.db --output ./CodeDiff.json

# Verbose output showing all changes
llm-cozodb-to-diff-writer --database ./parseltongue.db --output ./CodeDiff.json --verbose
```

## Example Output

```json
{
  "changes": [
    {
      "isgl1_key": "src_lib_rs-calculate_sum-fn-abc123",
      "file_path": "src/lib.rs",
      "operation": "EDIT",
      "future_code": "fn calculate_sum(a: i32, b: i32) -> i32 { a + b }",
      "interface_signature": "Function calculate_sum"
    },
    {
      "isgl1_key": "src_utils_rs-new_helper-fn-xyz789",
      "file_path": "src/utils.rs",
      "operation": "CREATE",
      "future_code": "fn new_helper() { println!(\"Helper\"); }",
      "interface_signature": "Function new_helper"
    }
  ],
  "metadata": {
    "total_changes": 2,
    "create_count": 1,
    "edit_count": 1,
    "delete_count": 0,
    "generated_at": "2025-10-30T12:34:56.789Z"
  }
}
```

## Integration in Pipeline

**Position**: After validation (Tool 4) → LLM applies changes → State reset (Tool 6)

```
[Tool 4: Validate] → [Tool 5: Generate JSON] → [LLM: Read & Apply] → [Tool 6: Reset]
                           ↓
                     CodeDiff.json
                           ↓
                    [LLM reads this file]
                           ↓
              [LLM writes actual code files]
```

**Input**: CozoDB entities with `Future_Action` (Create/Edit/Delete)
**Output**: Single `CodeDiff.json` file
**Validation**: Assumes Tool 4 pre-validated all `Future_Code`

## ISGL1 Key Formats

Tool 5 extracts file paths from two ISGL1 key formats:

1. **Line-based**: `rust:fn:calculate_sum:src_lib_rs:42-56`
   - Extracts: `src/lib.rs` (from 4th component)

2. **Hash-based**: `src_lib_rs-new_feature-fn-abc123`
   - Extracts: `src/lib.rs` (from 1st component)

## Architecture

### Core Components

- **DiffGenerator**: Queries CozoDB and orchestrates diff generation
- **Change**: Represents a single code change (Create/Edit/Delete)
- **CodeDiff**: Container for all changes with metadata
- **Operation**: Enum for Create/Edit/Delete actions

### Design Pattern

```rust
// Functional transformation pipeline
CozoDB entities → filter(has_future_action) → map(to_change) → CodeDiff
```

## TDD Implementation

Follows RED → GREEN → REFACTOR cycle:

- **RED**: 7 failing integration tests define contracts
- **GREEN**: Minimal implementation passing all tests
- **REFACTOR**: Idiomatic Rust patterns (pattern matching, Result/Option)

### Test Coverage

```bash
# All tests (19 total: 13 lib + 6 integration)
cargo test --package llm-cozodb-to-diff-writer

# Integration tests only
cargo test --package llm-cozodb-to-diff-writer --test diff_generator_tests
```

## Philosophy

> "Separation of concerns: Tool generates context, LLM applies changes"

Tool 5 embodies separation of concerns and the 4-entity architecture:
- **LLM**: Reasoning and decision-making
- **CozoDB**: Single source of truth
- **CodeDiff.json**: Interface between Tool and LLM
- **Codebase**: Real files (LLM's responsibility)

By generating JSON instead of writing files directly, we:
1. Enable LLM to reason about changes before applying
2. Maintain clear separation between tool and LLM responsibilities
3. Allow LLM to use its code writing expertise
4. Simplify Tool 5 to single-purpose JSON generation

## Error Handling

Fail-fast with context:
- CozoDB connection failed → Error with database path
- No changed entities → Warning, empty JSON
- Invalid ISGL1 key format → Error with key details
- JSON serialization failed → Error with entity details

No automatic retries. Fix root cause and re-run.

## Performance

- **Query**: <50ms for typical codebases (100-1000 entities)
- **Generation**: <100ms total including JSON serialization
- **Output size**: ~1-10KB per change (depends on code size)

## Next Steps After Running Tool 5

1. LLM reads `CodeDiff.json`
2. LLM applies changes to codebase files (using its own file writing capabilities)
3. Run `cargo build` to verify compilation
4. Run `cargo test` to verify functionality
5. Run Tool 6 to reset temporal state (if changes are committed)

## License

MIT OR Apache-2.0
