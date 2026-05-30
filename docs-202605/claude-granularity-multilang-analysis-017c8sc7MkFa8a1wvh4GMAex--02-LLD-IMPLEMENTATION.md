# Low-Level Design: Implementation Details

**Document Version:** 1.0
**Last Updated:** 2025-11-18
**Status:** Design Phase

## Executive Summary

This document provides **concrete implementation details** for the graph-based Rust compiler, including exact CozoDB schemas, Datalog queries, memory management strategies, and transaction patterns. Every section includes working code examples that can be directly implemented.

---

## 1. Complete CozoDB Schema Definitions

### 1.1 Lexing Phase Relations

#### File Relation
```datalog
:create file {
    id: Uuid =>           # Primary key
    path: String,         # Absolute file path
    hash: String,         # SHA256 of content
    last_modified: Int,   # Unix timestamp
    language: String,     # "rust", "python", etc.
    size_bytes: Int,      # File size
    line_count: Int       # Number of lines
}

# Indexes for fast lookups
::index create file:path { path }
::index create file:hash { hash }
```

#### Token Relation
```datalog
:create token {
    id: Uuid =>           # Primary key
    file_id: Uuid,        # Foreign key to file
    kind: String,         # TokenKind: "Ident", "Keyword", "Literal", etc.
    text: String,         # Raw lexeme (interned)
    span_start: Int,      # Start byte offset
    span_end: Int,        # End byte offset
    line: Int,            # Line number (1-indexed)
    column: Int,          # Column number (1-indexed)
    created_at: Int       # Unix timestamp
}

# Indexes for query performance
::index create token:file { file_id }
::index create token:position { file_id, span_start }
::index create token:kind { kind }
::index create token:line { file_id, line }
```

#### Token Sequence Relation
```datalog
:create token_sequence {
    from_token: Uuid,     # Previous token
    to_token: Uuid =>     # Next token (primary key)
    order_index: Int,     # Sequence number within file
    file_id: Uuid         # Denormalized for performance
}

::index create token_sequence:from { from_token }
::index create token_sequence:file { file_id, order_index }
```

#### Token Metadata (for errors/warnings)
```datalog
:create token_metadata {
    token_id: Uuid =>     # One-to-one with token
    error_kind: String?,  # "UnterminatedString", "InvalidNumber", etc.
    warning_kind: String?,
    suggestion: String?,  # Fix suggestion
    severity: String      # "error", "warning", "info"
}
```

#### String Interning Table
```datalog
:create interned_string {
    hash: Int =>          # FNV hash of string
    value: String,        # Actual string
    ref_count: Int        # How many tokens reference this
}

::index create interned_string:value { value }
```

### 1.2 Parsing Phase Relations

#### AST Node Relation
```datalog
:create ast_node {
    id: Uuid =>           # Primary key
    file_id: Uuid,        # File this node belongs to
    kind: String,         # "Function", "Struct", "Expr::BinaryOp", etc.
    depth: Int,           # Tree depth (root = 0)
    parent_id: Uuid?,     # Parent node (null for root)
    created_at: Int       # Unix timestamp
}

::index create ast_node:file { file_id }
::index create ast_node:parent { parent_id }
::index create ast_node:kind { kind }
```

#### AST Edge Relation (explicit parent-child)
```datalog
:create ast_edge {
    from_node: Uuid,      # Parent node
    to_node: Uuid =>      # Child node (primary key)
    edge_label: String,   # "body", "condition", "params", etc.
    child_index: Int,     # Order among siblings
    file_id: Uuid         # Denormalized
}

::index create ast_edge:from { from_node }
::index create ast_edge:to { to_node }
```

#### AST Attributes (flexible key-value storage)
```datalog
:create ast_attr {
    node_id: Uuid,        # Node this attribute belongs to
    key: String =>        # Attribute name
    value: Json,          # Value (string, number, bool, array, object)
    value_type: String    # "String", "Int", "Bool", "Ident", etc.
}

::index create ast_attr:node { node_id }
```

**Example AST Attributes:**
```json
// For Function node
{"name": "main", "visibility": "pub", "is_async": false}

// For BinaryOp node
{"operator": "+", "precedence": 10}

// For Literal node
{"lit_kind": "Int", "value": "42", "suffix": "i32"}
```

#### AST Span (source location)
```datalog
:create ast_span {
    node_id: Uuid =>      # One-to-one with ast_node
    span_start: Int,      # Start byte offset
    span_end: Int,        # End byte offset
    start_line: Int,      # Start line number
    start_col: Int,       # Start column
    end_line: Int,        # End line number
    end_col: Int          # End column
}
```

### 1.3 HIR (High-Level IR) Phase Relations

#### HIR Node Relation
```datalog
:create hir_node {
    id: Uuid =>           # Primary key
    file_id: Uuid,        # Source file
    kind: String,         # "Function", "Block", "Expr::Call", etc.
    ast_id: Uuid?,        # Original AST node (provenance)
    def_kind: String?,    # "fn", "struct", "enum", "const", null
    scope_id: Uuid?,      # Lexical scope this belongs to
    created_at: Int
}

::index create hir_node:file { file_id }
::index create hir_node:ast { ast_id }
::index create hir_node:scope { scope_id }
```

#### HIR Edge Relation
```datalog
:create hir_edge {
    from_node: Uuid,      # Source node
    to_node: Uuid,        # Target node
    edge_kind: String =>  # "contains", "calls", "returns", "references"
    edge_label: String?,  # Optional label (e.g., "argument_0")
    file_id: Uuid         # Denormalized
}

::index create hir_edge:from { from_node }
::index create hir_edge:to { to_node }
::index create hir_edge:kind { edge_kind }
```

#### HIR Definition Relation
```datalog
:create hir_def {
    id: Uuid =>           # Primary key
    name: String,         # Variable/function/type name
    hir_node_id: Uuid,    # HIR node where defined
    scope_id: Uuid,       # Scope of definition
    def_kind: String,     # "local", "fn", "struct", "enum", "trait"
    visibility: String,   # "pub", "pub(crate)", "private"
    span_start: Int,      # Definition site span
    span_end: Int
}

::index create hir_def:name { name }
::index create hir_def:scope { scope_id }
::index create hir_def:node { hir_node_id }
```

#### HIR Use Relation
```datalog
:create hir_use {
    id: Uuid =>           # Primary key
    def_id: Uuid,         # Which definition is used
    use_node_id: Uuid,    # HIR node where used
    use_kind: String,     # "read", "write", "call", "move"
    span_start: Int,      # Use site span
    span_end: Int
}

::index create hir_use:def { def_id }
::index create hir_use:node { use_node_id }
```

