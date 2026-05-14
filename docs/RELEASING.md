1. Bump version numbers in the following files:
  - `editor/src/Version/elm`
  - `engine/Cargo.toml`
  - `honey_lang/pyproject.toml`
  - `honey_libs/fuseflow/pyproject.toml`
  - `honey_libs/std-bio/pyproject.toml`
  - `honey_libs/std-bio/Makefile`
  - `honey_libs/std-bio/hb-bio.sh`
2. Run `make` in `editor` to update lock files
3. Update CHANGELOG section
4. Push commit
5. Add version tag: `git tag -a vX.Y.Z -m "Version X.Y.Z"`
6. Push version tag `git push origin vX.Y.Z`
7. Run `./publish.sh new-version` in `editor`
