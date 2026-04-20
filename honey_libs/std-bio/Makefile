.PHONY: check image run

check:
	ty check

image:
	docker build -t std-bio .

run:
	mkdir -p mnt
	docker run -it \
		--mount type=bind,source="$(shell pwd)/mnt",target="/root/mnt" \
		std-bio
