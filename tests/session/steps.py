"""Individual outcomes for checks sharing a process and its state."""
import json
import time
import traceback


class Steps:
    def __init__(self, path):
        self.path = path
        self.results = []

    def run(self, name, check, requires=(), precondition=None):
        previous = {r["name"]: r for r in self.results}
        if name in previous or set(requires) - previous.keys():
            raise ValueError(f"Doppelter Schritt oder unbekannte Abhängigkeit: {name}, {requires}")
        result = {"name": name, "requires": list(requires), "status": "running"}
        self.results.append(result)
        blocked = [name for name in requires if previous[name]["status"] != "passed"]
        started = time.monotonic()
        value = None
        if blocked:
            result.update(status="blocked", blocked_by=blocked)
        else:
            self.save()
            if precondition is not None:
                try:
                    precondition()
                except Exception as error:
                    result.update(status="blocked", error=f"Ausgangszustand ungültig: {error}",
                                  traceback=traceback.format_exc())
        if result["status"] == "running":
            try:
                value = check()
                # Evidence must survive a JSON report, not just the current process.
                json.dumps(value)
                result.update(status="passed", evidence=value)
            except Exception as error:
                result.update(status="failed", error=str(error), traceback=traceback.format_exc())
        result["seconds"] = round(time.monotonic() - started, 3)
        self.save()
        print(f"  {result['status'].upper():9} {name}", flush=True)
        return value if result["status"] == "passed" else None

    def save(self):
        self.path.write_text(json.dumps(self.results, indent=2, ensure_ascii=False) + "\n")

    def finish(self):
        failed = [r["name"] for r in self.results if r["status"] != "passed"]
        if failed:
            raise AssertionError(f"Prüfschritte nicht bestanden: {', '.join(failed)}; siehe {self.path}")
