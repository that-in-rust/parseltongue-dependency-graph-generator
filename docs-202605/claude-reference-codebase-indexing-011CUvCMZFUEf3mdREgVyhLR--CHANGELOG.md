# Changelog

All notable changes to parseltongue-04 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial crate structure following TDD principles
- Three-level validation hierarchy (Syntax → Build → Test)
- ValidationReport and ValidationOutput data structures
- Error handling with thiserror

### Planned
- Rust-analyzer LSP integration for enhanced validation
- Performance contract tests
- Integration with Tool 2 (LLM-to-cozoDB-writer) output
- Integration with Tool 5 (file writing) workflow
