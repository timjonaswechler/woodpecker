"""Commands cross the public CLI JSON input and the real network/session path."""
import json


def check_result(event, expected_error=None):
    if expected_error is not None:
        if event["kind"] != "rejected" or event["error"]["code"] != expected_error:
            raise AssertionError(event)
        return event["error"]
    if event["kind"] != "completed":
        raise AssertionError(event)
    return event["output"]


class Json:
    def __init__(self, session):
        self.session = session

    def submit(self, name, arguments):
        reply = self.session.server.cli(
            "session", "submit", self.session.id, "--command",
            json.dumps({"command": name, "arguments": arguments}, ensure_ascii=False),
        )
        if reply["kind"] != "pending" or reply["command"] != name:
            raise AssertionError(reply)
        return reply["request_id"]

    def outcome(self, request, name):
        return self.session.outcome(request, name)

    def command(self, name, arguments, expected_error=None):
        return check_result(self.outcome(self.submit(name, arguments), name), expected_error)

    def refuse(self, name, arguments, code):
        reply = self.session.server.cli("session", "submit", self.session.id, "--command",
            json.dumps({"command": name, "arguments": arguments}), success=False)
        assert reply["kind"] == "error" and reply["code"] == code, reply
        return reply

    def close(self):
        pass  # Each CLI invocation disconnects on exit.
