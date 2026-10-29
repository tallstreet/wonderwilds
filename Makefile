.DEFAULT_GOAL := help

.PHONY: help install dev guest

help:
	@printf '%s\n' \
	  'make install  Install locked JavaScript and Rust dependencies' \
	  'make dev      Start the local backend demo (PostgreSQL 17 required)' \
	  'make guest    Start only the guest frontend at http://127.0.0.1:5173'

install:
	npm --prefix frontend ci
	npm --prefix contracts ci
	npm --prefix qa/e2e ci
	cargo fetch --locked

# The launcher manages migrations, session setup, and frontend/API shutdown.
# WONDERWILDS_PG_BIN may be set in the environment or on the make command line.
dev:
	sh scripts/local-demo.sh

guest:
	npm --prefix frontend run dev -- --host 127.0.0.1 --port 5173 --strictPort
