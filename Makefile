IMAGE_VOLUME=cubic-images
INSTANCE_VOLUME=cubic-instances
BUILD_VOLUME=cubic-build
CARGO_VOLUME=cubic-cargo
DOCKER_CMD=docker run --rm -v .:/usr/local/app \
	-v ${IMAGE_VOLUME}:/tmp/cache \
	-v ${INSTANCE_VOLUME}:/tmp/data \
	-v ${BUILD_VOLUME}:/usr/local/app/target \
	-v ${CARGO_VOLUME}:/usr/local/cargo
IMAGE=cubic:latest
VULNLOG_IMAGE=ghcr.io/vulnlog/vulnlog:0.17.0
VULNLOG_CMD=docker run --rm --user $$(id -u):$$(id -g) -v .:/work ${VULNLOG_IMAGE}

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
	${VULNLOG_CMD} fmt --check vulnlog.yml

fix-format: build-image
	${DOCKER_CMD} ${IMAGE} cargo fmt
	${VULNLOG_CMD} fmt vulnlog.yml

lint: build-image
	${DOCKER_CMD} ${IMAGE} cargo clippy --all-targets -- -D warnings
	${VULNLOG_CMD} validate --strict vulnlog.yml

fix-lint: build-image
	${DOCKER_CMD} ${IMAGE} cargo clippy --all-targets --fix --allow-dirty --allow-staged

yamllint: build-image
	${DOCKER_CMD} ${IMAGE} yamllint --strict .

shellcheck: build-image
	${DOCKER_CMD} ${IMAGE} shellcheck .github/scripts/*.sh scripts/*.sh

test: build-image
	${DOCKER_CMD} ${IMAGE} cargo test

audit: build-image
	${DOCKER_CMD} ${IMAGE} cargo audit

update: build-image
	${DOCKER_CMD} ${IMAGE} cargo update

sh: build-image
	${DOCKER_CMD} --device=/dev/kvm -it ${IMAGE} bash

check: format lint yamllint shellcheck test audit

fix: fix-format fix-lint

build: build-image
	${DOCKER_CMD} ${IMAGE} cargo build

generate-image-list: build-image
	${DOCKER_CMD} ${IMAGE} cargo run --bin cubic-generate-image-list -- src/image/images.toml

doc: build-image
	@${DOCKER_CMD} -it ${IMAGE} ./scripts/generate-page.sh v0.0.0-dev
	@${DOCKER_CMD} -p 4000:4000 -it ${IMAGE} python3 -m http.server -d target/page 4000

suppress:
	@${VULNLOG_CMD} suppress vulnlog.yml -o .cargo/audit.toml

release: build-image
	sed "s/^\(version =\).*$$/\1 \"${version}\"/g" -i Cargo.toml
	make build
