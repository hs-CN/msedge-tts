use msedge_tts::voice::get_voices_list_async;

fn main() {
    smol::block_on(async {
        let voices = get_voices_list_async().await.unwrap();
        println!("{:#?}", voices);
    });
}
