# Buidling VtracerSwift

## Prerequisites

* rustup -- `curl https://sh.rustup.rs -sSf | sh`

* Add nightly toolchains source and install toolchain per each platform

```bash

rustup update
rustup toolchain install nightly
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add x86_64-apple-ios-macabi
rustup target add aarch64-apple-ios-macabi

```

## Building

* Run `bash make_xcframework.sh`
* XCFramework can be found in `build/xcframeworks`
