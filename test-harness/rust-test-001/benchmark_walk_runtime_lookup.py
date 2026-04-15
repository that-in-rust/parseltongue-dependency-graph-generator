#!/usr/bin/env python3
"""v313 walk-runtime benchmark for persisted lookup paths.

Source rationale:
- docs/A-20260415152053-v313-PRD-L2.md
- docs/strategic-research/A-20260408115716-pensieve-walk-runtime-thesis.md
- docs/strategic-research/A-20260408140806-walk-runtime-options-explainer.md

This benchmark exists to test the v313 split between:
- a SQL metadata layer for key resolution and catalog facts
- a walk snapshot layer for one-hop and two-hop impact traversal

The walk backend intentionally uses map-shaped storage:
- forward offsets + forward peers
- backward offsets + backward peers

It does not walk a table one row at a time.
"""

from __future__ import annotations

import argparse
import csv
import importlib
import importlib.util
import json
import math
import mmap
import os
import struct
import sys
import tempfile
import time
from collections import defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Callable, Iterable


UINT32_SIZE = 4
DEFAULT_EDGE_TYPE = "depends_on"
DEFAULT_WARMUP_PASSES = 5
DEFAULT_MEASURE_PASSES = 30
DEFAULT_LOOPS_PER_PASS = 500


@dataclass(frozen=True)
class NodeRecord:
    node_key: str
    node_type: str
    label: str
    parent_id: str
    file_path: str
    span: str


@dataclass(frozen=True)
class EdgeRecord:
    from_id: str
    edge_type: str
    to_id: str


@dataclass(frozen=True)
class QueryCorpus:
    family_name: str
    query_keys: tuple[str, ...]
    query_ids: tuple[int, ...]


@dataclass(frozen=True)
class MeasurementRecord:
    backend_name: str
    phase_name: str
    family_name: str
    query_mode: str
    pass_index: int
    operation_count: int
    total_ns: int
    mean_ns: float
    checksum: int


class BenchmarkError(RuntimeError):
    """Typed benchmark failure for clear CLI exits."""


def find_libsql_module_name_now() -> str | None:
    """Return the first supported libSQL Python module name if present."""

    if os.environ.get("PARSLETONGUE_BENCH_FORCE_MISSING_LIBSQL") == "1":
        return None

    for module_name in ("libsql", "libsql_experimental"):
        if importlib.util.find_spec(module_name) is not None:
            return module_name
    return None


def load_libsql_module_now():
    """Load the preinstalled libSQL module or fail without fallback."""

    module_name = find_libsql_module_name_now()
    if module_name is None:
        raise BenchmarkError(
            "libsql Python module is not installed locally; "
            "no download or install was attempted."
        )
    return importlib.import_module(module_name)


def load_harness_exports_now(base_dir: Path) -> tuple[list[NodeRecord], list[EdgeRecord]]:
    """Load the committed harness exports without mutation."""

    nodes_path = base_dir / "interface_nodes.csv"
    edges_path = base_dir / "interface_edges.csv"
    node_rows: list[NodeRecord] = []
    edge_rows: list[EdgeRecord] = []

    with nodes_path.open("r", encoding="utf-8", newline="") as handle:
        for row in csv.DictReader(handle):
            node_rows.append(
                NodeRecord(
                    node_key=row["node_id"],
                    node_type=row["node_type"],
                    label=row["label"],
                    parent_id=row["parent_id"],
                    file_path=row["file_path"],
                    span=row["span"],
                )
            )

    with edges_path.open("r", encoding="utf-8", newline="") as handle:
        for row in csv.DictReader(handle):
            edge_rows.append(
                EdgeRecord(
                    from_id=row["from_id"],
                    edge_type=row["edge_type"],
                    to_id=row["to_id"],
                )
            )

    return node_rows, edge_rows


def select_benchmark_edges_now(
    edge_rows: list[EdgeRecord], edge_type: str
) -> list[EdgeRecord]:
    """Keep only the surfaced edge type we want to benchmark."""

    return [edge for edge in edge_rows if edge.edge_type == edge_type]


def build_dense_node_index(
    node_rows: list[NodeRecord],
) -> tuple[list[str], dict[str, int]]:
    """Assign stable dense IDs using the committed node order."""

    node_keys = [row.node_key for row in node_rows]
    key_to_id = {node_key: index for index, node_key in enumerate(node_keys)}
    return node_keys, key_to_id


