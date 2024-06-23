SRC_FILES := $(wildcard src/*.rs)
BUILD_DIR := target

RUSTC_FLAGS_DEV := --color always
RUSTC_FLAGS_RELEASE := --color always --release

.PHONY: all clean run_dev run_release

watch: 
	@cargo watch -c -w src -x run

clean:
	@cargo clean

run_dev:
	@cargo run 

run_release: $(SRC_FILES)
	@cargo run $(RUSTC_FLAGS_RELEASE)