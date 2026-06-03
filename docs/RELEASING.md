**Note on versioning:** Everything is unversioned (pinned to 0.7.0) _except for_
the Honeybee Biology Library and Honeybee Biology Container. The versioning
scheme for those two are in lockstep and start go "Release 8", "Release 9", etc.
for _any_ possible change to the output scripts or environment, including new
or different:
- Cell/codegen
- Building blocks
- Environment (defined via the Dockerfile)
- IO behavior of Honey or Honeybee engine
and so on.

# To release

If there is a change in the Honeybee Biology Release (see above), take the
following steps (otherwise skip):

1. Bump release numbers in the following files:
  - `libs/bio/Makefile`
  - `libs/bio/bio.sh`
  - `libs/bio/launch-notebook.sh`
2. Run `make publish` in `libs/bio`

Then, to publish the latest editor/engine etc. to the web:

1. Run `make clean && make` in `editor`
2. Make and push a commit
3. Run `publish.sh` in `editor` to publish an unstable build;
   OR run `publish.sh stable` in `editor` to publish a stable build
