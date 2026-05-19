use msedge_tts::voice::get_voices_list_proxy;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let voices = get_voices_list_proxy("http://127.0.0.1:7897").unwrap();
    println!("{:#?}", voices);
    println!("{:?}", Instant::now() - start);
}
