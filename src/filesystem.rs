use git2::{Repository,Tree,Oid,ResetType,TreeBuilder};
use fuse::*;

use std::fs::File;
use std::io::Write;
use std::ffi::OsStr;
use std::path::Path;


use libc::c_int;
use time::Timespec;



pub struct GitFilesystem<'a>{
    repository : Repository,
    new_tree : TreeBuilder<'a>,
}
impl GitFilesystem {
    pub fn new(directory : &str, tag : &str) -> GitFilesystem {
        let repository = match Repository::open("test_repository") {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };
        {
            let target = match repository.revparse_single(tag) {
                Ok(a) => a,
                Err(e) => panic!("Failed to open tag:{}", e)
            };
            repository.reset(&target, ResetType::Hard, None);
        }
        let tree_builder = repository.treebuilder();
        GitFilesystem {
            repository,
        }
    }
}

impl Filesystem for GitFilesystem {

    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        Ok(())
    }
    fn destroy(&mut self, _req: &Request) {

    }
    fn lookup(&mut self, _req: &Request, _parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.repository.index() {
            Ok(index) => match index.get_path( Path::new(name),0) {
                Some(entry)  =>{
                    let fileAtr = FileAttr {
                        ino: entry.ino as u64,
                        size: entry.file_size as u64,
                        blocks: 1,
                        atime: Timespec::new(entry.mtime.seconds() as i64,
                                             entry.mtime.nanoseconds()  as i32),
                        mtime: Timespec::new(entry.mtime.seconds() as i64,
                                             entry.mtime.nanoseconds() as i32),
                        ctime: Timespec::new(entry.ctime.seconds() as i64,
                                             entry.ctime.nanoseconds() as i32),
                        kind: FileType::RegularFile,
                        perm: 0,
                        nlink: 0,
                        uid: entry.uid,
                        gid: entry.gid,
                        rdev: entry.dev,
                        flags: 0,
                        crtime: Timespec::new(0,0),
                    };
                    reply.entry(&Timespec::new(1,0,),&fileAtr,0);
                },
                None => {
                    reply.error(1)
                }
            },
            Err(e) => panic!("Failed to find a tree:{}",e),
        };
    }
    fn forget(&mut self, _req: &Request, _ino: u64, _nlookup: u64) {

    }
    fn getattr(&mut self, _req: &Request, _ino: u64, reply: ReplyAttr) {

    }
    fn setattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<Timespec>,
        _mtime: Option<Timespec>,
        _fh: Option<u64>, _crtime: Option<Timespec>,
        _chgtime: Option<Timespec>,
        _bkuptime: Option<Timespec>, _flags: Option<u32>,
        reply: ReplyAttr) {

    }
    fn readlink(&mut self, _req: &Request, _ino: u64, reply: ReplyData) {

    }
    fn mknod(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _rdev: u32,
        reply: ReplyEntry
    ) {  }
    fn mkdir(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        reply: ReplyEntry
    ) {  }
    fn unlink(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        reply: ReplyEmpty
    ) {  }
    fn rmdir(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        reply: ReplyEmpty
    ) {  }
    fn symlink(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _link: &Path,
        reply: ReplyEntry
    ) {  }
    fn rename(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _newparent: u64,
        _newname: &OsStr,
        reply: ReplyEmpty
    ) {  }
    fn link(
        &mut self,
        _req: &Request,
        _ino: u64,
        _newparent: u64,
        _newname: &OsStr,
        reply: ReplyEntry
    ) {  }
    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {

    }
    fn read(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _size: u32,
        reply: ReplyData
    ) {  }
    fn write(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _data: &[u8],
        _flags: u32,
        reply: ReplyWrite
    ) {  }
    fn flush(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        reply: ReplyEmpty
    ) {  }
    fn release(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _flags: u32,
        _lock_owner: u64,
        _flush: bool,
        reply: ReplyEmpty
    ) {  }
    fn fsync(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _datasync: bool,
        reply: ReplyEmpty
    ) {  }
    fn opendir(
        &mut self,
        _req: &Request,
        _ino: u64,
        _flags: u32,
        reply: ReplyOpen
    ) {  }
    fn readdir(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        reply: ReplyDirectory
    ) {  }
    fn releasedir(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _flags: u32,
        reply: ReplyEmpty
    ) {  }
    fn fsyncdir(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _datasync: bool,
        reply: ReplyEmpty
    ) {  }
    fn statfs(&mut self, _req: &Request, _ino: u64, reply: ReplyStatfs) {  }
    fn setxattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _name: &OsStr,
        _value: &[u8],
        _flags: u32,
        _position: u32,
        reply: ReplyEmpty
    ) {  }
    fn getxattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _name: &OsStr,
        _size: u32,
        reply: ReplyXattr
    ) {  }
    fn listxattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _size: u32,
        reply: ReplyXattr
    ) {  }
    fn removexattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _name: &OsStr,
        reply: ReplyEmpty
    ) {  }
    fn access(
        &mut self,
        _req: &Request,
        _ino: u64,
        _mask: u32,
        reply: ReplyEmpty
    ) {  }
    fn create(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _flags: u32,
        reply: ReplyCreate
    ) {  }
    fn getlk(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: u32,
        _pid: u32,
        reply: ReplyLock
    ) {  }
    fn setlk(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: u32,
        _pid: u32,
        _sleep: bool,
        reply: ReplyEmpty
    ) {  }
    fn bmap(
        &mut self,
        _req: &Request,
        _ino: u64,
        _blocksize: u32,
        _idx: u64,
        reply: ReplyBmap
    ) {  }
}