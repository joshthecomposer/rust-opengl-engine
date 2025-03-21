rm -rf ./target/debug/resources

mkdir -p ./target/debug/resources
mkdir -p ./target/debug/debug_out
mkdir -p ./target/debug/config

cp -r ./resources/. ./target/debug/resources
cp -r ./libs/. ./target/debug
cp -r ./debug_out/. ./target/debug/debug_out
cp -r ./config/. ./target/debug/config


# Cleanup some files:
rm -rf debug_out
mkdir debug_out