def build_query_corpora_now(
    filtered_edges: list[EdgeRecord], key_to_id: dict[str, int]
) -> list[QueryCorpus]:
    """Derive real query corpora from the harness graph shape."""

    forward_map: dict[str, list[str]] = defaultdict(list)
    reverse_map: dict[str, list[str]] = defaultdict(list)

    for edge in filtered_edges:
        forward_map[edge.from_id].append(edge.to_id)
        reverse_map[edge.to_id].append(edge.from_id)

    forward_keys = tuple(forward_map.keys())
    reverse_one_keys = tuple(reverse_map.keys())

    reverse_two_keys: list[str] = []
    for seed_key in reverse_one_keys:
        seen_keys = set(reverse_map.get(seed_key, []))
        frontier_keys = tuple(seen_keys)
        for direct_key in frontier_keys:
            seen_keys.update(reverse_map.get(direct_key, []))
        seen_keys.discard(seed_key)
        if seen_keys:
            reverse_two_keys.append(seed_key)

    corpora = [
        QueryCorpus(
            family_name="forward_one",
            query_keys=forward_keys,
            query_ids=tuple(key_to_id[key] for key in forward_keys),
        ),
        QueryCorpus(
            family_name="reverse_one",
            query_keys=reverse_one_keys,
            query_ids=tuple(key_to_id[key] for key in reverse_one_keys),
        ),
        QueryCorpus(
            family_name="reverse_two",
            query_keys=tuple(reverse_two_keys),
            query_ids=tuple(key_to_id[key] for key in reverse_two_keys),
        ),
    ]
    return corpora


def build_adjacency_arrays_now(
    node_keys: list[str],
    key_to_id: dict[str, int],
    filtered_edges: list[EdgeRecord],
) -> tuple[list[int], list[int], list[int], list[int]]:
    """Build CSR-like forward arrays and CSC-like reverse arrays."""

    node_count = len(node_keys)
    forward_lists: list[list[int]] = [[] for _ in range(node_count)]
    reverse_lists: list[list[int]] = [[] for _ in range(node_count)]

    for edge in filtered_edges:
        src_id = key_to_id[edge.from_id]
        dst_id = key_to_id[edge.to_id]
        forward_lists[src_id].append(dst_id)
        reverse_lists[dst_id].append(src_id)

    forward_offsets, forward_peers = flatten_neighbor_lists_now(forward_lists)
    reverse_offsets, reverse_peers = flatten_neighbor_lists_now(reverse_lists)
    return forward_offsets, forward_peers, reverse_offsets, reverse_peers


def flatten_neighbor_lists_now(
    grouped_neighbors: list[list[int]],
) -> tuple[list[int], list[int]]:
    """Flatten grouped neighbors into offsets + peers."""

    offsets = [0]
    peers: list[int] = []
    for neighbors in grouped_neighbors:
        peers.extend(neighbors)
        offsets.append(len(peers))
    return offsets, peers


def write_uint32_values_now(output_path: Path, values: Iterable[int]) -> None:
    """Write little-endian uint32 values to disk."""

    items = tuple(int(value) for value in values)
    output_path.write_bytes(struct.pack(f"<{len(items)}I", *items))


def read_uint32_values_now(input_path: Path) -> tuple[int, ...]:
    """Read little-endian uint32 values from disk."""

    raw_bytes = input_path.read_bytes()
    if not raw_bytes:
        return tuple()
    value_count = len(raw_bytes) // UINT32_SIZE
    return struct.unpack(f"<{value_count}I", raw_bytes)


def write_walk_snapshot_now(
    artifacts_dir: Path,
    node_keys: list[str],
    key_to_id: dict[str, int],
    filtered_edges: list[EdgeRecord],
    edge_type: str,
) -> Path:
    """Materialize the walk-runtime snapshot files."""

    snapshot_dir = artifacts_dir / "walk_snapshot"
    snapshot_dir.mkdir(parents=True, exist_ok=True)

    forward_offsets, forward_peers, reverse_offsets, reverse_peers = (
        build_adjacency_arrays_now(node_keys, key_to_id, filtered_edges)
    )

    manifest = {
        "edge_type": edge_type,
        "node_count": len(node_keys),
        "edge_count": len(filtered_edges),
        "forward_offsets_path": f"{edge_type}.fwd.offsets.bin",
        "forward_peers_path": f"{edge_type}.fwd.peers.bin",
        "reverse_offsets_path": f"{edge_type}.rev.offsets.bin",
        "reverse_peers_path": f"{edge_type}.rev.peers.bin",
        "node_keys_path": "node_keys.txt",
    }

    (snapshot_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2), encoding="utf-8"
    )
    (snapshot_dir / "node_keys.txt").write_text(
        "\n".join(node_keys) + "\n", encoding="utf-8"
    )

    write_uint32_values_now(
        snapshot_dir / f"{edge_type}.fwd.offsets.bin", forward_offsets
    )
    write_uint32_values_now(snapshot_dir / f"{edge_type}.fwd.peers.bin", forward_peers)
    write_uint32_values_now(
        snapshot_dir / f"{edge_type}.rev.offsets.bin", reverse_offsets
    )
    write_uint32_values_now(snapshot_dir / f"{edge_type}.rev.peers.bin", reverse_peers)
    return snapshot_dir


