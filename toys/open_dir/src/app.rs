use std::os::unix::fs::FileTypeExt;

pub enum FileType {
    File,
    Directory,
    SymbolicFile,
    BlockDevice,
    CharDevice,
    Fifo,
    Socket,
    Other,
}

pub struct App {
    pub contents: Vec<(String, FileType)>,
    pub curr_location: std::path::PathBuf,
    pub curr_line: u16,
    pub view_line_start: u16,
}

impl App {
    pub fn new() -> Self {
        App {
            contents: vec![], // contents list
            curr_location: std::path::PathBuf::from("./").canonicalize().unwrap(),
            curr_line: 0, // line (index) of contents list
            view_line_start: 0,
        }
    }

    fn get_file_type(path_str: &str) -> std::io::Result<FileType> {
        let metadata = match std::fs::metadata(path_str) {
            Ok(mt) => mt,
            Err(_) => {
                return Ok(FileType::Other);
            }
        };
        let file_type = metadata.file_type();

        if file_type.is_block_device() {
            Ok(FileType::BlockDevice)
        } else if file_type.is_char_device() {
            Ok(FileType::CharDevice)
        } else if file_type.is_fifo() {
            Ok(FileType::Fifo)
        } else if file_type.is_socket() {
            Ok(FileType::Socket)
        } else {
            Ok(FileType::Other)
        }
    }

    pub fn create_list_by_location(
        location: &std::path::PathBuf,
    ) -> Result<Vec<(String, FileType)>, std::io::Error> {
        // 1) get conents list of this location
        let mut paths = location.read_dir().unwrap();

        // 2) vector push: all stuffs in this location
        let mut contents_vec = Vec::<(String, FileType)>::new();
        if location.as_os_str() != std::path::Component::RootDir.as_os_str() {
            contents_vec.push((String::from(".."), FileType::Directory));
        }
        for result_path in &mut paths {
            let path = result_path.unwrap().path();

            let file_type = if path.is_symlink() {
                FileType::SymbolicFile
            } else if path.is_file() {
                FileType::File
            } else if path.is_dir() {
                FileType::Directory
            } else {
                App::get_file_type(path.to_str().unwrap()).unwrap()
            };

            contents_vec.push((
                String::from(path.file_name().unwrap().to_str().unwrap()),
                file_type,
            ));
        }

        Ok(contents_vec)
    }

    pub fn init(&mut self) -> Result<(), std::io::Error> {
        // 1) create current location's contents list
        self.contents = App::create_list_by_location(&self.curr_location)?;

        // 2) sort vector
        let sort_start_idx =
            if self.curr_location.as_os_str() == std::path::Component::RootDir.as_os_str() {
                0
            } else {
                1
            };
        let slice = &mut self.contents[sort_start_idx..];
        slice.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(())
    }

    pub fn change_directory(&mut self) -> Result<bool, std::io::Error> {
        if self.curr_line == 0
            && self.curr_location.as_os_str() != std::path::Component::RootDir.as_os_str()
        {
            self.curr_location.pop();
            return Ok(true);
        }

        let next_content = &self.contents[self.curr_line as usize];
        let path_string = next_content.0.clone();
        let file_type = &next_content.1;

        let ret = match file_type {
            FileType::Directory | FileType::SymbolicFile => Ok(true),
            _ => Ok(false),
        };

        if let Ok(true) = ret {
            let mut path = self.curr_location.clone();
            path.push(path_string);

            if !path.is_dir() {
                return Ok(false);
            }

            self.curr_location = path;
            self.curr_line = 0;
            self.view_line_start = 0;
        }

        ret
    }
}
