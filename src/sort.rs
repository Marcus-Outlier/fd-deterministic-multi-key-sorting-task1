use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::sync::OnceLock;
use std::time::SystemTime;

use crate::cli::SortField;
use crate::config::Config;
use crate::dir_entry::DirEntry;
use crate::filesystem;

static MIME_MAP: OnceLock<HashMap<String, &'static str>> = OnceLock::new();

fn get_mime_map() -> &'static HashMap<String, &'static str> {
    MIME_MAP.get_or_init(|| {
        let mut map = HashMap::new();

        // Populate standard common MIME types first as baseline fallback
        let builtins: &[(&str, &str)] = &[
            ("7z", "application/x-7z-compressed"),
            ("aac", "audio/aac"),
            ("abw", "application/x-abiword"),
            ("arc", "application/x-freearc"),
            ("avi", "video/x-msvideo"),
            ("avif", "image/avif"),
            ("azw", "application/vnd.amazon.ebook"),
            ("bin", "application/octet-stream"),
            ("bmp", "image/bmp"),
            ("bz", "application/x-bzip"),
            ("bz2", "application/x-bzip2"),
            ("c", "text/x-c"),
            ("cab", "application/vnd.ms-cab-compressed"),
            ("cc", "text/x-c++src"),
            ("cda", "application/x-cdf"),
            ("conf", "text/plain"),
            ("cpp", "text/x-c++src"),
            ("csh", "application/x-csh"),
            ("css", "text/css"),
            ("csv", "text/csv"),
            ("cxx", "text/x-c++src"),
            ("deb", "application/vnd.debian.binary-package"),
            ("doc", "application/msword"),
            (
                "docx",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ),
            ("eot", "application/vnd.ms-fontobject"),
            ("epub", "application/epub+zip"),
            ("exe", "application/x-msdos-program"),
            ("flac", "audio/flac"),
            ("flv", "video/x-flv"),
            ("gif", "image/gif"),
            ("go", "text/x-go"),
            ("gz", "application/gzip"),
            ("h", "text/x-chdr"),
            ("hh", "text/x-c++hdr"),
            ("hpp", "text/x-c++hdr"),
            ("htm", "text/html"),
            ("html", "text/html"),
            ("ico", "image/vnd.microsoft.icon"),
            ("ics", "text/calendar"),
            ("ini", "text/plain"),
            ("iso", "application/x-iso9660-image"),
            ("jar", "application/java-archive"),
            ("java", "text/x-java"),
            ("jpeg", "image/jpeg"),
            ("jpg", "image/jpeg"),
            ("js", "text/javascript"),
            ("json", "application/json"),
            ("jsonld", "application/ld+json"),
            ("kt", "text/x-kotlin"),
            ("less", "text/less"),
            ("log", "text/plain"),
            ("lua", "text/x-lua"),
            ("m4a", "audio/mp4"),
            ("m4v", "video/mp4"),
            ("markdown", "text/markdown"),
            ("md", "text/markdown"),
            ("mid", "audio/midi"),
            ("midi", "audio/midi"),
            ("mjs", "text/javascript"),
            ("mkv", "video/x-matroska"),
            ("mov", "video/quicktime"),
            ("mp3", "audio/mpeg"),
            ("mp4", "video/mp4"),
            ("mpeg", "video/mpeg"),
            ("mpg", "video/mpeg"),
            ("mpkg", "application/vnd.apple.installer+xml"),
            ("odp", "application/vnd.oasis.opendocument.presentation"),
            ("ods", "application/vnd.oasis.opendocument.spreadsheet"),
            ("odt", "application/vnd.oasis.opendocument.text"),
            ("oga", "audio/ogg"),
            ("ogg", "audio/ogg"),
            ("ogv", "video/ogg"),
            ("ogx", "application/ogg"),
            ("opus", "audio/opus"),
            ("otf", "font/otf"),
            ("pbm", "image/x-portable-bitmap"),
            ("pdf", "application/pdf"),
            ("pgm", "image/x-portable-graymap"),
            ("php", "application/x-httpd-php"),
            ("pl", "text/x-perl"),
            ("pm", "text/x-perl"),
            ("png", "image/png"),
            ("pnm", "image/x-portable-anymap"),
            ("ppm", "image/x-portable-pixmap"),
            ("ppt", "application/vnd.ms-powerpoint"),
            (
                "pptx",
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            ),
            ("ps", "application/postscript"),
            ("py", "text/x-python"),
            ("rar", "application/vnd.rar"),
            ("rb", "application/x-ruby"),
            ("rpm", "application/x-redhat-package-manager"),
            ("rs", "text/x-rust"),
            ("rst", "text/prs.fallenstein.rst"),
            ("rtf", "application/rtf"),
            ("scala", "text/x-scala"),
            ("scss", "text/x-scss"),
            ("sh", "application/x-sh"),
            ("sql", "application/sql"),
            ("svg", "image/svg+xml"),
            ("swf", "application/x-shockwave-flash"),
            ("tar", "application/x-tar"),
            ("tcl", "application/x-tcl"),
            ("tex", "application/x-tex"),
            ("tgz", "application/gzip"),
            ("tif", "image/tiff"),
            ("tiff", "image/tiff"),
            ("toml", "application/toml"),
            ("ts", "video/mp2t"),
            ("tsv", "text/tab-separated-values"),
            ("ttf", "font/ttf"),
            ("txt", "text/plain"),
            ("vsd", "application/vnd.visio"),
            ("wasm", "application/wasm"),
            ("wav", "audio/x-wav"),
            ("weba", "audio/webm"),
            ("webm", "video/webm"),
            ("webp", "image/webp"),
            ("woff", "font/woff"),
            ("woff2", "font/woff2"),
            ("xhtml", "application/xhtml+xml"),
            ("xls", "application/vnd.ms-excel"),
            (
                "xlsx",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            ("xml", "application/xml"),
            ("xul", "application/vnd.mozilla.xul+xml"),
            ("xz", "application/x-xz"),
            ("yaml", "application/yaml"),
            ("yml", "application/yaml"),
            ("zst", "application/zstd"),
            ("zip", "application/zip"),
        ];

        for &(ext, mime) in builtins {
            map.insert(ext.to_string(), mime);
        }

        // Also parse /etc/mime.types if available for any additional extensions,
        // without overriding the predictable builtin mappings above.
        if let Ok(content) = fs::read_to_string("/etc/mime.types") {
            let leaked_content: &'static str = Box::leak(content.into_boxed_str());
            for line in leaked_content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let mut parts = line.split_whitespace();
                if let Some(mime) = parts.next() {
                    for ext in parts {
                        let ext_lower = ext.to_ascii_lowercase();
                        map.entry(ext_lower).or_insert(mime);
                    }
                }
            }
        }

        map
    })
}

