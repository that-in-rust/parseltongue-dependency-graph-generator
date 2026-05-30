# Changelog

All notable changes to parseltongue-05 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial crate structure following TDD principles
- Ultra-minimalist file writing (NO backups)
- Three operations: Create, Edit, Delete
- ISGL1 key parsing to file paths
- Fail-fast error handling

### Design Principles
- NO backup files (.bak, .backup, ~)
- NO configuration options
- NO rollback mechanisms
- Single reliable write operation per file