#### HIR Scope Relation
```datalog
:create hir_scope {
    id: Uuid =>           # Primary key
    parent_scope: Uuid?,  # Parent scope (null for module scope)
    file_id: Uuid,        # File this scope belongs to
    scope_kind: String,   # "module", "function", "block", "loop"
    hir_node_id: Uuid     # HIR node that creates this scope
}

::index create hir_scope:parent { parent_scope }
::index create hir_scope:file { file_id }
```

### 1.4 Type Checking Phase Relations

#### Type Node Relation
```datalog
:create type_node {
    id: Uuid =>           # Primary key
    kind: String,         # "Concrete", "Variable", "Generic", "Infer"
    type_repr: String,    # "i32", "String", "?T0", etc.
    created_at: Int
}

::index create type_node:repr { type_repr }
```

#### Type Constraint Relation
```datalog
:create type_constraint {
    id: Uuid =>           # Primary key
    left_type: Uuid,      # Left side of constraint
    right_type: Uuid,     # Right side of constraint
    constraint_kind: String,  # "Equals", "Subtype", "Implements"
    hir_node_id: Uuid?,   # HIR node that generated this constraint
    reason: String        # Why this constraint exists (for errors)
}

::index create type_constraint:left { left_type }
::index create type_constraint:right { right_type }
```

#### Type Unification Relation
```datalog
:create type_unification {
    type_var: Uuid,       # Type variable
    unified_type: Uuid => # Concrete type it unified to
    unified_at: Int       # When unification happened
}

::index create type_unification:var { type_var }
```

#### Type Error Relation
```datalog
:create type_error {
    id: Uuid =>           # Primary key
    hir_node_id: Uuid,    # Where error occurred
    error_kind: String,   # "TypeMismatch", "UnresolvedType", etc.
    expected_type: Uuid?, # Expected type
    actual_type: Uuid?,   # Actual type
    message: String,      # Human-readable error
    span_start: Int,      # Error location
    span_end: Int
}

::index create type_error:node { hir_node_id }
```

### 1.5 MIR (Mid-Level IR) Phase Relations

#### MIR Function Relation
```datalog
:create mir_fn {
    id: Uuid =>           # Primary key
    hir_fn_id: Uuid,      # Corresponding HIR function
    name: String,         # Function name
    entry_block: Uuid,    # Entry basic block
    return_type: Uuid,    # Type of return value
    param_count: Int      # Number of parameters
}

::index create mir_fn:hir { hir_fn_id }
::index create mir_fn:name { name }
```

#### MIR Basic Block Relation
```datalog
:create mir_bb {
    id: Uuid =>           # Primary key
    fn_id: Uuid,          # Function this block belongs to
    block_index: Int,     # bb0, bb1, bb2, ...
    terminator_id: Uuid?  # Terminator statement (null if incomplete)
}

::index create mir_bb:fn { fn_id }
::index create mir_bb:index { fn_id, block_index }
```

#### MIR Statement Relation
```datalog
:create mir_stmt {
    id: Uuid =>           # Primary key
    bb_id: Uuid,          # Basic block this belongs to
    stmt_index: Int,      # Position within block
    kind: String,         # "Assign", "Call", "Drop", "Nop"
    created_at: Int
}

::index create mir_stmt:bb { bb_id }
::index create mir_stmt:index { bb_id, stmt_index }
```

#### MIR Terminator Relation
```datalog
:create mir_term {
    id: Uuid =>           # Primary key (one per basic block)
    bb_id: Uuid,          # Basic block this terminates
    kind: String,         # "Goto", "Return", "SwitchInt", "Call", "Drop"
    created_at: Int
}

::index create mir_term:bb { bb_id }
```

#### MIR Control Flow Edge
```datalog
:create mir_cfg_edge {
    from_bb: Uuid,        # Source basic block
    to_bb: Uuid =>        # Target basic block
    edge_kind: String,    # "Unconditional", "True", "False", "SwitchCase"
    case_value: Int?      # For SwitchInt edges
}

::index create mir_cfg_edge:from { from_bb }
::index create mir_cfg_edge:to { to_bb }
```

#### MIR Local Variable Relation
```datalog
:create mir_local {
    id: Uuid =>           # Primary key
    fn_id: Uuid,          # Function this belongs to
    local_index: Int,     # _0, _1, _2, ... (SSA-like)
    type_id: Uuid,        # Type of this local
    debug_name: String?,  # Original variable name (if any)
    is_temp: Bool         # True for compiler temporaries
}

::index create mir_local:fn { fn_id }
::index create mir_local:index { fn_id, local_index }
```

#### MIR Place Relation (memory locations)
```datalog
:create mir_place {
    id: Uuid =>           # Primary key
    local_id: Uuid,       # Base local variable
    projection: Json,     # Array of projections: ["Field(0)", "Deref", etc.]
    type_id: Uuid         # Type of the place
}

::index create mir_place:local { local_id }
```

#### MIR Rvalue Relation (right-hand side of assignments)
```datalog
:create mir_rvalue {
    id: Uuid =>           # Primary key
    stmt_id: Uuid,        # Statement this belongs to
    kind: String,         # "Use", "BinaryOp", "Ref", "Aggregate", etc.
    operands: Json,       # Array of operand IDs
    metadata: Json        # Kind-specific data
}

::index create mir_rvalue:stmt { stmt_id }
```

### 1.6 Cross-Cutting Relations

#### Provenance Relation (tracks IR lineage)
```datalog
:create provenance {
    derived_id: Uuid,     # Derived entity (HIR node, MIR stmt, etc.)
    derived_kind: String, # "hir_node", "mir_stmt", etc.
    source_id: Uuid,      # Source entity (AST node, HIR node, etc.)
    source_kind: String => # "ast_node", "hir_node", etc.
    transform: String     # Transformation that created this
}

::index create provenance:derived { derived_id, derived_kind }
::index create provenance:source { source_id, source_kind }
```

#### Invalidation Relation (incremental compilation)
```datalog
:create invalidated {
    entity_id: Uuid,      # ID of invalidated entity
    entity_kind: String => # "token", "ast_node", "hir_node", etc.
    invalidated_at: Int,  # When invalidated
    reason: String        # Why invalidated
}

::index create invalidated:entity { entity_id, entity_kind }
```

---

## 2. Concrete Datalog Queries for Each Phase

### 2.1 Lexing Queries

