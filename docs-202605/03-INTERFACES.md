# Interfaces: API Design and Integration Points

**Document Version:** 1.0
**Last Updated:** 2025-11-18
**Status:** Design Phase

## Executive Summary

This document defines the **public Rust APIs** and **integration interfaces** for the graph-based compiler. It provides concrete API examples, CozoDB schema interfaces, query builders, and integration points with external systems (LLVM, LSP, build tools).

---

## 1. Rust API Design for Each Compiler Phase

### 1.1 Core Compiler Trait

**Philosophy:** Each phase implements a common `CompilerPhase` trait

```rust
use uuid::Uuid;
use cozo::DbInstance;
use std::sync::Arc;

/// Result type for compiler operations
pub type CompilerResult<T> = Result<T, CompilerError>;

/// Common trait for all compiler phases
pub trait CompilerPhase {
    /// Name of this phase (e.g., "lexing", "parsing")
    fn name(&self) -> &str;

    /// Execute this phase for a specific file
    fn execute(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<PhaseOutput>;

    /// Check if this phase can be skipped (incremental compilation)
    fn can_skip(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<bool>;

    /// Invalidate cached results for this phase
    fn invalidate(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<()>;

    /// Get dependencies for this phase (which other phases must run first)
    fn dependencies(&self) -> Vec<String>;
}

/// Output from a compiler phase
#[derive(Debug, Clone)]
pub struct PhaseOutput {
    pub phase_name: String,
    pub file_id: Uuid,
    pub entities_created: usize,
    pub errors: Vec<CompilerError>,
    pub warnings: Vec<CompilerWarning>,
    pub duration_ms: u64,
}

/// Compiler error type
#[derive(Debug, Clone)]
pub struct CompilerError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    LexError,
    ParseError,
    TypeError,
    NameResolutionError,
    MirBuildError,
    CodegenError,
    InternalError,
}

/// Source span for error reporting
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}
```

### 1.2 Lexer API

```rust
use std::path::Path;

/// Lexer: Converts source code to token graph
pub struct Lexer {
    db: Arc<DbInstance>,
    interner: Arc<StringInterner>,
}

impl Lexer {
    /// Create a new lexer
    pub fn new(db: Arc<DbInstance>) -> Self {
        Self {
            db,
            interner: Arc::new(StringInterner::new()),
        }
    }

    /// Lex a source file
    pub fn lex_file(&self, file_path: &Path, source: &str) -> CompilerResult<LexOutput> {
        let start = std::time::Instant::now();

        // Compute file hash
        let file_hash = compute_sha256(source);

        // Check if file already lexed (incremental)
        if let Some(existing_hash) = self.get_file_hash(&file_path)? {
            if existing_hash == file_hash {
                return Ok(LexOutput::cached());
            }
        }

        // Tokenize using rustc_lexer
        let tokens = self.tokenize(source)?;

        // Insert into database
        let file_id = self.insert_file(file_path, &file_hash, source)?;
        self.insert_tokens(&tokens, file_id)?;
        self.build_token_sequence(file_id)?;

        let duration = start.elapsed();
        Ok(LexOutput {
            file_id,
            token_count: tokens.len(),
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// Tokenize source code
    fn tokenize(&self, source: &str) -> CompilerResult<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut offset = 0;
        let mut line = 1;
        let mut column = 1;

        for token in rustc_lexer::tokenize(source) {
            let text = &source[offset..offset + token.len as usize];

            // Convert rustc_lexer TokenKind to our TokenKind
            let kind = self.convert_token_kind(&token.kind);

            tokens.push(Token {
                id: Uuid::new_v4(),
                kind,
                text: self.interner.intern(text),
                span_start: offset,
                span_end: offset + token.len as usize,
                line,
                column,
            });

            // Update line/column tracking
            for ch in text.chars() {
                if ch == '\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
            }

            offset += token.len as usize;
        }

        Ok(tokens)
    }

    /// Insert tokens into database (bulk)
    fn insert_tokens(&self, tokens: &[Token], file_id: Uuid) -> CompilerResult<()> {
        use cozo::DataValue;

        // Prepare bulk insert data
        let token_data: Vec<DataValue> = tokens.iter().map(|t| {
            DataValue::List(vec![
                t.id.into(),
                file_id.into(),
                t.kind.to_string().into(),
                self.interner.get(t.text).unwrap().into(),
                (t.span_start as i64).into(),
                (t.span_end as i64).into(),
                (t.line as i64).into(),
                (t.column as i64).into(),
                now_timestamp().into(),
            ])
        }).collect();

        // Execute bulk insert
        self.db.run_script(r#"
            ?[id, file_id, kind, text, span_start, span_end, line, column, created_at] <- $tokens
            :put token { id, file_id, kind, text, span_start, span_end, line, column, created_at }
        "#, params!{"tokens" => token_data})?;

        Ok(())
    }
}

impl CompilerPhase for Lexer {
    fn name(&self) -> &str {
        "lexing"
    }

    fn execute(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<PhaseOutput> {
        let file_path = self.get_file_path(db, file_id)?;
        let source = std::fs::read_to_string(&file_path)?;

        let output = self.lex_file(&file_path.as_path(), &source)?;

        Ok(PhaseOutput {
            phase_name: "lexing".to_string(),
            file_id,
            entities_created: output.token_count,
            errors: vec![],
            warnings: vec![],
            duration_ms: output.duration_ms,
        })
    }

    fn can_skip(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<bool> {
        // Check if file hash matches
        let current_hash = self.get_current_file_hash(db, file_id)?;
        let stored_hash = self.get_stored_file_hash(db, file_id)?;
        Ok(current_hash == stored_hash)
    }

    fn invalidate(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<()> {
        db.run_script(r#"
            :rm token { id } <- *token{id, file_id: $file_id}
            :rm token_sequence { to_token } <- *token_sequence{to_token, file_id: $file_id}
        "#, params!{"file_id" => file_id})?;
        Ok(())
    }

    fn dependencies(&self) -> Vec<String> {
        vec![]  // Lexing has no dependencies
    }
}

/// Output from lexing
#[derive(Debug)]
pub struct LexOutput {
    pub file_id: Uuid,
    pub token_count: usize,
    pub duration_ms: u64,
}

impl LexOutput {
    fn cached() -> Self {
        Self {
            file_id: Uuid::nil(),
            token_count: 0,
            duration_ms: 0,
        }
    }
}

/// Token representation
#[derive(Debug, Clone)]
pub struct Token {
    pub id: Uuid,
    pub kind: TokenKind,
    pub text: u64,  // Interned string hash
    pub span_start: usize,
    pub span_end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Fn, Let, Mut, If, Else, While, For, Return, Struct, Enum, Impl, Trait, Pub, Use, Mod,

    // Identifiers and literals
    Ident, IntLiteral, FloatLiteral, StringLiteral, CharLiteral, BoolLiteral,

    // Operators
    Plus, Minus, Star, Slash, Percent,
    Eq, EqEq, Ne, Lt, Le, Gt, Ge,
    AmpAmp, PipePipe, Bang,

    // Delimiters
    OpenParen, CloseParen,
    OpenBrace, CloseBrace,
    OpenBracket, CloseBracket,

    // Punctuation
    Semi, Comma, Dot, Colon, ColonColon, Arrow, FatArrow,

    // Special
    Whitespace, Comment, Eof,
}

impl TokenKind {
    pub fn to_string(&self) -> &'static str {
        match self {
            TokenKind::Fn => "Fn",
            TokenKind::Let => "Let",
            TokenKind::Ident => "Ident",
            TokenKind::IntLiteral => "IntLiteral",
            TokenKind::Plus => "Plus",
            TokenKind::Semi => "Semi",
            // ... (complete mapping)
            _ => "Unknown",
        }
    }
}
```

