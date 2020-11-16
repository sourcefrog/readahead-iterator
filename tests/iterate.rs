// Copyright 2020 Martin Pool
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::thread::sleep;
use std::time::Duration;

use readahead_iterator::Readahead;

/// A lot like examples/sleepy, but with minimal sleeps.
#[test]
fn iterate_integers() {
    const N: usize = 100;
    let s: usize = Readahead::new(
        (0..N)
            .map(|i| i * 3)
            .inspect(|_| sleep(Duration::from_millis(1))),
        50,
    )
    .sum();
    assert_eq!(s, 3 * (N * (N - 1)) / 2);
}
