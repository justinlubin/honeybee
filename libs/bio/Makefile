RELEASE = 8
USER_FILES = user-files

.PHONY: check
check:
	ty check

.PHONY: publish
publish: amd64 arm64
	docker push ghcr.io/justinlubin/hb-bio:$(RELEASE)-amd64
	docker push ghcr.io/justinlubin/hb-bio:$(RELEASE)-arm64

.PHONY: image-export
image-export: amd64 arm64
	docker save ghcr.io/justinlubin/hb-bio:$(RELEASE)-amd64 \
		> images/hb-bio-$(RELEASE)-amd64.tar.gz
	docker save ghcr.io/justinlubin/hb-bio:$(RELEASE)-arm64 \
		> images/hb-bio-$(RELEASE)-arm64.tar.gz

.PHONY: clean
clean:
	rm -f images/hb-bio-$(RELEASE)-amd64.tar.gz
	rm -f images/hb-bio-$(RELEASE)-arm64.tar.gz
	
.PHONY: amd64
amd64:
	docker buildx build \
		--platform linux/amd64 \
		--output type=docker \
	    -t ghcr.io/justinlubin/hb-bio:$(RELEASE)-amd64 \
	    --build-arg RELEASE=$(RELEASE) \
	    --build-arg COMMIT=$(shell git rev-parse --short HEAD) \
	    --build-arg USER_FILES=$(USER_FILES) \
	    .

.PHONY: arm64
arm64:
	docker buildx build \
		--platform linux/arm64 \
		--output type=docker \
	    -t ghcr.io/justinlubin/hb-bio:$(RELEASE)-arm64 \
	    --build-arg RELEASE=$(RELEASE) \
	    --build-arg COMMIT=$(shell git rev-parse --short HEAD) \
	    --build-arg USER_FILES=$(USER_FILES) \
	    .
