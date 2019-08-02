all: echo/target/release/echo

echo/target/release/echo:
	@printf '\e[33mBuilding echo\n'
	@cargo build --release --manifest-path echo/Cargo.toml 2> /dev/null
	@cd ..

