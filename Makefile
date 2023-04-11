build:
	@cargo build

clean:
	@cargo clean

TESTS = ""
test:
	@cargo test $(TESTS) --offline -- --color=always --test-threads=1 --nocapture

mongostart:
	@sudo docker run -d -p 27017:27017 -v `pwd`/data/db:/data/db --name ethblockchain mongo

mongostop:
	@sudo docker stop ethblockchain && sudo docker rm ethblockchain

docs: build
	@cargo doc --no-deps

lint:
	@rustup component add clippy 2> /dev/null
	touch src/**
	cargo clippy --all-targets --all-features -- -D warnings

dev:
	@cargo run

.PHONY: build test docs lint