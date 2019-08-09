#!/bin/bash
# Babkock/unix 
# GNU coreutils re-implemented in Rust.
# build.sh
# (c) 2019 Tanner Babcock. MIT License.
#
# How to use:
#
# $ ./build.sh             # build all programs
# $ ./build.sh program     # build 'program': cd into the project, build, and cd out
# $ ./build.sh doc         # build docs for all programs
# $ ./build.sh doc program # build docs for 'program' and open them
#
declare -a programs=("cat" "echo" "head" "ls")
if [ -z $1 ]; then
	for i in "${programs[@]}"; do
		pushd "$i" > /dev/null
		printf "\e[93;1mBuilding %s\n" "$i"
		cargo build --release
		# Check whether the build succeeded or not, obviously don't continue if not
		if [ $? == 0 ]; then
			if [ $i == "cat" ]; then
				cp "target/release/main" ../build/cat
			elif [ $i == "ls" ]; then
				cp "target/release/main" ../build/ls
			else
				cp "target/release/$i" ../build
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
				printf "\e[91;1mDocs for crate '%s' failed to build\n" > /dev/stderr
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
else
	for i in "${programs[@]}"; do
		if [ $i == $1 ]; then
			pushd "$i" > /dev/null
			printf "\e[93;1mBuilding %s\n" "$i"
			cargo build --release
			if [ $? == 0 ]; then
				if [ $i == "cat" ]; then
					cp "target/release/main" ../build/cat
				elif [ $i == "ls" ]; then
					cp "target/release/main" ../build/ls
				else
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

