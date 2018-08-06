use git2::{Repository,Tree,Oid,ResetType,TreeBuilder,TreeEntry,Signature,Time,Blob};
use fuse::*;

use std::fs::File;
use std::io::Write;
use std::ffi::OsStr;
use std::path::Path;
use std::collections::VecDeque;

use libc::c_int;
use time::Timespec;
use time::SteadyTime;
use time::Duration;

// GENERAL TODO list
// Change strings to str with proper lifetimes
//


// TODO: Move inodeCollocion into a seperate mod
pub struct InodeEntry<'entry> {
    valid_until : SteadyTime,
    path : &'entry str,
}

pub struct InodeCollection<'a> {
    inodes : VecDeque<InodeEntry<'a>>,
    inodeoffset : u64,
    ttl : Duration,
}
impl<'a> InodeCollection<'a> {
    fn new (ttl : Option<usize>) -> InodeCollection<'a> {
        InodeCollection {
            inodes : VecDeque::new(),
            inodeoffset : 0u64,
            ttl : match ttl {
                Some(t) => Duration::seconds(t as i64),
                None => Duration::seconds(2i64),
            }
        }
    }
    fn clean(&mut self) {
        //removes all inodes that have expired
        let cur_time = SteadyTime::now();
        loop {
            match self.inodes.front() {
                Some(e) => if e.valid_until > cur_time {
                    break;
                },
                None => break,
            }
            self.inodes.pop_front();
            self.inodeoffset += 1;
        }

    }

    fn insert(&mut self, path : &'a str) -> u64 {

        self.inodes.push_back(InodeEntry {
            path,
            valid_until : SteadyTime::now()+self.ttl,
        });
        return self.inodeoffset + self.inodes.len() as u64;
    }
    fn remove(&mut self, inode : u64){
        // do nothing since removing an element in the middle will break the list
    }
    fn get(&mut self, inode : u64) -> Option<&str> {
        self.clean();
        match self.inodes.get((inode-self.inodeoffset) as usize) {
            Some(e) => Some(e.path),
            None => None,
        }
    }
}

pub struct FilesystemEntry<'a> {
    name : &'a str,
    original : boolean,
    file_type : FileType,
    oid : Option<Oid>,
    children : Vec<FilesystemEntry<'a>>,
}
pub enum Filetype {
    folder,
    file,
}
impl<'a> FilesystemEntry<'a> {
    pub fn new(file_type : FileType) -> FilesystemEntry<'a> {
        FilesystemEntry{
            original: false,
            file_type,
            oid : None,
            children : Vec::new(),

        }
    }
    pub fn fromTree(tree : &Tree, repo : &Repository, name : &'a str) -> FilesystemEntry<'a> {
        let mut children  = Vec::new();
        for entry in tree {
            children.push(File::fromEntry(entry,repo));
        }
        FilesystemEntry{
            name,
            original: true,
            file_type : FileType::folder,
            oid : tree.id,
            children,

        }
    }
    pub fn fromTreeEntry( treeEntry : &TreeEntry, repo : &Repository) -> FilesystemEntry<'a> {
        match treeEntry.kind() {
            Some(t) => {
                match t {
                    Tree => {
                        FilesystemEntry::fromTree(treeEntry.to_object(repo).unwrap().into_tree().unwrap(),
                                                  treeEntry.name().unwrap());
                    },
                    _ => {
                        FilesystemEntry{
                            name : treeEntry.name().unwrap(),
                            original: true,
                            file_type : FileType::file,
                            oid : treeEntry.id().unwrap(),
                            children : Vec::new(),
                        }

                    }
                }
            }
            None => {
                panic!("No file type found")
            }
        };
    }
}

pub struct GitFilesystem<'collection,'a>{
    repository : Repository,
    new_tree : Oid,
    commit_time : Timespec,
    referance : &'collection str,
    inods : InodeCollection<'a>,
    ttl : usize,
}
impl<'collection,'a> GitFilesystem<'collection,'a> {
    pub fn new (repo_path : &str,referance : &'collection str) -> GitFilesystem<'collection,'a> {
        let mut repository = match Repository::open(repo_path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };

        //TODO: might want to use as_commit() instead of peel_to_commit
        let curr_commit = repository.revparse_single(referance).unwrap().peel_to_commit().unwrap();
        let curr_tree = curr_commit.tree().unwrap();

        //Writes a copy of the current tree to git and saves the Oid, this is to hinder the original tree from getting deleted.
        let new_tree = repository.treebuilder(Some(&curr_tree)).unwrap().write().unwrap();

        //commit do not have nano seconds so sett it to 0
        let commit_time = Timespec::new(curr_commit.time().seconds(), 0);

        GitFilesystem {
            repository,
            new_tree,
            commit_time,
            referance,
            inods : InodeCollection::new(Some(10)),
            ttl : 10,
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
        match self.inods.get(parent) {
            Some(p) => {
                let path = Path::new((p.to_string()+"/"+name.to_str().unwrap()).as_str());
                match self.repository.find_tree( self.new_tree).unwrap().get_path(path) {
                    Ok(entry) => {
                        let fileAtr = self.getAttrs(entry);
                        fileAtr.ino = self.inods.insert(path.to_str().unwrap());
                        reply.entry(
                            &(Timespec::new(0,0)+self.inods.ttl),
                            & fileAtr,
                            0);
                    }
                    Err(e) => {
                        //TODO: Check if the error is "not found"
                        reply.error(2);
                    }
                }
            },
            None => {
                reply.error(2);
            },
        }
    }
    fn forget(&mut self, _req: &Request, _ino: u64, _nlookup: u64) {
        //Do nothing, we never forget a ino
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
        let mut new_tree_builder = self.getTree();
        let mut file_id : Oid;
        match new_tree_builder.get(name) {
            Ok(a) => {
                match a {
                    Some(e) => {
                        file_id = e.id();
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