### 1.3 Parser API

```rust
/// Parser: Converts token graph to AST graph
pub struct Parser {
    db: Arc<DbInstance>,
}

impl Parser {
    pub fn new(db: Arc<DbInstance>) -> Self {
        Self { db }
    }

    /// Parse a file (assumes tokens already in DB)
    pub fn parse_file(&self, file_id: Uuid) -> CompilerResult<ParseOutput> {
        let start = std::time::Instant::now();

        // Load tokens from database
        let tokens = self.load_tokens(file_id)?;

        // Parse tokens into AST
        let ast = self.parse_tokens(&tokens)?;

        // Insert AST into database
        self.insert_ast(&ast, file_id)?;

        let duration = start.elapsed();
        Ok(ParseOutput {
            file_id,
            ast_node_count: ast.node_count(),
            errors: ast.errors,
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// Load tokens from database
    fn load_tokens(&self, file_id: Uuid) -> CompilerResult<Vec<Token>> {
        let result = self.db.run_script(r#"
            ?[id, kind, text, span_start, span_end, line, column] :=
                *token{id, file_id: $file_id, kind, text, span_start, span_end, line, column},
                *token_sequence{to_token: id, order_index, file_id: $file_id}
                :order order_index
        "#, params!{"file_id" => file_id})?;

        // Convert CozoDB rows to Token structs
        result.rows.iter().map(|row| {
            Ok(Token {
                id: row[0].try_into()?,
                kind: TokenKind::from_str(row[1].as_str()?)?,
                text: row[2].as_str()?.to_string(),
                span_start: row[3].as_int()? as usize,
                span_end: row[4].as_int()? as usize,
                line: row[5].as_int()? as usize,
                column: row[6].as_int()? as usize,
            })
        }).collect()
    }

    /// Parse tokens into AST (recursive descent)
    fn parse_tokens(&self, tokens: &[Token]) -> CompilerResult<Ast> {
        let mut parser_state = ParserState::new(tokens);
        let root = self.parse_module(&mut parser_state)?;

        Ok(Ast {
            root,
            errors: parser_state.errors,
        })
    }

    /// Parse a module (top-level items)
    fn parse_module(&self, state: &mut ParserState) -> CompilerResult<AstNode> {
        let module_id = Uuid::new_v4();
        let mut items = Vec::new();

        while !state.is_at_end() {
            match self.parse_item(state) {
                Ok(item) => items.push(item),
                Err(e) => {
                    state.errors.push(e);
                    state.synchronize();  // Error recovery
                }
            }
        }

        Ok(AstNode {
            id: module_id,
            kind: AstKind::Module,
            children: items,
            attributes: HashMap::new(),
            span: Span::default(),
        })
    }

    /// Parse a single item (function, struct, etc.)
    fn parse_item(&self, state: &mut ParserState) -> CompilerResult<AstNode> {
        // Check for visibility
        let visibility = if state.consume(TokenKind::Pub) {
            "pub"
        } else {
            "private"
        };

        // Determine item kind
        if state.consume(TokenKind::Fn) {
            self.parse_function(state, visibility)
        } else if state.consume(TokenKind::Struct) {
            self.parse_struct(state, visibility)
        } else {
            Err(CompilerError::new(
                ErrorKind::ParseError,
                format!("Expected item, found {:?}", state.current()),
            ))
        }
    }

    /// Insert AST into database
    fn insert_ast(&self, ast: &Ast, file_id: Uuid) -> CompilerResult<()> {
        // Flatten AST into nodes and edges
        let (nodes, edges) = ast.flatten();

        // Bulk insert nodes
        let node_data: Vec<DataValue> = nodes.iter().map(|n| {
            DataValue::List(vec![
                n.id.into(),
                file_id.into(),
                n.kind.to_string().into(),
                (n.depth as i64).into(),
                n.parent_id.into(),
                now_timestamp().into(),
            ])
        }).collect();

        self.db.run_script(r#"
            ?[id, file_id, kind, depth, parent_id, created_at] <- $nodes
            :put ast_node { id, file_id, kind, depth, parent_id, created_at }
        "#, params!{"nodes" => node_data})?;

        // Bulk insert edges
        let edge_data: Vec<DataValue> = edges.iter().map(|e| {
            DataValue::List(vec![
                e.from_node.into(),
                e.to_node.into(),
                e.edge_label.into(),
                (e.child_index as i64).into(),
                file_id.into(),
            ])
        }).collect();

        self.db.run_script(r#"
            ?[from_node, to_node, edge_label, child_index, file_id] <- $edges
            :put ast_edge { from_node, to_node, edge_label, child_index, file_id }
        "#, params!{"edges" => edge_data})?;

        Ok(())
    }
}

impl CompilerPhase for Parser {
    fn name(&self) -> &str {
        "parsing"
    }

    fn execute(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<PhaseOutput> {
        let output = self.parse_file(file_id)?;

        Ok(PhaseOutput {
            phase_name: "parsing".to_string(),
            file_id,
            entities_created: output.ast_node_count,
            errors: output.errors,
            warnings: vec![],
            duration_ms: output.duration_ms,
        })
    }

    fn can_skip(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<bool> {
        // Check if AST exists and tokens haven't changed
        let has_ast = self.has_ast(db, file_id)?;
        let tokens_changed = self.tokens_invalidated(db, file_id)?;
        Ok(has_ast && !tokens_changed)
    }

    fn invalidate(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<()> {
        db.run_script(r#"
            :rm ast_node { id } <- *ast_node{id, file_id: $file_id}
            :rm ast_edge { to_node } <- *ast_edge{to_node, file_id: $file_id}
        "#, params!{"file_id" => file_id})?;
        Ok(())
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["lexing".to_string()]
    }
}

/// AST representation
#[derive(Debug)]
pub struct Ast {
    pub root: AstNode,
    pub errors: Vec<CompilerError>,
}

impl Ast {
    /// Flatten AST into nodes and edges for database insertion
    fn flatten(&self) -> (Vec<AstNodeFlat>, Vec<AstEdgeFlat>) {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        self.flatten_node(&self.root, None, 0, &mut nodes, &mut edges);
        (nodes, edges)
    }

    fn flatten_node(
        &self,
        node: &AstNode,
        parent_id: Option<Uuid>,
        depth: usize,
        nodes: &mut Vec<AstNodeFlat>,
        edges: &mut Vec<AstEdgeFlat>,
    ) {
        nodes.push(AstNodeFlat {
            id: node.id,
            kind: node.kind.clone(),
            depth,
            parent_id,
        });

        for (idx, child) in node.children.iter().enumerate() {
            edges.push(AstEdgeFlat {
                from_node: node.id,
                to_node: child.id,
                edge_label: format!("child_{}", idx),
                child_index: idx,
            });

            self.flatten_node(child, Some(node.id), depth + 1, nodes, edges);
        }
    }

    fn node_count(&self) -> usize {
        self.flatten().0.len()
    }
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub id: Uuid,
    pub kind: AstKind,
    pub children: Vec<AstNode>,
    pub attributes: HashMap<String, String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstKind {
    Module,
    Function,
    Struct,
    Enum,
    Impl,
    Block,
    ExprBinaryOp,
    ExprCall,
    ExprLiteral,
    ExprIdent,
    StmtLet,
    StmtExpr,
    // ... (complete enum)
}
```

