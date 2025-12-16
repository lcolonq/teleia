use base64::prelude::*;

#[cfg(target_arch = "wasm32")]
pub fn save<W>(id: &str, data: &W) where W: serde::Serialize {
    let window = web_sys::window().expect("failed to get window object");
    let storage = window.local_storage()
        .expect("failed to get local storage")
        .expect("local storage not present");
    let key = format!("{}_save", id);
    let val = bincode::serde::encode_to_vec(data, bincode::config::standard()).expect("failed to serialize save");
    let str = BASE64_STANDARD.encode(&val);
    storage.set_item(&key, &str).expect("failed to set save");
}

#[cfg(target_arch = "wasm32")]
pub fn load<W>(id: &str) -> Option<W> where W: serde::de::DeserializeOwned {
    let window = web_sys::window().expect("failed to get window object");
    let storage = window.local_storage()
        .expect("failed to get local storage")
        .expect("local storage not present");
    let key = format!("{}_save", id);
    let s = storage.get_item(&key).expect("failed to get save").expect("save not present");
    let bytes = BASE64_STANDARD.decode(&s).expect("failed to decode base64 for save");
    let (ret, _) = bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
        .expect("failed to deserialize save");
    Some(ret)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save<W>(id: &str, data: &W) where W: serde::Serialize {
    let pd = directories::ProjectDirs::from("", "milkfat", id).expect("failed to get save directory");
    let _ = std::fs::create_dir_all(pd.data_dir());
    let path = pd.data_dir().join("teleia.save");
    let mut file = std::fs::File::create(&path).expect("failed to open save file");
    // serde_json::to_writer(file, data).expect("failed to write save file");
    bincode::serde::encode_into_std_write(data, &mut file, bincode::config::standard())
        .expect("failed to write save file");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load<W>(id: &str) -> Option<W> where W: serde::de::DeserializeOwned {
    let pd = directories::ProjectDirs::from("", "milkfat", id).expect("failed to get save directory");
    let _ = std::fs::create_dir_all(pd.data_dir());
    let path = pd.data_dir().join("teleia.save");
    let mut file = std::fs::File::open(&path).ok()?;
    // serde_json::from_reader(file).ok()
    bincode::serde::decode_from_std_read(&mut file, bincode::config::standard()).ok()
}