def reconstruct_edges_from_offsets_now(
    node_keys: list[str], offsets: tuple[int, ...], peers: tuple[int, ...]
) -> set[tuple[str, str]]:
    """Rebuild an edge set for verification from offsets + peers."""

    rebuilt_edges: set[tuple[str, str]] = set()
    for node_id, node_key in enumerate(node_keys):
        start = offsets[node_id]
        end = offsets[node_id + 1]
        for peer_id in peers[start:end]:
            rebuilt_edges.add((node_key, node_keys[peer_id]))
    return rebuilt_edges


def open_libsql_connection_now(module, db_path: Path):
    """Open an on-disk libSQL connection using local-only URI styles."""

    failures: list[str] = []
    for candidate in (str(db_path), f"file:{db_path}"):
        try:
            return module.connect(candidate)
        except Exception as exc:  # pragma: no cover - exercised only with libsql present
            failures.append(f"{candidate}: {exc}")
    failure_text = "; ".join(failures)
    raise BenchmarkError(f"unable to open local libsql database: {failure_text}")


def write_libsql_catalog_now(
    artifacts_dir: Path,
    module,
    node_rows: list[NodeRecord],
    key_to_id: dict[str, int],
    filtered_edges: list[EdgeRecord],
) -> Path:
    """Create the SQL metadata catalog using the preinstalled libSQL module."""

    db_path = artifacts_dir / "catalog.libsql.db"
    connection = open_libsql_connection_now(module, db_path)
    try:
        cursor = connection.cursor()
        cursor.executescript(
            """
            DROP TABLE IF EXISTS nodes;
            DROP TABLE IF EXISTS edges;
            DROP TABLE IF EXISTS edge_keys;

            CREATE TABLE nodes(
                node_key TEXT PRIMARY KEY,
                dense_id INTEGER UNIQUE,
                node_type TEXT,
                label TEXT,
                file_path TEXT,
                span TEXT
            );

            CREATE TABLE edges(
                from_dense_id INTEGER,
                edge_type TEXT,
                to_dense_id INTEGER
            );

            CREATE TABLE edge_keys(
                from_key TEXT,
                edge_type TEXT,
                to_key TEXT
            );
            """
        )

        cursor.executemany(
            """
            INSERT INTO nodes(node_key, dense_id, node_type, label, file_path, span)
            VALUES (?, ?, ?, ?, ?, ?)
            """,
            [
                (
                    node.node_key,
                    key_to_id[node.node_key],
                    node.node_type,
                    node.label,
                    node.file_path,
                    node.span,
                )
                for node in node_rows
            ],
        )

        cursor.executemany(
            """
            INSERT INTO edges(from_dense_id, edge_type, to_dense_id)
            VALUES (?, ?, ?)
            """,
            [
                (key_to_id[edge.from_id], edge.edge_type, key_to_id[edge.to_id])
                for edge in filtered_edges
            ],
        )

        cursor.executemany(
            """
            INSERT INTO edge_keys(from_key, edge_type, to_key)
            VALUES (?, ?, ?)
            """,
            [(edge.from_id, edge.edge_type, edge.to_id) for edge in filtered_edges],
        )

        cursor.executescript(
            """
            CREATE INDEX idx_edges_type_from ON edges(edge_type, from_dense_id);
            CREATE INDEX idx_edges_type_to ON edges(edge_type, to_dense_id);
            CREATE INDEX idx_edge_keys_type_from ON edge_keys(edge_type, from_key);
            CREATE INDEX idx_edge_keys_type_to ON edge_keys(edge_type, to_key);
            """
        )
        connection.commit()
    finally:
        connection.close()
    return db_path


class CsvScanBackend:
    """Persisted CSV scanner that reopens the edge CSV per lookup."""

    def __init__(self, edges_csv_path: Path, node_keys: list[str], edge_type: str) -> None:
        self.edges_csv_path = edges_csv_path
        self.node_keys = node_keys
        self.edge_type = edge_type

    def close(self) -> None:
        return None


