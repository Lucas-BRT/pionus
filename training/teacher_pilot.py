"""A stronger model writes the subject of each commit of a set, blind to the author's (H19, stage 1).

- `build` writes batches of cases for the teacher to read: the repository's recent subjects, the changed files and
  the whole diff (`DIFF_CHARS`, where the product's prompt keeps what fits 1024 tokens), never the author's subject or
  the repository's name.
- `merge` reads the teacher's answers (`T-1.json`...: a list of `id`, `subject`), checks them and writes
  `teacher.jsonl`, one trace-shaped line per commit of the set in order, so that `pair_judge.py build - SET OUT
  teacher=teacher.jsonl` judges the teacher against the authors.

    python -I teacher_pilot.py build SET_DIR OUT_DIR [--batches 10]
    python -I teacher_pilot.py merge OUT_DIR

Standard library only.
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import audit_types  # noqa: E402

DIFF_CHARS = 20000
MAX_COLUMNS = 72


def case_id(line: int) -> str:
    return f"t{line:03d}"


def build(set_dir: Path, out_dir: Path, batches: int) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    with open(set_dir / "inputs.jsonl") as f:
        inputs = [json.loads(line) for line in f]
    # `render_case` reads these module globals: widen them only while it runs.
    saved = audit_types.FILE_CHARS, audit_types.DIFF_CHARS
    audit_types.FILE_CHARS = audit_types.DIFF_CHARS = DIFF_CHARS
    try:
        for number in range(batches):
            lines = list(range(number, len(inputs), batches))
            text = "\n\n".join(audit_types.render_case(case_id(n), inputs[n]) for n in lines)
            (out_dir / f"T-{number + 1}.md").write_text(text + "\n")
            (out_dir / f"T-{number + 1}.ids").write_text("\n".join(case_id(n) for n in lines) + "\n")
    finally:
        audit_types.FILE_CHARS, audit_types.DIFF_CHARS = saved
    (out_dir / "count").write_text(str(len(inputs)))


def merge(out_dir: Path) -> None:
    total = int((out_dir / "count").read_text())
    subjects: dict[str, str] = {}
    for path in sorted(out_dir.glob("T-*.json")):
        for answer in json.loads(path.read_text()):
            if not isinstance(answer, dict):
                sys.exit(f"{path.name}: an answer is not an object with an id and a subject: {answer!r:.60}")
            subject = answer.get("subject")
            if not isinstance(subject, str) or not subject.strip() or "\n" in subject:
                sys.exit(f"{path.name}: {answer.get('id')}: the subject is not one line")
            if len(subject) > MAX_COLUMNS:
                sys.exit(f"{path.name}: {answer.get('id')}: {len(subject)} columns, more than {MAX_COLUMNS}")
            if answer.get("id") in subjects:
                sys.exit(f"{answer['id']} written twice")
            subjects[answer["id"]] = subject.strip()
    missing = [case_id(n) for n in range(total) if case_id(n) not in subjects]
    unknown = set(subjects) - {case_id(n) for n in range(total)}
    if missing or unknown:
        sys.exit(f"{len(missing)} commits without a subject, e.g. {missing[:3]}; unknown ids {sorted(unknown)[:3]}")
    with open(out_dir / "teacher.jsonl", "w") as out:
        for n in range(total):
            out.write(json.dumps({"generation": {"message": {"subject": subjects[case_id(n)]}}}) + "\n")
    print(f"{total} subjects in {out_dir / 'teacher.jsonl'}")


def main() -> None:
    args = sys.argv[1:]
    if args[:1] == ["build"]:
        build(Path(args[1]), Path(args[2]), int(args[args.index("--batches") + 1]) if "--batches" in args else 10)
    elif args[:1] == ["merge"]:
        merge(Path(args[1]))
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