### 1.4 Type Checker API

```rust
/// Type checker: Infers types for HIR
pub struct TypeChecker {
    db: Arc<DbInstance>,
}

impl TypeChecker {
    pub fn new(db: Arc<DbInstance>) -> Self {
        Self { db }
    }

    /// Type check a file
    pub fn type_check_file(&self, file_id: Uuid) -> CompilerResult<TypeCheckOutput> {
        let start = std::time::Instant::now();

        // Generate type variables
        self.generate_type_variables(file_id)?;

        // Emit constraints
        self.emit_constraints(file_id)?;

        // Unify
        let unification_count = self.unify()?;

        // Check for errors
        let errors = self.collect_type_errors(file_id)?;

        let duration = start.elapsed();
        Ok(TypeCheckOutput {
            file_id,
            types_inferred: unification_count,
            errors,
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// Generate type variables for all HIR nodes
    fn generate_type_variables(&self, file_id: Uuid) -> CompilerResult<()> {
        self.db.run_script(r#"
            # Create type variable for each HIR expression node
            ?[id, kind, type_repr, created_at] <-
                *hir_node{id: node_id, file_id: $file_id, kind},
                # Only for nodes that need types
                kind in ["Expr::Literal", "Expr::BinaryOp", "Expr::Call", "Expr::Ident"],
                # Generate unique type variable
                id = gen_uuid(),
                kind = "Variable",
                type_repr = str_cat("?T", str(node_id)),
                created_at = now()
            :put type_node { id, kind, type_repr, created_at }
        "#, params!{"file_id" => file_id})?;

        Ok(())
    }

    /// Emit type constraints based on HIR
    fn emit_constraints(&self, file_id: Uuid) -> CompilerResult<()> {
        // Constraint 1: Literals have known types
        self.db.run_script(r#"
            ?[id, left_type, right_type, constraint_kind, hir_node_id, reason] <-
                *hir_node{id: hir_node_id, file_id: $file_id, kind: "Expr::Literal"},
                *hir_attr{node_id: hir_node_id, key: "lit_kind", value: lit_kind},
                *type_node{id: left_type, type_repr: str_cat("?T", str(hir_node_id))},
                # Create concrete type
                right_type_id = gen_uuid(),
                right_type_repr = if(lit_kind == "Int", "i32", "unknown"),
                id = gen_uuid(),
                constraint_kind = "Equals",
                reason = "literal_type"
            # First create the concrete type
            :put type_node {
                id: right_type_id,
                kind: "Concrete",
                type_repr: right_type_repr,
                created_at: now()
            }
            # Then create the constraint
            :put type_constraint {
                id, left_type, right_type: right_type_id,
                constraint_kind, hir_node_id, reason
            }
        "#, params!{"file_id" => file_id})?;

        // Constraint 2: Binary op operands must match
        self.db.run_script(r#"
            ?[id, left_type, right_type, constraint_kind, hir_node_id, reason] <-
                *hir_node{id: hir_node_id, file_id: $file_id, kind: "Expr::BinaryOp"},
                *hir_edge{from_node: hir_node_id, to_node: lhs_id, edge_label: "lhs"},
                *hir_edge{from_node: hir_node_id, to_node: rhs_id, edge_label: "rhs"},
                *type_node{id: left_type, type_repr: str_cat("?T", str(lhs_id))},
                *type_node{id: right_type, type_repr: str_cat("?T", str(rhs_id))},
                id = gen_uuid(),
                constraint_kind = "Equals",
                reason = "binary_op_operands"
            :put type_constraint {
                id, left_type, right_type,
                constraint_kind, hir_node_id, reason
            }
        "#, params!{"file_id" => file_id})?;

        // ... (more constraint rules)
        Ok(())
    }

    /// Unify type variables (fixpoint iteration)
    fn unify(&self) -> CompilerResult<usize> {
        let mut iteration = 0;
        let max_iterations = 100;

        loop {
            iteration += 1;
            if iteration > max_iterations {
                return Err(CompilerError::new(
                    ErrorKind::TypeError,
                    "Type unification did not converge".to_string(),
                ));
            }

            // Unify: ?T = concrete_type
            let result = self.db.run_script(r#"
                ?[type_var, unified_type, unified_at] <-
                    *type_constraint{left_type, right_type, constraint_kind: "Equals"},
                    *type_node{id: left_type, kind: "Variable"},
                    *type_node{id: right_type, kind: "Concrete"},
                    # Not already unified
                    not *type_unification{type_var: left_type},
                    type_var = left_type,
                    unified_type = right_type,
                    unified_at = now()
                :put type_unification { type_var, unified_type, unified_at }
            "#, Default::default())?;

            let unified_count = result.rows.len();
            if unified_count == 0 {
                break;  // Fixpoint reached
            }

            // Propagate unifications through constraints
            self.db.run_script(r#"
                ?[type_var, unified_type, unified_at] <-
                    *type_constraint{left_type: var1, right_type: var2, constraint_kind: "Equals"},
                    *type_unification{type_var: var1, unified_type},
                    *type_node{id: var2, kind: "Variable"},
                    # var2 not yet unified
                    not *type_unification{type_var: var2},
                    type_var = var2,
                    unified_at = now()
                :put type_unification { type_var, unified_type, unified_at }
            "#, Default::default())?;
        }

        // Count total unifications
        let count = self.db.run_script(r#"
            ?[count(type_var)] := *type_unification{type_var}
        "#, Default::default())?
            .rows.first()
            .and_then(|row| row.first())
            .and_then(|v| v.as_int())
            .unwrap_or(0);

        Ok(count as usize)
    }
}

impl CompilerPhase for TypeChecker {
    fn name(&self) -> &str {
        "type_checking"
    }

    fn execute(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<PhaseOutput> {
        let output = self.type_check_file(file_id)?;

        Ok(PhaseOutput {
            phase_name: "type_checking".to_string(),
            file_id,
            entities_created: output.types_inferred,
            errors: output.errors,
            warnings: vec![],
            duration_ms: output.duration_ms,
        })
    }

    fn can_skip(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<bool> {
        // Skip if types already inferred and HIR hasn't changed
        let has_types = self.has_types(db, file_id)?;
        let hir_changed = self.hir_invalidated(db, file_id)?;
        Ok(has_types && !hir_changed)
    }

    fn invalidate(&self, db: &DbInstance, file_id: Uuid) -> CompilerResult<()> {
        db.run_script(r#"
            :rm type_node { id } <- *type_node{id, /* filter by file_id somehow */}
            :rm type_constraint { id } <- *type_constraint{id, /* filter by file_id */}
            :rm type_unification { type_var } <- *type_unification{type_var, /* filter */}
        "#, params!{"file_id" => file_id})?;
        Ok(())
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["hir_lowering".to_string()]
    }
}
```

