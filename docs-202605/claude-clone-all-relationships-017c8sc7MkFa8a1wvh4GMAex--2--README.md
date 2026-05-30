# parseltongue-04: Rust Preflight Code Simulator

**Tool 4 in the Parseltongue 6-tool pipeline**

## Purpose

Validates proposed code changes through a three-level validation hierarchy:
1. **Syntax Validation** - Basic parsing and syntax correctness
2. **Build Validation** - Type checking and compilation verification
3. **Test Validation** - Test suite execution and pass/fail status

## Ultra-Minimalist Design

- **Single Purpose**: Validate code before writing to files
- **No Configuration**: Deterministic validation behavior
- **Clear Error Reporting**: Actionable error messages with recovery paths
- **Rust-Enhanced**: Full rust-analyzer LSP integration for Rust projects
- **Graceful Degradation**: Basic validation for non-Rust languages

## Usage

```bash
# Validate all levels (syntax, build, test)
parseltongue-04 --code-snippet "fn main() {}" --validation-type all

# Validate syntax only
parseltongue-04 --code-snippet "fn main() {}" --validation-type syntax

# Validate from file
parseltongue-04 --file path/to/code.rs --validation-type build
```

## Integration in Pipeline

**Position**: After LLM reasoning (Tool 2) → Before file writing (Tool 5)

**Input**: Proposed code changes from CozoDB (Future_Code field)
**Output**: ValidationReport with pass/fail status and detailed errors
**Recovery**: On failure, returns to LLM reasoning with specific error context

## Performance Contracts

- **Syntax Validation**: <100ms for small code (<1KB)
- **Build Validation**: <2000ms for medium code (1-10KB)
- **Test Validation**: Variable, depends on test count (typically 1-5 minutes)

## Architecture

Follows TDD-first principles with executable specifications:
- RED phase: Failing tests define contracts
- GREEN phase: Minimal implementation passes tests
- REFACTOR phase: Idiomatic Rust patterns and optimization

## License

MIT OR Apache-2.0
