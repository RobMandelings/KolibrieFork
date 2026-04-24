import json
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Mapping, Sequence, Tuple, Callable, Set
import pandas as pd


@dataclass
class TreeNode:
    frame_id: Optional[int]
    self_bytes: int = 0
    self_tg: int = 0
    agg_bytes: int = 0
    agg_tg: int = 0
    self_blocks: int = 0
    agg_blocks: int = 0
    self_avg_size: float = 0.0  # NEW
    agg_avg_size: float = 0.0  # NEW
    children: Dict[int, "TreeNode"] = field(default_factory=dict)
    frame_ids: List[int] = field(default_factory=list)
    labels: List[str] = field(default_factory=list)


def build_tree(dhat_json: dict) -> TreeNode:
    """Builds a call-tree over all pps entries."""
    root = TreeNode(frame_id=None)

    pps = dhat_json["pps"]
    # For convenience, fall back to 0 if some totals are missing
    for pp in pps:
        tb = pp.get("tb", 0)  # total bytes (or choose te/tg depending on dhat mode)
        tg = dhat_json.get("tg", 0)  # global 'tg' if you want; or pp.get("tg", 0)
        tk = pp.get("tbk", 0)  # NEW: total blocks for this PP

        fs = pp["fs"]  # list of frame indices forming the call stack suffix
        # In many dhat JSONs, fs is "root → leaf" or the reverse; adjust if needed
        node = root
        for frame_id in fs:
            if frame_id not in node.children:
                node.children[frame_id] = TreeNode(frame_id=frame_id)
            node = node.children[frame_id]

        # Add this allocation site's contribution to the leaf node
        node.self_bytes += tb
        node.self_tg += tg
        node.self_blocks += tk

    # After building, compute aggregated totals
    def compute_aggregates(n: TreeNode) -> None:
        total_bytes = n.self_bytes
        total_tg = n.self_tg
        total_blocks = n.self_blocks
        for child in n.children.values():
            compute_aggregates(child)
            total_bytes += child.agg_bytes
            total_tg += child.agg_tg
            total_blocks += child.agg_blocks
        n.agg_bytes = total_bytes
        n.agg_tg = total_tg
        n.agg_blocks = total_blocks

        # NEW: average sizes
        n.self_avg_size = n.self_bytes / n.self_blocks if n.self_blocks > 0 else 0.0
        n.agg_avg_size = n.agg_bytes / n.agg_blocks if n.agg_blocks > 0 else 0.0

    compute_aggregates(root)

    def compress_chains(node: TreeNode) -> None:
        # First recurse into children so they’re compressed
        for child in list(node.children.values()):
            compress_chains(child)

        # Now try to compress from this node downward
        while (
                len(node.children) == 1
                and node.self_bytes == 0
                and node.self_tg == 0
                and node.self_blocks == 0
        ):
            # Exactly one child, and this node has no self allocations:
            (child_id, child) = next(iter(node.children.items()))

            # If node.frame_ids is empty, start it with our own frame_id (if any)
            if not node.frame_ids and node.frame_id is not None:
                node.frame_ids.append(node.frame_id)

            # Append child's frame_id (and any existing frame_ids chain)
            if child.frame_ids:
                # child already represents a chain; extend with all of it
                node.frame_ids.append(child.frame_id)
                node.frame_ids.extend(child.frame_ids)
            else:
                node.frame_ids.append(child.frame_id)

            # Inherit aggregated totals and children from the child
            node.self_bytes += child.self_bytes
            node.self_tg += child.self_tg
            node.self_blocks += child.self_blocks

            node.agg_bytes = child.agg_bytes
            node.agg_tg = child.agg_tg
            node.agg_blocks = child.agg_blocks

            node.self_avg_size = node.self_bytes / node.self_blocks if node.self_blocks > 0 else 0.0
            node.agg_avg_size = node.agg_bytes / node.agg_blocks if node.agg_blocks > 0 else 0.0

            node.children = child.children

    compress_chains(root)
    root = filter_by_percentage(root, 1)
    return root


def format_frame(ftbl: List[str], frame_id: int) -> str:
    """Return a short function description from ftbl entry."""
    entry = ftbl[frame_id]
    # Example entry: "0x100fbb988: alloc::vec::Vec<T,A>::push (src/vec/mod.rs:994:22)"
    # You can parse this more nicely if you want.
    return entry


def print_tree(node: TreeNode, ftbl: List[str], indent: str = "", max_depth: int = 10, min_bytes: int = 0):
    """Pretty-print the tree as an indented table-like view."""
    # Skip the artificial root
    if node.frame_id is not None:
        name = format_frame(ftbl, node.frame_id)
        if node.agg_bytes < min_bytes:
            return  # pruning small contributions
        print(f"{indent}{node.agg_bytes:10d} B  ({node.self_bytes:10d} self)  {name}")

        indent += "  "

    if max_depth <= 0:
        return

    # Sort children by agg_bytes descending
    for child in sorted(node.children.values(), key=lambda c: c.agg_bytes, reverse=True):
        print_tree(child, ftbl, indent=indent, max_depth=max_depth - 1, min_bytes=min_bytes)