### 1.5 Compiler Pipeline Orchestrator

```rust
/// Orchestrates the entire compilation pipeline
pub struct CompilerPipeline {
    db: Arc<DbInstance>,
    phases: Vec<Box<dyn CompilerPhase>>,
}

impl CompilerPipeline {
    /// Create a new pipeline
    pub fn new(db: Arc<DbInstance>) -> Self {
        let phases: Vec<Box<dyn CompilerPhase>> = vec![
            Box::new(Lexer::new(db.clone())),
            Box::new(Parser::new(db.clone())),
            Box::new(HirLowering::new(db.clone())),
            Box::new(TypeChecker::new(db.clone())),
            Box::new(MirBuilder::new(db.clone())),
            Box::new(Optimizer::new(db.clone())),
            Box::new(Codegen::new(db.clone())),
        ];

        Self { db, phases }
    }

    /// Compile a single file through all phases
    pub fn compile_file(&self, file_path: &Path) -> CompilerResult<CompilationOutput> {
        let start = std::time::Instant::now();
        let file_id = self.get_or_create_file_id(file_path)?;

        let mut phase_outputs = Vec::new();
        let mut all_errors = Vec::new();

        for phase in &self.phases {
            println!("Running phase: {}", phase.name());

            // Check if phase can be skipped
            if phase.can_skip(&self.db, file_id)? {
                println!("  Skipped (up-to-date)");
                continue;
            }

            // Execute phase
            match phase.execute(&self.db, file_id) {
                Ok(output) => {
                    println!("  Created {} entities in {}ms",
                             output.entities_created, output.duration_ms);
                    all_errors.extend(output.errors.clone());
                    phase_outputs.push(output);
                }
                Err(e) => {
                    eprintln!("  Failed: {}", e.message);
                    all_errors.push(e.clone());
                    // Stop pipeline on error
                    break;
                }
            }
        }

        let duration = start.elapsed();
        Ok(CompilationOutput {
            file_id,
            file_path: file_path.to_path_buf(),
            phase_outputs,
            errors: all_errors,
            warnings: vec![],
            total_duration_ms: duration.as_millis() as u64,
            success: all_errors.is_empty(),
        })
    }

    /// Compile an entire project
    pub fn compile_project(&self, project_path: &Path) -> CompilerResult<ProjectCompilationOutput> {
        // Find all source files
        let source_files = self.find_source_files(project_path)?;

        // Build dependency graph
        let dep_graph = self.build_dependency_graph(&source_files)?;

        // Topological sort
        let compilation_order = dep_graph.topological_sort()?;

        // Compile in order
        let mut file_outputs = Vec::new();
        for file_path in compilation_order {
            let output = self.compile_file(&file_path)?;
            file_outputs.push(output);

            if !output.success {
                // Stop on first error (configurable)
                break;
            }
        }

        Ok(ProjectCompilationOutput {
            file_outputs,
            success: file_outputs.iter().all(|o| o.success),
        })
    }
}

#[derive(Debug)]
pub struct CompilationOutput {
    pub file_id: Uuid,
    pub file_path: PathBuf,
    pub phase_outputs: Vec<PhaseOutput>,
    pub errors: Vec<CompilerError>,
    pub warnings: Vec<CompilerWarning>,
    pub total_duration_ms: u64,
    pub success: bool,
}

#[derive(Debug)]
pub struct ProjectCompilationOutput {
    pub file_outputs: Vec<CompilationOutput>,
    pub success: bool,
}
```

