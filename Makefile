# ỌKỌ build and gate entry points (AGENTS.md §4.7, §8.1).
#
# `make check` is the whole-repo gate: line budget, formatting, types, lint and
# tests. Run it before claiming anything is done; a red gate reported as green
# is worse than no gate.

CARGO ?= cargo
# The crate has dependencies now (ADR 0007's libc), and `--offline` means a
# fresh clone cannot build at all: the gate has to be able to fetch what it
# needs. The flags are still overridable: `CARGO_FLAGS="--jobs 1" make check`.
CARGO_FLAGS ?= --jobs 1

.PHONY: check budget fmt typecheck lint test run watch svg

check: budget fmt typecheck lint test

budget:
	python3 tools/check_line_budget.py

fmt:
	$(CARGO) fmt --all --check

typecheck:
	$(CARGO) check $(CARGO_FLAGS) --all-targets

lint:
	$(CARGO) clippy $(CARGO_FLAGS) --all-targets -- -D warnings

test:
	$(CARGO) test $(CARGO_FLAGS) -- --test-threads=1

# One source of truth for the example: the SVG is translated, then painted.
# `run` paints one frame and stops, because a script wants a frame and not a
# program that never returns. `watch` is the console: it presents the panel
# and reloads it when the file changes.
run:
	$(CARGO) run $(CARGO_FLAGS) --bin oko-svg -- examples/panel.svg > target/panel.oko
	$(CARGO) run $(CARGO_FLAGS) --bin oko -- --once target/panel.oko

watch:
	$(CARGO) run $(CARGO_FLAGS) --bin oko-svg -- examples/panel.svg > target/panel.oko
	$(CARGO) run $(CARGO_FLAGS) --bin oko -- target/panel.oko

svg:
	$(CARGO) run $(CARGO_FLAGS) --bin oko-svg -- examples/panel.svg
