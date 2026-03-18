//! Core entity type definitions for Parseltongue.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ---------------------------------------------------------------------------
// EntityPrimaryKeyLocation
// ---------------------------------------------------------------------------

/// A primary key that locates an entity by file path and line range.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityPrimaryKeyLocation {
    pub file_path: String,
    pub start_line: i32,
    pub end_line: i32,
}

impl fmt::Display for EntityPrimaryKeyLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file_path, self.start_line, self.end_line)
    }
}

impl FromStr for EntityPrimaryKeyLocation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split from the RIGHT to handle file paths that contain colons.
        let mut parts = s.rsplitn(3, ':');
        let end_line_str = parts.next().ok_or_else(|| format!("malformed PK: {s}"))?;
        let start_line_str = parts.next().ok_or_else(|| format!("malformed PK: {s}"))?;
        let file_path = parts.next().ok_or_else(|| format!("malformed PK: {s}"))?;

        let start_line: i32 = start_line_str
            .parse()
            .map_err(|_| format!("invalid start_line in PK: {s}"))?;
        let end_line: i32 = end_line_str
            .parse()
            .map_err(|_| format!("invalid end_line in PK: {s}"))?;

        if file_path.is_empty() {
            return Err(format!("empty file_path in PK: {s}"));
        }

        Ok(Self {
            file_path: file_path.to_string(),
            start_line,
            end_line,
        })
    }
}

impl EntityPrimaryKeyLocation {
    /// Folder sentinel: both lines are -1.
    pub fn is_folder(&self) -> bool {
        self.start_line == -1 && self.end_line == -1
    }

    /// File sentinel: both lines are 0.
    pub fn is_file(&self) -> bool {
        self.start_line == 0 && self.end_line == 0
    }

    /// Code span: start_line >= 1.
    pub fn is_code_span(&self) -> bool {
        self.start_line >= 1
    }

    /// Create a folder sentinel for the given path.
    pub fn folder_sentinel(path: impl Into<String>) -> Self {
        Self {
            file_path: path.into(),
            start_line: -1,
            end_line: -1,
        }
    }

    /// Create a file sentinel for the given path.
    pub fn file_sentinel(path: impl Into<String>) -> Self {
        Self {
            file_path: path.into(),
            start_line: 0,
            end_line: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// EntityTypeClassification
// ---------------------------------------------------------------------------

/// Classification of a code entity.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityTypeClassification {
    Folder,
    FileParsable,
    FileUnparsable,
    FileConfig,
    Function,
    Method,
    Struct,
    Class,
    Enum,
    Trait,
    Interface,
    Impl,
    TypeAlias,
    Constant,
    Static,
    Macro,
    Module,
    Variable,
    Constructor,
    Namespace,
    Record,
    Object,
    Import,
    DocComment,
    Comment,
    Attribute,
    Whitespace,
    Dependency,
    PackageMeta,
    ConfigSection,
}

impl EntityTypeClassification {
    /// Returns the snake_case string representation.
    fn as_snake_case(&self) -> &'static str {
        match self {
            Self::Folder => "folder",
            Self::FileParsable => "file_parsable",
            Self::FileUnparsable => "file_unparsable",
            Self::FileConfig => "file_config",
            Self::Function => "function",
            Self::Method => "method",
            Self::Struct => "struct",
            Self::Class => "class",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Interface => "interface",
            Self::Impl => "impl",
            Self::TypeAlias => "type_alias",
            Self::Constant => "constant",
            Self::Static => "static",
            Self::Macro => "macro",
            Self::Module => "module",
            Self::Variable => "variable",
            Self::Constructor => "constructor",
            Self::Namespace => "namespace",
            Self::Record => "record",
            Self::Object => "object",
            Self::Import => "import",
            Self::DocComment => "doc_comment",
            Self::Comment => "comment",
            Self::Attribute => "attribute",
            Self::Whitespace => "whitespace",
            Self::Dependency => "dependency",
            Self::PackageMeta => "package_meta",
            Self::ConfigSection => "config_section",
        }
    }

    /// All 30 variants in declaration order.
    pub fn all_variants() -> &'static [Self] {
        &[
            Self::Folder,
            Self::FileParsable,
            Self::FileUnparsable,
            Self::FileConfig,
            Self::Function,
            Self::Method,
            Self::Struct,
            Self::Class,
            Self::Enum,
            Self::Trait,
            Self::Interface,
            Self::Impl,
            Self::TypeAlias,
            Self::Constant,
            Self::Static,
            Self::Macro,
            Self::Module,
            Self::Variable,
            Self::Constructor,
            Self::Namespace,
            Self::Record,
            Self::Object,
            Self::Import,
            Self::DocComment,
            Self::Comment,
            Self::Attribute,
            Self::Whitespace,
            Self::Dependency,
            Self::PackageMeta,
            Self::ConfigSection,
        ]
    }

