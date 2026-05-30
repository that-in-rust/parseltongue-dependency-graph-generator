# Esssence

Current token lenght of models = 200k tokens
Current size of code = 5000k tokens
You canNOT reason properly with such context length

imagine if your ceo asked for insights of last 1 week of data which at granular level exists at 1 minute intervals = 1 x 60 x 24 x 7 = 10080 data points

- do you show him each 10080 data points for insights to help him take decisions
- you can show him it to be aggregated percentile distribution or mean or sum of
  - 10080 data points at minute basis
  - 168 data points at hourly basis
  - 7 data points at day basis

RCA

- If CEO think some problem is there on Tuesday - does he start with 10800 data points OR does he first pick the day on which the issue might have happened?
- Then goes to hour
- then goes to minute

Similar analogy

- Map of India has 25 states
- Each State 20 cities
- Each City 50 key neighbourhoods
- We see 1 video of a neighbourhood of something thing wrong
- Do we run all the video clips of 25 x 20 x 50 = 50000 videos ?
  - Can we approximate which state
  - Can we approximate which city
  - Can we approximate which neighbourhood?

# Objective

With minimal context spent we give highest amount of signal to an LLM to reason throught the codebase

## Definitions

- Define interfaces : struct i.e. class, type; functions or generics ; statements; libraries which are basically functions; modules which are basically functions; files behave as modules;
- Code is written in filepath-filename-line-numbers
- Code in action by compiler is actually a graph of interfaces connected to each other via multiple relationships

## Why not graph databases to store code?

- Since the fundamental operations of a compiler are multi-relationship manipulators, a code should exist in a graph database
- An incremental way to think of it is, for the sake of humans, keep the files in tree lije filesystems, but to run a compiler immediately ingest them into a a graphdatabase for faster higher quality compilation
  - Level 1 - write code in filesystem - CURRENT SYTEM
  - Level 2 - ingest code in graphdatabase - run a faster x higher quality compiler to check what the agent did or you did almost like a REPL
  - Level 3 - before final binary run a traditional proven existing compile

## Why lines are the addresses of functions in a file? And should they be so?

### Assignment of Primary Key

What are the biggest disadvantages of a <filepath>-<file-name>-<interface-name>-<interface-type>-<line-start>-<line-end> = ADDRESS01?

Other types of interfaces can be ADDRESS01-<interface-name>-... - problem is line-start AND line-end IS DEPENDENT ON THE CONTENT OF THE 3 things

- current interface whose address you are looking at
  - it can grow in size
  - it can shrink in size
- preceding interfaces
- succeding interfaces - MAYBE ?

e.g.
"key": "rust:struct:Cli:\_\_crates_pt06-cozodb-make-future-code-current_src_cli_rs:7-23",
"file": "./crates/pt06-cozodb-make-future-code-current/src/cli.rs"

Especially in context of storing it in a database?

## Solutions

### Solution 01

Define an address of a code block

- file-path
  - file-name
    - interface-array-index-number
      - a text value

parseltongue-src-main.rs-[0] = `use clap::{Arg, ArgMatches, Command};`
parseltongue-src-main.rs-[1] = `use console::style;`
parseltongue-src-main.rs-[2] = `use anyhow::{Result, Context};`
parseltongue-src-main.rs-[3] = `use std::path::PathBuf;`
parseltongue-src-main.rs-[4] = `use std::collections::HashMap;`
parseltongue-src-main.rs-[5]= `use pt01_folder_to_cozodb_streamer::streamer::FileStreamer;`
parseltongue-src-main.rs-[6]-`use parseltongue_core::entities`= `use parseltongue_core::entities::{
    CodeEntity, TemporalState, InterfaceSignature, EntityType, Visibility,
    LineRange, Language, LanguageSpecificSignature, RustSignature,
    TddClassification, EntityClass, TestabilityLevel, ComplexityLevel, RiskLevel,
    EntityMetadata,
};`

Simulations

- If a particular array cell expands in text, then it can likely accommodate the whole code inside it

## Technical Architecture

### Compilation Strategy

- **Current State**: File-based compilation
- **Target Architecture**: Graph-based compilation
- **Migration Path**:
  1. File-based code editing (current)
  2. Graph-based analysis and validation
  3. Traditional compilation for final build

### Key Challenges

1. **Line Management**

   - Handle dynamic code insertion/deletion
   - Maintain line number consistency
   - Support for code block resizing

2. **Repository Analysis**
   - Automated API/repository cloning
   - Pattern extraction from source code
   - Multi-language analysis capabilities

## Visualization Requirements

- **Control Flow Diagrams**
- **Data Flow Visualizations**
- **Code Clustering**
- **Dependency Graphs**

## Implementation Notes

- **Language Support**:
  - **Analysis**: Multi-language
  - **Code Generation**: Rust-only
- **Workflow**:
  ```mermaid
  graph LR
      A[CozoDB Compilation] --> B[Code Generation]
      B --> C[File System Update]
  ```
- **Special Features**:

  - Automated repository analysis
  - Pattern recognition
  - Integration with version control

- New projects

- Graph based compiler
- Visualiser
- Optionality tool for search, edit etc etc

# User Journey

```mermaid

{{ ... }}
```
