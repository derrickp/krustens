cargo build --release
mkdir -p build
cp -R ./resources ./build
cp ./target/release/krustens ./build
zip -r krustens.zip ./build