    /// True for the 18 code-level searchable entity types.
    pub fn is_searchable(&self) -> bool {
        matches!(
            self,
            Self::Function
                | Self::Method
                | Self::Struct
                | Self::Class
                | Self::Enum
                | Self::Trait
                | Self::Interface
                | Self::Impl
                | Self::TypeAlias
                | Self::Constant
                | Self::Static
                | Self::Macro
                | Self::Module
                | Self::Variable
                | Self::Constructor
                | Self::Namespace
                | Self::Record
                | Self::Object
        )
    }

    /// True for the 4 structural types.
    pub fn is_structural(&self) -> bool {
        matches!(
            self,
            Self::Folder | Self::FileParsable | Self::FileUnparsable | Self::FileConfig
        )
    }

    /// True for the 3 coverage-only types.
    pub fn is_coverage_only(&self) -> bool {
        matches!(self, Self::Comment | Self::Whitespace | Self::Attribute)
    }
}

impl fmt::Display for EntityTypeClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_snake_case())
    }
}

impl FromStr for EntityTypeClassification {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "folder" => Ok(Self::Folder),
            "file_parsable" => Ok(Self::FileParsable),
            "file_unparsable" => Ok(Self::FileUnparsable),
            "file_config" => Ok(Self::FileConfig),
            "function" => Ok(Self::Function),
            "method" => Ok(Self::Method),
            "struct" => Ok(Self::Struct),
            "class" => Ok(Self::Class),
            "enum" => Ok(Self::Enum),
            "trait" => Ok(Self::Trait),
            "interface" => Ok(Self::Interface),
            "impl" => Ok(Self::Impl),
            "type_alias" => Ok(Self::TypeAlias),
            "constant" => Ok(Self::Constant),
            "static" => Ok(Self::Static),
            "macro" => Ok(Self::Macro),
            "module" => Ok(Self::Module),
            "variable" => Ok(Self::Variable),
            "constructor" => Ok(Self::Constructor),
            "namespace" => Ok(Self::Namespace),
            "record" => Ok(Self::Record),
            "object" => Ok(Self::Object),
            "import" => Ok(Self::Import),
            "doc_comment" => Ok(Self::DocComment),
            "comment" => Ok(Self::Comment),
            "attribute" => Ok(Self::Attribute),
            "whitespace" => Ok(Self::Whitespace),
            "dependency" => Ok(Self::Dependency),
            "package_meta" => Ok(Self::PackageMeta),
            "config_section" => Ok(Self::ConfigSection),
            other => Err(format!("unknown EntityTypeClassification: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// CodeEntityRecord
// ---------------------------------------------------------------------------

/// A record representing a parsed code entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeEntityRecord {
    pub pk: EntityPrimaryKeyLocation,
    pub entity_type: EntityTypeClassification,
    pub language: String,
    pub name: String,
    pub signature: String,
    pub snippet: String,
    pub doc_comment: Option<String>,
    pub wc: u32,
    pub file_hash: String,
    pub is_test: bool,
    pub codebase_id: i64,
    pub indexed_at: i64,

    // Optional rustc fields
    pub rustc_scope: Option<String>,
    pub rustc_sig: Option<String>,
    pub visibility: Option<String>,
    pub mir_calls: Option<String>,
    pub trait_impls: Option<String>,
}

// ---------------------------------------------------------------------------
// EdgeRecord & EdgeType
// ---------------------------------------------------------------------------

/// A record representing an edge between two entities.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeRecord {
    pub from_path: String,
    pub from_start: i32,
    pub from_end: i32,
    pub to_path: String,
    pub to_start: i32,
    pub to_end: i32,
    pub edge_type: String,
    pub codebase_id: i64,
}

/// Classification of edge relationships between entities.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    Calls,
    Imports,
    Implements,
    TypeRefs,
    FieldAccess,
    Contains,
}