def filter_by_percentage(root: TreeNode, min_percent: float) -> TreeNode:
    """
    Filter the tree so that only nodes with agg_bytes / root.agg_bytes * 100
    >= min_percent are kept, plus any ancestors needed to reach them.

    Operates in-place on `root.children`.
    """
    total = root.agg_bytes or 1  # avoid division by zero

    def recurse(node: TreeNode) -> bool:
        # Compute this node's percentage of total
        pct = (node.agg_bytes / total) * 100.0

        # First process children; keep only those that pass recursively
        kept_children: Dict[int, TreeNode] = {}
        for fid, child in node.children.items():
            if recurse(child):
                kept_children[fid] = child
        node.children = kept_children

        # We keep this node if:
        #   - it meets the percentage threshold itself, OR
        #   - it has any kept children (so we preserve the path to them)
        return pct >= min_percent or bool(node.children)

    # Call recurse on each child of the root, but do NOT drop the root itself
    kept_children: Dict[int, TreeNode] = {}
    for fid, child in root.children.items():
        if recurse(child):
            kept_children[fid] = child
    root.children = kept_children

    return root


FrameMatcher = Callable[[str], bool]  # takes ftbl entry, returns True/False


@dataclass
class StackPattern:
    label: str
    # ordered list of matchers; we’ll check if the node’s stack contains these in order
    matchers: List[FrameMatcher]
    negative_matchers: List[FrameMatcher] = None  # none of these may appear


def contains(substring: str) -> FrameMatcher:
    return lambda frame: substring in frame


patterns = [
    # Deep element clone (Event + Box) as part of window_closed
    StackPattern(
        label="element_clone_from_window_closed",
        matchers=[
            contains("<prototypes::prototype::event::Event<I> as core::clone::Clone>::clone"),
            contains("<alloc::vec::Vec<T,A> as core::clone::Clone>::clone"),
            contains("as prototypes::prototype::slide_strategy::WindowSnapshotStrategy<I>>::window_closed"),
        ],
        negative_matchers=[],  # no exclusions
    ),

    # Vector-only clone from window_closed (must NOT go through Event::clone / Box::clone)
    StackPattern(
        label="vector_clone_from_window_closed",
        matchers=[
            contains("<alloc::vec::Vec<T,A> as core::clone::Clone>::clone"),
            contains("as prototypes::prototype::slide_strategy::WindowSnapshotStrategy<I>>::window_closed"),
        ],
        negative_matchers=[
            contains("<prototypes::prototype::event::Event<I> as core::clone::Clone>::clone"),
        ],
    ),

    StackPattern(
        label="vec_collect_in_event_arrives (IRI collection)",
        matchers=[
            contains("<alloc::vec::Vec<T> as core::iter::traits::collect::FromIterator<T>>::from_iter"),
            contains("core::iter::traits::iterator::Iterator::collect"),
            contains("prototypes::prototype::sliding_window_op::SlidingWindowOperator<I,S>::event_arrives"),
        ],
    ),
]


def node_stack_indices(node: TreeNode) -> List[int]:
    if node.frame_id is None:
        return list(node.frame_ids)
    if node.frame_ids:
        return [node.frame_id] + list(node.frame_ids)
    return [node.frame_id]


def node_stack_strings(node: TreeNode, ftbl: List[str]) -> List[str]:
    return [ftbl[i] for i in node_stack_indices(node)]


def stack_matches_pattern(stack: List[str], pattern: StackPattern) -> bool:
    # 1) negative: if any negative matcher hits anywhere, reject
    if pattern.negative_matchers:
        for frame in stack:
            for neg in pattern.negative_matchers:
                if neg(frame):
                    return False

    # 2) positive: subsequence match as before
    i = 0
    for matcher in pattern.matchers:
        while i < len(stack) and not matcher(stack[i]):
            i += 1
        if i == len(stack):
            return False
        i += 1
    return True


def label_nodes_by_patterns(root: TreeNode, ftbl: List[str], patterns: List[StackPattern]) -> None:
    """
    Traverse the tree and assign semantic labels to nodes based on their call stack.
    A node can get multiple labels.
    """

    def recurse(node: TreeNode) -> None:
        stack = node_stack_strings(node, ftbl)

        for pattern in patterns:
            if stack_matches_pattern(stack, pattern):
                node.labels.append(pattern.label)

        for child in node.children.values():
            recurse(child)

    recurse(root)


