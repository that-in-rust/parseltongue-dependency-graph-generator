#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_hir;
extern crate rustc_span;

pub mod compiler_analysis_driver;
pub mod entity_enrichment_mapper;
pub mod mir_call_graph_extractor;
