Install cargo with your distro's package manager
or
Use rustup
```
curl https://sh.rustup.rs -sSf | sh
```

Install git and fuse

Navigate git_fue_rs dir and run
```
cargo run
```
USAGE
```
git filesystem 0.1.0

USAGE:
    git-fuse [OPTIONS] --mount_point <PATH> --git_path <PATH>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --tag <STRING>          What tag the filesystem should start at eks: "HEAD","v1.0"
    -m, --mount_point <PATH>    The path to where the filesystem will mount
    -g, --git_path <PATH>       Path to git repository
```