class LibsqlCatalogBackend:
    """SQL metadata backend for key and dense-id lookups."""

    def __init__(self, db_path: Path, module, edge_type: str) -> None:
        self.edge_type = edge_type
        self.connection = open_libsql_connection_now(module, db_path)

    def lookup_dense_id_now(self, node_key: str) -> int:
        cursor = self.connection.cursor()
        row = cursor.execute(
            "SELECT dense_id FROM nodes WHERE node_key = ?",
            (node_key,),
        ).fetchone()
        if row is None:
            raise KeyError(node_key)
        return int(row[0])

    def close(self) -> None:
        self.connection.close()


class UInt32MmapView:
    """Read-only uint32 view over a disk-backed memory map."""

    def __init__(self, input_path: Path) -> None:
        self._file = input_path.open("rb")
        self._mmap = mmap.mmap(self._file.fileno(), 0, access=mmap.ACCESS_READ)
        self.length = len(self._mmap) // UINT32_SIZE

    def read_value_now(self, index: int) -> int:
        return struct.unpack_from("<I", self._mmap, index * UINT32_SIZE)[0]

    def read_slice_now(self, start: int, end: int) -> tuple[int, ...]:
        if end <= start:
            return tuple()
        count = end - start
        return struct.unpack_from(
            f"<{count}I", self._mmap, start * UINT32_SIZE
        )

    def close(self) -> None:
        self._mmap.close()
        self._file.close()


class WalkSnapshotBackend:
    """Disk-backed walk snapshot using mmap for offsets and peers."""

    def __init__(self, snapshot_dir: Path, edge_type: str) -> None:
        self.snapshot_dir = snapshot_dir
        self.edge_type = edge_type
        self.node_keys = (snapshot_dir / "node_keys.txt").read_text(
            encoding="utf-8"
        ).splitlines()
        self.forward_offsets = UInt32MmapView(
            snapshot_dir / f"{edge_type}.fwd.offsets.bin"
        )
        self.forward_peers = UInt32MmapView(snapshot_dir / f"{edge_type}.fwd.peers.bin")
        self.reverse_offsets = UInt32MmapView(
            snapshot_dir / f"{edge_type}.rev.offsets.bin"
        )
        self.reverse_peers = UInt32MmapView(snapshot_dir / f"{edge_type}.rev.peers.bin")

    def lookup_forward_ids_now(self, dense_id: int) -> tuple[int, ...]:
        start = self.forward_offsets.read_value_now(dense_id)
        end = self.forward_offsets.read_value_now(dense_id + 1)
        return self.forward_peers.read_slice_now(start, end)

    def lookup_reverse_ids_now(self, dense_id: int) -> tuple[int, ...]:
        start = self.reverse_offsets.read_value_now(dense_id)
        end = self.reverse_offsets.read_value_now(dense_id + 1)
        return self.reverse_peers.read_slice_now(start, end)

    def close(self) -> None:
        self.forward_offsets.close()
        self.forward_peers.close()
        self.reverse_offsets.close()
        self.reverse_peers.close()


class CompositeWalkBackend:
    """Two-layer backend: SQL metadata lookup plus mmap walk snapshot."""

    def __init__(self, catalog_backend: LibsqlCatalogBackend, walk_backend: WalkSnapshotBackend) -> None:
        self.catalog_backend = catalog_backend
        self.walk_backend = walk_backend

    def close(self) -> None:
        self.walk_backend.close()
        self.catalog_backend.close()


def lookup_forward_neighbors_now(
    backend, query_value: str | int, query_mode: str
) -> tuple[str | int, ...]:
    """Return forward one-hop neighbors for the given backend."""

    if isinstance(backend, CsvScanBackend):
        if query_mode == "by_id":
            source_key = backend.node_keys[int(query_value)]
        else:
            source_key = str(query_value)
        matches: list[str] = []
        with backend.edges_csv_path.open("r", encoding="utf-8", newline="") as handle:
            for row in csv.DictReader(handle):
                if row["edge_type"] == backend.edge_type and row["from_id"] == source_key:
                    matches.append(row["to_id"])
        if query_mode == "by_id":
            key_to_id = {key: index for index, key in enumerate(backend.node_keys)}
            return tuple(key_to_id[key] for key in matches)
        return tuple(matches)

    if isinstance(backend, LibsqlCatalogBackend):
        cursor = backend.connection.cursor()
        if query_mode == "by_key":
            rows = cursor.execute(
                """
                SELECT to_key
                FROM edge_keys
                WHERE edge_type = ? AND from_key = ?
                ORDER BY to_key
                """,
                (backend.edge_type, str(query_value)),
            ).fetchall()
            return tuple(row[0] for row in rows)
        rows = cursor.execute(
            """
            SELECT to_dense_id
            FROM edges
            WHERE edge_type = ? AND from_dense_id = ?
            ORDER BY to_dense_id
            """,
            (backend.edge_type, int(query_value)),
        ).fetchall()
        return tuple(int(row[0]) for row in rows)

    if isinstance(backend, WalkSnapshotBackend):
        return backend.lookup_forward_ids_now(int(query_value))

    if isinstance(backend, CompositeWalkBackend):
        if query_mode == "by_id":
            return backend.walk_backend.lookup_forward_ids_now(int(query_value))
        dense_id = backend.catalog_backend.lookup_dense_id_now(str(query_value))
        neighbor_ids = backend.walk_backend.lookup_forward_ids_now(dense_id)
        return tuple(
            backend.walk_backend.node_keys[neighbor_id] for neighbor_id in neighbor_ids
        )

    raise TypeError(f"unsupported backend: {type(backend)!r}")


