# Map-Driven Explorer - Implementation Plan

## MVP Scope (Week 1-2)

### Goal
Prove the concept with a minimal working prototype that:
1. Creates `.parseltongue-explorer/` folder structure
2. Parses a simple map YAML
3. Clones 2-3 repos automatically
4. Indexes them into separate databases
5. Generates a basic `llm-context.md`

### Deliverables

#### 1. Folder Structure Creator
```rust
// crates/pt-explorer/src/init.rs

pub fn create_workspace(base_dir: &Path) -> Result<()> {
    let explorer_dir = base_dir.join(".parseltongue-explorer");

    fs::create_dir_all(explorer_dir.join("maps"))?;
    fs::create_dir_all(explorer_dir.join("clones"))?;
    fs::create_dir_all(explorer_dir.join("databases"))?;
    fs::create_dir_all(explorer_dir.join("outputs"))?;
    fs::create_dir_all(explorer_dir.join("web-research"))?;
    fs::create_dir_all(explorer_dir.join("cache"))?;

    // Create default map
    create_default_map(&explorer_dir.join("maps/default.yaml"))?;

    Ok(())
}
```

#### 2. Map YAML Parser
```rust
// crates/pt-explorer/src/map.rs

#[derive(Debug, Deserialize)]
pub struct Map {
    pub workspace: Workspace,
    pub repositories: Vec<Repository>,
    pub strategies: Option<Vec<Strategy>>,
    pub outputs: Option<OutputConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub branch: String,
    pub database: String,  // Auto-generated if not specified
    pub languages: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Strategy {
    pub name: String,
    pub databases: Vec<String>,
    pub queries: Vec<Query>,
}

#[derive(Debug, Deserialize)]
pub struct Query {
    pub r#type: String,  // "entity-search" | "dependency-graph"
    pub r#where: Option<String>,
    pub export_level: String,  // "level00" | "level01" | "level02"
}
```

#### 3. Git Clone Automation
```rust
// crates/pt-explorer/src/git.rs

pub fn clone_repositories(map: &Map, base_dir: &Path) -> Result<Vec<PathBuf>> {
    let clones_dir = base_dir.join(".parseltongue-explorer/clones");
    let mut cloned_paths = Vec::new();

    for repo in &map.repositories {
        let repo_path = clones_dir.join(&repo.name);

        if repo_path.exists() {
            println!("✓ Repository {} already exists, skipping", repo.name);
        } else {
            println!("Cloning {} → {}", repo.url, repo_path.display());

            Command::new("git")
                .args(&["clone", "--branch", &repo.branch, &repo.url, repo_path.to_str().unwrap()])
                .output()?;

            println!("✓ Cloned {}", repo.name);
        }

        cloned_paths.push(repo_path);
    }

    Ok(cloned_paths)
}
```

#### 4. Batch Indexing
```rust
// crates/pt-explorer/src/index.rs

pub fn index_repositories(map: &Map, base_dir: &Path) -> Result<Vec<IndexResult>> {
    let clones_dir = base_dir.join(".parseltongue-explorer/clones");
    let db_dir = base_dir.join(".parseltongue-explorer/databases");

    let mut results = Vec::new();

    for repo in &map.repositories {
        let repo_path = clones_dir.join(&repo.name);
        let db_path = db_dir.join(format!("{}.db", repo.name));
        let db_uri = format!("rocksdb:{}", db_path.display());

        println!("Indexing {} → {}", repo.name, db_uri);

        // Call PT01 (folder-to-cozodb-streamer)
        let output = Command::new("parseltongue")
            .args(&[
                "pt01-folder-to-cozodb-streamer",
                repo_path.to_str().unwrap(),
                "--db", &db_uri,
            ])
            .output()?;

        if output.status.success() {
            // Parse output to get entity/dependency counts
            let stdout = String::from_utf8_lossy(&output.stdout);
            let entity_count = extract_entity_count(&stdout);
            let dep_count = extract_dependency_count(&stdout);

            println!("✓ Indexed {} ({} entities, {} dependencies)",
                     repo.name, entity_count, dep_count);

            results.push(IndexResult {
                repo_name: repo.name.clone(),
                db_path,
                entity_count,
                dependency_count: dep_count,
            });
        } else {
            eprintln!("✗ Failed to index {}: {}",
                     repo.name, String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(results)
}

fn extract_entity_count(output: &str) -> usize {
    // Parse PT01 output for entity count
    // Example: "Stored 347 entities"
    output.lines()
        .find(|line| line.contains("entities"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}
```