#### Insert Tokens
```datalog
# Insert a batch of tokens (bulk insert for performance)
?[id, file_id, kind, text, span_start, span_end, line, column, created_at] <- [
    [gen_uuid(), $file_id, "Keyword", "fn", 0, 2, 1, 1, now()],
    [gen_uuid(), $file_id, "Whitespace", " ", 2, 3, 1, 3, now()],
    [gen_uuid(), $file_id, "Ident", "main", 3, 7, 1, 4, now()],
    # ... more tokens
]
:put token { id, file_id, kind, text, span_start, span_end, line, column, created_at }
```

#### Build Token Sequence
```datalog
# Create ordered sequence edges
?[from_token, to_token, order_index, file_id] <-
    *token{id: from_id, file_id, span_start: start1},
    *token{id: to_id, file_id, span_start: start2},
    start2 = start1 + 1,  # Adjacent tokens
    order_index = start1
:put token_sequence { from_token: from_id, to_token: to_id, order_index, file_id }
```

#### Query Tokens in Sequence
```datalog
# Get all tokens for a file in order
?[token_id, kind, text, span_start, span_end, line, column] :=
    *token{id: token_id, file_id: $file_id, kind, text, span_start, span_end, line, column},
    *token_sequence{to_token: token_id, order_index, file_id: $file_id}
    :order order_index
```

#### Find Identifiers by Name
```datalog
# Find all occurrences of a specific identifier
?[token_id, file_id, line, column] :=
    *token{id: token_id, file_id, kind: "Ident", text: $name, line, column}
```

#### Incremental: Invalidate Tokens for Changed File
```datalog
# Mark all tokens from a file as invalidated
?[entity_id, entity_kind, invalidated_at, reason] <-
    *token{id: entity_id, file_id: $changed_file_id},
    entity_kind = "token",
    invalidated_at = now(),
    reason = "file_changed"
:put invalidated { entity_id, entity_kind, invalidated_at, reason }

# Delete old tokens
:rm token { id } <-
    *token{id, file_id: $changed_file_id}

# Delete old sequences
:rm token_sequence { to_token } <-
    *token_sequence{to_token, file_id: $changed_file_id}
```

### 2.2 Parsing Queries

#### Insert AST Nodes (Example: Function Declaration)
```datalog
# Create function node
?[id, file_id, kind, depth, parent_id, created_at] <- [
    [$fn_id, $file_id, "Function", 0, null, now()]
]
:put ast_node { id, file_id, kind, depth, parent_id, created_at }

# Create function attributes
?[node_id, key, value, value_type] <- [
    [$fn_id, "name", "main", "String"],
    [$fn_id, "visibility", "pub", "String"],
    [$fn_id, "is_async", false, "Bool"]
]
:put ast_attr { node_id, key, value, value_type }

# Create child nodes (params, return type, body)
?[id, file_id, kind, depth, parent_id, created_at] <- [
    [$params_id, $file_id, "Params", 1, $fn_id, now()],
    [$body_id, $file_id, "Block", 1, $fn_id, now()]
]
:put ast_node { id, file_id, kind, depth, parent_id, created_at }

# Create edges
?[from_node, to_node, edge_label, child_index, file_id] <- [
    [$fn_id, $params_id, "params", 0, $file_id],
    [$fn_id, $body_id, "body", 1, $file_id]
]
:put ast_edge { from_node, to_node, edge_label, child_index, file_id }
```

#### Query AST Subtree
```datalog
# Get all descendants of a node (recursive query)
?[descendant_id, kind, depth] :=
    # Base case: direct children
    *ast_edge{from_node: $root_id, to_node: descendant_id},
    *ast_node{id: descendant_id, kind, depth}

?[descendant_id, kind, depth] :=
    # Recursive case: children of children
    *ast_edge{from_node: $root_id, to_node: child_id},
    *ast_edge{from_node: child_id, to_node: descendant_id},
    *ast_node{id: descendant_id, kind, depth}
    # CozoDB supports recursive queries natively
```

#### Find All Functions in a File
```datalog
?[fn_id, fn_name, visibility] :=
    *ast_node{id: fn_id, file_id: $file_id, kind: "Function"},
    *ast_attr{node_id: fn_id, key: "name", value: fn_name},
    *ast_attr{node_id: fn_id, key: "visibility", value: visibility}
```

### 2.3 HIR Lowering Queries

#### Create HIR Node from AST
```datalog
# Lower AST function to HIR
?[id, file_id, kind, ast_id, def_kind, scope_id, created_at] <-
    *ast_node{id: ast_id, file_id, kind: "Function"},
    *ast_attr{node_id: ast_id, key: "name", value: fn_name},
    id = gen_uuid(),
    hir_kind = "Function",
    def_kind = "fn",
    scope_id = $module_scope_id,
    created_at = now()
:put hir_node { id, file_id, kind: hir_kind, ast_id, def_kind, scope_id, created_at }

# Create definition
?[id, name, hir_node_id, scope_id, def_kind, visibility, span_start, span_end] <-
    *hir_node{id: hir_node_id, ast_id},
    *ast_attr{node_id: ast_id, key: "name", value: name},
    *ast_attr{node_id: ast_id, key: "visibility", value: visibility},
    *ast_span{node_id: ast_id, span_start, span_end},
    id = gen_uuid(),
    scope_id = $module_scope_id,
    def_kind = "fn"
:put hir_def { id, name, hir_node_id, scope_id, def_kind, visibility, span_start, span_end }
```

#### Build Def-Use Chains
```datalog
# Find all uses of a variable
?[use_id, use_node_id, use_kind, span_start, span_end] <-
    *hir_def{id: def_id, name: $var_name, scope_id},
    *hir_node{id: use_node_id, kind: "Expr::Path", scope_id: use_scope},
    # Check if use_scope is within def_scope (simplified)
    use_scope = scope_id,
    use_id = gen_uuid(),
    use_kind = "read",
    *ast_span{node_id: use_node_id, span_start, span_end}
:put hir_use { id: use_id, def_id, use_node_id, use_kind, span_start, span_end }
```

#### Desugar For Loop
```datalog
# Transform `for x in iter { body }` to HIR loop
# This is a complex transformation, shown conceptually

# Step 1: Create loop node
?[id, file_id, kind, ast_id, scope_id] <-
    *ast_node{id: ast_id, file_id, kind: "Expr::For"},
    id = gen_uuid(),
    kind = "Expr::Loop",
    scope_id = $current_scope
:put hir_node { id, file_id, kind, ast_id, scope_id, created_at: now() }

# Step 2: Create IntoIterator call
# Step 3: Create match expression
# Step 4: Create Some/None branches
# (Detailed expansion omitted for brevity, but follows same pattern)
```

### 2.4 Type Checking Queries

