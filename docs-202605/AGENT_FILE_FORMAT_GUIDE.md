# Claude Code Agent File Format Guide

## Overview

This guide documents the proper format for creating agent files in Claude Code. Agent files are specialized prompts that trigger automatically based on user requests or can be invoked explicitly.

---

## File Structure

### Basic Template

Every agent file must be a markdown file (`.md`) with YAML frontmatter containing all configuration and the system prompt.

```yaml
---
name: agent-name
description: |
  Clear, detailed description of what this agent does and when it should be used.

  Triggers:
  - Trigger condition 1
  - Trigger condition 2
  - "specific keyword"

  Core Innovation: What makes this agent unique.

Examples:
<example>
Context: When this agent should be used.
user: "Example user request"
assistant: "How the agent responds"
<commentary>Why this example demonstrates the agent's purpose.</commentary>
</example>

system_prompt: |
  # Agent Name v1.0

  **Identity**: Clear statement of what this agent is and does.

  [All agent instructions go here, properly indented with 2 spaces]

  ## Section 1

  Content...

  ## Section 2

  Content...

model: inherit
---
```

---

## Required Fields

### 1. `name` (Required)

**Format**: Lowercase with dashes, no spaces
**Purpose**: Unique identifier for the agent

```yaml
name: parseltongue-ultrathink
```

**Rules**:
- Must be lowercase
- Use dashes (`-`) instead of spaces
- Should be descriptive and unique
- Used for direct invocation: `@agent-name` in chat

**Examples**:
- ✅ `parseltongue-ultrathink`
- ✅ `rust-linter`
- ✅ `security-audit-helper`
- ❌ `Parseltongue Ultrathink` (has spaces and capitals)
- ❌ `my agent` (has space)

---

### 2. `description` (Required)

**Format**: Multi-line YAML string using `|` (pipe character)
**Purpose**: Most critical field for automatic delegation - tells Claude when to use this agent

```yaml
description: |
  Advanced codebase analysis agent using Interface Signature Graphs (ISG).

  Triggers:
  - Architecture analysis requests
  - Dependency mapping
  - "ultrathink" keyword

  Core Innovation: Combines ISG with CPU-based pre-filtering.
```

**Rules**:
- Must use `|` for multi-line content
- Should be detailed and specific (not vague)
- Include clear trigger conditions
- Explain what makes this agent unique
- Can include examples inline

**Good Description**:
```yaml
description: |
  Expert Rust linter that analyzes code snippets to find common errors,
  non-idiomatic patterns, and suggests improvements based on official
  Rust guidelines.

  Triggers:
  - Rust code review requests
  - "lint this rust code"
  - Idiomatic pattern analysis

  Checks for:
  - Unnecessary .clone() calls
  - Complex lifetimes
  - Improper Option/Result handling
```

**Bad Description** (too vague):
```yaml
description: |
  A rust agent.
```

---

### 3. `system_prompt` (Required)

**Format**: Multi-line YAML string using `|` (pipe character)
**Purpose**: Contains all agent instructions - the actual "persona" and behavior

```yaml
system_prompt: |
  # Agent Name v1.0

  **Identity**: You are an expert in X that does Y.

  ## Core Philosophy

  Your mission is to...

  ## Workflow

  1. Step 1
  2. Step 2

  ## Rules

  You MUST:
  - Rule 1
  - Rule 2

  You MUST NOT:
  - Restriction 1
  - Restriction 2
```

**Rules**:
- Must use `|` for multi-line content
- All content must be indented with **2 spaces** (YAML requirement)
- Can contain markdown formatting
- Can include code blocks (properly indented)
- Can include mermaid diagrams
- Should be comprehensive and detailed

**Indentation Example**:
```yaml
system_prompt: |
  # Header (2 spaces from left margin)

  Content here (2 spaces from left margin)

  ```bash
  # Code blocks work (2 spaces + triple backticks)
  echo "hello"
  ```

  More content (2 spaces from left margin)
```

---

### 4. `model` (Optional)

**Format**: String value
**Purpose**: Specifies which model to use for this agent

```yaml
model: inherit
```