#### 5. LLM Context Generator
```rust
// crates/pt-explorer/src/context.rs

pub fn generate_llm_context(
    strategy: &Strategy,
    query_results: Vec<QueryResult>,
    output_dir: &Path,
) -> Result<PathBuf> {
    let context_path = output_dir.join("llm-context.md");
    let mut context = String::new();

    // Header
    context.push_str(&format!("# Context Document: {}\n", strategy.name));
    context.push_str(&format!("Generated: {}\n\n", chrono::Utc::now()));
    context.push_str("---\n\n");

    // Reading order recommendation
    context.push_str("## Reading Order Recommendation\n\n");
    context.push_str("Read these sections in order for maximum comprehension:\n\n");

    let ordered_sections = order_by_relevance(&query_results);
    for (i, section) in ordered_sections.iter().enumerate() {
        context.push_str(&format!("{}. **{}**\n", i + 1, section.title));
    }
    context.push_str("\n---\n\n");

    // Sections
    for section in ordered_sections {
        context.push_str(&format!("## {}\n\n", section.title));

        for entity in section.entities {
            context.push_str(&format!("### {} ({}:{})\n",
                                     entity.name, entity.file_path, entity.line_range));
            context.push_str("```\n");
            context.push_str(&entity.signature);
            context.push_str("\n```\n\n");

            if !entity.dependencies.is_empty() {
                context.push_str("**Dependencies**:\n");
                for dep in &entity.dependencies {
                    context.push_str(&format!("- Calls: {}\n", dep));
                }
                context.push_str("\n");
            }
        }
    }

    // Summary statistics
    context.push_str("---\n\n## Summary Statistics\n\n");
    let total_entities: usize = query_results.iter().map(|r| r.entities.len()).sum();
    context.push_str(&format!("- **Total entities**: {}\n", total_entities));
    context.push_str(&format!("- **Databases queried**: {}\n", strategy.databases.len()));

    // Write to file
    fs::write(&context_path, context)?;

    Ok(context_path)
}

