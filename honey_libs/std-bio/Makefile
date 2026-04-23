USER_FILES = user-files

.PHONY: check
check:
	ty check

.PHONY: image
image:
	docker build -t std-bio .

.PHONY: run
run:
	mkdir -p $(USER_FILES)
	docker run \
		-p 8888:8888 \
		--mount type=bind,source="$(shell pwd)/$(USER_FILES)",target="/root/$(USER_FILES)" \
		std-bio \
		uv run jupyter lab \
			--no-browser \
			--ip 0.0.0.0 \
			--port 8888 \
			--notebook-dir user-files \
			--allow-root

.PHONY: irun
irun:
	mkdir -p $(USER_FILES)
	docker run -it \
		-p 8888:8888 \
		--mount type=bind,source="$(shell pwd)/$(USER_FILES)",target="/root/$(USER_FILES)" \
		std-bio
