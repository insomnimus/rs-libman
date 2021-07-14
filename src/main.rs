use libman::{control::Controller, handler};
use rspotify::blocking::{
    client::Spotify,
    oauth2::{SpotifyClientCredentials, SpotifyOAuth},
    util::get_token,
};
use std::{env, process};

const SCOPES: &str = "user-read-recently-played user-read-playback-state user-top-read playlist-modify-public user-modify-playback-state playlist-modify-private user-follow-modify user-read-currently-playing user-follow-read user-library-modify user-read-playback-position playlist-read-private user-library-read playlist-read-collaborative";

fn main() {
    let client_id = env::var("LIBMAN_ID").unwrap_or_else(|_| {
        println!("you must set the LIBMAN_ID env variable to a spotify client id");
        process::exit(2);
    });
    let client_secret = env::var("LIBMAN_SECRET").unwrap_or_else(|_| {
        println!("you must set the LIBMAN_SECRET env variable to a spotify api client secret");
        process::exit(2);
    });
    let redirect_uri = env::var("LIBMAN_REDIRECT_URI").unwrap_or_else(|_| {
        println!("you must set the LIBMAN_REDIRECT_URI env variable to a configured redirect uri");
        process::exit(2);
    });

    let mut oauth = SpotifyOAuth::default()
        .scope(SCOPES)
        .client_id(&client_id)
        .client_secret(&client_secret)
        .redirect_uri(&redirect_uri)
        .build();

    let client = match get_token(&mut oauth) {
        Some(token_info) => {
            let client_credential = SpotifyClientCredentials::default()
                .client_id(&client_id)
                .client_secret(&client_secret)
                .token_info(token_info)
                .build();
            Spotify::default()
                .client_credentials_manager(client_credential)
                .build()
        }
        None => {
            println!("auth failed");
            process::exit(2);
        }
    };

    let user = client.current_user().unwrap_or_else(|e| {
        println!("error getting current user: {}", e);
        process::exit(1);
    });

    println!(
        "welcome {}",
        user.display_name
            .as_ref()
            .map(|s| &s[..])
            .unwrap_or_default()
    );

    let mut controller = Controller::new(
        client,
        user.id,
        handler::default_handlers(),
        handler::default_track_handlers(),
        "@libman>".to_string(),
    );

    controller.start();
}
