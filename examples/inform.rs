extern crate mediainfo;

use mediainfo::MediaInfo;
use std::path::PathBuf;

fn main() {
    let mut media_info = MediaInfo::new();

    let sample_path = PathBuf::from("samples");
    let extnames = ["mp3", "m4a", "flac"];

    for ext in extnames.iter() {
        let filename = sample_path.join(format!("sample.{}", ext));

        media_info
            .open(&filename)
            .expect("It should open the file.");

        let result = media_info.option("output", "JSON");
        println!("option result: {:?}", result);

        println!("{}\n", media_info.inform().unwrap());

        // println!("Filename: {}", filename.to_str().as_ref().unwrap());

        media_info.close();
    }
}
