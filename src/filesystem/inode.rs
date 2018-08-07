use time::{SteadyTime,Duration};
use std::collections::VecDeque;


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
    pub fn new (ttl : Option<usize>) -> InodeCollection<'a> {
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

    pub fn insert(&mut self, path : &'a str) -> u64 {

        self.inodes.push_back(InodeEntry {
            path,
            valid_until : SteadyTime::now()+self.ttl,
        });
        return self.inodeoffset + self.inodes.len() as u64;
    }
    pub fn remove(&mut self, inode : u64){
        // do nothing since removing an element in the middle will break the list
    }
    pub fn get(&mut self, inode : u64) -> Option<&str> {
        self.clean();
        match self.inodes.get((inode-self.inodeoffset) as usize) {
            Some(e) => Some(e.path),
            None => None,
        }
    }
}