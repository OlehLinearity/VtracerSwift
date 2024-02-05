#!/bin/bash

# We need the SDK Root
export SDKROOT=`xcrun --sdk macosx --show-sdk-path`

echo "Building for iOS X86_64 (Simulator)..."
cargo build --target x86_64-apple-ios --release > /dev/null 2>&1

echo "Building for iOS arm64 (Simulator)..."
cargo +nightly build -Z build-std --target aarch64-apple-ios-sim --release > /dev/null 2>&1

echo "Building for Mac Catalyst X86_64..."
cargo +nightly build -Z build-std --target x86_64-apple-ios-macabi --release > /dev/null 2>&1

echo "Building for Mac Catalyst ARM64..."
cargo +nightly build -Z build-std --target aarch64-apple-ios-macabi --release > /dev/null 2>&1

echo "Building for ARM iOS..."
cargo build --target aarch64-apple-ios --release > /dev/null 2>&1


# Build Fat Libraries:ю
# lipo together the different architectures into a universal 'fat' file

BUILD_DIR=build
mkdir -p $BUILD_DIR/fat

echo "Building Fat Libaries"
lipo -create -output $BUILD_DIR/fat/libvtracer-catalyst.a target/{aarch64-apple-ios-macabi,x86_64-apple-ios-macabi}/release/libvtracer.a
lipo -create -output $BUILD_DIR/fat/libvtracer-sim.a target/{aarch64-apple-ios-sim,x86_64-apple-ios}/release/libvtracer.a
lipo -create -output $BUILD_DIR/fat/libvtracer-ios.a target/aarch64-apple-ios/release/libvtracer.a

# Building XCFramework for static libraries
xcodebuild -create-xcframework \
           -library $BUILD_DIR/fat/libvtracer-ios.a -headers vtracer-ffi \
           -library $BUILD_DIR/fat/libvtracer-sim.a -headers vtracer-ffi \
           -library $BUILD_DIR/fat/libvtracer-catalyst.a -headers vtracer-ffi \
           -output $BUILD_DIR/xcframeworks/Vtracer.xcframework


# Building XCFramework for VtracerSwift
xcodebuild archive -project VtracerSwift/VtracerSwift.xcodeproj -scheme VtracerSwift -sdk iphoneos -archivePath ./$BUILD_DIR/archives/ios.xcarchive SKIP_INSTALL=NO BUILD_LIBRARIES_FOR_DISTRIBUTION=YES > /dev/null 2>&1
xcodebuild archive -project VtracerSwift/VtracerSwift.xcodeproj -scheme VtracerSwift -sdk iphonesimulator -archivePath ./$BUILD_DIR/archives/ios-simulator.xcarchive SKIP_INSTALL=NO BUILD_LIBRARIES_FOR_DISTRIBUTION=YES > /dev/null 2>&1
xcodebuild archive -project VtracerSwift/VtracerSwift.xcodeproj -scheme VtracerSwift -destination 'platform=macOS,arch=x86_64,variant=Mac Catalyst' -archivePath ./$BUILD_DIR/archives/maccatalyst.xcarchive SKIP_INSTALL=NO BUILD_LIBRARIES_FOR_DISTRIBUTION=YES > /dev/null 2>&1

xcodebuild -create-xcframework \
    -archive $BUILD_DIR/archives/ios-simulator.xcarchive -framework VtracerSwift.framework \
    -archive $BUILD_DIR/archives/ios.xcarchive -framework VtracerSwift.framework \
    -archive $BUILD_DIR/archives/maccatalyst.xcarchive -framework VtracerSwift.framework \
    -output $BUILD_DIR/xcframeworks/VtracerSwift.xcframework