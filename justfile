#!/usr/bin/env just --justfile

default:
	@just --list

# Setup the development environment.
@setup-dev:
	#!/bin/sh

	set -e

	echo Installing required packages from apt
	sudo apt-get install bear valgrind libapt-pkg-dev clang-format codespell -y

	echo Setting up toolchains
	rustup toolchain install nightly
	rustup toolchain install stable

	echo Installing nightly \`rustfmt\`
	rustup toolchain install nightly --component rustfmt
	echo Nightly \`rustfmt\` successfully installed!

	echo Cleaning and building c++ compile commands
	cargo clean

	bear -- cargo build
	echo Development environment installed successfully!

	# Sudo is required to install packages with apt

# Run checks
check: spellcheck clippy
	@cargo +nightly fmt --check
	@echo Checks were successful!

# Remove generated artifacts
clean:
	@cargo clean
	@echo Done!

# Build the project
build:
	@cargo build
	@echo Project successfully built!

# Create the debs required for tests
create-test-debs:
	#!/bin/sh
	set -e

	cd tests/files/cache
	rm -f *.deb
	for pkg in *; do
		dpkg-deb --build "${pkg}";
	done

# Run all tests except for root
test +ARGS="":
	@just create-test-debs
	@cargo test --no-fail-fast -- --test-threads 1 --skip root {{ARGS}}

# Run only the root tests. Sudo password required!
@test-root +ARGS="":
	#!/bin/sh

	set -e

	just create-test-debs

	sudo -E /home/${USER}/.cargo/bin/cargo \
		test \
		--test root \
		-- --test-threads 1 {{ARGS}}

# Run leak tests. Requires root
@leak:
	#!/bin/sh

	set -e
	just create-test-debs
	cargo test --no-run

	test_binaries=$( \
		find target/debug/deps -executable -type f \
		-printf "%T@ %p\n" | sort -nr | awk '{print $2}' \
		| grep -v ".so"
	)

	for test in $test_binaries; do
		# Sudo is needed to memleak the root tests
		sudo valgrind --leak-check=full -- "${test}" --test-threads 1
	done

# Lint the codebase
clippy +ARGS="":
	@cargo clippy --all-targets --all-features --workspace -- --deny warnings {{ARGS}}
	@echo Lint successful!

# Format the codebase
@fmt +ARGS="":
	#!/bin/sh

	set -e

	cargo +nightly fmt --all -- {{ARGS}}
	cd apt-pkg-c
	clang-format -i *
	echo Codebase formatted successfully!

# Spellcheck the codebase
spellcheck +ARGS="--skip target* --skip .git*":
	@codespell --builtin clear,rare,informal,code --ignore-words-list mut,crate {{ARGS}}
	@echo Spellings look good!
