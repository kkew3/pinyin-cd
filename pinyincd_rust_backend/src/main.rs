use std::path::{Path, PathBuf, Component};
use std::collections::{HashSet, VecDeque};
use std::env;
use pinyin::ToPinyinMulti;

enum PinyinStyle {
    Plain,
    FirstLetter,
}

fn main() {
    let mut args = env::args().skip(1);
    let args_i = !args.next().unwrap().is_empty();
    let args_p = !args.next().unwrap().is_empty();
    let args_pattern = PathBuf::from(args.next().unwrap());
    let result = resolve(&args_pattern, args_i, args_p);
    for path in &result {
        match path.to_str() {
            Some(path) => println!("{}", path),
            None => eprintln!("pycd: ERROR: failed to print result {:?}", path.as_os_str()),
        }
    }
}


/// Reference: https://github.com/rust-lang/rfcs/issues/2208#issuecomment-342679694
fn normalize(p: &Path) -> PathBuf {
    let mut stack: Vec<Component> = Vec::new();

    // We assume .components() removes redundant consecutive path separators.
    // Note that .components() also does some normalization of '.' on its own anyways.
    // This '.' normalization happens to be compatible with the approach below.
    for component in p.components() {
        match component {
            // Drop CurDir components, do not even push onto the stack.
            Component::CurDir => {}

            // For ParentDir components, we need to use the contents of the stack.
            Component::ParentDir => {
                // Look at the top element of stack, if any.
                let top = stack.last().cloned();

                match top {
                    // A component is on the stack, need more pattern matching.
                    Some(c) => {
                        match c {
                            // Push the ParentDir on the stack.
                            Component::Prefix(_) => { stack.push(component); }

                            // The parent of a RootDir is itself, so drop the ParentDir (no-op).
                            Component::RootDir => {}

                            // A CurDir should never be found on the stack, since they are dropped when seen.
                            Component::CurDir => { unreachable!(); }

                            // If a ParentDir is found, it must be due to it piling up at the start of a path.
                            // Push the new ParentDir onto the stack.
                            Component::ParentDir => { stack.push(component); }

                            // If a Normal is found, pop it off.
                            Component::Normal(_) => { let _ = stack.pop(); }
                        }
                    }

                    // Stack is empty, so path is empty, just push.
                    None => { stack.push(component); }
                }
            }

            // All others, simply push onto the stack.
            _ => { stack.push(component); }
        }
    }

    // If an empty PathBuf would be return, instead return CurDir ('.').
    if stack.is_empty() {
        let c: &Path = Component::CurDir.as_ref();
        return PathBuf::from(c);
    }

    let mut norm_path = PathBuf::new();

    for item in &stack {
        let c: &Path = item.as_ref();
        norm_path.push(c);
    }

    norm_path
}


fn to_pinyin(string: &str, style: &PinyinStyle) -> Vec<String> {
    let mut pools: Vec<HashSet<String>> = Vec::new();

    for ch in string.chars() {
        let mut p = HashSet::new();
        match ch.to_pinyin_multi() {
            Some(multi) => {
                for pinyin in multi {
                    p.insert(String::from(
                        match style {
                            PinyinStyle::Plain => pinyin.plain(),
                            PinyinStyle::FirstLetter => pinyin.first_letter(),
                        }
                    ).replace("ü", "v"));
                }
            }
            None => {
                p.insert(String::from(ch));
            }
        }
        pools.push(p);
    }

    let mut result: Vec<Vec<&str>>;
    if !pools.is_empty() {
        result = vec![vec![]];
        for p in &pools {
            let mut next_result: Vec<Vec<&str>> = Vec::new();
            for x in result {
                for y in p {
                    let mut e = x.clone();
                    e.push(y);
                    next_result.push(e);
                }
            }
            result = next_result;
        }
    } else {
        result = vec![];
    }

    let mut joined_result: Vec<String> = Vec::new();
    for e in &result {
        joined_result.push(e.join(""));
    }

    joined_result
}


fn get_first_split_pattern(path: &Path) -> (PathBuf, Vec<Component>) {
    let mut basedir = PathBuf::new();
    let mut pattern = Vec::new();
    let mut in_basedir = true;
    for component in path.components() {
        if in_basedir {
            match component {
                Component::Prefix(_) | Component::ParentDir => {
                    basedir.push(component);
                }
                Component::RootDir | Component::CurDir => {
                    basedir.push(component);
                    in_basedir = false;
                }
                Component::Normal(_) => {
                    in_basedir = false;
                    pattern.push(component);
                }
            }
        } else {
            pattern.push(component);
        }
    }
    if basedir.as_os_str().is_empty() {
        basedir.push(Component::CurDir);
    }

    (basedir, pattern)
}


