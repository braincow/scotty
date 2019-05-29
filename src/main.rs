extern crate rspotify;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate dotenv;

use dotenv::dotenv;
use rspotify::spotify::client::Spotify;
use rspotify::spotify::util::{request_token, process_token};
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
    let (tx1, rx1): (Sender<bool>, Receiver<bool>) = mpsc::channel();
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

            if rx1.recv_timeout(Duration::from_secs(1)).unwrap_or(false) {
                debug!("Received 'done' signal from main thread. Closing local http server.");
                break;
            }
        }
        trace!("Shutting down local webserver");
    });

    let mut oauth = SpotifyOAuth::default()
        .scope("user-read-currently-playing")
        .build();

    if oauth.get_cached_token().is_none() {
        request_token(&mut oauth);
        process_token(&mut oauth, &mut rx.recv().unwrap());
    }
    let spotify: Spotify = match oauth.get_cached_token() {
        Some(token_info) => {
            tx1.send(true).unwrap();
            let client_credential = SpotifyClientCredentials::default()
                .token_info(token_info)
                .build();
            Spotify::default()
                .client_credentials_manager(client_credential)
                .build()
        },
        None => panic!("Auth / authz error.")
    };

    let item = spotify.current_playing(None).unwrap().unwrap().item.unwrap();
    println!("{:?}", item);
}
