use std::fs;
pub struct FileData {
    file_id: u16,
    filename: String,
}

pub struct Compiler {
    pub source_files: Vec<FileData>,
}

pub fn get_line(filename: &str, line_no: usize) -> Option<String> {
    for (i, lines) in fs::read_to_string(filename).ok()?.lines().enumerate() {
        if i + 1 == line_no {
            return Some(lines.to_string());
        }
    }
    return None;
}
impl FileData {
    pub fn new(file_id: u16, filename: &str) -> Self {
        let filename = filename.to_string();
        Self { file_id, filename }
    }
    pub fn get_file_id(&self) -> u16 {
        self.file_id
    }
    pub fn get_filename(&self) -> String {
        self.filename.clone()
    }
    pub fn change_file_id(&mut self, new_file_id: u16) {
        self.file_id = new_file_id;
    }
    pub fn change_filename(&mut self, new_filename: &str) {
        self.filename = new_filename.to_string();
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            source_files: Vec::new(),
        }
    }
    pub fn is_file_present(&self, file_id: u16) -> bool {
        for i in self.source_files.iter() {
            if i.get_file_id() == file_id {
                return true;
            }
        }
        return false;
    }
    pub fn add_file(&mut self, file: FileData) -> bool {
        if self.is_file_present(file.file_id) == false {
            self.source_files.push(file);
            return true;
        }
        return false;
    }
    pub fn get_filename(&self, file_id: u16) -> Option<String> {
        for i in self.source_files.iter() {
            if i.get_file_id() == file_id {
                return Some(i.get_filename());
            }
        }
        return None;
    }
}
