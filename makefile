CURRENT_DIR := $(dir $(MAKEFILE_LIST))

pack:
	@cd $(CURRENT_DIR)
	py "./export.py"
	py "./pack.py"