.DEFAULT_GOAL := build
STANDARDS ?= craftguild-standards
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DESTDIR ?=
CARGO_TARGET_DIR ?= backend/target
TARGET ?=
.PHONY: build test check acceptance install
build:
	$(STANDARDS) build $(if $(TARGET),--target $(TARGET),)
test:
	cargo test --locked --release --manifest-path backend/Cargo.toml
	npm --prefix frontend test
check:
	$(STANDARDS) check
acceptance:
	python3 scripts/acceptance.py
install:
	python3 scripts/install.py --bindir "$(BINDIR)" --destdir "$(DESTDIR)" --target-dir "$(CARGO_TARGET_DIR)" --target "$(TARGET)"
