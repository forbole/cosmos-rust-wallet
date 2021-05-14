current_dir := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
uid := $(shell id -u)
guid := $(shell id -g)
rust_version := 1.52.1
osx_sdk := 11.1
ios_sdk := 14.4
android_ndk := r21e

clean:
	rm -Rf $(current_dir)/target
	rm -Rf $(current_dir)/pkg

linux:
	@echo "Building crw-wallet for linux"
	docker run -u $(uid):$(guid) --rm -v $(current_dir):/workdir forbole/rust-builder:$(rust_version) \
		cargo build --release --target=x86_64-unknown-linux-gnu --features ffi

windows:
	@echo "Building crw-wallet for windows"
	docker run -u $(uid):$(guid) --rm -v $(current_dir):/workdir forbole/windows-rust-builder:$(rust_version) \
		cargo build --release --target=x86_64-pc-windows-gnu --features ffi

osx:
	@echo "Building crw-wallet for mac"
	docker run -u $(uid):$(guid) --rm -v $(current_dir):/workdir forbole/osx-rust-builder:$(rust_version)-$(osx_sdk) \
		cargo build --release --target=x86_64-apple-darwin --features ffi

android-aarch64:
	@echo "Building crw-wallet for android-aarch64"
	docker run -u $(uid):$(guid) --rm -v $(current_dir):/workdir forbole/android-rust-builder:$(rust_version)-$(android_ndk) \
    	cargo build --release --target=aarch64-linux-android --features ffi

android-armv7:
	@echo "Building crw-wallet for android-armv7"
	docker run -u $(uid):$(guid) --rm -v $(current_dir):/workdir forbole/android-rust-builder:$(rust_version)-$(android_ndk) \
        	cargo build --release --target=armv7-linux-androideabi --features ffi

android-x86_64:
	@echo "Building crw-wallet for android-x86_64 (Emulator)"
	docker run -u $(uid):$(guid) --rm -v $(current_dir):/workdir forbole/android-rust-builder:$(rust_version)-$(android_ndk) \
        	cargo build --release --target=x86_64-linux-android --features ffi

android: android-armv7 android-aarch64 android-x86_64

ios-aarch64:
	@echo "Building crw-wallet for iOS aarch64"
	docker run -u $(uid):$(guid) -e IOS_ARCH=arm64 --rm -v $(current_dir):/workdir forbole/ios-rust-builder:$(rust_version)-$(ios_sdk) \
            	cargo build --release --target=aarch64-apple-ios --features ffi

ios-x86_64:
	@echo "Building crw-wallet for iOS x86_64 (Emulator)"
	docker run -u $(uid):$(guid) -e IOS_ARCH=x86_64 --rm -v $(current_dir):/workdir forbole/ios-rust-builder:$(rust_version)-$(ios_sdk) \
            	cargo build --release --target=x86_64-apple-ios --features ffi

ios: ios-aarch64 ios-x86_64

wasm:
	@echo "Building crw-wallet for web"
	docker run -u $(uid):$(guid) --rm -v $(current_dir):/workdir forbole/wasm-rust-builder:$(rust_version) \
		wasm-pack build --release -- --features wasm-bindgen -v

all: linux windows osx android ios wasm