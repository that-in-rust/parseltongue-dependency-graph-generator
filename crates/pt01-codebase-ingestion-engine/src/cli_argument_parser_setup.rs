//! CLI argument parser for Parseltongue v3.0.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "parseltongue", version, about = "Parseltongue v3.0 Code Analysis")]
pub struct CliArguments {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Ingest a codebase into the index
    Ingest {
        /// Path to the codebase root
        path: String,
        /// Database path (default: .parseltongue/index.db)
        #[arg(long, default_value = ".parseltongue/index.db")]
        db: String,
        /// Verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Start the HTTP query server
    Serve {
        /// Database path
        #[arg(long, default_value = ".parseltongue/index.db")]
        db: String,
        /// Port number
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    /// Run rustc enrichment on a Rust project
    Enrich {
        /// Path to the Rust project
        path: String,
        /// Database path
        #[arg(long, default_value = ".parseltongue/index.db")]
        db: String,
    },
}
