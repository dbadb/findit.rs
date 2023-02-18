use std::io;
use std::fs::{self, DirEntry};
use std::ffi::{OsString, OsStr};
use std::path::Path;
use phf::{phf_map, phf_set};
use colored::*;

pub struct Config
{
    query: String,
    nocase: bool,
    ext: String,
    rootdir: String,
    invmatch: bool,
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
        args.next(); // skip argv[0]

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
            if debug
            {
                println!("rootdir: {rootdir}, query: {query}");
            }
            return Ok(Config { query, rootdir, nocase, ext, invmatch, debug });
        }
    }

    pub fn summarize(&self) -> String
    {
        let mut summary = format!("---- search for '{}'", self.query);
        if self.nocase
        {
            summary += ", nocase";
        }
        if self.ext.len() > 0
        {
            summary += &format!(", ext: {}", self.ext);
        }

        summary += " ----------------------------";
        return summary;
    }
}

pub fn run(config: &Config, ndirs: &mut u32, nfiles: &mut u32, nlines: &mut u32) -> io::Result<()>
{
    let root_path = Path::new(&config.rootdir);
    return visit_files_below(root_path, config, &search_file, 
                            ndirs, nfiles, nlines);
}

// https://doc.rust-lang.org/stable/std/fs/fn.read_dir.html
fn visit_files_below(dir: &Path, cfg: &Config,
        cb: &dyn Fn(&DirEntry, &Config, &OsString, &mut u32, &mut u32),
        ndirs: &mut u32, nfiles: &mut u32, nlines: &mut u32,
    ) -> io::Result<()>
{
    if dir.is_dir()
    {
        *ndirs += 1;
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
                match visit_files_below(&path, cfg, cb, ndirs, nfiles, nlines)
                {
                    Ok(()) => continue,
                    Err(e) => return Err(e)
                };
            }
            else
            {
                if file_is_interesting(&path, cfg)
                {
                    cb(&entry, cfg, &os_file_name, nfiles, nlines);
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

fn search_file(f: &DirEntry, cfg: &Config, osfn: &OsString, 
            nfiles: &mut u32, nlines: &mut u32)
{
    if cfg.debug 
    {
        println!("reading {:?}", osfn.to_str().unwrap().dimmed());
    }
    *nfiles += 1;
    match fs::read_to_string(f.path())
    {
        Ok(contents) =>
        {
            let mut nmatch = 0;
            for (line, lineno) in search(&cfg.query, &contents, cfg, nlines)
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
                    let oline = format!("{:width$}", lineno, width=4);
                    if line.len() < 80
                    {
                        println!(" {}: {}", oline.blue(), line);
                    }
                    else
                    {
                        println!(" {}: {:.80} {}", oline.blue(), line, "...".blue());
                    }
                }
            }
        },
        Err(_) =>
        {
            let fnm = f.path().into_os_string().into_string().unwrap();
            let nfnm = fnm.replace("\\", "/");
            println!("{} {}", nfnm.cyan(), "skipped".red());
        }
    }
}

pub fn search<'a>(query: &str, contents: &'a str, cfg: &Config, nlines: &mut u32)
    -> Vec<(&'a str, u32)>
{
    if cfg.nocase
    {
        let mut lineno: u32 = 1;
        let x:Vec<(&str, u32)> = contents .lines()
            .map(|line| 
                {
                    let ret = (line, lineno);
                    lineno += 1;
                    return ret;
                })
            .filter(|line| 
                {
                    line.0.to_lowercase().contains(query)
                })
            .collect();
        *nlines += lineno;
        return x;
    }
    else
    {
        let mut lineno = 1;
        let x = contents
            .lines()
            .map(|line| 
                {
                    let ret = (line, lineno);
                    lineno += 1;
                    return ret;
                })
            .filter(|line| 
                {
                    line.0.contains(query)
                })
            .collect();
        *nlines += lineno;
        return x;
    }
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
    "rs" => true,

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
    "toml" => true,
    
    "wav" =>false,

    "xml" => true,

    "yaml" => true,
    "yml" => true,

    "xpm" =>false,
    "zip" =>false, 
    "z" => false,
};

#[cfg(test)]
mod tests
{
    // use super::*;
} 
