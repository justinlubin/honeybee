# Honeybee Standard Bio

This folder contains the materials for the Honeybee Standard Bio library,
including the necessary files to build the Honeybee Standard Bio Docker image.

These files are:

- `Dockerfile`: used to create Honeybee Standard Bio Docker image
- `Makefile`: commands for typechecking and creating the Docker image
- `bio.py`: the Honeybee Biology library
- `edit-warning.md`: Warning to move to the root of the Docker image
- `entrypoint.sh` script run on Docker container start
- `environment`: files copied over to `/root/environment` in Docker image (and
  the `environment` folder of the user files directory if such a folder does
  not already exist)
- `playground`: scratch space for trying out ideas
- `pyproject.toml` uv project description for `bio.py` (not for the generated
  code, just for the development of the library!)
- `uv.lock`: uv-generated lockfile for above `pyproject.toml`

## Some tips for using the generated Jupyter notebooks

- Enable scrolling for output cells
- Use table of contents view
