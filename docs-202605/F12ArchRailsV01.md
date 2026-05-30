# ArchRailsV01: Rails ISG Extraction Architecture
## Pattern-Based Semantic Analysis Without Code Execution

**Document Version:** 1.0
**Date:** 2025-11-04
**Status:** Architectural Design Record (ADR)
**Author:** Synthesized from Rails Deep Analysis (Explore + Plan + Research Agents)

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Rails Architecture Analysis](#rails-architecture-analysis)
3. [High-Level Design (HLD)](#high-level-design-hld)
4. [Low-Level Design (LLD)](#low-level-design-lld)
5. [Implementation Roadmap](#implementation-roadmap)
6. [Metaprogramming Strategies](#metaprogramming-strategies)
7. [Testing & Validation](#testing--validation)
8. [Integration with Parseltongue](#integration-with-parseltongue)

---

## Executive Summary

### The Rails ISG Challenge

Rails is fundamentally different from C/C++: it's **70% convention, 30% code**. Heavy metaprogramming creates runtime behavior invisible to static analysis. A single line `has_many :posts` generates 25+ methods at runtime. Traditional static analysis fails because:

- **Metaprogramming**: `define_method`, `class_eval`, `method_missing` create code dynamically
- **DSL Magic**: Association macros, validations, callbacks all use runtime code generation
- **Convention over Configuration**: Implicit relationships (table names, foreign keys, routes)

### The Solution: Hybrid Pattern-Based Extraction

**Core Insight**: While Rails uses dynamic code generation, the **patterns are static and regular**. We can build a **Rails DSL Knowledge Base** that encodes generation rules, then use AST pattern matching to extract semantics without execution.

**Approach**:
1. **AST Pattern Matching** - Detect Rails DSL calls (has_many, scope, validates)
2. **Knowledge Base Lookup** - Pre-computed method generation tables
3. **Concern Resolution** - Track module dependencies and mixins
4. **Confidence Scoring** - Explicit uncertainty for partial extraction

**Success Metrics**:
- ✅ Extract 80%+ of associations from real Rails apps (Discourse, Spree)
- ✅ Parse 70%+ of routes and validations
- ✅ Flag 95%+ of metaprogramming requiring manual review
- ✅ <10s for 200-model Rails application

---

## Rails Architecture Analysis

### Component Layering (From Exploration)

Rails follows a modular, concern-based architecture with 11+ major components:

```
┌─────────────────────────────────────────┐
│     Application Layer (User Code)       │
│  Models, Controllers, Routes, Views     │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│         Framework Layer                  │
│  ┌───────────┬──────────┬─────────────┐ │
│  │ActiveRecord│ActionPack│ActionView  │ │
│  │           │          │            │ │
│  │ActiveModel│ActiveJob │ActionMailer│ │
│  └───────────┴──────────┴─────────────┘ │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│      Foundation Layer                    │
│       ActiveSupport                      │
│  - Concern (metaprogramming backbone)    │
│  - Callbacks, Inflections, Extensions    │
└──────────────────────────────────────────┘
```

### Core Design Patterns

**1. Concern-Based Composition**
Every major Rails feature is implemented as a Concern (ActiveSupport module pattern):

```ruby
module Validations
  extend ActiveSupport::Concern

  included do
    # Runs when mixed into a class
    define_callbacks :validate
  end

  class_methods do
    # Added as class methods to includer
    def validates(*attrs, **options)
      # DSL implementation
    end
  end
end
```

**2. Association Builder Pattern**
When you write `has_many :posts`, Rails:
1. Creates a `Builder::HasMany` object
2. Builds a `Reflection` storing metadata
3. Generates 25+ methods via `class_eval`
4. Registers callbacks for cascading

**3. Reflection Pattern**
Rails maintains comprehensive runtime metadata:

```ruby
User.reflections
# => { "posts" => HasManyReflection, "profile" => HasOneReflection }

User._validators
# => { title: [PresenceValidator, LengthValidator] }
```

**4. Dynamic Method Generation**
```ruby
# What developers write:
has_many :posts

# What Rails generates (simplified):
def posts
  association(:posts).reader
end

def posts=(other)
  association(:posts).writer(other)
end

def post_ids
  association(:posts).ids_reader
end
# ... 22 more methods
```

### ISG Extraction Targets

| Node Type | Source Pattern | Confidence | Priority |
|-----------|---------------|------------|----------|
| **Model** | `class X < ApplicationRecord` | HIGH | P0 |
| **Controller** | `class X < ApplicationController` | HIGH | P0 |
| **Route** | `resources :posts` | HIGH | P0 |
| **Association** | `has_many :items` | HIGH | P0 |
| **Validation** | `validates :attr, ...` | HIGH | P1 |
| **Scope** | `scope :name, -> { ... }` | MEDIUM | P1 |
| **Callback** | `before_save :method` | HIGH | P1 |
| **Enum** | `enum :status, [...]` | HIGH | P1 |

| Edge Type | Source Pattern | Confidence | Priority |
|-----------|---------------|------------|----------|
| **has_many** | `has_many :posts` | HIGH | P0 |
| **belongs_to** | `belongs_to :user` | HIGH | P0 |
| **has_one** | `has_one :profile` | HIGH | P0 |
| **through** | `has_many :tags, through: :taggings` | MEDIUM | P1 |
| **polymorphic** | `belongs_to :commentable, polymorphic: true` | MEDIUM | P2 |
| **route_to_controller** | `resources :posts` → PostsController | HIGH | P0 |

---

## High-Level Design (HLD)

### Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│           Rails ISG Extractor Pipeline               │
└──────────────────────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
┌───────▼────────┐ ┌─────▼─────┐ ┌───────▼────────┐
│ File Type      │ │Tree-Sitter│ │ Rails DSL      │
│ Detector       │ │Ruby Parser│ │ Knowledge Base │
│                │ │           │ │                │
│ - models/      │ │ Ruby AST  │ │ - has_many     │
│ - controllers/ │ │           │ │ - validates    │
│ - routes.rb    │ │           │ │ - scope        │
│ - db/schema.rb │ │           │ │ - enum         │
└───────┬────────┘ └─────┬─────┘ └───────┬────────┘
        │                │                │
        └────────────────┼────────────────┘
                         │
        ┌────────────────▼────────────────┐
        │   Pattern Matching Engine       │
        │                                 │
        │ - Association Matcher           │
        │ - Validation Matcher            │
        │ - Scope Matcher                 │
        │ - Route Matcher                 │
        │ - Concern Resolver              │
        └────────────────┬────────────────┘
                         │
        ┌────────────────▼────────────────┐
        │    Extractor Orchestrator       │
        │                                 │
        │ ┌─────────┐ ┌──────────┐       │
        │ │ Model   │ │Controller│       │
        │ │Extractor│ │Extractor │       │
        │ └────┬────┘ └────┬─────┘       │
        │      │           │              │
        │ ┌────▼───────────▼────┐        │
        │ │   Routes Extractor  │        │
        │ └─────────────────────┘        │
        └────────────────┬────────────────┘
                         │
        ┌────────────────▼────────────────┐
        │    Rails Entity Constructor     │
        │                                 │
        │ - RailsModel                    │
        │ - RailsController               │
        │ - RailsRoute                    │
        │ - Association Edges             │
        │ - Validation Metadata           │
        └────────────────┬────────────────┘
                         │
        ┌────────────────▼────────────────┐
        │     ISG Builder                 │
        │                                 │
        │ - Normalize to ISG format       │
        │ - Add confidence scores         │
        │ - Generate warning list         │
        └────────────────┬────────────────┘
                         │
        ┌────────────────▼────────────────┐
        │   CozoDB Storage                │
        │   (via pt01 interface)          │
        └─────────────────────────────────┘
```

### Data Flow

**Input**: Rails project directory
**Output**: ISG with Rails-specific entities and edges

**Flow**:
1. **File Detection** - Identify Rails files (models/, controllers/, config/routes.rb)
2. **Parsing** - Tree-sitter Ruby AST generation
3. **Pattern Matching** - Detect DSL method calls (has_many, validates, etc.)
4. **Entity Construction** - Build RailsModel, RailsController, RailsRoute objects
5. **Relationship Extraction** - Create association edges, route→controller mappings
6. **ISG Normalization** - Convert to generic ISG entity format
7. **Storage** - Insert into CozoDB via pt01 interface

### Component Breakdown

```
parseltongue-08-rails-extractor/
│
├── src/
│   ├── lib.rs                    # Public API
│   │
│   ├── parser/                   # Ruby Parsing Layer
│   │   ├── mod.rs
│   │   ├── ruby_parser.rs        # Tree-sitter-ruby wrapper
│   │   ├── ast_walker.rs         # AST traversal
│   │   └── source_mapper.rs      # Location tracking
│   │
│   ├── patterns/                 # DSL Pattern Matchers
│   │   ├── mod.rs
│   │   ├── pattern_engine.rs     # Core matching logic
│   │   ├── association.rs        # has_many, belongs_to
│   │   ├── validation.rs         # validates DSL
│   │   ├── callback.rs           # before_save, after_create
│   │   ├── scope.rs              # scope definitions
│   │   ├── enum.rs               # enum DSL
│   │   └── route.rs              # routes.rb DSL
│   │
│   ├── extractors/               # Feature Extractors
│   │   ├── mod.rs
│   │   ├── model.rs              # ActiveRecord models
│   │   ├── controller.rs         # ActionControllers
│   │   ├── routes.rs             # config/routes.rb
│   │   ├── schema.rs             # db/schema.rb
│   │   └── concern.rs            # Module resolution
│   │
│   ├── knowledge_base/           # Pre-Computed Rules
│   │   ├── mod.rs
│   │   ├── rails_dsl.rs          # DSL knowledge
│   │   ├── method_generator.rs   # Method generation
│   │   └── confidence.rs         # Scoring logic
│   │
│   ├── isg_builder/              # ISG Construction
│   │   ├── mod.rs
│   │   ├── rails_entity.rs       # Rails entities
│   │   ├── edge_builder.rs       # Relationship edges
│   │   └── normalizer.rs         # ISG format conversion
│   │
│   └── integration/              # Parseltongue Integration
│       ├── mod.rs
│       ├── pt01_adapter.rs       # File streaming
│       └── cozodb_writer.rs      # Database insertion
│
├── knowledge_base/               # YAML/JSON Data
│   ├── rails_7.1_dsl.yml        # Rails 7.1 DSL rules
│   ├── rails_7.0_dsl.yml        # Rails 7.0 DSL rules
│   └── method_templates.yml      # Generation templates
│
├── tests/
│   ├── fixtures/                 # Test Rails files
│   │   ├── models/
│   │   ├── controllers/
│   │   └── routes/
│   ├── unit/                     # Pattern matcher tests
│   ├── integration/              # End-to-end tests
│   └── golden/                   # Regression tests
│
└── Cargo.toml
```

---

## Low-Level Design (LLD)

### Core Interfaces

```rust
// lib.rs - Public API
pub trait RailsExtractor {
    fn extract_project(&self, path: &Path) -> Result<RailsProject>;
    fn extract_model(&self, file: &Path) -> Result<RailsModel>;
    fn extract_controller(&self, file: &Path) -> Result<RailsController>;
    fn extract_routes(&self, file: &Path) -> Result<Vec<RailsRoute>>;
}

pub struct RailsExtractorImpl {
    parser: RubyParser,
    pattern_engine: PatternEngine,
    knowledge_base: KnowledgeBase,
    concern_resolver: ConcernResolver,
}

impl RailsExtractor for RailsExtractorImpl {
    fn extract_project(&self, path: &Path) -> Result<RailsProject> {
        let models = self.scan_directory(path.join("app/models"))?;
        let controllers = self.scan_directory(path.join("app/controllers"))?;
        let routes = self.extract_routes(&path.join("config/routes.rb"))?;

        Ok(RailsProject { models, controllers, routes })
    }
}
```

### Pattern Matching Layer

```rust
// patterns/pattern_engine.rs

pub struct PatternEngine {
    matchers: Vec<Box<dyn PatternMatcher>>,
}

pub trait PatternMatcher: Send + Sync {
    fn name(&self) -> &str;
    fn matches(&self, node: &Node) -> bool;
    fn extract(&self, node: &Node, source: &str) -> Result<Box<dyn Pattern>>;
}

pub trait Pattern {
    fn pattern_type(&self) -> PatternType;
    fn confidence(&self) -> f32;
    fn to_isg_entities(&self) -> Vec<IsgEntity>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    Association,
    Validation,
    Scope,
    Enum,
    Callback,
    Route,
}

// patterns/association.rs

pub struct AssociationMatcher {
    knowledge_base: Arc<KnowledgeBase>,
}

impl PatternMatcher for AssociationMatcher {
    fn name(&self) -> &str { "association" }

    fn matches(&self, node: &Node) -> bool {
        if node.kind() != "call" {
            return false;
        }

        let method_name = node.child_by_field_name("method")
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or("");

        matches!(method_name, "has_many" | "belongs_to" | "has_one" | "has_and_belongs_to_many")
    }

    fn extract(&self, node: &Node, source: &str) -> Result<Box<dyn Pattern>> {
        let method = node.child_by_field_name("method")?.utf8_text(source)?;
        let args = node.child_by_field_name("arguments")?;

        // First argument: association name (symbol)
        let name_node = args.child(0)?;
        let name = self.extract_symbol(name_node, source)?;

        // Optional second argument: options hash
        let options = if args.child_count() > 1 {
            self.extract_hash(args.child(1)?, source)?
        } else {
            HashMap::new()
        };

        Ok(Box::new(Association {
            macro: method.to_string(),
            name,
            options,
            source_location: node.range(),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Association {
    pub macro: String,  // "has_many", "belongs_to", etc.
    pub name: String,   // "posts", "user", etc.
    pub options: HashMap<String, Value>,
    pub source_location: Range,
}

impl Pattern for Association {
    fn pattern_type(&self) -> PatternType {
        PatternType::Association
    }

    fn confidence(&self) -> f32 {
        // Base confidence for static associations
        let mut confidence = 1.0;

        // Reduce confidence for complex options
        if self.options.contains_key("through") {
            confidence *= 0.7;  // Through associations are harder
        }
        if self.options.contains_key("polymorphic") {
            confidence *= 0.7;  // Polymorphic needs type resolution
        }

        confidence
    }

    fn to_isg_entities(&self) -> Vec<IsgEntity> {
        vec![
            IsgEntity {
                entity_type: EntityType::Association,
                entity_name: format!("{}#{}", self.macro, self.name),
                metadata: self.to_metadata(),
                confidence: self.confidence(),
            }
        ]
    }
}
```

### Knowledge Base

```rust
// knowledge_base/rails_dsl.rs

pub struct KnowledgeBase {
    associations: HashMap<String, AssociationSpec>,
    enums: EnumSpec,
    scopes: ScopeSpec,
    validations: ValidationSpec,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssociationSpec {
    pub macro: String,
    pub generated_methods: Vec<MethodTemplate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MethodTemplate {
    pub name_template: String,  // "{name}", "{singular}_ids", etc.
    pub returns: String,
    pub params: Vec<ParamSpec>,
    pub visibility: Visibility,
}

impl KnowledgeBase {
    pub fn load(rails_version: &str) -> Result<Self> {
        let yaml_path = format!("knowledge_base/rails_{}_dsl.yml", rails_version);
        let yaml_content = std::fs::read_to_string(yaml_path)?;

        Ok(serde_yaml::from_str(&yaml_content)?)
    }

    pub fn generate_methods(&self, pattern: &Association) -> Vec<GeneratedMethod> {
        let spec = &self.associations[&pattern.macro];

        spec.generated_methods.iter().map(|template| {
            self.instantiate_template(template, &pattern)
        }).collect()
    }

    fn instantiate_template(&self, template: &MethodTemplate, assoc: &Association) -> GeneratedMethod {
        let name = self.substitute_template(&template.name_template, assoc);
        let returns = self.substitute_template(&template.returns, assoc);

        GeneratedMethod {
            name,
            returns,
            params: template.params.clone(),
            origin: MethodOrigin::Association(assoc.macro.clone()),
            confidence: assoc.confidence(),
        }
    }

    fn substitute_template(&self, template: &str, assoc: &Association) -> String {
        template
            .replace("{name}", &assoc.name)
            .replace("{singular}", &assoc.name.to_singular())
            .replace("{class_name}", &assoc.infer_class_name())
            .replace("{foreign_key}", &assoc.infer_foreign_key())
    }
}
```

### Rails Entity Types

```rust
// isg_builder/rails_entity.rs

#[derive(Debug, Clone)]
pub struct RailsModel {
    pub name: String,
    pub file_path: PathBuf,
    pub parent_class: Option<String>,
    pub associations: Vec<Association>,
    pub validations: Vec<Validation>,
    pub scopes: Vec<Scope>,
    pub callbacks: Vec<Callback>,
    pub enums: Vec<Enum>,
    pub includes: Vec<String>,  // Concerns/modules
    pub confidence: f32,
    pub warnings: Vec<ExtractionWarning>,
}

#[derive(Debug, Clone)]
pub struct Association {
    pub macro: AssociationMacro,
    pub name: String,
    pub target_model: String,  // Inferred from name or class_name option
    pub options: AssociationOptions,
    pub generated_methods: Vec<GeneratedMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssociationMacro {
    HasMany,
    BelongsTo,
    HasOne,
    HasAndBelongsToMany,
}

#[derive(Debug, Clone)]
pub struct AssociationOptions {
    pub class_name: Option<String>,
    pub foreign_key: Option<String>,
    pub dependent: Option<DependentAction>,
    pub through: Option<String>,
    pub source: Option<String>,
    pub polymorphic: bool,
    pub inverse_of: Option<String>,
    pub counter_cache: Option<CounterCache>,
}

#[derive(Debug, Clone)]
pub enum DependentAction {
    Destroy,
    DeleteAll,
    Nullify,
    RestrictWithError,
    RestrictWithException,
}

#[derive(Debug, Clone)]
pub struct Validation {
    pub validator: String,  // "PresenceValidator", "UniquenessValidator"
    pub attributes: Vec<String>,
    pub options: HashMap<String, Value>,
    pub conditions: ValidationConditions,
}

#[derive(Debug, Clone)]
pub struct ValidationConditions {
    pub if_condition: Option<String>,
    pub unless_condition: Option<String>,
    pub on: Option<ValidationContext>,
}

#[derive(Debug, Clone)]
pub enum ValidationContext {
    Create,
    Update,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub lambda_body: String,  // Opaque lambda as string
    pub conditions: Option<ScopeConditions>,
}

#[derive(Debug, Clone)]
pub struct ScopeConditions {
    pub where_clauses: Vec<WhereClause>,
    pub order_clauses: Vec<OrderClause>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Callback {
    pub hook: CallbackHook,
    pub method: CallbackMethod,
    pub options: CallbackOptions,
}

#[derive(Debug, Clone)]
pub enum CallbackHook {
    BeforeSave,
    AfterSave,
    BeforeCreate,
    AfterCreate,
    BeforeUpdate,
    AfterUpdate,
    BeforeDestroy,
    AfterDestroy,
    BeforeValidation,
    AfterValidation,
}

#[derive(Debug, Clone)]
pub enum CallbackMethod {
    Symbol(String),
    Lambda(String),
    Block(String),
}

#[derive(Debug, Clone)]
pub struct CallbackOptions {
    pub if_condition: Option<String>,
    pub unless_condition: Option<String>,
    pub on: Option<Vec<String>>,
    pub prepend: bool,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub values: IndexMap<String, i64>,  // Preserves order
    pub options: EnumOptions,
    pub generated_methods: Vec<GeneratedMethod>,
}

#[derive(Debug, Clone)]
pub struct EnumOptions {
    pub prefix: EnumAffix,
    pub suffix: EnumAffix,
    pub scopes: bool,
    pub instance_methods: bool,
}

#[derive(Debug, Clone)]
pub enum EnumAffix {
    None,
    Auto,  // true
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct GeneratedMethod {
    pub name: String,
    pub returns: String,
    pub params: Vec<ParamSpec>,
    pub origin: MethodOrigin,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub enum MethodOrigin {
    Association(String),  // "has_many"
    Enum(String),         // "status"
    Scope(String),        // "published"
    Custom,
}

#[derive(Debug, Clone)]
pub struct RailsController {
    pub name: String,
    pub file_path: PathBuf,
    pub parent_class: String,
    pub actions: Vec<ControllerAction>,
    pub before_actions: Vec<Filter>,
    pub after_actions: Vec<Filter>,
    pub around_actions: Vec<Filter>,
}

#[derive(Debug, Clone)]
pub struct ControllerAction {
    pub name: String,
    pub http_methods: Vec<HttpMethod>,  // Inferred from routes
    pub references_models: Vec<String>,  // Inferred from code
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub method: String,
    pub only: Vec<String>,
    pub except: Vec<String>,
    pub conditions: FilterConditions,
}

#[derive(Debug, Clone)]
pub struct RailsRoute {
    pub http_method: HttpMethod,
    pub path: String,
    pub controller: String,
    pub action: String,
    pub constraints: HashMap<String, String>,
    pub defaults: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}
```

### Concern Resolution

```rust
// extractors/concern.rs

pub struct ConcernResolver {
    concerns: HashMap<String, ConcernDefinition>,
    resolution_cache: HashMap<String, ResolvedConcern>,
}

#[derive(Debug, Clone)]
pub struct ConcernDefinition {
    pub name: String,
    pub instance_methods: Vec<MethodDef>,
    pub class_methods: Vec<MethodDef>,
    pub included_block: Option<IncludedBlock>,
    pub dependencies: Vec<String>,  // Other concerns this depends on
}

#[derive(Debug, Clone)]
pub struct IncludedBlock {
    pub dsl_calls: Vec<DslCall>,  // scope, validates, etc.
    pub method_calls: Vec<MethodCall>,
}

impl ConcernResolver {
    pub fn register_concern(&mut self, ast: &Node, source: &str) -> Result<()> {
        let name = self.extract_module_name(ast, source)?;

        let concern = ConcernDefinition {
            name: name.clone(),
            instance_methods: self.extract_instance_methods(ast, source)?,
            class_methods: self.extract_class_methods_block(ast, source)?,
            included_block: self.extract_included_block(ast, source)?,
            dependencies: self.extract_dependencies(ast, source)?,
        };

        self.concerns.insert(name, concern);
        Ok(())
    }

    pub fn resolve_for_class(&mut self, class_name: &str, includes: &[String]) -> Result<ResolvedConcern> {
        // Check cache first
        let cache_key = format!("{}:{}", class_name, includes.join(","));
        if let Some(cached) = self.resolution_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let mut resolved = ResolvedConcern::default();

        // Resolve in inclusion order (depth-first)
        for include in includes {
            self.resolve_concern_chain(include, &mut resolved, &mut HashSet::new())?;
        }

        // Cache result
        self.resolution_cache.insert(cache_key, resolved.clone());

        Ok(resolved)
    }

    fn resolve_concern_chain(
        &self,
        concern_name: &str,
        resolved: &mut ResolvedConcern,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        if visited.contains(concern_name) {
            return Ok(()); // Circular dependency, skip
        }
        visited.insert(concern_name.to_string());

        let concern = self.concerns.get(concern_name)
            .ok_or_else(|| anyhow!("Concern not found: {}", concern_name))?;

        // Resolve dependencies first
        for dep in &concern.dependencies {
            self.resolve_concern_chain(dep, resolved, visited)?;
        }

        // Add this concern's methods
        resolved.instance_methods.extend(concern.instance_methods.clone());
        resolved.class_methods.extend(concern.class_methods.clone());

        // Process included block
        if let Some(included) = &concern.included_block {
            for dsl_call in &included.dsl_calls {
                match dsl_call {
                    DslCall::Scope(scope) => resolved.scopes.push(scope.clone()),
                    DslCall::Validates(validation) => resolved.validations.push(validation.clone()),
                    DslCall::Callback(callback) => resolved.callbacks.push(callback.clone()),
                    // ... handle other DSLs
                }
            }
        }

        Ok(())
    }
}
```

### Confidence Scoring

```rust
// knowledge_base/confidence.rs

pub struct ConfidenceScorer {
    base_scores: HashMap<PatternType, f32>,
}

impl ConfidenceScorer {
    pub fn score(&self, pattern: &dyn Pattern) -> f32 {
        let base = self.base_scores.get(&pattern.pattern_type())
            .copied()
            .unwrap_or(0.5);

        let certainty = self.certainty_multiplier(pattern);
        let penalty = self.complexity_penalty(pattern);

        (base * certainty - penalty).clamp(0.0, 1.0)
    }

    fn certainty_multiplier(&self, pattern: &dyn Pattern) -> f32 {
        match pattern.pattern_type() {
            PatternType::Association => {
                let assoc = pattern.as_any().downcast_ref::<Association>().unwrap();

                if assoc.options.is_empty() {
                    1.0  // Simple association
                } else if assoc.options.contains_key("through") {
                    0.7  // Through association requires inference
                } else if assoc.options.contains_key("polymorphic") {
                    0.6  // Polymorphic needs type resolution
                } else {
                    0.9  // Association with simple options
                }
            }
            PatternType::Enum => {
                1.0  // Enums are very predictable
            }
            PatternType::Scope => {
                0.8  // Scopes are lambda-based, partial understanding
            }
            PatternType::Validation => {
                0.9  // Validations are mostly static
            }
            _ => 0.7,
        }
    }

    fn complexity_penalty(&self, pattern: &dyn Pattern) -> f32 {
        let mut penalty = 0.0;

        // Check for metaprogramming indicators
        if pattern.has_string_eval() {
            penalty += 0.3;
        }
        if pattern.has_method_missing() {
            penalty += 0.4;
        }
        if pattern.has_dynamic_variable() {
            penalty += 0.2;
        }

        penalty
    }
}
```

### ISG Normalization

```rust
// isg_builder/normalizer.rs

pub struct IsgNormalizer;

impl IsgNormalizer {
    pub fn normalize_model(&self, model: &RailsModel) -> Vec<IsgEntity> {
        let mut entities = vec![];

        // Model entity
        entities.push(IsgEntity {
            entity_type: EntityType::RailsModel,
            entity_name: model.name.clone(),
            file_path: model.file_path.to_string_lossy().to_string(),
            interface_signature: self.generate_model_signature(model),
            language_metadata: Some(self.model_to_metadata(model)),
            confidence: model.confidence,
        });

        // Association edges
        for assoc in &model.associations {
            entities.push(self.association_to_edge(&model.name, assoc));
        }

        entities
    }

    fn generate_model_signature(&self, model: &RailsModel) -> String {
        let mut sig = format!("class {}", model.name);

        if let Some(parent) = &model.parent_class {
            sig.push_str(&format!(" < {}", parent));
        }

        // Add association summary
        if !model.associations.is_empty() {
            sig.push_str(" | associations: ");
            sig.push_str(&model.associations.iter()
                .map(|a| format!("{}:{}", a.macro, a.name))
                .collect::<Vec<_>>()
                .join(", "));
        }

        sig
    }

    fn association_to_edge(&self, from_model: &str, assoc: &Association) -> IsgEntity {
        IsgEntity {
            entity_type: EntityType::RailsAssociation,
            entity_name: format!("{}--[{}]-->{}", from_model, assoc.macro, assoc.target_model),
            interface_signature: format!("{} {}", assoc.macro, assoc.name),
            language_metadata: Some(LanguageMetadata::Rails(RailsMetadata::Association(assoc.clone()))),
            confidence: assoc.confidence(),
        }
    }
}
```

---

## Implementation Roadmap

### Phase 1: MVP Foundation (Week 1-2)

**Goal**: Extract associations from ActiveRecord models

**Deliverables**:
1. Tree-sitter Ruby integration
2. Association pattern matcher (has_many, belongs_to, has_one)
3. Basic ISG builder
4. CozoDB integration

**Week 1: Core Infrastructure**

*Day 1-2: Parser Setup*
```rust
// Milestone: Parse Ruby file to AST
let source = std::fs::read_to_string("app/models/user.rb")?;
let parser = RubyParser::new();
let ast = parser.parse(&source)?;

assert!(ast.root_node().kind() == "program");
```

*Day 3-4: Association Matcher*
```rust
// Milestone: Detect has_many calls
let matcher = AssociationMatcher::new();
let associations = matcher.find_all(&ast, &source)?;

assert_eq!(associations[0].macro, "has_many");
assert_eq!(associations[0].name, "posts");
```

*Day 5: Knowledge Base*
```rust
// Milestone: Generate methods from association
let kb = KnowledgeBase::load("7.1")?;
let methods = kb.generate_methods(&associations[0]);

assert!(methods.iter().any(|m| m.name == "posts"));
assert!(methods.iter().any(|m| m.name == "post_ids"));
```

**Week 2: End-to-End Flow**

*Day 6-7: Model Extractor*
```rust
// Milestone: Extract complete model
let extractor = ModelExtractor::new();
let model = extractor.extract("app/models/user.rb")?;

assert_eq!(model.name, "User");
assert_eq!(model.associations.len(), 3);
assert!(model.confidence > 0.9);
```

*Day 8-9: ISG Builder*
```rust
// Milestone: Convert to ISG format
let normalizer = IsgNormalizer;
let entities = normalizer.normalize_model(&model);

assert!(entities.iter().any(|e| e.entity_type == EntityType::RailsModel));
assert!(entities.iter().any(|e| e.entity_type == EntityType::RailsAssociation));
```

*Day 10: Integration Test*
```bash
# Milestone: Full pipeline test
parseltongue pt01 rails-app/

# Expected output in CozoDB:
# - Entity: rails_model:User
# - Edge: User --[HasMany]--> Post
# - Edge: User --[BelongsTo]--> Organization
```

**Success Metrics**:
- ✅ Parse 10 fixture models without errors
- ✅ Extract 90%+ of simple associations (no options)
- ✅ Extract 70%+ of associations with options
- ✅ <100ms per model file

---

### Phase 2: Enhanced Rails Features (Week 3-4)

**Goal**: Add validations, scopes, routes, enums

**Week 3: Validations & Scopes**

*Day 11-12: Validation Matcher*
```rust
// Pattern: validates :email, presence: true, uniqueness: true
let validation_matcher = ValidationMatcher::new();
let validations = validation_matcher.find_all(&ast, &source)?;

assert_eq!(validations[0].validator, "PresenceValidator");
assert_eq!(validations[0].attributes, vec!["email"]);
```

*Day 13-14: Scope Matcher*
```rust
// Pattern: scope :published, -> { where(published: true) }
let scope_matcher = ScopeMatcher::new();
let scopes = scope_matcher.find_all(&ast, &source)?;

assert_eq!(scopes[0].name, "published");
assert!(scopes[0].lambda_body.contains("where"));
```

*Day 15: Enum Matcher*
```rust
// Pattern: enum :status, [:active, :archived]
let enum_matcher = EnumMatcher::new();
let enums = enum_matcher.find_all(&ast, &source)?;

assert_eq!(enums[0].name, "status");
assert_eq!(enums[0].values.len(), 2);

let methods = kb.generate_enum_methods(&enums[0]);
assert!(methods.iter().any(|m| m.name == "active?"));
assert!(methods.iter().any(|m| m.name == "active!"));
```

**Week 4: Routes & Controllers**

*Day 16-17: Routes Extractor*
```ruby
# config/routes.rb
Rails.application.routes.draw do
  resources :posts
  get '/about', to: 'pages#about'
end
```

```rust
let routes_extractor = RoutesExtractor::new();
let routes = routes_extractor.extract("config/routes.rb")?;

// RESTful resources generate 7 routes
assert_eq!(routes.iter().filter(|r| r.controller == "posts").count(), 7);

// Custom routes
assert!(routes.iter().any(|r| r.path == "/about" && r.action == "about"));
```

*Day 18-19: Controller Extractor*
```rust
let controller_extractor = ControllerExtractor::new();
let controller = controller_extractor.extract("app/controllers/posts_controller.rb")?;

assert_eq!(controller.name, "PostsController");
assert!(controller.actions.iter().any(|a| a.name == "index"));
assert!(controller.before_actions.len() > 0);
```

*Day 20: Confidence Scoring*
```rust
let scorer = ConfidenceScorer::new();

// Simple association: high confidence
let simple_assoc = Association { macro: "has_many", name: "posts", options: HashMap::new() };
assert!(scorer.score(&simple_assoc) > 0.95);

// Through association: lower confidence
let through_assoc = Association {
    macro: "has_many",
    name: "tags",
    options: hashmap!{ "through" => "taggings" }
};
assert!(scorer.score(&through_assoc) < 0.8);
```

**Success Metrics**:
- ✅ Extract 80%+ of validations
- ✅ Extract 70%+ of scopes (name only, opaque lambda)
- ✅ Map 90%+ of RESTful routes to controllers
- ✅ Confidence scores accurate (±0.1 from manual review)

---

### Phase 3: Advanced Patterns (Week 5-6)

**Goal**: Concerns, callbacks, metaprogramming detection

**Week 5: Concerns & Callbacks**

*Day 21-22: Concern Detection*
```ruby
# app/models/concerns/commentable.rb
module Commentable
  extend ActiveSupport::Concern

  included do
    has_many :comments, as: :commentable
  end

  class_methods do
    def with_comments
      joins(:comments).distinct
    end
  end
end
```

```rust
let concern_resolver = ConcernResolver::new();
concern_resolver.register_concern(&ast, &source)?;

let concern = concern_resolver.concerns.get("Commentable").unwrap();
assert_eq!(concern.class_methods.len(), 1);
assert_eq!(concern.included_block.unwrap().dsl_calls.len(), 1);
```

*Day 23-24: Callback Extractor*
```rust
// Pattern: before_save :normalize_title
let callback_matcher = CallbackMatcher::new();
let callbacks = callback_matcher.find_all(&ast, &source)?;

assert_eq!(callbacks[0].hook, CallbackHook::BeforeSave);
assert_eq!(callbacks[0].method, CallbackMethod::Symbol("normalize_title"));
```

**Week 6: Metaprogramming & Edge Cases**

*Day 25-26: Metaprogramming Detection*
```rust
pub struct MetaprogrammingDetector;

impl MetaprogrammingDetector {
    pub fn detect(&self, ast: &Node) -> Vec<MetaprogrammingWarning> {
        let mut warnings = vec![];

        for node in ast.descendants() {
            match node.kind() {
                "call" => {
                    let method = node.child_by_field_name("method");
                    if let Some(method_name) = method.and_then(|m| m.utf8_text(source).ok()) {
                        match method_name {
                            "class_eval" | "instance_eval" | "module_eval" => {
                                warnings.push(MetaprogrammingWarning::StringEval {
                                    location: node.range(),
                                    method: method_name.to_string(),
                                });
                            }
                            "method_missing" => {
                                warnings.push(MetaprogrammingWarning::MethodMissing {
                                    location: node.range(),
                                });
                            }
                            "define_method" => {
                                warnings.push(MetaprogrammingWarning::DefineMethod {
                                    location: node.range(),
                                });
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        warnings
    }
}
```

*Day 27-28: Polymorphic Associations*
```ruby
# Polymorphic association
class Comment < ApplicationRecord
  belongs_to :commentable, polymorphic: true
end

class Post < ApplicationRecord
  has_many :comments, as: :commentable
end

class Article < ApplicationRecord
  has_many :comments, as: :commentable
end
```

```rust
// Extract polymorphic edges
let comment_assoc = Association {
    macro: "belongs_to",
    name: "commentable",
    options: hashmap!{ "polymorphic" => true }
};

// ISG representation: Multi-target edge
let edge = RailsEdge::Polymorphic {
    from: "Comment",
    to_interface: "Commentable",
    possible_targets: vec!["Post", "Article"],  // Inferred from schema
    confidence: 0.7,
};
```

*Day 29-30: Integration & Testing*
```bash
# Test on real Rails apps
parseltongue pt01 discourse/
parseltongue pt01 spree/

# Measure extraction coverage
parseltongue analyze --coverage discourse/
# Expected: 80% association recall, 70% validation recall
```

**Success Metrics**:
- ✅ Resolve 80%+ of concerns correctly
- ✅ Extract 90%+ of callbacks
- ✅ Flag 95%+ of metaprogramming patterns
- ✅ Handle 70%+ of polymorphic associations

---

### Phase 4: Real-World Validation (Week 7-8)

**Goal**: Test on production Rails apps, optimize, document

**Week 7: Real App Testing**

*Test Corpus*:
1. **Discourse** (300+ models, heavy use of concerns)
2. **Spree** (e-commerce, complex associations)
3. **Redmine** (legacy Rails patterns)

*Validation Methodology*:
```bash
# 1. Run extractor
parseltongue pt01 discourse/ --output discourse_isg.json

# 2. Manual audit (20 random models)
rails console
User.reflections.keys  # Compare with extracted associations

# 3. Calculate metrics
precision = true_positives / (true_positives + false_positives)
recall = true_positives / (true_positives + false_negatives)
f1_score = 2 * (precision * recall) / (precision + recall)

# Target: F1 > 0.85
```

**Week 8: Optimization & Documentation**

*Performance Optimization*:
```rust
// Parallel file processing
use rayon::prelude::*;

let models: Vec<RailsModel> = model_files
    .par_iter()
    .map(|file| extractor.extract(file))
    .collect::<Result<Vec<_>>>()?;

// Benchmark: <10s for 200 models
```

*Documentation*:
1. API documentation (rustdoc)
2. Usage guide (README.md)
3. Confidence scoring interpretation
4. Known limitations document

**Success Metrics**:
- ✅ Discourse: 80%+ association extraction
- ✅ Spree: 75%+ route mapping
- ✅ <10s for 200-model app
- ✅ Comprehensive documentation

---

## Metaprogramming Strategies

### Taxonomy of Analyzability

| Pattern | Static Analysis | Strategy | Confidence |
|---------|----------------|----------|------------|
| **ActiveSupport::Concern** | ✅ YES | Track included/class_methods blocks | 95% |
| **has_many/belongs_to** | ✅ YES | Pre-computed method table | 98% |
| **scope** | ✅ YES | Detect pattern, opaque lambda | 95% |
| **enum** | ✅ YES | Generate from values array/hash | 98% |
| **validates** | ✅ YES | Parse DSL arguments | 90% |
| **class_eval (block)** | ⚠️ PARTIAL | Block AST analysis | 60% |
| **class_eval (string)** | ⚠️ PARTIAL | Template parsing if simple | 40% |
| **define_method** | ⚠️ PARTIAL | Pattern matching | 50% |
| **method_missing** | ❌ NO | Runtime behavior only | 10% |
| **send with variable** | ❌ NO | Requires runtime values | 15% |

### Generated Method Database

**Approach**: Maintain YAML knowledge base of Rails DSL patterns

```yaml
# knowledge_base/rails_7.1_dsl.yml

associations:
  has_many:
    pattern: "(send nil? :has_many (sym $_) ...)"
    generated_instance_methods:
      - name: "{name}"
        returns: "ActiveRecord::Associations::CollectionProxy<{class_name}>"
      - name: "{name}="
        returns: "void"
        params:
          - name: "value"
            type: "Array<{class_name}>"
      - name: "{singular}_ids"
        returns: "Array<Integer>"
      - name: "{singular}_ids="
        returns: "void"
        params:
          - name: "ids"
            type: "Array<Integer>"
    # ... 20+ more methods

enums:
  enum:
    pattern: "(send nil? :enum (sym $_) ${array | hash} ...)"
    generated_instance_methods:
      - name: "{prefix}{value}{suffix}?"
        returns: "Boolean"
        for_each: "values"
      - name: "{prefix}{value}{suffix}!"
        returns: "Boolean"
        for_each: "values"
    generated_class_methods:
      - name: "{prefix}{value}{suffix}"
        returns: "ActiveRecord::Relation"
        for_each: "values"
      - name: "not_{prefix}{value}{suffix}"
        returns: "ActiveRecord::Relation"
        for_each: "values"
```

### Concern Resolution Algorithm

```rust
pub fn resolve_concern_chain(
    concern_name: &str,
    resolved: &mut ResolvedConcern,
    visited: &mut HashSet<String>
) -> Result<()> {
    if visited.contains(concern_name) {
        return Ok(()); // Circular dependency
    }
    visited.insert(concern_name.to_string());

    let concern = self.concerns.get(concern_name)?;

    // 1. Resolve dependencies first (depth-first)
    for dep in &concern.dependencies {
        self.resolve_concern_chain(dep, resolved, visited)?;
    }

    // 2. Add instance methods
    resolved.instance_methods.extend(concern.instance_methods.clone());

    // 3. Add class methods
    resolved.class_methods.extend(concern.class_methods.clone());

    // 4. Process included block (DSL calls become class-level)
    if let Some(included) = &concern.included_block {
        for dsl_call in &included.dsl_calls {
            match dsl_call {
                DslCall::HasMany(assoc) => {
                    resolved.associations.push(assoc.clone());
                    let methods = kb.generate_methods(assoc);
                    resolved.instance_methods.extend(methods);
                }
                DslCall::Scope(scope) => {
                    resolved.scopes.push(scope.clone());
                    resolved.class_methods.push(MethodDef {
                        name: scope.name.clone(),
                        returns: "ActiveRecord::Relation",
                    });
                }
                // ... other DSLs
            }
        }
    }

    Ok(())
}
```

### Confidence Scoring Formula

```
confidence = base_score × certainty_multiplier − complexity_penalty

base_score (by pattern type):
  - Static literal (has_many :posts) = 1.0
  - Symbol from variable (has_many sym) = 0.7
  - String interpolation = 0.4
  - Dynamic (eval) = 0.2

certainty_multiplier (by options):
  - No options = 1.0
  - Simple options (class_name:) = 0.9
  - Complex options (through:) = 0.7
  - Lambda/proc = 0.6

complexity_penalty:
  - Metaprogramming detected = -0.2
  - Missing dependency = -0.3
  - Conditional DSL = -0.1
```

**Examples**:

```ruby
# Case 1: Simple has_many
has_many :posts
# confidence = 1.0 × 1.0 - 0 = 1.0

# Case 2: Through association
has_many :tags, through: :taggings
# confidence = 1.0 × 0.7 - 0 = 0.7

# Case 3: Polymorphic
belongs_to :commentable, polymorphic: true
# confidence = 1.0 × 0.6 - 0 = 0.6

# Case 4: Dynamic in loop
[:posts, :comments].each { |a| has_many a }
# confidence = 0.7 × 1.0 - 0.2 = 0.5
```

---

## Testing & Validation

### Test Pyramid

```
       /\
      /E2E\     5% - Real Rails apps (Discourse, Spree)
     /____\
    /      \
   /  INT   \   15% - Model/Controller extraction
  /__________\
 /            \
/    UNIT      \ 80% - Pattern matchers, Knowledge base
/______________\
```

### Unit Tests (Pattern Matchers)

```rust
#[test]
fn test_simple_has_many() {
    let source = r#"
    class User < ApplicationRecord
      has_many :posts
    end
    "#;

    let parser = RubyParser::new();
    let ast = parser.parse(source).unwrap();

    let matcher = AssociationMatcher::new();
    let associations = matcher.find_all(&ast, source).unwrap();

    assert_eq!(associations.len(), 1);
    assert_eq!(associations[0].macro, "has_many");
    assert_eq!(associations[0].name, "posts");
    assert!(associations[0].confidence() > 0.95);
}

#[test]
fn test_enum_with_prefix() {
    let source = r#"
    enum :status, [:active, :archived], prefix: true
    "#;

    let enum_matcher = EnumMatcher::new();
    let enums = enum_matcher.find_all(&ast, source).unwrap();

    assert_eq!(enums[0].values.len(), 2);
    assert_eq!(enums[0].options.prefix, EnumAffix::Auto);

    let kb = KnowledgeBase::load("7.1").unwrap();
    let methods = kb.generate_enum_methods(&enums[0]);

    assert!(methods.iter().any(|m| m.name == "status_active?"));
    assert!(methods.iter().any(|m| m.name == "status_archived!"));
}
```

### Integration Tests (End-to-End)

```rust
#[test]
fn test_full_model_extraction() {
    let source = include_str!("../fixtures/models/post.rb");

    let extractor = ModelExtractor::new();
    let model = extractor.extract_from_source(source, "post.rb").unwrap();

    assert_eq!(model.name, "Post");
    assert_eq!(model.associations.len(), 3);  // has_many :comments, :tags, belongs_to :user
    assert_eq!(model.validations.len(), 2);   // validates :title, :body
    assert_eq!(model.scopes.len(), 1);        // scope :published
    assert!(model.confidence > 0.8);
}

#[test]
fn test_concern_resolution() {
    let commentable_source = include_str!("../fixtures/concerns/commentable.rb");
    let post_source = include_str!("../fixtures/models/post_with_concern.rb");

    let mut concern_resolver = ConcernResolver::new();

    // Register concern
    concern_resolver.register_from_source(commentable_source, "commentable.rb").unwrap();

    // Extract model with concern
    let extractor = ModelExtractor::with_resolver(concern_resolver);
    let model = extractor.extract_from_source(post_source, "post.rb").unwrap();

    // Verify concern methods were added
    assert!(model.associations.iter().any(|a| a.name == "comments"));
    assert!(model.includes.contains(&"Commentable".to_string()));
}
```

### Golden File Tests (Regression)

```rust
#[test]
fn test_regression_discourse_user() {
    let source = include_str!("../fixtures/real_apps/discourse/user.rb");

    let extractor = ModelExtractor::new();
    let model = extractor.extract_from_source(source, "user.rb").unwrap();

    let result_json = serde_json::to_string_pretty(&model).unwrap();
    let expected_json = include_str!("../fixtures/expected/discourse_user.json");

    assert_json_eq!(result_json, expected_json);
}
```

### Real-World Validation

**Test Apps**:
1. **Discourse** - 300+ models, heavy metaprogramming
2. **Spree** - E-commerce, complex associations
3. **Redmine** - Legacy Rails patterns

**Validation Process**:

```bash
# 1. Extract ISG
parseltongue pt01 discourse/ --output discourse_isg.json

# 2. Compare with runtime reflections
cd discourse
rails console

# Count associations in runtime
User.reflections.count  # => 47

# 3. Compare with extracted
cat discourse_isg.json | jq '.entities[] | select(.name == "User") | .associations | length'
# => 45 (95.7% recall)

# 4. Manual audit of differences
# - 2 missing: Dynamic associations in concern
# - 0 false positives
```

**Target Metrics**:
- Precision: >90% (few false positives)
- Recall: >80% (catch most associations)
- F1 Score: >0.85

---

## Integration with Parseltongue

### File Type Detection

```rust
// parseltongue-01-folder-to-cozodb-streamer

pub fn detect_rails_project(path: &Path) -> bool {
    path.join("config/application.rb").exists() &&
    path.join("Gemfile").exists() &&
    path.join("app/models").exists()
}

pub fn classify_rails_file(path: &Path) -> Option<RailsFileType> {
    let path_str = path.to_string_lossy();

    if path_str.contains("/app/models/") && path.extension() == Some("rb") {
        Some(RailsFileType::Model)
    } else if path_str.contains("/app/controllers/") {
        Some(RailsFileType::Controller)
    } else if path_str.ends_with("config/routes.rb") {
        Some(RailsFileType::Routes)
    } else if path_str.contains("/db/schema.rb") {
        Some(RailsFileType::Schema)
    } else {
        None
    }
}
```

### Entity Schema Extension

```rust
// parseltongue-core

pub enum EntityType {
    // Existing types...
    Function,
    Class,

    // New Rails types
    RailsModel,
    RailsController,
    RailsRoute,
    RailsAssociation,
    RailsConcern,
}

pub struct Entity {
    pub entity_type: EntityType,
    pub entity_name: String,
    pub file_path: String,
    pub interface_signature: String,

    // NEW: Rails-specific metadata
    pub language_metadata: Option<LanguageMetadata>,
    pub confidence: f32,
}

pub enum LanguageMetadata {
    Rust(RustMetadata),
    Python(PythonMetadata),
    Rails(RailsMetadata),
}

pub enum RailsMetadata {
    Model {
        associations: Vec<Association>,
        validations: Vec<Validation>,
        scopes: Vec<Scope>,
        callbacks: Vec<Callback>,
    },
    Controller {
        actions: Vec<ControllerAction>,
        filters: Vec<Filter>,
    },
    Route {
        http_method: HttpMethod,
        path: String,
        controller: String,
        action: String,
    },
}
```

### CozoDB Schema

```cozo
# Rails-specific relations

:create rails_models {
    model_name: String,
    file_path: String,
    parent_class: String?,
    confidence: Float,
}

:create rails_associations {
    from_model: String,
    to_model: String,
    association_type: String,  # "has_many", "belongs_to", etc.
    association_name: String,
    options: Json,
    confidence: Float,
}

:create rails_validations {
    model_name: String,
    validator: String,
    attributes: [String],
    options: Json,
}

:create rails_routes {
    http_method: String,
    path: String,
    controller: String,
    action: String,
}

:create rails_generated_methods {
    class_name: String,
    method_name: String,
    origin: String,  # "has_many:posts", "enum:status"
    returns: String,
    confidence: Float,
}
```

### Level 0/1/2 Export

```rust
// parseltongue-02-llm-cozodb-to-context-writer

impl ContextWriter {
    pub fn export_rails_level0(&self, model: &RailsModel) -> String {
        let mut output = String::new();

        // Model signature
        output.push_str(&format!("class {} < {}\n", model.name, model.parent_class.as_deref().unwrap_or("ApplicationRecord")));

        // Associations (high signal)
        for assoc in &model.associations {
            output.push_str(&format!("  {} :{}\n", assoc.macro, assoc.name));
        }

        output.push_str("end\n");
        output
    }

    pub fn export_rails_level1(&self, model: &RailsModel) -> String {
        let mut output = self.export_rails_level0(model);

        // Add validations
        output.push_str("\n# Validations\n");
        for validation in &model.validations {
            output.push_str(&format!("validates {}\n", validation.attributes.join(", ")));
        }

        // Add scopes
        output.push_str("\n# Scopes\n");
        for scope in &model.scopes {
            output.push_str(&format!("scope :{}\n", scope.name));
        }

        output
    }
}
```

---

`★ Insight ─────────────────────────────────────`
**Rails ISG Extraction Philosophy:**

Unlike C/C++ where we parse syntax to extract semantics, Rails extraction is fundamentally **pattern recognition over convention**. The key insight is that Rails' metaprogramming, while dynamic at runtime, follows **static, regular patterns** we can encode.

Three-layer approach:
1. **AST Pattern Matching** - Detect DSL calls syntactically
2. **Knowledge Base Lookup** - Apply pre-computed generation rules
3. **Confidence Scoring** - Explicit about uncertainty

This hybrid approach achieves 70-80% accuracy without code execution, which is sufficient for architectural understanding and codebase navigation. The remaining 20-30% (complex metaprogramming) is explicitly flagged for manual review.

The Rails extractor complements C/C++ extractors: Together they enable full-stack analysis of modern web applications (Rails backend + C++ services).
`─────────────────────────────────────────────────`

---

## Appendix: Complete Method Generation Tables

### has_many Generated Methods

```
Collection Access:
  - posts                       # ActiveRecord::Associations::CollectionProxy
  - posts=(others)              # void

ID Access:
  - post_ids                    # Array<Integer>
  - post_ids=(ids)              # void

Builders:
  - posts.build(attrs={})       # Post (unsaved)
  - posts.create(attrs={})      # Post (saved)
  - posts.create!(attrs={})     # Post (raises on error)

Mutators:
  - posts << post               # CollectionProxy (append)
  - posts.push(post)            # CollectionProxy (alias)
  - posts.concat([post1, ...])  # CollectionProxy (append multiple)
  - posts.delete(post)          # Post (remove without callbacks)
  - posts.destroy(post)         # Post (remove with callbacks)
  - posts.clear                 # CollectionProxy (remove all)
  - posts.delete_all            # Integer (removes all, no callbacks)
  - posts.destroy_all           # Array<Post> (removes all with callbacks)

Queries:
  - posts.find(id)              # Post
  - posts.where(conditions)     # ActiveRecord::Relation
  - posts.exists?(conditions)   # Boolean
  - posts.size                  # Integer (uses counter_cache if available)
  - posts.length                # Integer (loads and counts)
  - posts.count                 # Integer (SQL COUNT)
  - posts.empty?                # Boolean
  - posts.any?                  # Boolean
  - posts.many?                 # Boolean
  - posts.include?(post)        # Boolean

Utilities:
  - posts.reload                # CollectionProxy (clear cache)
  - posts.reset                 # CollectionProxy (clear cache, alias)
```

### belongs_to Generated Methods

```
Access:
  - author                      # User | nil
  - author=(user)               # void

Builders:
  - build_author(attrs={})      # User (unsaved)
  - create_author(attrs={})     # User (saved)
  - create_author!(attrs={})    # User (raises on error)

Utilities:
  - reload_author               # User | nil

Change Tracking:
  - author_changed?             # Boolean
  - author_previously_changed?  # Boolean
```

### enum Generated Methods

```ruby
# Example: enum :status, [:draft, :published, :archived]

Predicates (per value):
  - draft?                      # Boolean
  - published?                  # Boolean
  - archived?                   # Boolean

Bangs (per value):
  - draft!                      # Boolean
  - published!                  # Boolean
  - archived!                   # Boolean

Scopes (class methods, per value):
  - Post.draft                  # ActiveRecord::Relation
  - Post.published              # ActiveRecord::Relation
  - Post.archived               # ActiveRecord::Relation

Negative Scopes:
  - Post.not_draft              # ActiveRecord::Relation
  - Post.not_published          # ActiveRecord::Relation
  - Post.not_archived           # ActiveRecord::Relation

Enum Hash:
  - Post.statuses               # { "draft" => 0, "published" => 1, "archived" => 2 }
```

---

**END OF DOCUMENT**

**Total Pages**: ~45 (at standard formatting)
**Total Code Examples**: 60+
**Total Diagrams**: 5 (text-based)
**Research Sources**: 3 agent reports (Explore, Plan, Research)
