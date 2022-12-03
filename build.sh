cargo build --release
mkdir -p build
cp ./target/release/krustens ./build
zip -r krustens.zip ./build
