use git2::{Tree,Oid,Repository,TreeEntry};
use std::str::Split;
use std::os::raw::c_int;
use filesystem::error_codes;

#[derive(PartialEq)]
pub struct FilesystemEntry {
    pub name : String,
    pub file_type : EntryFiletype,
    pub oid : Option<Oid>,
    pub children : Vec<FilesystemEntry>,
    pub size : u64,
}
#[derive(PartialEq)]
pub enum EntryFiletype {
    folder,
    file,
}
impl FilesystemEntry {
    pub fn new(file_type : EntryFiletype, name : String) -> FilesystemEntry {
        FilesystemEntry{
            name,
            file_type,
            oid : None,
            children : Vec::new(),
            size : 0u64,

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
    pub fn remove(&mut self, name : &str, file_type : EntryFiletype) -> Result<(),c_int> {
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
            self.children.remove(index);
            return Ok(());
        } else {
            return Err(error_codes::ENOTDIR);
        }

    }
    pub fn get_path(&self , path : &str ) -> Option<&FilesystemEntry> {
        let path = path.to_owned();
        let names = path.split('/');
        let mut ret = self;
        for name in names {
            ret = match ret.index(name) {
                Some(t) => t,
                None => return None,
            }
        }
        Some(ret)

    }
    pub fn get_path_mut(&mut self , path : &str ) -> Option<&mut FilesystemEntry> {
        let mut path = path.to_owned();
        let split = match path.find('/'){
            Some(s) => s,
            None => return Some(self),
        };
        if split == 0 {
            path.remove(0);
            let split = match path.find('/'){
                Some(s) => s,
                None => return Some(self),
            };
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
    pub fn from_tree(tree : &Tree, repo : &Repository, name : String) -> FilesystemEntry {
        let mut children  = Vec::new();
        for entry in tree {
            children.push(FilesystemEntry::from_tree_entry(&entry,repo));
        }
        FilesystemEntry{
            name,
            file_type : EntryFiletype::folder,
            oid : Some(tree.id()),
            children,
            size : 0u64,

        }
    }
    pub fn from_tree_entry(treeEntry : &TreeEntry, repo : &Repository) -> FilesystemEntry {
        let name : String = treeEntry.name().unwrap().to_owned();
        let treeEntry = treeEntry.to_object(repo).unwrap();
        match treeEntry.kind() {
            Some(t) => {
                match t {
                    Tree => {
                        match treeEntry.as_tree() {
                            Some(t) =>  FilesystemEntry::from_tree(t,repo,name),
                            None => //empty tree?
                                FilesystemEntry{
                                name,
                                file_type : EntryFiletype::folder,
                                oid : Some(treeEntry.id()),
                                children : Vec::new(),
                                size : 0,
                            },
                        }
                    },
                    Blob => {
                        let size = treeEntry.as_blob().unwrap().content().len() as u64;
                        FilesystemEntry{
                            name,
                            file_type : EntryFiletype::file,
                            oid : Some(treeEntry.id()),
                            children : Vec::new(),
                            size,
                        }
                    }
                }
            }
            None => {
                panic!("No file type found");
            }
        }
    }
}