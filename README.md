# unix

[![Build status](https://ci.appveyor.com/api/projects/status/4j1ho78h1npu9253?svg=true)](https://ci.appveyor.com/project/Babkock/unix) [![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FBabkock%2Funix.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2FBabkock%2Funix?ref=badge_shield)
[![](https://tokei.rs/b1/github/Babkock/unix)](https://github.com/XAMPPRocky/tokei)

This is a variety of GNU coreutils re-implemented in Rust. These examples aim to be efficient, fast, and less complicated.

To compile all of the programs, run the `build.sh` script. You can also compile any of the programs individually with

```
./build.sh programname
```

The compiled and stripped release binaries will be copied to the `build` directory. You can also build the documentation individually, or as a whole
using this script. `./build.sh doc` will build docs for all of the programs.

```
./build.sh doc programname
```

This will compile the docs for the specified program, and open them in your web browser.

