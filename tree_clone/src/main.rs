fn extract_name(path: &String) -> String {
    String::from(path.split_inclusive("/").last().unwrap())
}

fn create_dir_contents_vec(
    path_prefix: &mut String,
    path_str: &String,
    depth: u32,
    contents: &mut Vec<(String, String)>,
) -> (u32, u32) {
    let mut ret_cnt_dir_file: (u32, u32) = (0, 0);

    let path = &mut contents.last_mut().unwrap().1;
    let metainfo = match std::fs::metadata(path_str) {
        Ok(p) => p,
        Err(_) => {
            return (0, 1);
        }
    };

    // extract a name in path
    *path = extract_name(path);

    // is file?
    if metainfo.is_file() {
        return (ret_cnt_dir_file.0, ret_cnt_dir_file.1 + 1);
    // or direcotry?
    } else if path.chars().last().unwrap() != '/' {
        *path += "/";
        ret_cnt_dir_file.0 += 1;
    }

    // get contents in this directory
    let mut paths = match std::fs::read_dir(path_str) {
        Ok(t) => t,
        Err(_) => {
            return ret_cnt_dir_file;
        }
    };

    let mut path_vec = Vec::<String>::new();
    for path in &mut paths {
        path_vec.push(String::from(path.unwrap().path().to_str().unwrap()));
    }

    path_vec.sort();

    let mut i = 0 as usize;
    for path in &path_vec {
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
        let temp = create_dir_contents_vec(path_prefix, path, depth + 1, contents);

        ret_cnt_dir_file.0 += temp.0;
        ret_cnt_dir_file.1 += temp.1;

        // reback to next prefix
        for _ in 0..4 {
            path_prefix.pop();
        }

        i += 1;
    }

    ret_cnt_dir_file
}

fn create_contents_tree(contents: &Vec<(String, String)>) -> Option<String> {
    let mut tree = String::new();

    if contents.is_empty() {
        return None;
    }

    tree += &contents[0].1;
    tree += "\n";

    for (prefix, path) in &contents[1..contents.len()] {
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