**Options**:
- `inherit` - Use whatever model the user has selected (recommended default)
- `sonnet` - Force Sonnet model
- `opus` - Force Opus model
- `haiku` - Force Haiku model (for quick, simple tasks)

**When to Use**:
- Default: `inherit` (let user choose)
- Quick tasks: `haiku` (cost optimization)
- Complex reasoning: `opus` (if truly needed)

---

## YAML Frontmatter Rules

### The Pipe Character (`|`)

The `|` character in YAML preserves line breaks and formatting:

```yaml
description: |
  Line 1
  Line 2
  Line 3
```

This is essential for multi-line fields like `description` and `system_prompt`.

### Indentation Requirements

YAML requires consistent indentation:

```yaml
---
name: my-agent
description: |
  First line (NO indentation here, starts at column 3)
  Second line (same - column 3)
system_prompt: |
  # Header (2 spaces indent)

  Content (2 spaces indent)

  ```bash
  code here (2 spaces indent + backticks)
  ```
model: inherit
---
```

**Critical Rules**:
1. Top-level fields (`name`, `description`, etc.) - no indentation
2. Content after `|` - starts at the same column as the `|`
3. All system_prompt content - must be indented with 2 spaces
4. Code blocks within system_prompt - also 2 spaces + triple backticks

---

## Examples Section (Optional but Recommended)

Examples help Claude understand when to invoke your agent:

```yaml
description: |
  Agent description here.

Examples:
<example>
Context: Brief context about when this applies.
user: "What the user might say"
assistant: "How the agent should respond"
<commentary>Why this example is relevant.</commentary>
</example>

<example>
Context: Another scenario.
user: "Another user request"
assistant: "Another response"
<commentary>Explanation of this use case.</commentary>
</example>
```

**Best Practices**:
- Include 2-4 examples
- Cover different use cases
- Show the trigger conditions in action
- Add commentary explaining why the example matters

---

## Complete Example: Minimal Agent

```yaml
---
name: simple-greeter
description: |
  A friendly agent that responds to greetings with jokes.

  Triggers:
  - User says "hello"
  - User says "hi"
  - Greeting phrases

Examples:
<example>
Context: User greets the agent.
user: "Hello!"
assistant: "Hi there! Why did the programmer quit? Because they didn't get arrays!"
<commentary>Simple greeting trigger demonstrating the agent's personality.</commentary>
</example>

system_prompt: |
  # Simple Greeter v1.0

  **Identity**: You are a friendly greeter who tells programming jokes.

  ## Behavior

  When the user greets you:
  1. Respond warmly
  2. Tell a short programming joke
  3. Ask how you can help

  ## Rules

  - Always be friendly and welcoming
  - Keep jokes short (1-2 lines)
  - Stay on-topic after the greeting

model: inherit
---
```

---

## Complete Example: Complex Agent (Parseltongue Ultrathink)

```yaml
---
name: parseltongue-ultrathink
description: |
  Advanced codebase analysis agent using Interface Signature Graphs (ISG)
  with context window optimization and CPU-first filtering.

  Triggers:
  - Architecture analysis requests
  - Dependency mapping
  - "ultrathink" keyword
  - Circular dependency detection
  - API surface analysis
  - Security audits
  - Token-efficient codebase understanding
  - Learning from reference codebases (.ref pattern)

  Core Innovation: Combines parseltongue ISG with CPU-based pre-filtering
  (scc, Semgrep, ast-grep) to achieve 85-97% token reduction while
  maintaining superior analysis quality.

Examples:
<example>
Context: User wants comprehensive architecture analysis with token efficiency.
user: "Analyze the architecture of this 100K LOC codebase efficiently"
assistant: "I'll use the parseltongue-ultrathink agent with CPU-first filtering to analyze the architecture while maintaining 95%+ thinking space ratio (TSR)."
<commentary>Large codebase analysis benefits from multi-tier CPU filtering before ISG analysis to minimize token consumption.</commentary>
</example>

system_prompt: |
  # Parseltongue Ultrathink Agent v2.0

  **Identity**: You are a **context-efficient ISG analyst** that combines
  progressive disclosure, CPU-first filtering, and graph-based reasoning.

  ## Core Philosophy: Context Window as Thinking Space

  Every token of data is a token not available for reasoning.

  ### Multi-Tier Analysis Architecture

  You operate in 5 progressive tiers:

  ```
  Codebase (100%)
    ↓ Tier 1: Metrics (scc) → Filter to 30%
    ↓ Tier 2: Patterns (Semgrep) → Filter to 10%
    ↓ Tier 3: ISG → Filter to 5%
    ↓ Tier 4: LLM Reasoning → Analyze 5%
    ↓ Tier 5: Validation → Verify
  ```

  [... extensive system prompt content ...]

model: inherit
---
```

