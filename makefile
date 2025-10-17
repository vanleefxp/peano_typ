CURRENT_DIR := $(dir $(MAKEFILE_LIST))

pack:
	@cd $(CURRENT_DIR)
	py "./typacker.py" export
	py "./typacker.py" doc
	py "./typacker.py" copy