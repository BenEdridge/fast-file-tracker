#![feature(async_closure)]

use jwalk::WalkDirGeneric;
use rusqlite::{params, Connection as ConnectionRusqlite, DatabaseName};
use std::env;
use std::fs;
use std::fs::File;
use std::hash::Hasher;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use twox_hash::XxHash64;

#[derive(sqlx::FromRow, Debug)]
struct Entry {
    row_id: i64,
    uuid: String,
    xxhash64: String,
    file_path: Option<String>,
    size: Option<i64>,
    extension: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let start = Instant::now();

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let mut file_count: u64 = 0;
    let mut skip_count: u64 = 0;
    let mut total_size: u128 = 0;

    let dirname = &args[1];

    let mut path_vec: Vec<PathBuf> = Vec::with_capacity(1_500_000);
    let sqlite_queries: Arc<Mutex<Vec<String>>> =
        Arc::new(Mutex::new(Vec::with_capacity(1_500_000)));
    let conn_backup = ConnectionRusqlite::open_in_memory().unwrap();

    conn_backup
        .execute(
            "
        CREATE TABLE IF NOT EXISTS entries (
            xxhash64 TEXT NOT NULL,
            file_path TEXT NOT NULL
        );
        ",
            params![],
        )
        .unwrap();

    let finished_dir_walk = Instant::now();

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

                    // let path = path;
                    // hash_file_and_save(&path, &sqlite_queries);

                    file_count = file_count + 1;
                    total_size = total_size + size as u128;
                }
            }
            Err(e) => {
                println!("Failure to Read MetaData Error: {:?} path: {:?}", e, path);
            }
        };
    }

    let finished_dir_walk = finished_dir_walk.elapsed();
    println!("Read and Saved all Paths in: {:?}", finished_dir_walk);

    let finished_file_hash = Instant::now();

    path_vec.iter().for_each(|path| {
        hash_file_and_save(path, &sqlite_queries);
    });

    let finished_file_hash = finished_file_hash.elapsed();
    println!(
        "Hashed and Prepared all Database Inserts in: {:?}",
        finished_file_hash
    );

    let finished_db_insert = Instant::now();

    let state_ref = Arc::clone(&sqlite_queries);
    let sqlite_queries_guarded = state_ref.lock().unwrap();

    let tx = conn_backup.unchecked_transaction().unwrap();
    for q in sqlite_queries_guarded.iter() {
        let built_query = format!("INSERT INTO entries (xxhash64, file_path) VALUES {};", q);
        conn_backup.execute(&built_query, params![]).unwrap();
    }
    tx.commit().unwrap();

    let finished_db_insert = finished_db_insert.elapsed();
    println!(
        "Inserted all entries into in-memory DB in: {:?}",
        finished_db_insert
    );

    let finished_backup = Instant::now();

    conn_backup
        .backup(
            DatabaseName::Main,
            Path::new("fast_file_tracker.sqlite3"),
            None,
        )
        .unwrap();

    let finished_backup = finished_backup.elapsed();
    println!("Saved DB to file in: {:?}", finished_backup);

    let finished_duration = start.elapsed();
    let mb = total_size / 1024 / 1024;
    let mb_per_second = mb as f64 / finished_duration.as_secs_f64();

    print_stats(
        mb,
        mb_per_second,
        file_count,
        skip_count,
        finished_dir_walk,
        finished_file_hash,
        finished_db_insert,
        finished_backup,
        finished_duration,
    );
    Ok(())
}

fn build_insert_string(hash_result: &u64, path: &Path) -> String {
    let str = format!(
        "(
        \"{:16x}\",
        {:?});",
        hash_result, path
    );
    return str.clone();
}

fn print_stats(
    megabytes: u128,
    megabytes_per_sec: f64,
    file_count: u64,
    skip_count: u64,
    dir_list_time: Duration,
    hash_time: Duration,
    db_insert_time: Duration,
    db_copy_time: Duration,
    total_time: Duration,
) {
    println!(
        "Hashed {} Mb @ {:.2} Mb/s ({} files in {} directories) in {:.2?}",
        megabytes, megabytes_per_sec, file_count, skip_count, total_time
    );
    println!("");
    println!("Statistics:");
    println!("----------------------------------------");
    println!("{: <25} | {: <25?}", "Directory Traversal", dir_list_time);
    println!("{: <25} | {: <25?}", "Hashing", hash_time);
    println!(
        "{: <25} | {: <25?}",
        "Database Insert (memory)", db_insert_time
    );
    println!("{: <25} | {: <25?}", "Database Copy (Disk)", db_copy_time);
    println!("----------------------------------------");
    println!("{: <25} | {: <25?}", "Total Time", total_time);
}

fn hash_file_and_save(path: &Path, sqlite_queries: &Arc<Mutex<Vec<String>>>) {
    let file = File::open(&path).unwrap();
    let mut reader = BufReader::new(file);
    let mut hasher = XxHash64::with_seed(0);

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
    // println!("{:16x} {:?}", hash_result, path);

    let db_insert_string = build_insert_string(&hash_result, &path);

    let state_ref = Arc::clone(&sqlite_queries);
    let mut sqlite_queries_guarded = state_ref.lock().unwrap();
    sqlite_queries_guarded.push(db_insert_string);
}
