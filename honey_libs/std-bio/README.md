# Honeybee Standard Bio

This folder contains the materials for the Honeybee Standard Bio library,
including the necessary files to build the Honeybee Standard Bio Docker image.

These files are:

- `Dockerfile`: used to create Honeybee Standard Bio Docker image
- `Makefile`: commands for typechecking and creating the Docker image
- `environment`: files copied over to `/root/environment` in Docker image
- `playground`: scratch space for trying out ideas
- `pyproject.toml` uv project description for `std-bio.py` (not for the
  generated code, just for the development of the library!)
- `std-bio.py`: the Honeybee Standard Bio library
- `uv.lock`: uv-generated lockfile for above `pyproject.toml`