fn order_by_relevance(results: &[QueryResult]) -> Vec<Section> {
    // Order strategy:
    // 1. Public APIs first (pub fn, pub struct)
    // 2. Core business logic (non-test, non-helper functions)
    // 3. Database/persistence layer
    // 4. Error handling
    // 5. Utilities/helpers

    let mut sections = vec![
        Section { title: "Public API Surface".to_string(), entities: Vec::new() },
        Section { title: "Core Business Logic".to_string(), entities: Vec::new() },
        Section { title: "Database Layer".to_string(), entities: Vec::new() },
        Section { title: "Error Handling".to_string(), entities: Vec::new() },
        Section { title: "Utilities".to_string(), entities: Vec::new() },
    ];

    for result in results {
        for entity in &result.entities {
            // Classify entity
            if entity.signature.contains("pub fn") || entity.signature.contains("pub struct") {
                sections[0].entities.push(entity.clone());
            } else if entity.file_path.contains("db") || entity.file_path.contains("database") {
                sections[2].entities.push(entity.clone());
            } else if entity.name.contains("error") || entity.signature.contains("Result<") {
                sections[3].entities.push(entity.clone());
            } else if entity.name.contains("helper") || entity.name.contains("util") {
                sections[4].entities.push(entity.clone());
            } else {
                sections[1].entities.push(entity.clone());
            }
        }
    }

    sections.into_iter().filter(|s| !s.entities.is_empty()).collect()
}
```

#### 6. CLI Commands
```rust
// crates/pt-explorer/src/main.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "parseltongue-explorer")]
#[command(about = "Map-Driven Code Intelligence System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize .parseltongue-explorer/ workspace
    Init {
        #[arg(long)]
        map_name: Option<String>,
    },

    /// Clone repositories from map
    Clone {
        #[arg(long)]
        map: String,
    },

    /// Index repositories into databases
    Index {
        #[arg(long)]
        map: String,

        #[arg(long)]
        repo: Option<String>,  // Index specific repo only
    },

    /// Run a strategy (query + context generation)
    Query {
        #[arg(long)]
        map: String,

        #[arg(long)]
        strategy: String,
    },

    /// Clone + Index + Query (all-in-one)
    Run {
        #[arg(long)]
        map: String,

        #[arg(long)]
        all_strategies: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { map_name } => {
            let cwd = std::env::current_dir()?;
            create_workspace(&cwd)?;
            println!("✓ Initialized .parseltongue-explorer/");

            if let Some(name) = map_name {
                create_map(&cwd.join(".parseltongue-explorer/maps"), &name)?;
                println!("✓ Created map: {}.yaml", name);
            }
        }

        Commands::Clone { map } => {
            let map_data = load_map(&map)?;
            let cwd = std::env::current_dir()?;
            clone_repositories(&map_data, &cwd)?;
        }

        Commands::Index { map, repo } => {
            let map_data = load_map(&map)?;
            let cwd = std::env::current_dir()?;

            if let Some(repo_name) = repo {
                // Index specific repo only
                index_single_repository(&map_data, &cwd, &repo_name)?;
            } else {
                // Index all repos in map
                index_repositories(&map_data, &cwd)?;
            }
        }

        Commands::Query { map, strategy } => {
            let map_data = load_map(&map)?;
            let strategy_data = map_data.strategies
                .and_then(|strategies| strategies.into_iter().find(|s| s.name == strategy))
                .ok_or_else(|| anyhow!("Strategy '{}' not found in map", strategy))?;

            execute_strategy(&map_data, &strategy_data)?;
        }

        Commands::Run { map, all_strategies } => {
            let map_data = load_map(&map)?;
            let cwd = std::env::current_dir()?;

            // Step 1: Clone
            println!("Step 1/3: Cloning repositories...");
            clone_repositories(&map_data, &cwd)?;

            // Step 2: Index
            println!("\nStep 2/3: Indexing repositories...");
            index_repositories(&map_data, &cwd)?;

            // Step 3: Query
            println!("\nStep 3/3: Running strategies...");
            if all_strategies {
                for strategy in map_data.strategies.unwrap_or_default() {
                    execute_strategy(&map_data, &strategy)?;
                }
            }
        }
    }

    Ok(())
}
```

## Example Map (Starter Template)

```yaml
# .parseltongue-explorer/maps/rust-examples.yaml

workspace:
  name: "rust-examples"
  description: "Index 3 reference Rust projects"
  base_dir: ".parseltongue-explorer"

repositories:
  - name: "ripgrep"
    url: "https://github.com/BurntSushi/ripgrep.git"
    branch: "master"
    database: "ripgrep.db"
    languages: ["rust"]

  - name: "fd"
    url: "https://github.com/sharkdp/fd.git"
    branch: "master"
    database: "fd.db"
    languages: ["rust"]

  - name: "bat"
    url: "https://github.com/sharkdp/bat.git"
    branch: "master"
    database: "bat.db"
    languages: ["rust"]

strategies:
  - name: "find-cli-patterns"
    description: "Find CLI parsing patterns across all repos"
    databases: ["ripgrep.db", "fd.db", "bat.db"]
    queries:
      - type: "entity-search"
        where: "entity_name CONTAINS 'cli' OR entity_name CONTAINS 'parse'"
        export_level: "level01"

  - name: "find-main-functions"
    description: "Find main entry points"
    databases: ["ripgrep.db", "fd.db", "bat.db"]
    queries:
      - type: "entity-search"
        where: "entity_name = 'main'"
        export_level: "level02"

