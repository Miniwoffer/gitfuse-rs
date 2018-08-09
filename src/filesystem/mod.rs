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

pub struct GitFilesystem<'collection> {
    repository : Repository,
    new_tree : Oid,
    commit_time : Timespec,
    referance : &'collection str,
    inods : Vec<String>,
    files : filesystem_entry::FilesystemEntry,
    ttl : i64,
}
impl<'collection> GitFilesystem<'collection> {
    pub fn new (repo_path : &str,referance : &'collection str) -> GitFilesystem<'collection> {
        let mut repository = match Repository::open(repo_path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };
        let mut commit_time;
        let mut new_tree;
        let mut files;
        let mut inods = Vec::new();
        {
            //TODO: might want to use as_commit() instead of peel_to_commit
            let curr_commit = repository.revparse_single(referance).unwrap().peel_to_commit().unwrap();
            let curr_tree = curr_commit.tree().unwrap();
            inods.push("".to_string());
            inods.push("".to_string());//filesys inode starts at 1, this is faster then to add and sub everytime.

            files = filesystem_entry::FilesystemEntry::from_tree(&curr_tree,&repository,"".to_string(),"".to_string(),&mut inods);

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
            inods,
            ttl : 10,
            files,
        }
    }

    fn get_attrs(&self, entry : &filesystem_entry::FilesystemEntry) -> FileAttr {
        //TODO: find out what we can get from entry.filemode()
        let mut file_attr = FileAttr {
            ino: entry.ino as u64,
            size: 0,
            blocks: 1,
            atime: self.commit_time,
            mtime: self.commit_time,
            ctime: self.commit_time,
            kind:  entry.file_type,
            perm: 0,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
            crtime: self.commit_time, // TODO: Get repository creation time maybe?
        };
        match entry.file_type {
            FileType::Directory => {
                file_attr.size = 4096;
                file_attr.nlink = 2;
            },
            FileType::RegularFile => {
                file_attr.size = entry.size;
            }
            _ => {},
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

impl<'collection> Filesystem for GitFilesystem<'collection> {

    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        //we construct elsewhere
        Ok(())
    }
    fn destroy(&mut self, _req: &Request) {
        self.commit();
    }
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let mut path = self.inods[parent as usize].clone();
        if !path.is_empty(){
            path = path + "/";
        }
        path = path + name.to_str().unwrap();
        println!("lookup:{}",path);
        let file = match self.files.get_path(path.as_str()) {
                Some(e) => e,
                None => {
                    reply.error(error_codes::ENOENT);
                    return;
                }
        };
        println!("lookup found {} = {}",path,file.name);
        let ttl = Timespec::new(self.ttl,0);
        let file_attr = self.get_attrs(file);
        reply.entry(&ttl,&file_attr, 0); // TODO: What does generation do?

    }
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        let path = &self.inods[ino as usize];
        println!("getattr:{}",path);
        let file = match self.files.get_path(path.as_str()) {
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
    fn mkdir(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        reply: ReplyEntry
    ) {
        let path = self.inods[parent as usize].clone();
        let name = match name.to_str() {
            Some(s) => s.to_string(),
            None => {
                reply.error(error_codes::EPERM); // TODO: invalid name error??
                return
            },
        };
        let new_file = filesystem_entry::FilesystemEntry::new(FileType::Directory,
                                                              name,
                                                              path.to_string(),
                                                              &mut self.inods);
        let file_attr = self.get_attrs(&new_file);
        let file = match self.files.get_path_mut(path.as_str()) {
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
        let path = self.inods[parent as usize].clone();
        let file = match self.files.get_path_mut(path.as_str()) {
            Some(e) => e,
            None => {
                reply.error(error_codes::ENOENT);
                return;
            }
        };
        match file.remove(name, FileType::Directory,&mut self.inods) {
            Ok(_) => reply.ok(),
            Err(e) => reply.error(e),
        };

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
        let old_dir = self.inods[parent as usize].clone();
        let new_dir = self.inods[newparent as usize].clone();

    }
    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory
    ) {
        let path = &self.inods[ino as usize];
        println!("readdir:{}",path);
        let folder = match self.files.get_path(path.as_str()) {
            Some(e) => e,
            None => {
                reply.error(error_codes::ENOENT);
                return;
            }
        };
        println!("dir:{}",folder.name);
        if offset == 0 {
            reply.add(ino, 0, FileType::Directory, ".");
            reply.add(ino, 1, FileType::Directory, "..");
        }
        for index in (offset as usize)..folder.children.len() {
            let file = &folder.children[index];
            let file_type = file.file_type;
            let file_name = file.name.clone();
            let fileatr = file.ino as u64;
            reply.add(fileatr ,(index+2) as i64,file_type,file_name);
        }
        reply.ok();
    }
    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData
    ) {
        let path = &self.inods[ino as usize];
        let oid = match self.files.get_path(path.as_str()) {
            Some(e) => match e.oid {
                Some(e) => e,
                None => {
                    reply.error(error_codes::ENOENT);
                    return;
                }
            },
            None => {
                reply.error(error_codes::ENOENT);
                return;
            }
        };
        match self.repository.find_blob(oid) {
            Ok(blob) => {
                let (_,content) = blob.content().split_at(offset as usize);
                if content.len() >= size as usize {
                    let (content, _) = content.split_at(size as usize);
                }
                reply.data(content);

            }
            Err(e) => {
                reply.error(error_codes::ENOENT);
            }

        }
    }
    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {

    }
    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _flags: u32,
        reply: ReplyWrite
    ) {

    }
}