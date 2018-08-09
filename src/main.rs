

extern crate fuse;
extern crate git2;
extern crate libc;
extern crate time;

mod filesystem;

use std::path::Path;

fn main() {


    let filesys = filesystem::GitFilesystem::new("test_repository","HEAD");
    let path = Path::new("./test_filesystem");
    fuse::mount(filesys,&path,&[]).unwrap();
}
