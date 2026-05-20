use msedge_tts::voice::tokio_runtime::get_voices_list_proxy_async;
use std::time::Instant;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let start = Instant::now();
            let voices = get_voices_list_proxy_async("http://127.0.0.1:7897")
                .await
                .unwrap();
            println!("{:#?}", voices);
            println!("{:?}", Instant::now() - start);
        })
}