---

## 2. CozoDB Query Builder API

### 2.1 Type-Safe Query Builder

```rust
/// Builder for CozoDB queries
pub struct QueryBuilder {
    query: String,
    params: HashMap<String, DataValue>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            params: HashMap::new(),
        }
    }

    /// Select columns
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.query = format!("?[{}]", columns.join(", "));
        self
    }

    /// From relation
    pub fn from(mut self, relation: &str) -> Self {
        self.query.push_str(&format!(" := *{}", relation));
        self
    }

    /// Where clause
    pub fn where_clause(mut self, condition: &str) -> Self {
        self.query.push_str(&format!(", {}", condition));
        self
    }

    /// Add parameter
    pub fn param<T: Into<DataValue>>(mut self, name: &str, value: T) -> Self {
        self.params.insert(name.to_string(), value.into());
        self
    }

    /// Order by
    pub fn order_by(mut self, column: &str) -> Self {
        self.query.push_str(&format!(" :order {}", column));
        self
    }

    /// Limit
    pub fn limit(mut self, n: usize) -> Self {
        self.query.push_str(&format!(" :limit {}", n));
        self
    }

    /// Build and execute
    pub fn execute(self, db: &DbInstance) -> CompilerResult<CozoResult> {
        Ok(db.run_script(&self.query, self.params)?)
    }

    /// Build query string (for debugging)
    pub fn build(self) -> (String, HashMap<String, DataValue>) {
        (self.query, self.params)
    }
}

/// Example usage
fn example_query_builder() {
    let db = /* ... */;

    // Find all functions in a file
    let result = QueryBuilder::new()
        .select(&["fn_id", "fn_name"])
        .from("hir_def{id: fn_id, def_kind, name: fn_name}")
        .where_clause("def_kind == \"fn\"")
        .where_clause("file_id == $file_id")
        .param("file_id", some_file_id)
        .order_by("fn_name")
        .execute(&db)?;
}
```

### 2.2 Predefined Query Templates

```rust
/// Common query templates
pub struct QueryTemplates;

impl QueryTemplates {
    /// Get all tokens for a file in sequence
    pub fn tokens_in_sequence(file_id: Uuid) -> (String, HashMap<String, DataValue>) {
        let query = r#"
            ?[id, kind, text, span_start, span_end] :=
                *token{id, file_id: $file_id, kind, text, span_start, span_end},
                *token_sequence{to_token: id, order_index, file_id: $file_id}
                :order order_index
        "#.to_string();

        let mut params = HashMap::new();
        params.insert("file_id".to_string(), file_id.into());

        (query, params)
    }

    /// Get AST subtree
    pub fn ast_subtree(root_id: Uuid) -> (String, HashMap<String, DataValue>) {
        let query = r#"
            ?[node_id, kind, depth] :=
                *ast_edge{from_node: $root_id, to_node: node_id},
                *ast_node{id: node_id, kind, depth}

            ?[node_id, kind, depth] :=
                *ast_edge{from_node: $root_id, to_node: child_id},
                *ast_edge{from_node: child_id, to_node: node_id},
                *ast_node{id: node_id, kind, depth}
        "#.to_string();

        let mut params = HashMap::new();
        params.insert("root_id".to_string(), root_id.into());

        (query, params)
    }

    /// Find all uses of a definition
    pub fn find_uses(def_id: Uuid) -> (String, HashMap<String, DataValue>) {
        let query = r#"
            ?[use_node_id, use_kind, span_start, span_end] :=
                *hir_use{def_id: $def_id, use_node_id, use_kind, span_start, span_end}
        "#.to_string();

        let mut params = HashMap::new();
        params.insert("def_id".to_string(), def_id.into());

        (query, params)
    }

    /// Get type of an expression
    pub fn get_type(expr_id: Uuid) -> (String, HashMap<String, DataValue>) {
        let query = r#"
            ?[type_repr] :=
                *type_node{id: type_var, type_repr: var_repr},
                var_repr == str_cat("?T", str($expr_id)),
                *type_unification{type_var, unified_type},
                *type_node{id: unified_type, type_repr}
        "#.to_string();

        let mut params = HashMap::new();
        params.insert("expr_id".to_string(), expr_id.into());

        (query, params)
    }

    /// Get MIR for a function
    pub fn mir_for_function(fn_id: Uuid) -> (String, HashMap<String, DataValue>) {
        let query = r#"
            ?[bb_id, block_index, stmt_id, stmt_index, stmt_kind] :=
                *mir_fn{id: $fn_id},
                *mir_bb{fn_id: $fn_id, id: bb_id, block_index},
                *mir_stmt{bb_id, id: stmt_id, stmt_index, kind: stmt_kind}
                :order block_index, stmt_index
        "#.to_string();

        let mut params = HashMap::new();
        params.insert("fn_id".to_string(), fn_id.into());

        (query, params)
    }
}
```

