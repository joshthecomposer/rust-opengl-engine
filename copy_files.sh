rm -rf ./target/debug/resources
rm -rf ./target/release/resources

mkdir -p ./target/debug/resources
mkdir -p ./target/debug/debug_out
mkdir -p ./target/debug/config

mkdir -p ./target/release/resources
mkdir -p ./target/release/debug_out
mkdir -p ./target/release/config

cp -r ./resources/. ./target/debug/resources
cp -r ./libs/. ./target/debug
cp -r ./debug_out/. ./target/debug/debug_out
cp -r ./config/. ./target/debug/config

cp -r ./resources/. ./target/release/resources
cp -r ./libs/. ./target/release
cp -r ./debug_out/. ./target/release/debug_out
cp -r ./config/. ./target/release/config

# Cleanup some files:
rm -rf debug_out
mkdir debug_out
