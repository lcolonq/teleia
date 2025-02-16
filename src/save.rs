#[cfg(target_arch = "wasm32")]
pub fn save<W>(id: &str, data: &W) where W: serde::Serialize {
    let window = web_sys::window().expect("failed to get window object");
    let storage = window.local_storage()
        .expect("failed to get local storage")
        .expect("local storage not present");
    let key = format!("{}_save", id);
    let val = serde_json::to_string(data).expect("failed to serialize save");
    storage.set_item(&key, &val).expect("failed to set save");
}

#[cfg(target_arch = "wasm32")]
pub fn load<W>(id: &str) -> Option<W> where W: serde::de::DeserializeOwned {
    let window = web_sys::window().expect("failed to get window object");
    let storage = window.local_storage()
        .expect("failed to get local storage")
        .expect("local storage not present");
    let key = format!("{}_save", id);
    let s = storage.get_item(&key).expect("failed to get save").expect("save not present");
    let mut cur = std::io::Cursor::new(s);
    serde_json::from_reader(&mut cur).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save<W>(id: &str, data: &W) where W: serde::Serialize {
    let pd = directories::ProjectDirs::from("", "milkfat", id).expect("failed to get save directory");
    let _ = std::fs::create_dir_all(pd.data_dir());
    let path = pd.data_dir().join("teleia.save");
    let file = std::fs::File::create(&path).expect("failed to open save file");
    serde_json::to_writer(file, data).expect("failed to write save file");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load<W>(id: &str) -> Option<W> where W: serde::de::DeserializeOwned {
    let pd = directories::ProjectDirs::from("", "milkfat", id).expect("failed to get save directory");
    let _ = std::fs::create_dir_all(pd.data_dir());
    let path = pd.data_dir().join("teleia.save");
    let file = std::fs::File::open(&path).ok()?;
    serde_json::from_reader(file).ok()
}
