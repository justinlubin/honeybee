1. Bump version numbers for editor, engine, honey_lang, and honey_libs
2. Run `make` in `editor` to update lock files
3. Update CHANGELOG section
4. Push commit
5. Add version tag: `git tag -a vX.Y.Z -m "Version X.Y.Z"`
6. Push version tag `git push origin vX.Y.Z`
7. Run `./publish.sh new-version` in `editor`
