BUILDDIR      = _build

.PHONY: html
html:
	python -m sphinx -b html -W . "$(BUILDDIR)"

.PHONY: clean
clean:
	rm -rf $(BUILDDIR)/*
