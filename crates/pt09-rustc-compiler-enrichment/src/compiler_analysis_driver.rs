use anyhow::Result;
use crate::entity_enrichment_mapper::EnrichmentResult;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

struct EnrichmentCallbacks {
    results: Arc<Mutex<Vec<EnrichmentResult>>>,
}

impl Callbacks for EnrichmentCallbacks {
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        let mut results = self.results.lock().unwrap();
        // Iterate all top-level items in the local crate
        for item_id in tcx.hir_crate_items(()).free_items() {
            let def_id = item_id.owner_id.def_id.to_def_id();
            let def_path = tcx.def_path_str(def_id);
            let span = tcx.def_span(def_id);
            let source_map = tcx.sess.source_map();
            let lo = source_map.lookup_char_pos(span.lo());
            let hi = source_map.lookup_char_pos(span.hi());

            let file_path = lo.file.name.prefer_local_unconditionally().to_string_lossy().to_string();
            let start_line = lo.line as u32;
            let end_line = hi.line as u32;

            results.push(EnrichmentResult {
                file_path,
                start_line,
                end_line,
                rustc_scope: def_path,
                rustc_sig: None, // TODO: extract signature from TyCtxt
                visibility: "unknown".to_string(), // TODO: extract visibility
                mir_calls: vec![],   // TODO: extract from MIR
                trait_impls: vec![], // TODO: extract trait impls
            });
        }
        Compilation::Continue
    }
}

pub fn run_compiler_enrichment_pass(project_root: &Path) -> Result<Vec<EnrichmentResult>> {
    let results = Arc::new(Mutex::new(Vec::new()));

    // Find source file
    let src_lib = project_root.join("src/lib.rs");
    let src_main = project_root.join("src/main.rs");
    let source_file = if src_lib.exists() {
        src_lib
    } else if src_main.exists() {
        src_main
    } else {
        return Ok(vec![]); // No Rust source found
    };

    let args: Vec<String> = vec![
        "rustc".to_string(),
        "--edition=2021".to_string(),
        source_file.to_string_lossy().to_string(),
    ];

    // Run compiler -- this may fail if the project doesn't compile
    // Wrap in catch_unwind for safety
    let results_clone = results.clone();
    let mut cb = EnrichmentCallbacks { results: results_clone };
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rustc_driver::run_compiler(&args, &mut cb);
    }));

    match run_result {
        Ok(()) => {}
        Err(_) => tracing::warn!("Compiler panicked (enrichment skipped)"),
    }

    let collected = Arc::try_unwrap(results)
        .map(|mutex| mutex.into_inner().unwrap_or_default())
        .unwrap_or_else(|arc| arc.lock().unwrap().clone());

    Ok(collected)
}