---

## File Location

Agent files must be placed in the `.claude/` directory:

```
project-root/
├── .claude/
│   ├── agent-name.md          # Your agent file
│   ├── another-agent.md
│   └── AGENT_FILE_FORMAT_GUIDE.md  # This guide
└── [other project files]
```

---

## Mermaid Diagrams in Agents

You can include mermaid diagrams in the `system_prompt`:

```yaml
system_prompt: |
  ## Workflow Diagram

  ```mermaid
  flowchart TD
      A[Start] --> B{Decision?}
      B -->|Yes| C[Action 1]
      B -->|No| D[Action 2]
      C --> E[End]
      D --> E

      style C fill:#99C899
      style D fill:#C89999
  ```
```

**Color Guidelines**:
- Use minimal colors (3 max recommended)
- Neutral: `#9DB4C8` (blue-gray)
- Warning/Error: `#C89999` (soft red)
- Success: `#99C899` (soft green)
- All colors are WCAG AAA compliant (7:1+ contrast)

---

## Common Patterns

### The .ref Pattern (Reference Codebases)

For agents that learn from external code:

```yaml
system_prompt: |
  ## Reference Codebase Pattern

  When learning from external implementations:

  1. Create `.claude/.ref/` directory
  2. **CRITICAL**: Add `.claude/.ref/` to `.gitignore` FIRST
  3. Clone reference: `git clone <repo> .claude/.ref/<name>/`
  4. Analyze with parseltongue ISG
  5. Extract patterns
  6. Adapt to your codebase

  ### Safety Rules

  - ALWAYS add .ref to .gitignore before cloning
  - Check `git status` before committing
  - Use descriptive folder names
  - Clean up periodically
```

### Workflow-Based Agents

For agents with multiple task-specific workflows:

```yaml
system_prompt: |
  ## Workflow Decision Tree

  Match your task to the optimal workflow:

  - **Workflow 1**: Onboarding (15 min, 8K tokens)
  - **Workflow 2**: Bug Triage (20 min, 12K tokens)
  - **Workflow 3**: Security Audit (60 min, 28K tokens)

  ### Workflow 1: Onboarding

  [Detailed workflow steps...]

  ### Workflow 2: Bug Triage

  [Detailed workflow steps...]
```

---

## Testing Your Agent

### 1. Validate YAML Syntax

```bash
# Check if file parses correctly
python3 -c "import yaml; yaml.safe_load(open('.claude/your-agent.md').read())"
```

### 2. Test Invocation

In Claude Code, test your agent:

```
@your-agent-name please analyze this code
```

### 3. Test Auto-Delegation

Try a request that should trigger your agent automatically:

```
[Request that matches your trigger conditions]
```

---

## Best Practices

### 1. Clear Triggers

Make triggers specific and testable:

✅ **Good**:
```yaml
Triggers:
- User asks to "analyze architecture"
- User mentions "dependency mapping"
- User uses keyword "ultrathink"
```

❌ **Bad**:
```yaml
Triggers:
- Code analysis
- Help with stuff
```

### 2. Comprehensive System Prompt

Include:
- Identity statement
- Core philosophy/mission
- Detailed workflows
- Mandatory rules
- Forbidden actions
- Examples/templates

### 3. Version Your Agent

```yaml
system_prompt: |
  # Agent Name v2.1

  **Version History**:
  - v2.1: Added .ref pattern support
  - v2.0: Multi-tier CPU analysis
  - v1.0: Initial ISG-based implementation
```

### 4. Document Token Efficiency

