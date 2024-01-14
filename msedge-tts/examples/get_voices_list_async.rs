use msedge_tts::voice::get_voices_list_async;
use std::time::Instant;

fn main() {
    smol::block_on(async {
        let start = Instant::now();
        let voices = get_voices_list_async().await.unwrap();
        println!("{:#?}", voices);
        println!("{:?}", Instant::now() - start);
    });
}
