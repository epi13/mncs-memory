"""mncs-memory CLI: ingest, query, belief, replay, verify, benchmark."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from memory import codes, engine, model, native, specialists, store


def _mncs(args: argparse.Namespace) -> str:
    found = native.find_mncs(getattr(args, "mncs", None))
    if not found:
        print("mncs-memory: set MNCS_BIN or MNCS_LANGUAGE_ROOT",
              file=sys.stderr)
        raise SystemExit(2)
    return found


def _engine(args: argparse.Namespace) -> engine.MemoryEngine:
    entities_path = getattr(args, "entities", None)
    entities: dict[str, Any] = {}
    if entities_path:
        try:
            entities = json.loads(Path(entities_path).read_text(
                encoding="utf-8"))
        except (OSError, ValueError) as error:
            print(f"mncs-memory: cannot read {entities_path}: {error}",
                  file=sys.stderr)
            raise SystemExit(2)
    return engine.MemoryEngine(_mncs(args), args.state_dir, entities)


def cmd_ingest(args: argparse.Namespace) -> int:
    memory = _engine(args)
    try:
        document = json.loads(Path(args.file).read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        print(f"mncs-memory: cannot read {args.file}: {error}",
              file=sys.stderr)
        return 2
    observations = document if isinstance(document, list) else [document]
    try:
        for observation in observations:
            record = memory.ingest(observation)
            print(json.dumps(record))
    except (model.MemoryError, engine.EngineError,
            store.StoreError) as error:
        print(f"mncs-memory: ingest refused: {error}", file=sys.stderr)
        return 1
    return 0


def cmd_query(args: argparse.Namespace) -> int:
    memory = _engine(args)
    try:
        document = json.loads(Path(args.file).read_text(encoding="utf-8"))
        capsule = memory.query(document)
    except (OSError, ValueError, model.MemoryError,
            engine.EngineError) as error:
        print(f"mncs-memory: query failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps(capsule, indent=2))
    return 0


def cmd_belief(args: argparse.Namespace) -> int:
    memory = _engine(args)
    try:
        print(json.dumps(memory.current_belief(args.subject, args.topic),
                         indent=2))
    except engine.EngineError as error:
        print(f"mncs-memory: belief failed: {error}", file=sys.stderr)
        return 1
    return 0


def cmd_replay(args: argparse.Namespace) -> int:
    memory = _engine(args)
    try:
        entities = json.loads(Path(args.entities).read_text(
            encoding="utf-8")) if args.entities else {}
        document = json.loads(Path(args.file).read_text(encoding="utf-8"))
        observations = document if isinstance(document, list) else [
            document]
        resolver = specialists.EntityResolver(
            entities, faulty_alias=args.faulty_alias,
            faulty_target=args.faulty_target,
            version=args.processor_version)
        print(json.dumps(memory.replay(resolver, observations), indent=2))
    except (OSError, ValueError, model.MemoryError,
            engine.EngineError) as error:
        print(f"mncs-memory: replay failed: {error}", file=sys.stderr)
        return 1
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    try:
        keeper = store.MemoryStore(args.state_dir)
        keeper.reload()
    except store.StoreError as error:
        print(f"mncs-memory: CORRUPT: {error}", file=sys.stderr)
        return 1
    print(json.dumps({"ok": True, "counts": keeper.counts()}, indent=2))
    return 0


def cmd_benchmark(args: argparse.Namespace) -> int:
    from memory.benchmark import run_corpus
    try:
        corpus = json.loads(Path(args.corpus).read_text(encoding="utf-8"))
        report = run_corpus(_mncs(args), corpus)
    except (OSError, ValueError, model.MemoryError,
            engine.EngineError) as error:
        print(f"mncs-memory: benchmark failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps(report, indent=2))
    comparison = report["aggregate"]
    print(f"memory {comparison['memory_correct']}/"
          f"{report['scenario_count']} correct, "
          f"simple {comparison['simple_retrieval_correct']}/"
          f"{report['scenario_count']}, "
          f"memory irrelevant "
          f"{comparison['memory_irrelevant_fact_inclusion']}, "
          f"simple irrelevant "
          f"{comparison['simple_irrelevant_fact_inclusion']}",
          file=sys.stderr)
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="mncs-memory")
    parser.add_argument("--state-dir", default="state")
    parser.add_argument("--mncs", default=None)
    parser.add_argument("--entities", default=None)
    sub = parser.add_subparsers(dest="command", required=True)

    ingest = sub.add_parser("ingest")
    ingest.add_argument("--file", required=True)
    ingest.set_defaults(func=cmd_ingest)

    query = sub.add_parser("query")
    query.add_argument("--file", required=True)
    query.set_defaults(func=cmd_query)

    belief = sub.add_parser("belief")
    belief.add_argument("--subject", required=True)
    belief.add_argument("--topic", required=True)
    belief.set_defaults(func=cmd_belief)

    replay = sub.add_parser("replay")
    replay.add_argument("--file", required=True)
    replay.add_argument("--processor-version", default="reference-v1")
    replay.add_argument("--faulty-alias", default=None)
    replay.add_argument("--faulty-target", default=None)
    replay.set_defaults(func=cmd_replay)

    verify = sub.add_parser("verify")
    verify.set_defaults(func=cmd_verify)

    benchmark = sub.add_parser("benchmark")
    benchmark.add_argument("--corpus", required=True)
    benchmark.set_defaults(func=cmd_benchmark)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
