# S3 File Sync

This is a program used to send files to AWS S3 as they produced and clean up the synchronised directory.

It uses [notify-rs](https://docs.rs/notify/) to listen for filesystem changes. 


## Origin

This project arose out of a need to synchronise files produced on a Windows machine and consumed by several other
services via AWS S3.

In the beginning the synchronisation was done with `s3 sync` of the [AWS CLI](https://aws.amazon.com/cli/). This turned
out to be pretty slow and resource intensive as there are more than 10000 files generated each day.


## Design

As the sync operations with the AWS CLI started getting slower and slower, as well as files starting to accumulate,
there was a need for a program that would do the same thing, only in a more efficient manner :

* Listen to file system changes instead of scanning the whole directory tree regularly and comparing it to the S3 bucket
* Track which files have been successfully uploaded to S3 and delete them from the local file system

### Assumptions

This program is made for a particular use case and as such is based on some assumptions about the files it handles:

* The files are always added to the directory and never renamed or their contents modified. Such actions would be ignored.
* Synchronisation always happens from the server to the S3 bucket.
* The files have some sort of sequencing built into the name, which means that once a file has been dealt with no other
file with the same filename will appear. Such a file would be ignored.
* Only files using UTF-8 names will be handled. Other files will be ignored.

## Implementation

* Written in Rust
* Uses [notify-rs](https://docs.rs/notify/) for detecting filesystem changes
* Uses [rusqlite](https://docs.rs/rusqlite/) for persisting information about handled files


## Licensing

### Author

S3 File Sync is created by [Vlad Vasiliu](https://github.com/vladvasiliu/).

## License

This project is released under the terms of the GNU General Public License, version 3.
Please see [`COPYING`](COPYING) for the full text of the license.
