.PHONY: help analyze decode

PROG="cargo run --release"

help:
	@echo "Decoder will try caesar cipher first. If that doesn't work, it will try breaking it as a simple substitution cipher."
	@echo "See README.md for more info"
	@echo "make analyze ARGS=\"filename\"
	@echo "make decode ARGS=\"samples/sample1.txt\"
	@echo "make decode ARGS=\"samples/sample3.txt 1500\"

decode:
	cargo run --release decode $(ARGS)

analyze:
	cargo run --release analyze $(ARGS)

compile:
	cargo build --release