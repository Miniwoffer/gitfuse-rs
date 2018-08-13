#![cfg_attr(feature = "cargo-clippy", warn(clippy))]
#![cfg_attr(feature = "cargo-clippy", warn(clippy_pedantic))]

extern crate clap;
extern crate fuse;
extern crate git2;
extern crate libc;
extern crate time;

use clap::{App, Arg};

mod filesystem;
use std::path::Path;

fn main() {
    let args = App::new("git filesystem")
        .version("0.1.0")
        .arg(
            Arg::with_name("Repository path")
                .short("g")
                .long("git_path")
                .value_name("PATH")
                .help("Path to git repository")
                .takes_value(true)
                .required(true),
        ).arg(
            Arg::with_name("Git tag")
                .short("t")
                .long("tag")
                .value_name("STRING")
                .help("What tag the filesystem should start at eks: \"HEAD\",\"v1.0\"")
                .takes_value(true),
        ).arg(
            Arg::with_name("Mount point")
                .short("m")
                .long("mount_point")
                .value_name("PATH")
                .help("The path to where the filesystem will mount")
                .takes_value(true)
                .required(true),
        ).get_matches();

    let path = args.value_of("Repository path").unwrap();
    let git_tag = args.value_of("Git tag").unwrap_or("HEAD");
    let mount_point = args.value_of("Mount point").unwrap();

    {
        let filesys = filesystem::GitFilesystem::new(path, git_tag);
        let path = Path::new(mount_point);
        fuse::mount(filesys, &path, &[]).unwrap();
    }
    println!("Shutting down!");
}