#### Generate Type Variables
```datalog
# Create a type variable for each unknown type
?[id, kind, type_repr, created_at] <-
    *hir_node{id: node_id, kind: "Expr"},
    # Only if no type assigned yet
    not *type_unification{node_id},
    id = gen_uuid(),
    kind = "Variable",
    type_repr = str_cat("?T", str(node_id)),  # Generate unique name
    created_at = now()
:put type_node { id, kind, type_repr, created_at }
```

#### Emit Type Constraints
```datalog
# Binary op constraint: lhs type = rhs type = result type
?[id, left_type, right_type, constraint_kind, hir_node_id, reason] <-
    *hir_node{id: hir_node_id, kind: "Expr::BinaryOp"},
    *hir_edge{from_node: hir_node_id, to_node: lhs_id, edge_label: "lhs"},
    *hir_edge{from_node: hir_node_id, to_node: rhs_id, edge_label: "rhs"},
    *type_unification{node_id: lhs_id, unified_type: left_type},
    *type_unification{node_id: rhs_id, unified_type: right_type},
    id = gen_uuid(),
    constraint_kind = "Equals",
    reason = "binary_op_operands"
:put type_constraint { id, left_type, right_type, constraint_kind, hir_node_id, reason }
```

#### Unification Algorithm (Simplified)
```datalog
# Unify equal constraints iteratively
# This is a simplified version; full algorithm requires fixpoint iteration

# If ?T0 = i32, then unify
?[type_var, unified_type, unified_at] <-
    *type_constraint{left_type, right_type, constraint_kind: "Equals"},
    *type_node{id: left_type, kind: "Variable"},
    *type_node{id: right_type, kind: "Concrete", type_repr},
    type_var = left_type,
    unified_type = right_type,
    unified_at = now()
:put type_unification { type_var, unified_type, unified_at }

# Propagate unification through constraints
?[type_var, unified_type, unified_at] <-
    *type_constraint{left_type: var1, right_type: var2, constraint_kind: "Equals"},
    *type_unification{type_var: var1, unified_type},
    *type_node{id: var2, kind: "Variable"},
    # var2 is not yet unified
    not *type_unification{type_var: var2},
    type_var = var2,
    unified_at = now()
:put type_unification { type_var, unified_type, unified_at }
```

### 2.5 MIR Building Queries

#### Create MIR Function
```datalog
# Lower HIR function to MIR
?[id, hir_fn_id, name, entry_block, return_type, param_count] <-
    *hir_node{id: hir_fn_id, kind: "Function"},
    *hir_def{hir_node_id: hir_fn_id, name},
    # Create entry basic block
    entry_block = gen_uuid(),
    # Get return type from type checking
    *type_unification{node_id: hir_fn_id, unified_type: return_type},
    # Count parameters
    param_count = 0,  # Placeholder
    id = gen_uuid()
:put mir_fn { id, hir_fn_id, name, entry_block, return_type, param_count }

# Create entry basic block
?[id, fn_id, block_index, terminator_id] <-
    *mir_fn{id: fn_id, entry_block: id},
    block_index = 0,
    terminator_id = null
:put mir_bb { id, fn_id, block_index, terminator_id }
```

#### Build Control Flow Graph
```datalog
# Lower if-else to CFG
# HIR: if condition { true_branch } else { false_branch }
# MIR:
#   bb0: switchInt(condition) -> [bb1, bb2]
#   bb1: true_branch; goto bb3
#   bb2: false_branch; goto bb3
#   bb3: ...

# Create basic blocks
?[id, fn_id, block_index, terminator_id] <- [
    [$bb0_id, $fn_id, 0, null],
    [$bb1_id, $fn_id, 1, null],
    [$bb2_id, $fn_id, 2, null],
    [$bb3_id, $fn_id, 3, null]
]
:put mir_bb { id, fn_id, block_index, terminator_id }

# Create CFG edges
?[from_bb, to_bb, edge_kind, case_value] <- [
    [$bb0_id, $bb1_id, "True", null],
    [$bb0_id, $bb2_id, "False", null],
    [$bb1_id, $bb3_id, "Unconditional", null],
    [$bb2_id, $bb3_id, "Unconditional", null]
]
:put mir_cfg_edge { from_bb, to_bb, edge_kind, case_value }
```

#### Query Reachable Blocks (for Dead Code Elimination)
```datalog
# Find all blocks reachable from entry
?[reachable_bb] :=
    *mir_fn{id: $fn_id, entry_block},
    reachable_bb = entry_block

?[reachable_bb] :=
    ?[from_bb],  # Recursive query
    *mir_cfg_edge{from_bb, to_bb: reachable_bb}
```

---

## 3. Memory Management Strategies

### 3.1 String Interning

**Problem:** Tokens store text, leading to duplicate strings (e.g., "fn" appears many times)

**Solution:** Intern strings in a global table, store only hash in tokens

**Implementation:**
```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct StringInterner {
    strings: Arc<RwLock<HashMap<u64, String>>>,
}

impl StringInterner {
    pub fn intern(&self, s: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hasher);
        let hash = hasher.finish();

        let mut strings = self.strings.write().unwrap();
        strings.entry(hash).or_insert_with(|| s.to_string());
        hash
    }

    pub fn get(&self, hash: u64) -> Option<String> {
        let strings = self.strings.read().unwrap();
        strings.get(&hash).cloned()
    }
}
```

**CozoDB Integration:**
```datalog
# Store interned strings
?[hash, value, ref_count] <- [
    [hash_of("fn"), "fn", 150],      # "fn" appears 150 times
    [hash_of("main"), "main", 10],   # "main" appears 10 times
    # ...
]
:put interned_string { hash, value, ref_count }

# Update ref_count when adding tokens
?[hash, value, ref_count] <-
    *interned_string{hash: $str_hash, value, ref_count: old_count},
    ref_count = old_count + 1
:update interned_string { hash: $str_hash, ref_count }
```

**Memory Savings:**
- Without interning: 100K tokens × 10 bytes/string = 1 MB
- With interning: 1K unique strings × 10 bytes + 100K tokens × 8 bytes hash = 0.81 MB
- **Savings: 20%**

### 3.2 Lazy Loading Strategy

**Principle:** Only load data when queried, evict when not needed

**Implementation:**
```rust
pub struct LazyAstNode {
    id: Uuid,
    db: Arc<DbInstance>,
    cached_data: Option<AstNodeData>,
}

impl LazyAstNode {
    pub fn load(&mut self) -> Result<&AstNodeData> {
        if self.cached_data.is_none() {
            let query = r#"
                ?[kind, depth, parent_id] :=
                    *ast_node{id: $id, kind, depth, parent_id}
            "#;
            let result = self.db.run_script(query, params!{"id" => self.id})?;
            self.cached_data = Some(parse_result(result)?);
        }
        Ok(self.cached_data.as_ref().unwrap())
    }

    pub fn evict(&mut self) {
        self.cached_data = None;
    }
}
```

