"""Run original startup owners in the explicit image profile, with unchanged assertions."""
import unittest
from unittest.mock import patch

from tests import _support
from tests.startup import engine_startup, manual_startup, startup_failures


class HeadlessStartupTest(unittest.TestCase):
    def test_original_startup_owners_keep_their_readiness_failure_and_tick_contracts(self):
        original = _support.load_config
        def headless_config(*args, **kwargs):
            configuration = original(*args, **kwargs)
            configuration["launch"]["headless"] = {"width": 1440, "height": 900, "scale_factor": 1.0}
            configuration["launch"]["no_default_features"] = True
            return configuration
        # Only the fixture data changes; no product behavior, events, budgets,
        # original assertions or commands are mocked or reimplemented here.
        with patch.object(_support, "load_config", headless_config):
            manual_startup.ManualStartupTest.test_manual_startup_stays_at_zero_ticks_and_time(self)
        with patch.object(engine_startup, "load_config", headless_config):
            engine_startup.EngineStartupTest.test_engine_startup_reports_ticks_and_then_stays_idle(self)
        with patch.object(startup_failures, "load_config", headless_config):
            startup_failures.StartupFailuresTest.test_startup_prerequisites_and_budgets_fail_with_their_own_diagnostics(self)