---

## 3. Integration with LLVM

### 3.1 LLVM Codegen Interface

```rust
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::FunctionValue;

/// LLVM codegen: Converts MIR to LLVM IR
pub struct LlvmCodegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    db: Arc<DbInstance>,
}

impl<'ctx> LlvmCodegen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str, db: Arc<DbInstance>) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            db,
        }
    }

    /// Generate LLVM IR for a function
    pub fn codegen_function(&self, mir_fn_id: Uuid) -> CompilerResult<FunctionValue<'ctx>> {
        // Load MIR function metadata
        let fn_meta = self.load_mir_function_metadata(mir_fn_id)?;

        // Create LLVM function signature
        let fn_type = self.create_function_type(&fn_meta)?;
        let fn_value = self.module.add_function(&fn_meta.name, fn_type, None);

        // Load MIR basic blocks
        let basic_blocks = self.load_mir_basic_blocks(mir_fn_id)?;

        // Create LLVM basic blocks
        let mut llvm_blocks = HashMap::new();
        for bb in &basic_blocks {
            let llvm_bb = self.context.append_basic_block(fn_value, &format!("bb{}", bb.index));
            llvm_blocks.insert(bb.id, llvm_bb);
        }

        // Codegen each basic block
        for bb in &basic_blocks {
            let llvm_bb = llvm_blocks[&bb.id];
            self.builder.position_at_end(llvm_bb);

            // Codegen statements
            let statements = self.load_mir_statements(bb.id)?;
            for stmt in statements {
                self.codegen_statement(&stmt, &fn_value)?;
            }

            // Codegen terminator
            if let Some(term) = &bb.terminator {
                self.codegen_terminator(term, &llvm_blocks)?;
            }
        }

        Ok(fn_value)
    }

    /// Load MIR function metadata from DB
    fn load_mir_function_metadata(&self, fn_id: Uuid) -> CompilerResult<MirFunctionMetadata> {
        let result = self.db.run_script(r#"
            ?[name, return_type, param_count] :=
                *mir_fn{id: $fn_id, name, return_type, param_count}
        "#, params!{"fn_id" => fn_id})?;

        let row = result.rows.first().ok_or_else(|| {
            CompilerError::new(ErrorKind::InternalError, "MIR function not found".to_string())
        })?;

        Ok(MirFunctionMetadata {
            id: fn_id,
            name: row[0].as_str()?.to_string(),
            return_type: row[1].try_into()?,
            param_count: row[2].as_int()? as usize,
        })
    }

    /// Codegen a MIR statement
    fn codegen_statement(&self, stmt: &MirStatement, fn_value: &FunctionValue<'ctx>) -> CompilerResult<()> {
        match stmt.kind.as_str() {
            "Assign" => {
                // Load rvalue
                let rvalue = self.load_mir_rvalue(stmt.id)?;
                let value = self.codegen_rvalue(&rvalue, fn_value)?;

                // Store to lvalue
                let lvalue = self.load_mir_place(stmt.id)?;
                self.store_to_place(&lvalue, value)?;
            }
            "Call" => {
                // Codegen function call
                // ... (detailed implementation)
            }
            _ => {
                return Err(CompilerError::new(
                    ErrorKind::CodegenError,
                    format!("Unsupported MIR statement: {}", stmt.kind),
                ));
            }
        }

        Ok(())
    }

    /// Codegen a MIR terminator
    fn codegen_terminator(
        &self,
        term: &MirTerminator,
        llvm_blocks: &HashMap<Uuid, BasicBlock<'ctx>>,
    ) -> CompilerResult<()> {
        match term.kind.as_str() {
            "Goto" => {
                let target_bb = llvm_blocks[&term.target_bb];
                self.builder.build_unconditional_branch(target_bb);
            }
            "Return" => {
                let return_value = self.codegen_return_value(term)?;
                self.builder.build_return(Some(&return_value));
            }
            "SwitchInt" => {
                // Codegen switch statement
                // ... (detailed implementation)
            }
            _ => {
                return Err(CompilerError::new(
                    ErrorKind::CodegenError,
                    format!("Unsupported MIR terminator: {}", term.kind),
                ));
            }
        }

        Ok(())
    }

    /// Finalize and output LLVM IR
    pub fn finalize(&self, output_path: &Path) -> CompilerResult<()> {
        // Verify module
        if let Err(e) = self.module.verify() {
            return Err(CompilerError::new(
                ErrorKind::CodegenError,
                format!("LLVM module verification failed: {}", e),
            ));
        }

        // Write LLVM IR to file
        self.module.print_to_file(output_path)?;

        Ok(())
    }
}
```

---

## 4. Language Server Protocol (LSP) Integration

### 4.1 LSP Server Implementation

