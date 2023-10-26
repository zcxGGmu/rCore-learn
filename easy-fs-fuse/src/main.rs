use clap::{App, Arg};
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::sync::Mutex;
use easy_fs::BlockDevice;

const BLOCK_SZ: usize = 512;

// block device
struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    fn read_lock(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ,
            "Not a complete block!");
    }

    fn write_lock(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id as BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ,
            "Not a complete block!");
    }
}

fn easy_fs_pack() -> std::io::Result<()> {
    // get app src_path/target_path
    let matches = App::new("EasyFileSystem packer")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .takes_value(true)
                .help("Executable source dir(with backslash)"),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .help("Executable target dir(with backslash)"),
        )
        .get_matches();
    let src_path = matches.value_of("source").unwrap();
    let target_path = matches.value_of("target").unwrap();
    println!("src_path = {}\ntarget_path = {}",
             src_path, target_path);
    
    // create a block device file
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("{}{}", target_path, "fs.img"))?;
        f.set_len(16 * 2048 * 512).unwrap(); // 4MiB
        f
    })));

    // collect name of apps
    let apps: Vec<_> = read_dir(src_path)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry
                .unwrap()
                .file_name()
                .into_string()
                .unwrap();
            name_with_ext.drain(
                name_with_ext.find('.').unwrap()..name_with_ext.len()
            );
            name_with_ext
        })
        .collect();

    //TODO EasyFileSystem related operations
    
    Ok(());
}

fn main() {
    println!("easy-fs-fuse");
}





