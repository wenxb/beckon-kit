"""Resource-guard regression tests; no compilation or memory pressure is generated."""

import contextlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import run_limited


class ResourceGuardTests(unittest.TestCase):
    def run_in_scope(self, overrides=None, exit_code=0):
        values = {
            "memory.max": "8589934592",
            "memory.swap.max": "0",
            "memory.peak": "1048576",
            "memory.events": "oom 0\noom_kill 0\n",
        }
        values.update(overrides or {})
        with tempfile.TemporaryDirectory() as directory:
            group = Path(directory)
            for name, value in values.items():
                (group / name).write_text(value)
            with (
                patch.object(run_limited, "cgroup", return_value=group),
                patch.object(run_limited.Path, "cwd", return_value=group),
                patch.object(run_limited.sys, "argv", ["guard", "--in-scope", "--", "true"]),
                patch.object(run_limited.subprocess, "call", return_value=exit_code) as call,
                contextlib.redirect_stdout(io.StringIO()),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                result = run_limited.main()
                call.assert_called_once_with(["true"], cwd=group)
                return result

    def test_success_and_failure_statuses_are_preserved(self):
        self.assertEqual(self.run_in_scope(), 0)
        self.assertEqual(self.run_in_scope(exit_code=7), 7)

    def test_oom_cannot_be_hidden_by_a_successful_parent_process(self):
        self.assertEqual(self.run_in_scope({"memory.events": "oom 1\noom_kill 1\n"}), 1)

    def test_an_unbounded_or_swapping_scope_never_runs_the_command(self):
        for maximum, swap in [("max", "0"), ("17179869184", "0"), ("8589934592", "max")]:
            with (
                self.subTest(maximum=maximum, swap=swap),
                tempfile.TemporaryDirectory() as directory,
            ):
                group = Path(directory)
                (group / "memory.max").write_text(maximum)
                (group / "memory.swap.max").write_text(swap)
                with (
                    patch.object(run_limited, "cgroup", return_value=group),
                    patch.object(run_limited.sys, "argv", ["guard", "--in-scope", "--", "true"]),
                    patch.object(run_limited.subprocess, "call") as call,
                ):
                    with self.assertRaises(RuntimeError):
                        run_limited.main()
                    call.assert_not_called()

    def test_launcher_caps_resources_and_disables_out_of_scope_wrappers(self):
        with (
            patch.object(run_limited.sys, "platform", "linux"),
            patch.object(run_limited.sys, "argv", ["guard", "--", "cargo", "test"]),
            patch.object(run_limited.shutil, "which", return_value="/usr/bin/tool"),
            patch.object(run_limited.os, "sched_getaffinity", return_value=set(range(3, 15))),
            patch.object(run_limited.Path, "cwd", return_value=Path("/invoking/workspace")),
            patch.object(run_limited.subprocess, "call", return_value=0) as call,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(run_limited.main(), 0)
            command = call.call_args.args[0]
            self.assertIn("--unit=beckon-kit-verification", command)
            self.assertEqual(call.call_args.kwargs["cwd"], Path("/invoking/workspace"))
            self.assertIn(str(Path(run_limited.__file__).resolve()), command)
            for setting in ("MemoryMax=8589934592", "MemoryHigh=6442450944", "MemorySwapMax=0"):
                self.assertIn(setting, command)
            self.assertEqual(command[command.index("--cpu-list") + 1], "3,4,5,6,7,8,9,10")
            env = call.call_args.kwargs["env"]
            for wrapper in ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"):
                self.assertEqual(env[wrapper], "")
            for variable in ("CARGO_BUILD_JOBS", "MOLD_JOBS"):
                self.assertEqual(env[variable], "4")
            for variable in ("LP_NUM_THREADS", "RAYON_NUM_THREADS"):
                self.assertEqual(env[variable], "8")
            self.assertEqual(env["RUST_TEST_THREADS"], "1")

    def test_missing_systemd_does_not_fall_back_to_unrestricted_execution(self):
        with (
            patch.object(run_limited.sys, "platform", "linux"),
            patch.object(run_limited.sys, "argv", ["guard", "--", "cargo", "test"]),
            patch.object(run_limited.shutil, "which", return_value=None),
            patch.object(run_limited.subprocess, "call") as call,
        ):
            with self.assertRaises(RuntimeError):
                run_limited.main()
            call.assert_not_called()


if __name__ == "__main__":
    unittest.main()
