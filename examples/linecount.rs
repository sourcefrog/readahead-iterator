// Copyright 2020 Martin Pool
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Count lines in filenames specified on the command line.
//!
//! Opening files can take time on slow filesystems: open the
//! next few files in parallel with reading their lines.

use std::fs::File;
use std::io::{BufRead, BufReader};

use readahead_iterator::IntoReadahead;

pub fn main() {
    let args = std::env::args();
    if args.len() <= 1 {
        println!("usage: line_count_files FILE...");
        return;
    }
    // `Args` isn't `Send`, so copy the values we need.
    let filenames: Vec<String> = args.skip(1).collect();

    // You must use `into_iter()` here so that ownership of the `Vec<String>`
    // moves into the iterator, and can be moved into the thread.
    let total_lines: usize = filenames
        .into_iter()
        .filter_map(|filename| {
            File::open(filename.clone())
                .map_err(|err| eprintln!("failed to open {}: {:?}", filename, err))
                .map(|file| (filename, file))
                .ok()
        })
        .readahead(5)
        .map(|(filename, file)| {
            let line_count = BufReader::new(file).lines().count();
            println!("{:>8} {}", line_count, filename);
            line_count
        })
        .sum();
    println!();
    println!("{:>8} TOTAL", total_lines);
}
