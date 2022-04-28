//! A cool aspect of the Unix interface is that most resources in Unix are represented as files,
//! including devices such as the console, pipes, and of course, real files. The file descriptor
//! layer is the layer that archives this uniformity.

/// Each open file is represented by a `struct File`, which is a wrapper around either an inode or
/// a pipe, plus an I/O offset.
/// each call to `open` creates a new open file (a new `struct File`):
///     if multiple processes open the same file independently, the different instances will have
///     different I/O offsets.
pub struct File {
    // A reference count tracks the number of references to a particular open file.

    // A file can be open for reading or writing or both. The `readable` and `writable` fields
    // track this.
    readable: bool,
    writable: bool,
}