**Usage:**
```rust
let mut node = LazyAstNode::new(node_id, db.clone());
// No data loaded yet (0 bytes)

let data = node.load()?;  // Load on demand (~200 bytes)
// Use data...

node.evict();  // Free memory (back to 0 bytes)
```

### 3.3 Batch Operations

**Problem:** Inserting nodes one-by-one is slow (1 transaction per insert)

**Solution:** Batch inserts in single transaction

**Bad (1000 transactions):**
```rust
for token in tokens {
    db.run_script(
        "?[id, kind, text] <- [[$id, $kind, $text]] :put token { id, kind, text }",
        params!{"id" => token.id, "kind" => token.kind, "text" => token.text}
    )?;
}
```

**Good (1 transaction):**
```rust
let values: Vec<DataValue> = tokens.iter().map(|t| {
    DataValue::List(vec![
        t.id.into(),
        t.kind.into(),
        t.text.into(),
    ])
}).collect();

let query = format!(
    "?[id, kind, text] <- $data :put token {{ id, kind, text }}"
);
db.run_script(&query, params!{"data" => values})?;
```

**Performance:**
- 1000 individual inserts: ~2 seconds
- 1 batch insert: ~20 milliseconds
- **Speedup: 100x**

### 3.4 Database Caching Configuration

**CozoDB Cache Settings:**
```rust
use cozo::{DbInstance, DbConfig};

let config = DbConfig {
    cache_size: 512 * 1024 * 1024,  // 512 MB cache
    write_buffer_size: 64 * 1024 * 1024,  // 64 MB write buffer
    max_open_files: 1000,
    ..Default::default()
};

let db = DbInstance::new("rocksdb", "/path/to/db", config)?;
```

**Cache Strategy:**
- Hot queries (IDE features): Always cached
- Cold queries (historical analysis): Evicted when needed
- LRU eviction policy

---

## 4. Transaction Boundaries and ACID Properties

### 4.1 Transaction Patterns

#### Pattern 1: Single-Phase Transaction
```rust
pub fn lex_file(db: &DbInstance, file_path: &str) -> Result<()> {
    // Start transaction (implicit in CozoDB)
    let file_id = Uuid::new_v4();

    // Step 1: Insert file metadata
    db.run_script(r#"
        ?[id, path, hash, last_modified, language] <- [
            [$file_id, $path, $hash, $timestamp, "rust"]
        ]
        :put file { id, path, hash, last_modified, language }
    "#, params!{
        "file_id" => file_id,
        "path" => file_path,
        "hash" => compute_hash(file_path),
        "timestamp" => now(),
    })?;

    // Step 2: Insert tokens (batch)
    let tokens = tokenize(file_path)?;
    let token_data = prepare_token_data(tokens, file_id);
    db.run_script(r#"
        ?[id, file_id, kind, text, span_start, span_end, line, column, created_at] <- $tokens
        :put token { id, file_id, kind, text, span_start, span_end, line, column, created_at }
    "#, params!{"tokens" => token_data})?;

    // Step 3: Build sequence
    db.run_script(r#"
        ?[from_token, to_token, order_index, file_id] <-
            *token{id: from_id, file_id: $file_id, span_end: end1},
            *token{id: to_id, file_id: $file_id, span_start: start2},
            start2 = end1,
            order_index = end1
        :put token_sequence { from_token: from_id, to_token: to_id, order_index, file_id }
    "#, params!{"file_id" => file_id})?;

    // If we reach here, transaction commits automatically
    Ok(())
}
```

#### Pattern 2: Multi-Phase with Rollback
```rust
pub fn compile_file(db: &DbInstance, file_path: &str) -> Result<()> {
    // Phase 1: Lex
    if let Err(e) = lex_file(db, file_path) {
        eprintln!("Lexing failed: {}", e);
        // Transaction already rolled back
        return Err(e);
    }

    // Phase 2: Parse
    if let Err(e) = parse_file(db, file_path) {
        eprintln!("Parsing failed: {}", e);
        // Question: Do we rollback lexing too?
        // Answer: Depends on use case. For IDE, keep tokens even if parse fails.
        return Err(e);
    }

    // Phase 3: HIR lowering
    if let Err(e) = lower_to_hir(db, file_path) {
        eprintln!("HIR lowering failed: {}", e);
        return Err(e);
    }

    Ok(())
}
```

### 4.2 ACID Properties Exploited

#### Atomicity
**Scenario:** Inserting 10K tokens
**Without Atomicity:** If crash after 5K inserts, DB has incomplete data
**With Atomicity:** Either all 10K tokens inserted, or none

**Implementation:**
```rust
// CozoDB transactions are atomic by default
db.run_script(r#"
    ?[id, kind, text] <- $tokens
    :put token { id, kind, text }
"#, params!{"tokens" => token_data})?;
// Either all tokens inserted, or none (if error/crash)
```

#### Consistency
**Scenario:** Maintaining foreign key integrity (token.file_id → file.id)
**Without Consistency:** Dangling references possible
**With Consistency:** Use constraints to enforce

**Implementation:**
```datalog
# Define foreign key constraint (CozoDB syntax may vary)
::constraint token:file_fk {
    *token{file_id} => *file{id: file_id}
}
```

#### Isolation
**Scenario:** Multiple files being compiled in parallel
**Without Isolation:** Thread 1 reads incomplete data from Thread 2
**With Isolation:** Each thread sees consistent snapshot

**Implementation:**
```rust
use std::thread;

// Compile multiple files in parallel
let handles: Vec<_> = files.iter().map(|file_path| {
    let db = db.clone();  // CozoDB supports multi-threaded access
    let path = file_path.clone();
    thread::spawn(move || {
        lex_file(&db, &path)  // Each thread has isolated transaction
    })
}).collect();

for handle in handles {
    handle.join().unwrap()?;
}
```

#### Durability
**Scenario:** Crash during compilation
**Without Durability:** All progress lost
**With Durability:** Resume from last committed phase

