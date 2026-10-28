"""External gh-only fault tool. Literal REST fixtures, no network or real-gh fallback."""
import json
import os
from pathlib import Path
import sys
import time


def main():
    root = Path(os.environ["WOODPECKER_TEST_GH_ROOT"])
    config = json.loads((root / "mode.json").read_text())
    arguments = sys.argv[1:]
    search = arguments == ["api", "repos/{owner}/{repo}/issues?state=all&per_page=100", "--method", "GET", "--paginate"]
    publish = arguments == ["api", "repos/{owner}/{repo}/issues", "--method", "POST", "--input", "-"]
    if not (search or publish):
        raise RuntimeError("Unexpected gh arguments; refusing any real command")
    body = None if search else json.loads(sys.stdin.read())
    with (root / "calls.jsonl").open("a") as journal:
        journal.write(json.dumps({"pid": os.getpid(), "cwd": os.getcwd(), "arguments": arguments,
                                  "body": body, "prompt": os.environ.get("GH_PROMPT_DISABLED")}) + "\n")
    mode = config["mode"]
    marker = "<!-- bug_hunter-signature: " + config.get("signature", "") + " -->"
    issue = lambda number, content, **extra: {"number": number, "html_url": f"https://example.invalid/issues/{number}",
                                             "body": content, **extra}
    if mode == "hold" and search:
        # Existence is the gate: publish only the complete, closed JSON document.
        pending = root / f"entered-{os.getpid()}.next"
        pending.write_text(json.dumps({"pid": os.getpid()}) + "\n")
        pending.replace(root / "entered.json")
        deadline = time.monotonic() + 60
        while not (root / "release").exists():
            if time.monotonic() >= deadline:
                raise TimeoutError("Parent never released owned gh")
            time.sleep(0.02)
    if search:
        if mode in ("search-failed", "both-failed"):
            print("controlled search failure", file=sys.stderr)
            return 7
        if mode == "search-invalid":
            print("not JSON")
        elif mode == "late-invalid":
            print(json.dumps([issue(42, marker)]))
            print("broken last page")
        elif mode in ("duplicate-open", "duplicate-closed"):
            print(json.dumps([issue(3, marker, pull_request={}),
                              issue(4, "```text\n" + marker + "\n```"),
                              issue(5, marker + " trailing")]))
            print(json.dumps([issue(42, marker, state=mode.removeprefix("duplicate-"))]))
        else:
            print("[]\n[]")
    elif mode == "publish-failed":
        print("controlled publish failure", file=sys.stderr)
        return 8
    elif mode == "publish-invalid":
        print('{"number":0,"html_url":"not-an-issue"}')
    else:
        print('{"number":12,"html_url":"https://example.invalid/issues/12"}')
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