def lookup_reverse_neighbors_now(
    backend, query_value: str | int, query_mode: str, hop_count: int
) -> tuple[str | int, ...]:
    """Return reverse neighbors within the requested hop count."""

    if hop_count not in (1, 2):
        raise ValueError("hop_count must be 1 or 2")

    if isinstance(backend, CsvScanBackend):
        if query_mode == "by_id":
            target_key = backend.node_keys[int(query_value)]
        else:
            target_key = str(query_value)
        direct_hits: list[str] = []
        with backend.edges_csv_path.open("r", encoding="utf-8", newline="") as handle:
            for row in csv.DictReader(handle):
                if row["edge_type"] == backend.edge_type and row["to_id"] == target_key:
                    direct_hits.append(row["from_id"])
        if hop_count == 1:
            result_keys = tuple(direct_hits)
        else:
            seen_keys = set(direct_hits)
            for direct_key in tuple(direct_hits):
                seen_keys.update(
                    lookup_reverse_neighbors_now(backend, direct_key, "by_key", 1)
                )
            seen_keys.discard(target_key)
            result_keys = tuple(sorted(seen_keys))
        if query_mode == "by_id":
            key_to_id = {key: index for index, key in enumerate(backend.node_keys)}
            return tuple(key_to_id[key] for key in result_keys)
        return result_keys

    if isinstance(backend, LibsqlCatalogBackend):
        cursor = backend.connection.cursor()
        if query_mode == "by_key":
            direct_rows = cursor.execute(
                """
                SELECT from_key
                FROM edge_keys
                WHERE edge_type = ? AND to_key = ?
                ORDER BY from_key
                """,
                (backend.edge_type, str(query_value)),
            ).fetchall()
            direct_values = tuple(row[0] for row in direct_rows)
        else:
            direct_rows = cursor.execute(
                """
                SELECT from_dense_id
                FROM edges
                WHERE edge_type = ? AND to_dense_id = ?
                ORDER BY from_dense_id
                """,
                (backend.edge_type, int(query_value)),
            ).fetchall()
            direct_values = tuple(int(row[0]) for row in direct_rows)

        if hop_count == 1:
            return direct_values

        seen_values = set(direct_values)
        for direct_value in direct_values:
            seen_values.update(
                lookup_reverse_neighbors_now(backend, direct_value, query_mode, 1)
            )
        seen_values.discard(query_value)
        return tuple(sorted(seen_values))

    if isinstance(backend, WalkSnapshotBackend):
        direct_values = backend.lookup_reverse_ids_now(int(query_value))
        if hop_count == 1:
            return direct_values
        seen_ids = set(direct_values)
        for direct_id in direct_values:
            seen_ids.update(backend.lookup_reverse_ids_now(direct_id))
        seen_ids.discard(int(query_value))
        return tuple(sorted(seen_ids))

    if isinstance(backend, CompositeWalkBackend):
        if query_mode == "by_id":
            return lookup_reverse_neighbors_now(
                backend.walk_backend, int(query_value), "by_id", hop_count
            )
        dense_id = backend.catalog_backend.lookup_dense_id_now(str(query_value))
        result_ids = lookup_reverse_neighbors_now(
            backend.walk_backend, dense_id, "by_id", hop_count
        )
        return tuple(
            backend.walk_backend.node_keys[result_id] for result_id in result_ids
        )

    raise TypeError(f"unsupported backend: {type(backend)!r}")


def run_family_lookup_now(
    backend, family_name: str, query_value: str | int, query_mode: str
) -> tuple[str | int, ...]:
    """Dispatch a family lookup for measurement and correctness checks."""

    if family_name == "forward_one":
        return lookup_forward_neighbors_now(backend, query_value, query_mode)
    if family_name == "reverse_one":
        return lookup_reverse_neighbors_now(backend, query_value, query_mode, 1)
    if family_name == "reverse_two":
        return lookup_reverse_neighbors_now(backend, query_value, query_mode, 2)
    raise ValueError(f"unsupported family: {family_name}")


