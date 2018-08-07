use git2::{Tree,Oid,Repository,TreeEntry};


pub struct FilesystemEntry<'a> {
    name : &'a str,
    file_type : EntryFiletype,
    oid : Option<Oid>,
    children : Vec<FilesystemEntry<'a>>,
}
pub enum EntryFiletype {
    folder,
    file,
}
impl<'a> FilesystemEntry<'a> {
    pub fn new(file_type : EntryFiletype, name : &'a str) -> FilesystemEntry<'a> {
        FilesystemEntry{
            name,
            file_type,
            oid : None,
            children : Vec::new(),

        }
    }
    pub fn from_tree(tree : &Tree, repo : &Repository, name : &'a str) -> FilesystemEntry<'a> {
        let mut children  = Vec::new();
        for entry in tree {
            children.push(File::fromEntry(entry,repo));
        }
        FilesystemEntry{
            name,
            file_type : EntryFiletype::folder,
            oid : tree.id,
            children,

        }
    }
    pub fn from_tree_entry(treeEntry : &TreeEntry, repo : &Repository) -> FilesystemEntry<'a> {
        match treeEntry.kind() {
            Some(t) => {
                match t {
                    Tree => {
                        FilesystemEntry::from_tree(&treeEntry.to_object(repo).unwrap().into_tree().unwrap(),
                                                   repo,
                                                   treeEntry.name().unwrap())
                    },
                    _ => {
                        FilesystemEntry{
                            name : treeEntry.name().unwrap(),
                            file_type : EntryFiletype::file,
                            oid : treeEntry.id().unwrap(),
                            children : Vec::new(),
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