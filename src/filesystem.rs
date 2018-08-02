use git2::{Repository,Tree,Oid,ResetType,TreeBuilder,TreeEntry,Signature,Time};
use fuse::*;

use std::fs::File;
use std::io::Write;
use std::ffi::OsStr;
use std::path::Path;
use std::collections::BTreeMap;

use libc::c_int;
use time::Timespec;

pub struct GitFilesystem{
    repository : Repository,
    new_tree : Oid,
    commit_time : Timespec,
    referance : String,
    inods : BTreeMap<u64,String>,
    inodeBack : u64,
    inodeFront : u64,
}
impl GitFilesystem {
    fn new (referance : String) -> GitFilesystem {
        let mut repository = match Repository::open("test_repository") {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };

        //TODO: might want to use as_commit() instead of peel_to_commit
        let curr_commit = repository.revparse_single(referance.as_str()).unwrap().peel_to_commit().unwrap();
        let curr_tree = curr_commit.tree().unwrap();

        //Writes a copy of the current tree to git and saves the Oid, this is to hinder the original tree from getting deleted.
        let new_tree = repository.treebuilder(Some(&curr_tree)).unwrap().write().unwrap();

        //commit do not have nano seconds so sett it to 0
        let commit_time = time::Timespec::new(curr_commit.time().seconds(), 0);

        GitFilesystem {
            repository,
            new_tree,
            commit_time,
            referance,
            inods: BTreeMap::new(),
            inodeBack : 0u64,
            inodeFront : 0u64
        }
    }
    fn getTree(&mut self) -> TreeBuilder {
        self.repository.treebuilder(Some(&self.repository.find_tree(self.new_tree).unwrap()) ).unwrap()
    }
    fn setTree(&mut self, treebuilder : &TreeBuilder) {
        //TODO: print the new oid to a file
        self.new_tree =  treebuilder.write().unwrap();
    }

    fn getAttrs(&mut self, entry : TreeEntry) -> FileAttr {
        //TODO: find out what we can get from entry.filemode()
        let mut fileAtr = FileAttr {
            ino: 0,
            size: 0,
            blocks: 1,
            atime: self.commit_time,
            mtime: self.commit_time,
            ctime: self.commit_time,
            kind:  FileType::RegularFile,
            perm: 0,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
            crtime: self.commit_time, // TODO: Get repository creation time maybe?
        };
        match entry.kind() {
            Some(t) => {
                match t {
                    Tree => {
                        fileAtr.kind = FileType::Directory;
                        fileAtr.size = 4096;
                        fileAtr.nlink = 2;
                    },
                    Blob => {
                        fileAtr.size = entry.to_object(&self.repository).unwrap().as_blob().unwrap().content().len() as u64;
                    },
                    _ => {
                        // Should only be a Tree or a Blob, but i guess we default to doing nothing
                        // TODO : might want to panic here, since there shouldn't be a commit,tag or any in here.

                    }
                }
            }
            None => {
                panic!("No file type found")
            }
        };
        fileAtr
    }

    fn commit(&mut self) {
        let last_commit = repository.revparse_single(referance.as_str()).unwrap().peel_to_commit().unwrap();
        let tree = self.repository.find_tree(self.new_tree).unwrap();
        let sign = Signature::now("git-fs","").unwrap();

        //TODO: Do we update the ref? if not we need to find another way to get "last_commit"
        self.repository.commit(referance.as_str(),
                               &sign,
                               &sign,
                               "Automated commit from git-fs",
                               &tree,
                               last_commit);
    }

    fn get_new_inode() {

    }
}

impl Filesystem for GitFilesystem {

    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        //we construct elsewhere
        Ok(())
    }
    fn destroy(&mut self, _req: &Request) {
        self.commit();
    }
    fn lookup(&mut self, _req: &Request, _parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.repository.find_tree(self.new_tree).unwrap().get_path(name) {
            Ok(o) => match o {
                Some(entry) => {
                    let fileAtr = self.getAttrs(entry);
                    //TODO: what should ttl be?
                    reply.entry(&Timespec::new(1,0,),&fileAtr,0);
                }
                None => {
                    reply.error(2);
                }
            }
            Err(e) => {
                panic!(e);
            }

        }
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
        //do nothing unless we can sett the repository's head, so we can access the index
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
    ) {


    }
    fn mkdir(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        reply: ReplyEntry
    ) {
        //cant create a empty dir, so make a dummy file.


    }
    fn unlink(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        reply: ReplyEmpty
    ) {
        // git does not support links.
    }
    fn rmdir(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        reply: ReplyEmpty
    ) {

    }
    fn symlink(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _link: &Path,
        reply: ReplyEntry
    ) {
        // git does not support links.
    }
    fn rename(
        &mut self,
        _req: &Request,
        _parent: u64,
        name: &OsStr,
        _newparent: u64,
        newname: &OsStr,
        reply: ReplyEmpty
    ) {
        let new_tree_builder = self.getTree();

        match new_tree_builder.get(name) {
            Ok(a) => {
                match a {
                    Some(e) => {
                        new_tree_builder.remove(name).unwrap();// panic on error
                        match new_tree_builder.insert(newname,e.id(),e.filemode()) {
                            Ok(entry) => {
                                reply.ok();
                            }
                            Err(e) => {
                                println!("{}",e); // TODO: find what errors exist
                                reply.error(17); // lets just say that the file already exists for now
                            }
                        }
                    }
                    None => {
                        reply.error(2); //no such file
                    }
                }

            }
            Err(e) => {
                panic!(e);
            }
        };
    }
    fn link(
        &mut self,
        _req: &Request,
        _ino: u64,
        _newparent: u64,
        _newname: &OsStr,
        reply: ReplyEntry
    ) {
        // git does not support links.
    }
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
    ) {

    }
    fn write(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _data: &[u8],
        _flags: u32,
        reply: ReplyWrite
    ) {

    }
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