impl EdgeType {
    fn as_snake_case(&self) -> &'static str {
        match self {
            Self::Calls => "calls",
            Self::Imports => "imports",
            Self::Implements => "implements",
            Self::TypeRefs => "type_refs",
            Self::FieldAccess => "field_access",
            Self::Contains => "contains",
        }
    }

    /// All 6 variants in declaration order.
    pub fn all_variants() -> &'static [Self] {
        &[
            Self::Calls,
            Self::Imports,
            Self::Implements,
            Self::TypeRefs,
            Self::FieldAccess,
            Self::Contains,
        ]
    }
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_snake_case())
    }
}

impl FromStr for EdgeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "calls" => Ok(Self::Calls),
            "imports" => Ok(Self::Imports),
            "implements" => Ok(Self::Implements),
            "type_refs" => Ok(Self::TypeRefs),
            "field_access" => Ok(Self::FieldAccess),
            "contains" => Ok(Self::Contains),
            other => Err(format!("unknown EdgeType: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pk_round_trip() {
        let pk: EntityPrimaryKeyLocation = "src/main.rs:10:25".parse().unwrap();
        assert_eq!(pk.file_path, "src/main.rs");
        assert_eq!(pk.start_line, 10);
        assert_eq!(pk.end_line, 25);
        assert_eq!(pk.to_string(), "src/main.rs:10:25");
        assert!(pk.is_code_span());
        assert!(!pk.is_file());
        assert!(!pk.is_folder());
    }

    #[test]
    fn pk_sentinels() {
        let folder = EntityPrimaryKeyLocation::folder_sentinel("src");
        assert!(folder.is_folder());
        assert!(!folder.is_file());
        assert!(!folder.is_code_span());
        assert_eq!(folder.file_path, "src");

        let file = EntityPrimaryKeyLocation::file_sentinel("src/main.rs");
        assert!(file.is_file());
        assert!(!file.is_folder());
        assert!(!file.is_code_span());
        assert_eq!(file.file_path, "src/main.rs");
    }

    #[test]
    fn pk_parse_error_invalid() {
        assert!("invalid".parse::<EntityPrimaryKeyLocation>().is_err());
    }

    #[test]
    fn pk_parse_error_non_numeric() {
        assert!("a:b:c".parse::<EntityPrimaryKeyLocation>().is_err());
    }

    #[test]
    fn entity_type_round_trip_all() {
        for variant in EntityTypeClassification::all_variants() {
            let s = variant.to_string();
            let parsed: EntityTypeClassification = s.parse().unwrap();
            assert_eq!(&parsed, variant, "round-trip failed for {s}");
        }
        assert_eq!(EntityTypeClassification::all_variants().len(), 30);
    }

    #[test]
    fn is_searchable_count() {
        let count = EntityTypeClassification::all_variants()
            .iter()
            .filter(|v| v.is_searchable())
            .count();
        assert_eq!(count, 18);
    }

    #[test]
    fn is_structural_count() {
        let count = EntityTypeClassification::all_variants()
            .iter()
            .filter(|v| v.is_structural())
            .count();
        assert_eq!(count, 4);
    }

    #[test]
    fn is_coverage_only_count() {
        let count = EntityTypeClassification::all_variants()
            .iter()
            .filter(|v| v.is_coverage_only())
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn edge_type_round_trip_all() {
        for variant in EdgeType::all_variants() {
            let s = variant.to_string();
            let parsed: EdgeType = s.parse().unwrap();
            assert_eq!(&parsed, variant, "round-trip failed for {s}");
        }
        assert_eq!(EdgeType::all_variants().len(), 6);
    }
}
