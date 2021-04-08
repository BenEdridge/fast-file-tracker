## Fast File Tracker

An extremely fast file tracker and validator using xxhash 

> A work in progress 😉

- NVMe speed file hashing with [xxHash](https://cyan4973.github.io/xxHash/)
- Parallel directory traversal and hashing with [Rayon](https://github.com/rayon-rs/rayon)
- Automatically loads data into in a DB for querying or search functionality
- Tiny 3M~ binary


## Getting Started

Tested on Ubuntu 20.10 Desktop PC:
- AMD Ryzen 5 3600 @3.4Ghz
- 64GB RAM
- Samsung 970 EVO NVMe (500GB)

`sudo fast-file-tracker ~/`

Cold:
```
Read and Saved all Paths in: 9.87451401s
Hashed and Prepared all Database Inserts in: 14.760232406s
Inserted all entries into in-memory DB in: 3.858201563s
Saved DB to file in: 365.136776ms
Hashed 23012 Mb @ 797.41 Mb/s (1205573 files in 200709 directories) in 28.86s

Statistics:
----------------------------------------
Directory Traversal       | 9.87451401s
Hashing                   | 14.760232406s
Database Insert (memory)  | 3.858201563s
Database Copy (Disk)      | 365.136776ms
----------------------------------------
Total Time                | 28.858343917s
```

Hot (Immediate Retry of the above):
```
Read and Saved all Paths in: 2.779707316s
Hashed and Prepared all Database Inserts in: 1.347903273s
Inserted all entries into in-memory DB in: 3.789592278s
Saved DB to file in: 556.772962ms
Hashed 23114 Mb @ 2727.56 Mb/s (1205579 files in 200697 directories) in 8.47s

Statistics:
----------------------------------------
Directory Traversal       | 2.779707316s
Hashing                   | 1.347903273s
Database Insert (memory)  | 3.789592278s
Database Copy (Disk)      | 556.772962ms
----------------------------------------
Total Time                | 8.474256153s
```

`sudo fast-file-tracker /var/log/`

```
Read and Saved all Paths in: 6.697776ms
Hashed and Prepared all Database Inserts in: 1.348348521s
Inserted all entries into in-memory DB in: 2.190764ms
Saved DB to file in: 46.07565ms
Hashed 4612 Mb @ 3285.87 Mb/s (692 files in 33 directories) in 1.40s

Statistics:
----------------------------------------
Directory Traversal       | 6.697776ms
Hashing                   | 1.348348521s
Database Insert (memory)  | 2.190764ms
Database Copy (Disk)      | 46.07565ms
----------------------------------------
Total Time                | 1.40358399s
```
