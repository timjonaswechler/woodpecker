"""Real image EngineReady and RGB capture, without a native window or hidden ticks."""
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, load_config, owned_process, start_server, write_app_config, wait_process_gone
from tests.pointer import layout_target, read_components
from tests.text import SETTINGS, TEXT_INPUT
from tests.headless._windows import owned_windows


class HeadlessBasisTest(unittest.TestCase):
    def test_signalstation_image_ready_capture_and_pause_close_cleanly(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Explicitly authorized real Metal/GPU execution required")
        from PIL import Image
        server = start_server(self, "headless-basis")
        configuration = load_config(ROOT / "tests/headless/app.toml")
        configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-process-probe", "--woodpecker-legacy-ui", "--woodpecker-clock-snapshot"]
        path = server.artifacts / "headless.toml"
        write_app_config(path, configuration)
        session_id = server.cli("session", "create", "--config", str(path))["id"]
        deadline = time.monotonic() + 90
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "EngineReady":
                break
            self.assertIn(detail["state"], ("Starting", "SessionReady"), detail)
            if time.monotonic() >= deadline:
                self.fail(f"Image EngineReady deadline; see {server.artifacts}")
            time.sleep(0.05)
        root = Path(detail["artifact_dir"])
        identity = owned_process(server, root)
        self.assertEqual(owned_windows(identity["pid"]), [], "The real app must own no native windows")
        server.record(["observation", "owned-native-windows"], pid=identity["pid"], windows=[])
        # Time<Real> contains nonserializable Instants. The read-only diagnostic
        # samples its real scalars/update timestamps on every extraction, also idle.
        types = ("test_app::woodpecker::SimulationTicks", "bevy_time::time::Time<()>",
                 "test_app::diagnostics::clock::RealSnapshot",
                 "bevy_time::time::Time<bevy_time::virt::Virtual>",
                 "bevy_time::time::Time<bevy_time::fixed::Fixed>")
        before = {name: server.read_resource(session_id, name) for name in types}
        self.assertGreater(before[types[0]]["ticks"], 0)
        first = server.call(session_id, {"command": "screenshot.capture", "arguments": {"path": "headless/a.png"}}, timeout=35)
        time.sleep(0.25)
        second = server.call(session_id, {"command": "screenshot.capture", "arguments": {"path": "headless/paused.png"}}, timeout=35)
        self.assertEqual({name: server.read_resource(session_id, name) for name in types}, before)
        self.assertEqual((root / "headless/a.png").read_bytes(), (root / "headless/paused.png").read_bytes())
        with Image.open(root / "headless/a.png") as image:
            self.assertEqual(image.mode, "RGB")
            self.assertEqual(image.size, (1440, 900))
            self.assertGreater(len(set(image.getdata())), 100)
        server.record(["observation", "image-ready-paused-capture"], first=first, second=second, clocks=before)

        def tick():
            self.assertEqual(server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}}),
                             {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})

        button = "bevy_ui_widgets::button::Button"
        legacy = "bevy_ui::focus::DeprecatedInteraction"
        target = layout_target(server, session_id, button, label="Gesamtkarte [G]")
        for command, arguments in (("input.pointer.move_to", {"position": target["position"]}),
                                   ("input.pointer.press", {"button": "left"})):
            server.call(session_id, {"command": command, "arguments": arguments})
            tick()
        interaction, = [row for row in read_components(server, session_id, [button, legacy], [legacy])
                        if row["entity"] == target["entity"]]
        self.assertEqual(interaction["values"][legacy], "Pressed", "Legacy and widget UI must both see the real image pointer")
        self.assertEqual(server.read_resource(session_id, "test_app::player::camera::ViewState")["mode"], "Map")
        server.call(session_id, {"command": "screenshot.capture", "arguments": {"path": "headless/map.png"}}, timeout=35)
        self.assertNotEqual((root / "headless/a.png").read_bytes(), (root / "headless/map.png").read_bytes())
        server.call(session_id, {"command": "input.pointer.release", "arguments": {"button": "left"}})
        tick()
        server.record(["observation", "actual-ui-map"], target=target, action_ticks=3)
        # Additional explicit 6 focus ticks + 2 edit/mirror ticks, not repair ticks.
        for marker, selector in ((button, {"label": "Einstellungen [M]"}), (TEXT_INPUT, {"name": "Notizen"})):
            notes = layout_target(server, session_id, marker, **selector)
            for command, arguments in (("input.pointer.move_to", {"position": notes["position"]}),
                                       ("input.pointer.press", {"button": "left"}),
                                       ("input.pointer.release", {"button": "left"})):
                server.call(session_id, {"command": command, "arguments": arguments})
                tick()
        seed = server.read_resource(session_id, SETTINGS)
        server.call(session_id, {"command": "screenshot.capture", "arguments": {"path": "headless/notes-before.png"}}, timeout=35)
        text = "Grüße 🦉\nCafe\u0301 – 東京"
        server.call(session_id, {"command": "input.text.input", "arguments": {"text": text}})
        self.assertEqual(server.read_resource(session_id, SETTINGS), seed)
        tick()
        tick()
        self.assertEqual(server.read_resource(session_id, SETTINGS), {**seed, "notes": text})
        server.call(session_id, {"command": "screenshot.capture", "arguments": {"path": "headless/notes.png"}}, timeout=35)
        node = "bevy_ui::ui_node::ComputedNode"
        transform = "bevy_ui::ui_transform::UiGlobalTransform"
        layout, = [row for row in read_components(server, session_id, [TEXT_INPUT, node, transform], [node, transform])
                   if row["entity"] == notes["entity"]]
        width, height = layout["values"][node]["size"]
        cx, cy = layout["values"][transform][4:6]
        box = tuple(round(value) for value in (cx-width/2+3, cy-height/2+3, cx+width/2-3, cy+height/2-3))
        with Image.open(root / "headless/notes-before.png") as a, Image.open(root / "headless/notes.png") as b:
            changed = sum(x != y for x, y in zip(a.crop(box).getdata(), b.crop(box).getdata()))
        self.assertGreater(changed, 50, "Real text glyphs must change the notes field's image pixels")
        server.record(["observation", "actual-unicode-glyphs"], text=text, box=box, changed_pixels=changed, action_ticks=8)
        metrics = server.read_resource(session_id, "test_app::performance::Metrics")
        self.assertGreater(metrics["rendered_frames"], 0, "The render diagnostics must count actual image frames")
        server.call(session_id, {"command": "input.keyboard.press", "arguments": {"key": "f8"}})
        tick()
        self.assertTrue(server.read_resource(session_id, "test_app::performance::Metrics")["overlay"])
        server.call(session_id, {"command": "screenshot.capture", "arguments": {"path": "headless/overlay.png"}}, timeout=35)
        with Image.open(root / "headless/notes.png") as a, Image.open(root / "headless/overlay.png") as b:
            overlay_changed = sum(x != y for x, y in zip(a.crop((856, 658, 1416, 790)).getdata(), b.crop((856, 658, 1416, 790)).getdata()))
        self.assertGreater(overlay_changed, 1000, "The existing GPU overlay must draw on the image target")
        server.call(session_id, {"command": "input.keyboard.release", "arguments": {"key": "f8"}})
        tick()
        self.assertEqual(server.read_resource(session_id, types[0])["ticks"], before[types[0]]["ticks"] + 13)
        real_after = server.read_resource(session_id, "test_app::diagnostics::clock::RealSnapshot")
        self.assertGreater(real_after["elapsed"], before[types[2]]["elapsed"], "The Real snapshot must be live, not a constant/frozen probe")
        server.record(["observation", "real-clock-live-after-counted-ticks"], before=before[types[2]], after=real_after)
        server.record(["observation", "image-overlay"], changed_pixels=overlay_changed, action_ticks=2)
        self.assertEqual(owned_windows(identity["pid"]), [])
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        wait_process_gone(identity)
