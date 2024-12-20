use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from("./src/generated");
    fs::create_dir_all(&out_dir)?;
    tonic_build::configure()
        .out_dir(&out_dir)
        .message_attribute(".", "#[derive(Deserialize, Serialize)]")
        .file_descriptor_set_path(out_dir.join("reflection.bin"))
        .build_server(true)
        .build_client(false)
        .compile_protos(&["moneyview.proto"], &["proto"])?;

    let string_to_add = "use serde::{Serialize, Deserialize};";
    // Lese den Ordner und iteriere über jede Datei
    for entry in fs::read_dir(out_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Überprüfe, ob der Eintrag eine Datei ist und die Endung .rs hat
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            // Lese den bestehenden Inhalt der Datei
            let mut file_content = String::new();
            {
                let mut file = fs::File::open(&path)?;
                file.read_to_string(&mut file_content)?;
            }

            // Öffne die Datei im Schreibmodus, um sie neu zu schreiben
            let mut file = OpenOptions::new().write(true).truncate(true).open(&path)?;

            // Füge den neuen String an den Anfang und den ursprünglichen Inhalt dahinter
            file.write_all(string_to_add.as_bytes())?;
            file.write_all(file_content.as_bytes())?;
        }
    }

    Ok(())
}
