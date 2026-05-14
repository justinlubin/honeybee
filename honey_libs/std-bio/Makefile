HB_VERSION = v0.6.1
USER_FILES = user-files

.PHONY: check
check:
	ty check

.PHONY: image-export
image-export:
	docker buildx build \
		--platform linux/amd64 \
		--output type=docker,dest=- \
	    -t std-bio \
	    --build-arg VERSION=$(HB_VERSION) \
	    --build-arg COMMIT=$(shell git rev-parse --short HEAD) \
	    --build-arg USER_FILES=$(USER_FILES) \
	    . \
	    | gzip > images/std-bio-amd64.tar.gz
	docker buildx build \
		--platform linux/arm64 \
		--output type=docker,dest=- \
	    -t std-bio \
	    --build-arg VERSION=$(HB_VERSION) \
	    --build-arg COMMIT=$(shell git rev-parse --short HEAD) \
	    --build-arg USER_FILES=$(USER_FILES) \
	    . \
	    | gzip > images/std-bio-arm64.tar.gz

.PHONY: clean
clean:
	rm -f images/std-bio-amd64.tar.gz
	rm -f images/std-bio-arm64.tar.gz

.PHONY: image
image:
	docker build \
	    -t std-bio \
	    --build-arg VERSION=$(HB_VERSION) \
	    --build-arg COMMIT=$(shell git rev-parse --short HEAD) \
	    --build-arg USER_FILES=$(USER_FILES) \
	    .

.PHONY: run
run:
	mkdir -p $(USER_FILES)
	docker run -it \
		-p 8888:8888 \
		--mount type=bind,source="$(shell pwd)/$(USER_FILES)",target="/root/$(USER_FILES)" \
		${RUN_ARGS} \
		std-bio

.PHONY: irun
irun:
	RUN_ARGS="--entrypoint /usr/bin/bash" make run