def filter_tree_by_labels(root: TreeNode, keep_labels: Set[str]) -> TreeNode:
    """
    Prune the tree so it only contains:
      - nodes whose labels intersect keep_labels, and
      - all their ancestors (up to root).

    Modifies root.children in-place and returns root.
    Root itself is never removed.
    """

    def recurse(node: TreeNode) -> bool:
        # First process children and keep only those whose subtree matches
        kept_children: Dict[int, TreeNode] = {}
        for fid, child in node.children.items():
            if recurse(child):
                kept_children[fid] = child
        node.children = kept_children

        # Does this node have a matching label?
        has_label = any(label in keep_labels for label in node.labels)

        # Keep this node if:
        #   - it has a desired label, OR
        #   - it has any kept children (so we keep ancestors)
        return has_label or bool(node.children)

    # Apply to children of root; we never drop the root node itself
    kept_children: Dict[int, TreeNode] = {}
    for fid, child in root.children.items():
        if recurse(child):
            kept_children[fid] = child
    root.children = kept_children

    return root


def dataframe_from_labeled_tree(root: TreeNode) -> pd.DataFrame:
    """
    Return a pandas DataFrame with:
      - one TOTAL row
      - one row per labeled node

    Columns:
      - total_bytes
      - total_bytes_pct
      - total_blocks
      - total_blocks_pct
      - average_size
    """
    rows = []

    total_bytes = root.agg_bytes
    total_blocks = root.agg_blocks

    # TOTAL row
    rows.append({
        "label": "TOTAL",
        "total_bytes": root.agg_bytes,
        "total_bytes_pct": 100.0,
        "total_blocks": root.agg_blocks,
        "total_blocks_pct": 100.0,
        "average_size": root.agg_avg_size,
    })

    def recurse(node: TreeNode):
        if node.labels:
            for label in node.labels:
                rows.append({
                    "label": label,
                    "total_bytes": node.agg_bytes,
                    "total_bytes_pct": (node.agg_bytes / total_bytes * 100.0) if total_bytes > 0 else 0.0,
                    "total_blocks": node.agg_blocks,
                    "total_blocks_pct": (node.agg_blocks / total_blocks * 100.0) if total_blocks > 0 else 0.0,
                    "average_size": node.agg_avg_size,
                })
        for child in node.children.values():
            recurse(child)

    recurse(root)

    df = pd.DataFrame(rows)

    # If the same label appears multiple times, aggregate it
    df = (
        df.groupby("label", as_index=False)
            .agg({
            "total_bytes": "sum",
            "total_blocks": "sum",
        })
    )

    # Recompute percentages and average size after aggregation
    df["total_bytes_pct"] = (df["total_bytes"] / total_bytes * 100.0) if total_bytes > 0 else 0.0
    df["total_blocks_pct"] = (df["total_blocks"] / total_blocks * 100.0) if total_blocks > 0 else 0.0
    df["average_size"] = df["total_bytes"] / df["total_blocks"]
    df.loc[df["total_blocks"] == 0, "average_size"] = 0.0

    # Optional: put TOTAL first
    if "TOTAL" in df["label"].values:
        total_row = df[df["label"] == "TOTAL"]
        other_rows = df[df["label"] != "TOTAL"]
        df = pd.concat([total_row, other_rows], ignore_index=True)

    return df.set_index("label")


def label_summary_table(rows: List[dict]) -> pd.DataFrame:
    """
    rows: output of table_from_labeled_tree(root)

    Returns a DataFrame with:
      - index = label (including 'TOTAL')
      - columns: bytes, pct_of_total
    """
    df = pd.DataFrame(rows)

    # TOTAL row (should be unique)
    total = df.loc[df["label"] == "TOTAL", "node_agg_bytes"].iloc[0]

    # Aggregate bytes per label, including TOTAL
    df_labels = (
        df.groupby("label", as_index=False)["node_agg_bytes"]
            .sum()
            .rename(columns={"node_agg_bytes": "bytes"})
    )

    # Percentage of total; TOTAL will naturally be 100%
    df_labels["pct"] = df_labels["bytes"] / total * 100.0

    # Label as index if you like
    df_labels = df_labels.set_index("label")

    return df_labels


def parse_dhat(dhat_path: str) -> pd.DataFrame:
    with open(dhat_path, "r") as f:
        dhat_data = json.load(f)

    ftbl = dhat_data["ftbl"]
    root = build_tree(dhat_data)
    label_nodes_by_patterns(root, ftbl, patterns)
    filter_tree_by_labels(root, {
        "element_clone_from_window_closed",
        "vector_clone_from_window_closed"})

    return dataframe_from_labeled_tree(root)

if __name__ == "__main__":
    parse_dhat("heap.json")