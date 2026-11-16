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
            ("tcl", "text/x-tcl"),
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

fn file_type_rank(entry: &DirEntry) -> Option<u8> {
    let ft = entry.file_type()?;
    Some(if ft.is_dir() {
        0
    } else if ft.is_symlink() {
        1
    } else if ft.is_file() {
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
    })
}

fn entry_size(entry: &DirEntry) -> Option<u64> {
    if !entry.file_type().is_some_and(|ft| ft.is_file()) {
        return None;
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

fn compare_option<T, F>(a: Option<T>, b: Option<T>, missing_last: bool, cmp_fn: F) -> Ordering
where
    F: FnOnce(T, T) -> Ordering,
{
    match (a, b) {
        (Some(a), Some(b)) => cmp_fn(a, b),
        (Some(_), None) => {
            if missing_last {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (None, Some(_)) => {
            if missing_last {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (None, None) => Ordering::Equal,
    }
}

fn compare_bytes(a: &[u8], b: &[u8], case_sensitive: bool, natural: bool) -> Ordering {
    if natural {
        compare_natural_bytes(a, b, case_sensitive)
    } else if case_sensitive {
        a.cmp(b)
    } else {
        compare_case_insensitive_bytes(a, b)
    }
}

fn compare_case_insensitive_bytes(a: &[u8], b: &[u8]) -> Ordering {
    for (&ca, &cb) in a.iter().zip(b.iter()) {
        let ord = ca.to_ascii_lowercase().cmp(&cb.to_ascii_lowercase());
        if ord != Ordering::Equal {
            return ord;
        }
    }
    a.len().cmp(&b.len())
}

fn compare_natural_bytes(a: &[u8], b: &[u8], case_sensitive: bool) -> Ordering {
    let mut ia = 0;
    let mut ib = 0;
    let mut leading_zero_diff = Ordering::Equal;

    while ia < a.len() && ib < b.len() {
        let da = a[ia].is_ascii_digit();
        let db = b[ib].is_ascii_digit();

        if da && db {
            // Find end of numeric runs
            let start_a = ia;
            while ia < a.len() && a[ia].is_ascii_digit() {
                ia += 1;
            }
            let start_b = ib;
            while ib < b.len() && b[ib].is_ascii_digit() {
                ib += 1;
            }

            let num_a = &a[start_a..ia];
            let num_b = &b[start_b..ib];

            // Strip leading zeros
            let trimmed_a = strip_leading_zeros(num_a);
            let trimmed_b = strip_leading_zeros(num_b);

            // Compare by number of significant digits first, then lexicographically
            let ord = trimmed_a
                .len()
                .cmp(&trimmed_b.len())
                .then_with(|| trimmed_a.cmp(trimmed_b));

            if ord != Ordering::Equal {
                return ord;
            }

            if leading_zero_diff == Ordering::Equal {
                leading_zero_diff = num_a.len().cmp(&num_b.len());
            }
        } else {
            let ca = if case_sensitive {
                a[ia]
            } else {
                a[ia].to_ascii_lowercase()
            };
            let cb = if case_sensitive {
                b[ib]
            } else {
                b[ib].to_ascii_lowercase()
            };
            let ord = ca.cmp(&cb);
            if ord != Ordering::Equal {
                return ord;
            }
            ia += 1;
            ib += 1;
        }
    }

    (a.len() - ia).cmp(&(b.len() - ib)).then(leading_zero_diff)
}

fn strip_leading_zeros(s: &[u8]) -> &[u8] {
    let first_non_zero = s.iter().position(|&b| b != b'0').unwrap_or(s.len());
    &s[first_non_zero..]
}

fn compare_osstr(a: &OsStr, b: &OsStr, config: &Config) -> Ordering {
    let bytes_a = filesystem::osstr_to_bytes(a);
    let bytes_b = filesystem::osstr_to_bytes(b);
    compare_bytes(
        &bytes_a,
        &bytes_b,
        config.sort_case_sensitive,
        config.sort_natural,
    )
}

fn group_rank(entry: &DirEntry, config: &Config) -> u8 {
    // If both --dirs-first and --files-first are passed, --files-first wins.
    if config.files_first {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            0
        } else {
            1
        }
    } else if config.dirs_first {
        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
            0
        } else {
            1
        }
    } else {
        0
    }
}

fn hash_entry_with_seed(seed: u64, path: &OsStr) -> u64 {
    // SplitMix64-inspired mixing with FNV-1a over path bytes for deterministic pseudo-random key
    let bytes = filesystem::osstr_to_bytes(path);
    let mut h = seed ^ 0xcbf29ce484222325;
    for &b in bytes.iter() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h = (h ^ (h >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94d049bb133111eb);
    h ^ (h >> 31)
}

fn compare_by_field(a: &DirEntry, b: &DirEntry, field: SortField, config: &Config) -> Ordering {
    let missing_last = config.sort_missing_last;
    match field {
        SortField::Name => {
            let name_a = a.path().file_name().unwrap_or_else(|| a.path().as_os_str());
            let name_b = b.path().file_name().unwrap_or_else(|| b.path().as_os_str());
            compare_osstr(name_a, name_b, config)
        }
        SortField::Size => compare_option(entry_size(a), entry_size(b), missing_last, |sa, sb| {
            sa.cmp(&sb)
        }),
        SortField::Path => compare_osstr(
            a.stripped_path(config).as_os_str(),
            b.stripped_path(config).as_os_str(),
            config,
        ),
        SortField::Modified => compare_option(
            entry_modified(a),
            entry_modified(b),
            missing_last,
            |ma, mb| ma.cmp(&mb),
        ),
        SortField::Created => compare_option(
            entry_created(a),
            entry_created(b),
            missing_last,
            |ca, cb| ca.cmp(&cb),
        ),
        SortField::Accessed => compare_option(
            entry_accessed(a),
            entry_accessed(b),
            missing_last,
            |aa, ab| aa.cmp(&ab),
        ),
        SortField::Extension => compare_option(
            a.path().extension().filter(|e| !e.is_empty()),
            b.path().extension().filter(|e| !e.is_empty()),
            missing_last,
            |ea, eb| compare_osstr(ea, eb, config),
        ),
        SortField::Depth => entry_depth(a).cmp(&entry_depth(b)),
        SortField::Type => compare_option(
            file_type_rank(a),
            file_type_rank(b),
            missing_last,
            |ta, tb| ta.cmp(&tb),
        ),
        SortField::Mime => compare_bytes(
            mime_for_entry(a).as_bytes(),
            mime_for_entry(b).as_bytes(),
            config.sort_case_sensitive,
            false,
        ),
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

fn fallback_path_cmp(a: &DirEntry, b: &DirEntry, config: &Config) -> Ordering {
    compare_osstr(
        a.stripped_path(config).as_os_str(),
        b.stripped_path(config).as_os_str(),
        config,
    )
    .then_with(|| a.stripped_path(config).cmp(b.stripped_path(config)))
    .then_with(|| a.path().cmp(b.path()))
}

pub fn sort_entries(entries: &mut [DirEntry], config: &Config) {
    if !config.is_sorting() {
        entries.sort();
        return;
    }

    let has_random = config.sort.contains(&SortField::Random);

    if has_random {
        let seed = config.sort_seed.unwrap_or_else(|| {
            let time_nanos = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            time_nanos ^ fastrand::u64(..)
        });

        let mut keyed: Vec<(u64, usize)> = entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let key = hash_entry_with_seed(seed, entry.stripped_path(config).as_os_str());
                (key, i)
            })
            .collect();

        keyed.sort_by(|&(rand_a, idx_a), &(rand_b, idx_b)| {
            let a = &entries[idx_a];
            let b = &entries[idx_b];

            let mut ord = group_rank(a, config).cmp(&group_rank(b, config));
            if ord == Ordering::Equal {
                for &field in &config.sort {
                    ord = match field {
                        SortField::Random => rand_a.cmp(&rand_b),
                        _ => compare_by_field(a, b, field, config),
                    };
                    if ord != Ordering::Equal {
                        break;
                    }
                }
            }
            if ord == Ordering::Equal {
                ord = fallback_path_cmp(a, b, config);
            }
            if config.reverse { ord.reverse() } else { ord }
        });

        let mut sorted: Vec<Option<DirEntry>> = entries.iter_mut().map(|_| None).collect();
        for (new_pos, (_, old_idx)) in keyed.into_iter().enumerate() {
            // Temporary placeholder to move out of slice and back in
            sorted[new_pos] = Some(std::mem::replace(
                &mut entries[old_idx],
                DirEntry::broken_symlink(std::path::PathBuf::new()),
            ));
        }
        for (slot, val) in entries.iter_mut().zip(sorted) {
            *slot = val.unwrap();
        }
    } else {
        entries.sort_by(|a, b| {
            let mut ord = group_rank(a, config).cmp(&group_rank(b, config));
            if ord == Ordering::Equal {
                for &field in &config.sort {
                    ord = compare_by_field(a, b, field, config);
                    if ord != Ordering::Equal {
                        break;
                    }
                }
            }
            if ord == Ordering::Equal {
                ord = fallback_path_cmp(a, b, config);
            }
            if config.reverse { ord.reverse() } else { ord }
        });
    }
}
