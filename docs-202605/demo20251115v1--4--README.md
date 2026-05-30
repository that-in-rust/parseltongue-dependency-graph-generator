# parseltongue-06: cozoDB-make-future-code-current

**Tool 6 in the Parseltongue 6-tool pipeline - The Final Step**

## Purpose

State reset manager that makes Future_Code become Current_Code by:
1. Deleting the CodeGraph table (NO backups)
2. Recreating the schema
3. Re-indexing the codebase from actual files

## Ultra-Minimalist Design

- **NO BACKUP METADATA**: No .snapshot, .backup, or metadata files
- **NO CONFIGURATION**: Single deterministic reset operation
- **NO ROLLBACK**: Permanent state reset
- **NO COMPLEXITY**: Delete → Recreate → Re-index

## Usage

```bash
# Reset database state and re-index codebase
parseltongue-06 --database ./parseltongue.db --project-path ./my-project
```

## Integration in Pipeline

**Position**: After file writing (Tool 5) → Completes the cycle

**Flow**:
1. Tool 5 writes Future_Code to files
2. Tool 6 deletes CodeGraph table
3. Tool 1 re-indexes from actual files
4. New baseline established for next iteration

## Philosophy

> "The codebase is the single source of truth"

Tool 6 embodies this by resetting temporal state and re-establishing the baseline from actual source files. No metadata preservation, no snapshots - just clean state reset.

## License

MIT OR Apache-2.0
