# readahead-iterator Release History

## Unreleased

- Bug fix: Thread no longer panics when the receiver is dropped early,
  such as when using `.take()` to consume only part of the iterator.

## v0.1.1 2021-04-26

- Bug fix: Continue returning `None` repeatedly after the inner iterator
  terminates, rather than only once.

## v0.1.0 2021-04-25

- Initial release.
