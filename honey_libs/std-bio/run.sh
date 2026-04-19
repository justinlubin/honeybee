mkdir -p mnt

docker run -it \
    --mount type=bind,source="$(pwd)/mnt",target="/root/mnt" \
    std-bio
