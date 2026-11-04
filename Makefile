.DEFAULT_GOAL := build
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DESTDIR ?=
CARGO_TARGET_DIR ?= target
TARGET ?=
RELEASE_ARGS ?= --apply
BOOTSTRAP = cargo run --locked --quiet -p craftguild-standards --no-default-features --features bootstrap --bin craftguild-standards-bootstrap --
TARGET_ARG = $(if $(TARGET),--target $(TARGET),)

.PHONY: build install test check acceptance banner release
release:
	$(BOOTSTRAP) release $(RELEASE_ARGS)

build:
	$(BOOTSTRAP) build $(TARGET_ARG)

install:
	python3 scripts/install.py --bindir "$(BINDIR)" --destdir "$(DESTDIR)" --target-dir "$(CARGO_TARGET_DIR)" --target "$(TARGET)"

test:
	cargo test --locked --workspace --no-default-features --features bootstrap
	npm test
	python3 -m unittest discover -s scripts/tests

check:
	$(BOOTSTRAP) check

acceptance:
	python3 scripts/acceptance.py

banner:
	$(BOOTSTRAP) banner standards --output assets/banner.txt
