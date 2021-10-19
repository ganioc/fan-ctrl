#! /bin/sh
cross build --target aarch64-unknown-linux-gnu --release

bin="target/aarch64-unknown-linux-gnu/release/dev-monitor"
strip $bin

mkdir -p build-deb/usr/local/bin
cp $bin build-deb/usr/local/bin/

dpkg-deb --root-owner-group --build ./build-deb dev-monitor.deb
