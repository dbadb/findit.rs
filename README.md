# findit.rs
My first rust program. Runs a simple text search on all text files below a root directory.

Usage:  findit [opts] [rootdir] searchterm

Opts: -x ext, -i, -L, -d

-x: search only files matching this extension
-i: ignore case
-L: enumerate filenames that __don't__ contain `searchterm`
-d: debug/verbose


