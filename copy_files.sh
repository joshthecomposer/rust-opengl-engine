rm -rf ./target/debug/resources

# mkdir -p ./target/debug/resources/textures
# mkdir -p ./target/debug/resources/shaders
# mkdir -p ./target/debug/resources/models

cp -r ./resources/. ./target/debug/resources
cp -r ./libs/. ./target/debug
