licensedir = $(datarootdir)/licenses/fontship
datadir = $(datarootdir)/fontship

bin_SCRIPTS = fontship
python_PYTHON = fontship.py
dist_doc_DATA = README.md
license_DATA = LICENSE
dist_data_DATA = src/Makefile

CLEANFILES = $(bin_SCRIPTS)

.PHONY: docker
docker: Dockerfile build-aux/docker-entrypoint.sh
	docker build \
		--build-arg VCS_REF="$(VERSION)" \
		--tag theleagueof/fontship:HEAD \
		./