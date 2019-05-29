extern crate rspotify;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate dotenv;

use dotenv::dotenv;
use rspotify::spotify::client::Spotify;
use rspotify::spotify::util::get_token;
use rspotify::spotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth};
use std::thread;
use std::time::Duration;
use micro_http_server::MicroHTTP;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NAME: &'static str = env!("CARGO_PKG_NAME");

fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    info!("Started {} ({})", NAME, VERSION);

    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let tx1 = mpsc::Sender::clone(&tx);
    thread::spawn(move || {
        let server = MicroHTTP::new("127.0.0.1:65510").expect("Could not create server.");
        debug!("Listening Spotify OAuth redirect at http://127.0.0.1:65510/");
        loop {
            let result = server.next_client();
            if result.is_err() {
                error!("Local response server rerver failed: {:?}", result);
                break;
            }

            match result.unwrap() {
                None => {},
                Some(mut client) => {
                    if client.request().is_none() {
                        debug!("Client {} did not request anything", client.addr());
                        client.respond_ok("No request :(".as_bytes()).expect("Could not send data to client!");
                    } else {
                        let request_copy = client.request().as_ref().unwrap().clone();
                        debug!("{:?}", request_copy);
                        client.respond_ok("Scotty has received token from Spotify. You can now close this window :)".as_bytes()).unwrap();
                        tx.send(request_copy).unwrap();
                    }
                }
            };

            let ctrl_message = rx.recv_timeout(Duration::from_secs(1)).unwrap_or("timeout".to_string());
            if ctrl_message == "done" {
                debug!("Received 'done' signal from main thread. Closing local http server.");
                break;
            }
        }
        trace!("Shutting down local webserver");
    });

    let mut oauth = SpotifyOAuth::default()
        .scope("user-read-recently-played")
        .build();

    match get_token(&mut oauth) {
        Some(token_info) => {
            let client_credential = SpotifyClientCredentials::default()
                .token_info(token_info)
                .build();
            tx1.send("done".to_string()).unwrap();
            let spotify = Spotify::default()
                .client_credentials_manager(client_credential)
                .build();
            let _history = spotify.current_user_recently_played(10);
            //println!("{:?}", history);
        },
        None => error!("Auth / authz error.")
    };
}
