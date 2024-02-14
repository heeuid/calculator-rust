fn extract_name(path: &String) -> String {
    String::from(path.split_inclusive("/").last().unwrap())
}

enum FileType {
    Symbolic,
    File,
    Directory,
    None,
}

fn check_file_type(path: &mut String, depth: u32) -> FileType {
    // get metadata for symbolic check
    let metainfo_symlink = match std::fs::symlink_metadata(&path) {
        Ok(p) => p,
        Err(_) => {
            return FileType::None;
        }
    };

    // get metadata for file and dir check
    let metainfo = match std::fs::metadata(&path) {
        Ok(p) => p,
        Err(_) => {
            return FileType::None;
        }
    };

    // is symbolic?
    if metainfo_symlink.is_symlink() && depth > 0 {
        let target_path = std::fs::read_link(&path).unwrap();

        // extract a name in path
        *path = extract_name(&path);

        *path += " -> ";
        *path += target_path.as_path().to_str().unwrap();

        return FileType::Symbolic;
    }

    // extract a name in path
    *path = extract_name(path);

    // is file?
    if metainfo.is_file() {
        return FileType::File;
    // or direcotry?
    } else if path.chars().last().unwrap() != '/' {
        *path += "/";
    }

    return FileType::Directory;
}

fn fill_contents_vec(
    path_prefix: &mut String,
    path_vec: &Vec<String>,
    contents: &mut Vec<(String, String)>,
    depth: u32,
) -> (u32, u32) {
    let mut ret_cnt = (0 as u32, 0 as u32);
    let mut i = 0 as usize;

    for path in path_vec {
        let prefix_last_piece: &str;
        if i == path_vec.len() - 1 {
            prefix_last_piece = "└───";
        } else {
            prefix_last_piece = "├───";
        }

        // fill contents
        contents.push((
            String::from(path_prefix.as_str()) + prefix_last_piece, //"+---",
            String::from(path),
        ));

        // set next-next prefix
        if i == path_vec.len() - 1 {
            *path_prefix += "    ";
        } else {
            *path_prefix += "│   "; //"|   ";
        }

        // recursive searching
        let temp_cnt = create_dir_contents_vec(path_prefix, path, depth + 1, contents);

        ret_cnt.0 += temp_cnt.0;
        ret_cnt.1 += temp_cnt.1;

        // reback to next prefix
        for _ in 0..4 {
            path_prefix.pop();
        }

        i += 1;
    }

    ret_cnt
}

fn create_dir_contents_vec(
    path_prefix: &mut String,
    path_str: &String,
    depth: u32,
    contents: &mut Vec<(String, String)>,
) -> (u32, u32) {
    let mut ret_cnt_dir_file: (u32, u32) = (0, 0);

    let mut last_path = &mut contents.last_mut().unwrap().1;

    match check_file_type(&mut last_path, depth) {
        FileType::Symbolic => {
            ret_cnt_dir_file.1 += 1;
            return ret_cnt_dir_file;
        }
        FileType::File => {
            ret_cnt_dir_file.1 += 1;
            return ret_cnt_dir_file;
        }
        FileType::Directory => {
            ret_cnt_dir_file.0 += 1;
        }
        FileType::None => {
            return ret_cnt_dir_file;
        }
    }

    // flush tree from contents vector
    if contents.len() >= 500 {
        let tree = create_contents_tree(&contents);

        if let Some(t) = tree {
            print!("{}", t);
        }
        contents.clear();
    }

    // get contents in this directory
    let mut paths = match std::fs::read_dir(path_str) {
        Ok(t) => t,
        Err(_) => {
            // character device, ...
            ret_cnt_dir_file.1 += 1;
            return ret_cnt_dir_file;
        }
    };

    // push files and directories in this directory into path_vec
    let mut path_vec = Vec::<String>::new();
    for path in &mut paths {
        path_vec.push(String::from(path.unwrap().path().to_str().unwrap()));
    }

    path_vec.sort();

    // fill contents vector with sorted path_vec
    let temp_cnt = fill_contents_vec(path_prefix, &path_vec, contents, depth);

    // count files and directories
    ret_cnt_dir_file.0 += temp_cnt.0;
    ret_cnt_dir_file.1 += temp_cnt.1;

    ret_cnt_dir_file
}

fn create_contents_tree(contents: &Vec<(String, String)>) -> Option<String> {
    let mut tree = String::new();

    if contents.is_empty() {
        return None;
    }

    tree += &contents[0].0;
    tree += &contents[0].1;
    tree += "\n";

    for (prefix, path) in contents {
        tree += prefix;
        tree += path;
        tree += "\n";
    }

    Some(tree)
}

fn print_help(cmd: &String) {
    println!("USAGE: {} [<path> or --help]", cmd);
}

fn main() -> Result<(), String> {
    let mut contents = Vec::<(String, String)>::new();

    // check argument 1
    let arg1 = match std::env::args().nth(1) {
        Some(p) => p,
        None => String::from("./"),
    };

    // check if --help argument
    let path: &String;
    if arg1 == "--help" {
        print_help(&std::env::args().nth(0).unwrap());
        return Ok(());
    } else {
        path = &arg1;
    }

    // check the # of arguments
    match std::env::args().nth(2) {
        Some(_) => {
            println!("USAGE: {} [<path> or --help]", path);
            return Err(String::from(
                "Add one or zero argument as a directory to search",
            ));
        }
        None => {}
    }

    //is path (an argument) directory?
    match std::fs::metadata(path) {
        Ok(t) => {
            if !t.is_dir() {
                println!("USAGE: {} [<path> or --help]", path);
                return Err(format!("{} is not a directory.", path));
            }
        }
        Err(_) => {
            println!("USAGE: {} [<path> or --help]", path);
            return Err(format!("{} is not a directory.", path));
        }
    }

    // root direcotry to search
    contents.push((String::new(), String::from(path)));

    // search and get a vector of files and diretories to make a tree
    let cnt_dir_file = create_dir_contents_vec(&mut String::from(""), path, 0, &mut contents);

    // make a tree
    let tree = create_contents_tree(&contents);

    if let Some(t) = tree {
        print!("{}", t);
    }

    println!("\n{} direotries, {} files", cnt_dir_file.0, cnt_dir_file.1);

    Ok(())
}
