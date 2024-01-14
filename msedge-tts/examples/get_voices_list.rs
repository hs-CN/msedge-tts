use msedge_tts::voice::get_voices_list;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let voices = get_voices_list().unwrap();
    println!("{:#?}", voices);
    println!("{:?}", Instant::now() - start);
}
