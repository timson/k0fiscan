install:
	@which uv || pip install uv
	uv pip install ruff black mypy


check:
	uv run ruff check gen_services.py
	uv run black --check gen_services.py
	uv run mypy gen_services.py

fmt:
	uv run black gen_services.py
