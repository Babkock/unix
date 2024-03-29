# unix

## This repository is archived. [It has moved to GitLab.](https://gitlab.com/tbcargo/unix)

[![Build Status](https://travis-ci.org/Babkock/unix.svg?branch=master)](https://travis-ci.org/Babkock/unix) [![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FBabkock%2Funix.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2FBabkock%2Funix?ref=badge_shield)

This is a variety of GNU coreutils re-implemented in Rust. These examples aim to be efficient, fast, and less complicated.

To compile all of the programs, run the `builder` script with no arguments. You can also compile any of the programs individually with

```
$ ./builder program
```

The compiled and stripped release binaries will be copied to the `build` directory. You can also build the documentation individually, or as a whole
using this script. `./builder doc` will build docs for all of the programs.

```
$ ./builder doc program
Building docs for program
...
```

This will compile the docs for the specified program, and open them in your web browser. The `builder` script is also capable of running unit
tests for any of the programs.

```
$ ./builder test program
Testing program
...
```

Like with doc, `./builder test` will compile and test the whole group.