**Implementation:**
```rust
pub fn compile_with_resume(db: &DbInstance, file_path: &str) -> Result<()> {
    let file_id = get_or_create_file_id(db, file_path)?;

    // Check what phases have completed
    let has_tokens = db.run_script(r#"
        ?[count(id)] := *token{id, file_id: $file_id}
    "#, params!{"file_id" => file_id})?.rows.first()
        .and_then(|row| row.first())
        .map(|v| v.as_int().unwrap_or(0) > 0)
        .unwrap_or(false);

    if !has_tokens {
        println!("Resuming from lexing phase");
        lex_file(db, file_path)?;
    }

    let has_ast = db.run_script(r#"
        ?[count(id)] := *ast_node{id, file_id: $file_id}
    "#, params!{"file_id" => file_id})?.rows.first()
        .and_then(|row| row.first())
        .map(|v| v.as_int().unwrap_or(0) > 0)
        .unwrap_or(false);

    if !has_ast {
        println!("Resuming from parsing phase");
        parse_file(db, file_path)?;
    }

    // ... and so on for other phases
    Ok(())
}
```

---

## 5. Caching and Invalidation Strategies

### 5.1 Query Result Caching

**Application-Level Cache:**
```rust
use lru::LruCache;
use std::sync::{Arc, Mutex};

pub struct QueryCache {
    cache: Arc<Mutex<LruCache<String, CozoResult>>>,
}

impl QueryCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
        }
    }

    pub fn get_or_query<F>(&self, key: &str, query_fn: F) -> Result<CozoResult>
    where
        F: FnOnce() -> Result<CozoResult>
    {
        let mut cache = self.cache.lock().unwrap();

        if let Some(result) = cache.get(key) {
            return Ok(result.clone());
        }

        drop(cache);  // Release lock before expensive query

        let result = query_fn()?;

        let mut cache = self.cache.lock().unwrap();
        cache.put(key.to_string(), result.clone());

        Ok(result)
    }
}
```

**Usage:**
```rust
let cache = QueryCache::new(1000);

// Query with caching
let result = cache.get_or_query("all_functions", || {
    db.run_script(r#"
        ?[fn_id, fn_name] := *hir_def{id: fn_id, def_kind: "fn", name: fn_name}
    "#, Default::default())
})?;
```

### 5.2 Incremental Invalidation

**Strategy:** When a file changes, invalidate only affected subgraphs

**Algorithm:**
```rust
pub fn invalidate_file_changes(db: &DbInstance, file_id: Uuid) -> Result<()> {
    // Step 1: Mark tokens as invalidated
    db.run_script(r#"
        ?[entity_id, entity_kind, invalidated_at, reason] <-
            *token{id: entity_id, file_id: $file_id},
            entity_kind = "token",
            invalidated_at = now(),
            reason = "file_changed"
        :put invalidated { entity_id, entity_kind, invalidated_at, reason }
    "#, params!{"file_id" => file_id})?;

    // Step 2: Propagate to AST
    db.run_script(r#"
        ?[entity_id, entity_kind, invalidated_at, reason] <-
            *ast_node{id: entity_id, file_id: $file_id},
            entity_kind = "ast_node",
            invalidated_at = now(),
            reason = "source_invalidated"
        :put invalidated { entity_id, entity_kind, invalidated_at, reason }
    "#, params!{"file_id" => file_id})?;

    // Step 3: Propagate to HIR (only nodes derived from invalidated AST)
    db.run_script(r#"
        ?[entity_id, entity_kind, invalidated_at, reason] <-
            *invalidated{entity_id: ast_id, entity_kind: "ast_node"},
            *hir_node{id: entity_id, ast_id},
            entity_kind = "hir_node",
            invalidated_at = now(),
            reason = "ast_invalidated"
        :put invalidated { entity_id, entity_kind, invalidated_at, reason }
    "#, Default::default())?;

    // Step 4: Propagate to types and MIR (similarly)
    // ...

    Ok(())
}
```

**Incremental Recomputation:**
```rust
pub fn recompile_invalidated(db: &DbInstance) -> Result<()> {
    // Find all invalidated files
    let files = db.run_script(r#"
        ?[file_id] :=
            *invalidated{entity_id: token_id, entity_kind: "token"},
            *token{id: token_id, file_id},
            distinct file_id
    "#, Default::default())?;

    for row in files.rows {
        let file_id: Uuid = row[0].try_into()?;

        // Delete invalidated data
        db.run_script(r#"
            :rm token { id } <-
                *invalidated{entity_id: id, entity_kind: "token"}
            :rm ast_node { id } <-
                *invalidated{entity_id: id, entity_kind: "ast_node"}
            # ... delete from other relations
        "#, Default::default())?;

        // Recompile
        let file_path = get_file_path(db, file_id)?;
        compile_file(db, &file_path)?;

        // Clear invalidation markers
        db.run_script(r#"
            :rm invalidated { entity_id, entity_kind } <-
                *token{id, file_id: $file_id},
                entity_id = id,
                entity_kind = "token"
            # ... clear from other entity kinds
        "#, params!{"file_id" => file_id})?;
    }

    Ok(())
}
```

### 5.3 Cache Warming Strategy

**Concept:** Pre-load frequently accessed data into cache

**Implementation:**
```rust
pub fn warm_cache(db: &DbInstance, cache: &QueryCache) -> Result<()> {
    // Warm up: all function names
    cache.get_or_query("all_functions", || {
        db.run_script(r#"
            ?[fn_id, fn_name] := *hir_def{id: fn_id, def_kind: "fn", name: fn_name}
        "#, Default::default())
    })?;

    // Warm up: all type definitions
    cache.get_or_query("all_types", || {
        db.run_script(r#"
            ?[type_id, type_repr] := *type_node{id: type_id, kind: "Concrete", type_repr}
        "#, Default::default())
    })?;

    // Warm up: current file's AST (for IDE)
    let current_file = get_current_file()?;
    cache.get_or_query(&format!("ast:{}", current_file), || {
        db.run_script(r#"
            ?[node_id, kind] := *ast_node{id: node_id, file_id: $file_id, kind}
        "#, params!{"file_id" => current_file})
    })?;

    Ok(())
}
```

---

## 6. Parallel Compilation Design

### 6.1 File-Level Parallelism

**Strategy:** Compile independent files in parallel

**Implementation:**
```rust
use rayon::prelude::*;

pub fn compile_project_parallel(db: &DbInstance, file_paths: &[String]) -> Result<()> {
    // Build dependency graph first
    let dep_graph = build_dependency_graph(file_paths)?;

    // Topological sort to find compilation order
    let layers = topological_layers(&dep_graph)?;

    // Compile layer by layer (files in same layer are independent)
    for layer in layers {
        layer.par_iter().try_for_each(|file_path| {
            let db = db.clone();  // CozoDB is thread-safe
            compile_file(&db, file_path)
        })?;
    }

    Ok(())
}
```

### 6.2 Phase-Level Parallelism

**Strategy:** Within a phase, parallelize independent operations

