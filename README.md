EVFS: Encryptable Virtual File System
=====================================

EVFS is an easy-to-use, cross-platform encryptable virtual file system abstraction layer for your game or application.

The main trait is `evfs::FileSystem`, which provides methods for reading and writing files, as well as creating directories. The trait is implemented for various file system backends, including:
- `evfs::LocalFileSystem`: A file system that reads and writes files to the local file system.
- `evfs::EncryptedFileSystem`: A file system that encrypts and decrypts files using a symmetric encryption algorithm.
- `evfs::ArchiveFileSystem`: A file system that reads and writes files to an archive file under `.eva` extension.
