DOCKER_CMD=docker run --rm -v .:/usr/local/app
IMAGE=cubic:latest

build-image:
	if [ -z "`docker images -q ${IMAGE}`" ]; then docker build -t ${IMAGE} .; fi

clean: build-image
	${DOCKER_CMD} ${IMAGE} cargo clean

cleanall: build-image
	docker image rm -f ${IMAGE}

format: build-image
	${DOCKER_CMD} ${IMAGE} cargo fmt --check

fix-format: build-image
	${DOCKER_CMD} ${IMAGE} cargo fmt

lint: build-image
	${DOCKER_CMD} ${IMAGE} cargo clippy -- -D warnings

fix-lint: build-image
	${DOCKER_CMD} ${IMAGE} cargo clippy --fix --allow-dirty --allow-staged

test: build-image
	${DOCKER_CMD} ${IMAGE} cargo test

audit: build-image
	${DOCKER_CMD} ${IMAGE} cargo audit --ignore RUSTSEC-2023-0071 # no fix available so far

update: build-image
	${DOCKER_CMD} ${IMAGE} cargo update

sh: build-image
	${DOCKER_CMD} -it ${IMAGE} bash

check: format lint test audit

fix: fix-format fix-lint

build: build-image
	${DOCKER_CMD} ${IMAGE} cargo build

doc: build-image
	${DOCKER_CMD} ${IMAGE} sphinx-build docs target/doc && python3 -m http.server -d target/doc 4000

release: build-image
	sed "s/^\(version =\).*$$/\1 \"${version}\"/g" -i Cargo.toml
	sed "s/^\\(version:\\).*$$/\\1 '${version}'/g" -i snapcraft.yaml
	sed "s/^\\(release = \\).*$$/\\1 'v${version}'/g" -i docs/conf.py
	make build