fn resolve(path: &PathBuf, pinyin_firstletter: bool, prefix: bool) -> Vec<PathBuf> {
    if path.as_os_str().is_empty() {
        return match home::home_dir() {
            Some(path) => {
                vec![path]
            }
            None => {
                eprintln!("pycd: ERROR: failed to fetch home directory");
                vec![]
            }
        };
    }

    let path = normalize(path);

    let style = match pinyin_firstletter {
        true => PinyinStyle::FirstLetter,
        false => PinyinStyle::Plain,
    };
    let mut matched_directories = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(get_first_split_pattern(&path));
    while !queue.is_empty() {
        let (basedir, pattern) = queue.pop_front().unwrap();
        if pattern.is_empty() {
            matched_directories.push(basedir);
        } else {
            let cur_pattern = match pattern[0].as_os_str().to_str() {
                Some(s) => s,
                None => {
                    eprintln!("pycd: ERROR: pattern[0] {:?} not utf-8", pattern[0].as_os_str());
                    return matched_directories;
                }
            };
            let readdir_it = match basedir.read_dir() {
                Ok(it) => it,
                Err(_) => {
                    eprintln!("pycd: ERROR: read_dir call failed");
                    return matched_directories;
                }
            };
            for entry in readdir_it {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => {
                        eprintln!("pycd: ERROR: read_dir iterating failed");
                        continue;
                    }
                };
                let is_dir = match entry.file_type() {
                    Ok(ft) => ft.is_dir(),
                    Err(_) => true,
                };
                if is_dir {
                    let entry_name_os = entry.file_name();
                    let entry_name = match entry_name_os.to_str() {
                        Some(name) => name,
                        None => {
                            eprintln!("pycd: ERROR: entry.file_name() {:?} not utf-8", entry_name_os);
                            continue;
                        }
                    };
                    let pinyins = to_pinyin(entry_name, &style);
                    for py in &pinyins {
                        if prefix && py.starts_with(cur_pattern) || (!prefix && py == cur_pattern) {
                            let mut next_basedir = basedir.clone();
                            next_basedir.push(entry_name_os);
                            let mut next_pattern = pattern.clone();
                            next_pattern.remove(0);
                            queue.push_back((next_basedir, next_pattern));
                            break;
                        }
                    }
                }
            }
        }
    }

    matched_directories
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn test_normalize() {
        let p = PathBuf::from("../../home/thatsgobbles/././music/../code/..");
        let np = normalize(&p);
        assert_eq!(np, PathBuf::from("../../home/thatsgobbles"));

        let p = PathBuf::from("/home//thatsgobbles/music/");
        let np = normalize(&p);
        assert_eq!(np, PathBuf::from("/home/thatsgobbles/music"));

        let p = PathBuf::from("/../../home/thatsgobbles/././code/../music/..");
        let np = normalize(&p);
        assert_eq!(np, PathBuf::from("/home/thatsgobbles"));

        let p = PathBuf::from("..");
        let np = normalize(&p);
        assert_eq!(np, PathBuf::from(".."));

        let p = PathBuf::from("/..");
        let np = normalize(&p);
        assert_eq!(np, PathBuf::from("/"));

        let p = PathBuf::from("../");
        let np = normalize(&p);
        assert_eq!(np, PathBuf::from(".."));

        let p = PathBuf::from("/");
        let np = normalize(&p);
        assert_eq!(np, PathBuf::from("/"));

        let p = PathBuf::new();
        let np = normalize(&p);
        assert_eq!(np, PathBuf::from("."));
    }

    #[test]
    fn test_to_pinyin() {
        let mut p = to_pinyin("", &PinyinStyle::Plain);
        p.sort();
        let expected: Vec<String> = vec![];
        assert_eq!(p, expected);

        let mut p = to_pinyin("shell", &PinyinStyle::Plain);
        p.sort();
        let mut expected: Vec<String> = vec!["shell".to_string()];
        expected.sort();
        assert_eq!(p, expected);

        let mut p = to_pinyin("sh中心ll", &PinyinStyle::Plain);
        p.sort();
        let mut expected: Vec<String> = vec!["shzhongxinll".to_string()];
        expected.sort();
        assert_eq!(p, expected);

        let mut p = to_pinyin("sh折扣ll", &PinyinStyle::Plain);
        p.sort();
        let mut expected: Vec<String> = vec![
            "shzhekoull".to_string(),
            "shshekoull".to_string(),
            "shtikoull".to_string(),
        ];
        expected.sort();
        assert_eq!(p, expected);


        let mut p = to_pinyin("shell", &PinyinStyle::FirstLetter);
        p.sort();
        let mut expected: Vec<String> = vec!["shell".to_string()];
        expected.sort();
        assert_eq!(p, expected);

        let mut p = to_pinyin("sh中心ll", &PinyinStyle::FirstLetter);
        p.sort();
        let mut expected: Vec<String> = vec!["shzxll".to_string()];
        expected.sort();
        assert_eq!(p, expected);

        let mut p = to_pinyin("sh折扣ll", &PinyinStyle::FirstLetter);
        p.sort();
        let mut expected: Vec<String> = vec![
            "shzkll".to_string(),
            "shskll".to_string(),
            "shtkll".to_string(),
        ];
        expected.sort();
        assert_eq!(p, expected);
    }

    #[test]
    fn test_get_first_split_pattern() {
        let path = PathBuf::from("../../hello/world");
        let (basedir, pattern) = get_first_split_pattern(&path);
        assert_eq!(basedir, PathBuf::from("../.."));
        assert_eq!(pattern, vec![
            Component::Normal(OsStr::new("hello")),
            Component::Normal(OsStr::new("world")),
        ]);

        let path = PathBuf::from("hello/world/again");
        let (basedir, pattern) = get_first_split_pattern(&path);
        assert_eq!(basedir, PathBuf::from("."));
        assert_eq!(pattern, vec![
            Component::Normal(OsStr::new("hello")),
            Component::Normal(OsStr::new("world")),
            Component::Normal(OsStr::new("again")),
        ]);

        let path = PathBuf::from("../hello");
        let (basedir, pattern) = get_first_split_pattern(&path);
        assert_eq!(basedir, PathBuf::from(".."));
        assert_eq!(pattern, vec![
            Component::Normal(OsStr::new("hello")),
        ]);

        let path = PathBuf::from("../..");
        let (basedir, pattern) = get_first_split_pattern(&path);
        assert_eq!(basedir, PathBuf::from("../.."));
        assert!(pattern.is_empty());

        let path = PathBuf::from("hello");
        let (basedir, pattern) = get_first_split_pattern(&path);
        assert_eq!(basedir, PathBuf::from("."));
        assert_eq!(pattern, vec![
            Component::Normal(OsStr::new("hello")),
        ]);

        let path = PathBuf::from("/");
        let (basedir, pattern) = get_first_split_pattern(&path);
        assert_eq!(basedir, PathBuf::from("/"));
        assert!(pattern.is_empty());

        let path = PathBuf::from("/hello");
        let (basedir, pattern) = get_first_split_pattern(&path);
        assert_eq!(basedir, PathBuf::from("/"));
        assert_eq!(pattern, vec![
            Component::Normal(OsStr::new("hello")),
        ]);

        let path = PathBuf::from(".");
        let (basedir, pattern) = get_first_split_pattern(&path);
        assert_eq!(basedir, PathBuf::from("."));
        assert!(pattern.is_empty());
    }

    #[test]
    fn test_resolve() {
        let p = PathBuf::from("test");
        let result = resolve(&p, false, false);
        assert_eq!(result, vec![PathBuf::from("./test")]);

        let p = PathBuf::from("te");
        let result = resolve(&p, false, false);
        assert!(result.is_empty());

        let p = PathBuf::from("te");
        let result = resolve(&p, false, true);
        assert_eq!(result, vec![PathBuf::from("./test")]);

        let p = PathBuf::from("test/zhongxin/zhekoush");
        let result = resolve(&p, false, false);
        assert_eq!(result, vec![PathBuf::from("./test/中心/折扣sh")]);

        let p = PathBuf::from("test/zhongxin/shekoush");
        let result = resolve(&p, false, false);
        assert_eq!(result, vec![PathBuf::from("./test/中心/折扣sh")]);

        let p = PathBuf::from("test/zx/zksh");
        let result = resolve(&p, true, false);
        assert_eq!(result, vec![PathBuf::from("./test/中心/折扣sh")]);

        let p = PathBuf::from("te/zho/zhek");
        let result = resolve(&p, false, true);
        assert_eq!(result, vec![PathBuf::from("./test/中心/折扣sh")]);

        let p = PathBuf::from("te/z/zk");
        let result = resolve(&p, true, true);
        assert_eq!(result, vec![PathBuf::from("./test/中心/折扣sh")]);

        let p = PathBuf::from("te/z/s");
        let mut result = resolve(&p, false, true);
        result.sort();
        let mut expected_result = vec![
            PathBuf::from("./test/中心/蛇"),
            PathBuf::from("./test/中心/折扣sh"),
        ];
        expected_result.sort();
        assert_eq!(result, expected_result);

        let p = PathBuf::from("te/z/s");
        let mut result = resolve(&p, true, true);
        result.sort();
        let mut expected_result = vec![
            PathBuf::from("./test/中心/蛇"),
            PathBuf::from("./test/中心/折扣sh"),
        ];
        expected_result.sort();
        assert_eq!(result, expected_result);

        let p = PathBuf::from("../pinyincd_rust_backend/test/weituomapinyin");
        let result = resolve(&p, false, false);
        assert_eq!(result, vec![PathBuf::from("../pinyincd_rust_backend/test/威妥玛拼音")]);

        let p = PathBuf::from("test/weituomapinyin/zhanlve");
        let result = resolve(&p, false, false);
        assert_eq!(result, vec![PathBuf::from("./test/威妥玛拼音/战略")]);
    }
}