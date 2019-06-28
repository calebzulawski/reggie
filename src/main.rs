mod handler;

fn main() {
    let matches = clap::App::new("reggie")
        .arg(
            clap::Arg::with_name("key")
                .short("k")
                .long("key")
                .value_name("KEY")
                .required(true)
                .takes_value(true)
                .help("Set the API key"),
        )
        .get_matches();

    let cli = slack::RtmClient::login(&matches.value_of("key").unwrap()).unwrap();
    let r = cli.run(&mut handler::Handler::new(&cli));

    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}