```rust
use lsp_server::{Connection, Message, Request, Response};
use lsp_types::*;

/// LSP server for graph-based compiler
pub struct GraphCompilerLsp {
    db: Arc<DbInstance>,
    connection: Connection,
}

impl GraphCompilerLsp {
    pub fn new(db: Arc<DbInstance>) -> Self {
        let (connection, _io_threads) = Connection::stdio();
        Self { db, connection }
    }

    pub fn run(&self) -> CompilerResult<()> {
        // Main LSP loop
        for msg in &self.connection.receiver {
            match msg {
                Message::Request(req) => {
                    self.handle_request(req)?;
                }
                Message::Notification(not) => {
                    self.handle_notification(not)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn handle_request(&self, req: Request) -> CompilerResult<()> {
        match req.method.as_str() {
            "textDocument/completion" => {
                let params: CompletionParams = serde_json::from_value(req.params)?;
                let result = self.handle_completion(params)?;
                let response = Response::new_ok(req.id, result);
                self.connection.sender.send(Message::Response(response))?;
            }
            "textDocument/definition" => {
                let params: GotoDefinitionParams = serde_json::from_value(req.params)?;
                let result = self.handle_goto_definition(params)?;
                let response = Response::new_ok(req.id, result);
                self.connection.sender.send(Message::Response(response))?;
            }
            "textDocument/references" => {
                let params: ReferenceParams = serde_json::from_value(req.params)?;
                let result = self.handle_find_references(params)?;
                let response = Response::new_ok(req.id, result);
                self.connection.sender.send(Message::Response(response))?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle code completion
    fn handle_completion(&self, params: CompletionParams) -> CompilerResult<CompletionList> {
        let file_id = self.get_file_id(&params.text_document_position.text_document.uri)?;
        let position = params.text_document_position.position;

        // Find AST node at position
        let node_id = self.find_node_at_position(file_id, position)?;

        // Get scope at this position
        let scope_id = self.get_scope(node_id)?;

        // Query all definitions in scope
        let defs = self.db.run_script(r#"
            ?[name, def_kind] :=
                *hir_def{id, name, scope_id: $scope_id, def_kind}
        "#, params!{"scope_id" => scope_id})?;

        // Convert to completion items
        let items = defs.rows.iter().map(|row| {
            CompletionItem {
                label: row[0].as_str().unwrap().to_string(),
                kind: Some(self.def_kind_to_completion_kind(row[1].as_str().unwrap())),
                ..Default::default()
            }
        }).collect();

        Ok(CompletionList {
            is_incomplete: false,
            items,
        })
    }

    /// Handle go-to-definition
    fn handle_goto_definition(&self, params: GotoDefinitionParams) -> CompilerResult<Option<Location>> {
        let file_id = self.get_file_id(&params.text_document_position_params.text_document.uri)?;
        let position = params.text_document_position_params.position;

        // Find identifier at position
        let ident_node_id = self.find_ident_at_position(file_id, position)?;

        // Find definition this identifier refers to
        let def_result = self.db.run_script(r#"
            ?[def_file_id, span_start, span_end, start_line, start_col] :=
                *hir_use{use_node_id: $ident_node_id, def_id},
                *hir_def{id: def_id, span_start, span_end},
                *hir_node{id: def_id, file_id: def_file_id},
                # Get file path
                *file{id: def_file_id, path},
                # Convert span to line/col (simplified)
                start_line = 1, start_col = span_start
        "#, params!{"ident_node_id" => ident_node_id})?;

        if let Some(row) = def_result.rows.first() {
            let def_file_path = self.get_file_path(row[0].try_into()?)?;
            let start_line = row[3].as_int()? as u32;
            let start_col = row[4].as_int()? as u32;

            Ok(Some(Location {
                uri: Url::from_file_path(def_file_path).unwrap(),
                range: Range {
                    start: Position { line: start_line, character: start_col },
                    end: Position { line: start_line, character: start_col + 10 },
                },
            }))
        } else {
            Ok(None)
        }
    }

    /// Handle find-all-references
    fn handle_find_references(&self, params: ReferenceParams) -> CompilerResult<Vec<Location>> {
        let file_id = self.get_file_id(&params.text_document_position.text_document.uri)?;
        let position = params.text_document_position.position;

        // Find identifier at position
        let ident_node_id = self.find_ident_at_position(file_id, position)?;

        // Find definition
        let def_result = self.db.run_script(r#"
            ?[def_id] := *hir_use{use_node_id: $ident_node_id, def_id}
        "#, params!{"ident_node_id" => ident_node_id})?;

        let def_id: Uuid = def_result.rows.first()
            .and_then(|row| row.first())
            .ok_or_else(|| CompilerError::new(ErrorKind::InternalError, "No definition found".to_string()))?
            .try_into()?;

        // Find all uses of this definition
        let uses = self.db.run_script(r#"
            ?[use_file_id, span_start, span_end, line, col] :=
                *hir_use{def_id: $def_id, use_node_id, span_start, span_end},
                *hir_node{id: use_node_id, file_id: use_file_id},
                # Convert span to line/col
                line = 1, col = span_start
        "#, params!{"def_id" => def_id})?;

        // Convert to LSP locations
        let locations = uses.rows.iter().map(|row| {
            let file_path = self.get_file_path(row[0].try_into().unwrap()).unwrap();
            let line = row[3].as_int().unwrap() as u32;
            let col = row[4].as_int().unwrap() as u32;

            Location {
                uri: Url::from_file_path(file_path).unwrap(),
                range: Range {
                    start: Position { line, character: col },
                    end: Position { line, character: col + 10 },
                },
            }
        }).collect();

        Ok(locations)
    }
}
```

---

## 5. File I/O and Change Detection

### 5.1 File Watcher Interface

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;

/// File watcher for incremental compilation
pub struct FileWatcher {
    db: Arc<DbInstance>,
    watcher: notify::RecommendedWatcher,
}