If your agent focuses on efficiency:

```yaml
system_prompt: |
  ## Token Budget

  - Level 0: 2-5K tokens (97.5% TSR)
  - Level 1: 20-30K tokens (85% TSR)
  - Level 2: 50-60K tokens (70% TSR)

  **TSR (Thinking Space Ratio)** = (Available Context - Data Tokens) / Available Context
```

### 5. Include Safety Guardrails

```yaml
system_prompt: |
  ## Safety Guardrails

  **Check 1: Validation**
  ```
  IF condition NOT met THEN
    STOP
    REPORT: "Error message"
    SUGGEST: "Alternative approach"
  END IF
  ```
```

---

## Common Mistakes to Avoid

### ❌ Missing `|` After Field Names

**Wrong**:
```yaml
description:
  This is my description
  with multiple lines
```

**Right**:
```yaml
description: |
  This is my description
  with multiple lines
```

### ❌ Incorrect Indentation

**Wrong**:
```yaml
system_prompt: |
# Header (no indent - WRONG!)
Content here (no indent - WRONG!)
```

**Right**:
```yaml
system_prompt: |
  # Header (2 spaces - correct)
  Content here (2 spaces - correct)
```

### ❌ Using Uppercase or Spaces in Name

**Wrong**:
```yaml
name: My Agent Name
```

**Right**:
```yaml
name: my-agent-name
```

### ❌ Vague Description

**Wrong**:
```yaml
description: |
  A helpful agent.
```

**Right**:
```yaml
description: |
  Expert Rust code reviewer that identifies idiomatic patterns,
  performance issues, and safety concerns in Rust codebases.

  Triggers:
  - "review my rust code"
  - Rust code quality analysis
  - Idiomatic pattern suggestions
```

---

## Maintenance and Updates

### When to Update Your Agent

1. **User feedback**: "This doesn't work as expected"
2. **New features**: Add new workflows or capabilities
3. **Bug fixes**: Correct errors in instructions
4. **Performance improvements**: Better token efficiency
5. **Clarity improvements**: Make instructions clearer

### Version Bumping Guide

- **Patch (v1.0.0 → v1.0.1)**: Bug fixes, typo corrections
- **Minor (v1.0.0 → v1.1.0)**: New workflows, added features
- **Major (v1.0.0 → v2.0.0)**: Breaking changes, complete restructure

### Commit Message Format

```bash
git commit -m "feat(agent-name): Add new workflow for X

- Added Workflow 8: Reference Learning
- Updated decision tree with WF8
- Added mermaid diagram for .ref pattern
- Documented safety rules for gitignore

Addresses user need to learn from external codebases.
"
```

---

## Resources

### YAML Documentation
- [YAML Specification](https://yaml.org/spec/)
- [YAML Multi-line Strings](https://yaml-multiline.info/)

### Mermaid Documentation
- [Mermaid Flowcharts](https://mermaid.js.org/syntax/flowchart.html)
- [Mermaid Styling](https://mermaid.js.org/config/theming.html)

### Accessibility (Colors)
- [WCAG Contrast Checker](https://webaim.org/resources/contrastchecker/)
- Aim for 7:1 ratio (AAA standard)

---

## Appendix: Quick Reference

### Minimal Agent Template

```yaml
---
name: agent-name
description: |
  What this agent does.

  Triggers:
  - Trigger 1
  - Trigger 2

system_prompt: |
  # Agent Name v1.0

  You are an agent that does X.

model: inherit
---
```

### Field Summary

| Field | Required | Type | Purpose |
|-------|----------|------|---------|
| `name` | ✅ Yes | String (lowercase-with-dashes) | Agent identifier |
| `description` | ✅ Yes | Multi-line string (`\|`) | Trigger conditions, what agent does |
| `system_prompt` | ✅ Yes | Multi-line string (`\|`) | All agent instructions |
| `model` | ❌ No | String (`inherit`/`sonnet`/`opus`/`haiku`) | Model selection |

---

**Last Updated**: 2025-11-03
**Version**: 1.0
**Author**: Parseltongue Project

This guide is a living document. Update it as you discover new patterns or best practices for agent development.
