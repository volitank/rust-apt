#!/usr/bin/python3
import sys
import os
import re
from argparse import ArgumentParser
from argparse import RawTextHelpFormatter
from shutil import which
from subprocess import CalledProcessError, run
from pathlib import Path

from apt import Cache
from requests import get

parser = ArgumentParser(formatter_class=RawTextHelpFormatter)
sub_parser = parser.add_subparsers(required=True, dest="command")

# Parser for the setup subcommand
setup_parser = sub_parser.add_parser(
	"setup",
	formatter_class=RawTextHelpFormatter,
	help="Setup the development environment for rust-apt.",
	description=(
		"Setup installs cargo, bear, libapt-pkg-dev, clang-format and valgrind\n\n"
		"Setup will then build 'rust-apt' with bear\n"
		"to generate compile_commands.json for c++ linting."
	),
)

# Parser for the test subcommand
test_parser = sub_parser.add_parser(
	"test",
	formatter_class=RawTextHelpFormatter,
	help="Run unit/integration tests.",
	description=(
		"Run unit/integration tests.\n\n"
		"If no functions are specified then all tests will be run.\n\n"
		"If '--leaks' is used all tests will be compiled\n"
		"and then run with valgrind to check for memory leaks."
	),
)
test_parser.add_argument(
	"functions",
	nargs="*",
	help="Test specific functions.",
)
test_parser.add_argument(
	"--leaks",
	action="store_true",
	help="Test for memory leaks.",
)
test_parser.add_argument(
	"--show-output",
	action="store_true",
	help="Display the output for the tests.",
)

# Parser for the format subcommand
format_parser = sub_parser.add_parser(
	"format",
	formatter_class=RawTextHelpFormatter,
	help="Format the rust-apt code with 'cargo fmt' and 'clang-format'.",
	description="Format the rust-apt code with 'cargo fmt' and 'clang-format'.",
)

args = parser.parse_args()


# Different Safety Checks
path_check = (
	Path("./Cargo.toml"),
	Path("./apt-pkg-c"),
	Path("./src"),
	Path("./ORIGINAL.MIT"),
)

for path in path_check:
	if not path.exists():
		sys.exit("Error: It appears you are not in the 'rust-apt' root directory.")

if not which("apt-get"):
	sys.exit("Error: This system must have 'apt'")


def run_cmd(cmd: str, input=None):
	"""Wrap subprocess.run for easy exit if returncode is bad."""
	try:
		run(cmd.split(), input=input, check=True)
	except CalledProcessError as error:
		print(f"Error: Command {error.cmd} failed!")
		sys.exit(error.returncode)


def install_cargo() -> bool:
	"""Install cargo if it's needed."""
	if not which("cargo"):
		print("Starting rustup installer...")
		resp = get("https://sh.rustup.rs")
		resp.raise_for_status()
		os.environ["PATH"] = f"{os.environ['PATH']}:{os.environ['HOME']}/.cargo/bin"
		run_cmd("/bin/sh", input=resp.content)
		return True
	return False


def install_apt_dependencies() -> None:
	"""Install any dependencies needed from apt."""
	cache = Cache()
	dev_deps = (
		"bear",
		"libapt-pkg-dev",
		"clang-format",
		"valgrind",
	)

	not_found = []
	needs_install = []
	for pkg_name in dev_deps:
		if pkg_name not in cache:
			not_found.append(pkg_name)
			continue
		if not cache[pkg_name].installed:
			needs_install.append(pkg_name)

	if not_found:
		sys.exit(f"Error: Can not locate: {', '.join(not_found)}")
	if needs_install:
		print("The following packages need to be installed:")
		print(f"  {', '.join(needs_install)}")

		print("Starting apt-get...")
		# Use regular run because we don't care if update fails.
		run(["sudo", "apt-get", "update"])
		run_cmd(f"sudo apt-get install {' '.join(needs_install)}")


# This is the start of the main program
if args.command == "format":
	# Format rust code.
	run_cmd("cargo +nightly fmt")
	# Format c++ code.
	run_cmd("clang-format -i apt-pkg-c/apt-pkg.cc apt-pkg-c/apt-pkg.h")

if args.command == "setup":
	cargo = install_cargo()

	# Make sure that nightly is installed and then update
	run_cmd("rustup default nightly")
	run_cmd("rustup default stable")
	run_cmd("rustup update")

	install_apt_dependencies()

	run_cmd("cargo clean")
	run_cmd("bear -- cargo build")
	if cargo:
		print(
			"\nCargo was just installed. You may need to restart your shell to access the commands."
		)

if args.command == "test":
	cargo_footer = (
		"--nocapture --test-threads 1" if args.show_output else "--test-threads 1"
	)

	if args.leaks:
		# Compile the test binary and regex it's path
		binary = re.findall(
			r"^  Executable tests/tests.rs.\((.*?)\)$",
			run(
				"cargo test --no-run".split(),
				capture_output=True,
				text=True,
				check=True,
			).stderr,
			re.MULTILINE,
		)[0]

		# Run valgrind against the test binary
		run(f"valgrind --leak-check=full -- {binary} {cargo_footer}".split())
		sys.exit()

	if not args.functions:
		run_cmd(f"cargo test -- {cargo_footer}")
		sys.exit()

	for arg in args.functions:
		run_cmd(f"cargo test {arg} -- {cargo_footer}")
