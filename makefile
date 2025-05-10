build: 
	cargo build

run:
	cargo run

clean:
	cargo clean

update:
	rustup update stable

rebase:
	git origin rebase 

reset:
	git reset HEAD --hard