rm -rf ./target/debug/resources

mkdir -p ./target/debug/resources

cp -r ./resources/. ./target/debug/resources
cp -r ./libs/. ./target/debug


# Cleanup some files:
rm -rf debug_out
mkdir debug_out
