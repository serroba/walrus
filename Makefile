.PHONY: lint format typecheck test coverage check

lint:
	uv run ruff check src tests

format:
	uv run ruff format src tests

typecheck:
	uv run mypy src tests

test:
	uv run pytest

coverage:
	uv run pytest --cov-report=xml --cov-report=term-missing

check: lint typecheck test
