# findit.rs
My first rust program. Runs a simple text search on all text files below a root directory.

Usage:  findit [opts] [rootdir] searchterm

Opts: -x ext, -i, -L, -d

|opt|Description|
|:--|:--|
|-x| search only files matching provided file extension (eg `rs`)|
|-i| ignore case|
|-L| enumerate filenames that __don't__ contain `searchterm`|
|-d| debug/verbose|


