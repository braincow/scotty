# Beam me up, Scotty!
Scotty is a simple test application written in Rust to experiment with different Rust crates and workflows for achieving authentication and authorization with Spotify. On succeeding this it simply in loop prints out what the authorized user is currently playing at.


## Configuration

To make this app run you need to create Spotify app id and secret. Also this app should have as its white listed return address http://localhost:65510 which is subsequently the address in which a small micro http server is spawned by Scotty to catch the users return from Spotify OAuth portal and the dispatched token.

You can as always place them in file called .env or simply set them up in your local environment. Example of the .env file below:
```
CLIENT_ID=yourclientidgoeshere
CLIENT_SECRET=superdupersecretstringhere
RUST_LOG=info
```

RUST_LOG is optional, but by fiddling with it you can get more verbose output.

## Running the app

As always its easiest to run the app with cargo:
```cargo run```
