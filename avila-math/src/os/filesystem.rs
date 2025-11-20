use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Abstração de filesystem com operações comuns
pub struct FileSystem;

impl FileSystem {
    /// Lê um arquivo completo como string
    pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
        fs::read_to_string(path)
    }

    /// Lê um arquivo completo como bytes
    pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    /// Escreve string em arquivo (sobrescreve)
    pub fn write<P: AsRef<Path>>(path: P, contents: impl AsRef<[u8]>) -> io::Result<()> {
        fs::write(path, contents)
    }

    /// Anexa conteúdo ao final do arquivo
    pub fn append<P: AsRef<Path>>(path: P, contents: impl AsRef<[u8]>) -> io::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(contents.as_ref())
    }

    /// Copia arquivo
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
        fs::copy(from, to)
    }

    /// Move/renomeia arquivo
    pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
        fs::rename(from, to)
    }

    /// Remove arquivo
    pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
        fs::remove_file(path)
    }

    /// Cria diretório (e pais se necessário)
    pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    /// Remove diretório (recursivo)
    pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
        fs::remove_dir_all(path)
    }

    /// Verifica se arquivo existe
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Verifica se é arquivo
    pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_file()
    }

    /// Verifica se é diretório
    pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_dir()
    }

    /// Lista entradas de um diretório
    pub fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<Vec<PathBuf>> {
        let entries = fs::read_dir(path)?;
        let mut paths = Vec::new();

        for entry in entries {
            if let Ok(entry) = entry {
                paths.push(entry.path());
            }
        }

        Ok(paths)
    }

    /// Obtém metadados de arquivo/diretório
    pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<FileMetadata> {
        let meta = fs::metadata(path)?;
        Ok(FileMetadata::from_std(meta))
    }

    /// Cria um link simbólico
    #[cfg(unix)]
    pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
        std::os::unix::fs::symlink(src, dst)
    }

    #[cfg(windows)]
    pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
        std::os::windows::fs::symlink_file(src, dst)
    }
}

/// Metadados de arquivo
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub readonly: bool,
}

impl FileMetadata {
    fn from_std(meta: fs::Metadata) -> Self {
        Self {
            size: meta.len(),
            is_file: meta.is_file(),
            is_dir: meta.is_dir(),
            is_symlink: meta.is_symlink(),
            readonly: meta.permissions().readonly(),
        }
    }
}

/// File handle com buffer e operações convenientes
pub struct FileHandle {
    file: File,
    path: PathBuf,
}

impl FileHandle {
    /// Abre arquivo para leitura
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let file = File::open(path)?;
        Ok(Self {
            file,
            path: path_buf,
        })
    }

    /// Cria arquivo para escrita (sobrescreve)
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let file = File::create(path)?;
        Ok(Self {
            file,
            path: path_buf,
        })
    }

    /// Abre com opções customizadas
    pub fn open_with_options<P: AsRef<Path>>(
        path: P,
        read: bool,
        write: bool,
        create: bool,
        append: bool,
    ) -> io::Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .read(read)
            .write(write)
            .create(create)
            .append(append)
            .open(path)?;
        Ok(Self {
            file,
            path: path_buf,
        })
    }

    /// Retorna o caminho do arquivo
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Obtém metadados
    pub fn metadata(&self) -> io::Result<FileMetadata> {
        let meta = self.file.metadata()?;
        Ok(FileMetadata::from_std(meta))
    }

    /// Cria um reader com buffer
    pub fn reader(self) -> BufReader<File> {
        BufReader::new(self.file)
    }

    /// Cria um writer com buffer
    pub fn writer(self) -> BufWriter<File> {
        BufWriter::new(self.file)
    }
}

impl Read for FileHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for FileHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for FileHandle {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

/// Path utilities
pub struct PathUtil;

impl PathUtil {
    /// Obtém nome do arquivo
    pub fn filename<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }

    /// Obtém extensão do arquivo
    pub fn extension<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string())
    }

    /// Obtém diretório pai
    pub fn parent<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
        path.as_ref().parent().map(|p| p.to_path_buf())
    }

    /// Junta caminhos
    pub fn join<P: AsRef<Path>, Q: AsRef<Path>>(base: P, path: Q) -> PathBuf {
        base.as_ref().join(path)
    }

    /// Converte para caminho absoluto
    pub fn canonicalize<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
        fs::canonicalize(path)
    }

    /// Obtém diretório de trabalho atual
    pub fn current_dir() -> io::Result<PathBuf> {
        std::env::current_dir()
    }

    /// Define diretório de trabalho atual
    pub fn set_current_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
        std::env::set_current_dir(path)
    }
}

/// Directory walker - itera recursivamente por diretórios
pub struct DirectoryWalker {
    stack: Vec<PathBuf>,
    recursive: bool,
}

impl DirectoryWalker {
    pub fn new<P: AsRef<Path>>(root: P, recursive: bool) -> io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        if !root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path is not a directory",
            ));
        }

        Ok(Self {
            stack: vec![root],
            recursive,
        })
    }

    pub fn walk<F>(&mut self, mut callback: F) -> io::Result<()>
    where
        F: FnMut(&Path, &FileMetadata) -> io::Result<bool>,
    {
        while let Some(path) = self.stack.pop() {
            let entries = fs::read_dir(&path)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                let meta = FileMetadata::from_std(entry.metadata()?);

                if callback(&path, &meta)? {
                    if meta.is_dir && self.recursive {
                        self.stack.push(path);
                    }
                }
            }
        }

        Ok(())
    }
}

/// File watcher para monitorar mudanças (simplificado)
pub struct FileWatcher {
    path: PathBuf,
    last_modified: Option<std::time::SystemTime>,
}

impl FileWatcher {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let last_modified = fs::metadata(&path)?.modified().ok();

        Ok(Self {
            path,
            last_modified,
        })
    }

    pub fn has_changed(&mut self) -> io::Result<bool> {
        let current_modified = fs::metadata(&self.path)?.modified().ok();

        if current_modified != self.last_modified {
            self.last_modified = current_modified;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_read_write() {
        let path = "test_file.txt";
        let content = "Hello, World!";

        FileSystem::write(path, content).unwrap();
        let read_content = FileSystem::read_to_string(path).unwrap();

        assert_eq!(content, read_content);

        FileSystem::remove_file(path).unwrap();
    }

    #[test]
    fn test_file_handle() {
        let path = "test_handle.txt";

        {
            let mut handle = FileHandle::create(path).unwrap();
            handle.write_all(b"test").unwrap();
        }

        {
            let mut handle = FileHandle::open(path).unwrap();
            let mut buffer = String::new();
            handle.read_to_string(&mut buffer).unwrap();
            assert_eq!(buffer, "test");
        }

        FileSystem::remove_file(path).unwrap();
    }

    #[test]
    fn test_path_util() {
        let path = PathBuf::from("test/dir/file.txt");

        assert_eq!(PathUtil::filename(&path), Some("file.txt".to_string()));
        assert_eq!(PathUtil::extension(&path), Some("txt".to_string()));
    }
}
