IMAGE_VOLUME=cubic-images
INSTANCE_VOLUME=cubic-instances
BUILD_VOLUME=cubic-build
CARGO_VOLUME=cubic-cargo
DOCKER_CMD=docker run --rm -v .:/usr/local/app \
	-v ${IMAGE_VOLUME}:/tmp/cache \
	-v ${INSTANCE_VOLUME}:/tmp/data \
	-v ${BUILD_VOLUME}:/usr/local/app/target \
	-v ${CARGO_VOLUME}:/usr/local/cargo \
	-p 4000:4000
IMAGE=cubic:latest

CMDS= run create instances images ports show modify console ssh scp start stop \
		restart rename clone delete prune completions

volume-%:
	@if [ -z "`docker images -q $<`" ]; then docker build -t < .; fi

build-image: volume-${IMAGE_VOLUME} volume-${INSTANCE_VOLUME} volume-${BUILD_VOLUME} volume-${CARGO_VOLUME}
	@if [ -z "`docker images -q ${IMAGE}`" ]; then docker build -t ${IMAGE} .; fi

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
	${DOCKER_CMD} ${IMAGE} cargo audit

update: build-image
	${DOCKER_CMD} ${IMAGE} cargo update

sh: build-image
	${DOCKER_CMD} -it ${IMAGE} bash

check: format lint test audit

fix: fix-format fix-lint

build: build-image
	${DOCKER_CMD} ${IMAGE} cargo build

doc: build-image
	@${DOCKER_CMD} -it ${IMAGE} ./scripts/generate-docs.sh dev
	@${DOCKER_CMD} -it ${IMAGE} sphinx-build docs target/doc
	@${DOCKER_CMD} -it ${IMAGE} python3 -m http.server -d target/doc 4000

release: build-image
	sed "s/^\(version =\).*$$/\1 \"${version}\"/g" -i Cargo.toml
	make build
