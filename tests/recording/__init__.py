"""Recording tests and pure JSONL artifact encoding/decoding; no controls or ticks."""
import json


def write_jsonl(path, rows):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                            for row in rows), encoding="utf-8")


def read_jsonl(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise AssertionError(f"Duplicate recording field {key}: {path}")
            result[key] = value
        return result

    return [json.loads(line, object_pairs_hook=unique) for line in path.read_text(encoding="utf-8").splitlines()]