pub fn mime_for_entry(entry: &DirEntry) -> &'static str {
    if let Some(ft) = entry.file_type() {
        if ft.is_dir() {
            return "inode/directory";
        }
        if ft.is_symlink() {
            return "inode/symlink";
        }
        if filesystem::is_block_device(ft) {
            return "inode/blockdevice";
        }
        if filesystem::is_char_device(ft) {
            return "inode/chardevice";
        }
        if filesystem::is_pipe(ft) {
            return "inode/fifo";
        }
        if filesystem::is_socket(ft) {
            return "inode/socket";
        }
    }

    if let Some(ext) = entry.path().extension().and_then(OsStr::to_str) {
        let ext_lower = ext.to_ascii_lowercase();
        if let Some(&mime) = get_mime_map().get(&ext_lower) {
            return mime;
        }
    }

    "application/octet-stream"
}

fn file_type_rank(entry: &DirEntry) -> u8 {
    if let Some(ft) = entry.file_type() {
        if ft.is_file() {
            0
        } else if ft.is_dir() {
            1
        } else if ft.is_symlink() {
            2
        } else if filesystem::is_pipe(ft) {
            3
        } else if filesystem::is_socket(ft) {
            4
        } else if filesystem::is_char_device(ft) {
            5
        } else if filesystem::is_block_device(ft) {
            6
        } else {
            7
        }
    } else {
        8
    }
}

fn entry_size(entry: &DirEntry) -> Option<u64> {
    if entry.file_type().is_some_and(|ft| ft.is_dir()) {
        return Some(0);
    }
    entry.metadata().map(|m| m.len())
}

fn entry_modified(entry: &DirEntry) -> Option<SystemTime> {
    entry.metadata().and_then(|m| m.modified().ok())
}

fn entry_created(entry: &DirEntry) -> Option<SystemTime> {
    entry.metadata().and_then(|m| m.created().ok())
}

fn entry_accessed(entry: &DirEntry) -> Option<SystemTime> {
    entry.metadata().and_then(|m| m.accessed().ok())
}

