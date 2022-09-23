#!/usr/bin/env just --justfile

# Setup the development environment
setup-dev:
	@echo Installing required packages from apt
	@sudo apt-get install bear valgrind libapt-pkg-dev clang-format codespell -y

	@echo Setting up toolchains
	@rustup toolchain install nightly
	@rustup toolchain install stable

	@echo Installing nightly \`rustfmt\`
	@rustup toolchain install nightly --component rustfmt
	@echo Nightly \`rustfmt\` successfully installed!

	@echo Cleaning and building c++ compile commands
	@cargo clean
	@bear -- cargo build

	@echo Development environment installed successfully!

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

# Run the tests
test +ARGS="":
	@cargo test --doc
	@cargo test -- --test-threads 1 {{ARGS}}

test_root +ARGS="":
	@cargo test --no-run
	@sudo -- $( \
		find target/debug/deps/ \
		-executable \
		-type f \
		-name "tests-*" \
		-printf "%T@ %p\n" | sort -nr | awk '{print $2}' \
	) --test-threads 1 {{ARGS}}


# Run leak tests. Requires root
leak:
	@cargo test --no-run
	@sudo -- valgrind --leak-check=full -- $( \
		find target/debug/deps/ \
		-executable \
		-type f \
		-name "tests-*" \
		-printf "%T@ %p\n" | sort -nr | awk '{print $2}' \
	) --test-threads 1

# Lint the codebase
clippy +ARGS="":
	@cargo clippy --all-targets --all-features --workspace -- --deny warnings {{ARGS}}
	@echo Lint successful!

# Format the codebase
fmt +ARGS="":
	@cargo +nightly fmt --all -- {{ARGS}}
	@clang-format -i \
		apt-pkg-c/cache.cc \
		apt-pkg-c/cache.h \
		apt-pkg-c/configuration.cc \
		apt-pkg-c/configuration.h \
		apt-pkg-c/progress.cc \
		apt-pkg-c/progress.h \
		apt-pkg-c/util.cc \
		apt-pkg-c/util.h \
		apt-pkg-c/depcache.cc \
		apt-pkg-c/depcache.h \
		apt-pkg-c/records.cc \
		apt-pkg-c/records.h \
		apt-pkg-c/resolver.cc \
		apt-pkg-c/resolver.h \
		apt-pkg-c/package.cc \
		apt-pkg-c/package.h \
		apt-pkg-c/pkgmanager.cc \
		apt-pkg-c/pkgmanager.h
	@echo Codebase formatted successfully!

# Spellcheck the codebase
spellcheck +ARGS="--skip target*":
	@codespell --builtin clear,rare,informal,code --ignore-words-list mut,crate {{ARGS}}
	@echo Spellings look good!
