use msedge_tts::voice::get_voices_list_proxy;

fn main() {
    // socks4 proxy
    let voices = get_voices_list_proxy("socks4a://localhost:7897").unwrap();
    println!("{:#?}", voices);

    // socks5 proxy
    let voices = get_voices_list_proxy("socks5h://localhost:7897").unwrap();
    println!("{:#?}", voices);

    // http proxy
    let voices = get_voices_list_proxy("localhost:7897").unwrap();
    println!("{:#?}", voices);
}