outputs:
  base_folder: "outputs/{{date}}_{{strategy_name}}/"
  formats: ["json", "toon", "markdown"]
```

## Testing Plan

### Manual Test (Validate Concept)
```bash
# 1. Initialize
cargo run --bin parseltongue-explorer -- init --map-name rust-examples

# 2. Clone + Index + Query
cargo run --bin parseltongue-explorer -- run --map rust-examples.yaml --all-strategies

# Expected output:
# ✓ Cloned ripgrep → .parseltongue-explorer/clones/ripgrep/
# ✓ Cloned fd → .parseltongue-explorer/clones/fd/
# ✓ Cloned bat → .parseltongue-explorer/clones/bat/
# ✓ Indexed ripgrep.db (523 entities, 1247 dependencies)
# ✓ Indexed fd.db (347 entities, 892 dependencies)
# ✓ Indexed bat.db (412 entities, 1053 dependencies)
# ✓ Executed strategy: find-cli-patterns
# ✓ Generated: .parseltongue-explorer/outputs/2025-11-08_find-cli-patterns/llm-context.md

# 3. Verify output
cat .parseltongue-explorer/outputs/2025-11-08_find-cli-patterns/llm-context.md

# Should see:
# - Ordered sections (Public APIs, Core Logic, etc.)
# - Function signatures from all 3 repos
# - Dependencies listed
# - Summary statistics
```

### Success Criteria
- [ ] Folder structure created correctly
- [ ] 3 repos cloned without errors
- [ ] 3 databases created with > 300 entities each
- [ ] `llm-context.md` generated with ordered sections
- [ ] Total time < 2 minutes (clone + index + query)

## Timeline

### Week 1
- [x] Research document (DONE)
- [ ] Create `pt-explorer` crate
- [ ] Implement folder structure creator
- [ ] Implement map YAML parser
- [ ] Implement git clone automation

### Week 2
- [ ] Implement batch indexing
- [ ] Implement basic context generator (no ordering yet)
- [ ] Build CLI commands (init, clone, index, query)
- [ ] Test with 3 Rust repos
- [ ] Iterate based on results

## Open Questions for Week 1

1. **Should pt-explorer be a separate binary or part of main parseltongue CLI?**
   - Separate: More focused, easier to iterate
   - Integrated: Single entry point, less complexity
   - **Recommendation**: Start separate, merge later

2. **Should we use existing PT tools or rewrite?**
   - Use existing: Faster, proven, less work
   - Rewrite: More control, better integration
   - **Recommendation**: Use existing PT01/PT02 via CLI calls

3. **Should we support incremental indexing in MVP?**
   - Yes: Better UX, faster re-indexing
   - No: Simpler implementation, less risk
   - **Recommendation**: No for MVP, add in Week 3-4

4. **Should context ordering be ML-based or rule-based?**
   - ML: Better quality, more sophisticated
   - Rules: Faster, more predictable, easier to debug
   - **Recommendation**: Rules for MVP, ML later

## Files to Create

```
parseltongue/
├── crates/
│   └── pt-explorer/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs          # CLI entry point
│           ├── init.rs          # Workspace creation
│           ├── map.rs           # YAML parsing
│           ├── git.rs           # Clone automation
│           ├── index.rs         # Batch indexing
│           ├── context.rs       # LLM context generation
│           └── lib.rs           # Public API
│
└── .parseltongue-explorer/      # Created on init (gitignored)
    ├── maps/
    │   └── rust-examples.yaml
    ├── clones/
    ├── databases/
    ├── outputs/
    └── cache/
```

## Next Action

**Decision point**: Should we proceed with implementation?

If yes:
1. Create `crates/pt-explorer/` directory
2. Set up Cargo.toml with dependencies (clap, serde_yaml, anyhow)
3. Implement `init.rs` (simplest component)
4. Test folder creation

If no:
1. What aspects of the design need revision?
2. What additional research is needed?
