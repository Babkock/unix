#!/bin/bash
# Babkock/unix 
# GNU coreutils re-implemented in Rust.
#
# builder
# (c) 2019 Tanner Babcock. MIT License.
#
# How to use:
#
# $ ./builder             # build all programs
# $ ./builder program     # build 'program': cd into the project, build, and cd out
#
# $ ./builder doc         # build docs for all programs
# $ ./builder doc program # build docs for 'program' and open them
#
# $ ./builder test         # test all programs
# $ ./builder test program # test 'program' only
#
declare -a programs=("cat" "chown" "echo" "head" "ls")

# programs built with multiple source files and have a 'main' binary
declare -a multi_source_programs=("cat" "chown" "ls")

# programs that have tests written for them
declare -a tested_programs=("cat" "chown" "ls")

if [ -z $1 ]; then
	for i in "${programs[@]}"; do
		pushd "$i" > /dev/null
		printf "\e[93;1mBuilding %s\n" "$i"
		cargo build --release
		# Check whether the build succeeded or not, obviously don't continue if not
		if [ $? == 0 ]; then
			cp "target/release/main" ../build/$i 2> /dev/null
			if [ $? == 1 ]; then
				cp "target/release/$i" ../build/$i 2> /dev/null
			fi
			strip --strip-all ../build/$i
		else
			popd > /dev/null
			printf "\e[91;1mCrate '%s' failed to build\n" > /dev/stderr
			exit 1
		fi
		popd > /dev/null
	done
elif [ $1 == "doc" ]; then
	if [ -z $2 ]; then
		for i in "${programs[@]}"; do
			pushd "$i" > /dev/null
			printf "\e[93;1mBuilding docs for %s\n" "$i"
			cargo doc
			if [ $? != 0 ]; then
				popd > /dev/null
				printf "\e[91;1mDocs for crate '%s' failed to build\n" "$i" > /dev/stderr
				exit 1
			fi
			popd > /dev/null
		done
	else
		pushd "$2" > /dev/null
		printf "\e[93;1mBuilding docs for %s\n" "$2"
		cargo doc --open
		popd > /dev/null
	fi
elif [ $1 == "test" ]; then
	if [ -z $2 ]; then
		for i in "${tested_programs[@]}"; do
			pushd "$i" > /dev/null
			printf "\e[93;1mTesting %s\n" "$i"
			cargo test --lib
			if [ $? != 0 ]; then
				popd > /dev/null
				printf "\e[91;1mTests failed for %s\n" "$i" > /dev/stderr
				exit 1
			fi
			popd > /dev/null
		done
	else
		pushd "$2" > /dev/null
		printf "\e[93;1mTesting %s\n" "$2"
		cargo test --lib
		popd > /dev/null
	fi
else
	for i in "${programs[@]}"; do
		if [ $i == $1 ]; then
			pushd "$i" > /dev/null
			printf "\e[93;1mBuilding %s\n" "$i"
			cargo build --release
			if [ $? == 0 ]; then
				cp "target/release/main" ../build/$i
				if [ $? == 1 ]; then
					cp "target/release/$i" ../build
				fi
			else
				popd > /dev/null
				printf "\e[91;1m%s failed to build\n" "$i" > /dev/stderr
				exit 1
			fi
			popd > /dev/null
			exit 0
		fi
	done
fi

