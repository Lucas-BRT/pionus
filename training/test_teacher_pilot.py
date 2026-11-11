"""Checks `teacher_pilot` on a few cases made up for it.

    python -I test_teacher_pilot.py
"""

import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import audit_types  # noqa: E402
from teacher_pilot import build, case_id, merge  # noqa: E402


def make_set(root: Path, n: int) -> Path:
    set_dir = root / "set"
    set_dir.mkdir()
    (set_dir / "inputs.jsonl").write_text("".join(json.dumps({
        "subjects": ["fix: older"], "staged": [],
        "files": [{"path": f"f{k}.rs", "lines": [1, 0], "hunks": "@@ -0,0 +1 @@\n+" + "x" * 3000}],
    }) + "\n" for k in range(n)))
    (set_dir / "meta.jsonl").write_text("".join(
        json.dumps({"repo": "o/secret", "hash": f"h{k}", "target": f"fix: AUTHOR{k}"}) + "\n" for k in range(n)))
    return set_dir


def test_the_teacher_sees_the_whole_diff_and_never_the_author() -> None:
    saved = audit_types.FILE_CHARS, audit_types.DIFF_CHARS
    with tempfile.TemporaryDirectory() as d:
        root = Path(d)
        build(make_set(root, 5), root / "out", 2)
        text = "".join(p.read_text() for p in (root / "out").glob("T-*.md"))
        assert "x" * 3000 in text                       # a 3,000 character file is not cut at the audit's 1,200
        assert "secret" not in text and "AUTHOR" not in text
        assert (audit_types.FILE_CHARS, audit_types.DIFF_CHARS) == saved   # the audit's limits are back
        ids = [i for p in sorted((root / "out").glob("T-*.ids")) for i in p.read_text().split()]
        assert sorted(ids) == [case_id(k) for k in range(5)]


def test_merge_checks_the_answers_and_keeps_the_order_of_the_set() -> None:
    with tempfile.TemporaryDirectory() as d:
        root = Path(d)
        build(make_set(root, 4), root / "out", 2)
        out = root / "out"
        answers = {1: [("t001", "fix: b"), ("t003", "fix: d")], 2: [("t000", "fix: a"), ("t002", "fix: c")]}
        for number, rows in answers.items():
            (out / f"T-{number}.json").write_text(json.dumps([{"id": i, "subject": s} for i, s in rows]))
        merge(out)
        written = [json.loads(line)["generation"]["message"]["subject"] for line in open(out / "teacher.jsonl")]
        assert written == ["fix: a", "fix: b", "fix: c", "fix: d"]
        (out / "T-2.json").write_text(json.dumps([{"id": "t000", "subject": "x" * 73}, {"id": "t002", "subject": "c"}]))
        try:
            merge(out)
        except SystemExit as stop:
            assert "73 columns" in str(stop)
        else:
            raise AssertionError("a subject over 72 columns was accepted")
        (out / "T-2.json").write_text(json.dumps(["fix: c"]))
        try:
            merge(out)
        except SystemExit as stop:
            assert "not an object" in str(stop)
        else:
            raise AssertionError("an answer that is not an object was accepted")


if __name__ == "__main__":
    for name, test in sorted(globals().items()):
        if name.startswith("test_"):
            test()
            print(f"ok {name}")
