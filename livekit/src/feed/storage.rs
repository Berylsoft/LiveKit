use sled::*;

pub fn open_storage<P: AsRef<std::path::Path>>(path: P) -> Result<Db> {
    Config::new().path(path).open()
}
