use std::env;
use std::fs;
use std::fs::File;
use std::hash::Hasher;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;
use std::time::Instant;

use twox_hash::XxHash32;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use jwalk::WalkDirGeneric;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let mut file_count = 0;
    let mut skip_count = 0;
    let mut total_size: u128 = 0;

    let dirname = &args[1];

    let mut path_vec: Vec<PathBuf> = Vec::with_capacity(1_500_000);

    let walk_dir = WalkDirGeneric::<(usize, bool)>::new(&dirname).process_read_dir(
        |_read_dir_state, children| {
            children.retain(|dir_entry_result| {
                dir_entry_result
                    .as_ref()
                    .map(|dir_entry| {
                        let file_type = dir_entry.file_type();
                        !file_type.is_block_device()
                            && !file_type.is_socket()
                            && !file_type.is_fifo()
                            && !file_type.is_symlink()
                    })
                    .unwrap_or(false)
            });
        },
    );

    let start = Instant::now();

    for entry in walk_dir
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let metadata = fs::metadata(&path);

        let _metadata = match metadata {
            Ok(metadata) => {
                let size = metadata.len();
                let file_type = metadata.file_type();

                if file_type.is_dir() || size == 0 {
                    skip_count = skip_count + 1;
                } else {
                    path_vec.push(path);

                    file_count = file_count + 1;
                    total_size = total_size + size as u128;
                }
            }
            Err(e) => {
                println!("Failure to Read MetaData Error: {:?} path: {:?}", e, path);
            }
        };
    }

    let finished_duration = start.elapsed();
    println!("Read and Saved all Paths in: {:?}", finished_duration);

    path_vec.par_iter().for_each(|path| {
        let file = File::open(&path).unwrap();
        let mut reader = BufReader::new(file);

        let mut hasher = XxHash32::with_seed(0);

        loop {
            let length = {
                let buffer = reader.fill_buf().unwrap();

                if buffer.is_empty() {
                    break;
                };

                hasher.write(buffer);
                buffer.len()
            };
            reader.consume(length);
        }
        let hash_result = hasher.finish();
        println!("{:16x} {:?}", hash_result, path);
    });

    let finished_duration = start.elapsed();
    let gb = total_size / 1024 / 1024;
    let gb_per_second = gb as f64 / finished_duration.as_secs_f64();

    println!(
        "Hashed {} Mb @ {:.2} Mb/s ({} files in {} directories) in {:.2?}",
        gb, gb_per_second, file_count, skip_count, finished_duration
    );

    Ok(())
}