impl FileWatcher {
    pub fn new(db: Arc<DbInstance>, watch_path: &Path) -> CompilerResult<Self> {
        let (tx, rx) = channel();

        let watcher = watcher(tx, Duration::from_secs(1))?;
        watcher.watch(watch_path, RecursiveMode::Recursive)?;

        // Spawn thread to handle file change events
        let db_clone = db.clone();
        std::thread::spawn(move || {
            for event in rx {
                match event {
                    Ok(notify::Event::Modify(path)) => {
                        if let Err(e) = Self::handle_file_change(&db_clone, &path) {
                            eprintln!("Error handling file change: {}", e);
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(Self { db, watcher })
    }

    fn handle_file_change(db: &DbInstance, path: &Path) -> CompilerResult<()> {
        println!("File changed: {:?}", path);

        // Recompute hash
        let source = std::fs::read_to_string(path)?;
        let new_hash = compute_sha256(&source);

        // Check if hash actually changed
        let file_id = Self::get_file_id(db, path)?;
        let old_hash = Self::get_file_hash(db, file_id)?;

        if new_hash != old_hash {
            println!("  Hash changed, invalidating...");

            // Invalidate this file's data
            Self::invalidate_file(db, file_id)?;

            // Recompile
            let pipeline = CompilerPipeline::new(db.clone());
            pipeline.compile_file(path)?;

            println!("  Recompiled successfully");
        } else {
            println!("  Hash unchanged, skipping");
        }

        Ok(())
    }
}
```

---

## 6. Debugging and Introspection APIs

### 6.1 Graph Visualization Interface

```rust
/// Generate DOT graph for visualization
pub struct GraphVisualizer {
    db: Arc<DbInstance>,
}

impl GraphVisualizer {
    /// Visualize AST for a file
    pub fn visualize_ast(&self, file_id: Uuid, output_path: &Path) -> CompilerResult<()> {
        let nodes = self.db.run_script(r#"
            ?[id, kind] := *ast_node{id, file_id: $file_id, kind}
        "#, params!{"file_id" => file_id})?;

        let edges = self.db.run_script(r#"
            ?[from_node, to_node, edge_label] :=
                *ast_edge{from_node, to_node, edge_label, file_id: $file_id}
        "#, params!{"file_id" => file_id})?;

        // Generate DOT format
        let mut dot = String::from("digraph AST {\n");

        for row in nodes.rows {
            let id = row[0].as_str()?;
            let kind = row[1].as_str()?;
            dot.push_str(&format!("  \"{}\" [label=\"{}\"];\n", id, kind));
        }

        for row in edges.rows {
            let from = row[0].as_str()?;
            let to = row[1].as_str()?;
            let label = row[2].as_str()?;
            dot.push_str(&format!("  \"{}\" -> \"{}\" [label=\"{}\"];\n", from, to, label));
        }

        dot.push_str("}\n");

        std::fs::write(output_path, dot)?;
        Ok(())
    }

    /// Visualize MIR CFG for a function
    pub fn visualize_mir_cfg(&self, fn_id: Uuid, output_path: &Path) -> CompilerResult<()> {
        let blocks = self.db.run_script(r#"
            ?[bb_id, block_index] := *mir_bb{id: bb_id, fn_id: $fn_id, block_index}
        "#, params!{"fn_id" => fn_id})?;

        let edges = self.db.run_script(r#"
            ?[from_bb, to_bb, edge_kind] :=
                *mir_cfg_edge{from_bb, to_bb, edge_kind},
                *mir_bb{id: from_bb, fn_id: $fn_id}
        "#, params!{"fn_id" => fn_id})?;

        // Generate DOT format
        let mut dot = String::from("digraph CFG {\n");
        dot.push_str("  node [shape=box];\n");

        for row in blocks.rows {
            let bb_id = row[0].as_str()?;
            let index = row[1].as_int()?;
            dot.push_str(&format!("  \"{}\" [label=\"bb{}\"];\n", bb_id, index));
        }

        for row in edges.rows {
            let from = row[0].as_str()?;
            let to = row[1].as_str()?;
            let kind = row[2].as_str()?;
            dot.push_str(&format!("  \"{}\" -> \"{}\" [label=\"{}\"];\n", from, to, kind));
        }

        dot.push_str("}\n");

        std::fs::write(output_path, dot)?;
        Ok(())
    }
}
```

### 6.2 Query Profiler

```rust
/// Profile CozoDB query performance
pub struct QueryProfiler {
    db: Arc<DbInstance>,
    stats: Arc<Mutex<HashMap<String, QueryStats>>>,
}

#[derive(Default)]
struct QueryStats {
    execution_count: usize,
    total_duration_ms: u64,
    avg_duration_ms: u64,
}

impl QueryProfiler {
    pub fn new(db: Arc<DbInstance>) -> Self {
        Self {
            db,
            stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Execute query with profiling
    pub fn execute_with_profiling(
        &self,
        query_name: &str,
        query: &str,
        params: HashMap<String, DataValue>,
    ) -> CompilerResult<CozoResult> {
        let start = std::time::Instant::now();

        let result = self.db.run_script(query, params)?;

        let duration = start.elapsed().as_millis() as u64;

        // Update stats
        let mut stats = self.stats.lock().unwrap();
        let entry = stats.entry(query_name.to_string()).or_insert_with(Default::default);
        entry.execution_count += 1;
        entry.total_duration_ms += duration;
        entry.avg_duration_ms = entry.total_duration_ms / entry.execution_count as u64;

        Ok(result)
    }

    /// Get profiling report
    pub fn get_report(&self) -> String {
        let stats = self.stats.lock().unwrap();

        let mut report = String::from("Query Profiling Report\n");
        report.push_str("======================\n\n");

        for (name, stat) in stats.iter() {
            report.push_str(&format!(
                "{}: {} executions, avg {}ms, total {}ms\n",
                name, stat.execution_count, stat.avg_duration_ms, stat.total_duration_ms
            ));
        }

        report
    }
}
```

---

## 7. Summary

This document provides **concrete, implementable APIs** for the graph-based compiler:

**Key Components:**
1. **Compiler Phase API**: Common trait for all phases
2. **Pipeline Orchestrator**: Manages compilation workflow
3. **Query Builder**: Type-safe CozoDB query construction
4. **LLVM Integration**: Codegen from MIR to LLVM IR
5. **LSP Server**: IDE integration (completion, goto-def, find-refs)
6. **File Watcher**: Incremental compilation on file changes
7. **Debugging Tools**: Graph visualization, query profiling

**Next Document:** `04-RUBBER-DUCK-SIMULATIONS.md` (Concrete Examples)