**Example: Parallel Type Checking**
```rust
pub fn type_check_parallel(db: &DbInstance) -> Result<()> {
    // Get all functions
    let functions = db.run_script(r#"
        ?[fn_id] := *hir_def{id: fn_id, def_kind: "fn"}
    "#, Default::default())?;

    // Type check each function in parallel
    functions.rows.par_iter().try_for_each(|row| {
        let fn_id: Uuid = row[0].try_into()?;
        let db = db.clone();
        type_check_function(&db, fn_id)
    })?;

    Ok(())
}
```

### 6.3 Transaction Isolation for Parallelism

**Challenge:** Parallel transactions must not interfere

**Solution:** Use CozoDB's MVCC (Multi-Version Concurrency Control)

**Example:**
```rust
// Thread 1: Compile file A
thread::spawn(move || {
    db.run_script(r#"
        ?[id, file_id, kind] <- [[$token_id, $file_a_id, "Ident"]]
        :put token { id, file_id, kind }
    "#, params!{"token_id" => t1_id, "file_a_id" => file_a})
});

// Thread 2: Compile file B (simultaneously)
thread::spawn(move || {
    db.run_script(r#"
        ?[id, file_id, kind] <- [[$token_id, $file_b_id, "Ident"]]
        :put token { id, file_id, kind }
    "#, params!{"token_id" => t2_id, "file_b_id" => file_b})
});

// Both transactions can proceed without blocking
// CozoDB ensures serializability
```

---

## 7. Error Handling and Recovery

### 7.1 Error Storage in Graph

**Principle:** Errors are first-class entities in the graph

**Schema:**
```datalog
:create compilation_error {
    id: Uuid =>
    phase: String,        # "lexing", "parsing", "type_checking", etc.
    error_kind: String,   # "SyntaxError", "TypeError", etc.
    entity_id: Uuid?,     # Associated entity (token, AST node, etc.)
    message: String,      # Human-readable error
    span_start: Int,      # Error location
    span_end: Int,
    severity: String,     # "error", "warning", "info"
    created_at: Int
}
```

### 7.2 Partial Compilation with Errors

**Concept:** Continue compilation even if errors occur, collect all errors

**Implementation:**
```rust
pub fn parse_file_tolerant(db: &DbInstance, file_id: Uuid) -> Result<Vec<CompilationError>> {
    let mut errors = Vec::new();

    // Get tokens
    let tokens = query_tokens(db, file_id)?;

    // Parse with error recovery
    let mut parser = Parser::new(tokens);
    while !parser.is_at_end() {
        match parser.parse_item() {
            Ok(ast_node) => {
                // Insert AST node
                insert_ast_node(db, &ast_node)?;
            }
            Err(e) => {
                // Record error but continue
                errors.push(e);
                insert_error(db, &e)?;

                // Synchronize to next item
                parser.synchronize();
            }
        }
    }

    Ok(errors)
}
```

**Insert Error:**
```rust
fn insert_error(db: &DbInstance, error: &CompilationError) -> Result<()> {
    db.run_script(r#"
        ?[id, phase, error_kind, message, span_start, span_end, severity, created_at] <- [
            [$id, $phase, $error_kind, $message, $span_start, $span_end, $severity, now()]
        ]
        :put compilation_error { id, phase, error_kind, message, span_start, span_end, severity, created_at }
    "#, params!{
        "id" => error.id,
        "phase" => error.phase,
        "error_kind" => error.error_kind,
        "message" => error.message,
        "span_start" => error.span.start,
        "span_end" => error.span.end,
        "severity" => error.severity,
    })?;
    Ok(())
}
```

### 7.3 Querying Errors

**All errors in a file:**
```datalog
?[error_id, phase, error_kind, message, span_start, span_end, severity] :=
    *compilation_error{id: error_id, phase, error_kind, message, span_start, span_end, severity, entity_id},
    *token{id: entity_id, file_id: $file_id}  # Assuming entity is a token
```

**Errors by severity:**
```datalog
?[count(id)] :=
    *compilation_error{id, severity: "error"}
```

### 7.4 Crash Recovery

**Scenario:** Compiler crashes mid-compilation

**Recovery Strategy:**
1. Check which files have incomplete data
2. Delete incomplete data
3. Resume compilation from last complete phase

**Implementation:**
```rust
pub fn recover_from_crash(db: &DbInstance) -> Result<()> {
    // Find files with tokens but no AST (parsing crashed)
    let incomplete_files = db.run_script(r#"
        ?[file_id] :=
            *token{file_id},
            not *ast_node{file_id}
    "#, Default::default())?;

    for row in incomplete_files.rows {
        let file_id: Uuid = row[0].try_into()?;
        println!("Recovering file: {}", file_id);

        // Delete incomplete tokens
        db.run_script(r#"
            :rm token { id } <- *token{id, file_id: $file_id}
        "#, params!{"file_id" => file_id})?;

        // Restart compilation
        let file_path = get_file_path(db, file_id)?;
        compile_file(db, &file_path)?;
    }

    Ok(())
}
```

---

## 8. Optimization: Graph Rewrite Rules

### 8.1 Constant Propagation

**Rule:** If `x = const` and `y = x`, then `y = const`

**Datalog Implementation:**
```datalog
# Find constant assignments
?[local_id, const_value] :=
    *mir_stmt{id: stmt_id, kind: "Assign"},
    *mir_place{stmt_id, local_id},
    *mir_rvalue{stmt_id, kind: "Const", metadata},
    const_value = get(metadata, "value")

# Find uses of constant
?[use_stmt_id, const_value] :=
    ?[local_id, const_value],  # Constant definition
    *mir_rvalue{stmt_id: use_stmt_id, kind: "Use", operands},
    operands = [local_id]  # Uses the constant local

# Rewrite uses to constant
:replace mir_rvalue {
    stmt_id: use_stmt_id,
    kind: "Const",
    operands: [],
    metadata: {"value": const_value}
} <-
    ?[use_stmt_id, const_value]
```

### 8.2 Dead Code Elimination

**Rule:** Remove unreachable basic blocks

**Datalog Implementation:**
```datalog
# Find reachable blocks (from entry)
?[reachable_bb] :=
    *mir_fn{id: $fn_id, entry_block: reachable_bb}

?[reachable_bb] :=
    ?[from_bb],  # Recursive
    *mir_cfg_edge{from_bb, to_bb: reachable_bb}

# Find unreachable blocks
?[unreachable_bb] :=
    *mir_bb{id: unreachable_bb, fn_id: $fn_id},
    not ?[unreachable_bb]  # Not in reachable set

# Delete unreachable blocks
:rm mir_bb { id } <-
    ?[id]  # Unreachable blocks
```