fn entry_depth(entry: &DirEntry) -> usize {
    entry.depth().unwrap_or_else(|| {
        filesystem::strip_current_dir(entry.path())
            .components()
            .count()
    })
}

fn compare_option_asc<T: Ord>(a: Option<T>, b: Option<T>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_by_field(a: &DirEntry, b: &DirEntry, field: SortField, config: &Config) -> Ordering {
    match field {
        SortField::Name => {
            let name_a = a.path().file_name().unwrap_or_else(|| a.path().as_os_str());
            let name_b = b.path().file_name().unwrap_or_else(|| b.path().as_os_str());
            name_a.cmp(name_b)
        }
        SortField::Size => compare_option_asc(entry_size(a), entry_size(b)),
        SortField::Path => a.stripped_path(config).cmp(b.stripped_path(config)),
        SortField::Modified => compare_option_asc(entry_modified(a), entry_modified(b)),
        SortField::Created => compare_option_asc(entry_created(a), entry_created(b)),
        SortField::Accessed => compare_option_asc(entry_accessed(a), entry_accessed(b)),
        SortField::Extension => {
            let ext_a = a.path().extension().unwrap_or_else(|| OsStr::new(""));
            let ext_b = b.path().extension().unwrap_or_else(|| OsStr::new(""));
            ext_a.cmp(ext_b)
        }
        SortField::Depth => entry_depth(a).cmp(&entry_depth(b)),
        SortField::Type => file_type_rank(a).cmp(&file_type_rank(b)),
        SortField::Mime => mime_for_entry(a).cmp(mime_for_entry(b)),
        SortField::Length => {
            let len_a = a
                .path()
                .file_name()
                .unwrap_or_else(|| a.path().as_os_str())
                .len();
            let len_b = b
                .path()
                .file_name()
                .unwrap_or_else(|| b.path().as_os_str())
                .len();
            len_a.cmp(&len_b)
        }
        SortField::PathLength => {
            let len_a = a.stripped_path(config).as_os_str().len();
            let len_b = b.stripped_path(config).as_os_str().len();
            len_a.cmp(&len_b)
        }
        SortField::Random => Ordering::Equal,
    }
}

pub fn sort_entries(entries: &mut [DirEntry], config: &Config) {
    if config.sort.is_empty() {
        entries.sort_by(|a, b| {
            let ord = a
                .stripped_path(config)
                .cmp(b.stripped_path(config))
                .then_with(|| a.path().cmp(b.path()));
            if config.reverse { ord.reverse() } else { ord }
        });
        return;
    }

    let has_random = config.sort.contains(&SortField::Random);

    if has_random {
        // Assign random keys to each entry so random sorting is a valid total order
        let mut keyed: Vec<(u64, usize)> =
            (0..entries.len()).map(|i| (fastrand::u64(..), i)).collect();

        keyed.sort_by(|&(rand_a, idx_a), &(rand_b, idx_b)| {
            let a = &entries[idx_a];
            let b = &entries[idx_b];
            let mut ord = Ordering::Equal;
            for &field in &config.sort {
                ord = match field {
                    SortField::Random => rand_a.cmp(&rand_b),
                    _ => compare_by_field(a, b, field, config),
                };
                if ord != Ordering::Equal {
                    break;
                }
            }
            if ord == Ordering::Equal {
                ord = a
                    .stripped_path(config)
                    .cmp(b.stripped_path(config))
                    .then_with(|| a.path().cmp(b.path()));
            }
            if config.reverse { ord.reverse() } else { ord }
        });

        // Apply permutation in-place or via temporary Vec
        let mut perm: Vec<usize> = keyed.into_iter().map(|(_, idx)| idx).collect();
        for i in 0..entries.len() {
            while perm[i] != i {
                let target = perm[i];
                entries.swap(i, target);
                perm.swap(i, target);
            }
        }
    } else {
        entries.sort_by(|a, b| {
            let mut ord = Ordering::Equal;
            for &field in &config.sort {
                ord = compare_by_field(a, b, field, config);
                if ord != Ordering::Equal {
                    break;
                }
            }
            if ord == Ordering::Equal {
                ord = a
                    .stripped_path(config)
                    .cmp(b.stripped_path(config))
                    .then_with(|| a.path().cmp(b.path()));
            }
            if config.reverse { ord.reverse() } else { ord }
        });
    }
}
