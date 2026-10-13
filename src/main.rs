use dbus::{
    arg::prop_cast,
    blocking::{Connection, stdintf::org_freedesktop_dbus::Properties},
};
use std::process::{Command, Stdio};
use std::{thread, time::Duration};

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
    spawn("spotify-launcher");

    let connection = Connection::new_session()?;

    loop {
        match get_state(&connection) {
            Ok(state) => handle_state(state),
            Err(error) => println!("Couldnt fetch state: {error}"),
        }

        thread::sleep(Duration::from_secs(3));
    }
}

fn get_state(connection: &Connection) -> Result<State, dbus::Error> {
    let proxy = connection.with_proxy(SPOTIFY, PATH, Duration::from_millis(500));
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
        return;
    }

    if !state.can_prev || !state.can_pause {
        run("pkill -x spotify");
        thread::sleep(Duration::from_millis(500));

        spawn("spotify-launcher");

        loop {
            if run("playerctl -p spotify status") {
                break;
            }

            thread::sleep(Duration::from_millis(500));
        }

        run("playerctl -p spotify play");
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

fn spawn(cmd: &str) {
    if let Ok(mut child) = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        thread::spawn(move || {
            let _ = child.wait();
        });
    }
}
