from __future__ import annotations

import json
import mmap
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent / "rust-test-001"
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import benchmark_walk_runtime_lookup as bench


class BenchmarkWalkRuntimeLookupTests(unittest.TestCase):
    def setUp(self) -> None:
        self.base_dir = SCRIPT_DIR
        self.node_rows, self.edge_rows = bench.load_harness_exports_now(self.base_dir)
        self.filtered_edges = bench.select_benchmark_edges_now(
            self.edge_rows, bench.DEFAULT_EDGE_TYPE
        )
        self.node_keys, self.key_to_id = bench.build_dense_node_index(self.node_rows)

    def test_req_bench_001_loads_real_harness_exports(self) -> None:
        self.assertEqual(len(self.node_rows), 39)
        self.assertEqual(len(self.edge_rows), 67)
        self.assertEqual(len(self.filtered_edges), 31)

    def test_req_bench_002_writes_expected_snapshot_counts(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            snapshot_dir = bench.write_walk_snapshot_now(
                Path(temp_dir),
                self.node_keys,
                self.key_to_id,
                self.filtered_edges,
                bench.DEFAULT_EDGE_TYPE,
            )
            manifest = json.loads(
                (snapshot_dir / "manifest.json").read_text(encoding="utf-8")
            )
            forward_offsets = bench.read_uint32_values_now(
                snapshot_dir / f"{bench.DEFAULT_EDGE_TYPE}.fwd.offsets.bin"
            )
            forward_peers = bench.read_uint32_values_now(
                snapshot_dir / f"{bench.DEFAULT_EDGE_TYPE}.fwd.peers.bin"
            )
            reverse_offsets = bench.read_uint32_values_now(
                snapshot_dir / f"{bench.DEFAULT_EDGE_TYPE}.rev.offsets.bin"
            )
            reverse_peers = bench.read_uint32_values_now(
                snapshot_dir / f"{bench.DEFAULT_EDGE_TYPE}.rev.peers.bin"
            )

            self.assertEqual(manifest["node_count"], len(self.node_rows))
            self.assertEqual(manifest["edge_count"], len(self.filtered_edges))
            self.assertEqual(len(forward_offsets), len(self.node_rows) + 1)
            self.assertEqual(len(reverse_offsets), len(self.node_rows) + 1)
            self.assertEqual(len(forward_peers), len(self.filtered_edges))
            self.assertEqual(len(reverse_peers), len(self.filtered_edges))

    def test_req_bench_002_snapshot_reconstructs_edge_set(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            snapshot_dir = bench.write_walk_snapshot_now(
                Path(temp_dir),
                self.node_keys,
                self.key_to_id,
                self.filtered_edges,
                bench.DEFAULT_EDGE_TYPE,
            )
            forward_offsets = bench.read_uint32_values_now(
                snapshot_dir / f"{bench.DEFAULT_EDGE_TYPE}.fwd.offsets.bin"
            )
            forward_peers = bench.read_uint32_values_now(
                snapshot_dir / f"{bench.DEFAULT_EDGE_TYPE}.fwd.peers.bin"
            )
            rebuilt_edges = bench.reconstruct_edges_from_offsets_now(
                self.node_keys, forward_offsets, forward_peers
            )
            expected_edges = {
                (edge.from_id, edge.to_id) for edge in self.filtered_edges
            }
            self.assertEqual(rebuilt_edges, expected_edges)

    def test_req_bench_004_missing_libsql_exits_clearly(self) -> None:
        command = [sys.executable, str(SCRIPT_DIR / "benchmark_walk_runtime_lookup.py")]
        result = subprocess.run(
            command,
            cwd=SCRIPT_DIR,
            env={
                **os.environ,
                "PARSLETONGUE_BENCH_FORCE_MISSING_LIBSQL": "1",
            },
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        combined_text = f"{result.stdout}\n{result.stderr}"
        self.assertIn("libsql", combined_text)
        self.assertIn("no download or install was attempted", combined_text)

    def test_req_bench_005_query_corpus_covers_real_lookup_nodes(self) -> None:
        corpora = bench.build_query_corpora_now(self.filtered_edges, self.key_to_id)
        corpus_by_name = {corpus.family_name: corpus for corpus in corpora}
        self.assertEqual(len(corpus_by_name["forward_one"].query_keys), 19)
        self.assertEqual(len(corpus_by_name["reverse_one"].query_keys), 23)
        self.assertEqual(len(corpus_by_name["reverse_two"].query_keys), 23)

    def test_req_bench_006_csv_by_key_and_by_id_match(self) -> None:
        backend = bench.CsvScanBackend(
            self.base_dir / "interface_edges.csv",
            self.node_keys,
            bench.DEFAULT_EDGE_TYPE,
        )
        corpora = bench.build_query_corpora_now(self.filtered_edges, self.key_to_id)
        for corpus in corpora:
            for query_key, query_id in zip(corpus.query_keys, corpus.query_ids):
                key_result = bench.run_family_lookup_now(
                    backend, corpus.family_name, query_key, "by_key"
                )
                id_result = bench.run_family_lookup_now(
                    backend, corpus.family_name, query_id, "by_id"
                )
                translated_ids = tuple(sorted(self.key_to_id[key] for key in key_result))
                canonical_ids = tuple(sorted(int(node_id) for node_id in id_result))
                self.assertEqual(translated_ids, canonical_ids)

    def test_req_bench_007_walk_backend_uses_mmap_views(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            snapshot_dir = bench.write_walk_snapshot_now(
                Path(temp_dir),
                self.node_keys,
                self.key_to_id,
                self.filtered_edges,
                bench.DEFAULT_EDGE_TYPE,
            )
            backend = bench.WalkSnapshotBackend(snapshot_dir, bench.DEFAULT_EDGE_TYPE)
            try:
                self.assertIsInstance(backend.forward_offsets._mmap, mmap.mmap)
                self.assertIsInstance(backend.forward_peers._mmap, mmap.mmap)
                self.assertFalse(hasattr(backend, "forward_map"))
                self.assertFalse(hasattr(backend, "reverse_map"))
            finally:
                backend.close()

    def test_req_bench_008_and_009_report_outputs_metrics(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_dir = Path(temp_dir)
            records = [
                bench.MeasurementRecord(
                    backend_name="csv",
                    phase_name="hot_lookup",
                    family_name="forward_one",
                    query_mode="by_key",
                    pass_index=0,
                    operation_count=10,
                    total_ns=1000,
                    mean_ns=100.0,
                    checksum=3,
                ),
                bench.MeasurementRecord(
                    backend_name="csv",
                    phase_name="hot_lookup",
                    family_name="forward_one",
                    query_mode="by_key",
                    pass_index=1,
                    operation_count=10,
                    total_ns=1200,
                    mean_ns=120.0,
                    checksum=3,
                ),
            ]
            dataset_facts = {"node_count": 39, "edge_count": 67, "query_counts": {}}
            artifact_paths = {"artifacts_dir": str(output_dir)}
            payload = bench.report_benchmark_summary_now(
                output_dir, records, dataset_facts, artifact_paths
            )
            self.assertTrue((output_dir / "benchmark_raw.csv").exists())
            self.assertTrue((output_dir / "benchmark_report.json").exists())
            self.assertEqual(payload["dataset_facts"]["node_count"], 39)
            self.assertEqual(len(payload["summary_rows"]), 1)
            self.assertIn("p95_ns", payload["summary_rows"][0])

    def test_req_bench_010_docstring_references_architecture_docs(self) -> None:
        self.assertIsNotNone(bench.__doc__)
        assert bench.__doc__ is not None
        self.assertIn("A-20260415152053-v313-PRD-L2.md", bench.__doc__)
        self.assertIn("A-20260408115716-pensieve-walk-runtime-thesis.md", bench.__doc__)
        self.assertIn("A-20260408140806-walk-runtime-options-explainer.md", bench.__doc__)

    @unittest.skipUnless(
        bench.find_libsql_module_name_now() is not None,
        "libsql is not installed locally",
    )
    def test_req_bench_003_writes_indexed_libsql_catalog(self) -> None:
        module = bench.load_libsql_module_now()
        with tempfile.TemporaryDirectory() as temp_dir:
            db_path = bench.write_libsql_catalog_now(
                Path(temp_dir),
                module,
                self.node_rows,
                self.key_to_id,
                self.filtered_edges,
            )
            connection = bench.open_libsql_connection_now(module, db_path)
            try:
                cursor = connection.cursor()
                node_count = cursor.execute("SELECT COUNT(*) FROM nodes").fetchone()[0]
                edge_count = cursor.execute("SELECT COUNT(*) FROM edges").fetchone()[0]
                key_edge_count = cursor.execute(
                    "SELECT COUNT(*) FROM edge_keys"
                ).fetchone()[0]
                self.assertEqual(node_count, len(self.node_rows))
                self.assertEqual(edge_count, len(self.filtered_edges))
                self.assertEqual(key_edge_count, len(self.filtered_edges))
            finally:
                connection.close()


if __name__ == "__main__":
    unittest.main()
