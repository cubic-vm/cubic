IMAGE=cubic:latest
DOCKER_CMD=docker run --rm -it -v .:/usr/local/app \
	-v cubic-images:/tmp/cache \
	-v cubic-instances:/tmp/data \
	-v cubic-build:/usr/local/app/target \
	-v cubic-cargo:/usr/local/cargo
VULNLOG_IMAGE=ghcr.io/vulnlog/vulnlog:0.17.0
VULNLOG_CMD=docker run --rm --user $$(id -u):$$(id -g) -v .:/work ${VULNLOG_IMAGE}

CMDS= run create instances images ports show modify console ssh scp start stop \
		restart rename clone delete prune completions

build-image:
	@if [ -z "`docker images -q ${IMAGE}`" ]; then docker build -t ${IMAGE} .; fi

sh: build-image
	${DOCKER_CMD} --device=/dev/kvm -p 4000:4000 ${IMAGE} bash

cleanall:
	docker image rm -f ${IMAGE}

clean:
	cargo clean

format:
	cargo fmt --check

fix-format:
	cargo fmt

lint:
	cargo clippy --all-targets -- -D warnings

fix-lint:
	cargo clippy --all-targets --fix --allow-dirty --allow-staged

yamllint:
	yamllint --strict .

shellcheck:
	shellcheck .github/scripts/*.sh scripts/*.sh

test:
	cargo test

audit:
	cargo audit

update:
	cargo update

check: format lint yamllint shellcheck test audit

fix: fix-format fix-lint

build:
	cargo build

generate-image-list:
	cargo run --bin cubic-generate-image-list -- src/image/images.toml

doc:
	@./scripts/generate-page.sh v0.0.0-dev
	@python3 -m http.server -d target/page 4000

vulnlog:
	${VULNLOG_CMD} validate --strict vulnlog.yml
	${VULNLOG_CMD} fmt --check vulnlog.yml

fix-vulnlog:
	${VULNLOG_CMD} fmt vulnlog.yml

suppress:
	@${VULNLOG_CMD} suppress vulnlog.yml -o .cargo/audit.toml

release:
	sed "s/^\(version =\).*$$/\1 \"${version}\"/g" -i Cargo.toml
	make build
