.PHONY: lint format typecheck test coverage check

lint:
	ruff check src tests

format:
	ruff format src tests

typecheck:
	mypy src tests

test:
	pytest

coverage:
	pytest --cov-report=xml --cov-report=term-missing

check: lint typecheck test
