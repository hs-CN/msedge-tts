use msedge_tts::voice::get_voices_list_async;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let voices = get_voices_list_async().await.unwrap();
    println!("{:#?}", voices);
    println!("{:?}", Instant::now() - start);
}
