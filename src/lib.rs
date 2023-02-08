use std::io;
use std::fs::{self, DirEntry};
use std::ffi::{OsString, OsStr};
use std::path::Path;
use phf::{phf_map, phf_set};
use colored::*;

pub struct Config
{
    query: String,
    rootdir: String,
    ext: String,
    invmatch: bool,
    nocase: bool,
    debug: bool,
}

impl Config
{
    pub fn new(mut args: impl Iterator<Item = String>) 
            -> Result<Config, &'static str>
    {
        let mut query: String = String::new();
        let mut rootdir:String = String::from(".");
        let mut nocase : bool = false;
        let mut invmatch : bool  = false;
        let mut ext: String = String::new();
        let mut err = true;
        let mut debug: bool = false;


        // let apppath = Path::new(&args[0]);
        // let appnm = apppath.file_stem().unwrap();
        while let Some(arg) = args.next()
        {
            if arg == "-x" // -x ext
            {
                if let Some(field) = args.next()
                {
                    ext = field;
                }
            }
            else
            if arg == "-i"
            {
                nocase = true;
            }
            else
            if arg == "-L" // following grep (--files-without-match)
            {
                invmatch = true;
            }
            else
            if arg == "-d" // 
            {
                debug = true;
            }
            else
            {
                if query.len() > 0
                {
                    rootdir = query;
                    query = arg;
                }
                else
                {
                    query = arg;
                    err = false;
                }
            }
        }
        if err
        {
            return Err(
"\n\nUsage: findit [opts] [startdir] string 
   opts: -x ext, -i[gnorecase], -L[files-no-match], -d[ebug]\n\n");
        }
        else
        {
            if nocase
            {
                query = query.to_lowercase();
            }
            return Ok(Config { query, rootdir, nocase, ext, invmatch, debug });
        }
    }
}

pub fn run(config: &Config) -> io::Result<()>
{
    let root_path = Path::new(&config.rootdir);
    return visit_files_below(root_path, config, &search_file);
}

// https://doc.rust-lang.org/stable/std/fs/fn.read_dir.html
fn visit_files_below(dir: &Path, cfg: &Config,
        cb: &dyn Fn(&DirEntry, &Config, &OsString)) -> io::Result<()>
{
    if dir.is_dir()
    {
        if cfg.debug 
        {
            println!("visiting {:?}", dir.as_os_str());
        }
        for entry in fs::read_dir(dir)?
        {
            let entry = entry?;
            let path = entry.path();
            let os_file_name = entry.file_name();
            let file_name = os_file_name.to_str().unwrap();
            if IGNORE_DIRENT_SET.contains(file_name)
            {
                if cfg.debug
                {
                    println!("ignoring {}", file_name.red());
                }
                continue;
            }
            if path.is_dir()
            {
                match visit_files_below(&path, cfg, cb)
                {
                    Ok(()) => continue,
                    Err(e) => return Err(e)
                };
            }
            else
            {
                if file_is_interesting(&path, cfg)
                {
                    cb(&entry, cfg, &os_file_name);
                }
            }
        }
    }
    return Ok(());
}

fn file_is_interesting(file_path: &Path, cfg: &Config) -> bool
{
    let x = file_path.extension();
    match x
    {
        None => cfg.ext == "",
        Some(x) =>
        {
            if cfg.ext != "" 
            {
                return cfg.ext.as_str() == x;
            }
            else
            {
                return !fileext_implies_binary(x);
            }
        }
    }
}

fn fileext_implies_binary(ext: &OsStr) -> bool
{
    let key = ext.to_str().unwrap();
    if FILE_EXT_MAP.contains_key(key)
    {
        return !FILE_EXT_MAP[key];
    }
    else
    {
        return true;
    }
}

fn search_file(f: &DirEntry, cfg: &Config, osfn: &OsString)
{
    if cfg.debug 
    {
        println!("reading {:?}", osfn.to_str().unwrap());
    }
    match fs::read_to_string(f.path())
    {
        Ok(contents) =>
        {
            let mut nmatch = 0;
            for (line, lineno) in search(&cfg.query, &contents, cfg)
            {
                nmatch += 1;
                if !cfg.invmatch
                {
                    if nmatch == 1
                    {
                        let fnm = f.path().into_os_string().into_string().unwrap();
                        let nfnm = fnm.replace("\\", "/");
                        println!("{}", nfnm.cyan());
                    }
                    let oline = format!("{}", lineno);
                    println!(" {}: {line}", oline.red());
                }
            }
        },
        Err(_) =>
        {
            println!("{} {} -------------------", 
                "Problem reading".red(),
                osfn.to_str().unwrap().yellow());
        }
    }
}

pub fn search<'a>(query: &str, contents: &'a str, cfg: &Config)
    -> Vec<(&'a str, i32)>
{
    let mut results = Vec::new();
    let mut lineno = 1;
    if cfg.nocase
    {
        for line in contents.lines()
        {
            if line.to_lowercase().contains(query)
            {
                results.push((line, lineno));
            }
            lineno += 1;
        }
    }
    else
    {
        for line in contents.lines()
        {
            if line.contains(query)
            {
                results.push((line, lineno));
            }
            lineno += 1;
        }
    }
    return results;
}

static IGNORE_DIRENT_SET: phf::Set<&'static str> =
phf_set! {
    "_built",
    "_install",
    "_publish",
    "_package",
    "_unused",
    "node_modules",
    ".git",
    ".built",
    "target", // rust
};

static FILE_EXT_MAP: phf::Map<&'static str, bool> = 
phf_map!  {
    "a" => false,
	"apk" => false,
    "avi" => false,

    "bkm" => false,
    "bin" => false,

    "cpp" => true,
    "c" => true,
    "cc" => true,
    "css" => true,
    "cs" => true,
    "ck" => true,
    "class" => false,

    "db" => false, // msvc
    "db-wal" => false,
    "db-shm" => false,
    "dll" => false,
    "dis" => false,
    "dtex" => false,

    "elf" =>false, 
    "env" =>false, 
    "exe" =>false,
    "exr" =>false, 
    "exp" =>false, 

    "gif" =>false, 
    "gz" =>false,
    "go" => true,

    "html" => true,
    "htm" => true,
    "h" => true,
    "hpp" => true,

    "icns" =>false,
    "ilk" => false, // msvc
    "ini" => true,
    "inl" => true,
    "ino" => true,
    "ipch" => false, // msvc

    "jar" =>false,
    "java" => true,
    "js" => true,
    "json" => true,
    "jsmk" => true,

    "lib" => false, // msvc
    "Lib" => false, // msvc

    "mel" => true,
    "md" => true,

    "nib" =>false,

    "o" => false,
    "obj" => false, // msvc

    "py" => true,
    "pymk" => true,
    "png" =>false, 
    "pic" =>false, 
    "pyc" =>false, 
    "pyo" =>false, 
    "pdb" =>false,  // msvc
    "pdf" =>false, 
    "ptc" =>false,

    "rc" => true, // msvc resource
    "rtf" =>false, 
    "rib" =>false,

    "sf2" => false, // soundfile
    "sdf" => false, // msvc
    "shm" => false, // msvc
    "sl" => false,
    "slo" => false, 
    "so" => false, 
    "suo" => false, // juce
	"swp" =>false, 
    "sqlite" => false,
    "sql" => false,

    "tcl" => true,
    "txt" => true,
    "tar" =>false, 
    "tgz" =>false, 
    "tif" =>false, 
    "tiff" =>false, 
    "tex" =>false,
    "ttf" =>false,
    "tpp" =>true,
    
    "wav" =>false,

    "xml" => true,
    "xpm" =>false,
    "zip" =>false, 
    "z" => false,
};

#[cfg(test)]
mod tests
{
    // use super::*;
} 
