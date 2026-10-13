use dbus::{
    arg::prop_cast,
    blocking::{Connection, stdintf::org_freedesktop_dbus::Properties},
};
use std::process::{Command, Stdio};
use std::{thread, time::Duration};

const POLL: Duration = Duration::from_secs(2);
const TIMEOUT: Duration = Duration::from_secs(5);

const SPOTIFY: &str = "org.mpris.MediaPlayer2.spotify";
const PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER: &str = "org.mpris.MediaPlayer2.Player";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    is_playing: bool,

    can_pause: bool,
    can_prev: bool,
}

fn main() -> Result<(), dbus::Error> {
    let connection = Connection::new_session()?;

    println!("Starting loop");
    let mut count = 0;

    loop {
        println!("Iteration {0}", count);
        count += 1;

        match get_state(&connection) {
            Ok(state) => handle_state(state),
            Err(error) => println!("Couldnt fetch state: {error}"),
        }

        thread::sleep(POLL);
    }
}

fn get_state(connection: &Connection) -> Result<State, dbus::Error> {
    let proxy = connection.with_proxy(SPOTIFY, PATH, TIMEOUT);
    let properties = match proxy.get_all(PLAYER) {
        Ok(properties) => properties,
        Err(error) => return Err(error),
    };

    let is_playing = match prop_cast::<String>(&properties, "PlaybackStatus") {
        Some(status) => status == "Playing",
        None => false,
    };

    let read_prop = |prop: &str| {
        prop_cast::<bool>(&properties, prop)
            .copied()
            .unwrap_or(false)
    };

    Ok(State {
        is_playing,

        can_pause: read_prop("CanPause"),
        can_prev: read_prop("CanGoPrevious"),
    })
}

fn handle_state(state: State) {
    if !state.is_playing {
        println!("Not playing, not touching");
        return;
    }

    if !state.can_prev || !state.can_pause {
        println!("Rebooting");
        run("pkill spotify");
        run("spotify-launcher");
        println!("Rebooted");

        thread::sleep(Duration::from_secs(1));

        run("playerctl play");
        println!("Pressed played");
    }
}

fn run(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_or(false, |s| s.success())
}