def percentile_sorted_now(sorted_values: list[float], percentile: float) -> float:
    """Compute a simple linear-interpolated percentile."""

    if not sorted_values:
        return 0.0
    if len(sorted_values) == 1:
        return sorted_values[0]
    rank = percentile * (len(sorted_values) - 1)
    lower_index = math.floor(rank)
    upper_index = math.ceil(rank)
    if lower_index == upper_index:
        return sorted_values[lower_index]
    weight = rank - lower_index
    return (
        sorted_values[lower_index] * (1.0 - weight)
        + sorted_values[upper_index] * weight
    )


def measure_phase_latency_now(
    backend_name: str,
    phase_name: str,
    query_mode: str,
    family_name: str,
    query_values: tuple[str | int, ...],
    backend_factory: Callable[[], object],
    warmup_passes: int,
    measure_passes: int,
    loops_per_pass: int,
) -> list[MeasurementRecord]:
    """Measure one backend / phase / query family."""

    records: list[MeasurementRecord] = []
    total_passes = warmup_passes + measure_passes

    def close_backend_now(backend_object: object) -> None:
        close_method = getattr(backend_object, "close", None)
        if callable(close_method):
            close_method()

    for pass_index in range(total_passes):
        checksum = 0
        operation_count = len(query_values) * loops_per_pass
        start_ns = time.perf_counter_ns()
        if phase_name == "hot_lookup":
            backend = backend_factory()
            try:
                for _ in range(loops_per_pass):
                    for query_value in query_values:
                        checksum += len(
                            run_family_lookup_now(
                                backend, family_name, query_value, query_mode
                            )
                        )
            finally:
                close_backend_now(backend)
        else:
            for _ in range(loops_per_pass):
                for query_value in query_values:
                    backend = backend_factory()
                    try:
                        checksum += len(
                            run_family_lookup_now(
                                backend, family_name, query_value, query_mode
                            )
                        )
                    finally:
                        close_backend_now(backend)
        total_ns = time.perf_counter_ns() - start_ns
        mean_ns = total_ns / operation_count if operation_count else 0.0
        if pass_index >= warmup_passes:
            records.append(
                MeasurementRecord(
                    backend_name=backend_name,
                    phase_name=phase_name,
                    family_name=family_name,
                    query_mode=query_mode,
                    pass_index=pass_index - warmup_passes,
                    operation_count=operation_count,
                    total_ns=total_ns,
                    mean_ns=mean_ns,
                    checksum=checksum,
                )
            )
    return records


def measure_backend_latency_now(
    backend_name: str,
    backend_factory: Callable[[], object],
    corpora: list[QueryCorpus],
    warmup_passes: int,
    measure_passes: int,
    loops_per_pass: int,
) -> list[MeasurementRecord]:
    """Measure hot and cold latencies for all corpora and modes."""

    records: list[MeasurementRecord] = []
    for corpus in corpora:
        mode_pairs = [
            ("by_key", corpus.query_keys),
            ("by_id", corpus.query_ids),
        ]
        for query_mode, query_values in mode_pairs:
            if not query_values:
                continue
            records.extend(
                measure_phase_latency_now(
                    backend_name=backend_name,
                    phase_name="cold_open_lookup",
                    query_mode=query_mode,
                    family_name=corpus.family_name,
                    query_values=query_values,
                    backend_factory=backend_factory,
                    warmup_passes=warmup_passes,
                    measure_passes=measure_passes,
                    loops_per_pass=loops_per_pass,
                )
            )
            records.extend(
                measure_phase_latency_now(
                    backend_name=backend_name,
                    phase_name="hot_lookup",
                    query_mode=query_mode,
                    family_name=corpus.family_name,
                    query_values=query_values,
                    backend_factory=backend_factory,
                    warmup_passes=warmup_passes,
                    measure_passes=measure_passes,
                    loops_per_pass=loops_per_pass,
                )
            )
    return records


