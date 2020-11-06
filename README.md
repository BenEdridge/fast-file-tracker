## Fast File Tracker

An extremely fast file tracker and validator using xxhash 

> A work in progress 😉

- NVMe speed file hashing with [xxHash](https://cyan4973.github.io/xxHash/)
- Parallel directory traversal and hashing with [Rayon](https://github.com/rayon-rs/rayon)
- Automatically loads data into in a DB for querying or search functionality


## Getting Started

Tested on Macbook Pro 2015
- 2.2 Ghz Core i7
- 16GB RAM
- APPLE SSD SM0256G

### Cold

```
$ cargo run ~/Desktop
...
    108e477e "/Users/admin/Desktop/73151555826473.txt"
    99519270 "/Users/admin/Desktop/73321555826536.txt"
    5304d46 "/Users/admin/Desktop/73491555826643.txt"
Hashed 1089 Mb @ 691 Mb/s (5137 files in 994 directories) in 1.44s
```

### Hot (Cached Data) 

```
$ cargo run ~/Desktop
...
    108e477e "/Users/admin/Desktop/73151555826473.txt"
    99519270 "/Users/admin/Desktop/73321555826536.txt"
    5304d46 "/Users/admin/Desktop/73491555826643.txt"
Hashed 1089 Mb @ 3448.41 Mb/s (5137 files in 994 directories) in 315.80ms
```

### Cold (Large External SSD via USB3)

```
Hashed 489486 Mb @ 385.90 Mb/s (461450 files in 105056 directories) in 1268.42s
```