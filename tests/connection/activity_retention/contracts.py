"""Large real results, delayed polling and a client that stops reading."""
import json
import socket

from tests.catalog import scenario
from tests.interfaces.connection import Connection, snapshot
from tests.recording.contracts import read_recording
from tests.session.runtime import until

MIB = 1024 * 1024


def query(name):
    return {"source": "resources", "selector": {"kind": "type", "type_path": f"activity_fixture::{name}"},
            "projection": {"kind": "value"}}


def poll(session, cursor, gap=False):
    reply = session.server.cli("session", "poll", session.id, "--cursor", json.dumps(cursor),
                               "--wait-ms", "0", success=not gap)
    assert reply["kind"] == ("gap" if gap else "activity"), reply
    if gap:
        assert "unknown" in reply["message"], reply
    return reply


@scenario("connection", "activity_gaps_never_invent_success", capabilities=(),
          programs=("activity",), independent=True)
def activity_gaps_never_invent_success(ctx):
    with ctx.session(ctx.program("activity", tracing=True)) as driver:
        session = driver.session
        root = session.artifacts
        counter = driver.command("inspect.query", query("Counter"))["items"][0]["result"]["value"]["value"]
        session.pids.add(counter["process_id"])
        assert counter["ticks"] == 0
        driver.command("recording.start", {"path": "large.jsonl"})
        old = snapshot(session)["cursor"]
        oversized = driver.submit("inspect.query", query("Large"))
        until(lambda: not snapshot(session)["pending"], "Großes Inspect blieb Pending")
        recovered = poll(session, old, gap=True)["cursor"]
        remaining = poll(session, recovered)
        assert not any(e["event"].get("request_id") == oversized for e in remaining["entries"])
        # Continue at the visible gap, without assigning any result to the lost request.
        session.cursor = remaining["cursor"]
        driver.command("recording.stop", {})
        rows = read_recording(root / "large.jsonl")
        text = rows[1]["outcome"]["output"]["items"][0]["result"]["value"]["value"]["text"]
        assert len(text) == 5 * MIB and set(text) == {"x"}
        assert rows[-1]["recorded_commands"] == 1

        lagged = snapshot(session)["cursor"]
        requests = []
        for _ in range(3):
            request = driver.submit("inspect.query", query("Medium"))
            output = driver.outcome(request, "inspect.query")
            assert output["kind"] == "completed", output
            text = output["output"]["items"][0]["result"]["value"]["value"]["text"]
            assert len(text) == 2 * MIB and set(text) == {"m"}
            requests.append(request)
        recovered = poll(session, lagged, gap=True)["cursor"]
        retained = poll(session, recovered)
        events = [e["event"] for e in retained["entries"]]
        assert not any(e.get("request_id") == requests[0] for e in events)
        assert any(e.get("request_id") == requests[-1] and e["kind"] == "completed" for e in events)

        with Connection(session) as blocked:
            blocked.send({"kind": "poll", "cursor": recovered, "wait_ms": 0})
            assert blocked.socket.recv(1, socket.MSG_PEEK), "Server hat die große Antwort nicht begonnen"
            assert driver.command("inspect.query", query("Counter"))["items"][0]["result"]["value"]["value"]["ticks"] == 0

        before_report = snapshot(session)["cursor"]
        request = driver.submit("tick.warp.start", {"ticks": 1})
        until(lambda: not snapshot(session)["pending"], "Report-Tick blieb Pending")
        path = until(lambda: next((root / "reports").glob("*.md"), None), "Großer Report fehlt")
        markdown = path.read_text()
        assert len(markdown.encode()) > 4 * MIB and "x" * (5 * MIB) in markdown
        # Report persistence can precede its Activity publication.
        def report_gap():
            with Connection(session) as connection:
                result = connection.call({"kind": "poll", "cursor": before_report, "wait_ms": 0})
                assert result["kind"] in ("gap", "activity"), result
                return result if result["kind"] == "gap" else None
        gap = until(report_gap, "Großer Report wurde nicht als Activity-Lücke sichtbar")
        assert "unknown" in gap["message"]
        retained = poll(session, gap["cursor"])
        assert not any(e["event"]["kind"] == "report" for e in retained["entries"])
        session.cursor = retained["cursor"]
        assert snapshot(session)["state"] == "SessionReady"
        assert driver.command("inspect.query", query("Counter"))["items"][0]["result"]["value"]["value"]["ticks"] == 1
        driver.close()
        session.stop()
