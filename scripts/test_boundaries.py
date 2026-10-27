"""Manifest boundary regressions; Cargo is never invoked."""

from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_boundaries


class LocalDependencies(unittest.TestCase):
    def check_dependency(self, table: str, dependency: str, vendored: bool = False) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "vendor/gpui-pre-platform" if vendored else root
            source.mkdir(parents=True, exist_ok=True)
            file = source / "Cargo.toml"
            file.write_text(f'[{table}]\nengine = {dependency}\n')
            with patch.object(check_boundaries, "ROOT", root):
                check_boundaries.check_local_dependencies(file)

    def test_explicit_paths_work_from_root_and_vendor_manifests(self):
        for vendored, path in [(False, "vendor/gpui-pre"), (True, "../gpui-pre")]:
            with self.subTest(vendored=vendored):
                self.check_dependency(
                    "dependencies",
                    '{ package = "gpui-pre", version = "=0.3.9", path = "' + path + '" }',
                    vendored,
                )

    def test_missing_wrong_and_unpinned_paths_are_rejected_in_every_dependency_table(self):
        dependencies = [
            '{ package = "gpui-pre", version = "=0.3.9" }',
            '{ package = "gpui-pre", version = "=0.3.9", path = "elsewhere" }',
            '{ package = "gpui-pre", version = "0.3.9", path = "vendor/gpui-pre" }',
        ]
        for prefix in ("", 'target.\'cfg(target_os = "linux")\'.'):
            for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
                for dependency in dependencies:
                    with self.subTest(table=prefix + kind, dependency=dependency):
                        with self.assertRaises(AssertionError):
                            self.check_dependency(prefix + kind, dependency)

    def test_unaliased_registry_dependency_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            file = root / "Cargo.toml"
            file.write_text('[dependencies]\ngpui-pre = "=0.3.9"\n')
            with patch.object(check_boundaries, "ROOT", root), self.assertRaises(AssertionError):
                check_boundaries.check_local_dependencies(file)


if __name__ == "__main__":
    unittest.main()
