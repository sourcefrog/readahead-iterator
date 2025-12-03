use readahead_iterator::IntoReadahead;

fn main() {
    (0..)
        .into_iter()
        .readahead(3)
        .take(100)
        .for_each(|x| println!("{}", x));
}
