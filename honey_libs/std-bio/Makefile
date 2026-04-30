USER_FILES = user-files

.PHONY: check
check:
	ty check

.PHONY: image
image:
	docker build \
	    -t std-bio \
	    --build-arg VERSION=v0.6.0 \
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
