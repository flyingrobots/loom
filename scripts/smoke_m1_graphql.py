#!/usr/bin/env python3
"""
M1 GraphQL Smoke Script

This script is intended to be a mechanically runnable version of the "M1 proof hooks"
documented in `docs/ROADMAP/README.md`.

It does NOT start `jitosd`. It assumes `jitosd` is already running and serving GraphQL.

Usage:
  python3 scripts/smoke_m1_graphql.py
  python3 scripts/smoke_m1_graphql.py --url http://127.0.0.1:8080/graphql
  python3 scripts/smoke_m1_graphql.py --verbose

Exit codes:
  0 = PASS
  1 = FAIL (assertion failed)
  2 = FAIL (transport / GraphQL error)
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Any, Dict, Optional, Tuple


DEFAULT_URL = "http://127.0.0.1:8080/graphql"


class SmokeTransportError(RuntimeError):
    pass


class SmokeGraphQLError(RuntimeError):
    pass


@dataclass(frozen=True)
class GraphQLResponse:
    data: Dict[str, Any]
    raw: Dict[str, Any]


def _post_json(url: str, payload: Dict[str, Any], timeout_s: float) -> Dict[str, Any]:
    body = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        url=url,
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout_s) as resp:
            text = resp.read().decode("utf-8")
    except urllib.error.HTTPError as e:
        raise SmokeTransportError(f"HTTP error: {e.code} {e.reason}") from e
    except urllib.error.URLError as e:
        raise SmokeTransportError(f"URL error: {e.reason}") from e

    try:
        return json.loads(text)
    except json.JSONDecodeError as e:
        raise SmokeTransportError(f"invalid JSON response: {e}") from e


def gql(url: str, query: str, variables: Optional[Dict[str, Any]], timeout_s: float) -> GraphQLResponse:
    payload: Dict[str, Any] = {"query": query}
    if variables is not None:
        payload["variables"] = variables

    raw = _post_json(url, payload, timeout_s=timeout_s)
    if "errors" in raw and raw["errors"]:
        raise SmokeGraphQLError(json.dumps(raw["errors"], indent=2, sort_keys=True))
    data = raw.get("data")
    if not isinstance(data, dict):
        raise SmokeGraphQLError(f"missing/invalid GraphQL data field: {raw}")
    return GraphQLResponse(data=data, raw=raw)


def _expect_hex64(s: Any, label: str) -> str:
    if not isinstance(s, str):
        raise AssertionError(f"{label}: expected string, got {type(s).__name__}")
    if len(s) != 64:
        raise AssertionError(f"{label}: expected 64-char hex string, got len={len(s)} ({s!r})")
    for ch in s:
        if ch not in "0123456789abcdef":
            raise AssertionError(f"{label}: expected lowercase hex, got {s!r}")
    return s


def _get_digest(resp: GraphQLResponse) -> str:
    graph = resp.data.get("graph")
    if not isinstance(graph, dict):
        raise AssertionError(f"graph: expected object, got {type(graph).__name__}")
    return _expect_hex64(graph.get("digest"), "graph.digest")


def run_smoke(url: str, timeout_s: float, verbose: bool) -> None:
    q_system_digest = """
      query {
        graph(view: { kind: SYSTEM }) { digest }
      }
    """

    m_create_sws = """
      mutation {
        createSws { sws { id } }
      }
    """

    m_apply_add_node = """
      mutation {
        applyRewrite(
          view: { kind: SWS, swsId: "0" }
          rewrite: {
            ops: [{ op: "AddNode", data: { kind: "demo", payload_b64: "aGVsbG8=" } }]
          }
        ) {
          accepted
          receipt { rewriteIdx viewDigest }
        }
      }
    """

    q_sws_digest = """
      query {
        graph(view: { kind: SWS, swsId: "0" }) { digest }
      }
    """

    q_system_digest_again = q_system_digest

    q_rewrites = """
      query {
        rewrites(view: { kind: SWS, swsId: "0" }, page: { first: 100 }) { idx }
      }
    """

    # 1) System digest (H0)
    r0 = gql(url, q_system_digest, variables=None, timeout_s=timeout_s)
    if verbose:
        print("system digest response:", json.dumps(r0.raw, indent=2, sort_keys=True))
    h0 = _get_digest(r0)
    print(f"PASS: system digest (H0) = {h0}")

    # 2) createSws expects deterministic "0" for first allocation after boot
    r1 = gql(url, m_create_sws, variables=None, timeout_s=timeout_s)
    if verbose:
        print("createSws response:", json.dumps(r1.raw, indent=2, sort_keys=True))
    create_sws = r1.data.get("createSws")
    if not isinstance(create_sws, dict) or not isinstance(create_sws.get("sws"), dict):
        raise AssertionError("createSws.sws: missing/invalid")
    sws_id = create_sws["sws"].get("id")
    if sws_id != "0":
        raise AssertionError(f"createSws.sws.id: expected '0', got {sws_id!r}")
    print("PASS: createSws returned swsId=0")

    # 3) apply AddNode into SWS 0
    r2 = gql(url, m_apply_add_node, variables=None, timeout_s=timeout_s)
    if verbose:
        print("applyRewrite response:", json.dumps(r2.raw, indent=2, sort_keys=True))
    ar = r2.data.get("applyRewrite")
    if not isinstance(ar, dict):
        raise AssertionError("applyRewrite: missing/invalid")

    accepted = ar.get("accepted")
    if accepted is not True:
        raise AssertionError(f"applyRewrite.accepted: expected true, got {accepted!r}")

    receipt = ar.get("receipt")
    if not isinstance(receipt, dict):
        raise AssertionError("applyRewrite.receipt: missing/invalid")

    # M1 contract: deterministic receipt fields (rewriteIdx, viewDigest)
    rewrite_idx = receipt.get("rewriteIdx")
    view_digest = receipt.get("viewDigest")
    if not isinstance(rewrite_idx, int):
        # GraphQL implementations sometimes serialize custom scalars as strings.
        if isinstance(rewrite_idx, str) and rewrite_idx.isdigit():
            rewrite_idx = int(rewrite_idx)
        else:
            raise AssertionError(f"receipt.rewriteIdx: expected int (or digit string), got {rewrite_idx!r}")
    _expect_hex64(view_digest, "receipt.viewDigest")

    if rewrite_idx != 0:
        raise AssertionError(f"receipt.rewriteIdx: expected 0 for first rewrite, got {rewrite_idx}")
    print("PASS: applyRewrite accepted + receipt is well-formed (rewriteIdx=0)")

    # 4) sws digest changes; system digest unchanged
    r3 = gql(url, q_sws_digest, variables=None, timeout_s=timeout_s)
    if verbose:
        print("sws digest response:", json.dumps(r3.raw, indent=2, sort_keys=True))
    h_sws = _get_digest(r3)
    if h_sws == h0:
        raise AssertionError("expected SWS digest to differ from system digest after AddNode")
    print(f"PASS: sws digest != H0 ({h_sws} != {h0})")

    r4 = gql(url, q_system_digest_again, variables=None, timeout_s=timeout_s)
    if verbose:
        print("system digest (again) response:", json.dumps(r4.raw, indent=2, sort_keys=True))
    h0_again = _get_digest(r4)
    if h0_again != h0:
        raise AssertionError("expected system digest to remain unchanged (system is immutable in M1)")
    print("PASS: system digest unchanged after SWS rewrite")

    # 5) rewrite log ordering (idx ascending) and contains idx=0
    r5 = gql(url, q_rewrites, variables=None, timeout_s=timeout_s)
    if verbose:
        print("rewrites response:", json.dumps(r5.raw, indent=2, sort_keys=True))

    events = r5.data.get("rewrites")
    if not isinstance(events, list):
        raise AssertionError("rewrites: expected list")

    idxs = []
    for ev in events:
        if not isinstance(ev, dict) or "idx" not in ev:
            raise AssertionError("rewrites: expected objects with idx")
        idx = ev["idx"]
        if isinstance(idx, str) and idx.isdigit():
            idx = int(idx)
        if not isinstance(idx, int):
            raise AssertionError(f"rewrites.idx: expected int (or digit string), got {idx!r}")
        idxs.append(idx)

    if idxs != sorted(idxs):
        raise AssertionError(f"rewrites: expected idx ascending, got {idxs}")
    if 0 not in idxs:
        raise AssertionError(f"rewrites: expected idx list to include 0, got {idxs}")
    print("PASS: rewrites are ordered and include idx=0")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", default=DEFAULT_URL, help=f"GraphQL URL (default: {DEFAULT_URL})")
    ap.add_argument("--timeout", type=float, default=5.0, help="HTTP timeout in seconds (default: 5)")
    ap.add_argument("--verbose", action="store_true", help="Print full GraphQL responses")
    args = ap.parse_args()

    try:
        run_smoke(args.url, timeout_s=args.timeout, verbose=args.verbose)
    except AssertionError as e:
        print(f"FAIL: {e}", file=sys.stderr)
        return 1
    except (SmokeTransportError, SmokeGraphQLError) as e:
        print(f"FAIL: {e}", file=sys.stderr)
        return 2
    except Exception as e:  # defensive
        print(f"FAIL: unexpected error: {e}", file=sys.stderr)
        return 2

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

