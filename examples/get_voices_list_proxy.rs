use msedge_tts::voice::get_voices_list_proxy;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let voices = get_voices_list_proxy("http://localhost:7897").unwrap();
    println!("{:#?}", voices);
    println!("{:?}", Instant::now() - start);
}
