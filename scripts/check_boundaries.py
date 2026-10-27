#!/usr/bin/env python3
"""Check the standalone component crate's dependency and source boundaries."""

from pathlib import Path
import re
import json
import os
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
FORBIDDEN = {"gpui-kit", "gpui-kit-assets", "gpui-base", "gpui-component", "bezel"}
ENGINE_VERSION = "0.3.9"
VENDORED = {
    "gpui-pre", "gpui-pre-platform", "gpui-pre-linux", "gpui-pre-wgpu",
    "gpui-pre-apple", "gpui-pre-macos", "gpui-pre-windows", "gpui-pre-web",
}


def check_local_dependencies(file: Path) -> None:
    manifest = tomllib.loads(file.read_text())
    tables = [manifest, *manifest.get("target", {}).values()]
    for table in tables:
        for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
            for name, dependency in table.get(kind, {}).items():
                package = dependency.get("package", name) if isinstance(dependency, dict) else name
                if package not in VENDORED:
                    continue
                assert isinstance(dependency, dict) and isinstance(dependency.get("path"), str), (
                    f"{file}: {kind} {name} must explicitly use the local {package} path"
                )
                assert (file.parent / dependency["path"]).resolve() == (ROOT / "vendor" / package).resolve(), (
                    f"{file}: {kind} {name} points outside the vendored {package}"
                )
                assert dependency.get("version") == f"={ENGINE_VERSION}", (
                    f"{file}: {kind} {name} must pin ={ENGINE_VERSION}"
                )


def check_engine_sources() -> None:
    check_local_dependencies(ROOT / "Cargo.toml")
    for name in VENDORED:
        source = ROOT / "vendor" / name
        file = source / "Cargo.toml"
        package = tomllib.loads(file.read_text())["package"]
        assert package["name"] == name and package["version"] == ENGINE_VERSION
        assert package["license"] == "Apache-2.0", f"{name}: unexpected upstream license"
        assert (source / "LICENSE-APACHE").is_file(), f"{name}: upstream license missing"
        check_local_dependencies(file)

    # New/split modules obey the source limit; unrelated upstream legacy files
    # keep their original layout instead of undergoing a cosmetic mass rewrite.
    modules = [
        ROOT / "vendor/gpui-pre/src/scene",
        ROOT / "vendor/gpui-pre/src/window/backdrop.rs",
        ROOT / "vendor/gpui-pre/src/window/rounded_clip.rs",
        ROOT / "vendor/gpui-pre/src/platform/test/window_frame.rs",
        ROOT / "vendor/gpui-pre/src/app/test_context/window_frame.rs",
        ROOT / "vendor/gpui-pre/src/app/test_context/backdrop.rs",
        ROOT / "vendor/gpui-pre-wgpu/src/backdrop",
        ROOT / "vendor/gpui-pre-wgpu/src/wgpu_renderer",
    ]
    for module in modules:
        for file in module.rglob("*") if module.is_dir() else [module]:
            if file.is_file() and file.suffix in {".rs", ".wgsl"}:
                assert len(file.read_text().splitlines()) <= 600, f"{file}: exceeds 600 lines"


def check_engine_resolution() -> None:
    metadata = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        cwd=ROOT, text=True, capture_output=True, check=True,
    )
    engines = [package for package in json.loads(metadata.stdout)["packages"] if package["name"] == "gpui-pre"]
    assert len(engines) == 1 and engines[0]["source"] is None, "expected exactly one local GPUI engine"
    assert engines[0]["version"] == ENGINE_VERSION, "resolved GPUI engine version differs"
    assert Path(engines[0]["manifest_path"]).resolve() == (ROOT / "vendor/gpui-pre/Cargo.toml").resolve(), (
        "resolved GPUI engine is not the vendored source"
    )


def check() -> None:
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    assert manifest["features"]["default"] == [], "platform/gallery must remain opt-in"
    assert manifest["dependencies"]["gpui"]["package"] == "gpui-pre"
    platform = manifest["dependencies"]["gpui_platform"]
    assert platform["package"] == "gpui-pre-platform"
    assert platform["optional"], "platform bootstrap leaked into the default library"
    check_engine_sources()
    check_engine_resolution()

    for directory, children, files in os.walk(ROOT):
        children[:] = [name for name in children if name not in {"vendor", "target", ".git"}]
        files = [name for name in files if name != "rust-toolchain.toml"]
        for name in files:
            file = Path(directory) / name
            if file.suffix not in {".rs", ".py", ".toml", ".md"}:
                continue
            count = len(file.read_text().splitlines())
            assert count <= 600, f"{file.relative_to(ROOT)} exceeds 600 lines: {count}"

    for features in [[], ["--features", "gallery,test-support"]]:
        result = subprocess.run(
            ["cargo", "tree", "--locked", "-p", "beckon-kit", "--target", "all", "--edges", "normal",
             "--prefix", "none", "--format", "{p}", *features],
            cwd=ROOT, text=True, capture_output=True, check=True,
        )
        packages = {line.split()[0] for line in result.stdout.splitlines() if line.strip()}
        forbidden = packages & FORBIDDEN
        forbidden |= {name for name in packages if name.startswith("bezel-")}
        forbidden |= {name for name in packages if name.startswith("beckon-") and name != "beckon-kit"}
        assert not forbidden, f"forbidden component/business dependencies: {sorted(forbidden)}"
        engines = set(re.findall(r"^(gpui-pre|gpui-fast|gpui|bezel-gpui) v(\S+)", result.stdout, re.M))
        assert engines == {("gpui-pre", ENGINE_VERSION)}, f"expected one vendored engine identity, found {engines}"

    print("beckon-kit: independent dependencies, one local GPUI engine, same-version licensed vendors, authored modules ≤ 600 lines")


if __name__ == "__main__":
    check()
