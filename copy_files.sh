rm -rf ./target/debug/resources

mkdir -p ./target/debug/resources/textures
mkdir -p ./target/debug/resources/shaders

cp -r ./resources/shaders/. ./target/debug/resources/shaders
cp -r ./resources/textures/. ./target/debug/resources/textures