def aggregate_measurements_now(
    records: list[MeasurementRecord],
) -> list[dict[str, object]]:
    """Aggregate pass-level mean latencies into summary stats."""

    grouped: dict[tuple[str, str, str, str], list[MeasurementRecord]] = defaultdict(list)
    for record in records:
        group_key = (
            record.backend_name,
            record.phase_name,
            record.family_name,
            record.query_mode,
        )
        grouped[group_key].append(record)

    summary_rows: list[dict[str, object]] = []
    for group_key, group_records in sorted(grouped.items()):
        mean_values = sorted(record.mean_ns for record in group_records)
        operation_counts = sorted(record.operation_count for record in group_records)
        summary_rows.append(
            {
                "backend_name": group_key[0],
                "phase_name": group_key[1],
                "family_name": group_key[2],
                "query_mode": group_key[3],
                "pass_count": len(group_records),
                "operation_count": operation_counts[0] if operation_counts else 0,
                "mean_ns": sum(mean_values) / len(mean_values) if mean_values else 0.0,
                "p50_ns": percentile_sorted_now(mean_values, 0.50),
                "p95_ns": percentile_sorted_now(mean_values, 0.95),
                "min_ns": min(mean_values) if mean_values else 0.0,
                "max_ns": max(mean_values) if mean_values else 0.0,
            }
        )
    return summary_rows


def write_raw_measurements_now(
    output_path: Path, records: list[MeasurementRecord]
) -> None:
    """Write pass-level raw measurements."""

    fieldnames = [
        "backend_name",
        "phase_name",
        "family_name",
        "query_mode",
        "pass_index",
        "operation_count",
        "total_ns",
        "mean_ns",
        "checksum",
    ]
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        for record in records:
            writer.writerow(asdict(record))


def report_benchmark_summary_now(
    output_dir: Path,
    records: list[MeasurementRecord],
    dataset_facts: dict[str, object],
    artifact_paths: dict[str, str],
) -> dict[str, object]:
    """Write summary JSON / CSV and print a compact stdout table."""

    summary_rows = aggregate_measurements_now(records)
    raw_csv_path = output_dir / "benchmark_raw.csv"
    report_json_path = output_dir / "benchmark_report.json"

    write_raw_measurements_now(raw_csv_path, records)
    report_payload = {
        "dataset_facts": dataset_facts,
        "artifact_paths": artifact_paths,
        "summary_rows": summary_rows,
    }
    report_json_path.write_text(
        json.dumps(report_payload, indent=2), encoding="utf-8"
    )

    headers = (
        "backend".ljust(10),
        "phase".ljust(17),
        "family".ljust(12),
        "mode".ljust(7),
        "mean_ns".rjust(12),
        "p50_ns".rjust(12),
        "p95_ns".rjust(12),
    )
    print(" ".join(headers))
    for row in summary_rows:
        print(
            " ".join(
                [
                    str(row["backend_name"]).ljust(10),
                    str(row["phase_name"]).ljust(17),
                    str(row["family_name"]).ljust(12),
                    str(row["query_mode"]).ljust(7),
                    f"{row['mean_ns']:.2f}".rjust(12),
                    f"{row['p50_ns']:.2f}".rjust(12),
                    f"{row['p95_ns']:.2f}".rjust(12),
                ]
            )
        )
    return report_payload


def validate_query_parity_now(
    csv_backend: CsvScanBackend,
    walk_backend: WalkSnapshotBackend,
    key_to_id: dict[str, int],
    corpora: list[QueryCorpus],
) -> None:
    """Verify that CSV scan and walk snapshot agree on structural lookups."""

    for corpus in corpora:
        for query_key, query_id in zip(corpus.query_keys, corpus.query_ids):
            csv_forward_by_key = run_family_lookup_now(
                csv_backend, corpus.family_name, query_key, "by_key"
            )
            csv_forward_by_id = run_family_lookup_now(
                csv_backend, corpus.family_name, query_id, "by_id"
            )
            walk_by_id = run_family_lookup_now(
                walk_backend, corpus.family_name, query_id, "by_id"
            )

            csv_ids_from_keys = tuple(sorted(key_to_id[key] for key in csv_forward_by_key))
            csv_ids_by_id = tuple(sorted(int(node_id) for node_id in csv_forward_by_id))
            walk_ids_canonical = tuple(sorted(int(node_id) for node_id in walk_by_id))
            if csv_ids_from_keys != csv_ids_by_id:
                raise BenchmarkError(
                    f"CSV by_key/by_id mismatch for {corpus.family_name}:{query_key}"
                )
            if walk_ids_canonical != csv_ids_from_keys:
                raise BenchmarkError(
                    f"walk snapshot mismatch for {corpus.family_name}:{query_key}"
                )


