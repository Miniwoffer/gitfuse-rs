use git2::{Tree,Oid,Repository,TreeEntry};
use std::str::Split;
use std::os::raw::c_int;
use filesystem::error_codes;

use fuse::FileType;
#[derive(PartialEq)]
pub struct FilesystemEntry {
    pub name : String,
    pub file_type : FileType,
    pub oid : Option<Oid>,
    pub ino : usize,
    pub children : Vec<FilesystemEntry>,
    pub size : u64,
    pub file_mode : i32,

    //write functionality
    pub content : Option<Vec<u8>>,
    pub write : bool,
    pub write_mode : u32,

}
struct Git_entry {
    pub oid : Oid,
    pub name : String,
    pub file_mode : i32,
}
impl FilesystemEntry {
    pub fn new(file_type : FileType, name : String, path : String, inodes : &mut Vec<String>) -> FilesystemEntry {
        inodes.push(path+"/"+name.as_str());
        FilesystemEntry {
            name,
            file_type,
            oid: None,
            ino: inodes.len() - 1,
            children: Vec::new(),
            size: 0u64,
            content: match file_type {
                FileType::RegularFile => Some(Vec::new()),
                _ => None,
            },
            write: false,
            write_mode : 0,
            file_mode : 0,
        }
    }
    pub fn add(&mut self, file : FilesystemEntry) -> Option<&FilesystemEntry>{
        match self.index(file.name.as_str()) {
            Some(t) => return None,
            None => {},
        }
        self.children.push(file);
        Some(self.children.last().unwrap())
    }
    pub fn remove(&mut self, name : &str, file_type : FileType, inodes : &mut Vec<String>) -> Result<(),c_int> {
        let mut index = None;
        for i in 0..self.children.len() {
            if self.children[i].name == name {
                index = Some(i);
            }
        }
        let index = match index {
            Some(t) => t,
            None => return Err(error_codes::ENOENT),
        };
        if self.children[index].file_type == file_type {
            inodes.remove(self.children[index].ino);
            self.children.remove(index);
            return Ok(());
        } else {
            return Err(error_codes::ENOTDIR);
        }

    }
    pub fn get_path(&self , path : &str ) -> Option<&FilesystemEntry> {
        let mut path = path.to_owned();
        if path.is_empty() {
            return Some(self);
        }
        let (name, rest) = match path.find('/'){
            Some(s) => {
                let (n,a) = path.split_at(s);
                let (_,a) = a.split_at(1);
                (n,a)
            },
            None => (path.as_str(),""),
        };
        //let (name,rest) = path.split_at(split);
        //let (_,rest) = rest.split_at(1);
        println!("Split {}:{}",name,rest);
        let ret = match self.index(name) {
            Some(s) => match s.get_path(rest) {
                Some(s) => s,
                None => return None,
            },
            None => return None,
        };
        Some(ret)
    }
    pub fn get_path_mut(&mut self , path : &str ) -> Option<&mut FilesystemEntry> {
        let mut path = path.to_owned();
        let split = match path.find('/'){
            Some(s) => s,
            None => return Some(self),
        };
        let (name,rest) = path.split_at(split);

        let ret = match self.indexMut(name) {
            Some(s) => match s.get_path_mut(rest) {
                Some(s) => s,
                None => return None,
            },
            None => return None,
        };
        Some(ret)
    }
    pub fn index(&self ,index : &str) -> Option<&FilesystemEntry> {
        for child in self.children.iter() {
            if child.name == index {
                return Some(child)
            }

        }
        None

    }
    pub fn indexMut(&mut self,index : & str) -> Option<&mut FilesystemEntry> {
        for child in self.children.iter_mut() {
            if child.name == index {
                return Some(child)
            }

        }
        None
    }
    pub fn from_tree(tree : &Tree, repo : &Repository, name : String, mut path : String, inodes : &mut Vec<String>, file_mode : i32) -> FilesystemEntry {
        let mut children  = Vec::new();
        if !path.is_empty() {
            path = path + "/";
        }
        for entry in tree {
            children.push(FilesystemEntry::from_tree_entry(&entry,repo,path.clone()+name.as_str(),inodes));
        }

        inodes.push(path.clone() + name.as_str());
        println!("Folder:{}",path+name.as_str());
        FilesystemEntry{
            name,
            file_type : FileType::Directory,
            oid : Some(tree.id()),
            ino : inodes.len()-1,
            children,
            size : 0u64,
            content : None,
            write : false,
            write_mode : 0,
            file_mode,

        }
    }
    pub fn from_tree_entry(treeEntry : &TreeEntry, repo : &Repository, path:String, inodes : &mut Vec<String>) -> FilesystemEntry {
        let name : String = treeEntry.name().unwrap().to_owned();
        let file_mode = treeEntry.filemode();
        let treeEntry = treeEntry.to_object(repo).unwrap();
        let mut full_path = path.clone();
        if !full_path.is_empty(){
            full_path = full_path + "/";
        }
        full_path = full_path + name.as_str();

        let oid = treeEntry.id();

        match treeEntry.clone().into_blob() {
            Ok(f) => {
                println!("File:{}",full_path);
                let size = f.content().len() as u64;
                inodes.push(full_path);
                return FilesystemEntry {
                    name,
                    file_type : FileType::RegularFile,
                    oid : Some(oid),
                    ino: inodes.len() - 1,
                    children : Vec::new(),
                    size,
                    content : Some(Vec::new()),
                    write : false,
                    write_mode : 0,
                    file_mode,
                }

            },
            Err(e) => {},
        };
        match treeEntry.as_tree() {
            Some(t) =>  FilesystemEntry::from_tree(t,repo,name,path,inodes,file_mode),
            None => //empty tree?
                {
                    println!("Folder:{}",full_path);
                    inodes.push(full_path);
                    FilesystemEntry {
                        name,
                        file_type: FileType::Directory,
                        oid: Some(oid),
                        ino: inodes.len() - 1,
                        children: Vec::new(),
                        size: 0,
                        content : None,
                        write : false,
                        write_mode : 0,
                        file_mode,
                    }
                }
        }
    }
    pub fn to_git_object(&self, repo :&mut Repository) -> Option<Oid> {
        match self.file_type {
            FileType::RegularFile => {
                return self.oid;

            },
            FileType::Directory => {
                let mut entries = Vec::new();
                for child in &self.children {
                    let oid = match child.to_git_object(repo) {
                        Some(oid) => oid,
                        None => {continue}
                    };
                    let name = child.name.clone();
                    let file_mode = child.file_mode.clone();
                    entries.push(Git_entry{name,oid,file_mode});
                }
                match repo.treebuilder(None) {
                    Ok(mut tb) => {
                        for entry in entries {
                            tb.insert(entry.name,entry.oid,entry.file_mode);
                        }
                        Some(tb.write().unwrap())
                    },
                    Err(e) => {
                        panic!(e);
                    }

                }
            }
            _ => None,
        }
    }
}