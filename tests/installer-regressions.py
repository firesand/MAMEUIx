#!/usr/bin/env python3
"""Exercise setup and package-recipe blocks with isolated commands; never install or build."""

import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
INSTALLERS = ("install-arch.sh", "install-debian.sh", "install-rpm.sh")


def block(script, start, end):
    source = (ROOT / script).read_text()
    return source[source.index(start):source.index(end, source.index(start))]


class InstallerRegressions(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="mameuix-installer-regression-")
        self.addCleanup(self.temp.cleanup)
        self.work = Path(self.temp.name)
        self.bin = self.work / "bin"
        self.bin.mkdir()
        self.log = self.work / "commands.log"
        self.env = os.environ.copy()
        self.env.update(PATH=str(self.bin), INSTALLER_TEST_LOG=str(self.log))
        for name in ("grep", "sort", "head"):
            (self.bin / name).symlink_to(shutil.which(name))
        for name in ("sudo", "curl", "rustup", "git"):
            self.stub(name, 'printf "forbidden %s\\n" "$0" >> "$INSTALLER_TEST_LOG"\nexit 93\n')

    def stub(self, name, body):
        path = self.bin / name
        path.write_text("#!/bin/bash\n" + body)
        path.chmod(0o755)

    def run_block(self, code):
        return subprocess.run(
            ["/bin/bash", "-c", "set -e\nprint_status() { printf '%s\\n' \"$1\"; }\n"
             "print_success() { printf '%s\\n' \"$1\"; }\n"
             "print_warning() { printf '%s\\n' \"$1\"; }\n"
             "print_error() { printf '%s\\n' \"$1\" >&2; }\n" + code],
            cwd=self.work, env=self.env, text=True, capture_output=True,
        )

    def commands(self):
        return self.log.read_text() if self.log.exists() else ""

    def rust_block(self, script):
        install_end = "# MAME installation" if script == "install-arch.sh" else "# Verify Rust installation"
        return block(script, "# Rust installation:", install_end) + block(
            script, "# Verify Rust installation", "# Select the source checkout"
        )

    def set_rust(self, rust="1.88.0", cargo="1.88.0"):
        for tool, version in (("rustc", rust), ("cargo", cargo)):
            path = self.bin / tool
            if path.exists():
                path.unlink()
            if version is not None:
                self.stub(tool, f'printf "{tool} {version} (test)\\n"\n')

    def test_distro_toolchain_does_not_require_or_update_rustup(self):
        (self.bin / "rustup").unlink()
        self.set_rust()
        for script in INSTALLERS:
            with self.subTest(script=script):
                result = self.run_block(self.rust_block(script))
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(self.commands(), "")

    def test_existing_rustup_toolchain_is_not_updated(self):
        self.set_rust("1.90.0-nightly", "1.90.0-nightly")
        for script in INSTALLERS:
            with self.subTest(script=script):
                result = self.run_block(self.rust_block(script))
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(self.commands(), "")

    def test_old_rust_or_cargo_is_rejected(self):
        for rust, cargo in (("1.87.0", "1.88.0"), ("1.88.0", "1.87.0")):
            self.set_rust(rust, cargo)
            for script in INSTALLERS:
                with self.subTest(script=script, rust=rust, cargo=cargo):
                    result = self.run_block(self.rust_block(script))
                    self.assertEqual(result.returncode, 1)
                    self.assertIn("1.88.0 or newer is required", result.stderr)
                    self.assertEqual(self.commands(), "")

    def test_missing_rust_or_cargo_does_not_replace_toolchain(self):
        for rust, cargo in (("1.88.0", None), (None, "1.88.0")):
            self.set_rust(rust, cargo)
            for script in INSTALLERS:
                with self.subTest(script=script, rust=rust, cargo=cargo):
                    result = self.run_block(self.rust_block(script))
                    self.assertEqual(result.returncode, 1)
                    self.assertIn("is missing", result.stderr)
                    self.assertEqual(self.commands(), "")

    def test_unconfigured_rustup_is_not_reinstalled(self):
        for script in INSTALLERS:
            with self.subTest(script=script):
                result = self.run_block(self.rust_block(script))
                self.assertEqual(result.returncode, 1)
                self.assertIn("Configure your rustup toolchain and PATH", result.stderr)
                self.assertEqual(self.commands(), "")

    def checkout_block(self, script):
        return block(script, "# Select the source checkout", "# Build the project") + "pwd\n"

    def test_fresh_clone_uses_explicit_case_correct_destination(self):
        self.stub("git", 'printf "%s\\n" "$*" >> "$INSTALLER_TEST_LOG"\n'
                  '[[ "$1" == clone && "$3" == MAMEUIx && "$#" == 3 ]] || exit 94\n'
                  '/bin/mkdir "$3"\nprintf \'[package]\\nname = "mameuix"\\n\' > "$3/Cargo.toml"\n')
        for script in INSTALLERS:
            with self.subTest(script=script):
                result = self.run_block(self.checkout_block(script))
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue(result.stdout.rstrip().endswith(str(self.work / "MAMEUIx")))
                self.assertEqual(self.commands().strip(), "clone https://github.com/firesand/MAMEUIx.git MAMEUIx")
                shutil.rmtree(self.work / "MAMEUIx")
                self.log.unlink()

    def test_current_checkout_and_wip_are_preserved(self):
        (self.work / "Cargo.toml").write_text('[package]\nname = "mameuix"\n')
        marker = self.work / "local-work.txt"
        marker.write_text("uncommitted work\n")
        for script in INSTALLERS:
            with self.subTest(script=script):
                result = self.run_block(self.checkout_block(script))
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue(result.stdout.rstrip().endswith(str(self.work)))
                self.assertFalse((self.work / "MAMEUIx").exists())
                self.assertEqual(marker.read_text(), "uncommitted work\n")
                self.assertEqual(self.commands(), "")

    def test_existing_child_checkout_is_reused_without_pull(self):
        checkout = self.work / "MAMEUIx"
        checkout.mkdir()
        (checkout / "Cargo.toml").write_text('[package]\nname = "mameuix"\n')
        for script in INSTALLERS:
            with self.subTest(script=script):
                result = self.run_block(self.checkout_block(script))
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue(result.stdout.rstrip().endswith(str(checkout)))
                self.assertEqual(self.commands(), "")

    def test_unrelated_destination_is_not_overwritten(self):
        (self.work / "MAMEUIx").write_text("unrelated file\n")
        for script in INSTALLERS:
            with self.subTest(script=script):
                result = self.run_block(self.checkout_block(script))
                self.assertEqual(result.returncode, 1)
                self.assertEqual((self.work / "MAMEUIx").read_text(), "unrelated file\n")
                self.assertEqual(self.commands(), "")

    def arch_block(self):
        return block("install-arch.sh", "# Arch prerequisite:", "# Rust installation:")

    def test_arch_installs_from_cached_database_without_sync_or_upgrade(self):
        self.stub("pacman", 'printf "pacman %s\\n" "$*" >> "$INSTALLER_TEST_LOG"\nexit 1\n')
        self.stub("sudo", 'printf "sudo %s\\n" "$*" >> "$INSTALLER_TEST_LOG"\n')
        result = self.run_block(self.arch_block())
        self.assertEqual(result.returncode, 0, result.stderr)
        commands = self.commands().splitlines()
        self.assertEqual(commands[0], "pacman -Qu")
        self.assertEqual(len(commands), 2)
        self.assertTrue(commands[1].startswith("sudo pacman -S --needed --noconfirm "))
        self.assertNotIn("-Sy", self.commands())

    def test_arch_pending_upgrades_prevent_package_installation(self):
        self.stub("pacman", 'printf "pacman %s\\n" "$*" >> "$INSTALLER_TEST_LOG"\n'
                  'printf "glibc 2.40-1 -> 2.41-1\\n"\n')
        result = self.run_block(self.arch_block())
        self.assertEqual(result.returncode, 1)
        self.assertIn("Pending system upgrades detected", result.stderr)
        self.assertEqual(self.commands().strip(), "pacman -Qu")

    def test_arch_query_failure_prevents_package_installation(self):
        self.stub("pacman", 'exit 2\n')
        result = self.run_block(self.arch_block())
        self.assertEqual(result.returncode, 2)
        self.assertIn("Could not check", result.stderr)
        self.assertEqual(self.commands(), "")

    def test_local_arch_recipes_build_current_cargo_version_without_changing_release_recipe(self):
        for name in ("cp", "cut", "gzip", "ls", "mkdir", "mktemp", "rm", "sed", "sha256sum", "tar"):
            (self.bin / name).symlink_to(shutil.which(name))
        self.env["EXPECTED_VERSION"] = "0.1.8"
        # Deliberately retain an older, remotely sourced release recipe. The
        # helper must use this checkout even before the next tag/hash exists.
        historical_recipe = re.sub(
            r"^pkgver=.*$", "pkgver=0.1.6", (ROOT / "PKGBUILD").read_text(), flags=re.MULTILINE
        )
        (self.work / "PKGBUILD").write_text(historical_recipe)
        (self.work / "Cargo.toml").write_text('[package]\nname = "mameuix"\nversion = "0.1.8"\n')
        self.stub("makepkg", r'''set -euo pipefail
source ./PKGBUILD
[[ "$pkgver" == "$EXPECTED_VERSION" ]]
[[ "${#source[@]}" == 1 ]]
[[ "${source[0]}" == "mameuix-$EXPECTED_VERSION.tar.gz" ]]
printf '%s  %s\n' "${sha256sums[0]}" "${source[0]}" | sha256sum -c
tar -xOzf "${source[0]}" "MAMEUIx-$EXPECTED_VERSION/Cargo.toml" |
    grep -Fx "version = \"$EXPECTED_VERSION\""
printf 'makepkg %s\n' "$pkgver" >> "$INSTALLER_TEST_LOG"
printf 'stub package\n' > "$pkgname-$pkgver-$pkgrel-x86_64.pkg.tar.zst"
''')
        helpers = {
            "build-arch.sh": (ROOT / "build-arch.sh").read_text(),
            # Run only the package-assembly block, excluding distro setup and
            # optional installation prompts. The real recipe edits are intact.
            "build-arch-package.sh": (
                block("build-arch-package.sh", "VERSION=$(grep", "# Clean previous builds")
                + block("build-arch-package.sh", "BUILD_TMP_DIR=$(mktemp -d)", "# Find the built package")
            ),
            "build-packages.sh": (
                block("build-packages.sh", "# RPM uses", "# Function to build Debian package")
                + block("build-packages.sh", "# Function to build Arch package", "# Function to build AppImage")
                + "\nbuild_arch\n"
            ),
        }
        for script, code in helpers.items():
            with self.subTest(script=script):
                result = self.run_block(code)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertEqual(self.commands(), "makepkg 0.1.8\n")
                self.assertEqual((self.work / "PKGBUILD").read_text(), historical_recipe)
                package = self.work / "mameuix-0.1.8-1-x86_64.pkg.tar.zst"
                self.assertTrue(package.is_file())
                package.unlink()
                self.log.unlink()


if __name__ == "__main__":
    unittest.main(verbosity=2)