### 8.3 Function Inlining

**Rule:** Replace function call with function body

**Conceptual (complex to implement fully):**
```datalog
# Find small functions (candidate for inlining)
?[fn_id, body_size] :=
    *mir_fn{id: fn_id},
    *mir_bb{fn_id, id: bb_id},
    *mir_stmt{bb_id, id: stmt_id},
    body_size = count(stmt_id),
    body_size < 10  # Inline only small functions

# Find call sites
?[call_stmt_id, callee_fn_id] :=
    *mir_stmt{id: call_stmt_id, kind: "Call"},
    *mir_rvalue{stmt_id: call_stmt_id, kind: "Call", metadata},
    callee_fn_id = get(metadata, "callee_id"),
    ?[callee_fn_id, body_size]  # Callee is inlinable

# Inline: Copy callee's MIR into caller
# (This is complex and involves graph copying, omitted for brevity)
```

---

## 9. Performance Optimization Techniques

### 9.1 Index Design

**Principle:** Create indexes for frequently queried columns

**Example Indexes:**
```datalog
# Frequently query tokens by file
::index create token:file { file_id }

# Frequently query AST nodes by kind
::index create ast_node:kind { kind }

# Frequently query HIR uses by definition
::index create hir_use:def { def_id }

# Composite index for MIR queries
::index create mir_stmt:bb_index { bb_id, stmt_index }
```

**Performance Impact:**
- Without index: O(n) scan through all tokens
- With index: O(log n) lookup by file_id
- **Speedup: 100x-1000x for large databases**

### 9.2 Query Optimization Tips

**Tip 1: Filter early**
```datalog
# Bad: Filter after join
?[token_id, kind] :=
    *token{id: token_id, file_id, kind},
    *file{id: file_id, path: $path},
    kind = "Ident"

# Good: Filter before join
?[token_id, kind] :=
    *token{id: token_id, file_id, kind: "Ident"},
    *file{id: file_id, path: $path}
```

**Tip 2: Use aggregation wisely**
```datalog
# Bad: Count all, then filter
?[kind, count] :=
    *token{id, kind},
    count = count(id),
    count > 100

# Good: Filter first, then count
?[kind, count] :=
    *token{id, kind, file_id: $file_id},  # Filter by file first
    count = count(id),
    count > 100
```

**Tip 3: Avoid Cartesian products**
```datalog
# Bad: Cartesian product
?[token1_id, token2_id] :=
    *token{id: token1_id},
    *token{id: token2_id},
    token1_id != token2_id

# Good: Use meaningful join condition
?[prev_id, next_id] :=
    *token_sequence{from_token: prev_id, to_token: next_id}
```

### 9.3 Denormalization for Performance

**Trade-off:** Store redundant data for faster queries

**Example:**
```datalog
# Normalized: file_id in token, need join to get file path
?[token_id, file_path] :=
    *token{id: token_id, file_id},
    *file{id: file_id, path: file_path}

# Denormalized: file_path in token_sequence (redundant)
:create token_sequence {
    from_token: Uuid,
    to_token: Uuid =>
    order_index: Int,
    file_id: Uuid,
    file_path: String  # Denormalized for faster access
}

# Now can query without join
?[token_id, file_path] :=
    *token_sequence{to_token: token_id, file_path}
```

**Cost:** Extra storage (~50 bytes per row)
**Benefit:** 2x faster queries (no join)

---

## 10. Testing Strategy

### 10.1 Unit Tests for Queries

**Test each query in isolation:**
```rust
#[test]
fn test_token_sequence_query() {
    let db = create_test_db();

    // Insert test data
    db.run_script(r#"
        ?[id, file_id, kind, text, span_start, span_end] <- [
            ["t1", "f1", "Ident", "foo", 0, 3],
            ["t2", "f1", "Ident", "bar", 4, 7]
        ]
        :put token { id, file_id, kind, text, span_start, span_end }
    "#, Default::default()).unwrap();

    db.run_script(r#"
        ?[from_token, to_token, order_index] <- [
            ["t1", "t2", 0]
        ]
        :put token_sequence { from_token, to_token, order_index }
    "#, Default::default()).unwrap();

    // Query
    let result = db.run_script(r#"
        ?[from_id, to_id] :=
            *token_sequence{from_token: from_id, to_token: to_id}
    "#, Default::default()).unwrap();

    // Assert
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], DataValue::from("t1"));
    assert_eq!(result.rows[0][1], DataValue::from("t2"));
}
```

### 10.2 Integration Tests for Phases

**Test entire phase end-to-end:**
```rust
#[test]
fn test_lex_parse_integration() {
    let db = create_test_db();
    let source = "fn main() {}";

    // Lex
    lex_file(&db, "test.rs", source).unwrap();

    // Verify tokens
    let tokens = db.run_script(r#"
        ?[kind, text] :=
            *token{kind, text},
            *token_sequence{to_token: id, order_index},
            :order order_index
    "#, Default::default()).unwrap();

    assert_eq!(tokens.rows.len(), 5);  // fn, main, (, ), {, }

    // Parse
    parse_file(&db, "test.rs").unwrap();

    // Verify AST
    let ast = db.run_script(r#"
        ?[kind] := *ast_node{kind: "Function"}
    "#, Default::default()).unwrap();

    assert_eq!(ast.rows.len(), 1);
}
```

### 10.3 Property-Based Testing

**Test invariants across random inputs:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_lex_preserves_source(source in "\\PC{0,100}") {
        let db = create_test_db();
        lex_file(&db, "test.rs", &source).unwrap();

        // Reconstruct source from tokens
        let reconstructed = db.run_script(r#"
            ?[text] :=
                *token{text},
                *token_sequence{to_token: id, order_index},
                :order order_index
        "#, Default::default()).unwrap()
            .rows.iter()
            .map(|row| row[0].as_str().unwrap())
            .collect::<String>();

        // Property: Lexing should preserve source
        assert_eq!(source, reconstructed);
    }
}
```

---

## 11. Summary

This LLD document provides **concrete, implementable details** for the graph-based Rust compiler:

**Key Takeaways:**
1. **Complete CozoDB schemas** for all compilation phases
2. **Real Datalog queries** for transformations
3. **Memory management** via interning, lazy loading, batching
4. **ACID transactions** for reliability
5. **Incremental compilation** via invalidation
6. **Parallel execution** with isolation
7. **Error handling** as first-class graph entities
8. **Optimization** via graph rewrite rules

**Next Steps:**
- Implement lexing phase with these schemas
- Benchmark query performance
- Build parser on top of token graph
- Iterate based on real-world usage

**Next Document:** `03-INTERFACES.md` (API Design)
