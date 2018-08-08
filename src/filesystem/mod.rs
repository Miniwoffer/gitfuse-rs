mod inode;
mod filesystem_entry;
pub mod error_codes;

use git2::{Repository,Oid,Signature};
use fuse::*;

use std::io::Write;
use std::ffi::OsStr;
use std::path::Path;


use std::os::raw::c_int;
use time::Timespec;

// TODO: Check all error codes

pub struct GitFilesystem<'collection,'a> {
    repository : Repository,
    new_tree : Oid,
    commit_time : Timespec,
    referance : &'collection str,
    inods : inode::InodeCollection<'a>,
    files : filesystem_entry::FilesystemEntry,
    ttl : i64,
}
impl<'collection,'a> GitFilesystem<'collection,'a> {
    pub fn new (repo_path : &str,referance : &'collection str) -> GitFilesystem<'collection,'a> {
        let mut repository = match Repository::open(repo_path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };
        let mut commit_time;
        let mut new_tree;
        let mut files;
        {
            //TODO: might want to use as_commit() instead of peel_to_commit
            let curr_commit = repository.revparse_single(referance).unwrap().peel_to_commit().unwrap();
            let curr_tree = curr_commit.tree().unwrap();
            files = filesystem_entry::FilesystemEntry::from_tree(&curr_tree,&repository,"root".to_string());

            //Writes a copy of the current tree to git and saves the Oid, this is to hinder the original tree from getting deleted.
            new_tree = repository.treebuilder(Some(&curr_tree)).unwrap().write().unwrap();

            //commit do not have nano seconds so sett it to 0
            commit_time = Timespec::new(curr_commit.time().seconds(), 0);
        }

        GitFilesystem {
            repository,
            new_tree,
            commit_time,
            referance,
            inods : inode::InodeCollection::new(Some(10)),
            ttl : 10,
            files,
        }
    }

    fn get_attrs(&self, entry : &filesystem_entry::FilesystemEntry) -> FileAttr {
        //TODO: find out what we can get from entry.filemode()
        let mut file_attr = FileAttr {
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
        match entry.file_type {
            filesystem_entry::EntryFiletype::folder => {
                file_attr.kind = FileType::Directory;
                file_attr.size = 4096;
                file_attr.nlink = 2;
            },
            filesystem_entry::EntryFiletype::file => {
                file_attr.size = entry.size;
            }
        };
        file_attr
    }

    fn commit(&mut self) {
        let last_commit = self.repository.revparse_single(self.referance).unwrap().peel_to_commit().unwrap();
        let tree = self.repository.find_tree(self.new_tree).unwrap();
        let sign = Signature::now("git-fs","").unwrap();

        //TODO: Do we update the ref? if not we need to find another way to get "last_commit"
        self.repository.commit(Some(self.referance),
                               &sign,
                               &sign,
                               "Automated commit from git-fs",
                               &tree,
                               &[&last_commit]);
    }
}

impl<'collection,'a> Filesystem for GitFilesystem<'collection,'a> {

    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        //we construct elsewhere
        Ok(())
    }
    fn destroy(&mut self, _req: &Request) {
        self.commit();
    }
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let path = match self.inods.get(parent) {
                Some(e) => e.to_string() + name.to_str().unwrap(),
                None => {
                    reply.error(error_codes::EBADF);
                    return
                },
            };
        let file = match self.files.get_path(path.as_str()) {
                Some(e) => e,
                None => {
                    reply.error(error_codes::ENOENT);
                    return;
                }
            };
        let ttl = Timespec::new(self.ttl,0);
        let file_attr = self.get_attrs(file);
        reply.entry(&ttl,&file_attr, 0); // TODO: What does generation do?

    }
    fn forget(&mut self, _req: &Request, _ino: u64, _nlookup: u64) {
        //Do nothing, we never forget a ino. It would break collection
    }
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        let path = match self.inods.get(ino) {
            Some(e) => e,
            None => {
                reply.error(error_codes::EBADF);
                return
            },
        };
        let file = match self.files.get_path(path) {
            Some(e) => e,
            None => {
                reply.error(error_codes::ENOENT);
                return;
            }
        };
        let ttl = Timespec::new(self.ttl,0);
        let file_attr = self.get_attrs(file);
        reply.attr(&ttl,&file_attr);
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
        parent: u64,
        name: &OsStr,
        _mode: u32,
        reply: ReplyEntry
    ) {
        let name = match name.to_str() {
            Some(s) => s.to_string(),
            None => {
                reply.error(error_codes::EPERM); // TODO: invalid name error??
                return
            },
        };
        let new_file = filesystem_entry::FilesystemEntry::new(filesystem_entry::EntryFiletype::folder, name);
        let file_attr = self.get_attrs(&new_file);
        let path = match self.inods.get(parent) {
            Some(e) => e,
            None => {
                reply.error(error_codes::EBADF);
                return
            },
        };
        let file = match self.files.get_path_mut(path) {
            Some(e) => e,
            None => {
                reply.error(error_codes::ENOENT);
                return;
            }
        };
        match file.add(new_file) {
            Some(e) => e,
            None => {
                reply.error(error_codes::EEXIST);
                return
            },
        };
        let ttl = Timespec::new(self.ttl,0);
        reply.entry(&ttl,&file_attr,0);
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
        parent: u64,
        name: &OsStr,
        reply: ReplyEmpty
    ) {
        let name = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(error_codes::EPERM); // TODO: invalid name error??
                return
            },
        };
        let path = match self.inods.get(parent) {
            Some(e) => e,
            None => {
                reply.error(error_codes::EBADF);
                return
            },
        };
        let file = match self.files.get_path_mut(path) {
            Some(e) => e,
            None => {
                reply.error(error_codes::ENOENT);
                return;
            }
        };
        match file.remove(name, filesystem_entry::EntryFiletype::folder) {
            Ok(_) => reply.ok(),
            Err(e) => reply.error(e),
        };

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
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        reply: ReplyEmpty
    ){
        self.inods.clean();
        let old_dir = match self.inods.get(parent) {
                Some(e) => e,
                None => {
                    reply.error(2);
                    return
                },
            };
        let new_dir = match self.inods.get(newparent) {
            Some(e) => e,
            None => {
                reply.error(2);
                return
            },
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