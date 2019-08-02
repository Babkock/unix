all: echo/target/release/echo head/target/release/head

echo/target/release/echo:
	@printf '\e[33mBuilding echo\n'
	@cargo build --release --manifest-path echo/Cargo.toml 2> /dev/null

head/target/release/head:
	@printf '\e[33mBuilding head\n'
	@cargo build --release --manifest-path head/Cargo.toml 2> /dev/null
	
