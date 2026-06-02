use anyhow::Error;
use axum::routing::any;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    mem::take,
    path::Path,
};

const PAGE_SIZE: u64 = 4096;

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    id: i32,
    name: String,
    age: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct DbHeader {
    version: u32,
    root_page_id: u64,
    page_size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Node {
    keys: Vec<i32>,
    values: Vec<u64>,
    is_leaf: bool,
}

trait BNaryTree {
    fn search(&mut self, key: i32) -> Result<u64, anyhow::Error>;
    fn insert(&mut self, key: i32, value: u64) -> &mut Self;
    fn remove(&mut self, key: i32, value: u64) -> String;
}

impl BNaryTree for Node {
    fn search(&mut self, key: i32) -> Result<u64, anyhow::Error> {
        let mut low = 0;
        let mut high = self.keys.len() - 1;

        while low <= high {
            let mid = usize::midpoint(low, high);
            if self.keys[mid] == key {
                return Ok(self.values[mid]);
            }
            if self.keys[mid] < key {
                high = mid + 1;
            }
            if self.keys[mid] > key {
                low = mid - 1;
            }
        }
        Err(anyhow::anyhow!("Key {} not found in this index", key))
    }

    fn insert(&mut self, key: i32, value: u64) -> &mut Self {
        let keys = &self.keys;

        if self.is_leaf == false {
            if keys[keys.len() - 1] < key {
                //Access the address of the last key
            }
            if keys[0] > key {
                //Access the address of the first key
            }
        } else {
            self.keys.push(key);
            self.values.push(value);
        }
        // fsync() here
        self
    }

    fn remove(&mut self, key: i32, value: u64) -> String {
        "Remove".to_string()
    }
}

fn create_db_header() -> Result<File, anyhow::Error> {
    let mut index_file = match read("data.db".to_string()) {
        Ok(f) => f,
        Err(_e) => {
            println!("Index file not found, create new one...");
            create_page("data.db".to_string())?
        }
    };
    let header = DbHeader {
        version: 1,
        root_page_id: 1,
        page_size: 4096,
    };
    index_file.set_len(4096)?;
    let byte_code_header = bincode::serialize(&header)?;
    index_file.write_all(&byte_code_header)?;
    return Ok(index_file);
}

fn create_index(mut index_file: File) -> Result<File, anyhow::Error> {
    index_file.seek(SeekFrom::End(0))?;

    let root = Node {
        keys: Vec::new(),
        values: Vec::new(),
        is_leaf: true,
    };

    let mut root_bytes = bincode::serialize(&root)?;
    root_bytes.resize(PAGE_SIZE as usize, 0);
    index_file.write_all(&root_bytes)?;
    return Ok(index_file);
}

fn create_page(file_name: String) -> Result<File, anyhow::Error> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_name)?;
    file.set_len(4096)?;
    return Ok(file);
}

fn read_page<T: serde::de::DeserializeOwned>(file: &mut File, page_id: u64) -> anyhow::Result<T> {
    let off_set = page_id * PAGE_SIZE;

    file.seek(SeekFrom::Start(off_set))?;
    let mut take_file = file.take(PAGE_SIZE);

    let object: T = bincode::deserialize_from(&mut take_file)?;
    Ok(object)
}

fn write(mut file: File, bytes: Vec<u8>, index_key_id: i32) -> Result<(), anyhow::Error> {
    // Save object into .bin file
    file.write_all(&bytes)?;
    let current_write_offset = file.seek(SeekFrom::End(0))?;

    // Modify the index .db file
    let mut index_page = read("data.db".to_string())?;

    let header: DbHeader = read_page::<DbHeader>(&mut index_page, 0)?;
    let mut index_node: Node = read_page::<Node>(&mut index_page, header.root_page_id)?;
    let node = index_node.insert(index_key_id, current_write_offset);

    index_page.seek(SeekFrom::Start(header.root_page_id * PAGE_SIZE))?;

    let mut bytes = bincode::serialize(&node)?;
    bytes.resize(4096, 0);

    index_page.write_all(&bytes)?;
    index_page.flush()?;

    println!("Node keys: {:?}", node.keys);
    println!("Node values: {:?}", node.values);
    Ok(())
}

fn read(file_name: String) -> Result<File, anyhow::Error> {
    Ok(OpenOptions::new().read(true).write(true).open(file_name)?)
}

fn read_index_file(index_key: i32) -> Result<u64, anyhow::Error> {
    let mut data_file = read("data.db".to_string())?;
    //TODO first get the root_id from the Header page
    //We assume the root always in Page 1
    let mut index_page = read_page::<Node>(&mut data_file, 1)?;
    let data_offset = index_page.search(index_key)?;
    Ok(data_offset)
}

fn main() -> Result<(), anyhow::Error> {
    let person = Person {
        id: 1,
        name: "Mike".to_string(),
        age: 20,
    };
    let encoded = bincode::serialize(&person)?;

    // Create initial header and index
    let db_header = create_db_header()?;
    let db_index = create_index(db_header)?;

    let write_file = match read("user.bin".to_string()) {
        Ok(f) => f,
        Err(_e) => {
            println!("Database not found, create new one...");
            create_page("user.bin".to_string())?
        }
    };

    write(write_file, encoded, person.id)?;

    // let mut read_file = match read("user.bin".to_string()) {
    //     Ok(f) => f,
    //     Err(_e) => {
    //         println!("Database not found, create new one...");
    //         create_page("user.bin".to_string())?
    //     }
    // };
    // let user_byte_offset = read_index_file(3)?;
    // read_file.seek(SeekFrom::Start(user_byte_offset))?;
    // let user: Person = bincode::deserialize_from(&mut read_file)?;
    // println!("Person name: {}", user.name);

    Ok(())
}
