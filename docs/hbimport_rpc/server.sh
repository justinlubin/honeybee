set -e

cd ../engine

cargo b --quiet

./target/debug/honeybee interact \
  --machine-readable \
  -l ../editor/www/bio.hblib.toml \
  ../editor/www/example.hb.toml
