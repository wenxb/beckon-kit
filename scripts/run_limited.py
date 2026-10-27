#!/usr/bin/env python3
"""Run verification in a Linux user scope with a hard, process-tree memory cap."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys



def cgroup() -> Path:
    for line in Path("/proc/self/cgroup").read_text().splitlines():
        if line.startswith("0::"):
            return Path("/sys/fs/cgroup") / line.removeprefix("0::").lstrip("/")
    raise RuntimeError("cgroup v2 is required for process-tree memory accounting")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--memory-mib", type=int, default=8192)
    parser.add_argument("--in-scope", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        parser.error("provide a command after --")
    if not 512 <= args.memory_mib <= 8192:
        parser.error("memory cap must be between 512 and 8192 MiB")

    cwd = Path.cwd()
    maximum = args.memory_mib * 1024 * 1024
    if args.in_scope:
        group = cgroup()
        actual = (group / "memory.max").read_text().strip()
        if actual == "max" or int(actual) > maximum:
            raise RuntimeError("verification scope has no effective memory cap")
        if (group / "memory.swap.max").read_text().strip() != "0":
            raise RuntimeError("verification scope must not use swap")
        limit_reached = False
        try:
            result = subprocess.call(command, cwd=cwd)
        finally:
            peak = int((group / "memory.peak").read_text()) / (1024 * 1024)
            print(f"beckon-kit verification: cgroup peak {peak:.0f} MiB / {args.memory_mib} MiB", flush=True)
            events = (group / "memory.events").read_text().splitlines()
            for event in events:
                if event.startswith(("oom ", "oom_kill ")) and event.split()[1] != "0":
                    limit_reached = True
                    print(f"verification resource limit reached: {event}", file=sys.stderr)
        return result if result != 0 else int(limit_reached)

    if sys.platform != "linux" or not shutil.which("systemd-run") or not shutil.which("taskset"):
        raise RuntimeError("requires Linux, cgroup v2 and a user systemd manager; no unbounded fallback")

    # A shared sccache daemon lives outside this scope. Run rustc directly so the
    # memory cap covers compilation as well as Cargo, linking and GPU validation.
    env = os.environ.copy()
    env.update({
        "RUSTC_WRAPPER": "",
        "RUSTC_WORKSPACE_WRAPPER": "",
        "CARGO_BUILD_JOBS": "4",
        "RUST_TEST_THREADS": "1",
        "MOLD_JOBS": "4",
        "LP_NUM_THREADS": "8",
        "RAYON_NUM_THREADS": "8",
    })
    cpus = sorted(os.sched_getaffinity(0))[:8]
    print(f"beckon-kit verification: 4 Cargo jobs, {len(cpus)} CPUs, {args.memory_mib} MiB maximum, no swap", flush=True)
    return subprocess.call([
        "systemd-run", "--user", "--scope", "--quiet", "--collect",
        "--unit=beckon-kit-verification",
        "-p", f"MemoryHigh={maximum * 3 // 4}",
        "-p", f"MemoryMax={maximum}",
        "-p", "MemorySwapMax=0",
        "taskset", "--cpu-list", ",".join(map(str, cpus)),
        sys.executable, str(Path(__file__).resolve()),
        "--in-scope", "--memory-mib", str(args.memory_mib), "--", *command,
    ], cwd=cwd, env=env)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError) as error:
        print(f"beckon-kit verification: {error}", file=sys.stderr)
        raise SystemExit(1)
