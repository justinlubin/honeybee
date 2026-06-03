# Honeybee Biology Library

This folder contains the materials for the Honeybee Biology Library,
including the necessary files to build the Honeybee Biology Image (a
Docker/podman image that contains all the dependencies of the Honeybee Biology
Library pre-installed).

These files are:

- `image-files/`: files copied over to the image
- `Dockerfile`: used to create Honeybee Biology Image
- `Makefile`: commands for typechecking and creating the Honeybee Biology Image
- `bio.py`: the Honeybee Biology Library
- `playground`: scratch space for trying out ideas
- `pyproject.toml` uv project description for `bio.py` (not for the generated
  code, just for the development of the library!)
- `uv.lock`: uv-generated lockfile for above `pyproject.toml`

The image files in `image-files` are:
- `edit-warning.md`: Warning to move to the root of the Honeybee Biology Image
- `entrypoint.sh` script run on Honeybee Biology Container start
- `environment`: files copied over to `/root/environment` in Honeybee Biology
  Image (and in the `environment` folder of the user-files directory if such a
  folder does not already exist)

## Some tips for using the generated Jupyter notebooks

- Enable scrolling for output cells
- Use table of contents view