def parse_cli_arguments_now(argv: list[str]) -> argparse.Namespace:
    """Parse the benchmark CLI."""

    parser = argparse.ArgumentParser(
        description="Benchmark persisted walk-runtime lookup paths for v313."
    )
    parser.add_argument(
        "--input-dir",
        default=str(Path(__file__).resolve().parent),
        help="Directory containing interface_nodes.csv and interface_edges.csv",
    )
    parser.add_argument(
        "--artifacts-dir",
        default=None,
        help="Directory for generated snapshot, database, and reports",
    )
    parser.add_argument(
        "--edge-type",
        default=DEFAULT_EDGE_TYPE,
        help=f"Surfaced edge type to benchmark (default: {DEFAULT_EDGE_TYPE})",
    )
    parser.add_argument(
        "--warmup-passes",
        type=int,
        default=DEFAULT_WARMUP_PASSES,
        help=f"Warmup passes per backend/family (default: {DEFAULT_WARMUP_PASSES})",
    )
    parser.add_argument(
        "--measure-passes",
        type=int,
        default=DEFAULT_MEASURE_PASSES,
        help=f"Measured passes per backend/family (default: {DEFAULT_MEASURE_PASSES})",
    )
    parser.add_argument(
        "--loops-per-pass",
        type=int,
        default=DEFAULT_LOOPS_PER_PASS,
        help=f"Lookup loops per pass (default: {DEFAULT_LOOPS_PER_PASS})",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    """CLI entrypoint."""

    args = parse_cli_arguments_now(argv or sys.argv[1:])
    base_dir = Path(args.input_dir).resolve()
    if args.artifacts_dir is None:
        artifacts_dir = base_dir / "bench_artifacts"
    else:
        artifacts_dir = Path(args.artifacts_dir).resolve()

    try:
        libsql_module = load_libsql_module_now()
        node_rows, edge_rows = load_harness_exports_now(base_dir)
        filtered_edges = select_benchmark_edges_now(edge_rows, args.edge_type)
        node_keys, key_to_id = build_dense_node_index(node_rows)
        corpora = build_query_corpora_now(filtered_edges, key_to_id)

        artifacts_dir.mkdir(parents=True, exist_ok=True)
        snapshot_dir = write_walk_snapshot_now(
            artifacts_dir, node_keys, key_to_id, filtered_edges, args.edge_type
        )
        libsql_db_path = write_libsql_catalog_now(
            artifacts_dir, libsql_module, node_rows, key_to_id, filtered_edges
        )

        csv_backend = CsvScanBackend(
            base_dir / "interface_edges.csv", node_keys, args.edge_type
        )
        walk_backend = WalkSnapshotBackend(snapshot_dir, args.edge_type)
        try:
            validate_query_parity_now(csv_backend, walk_backend, key_to_id, corpora)
        finally:
            walk_backend.close()

        csv_factory = lambda: CsvScanBackend(
            base_dir / "interface_edges.csv", node_keys, args.edge_type
        )
        libsql_factory = lambda: LibsqlCatalogBackend(
            libsql_db_path, libsql_module, args.edge_type
        )
        walk_factory = lambda: CompositeWalkBackend(
            LibsqlCatalogBackend(libsql_db_path, libsql_module, args.edge_type),
            WalkSnapshotBackend(snapshot_dir, args.edge_type),
        )

        records: list[MeasurementRecord] = []
        records.extend(
            measure_backend_latency_now(
                "csv",
                csv_factory,
                corpora,
                args.warmup_passes,
                args.measure_passes,
                args.loops_per_pass,
            )
        )
        records.extend(
            measure_backend_latency_now(
                "libsql",
                libsql_factory,
                corpora,
                args.warmup_passes,
                args.measure_passes,
                args.loops_per_pass,
            )
        )
        records.extend(
            measure_backend_latency_now(
                "walk",
                walk_factory,
                corpora,
                args.warmup_passes,
                args.measure_passes,
                args.loops_per_pass,
            )
        )

        dataset_facts = {
            "node_count": len(node_rows),
            "edge_count": len(edge_rows),
            "filtered_edge_count": len(filtered_edges),
            "query_counts": {
                corpus.family_name: len(corpus.query_keys) for corpus in corpora
            },
            "warmup_passes": args.warmup_passes,
            "measure_passes": args.measure_passes,
            "loops_per_pass": args.loops_per_pass,
        }
        artifact_paths = {
            "base_dir": str(base_dir),
            "artifacts_dir": str(artifacts_dir),
            "snapshot_dir": str(snapshot_dir),
            "libsql_db_path": str(libsql_db_path),
            "raw_csv_path": str(artifacts_dir / "benchmark_raw.csv"),
            "report_json_path": str(artifacts_dir / "benchmark_report.json"),
        }
        report_benchmark_summary_now(artifacts_dir, records, dataset_facts, artifact_paths)
        return 0
    except BenchmarkError as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
