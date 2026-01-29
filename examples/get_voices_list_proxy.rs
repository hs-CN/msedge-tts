use msedge_tts::voice::get_voices_list_proxy;

fn main() {
    // socks4 proxy
    let voices = get_voices_list_proxy("socks4a://localhost:10808", None, None).unwrap();
    println!("{:#?}", voices);

    // socks5 proxy
    let voices = get_voices_list_proxy("socks5h://localhost:10808", None, None).unwrap();
    println!("{:#?}", voices);

    // http proxy
    let voices = get_voices_list_proxy("localhost:10809", None, None).unwrap();
    println!("{:#?}", voices);
}
