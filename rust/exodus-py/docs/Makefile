# Makefile for Sphinx documentation

# You can set these variables from the command line.
SPHINXOPTS    = -W --keep-going
SPHINXBUILD   = python3 -m sphinx
SOURCEDIR     = .
BUILDDIR      = _build

# Put it first so that "make" without argument is like "make help".
help:
	@$(SPHINXBUILD) -M help "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

.PHONY: help Makefile

# Catch-all target: route all unknown targets to Sphinx using the new
# "make mode" option.  $(O) is meant as a shortcut for $(SPHINXOPTS).
%: Makefile
	@$(SPHINXBUILD) -M $@ "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

# Custom targets
.PHONY: clean html livehtml

clean:
	rm -rf $(BUILDDIR)

html:
	@$(SPHINXBUILD) -b html $(SPHINXOPTS) "$(SOURCEDIR)" "$(BUILDDIR)/html"
	@echo
	@echo "Build finished. The HTML pages are in $(BUILDDIR)/html."

# Auto-reload server (requires sphinx-autobuild)
livehtml:
	@command -v sphinx-autobuild >/dev/null 2>&1 || { \
		echo "sphinx-autobuild not found. Install with: pip install sphinx-autobuild"; \
		exit 1; \
	}
	sphinx-autobuild "$(SOURCEDIR)" "$(BUILDDIR)/html" $(SPHINXOPTS) $(O)
