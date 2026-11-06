.DEFAULT_GOAL := build
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DESTDIR ?=
TARGET ?=
export TARGET ARTIFACT VARIANT TOOLCHAIN FEATURES NO_DEFAULT_FEATURES
ifneq ($(origin PROFILE),undefined)
export PROFILE
endif
ifneq ($(origin CARGO_TARGET_DIR),undefined)
export CARGO_TARGET_DIR
endif
RELEASE_ARGS ?= --apply
BOOTSTRAP = cargo run --offline --locked --quiet -p craftguild-standards --no-default-features --features bootstrap --bin craftguild-standards-bootstrap --
TARGET_ARG = $(if $(TARGET),--target $(TARGET),)

.PHONY: build install installer package test check acceptance banner release
installer:
	$(BOOTSTRAP) installer $(INSTALLER_ARGS)

package:
	python3 scripts/engine.py package $(PACKAGE_ARGS)

release:
	$(BOOTSTRAP) release $(RELEASE_ARGS)

# Build both initializer and full engine; only the initializer is installed.
build:
	$(BOOTSTRAP) build $(TARGET_ARG) $(BUILD_ARGS)

install:
	python3 scripts/install.py --bindir "$(BINDIR)" --destdir "$(DESTDIR)" --target-dir "$(CARGO_TARGET_DIR)" --target "$(TARGET)" --profile "$(PROFILE)"

test:
	$(BOOTSTRAP) test

check:
	$(BOOTSTRAP) check

acceptance:
	$(BOOTSTRAP) acceptance

banner:
	$(BOOTSTRAP) banner standards --output assets/banner.txt
