use msedge_tts::voice::get_voices_list;

fn main() {
    let voices = get_voices_list().unwrap();
    println!("{:#?}", voices);
}
