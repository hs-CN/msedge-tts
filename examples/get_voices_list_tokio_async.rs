use msedge_tts::voice::get_voices_list_async;
use std::time::Instant;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let start = Instant::now();
            let voices = get_voices_list_async().await.unwrap();
            println!("{:#?}", voices);
            println!("{:?}", Instant::now() - start);
        })
}
