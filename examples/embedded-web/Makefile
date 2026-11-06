.DEFAULT_GOAL := build
STANDARDS ?= craftguild-standards-engine
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DESTDIR ?=
CARGO_TARGET_DIR ?= target
TARGET ?=
RELEASE_ARGS ?= --apply
.PHONY: build test check acceptance install release
release:
	$(STANDARDS) release $(RELEASE_ARGS)
build:
	$(STANDARDS) build $(if $(TARGET),--target $(TARGET),)
test:
	cargo test --locked --release
	npm --prefix web test
check:
	$(STANDARDS) check
acceptance:
	python3 scripts/acceptance.py
install:
	python3 scripts/install.py --bindir "$(BINDIR)" --destdir "$(DESTDIR)" --target-dir "$(CARGO_TARGET_DIR)" --target "$(TARGET)"
