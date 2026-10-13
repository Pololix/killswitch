use dbus::arg::prop_cast;
use dbus::blocking::Connection;
use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
use std::{thread, time::Duration};

/// Returns Ok(None) if Spotify isn't running.
fn read_state(conn: &Connection) -> Result<Option<State>, dbus::Error> {
    let proxy = conn.with_proxy(SPOTIFY, PATH, DBUS_TIMEOUT);

    let props = match proxy.get_all(PLAYER) {
        Ok(p) => p,
        Err(e) => {
            // Spotify not on the bus (not running / just quit)
            return match e.name() {
                Some("org.freedesktop.DBus.Error.ServiceUnknown")
                | Some("org.freedesktop.DBus.Error.NameHasNoOwner") => Ok(None),
                _ => Err(e),
            };
        }
    };

    let missing = |name: &str| dbus::Error::new_failed(&format!("missing property {name}"));

    let status =
        prop_cast::<String>(&props, "PlaybackStatus").ok_or_else(|| missing("PlaybackStatus"))?;
    let can_pause = *prop_cast::<bool>(&props, "CanPause").ok_or_else(|| missing("CanPause"))?;
    let can_next = *prop_cast::<bool>(&props, "CanGoNext").ok_or_else(|| missing("CanGoNext"))?;
    let can_prev =
        *prop_cast::<bool>(&props, "CanGoPrevious").ok_or_else(|| missing("CanGoPrevious"))?;

    Ok(Some(State {
        is_playing: status == "Playing", // "Playing" | "Paused" | "Stopped"
        can_pause,
        can_next,
        can_prev,
    }))
}

fn main() -> Result<(), dbus::Error> {
    let conn = Connection::new_session()?;
    let mut last: Option<State> = None;

    loop {
        match read_state(&conn) {
            Ok(state) => {
                // Only react when something actually changed
                if state != last {
                    match &state {
                        Some(s) => println!("state changed: {s:?}"),
                        None => println!("spotify not running"),
                    }
                    last = state;
                }
            }
            Err(e) => eprintln!("poll error: {e}"),
        }

        thread::sleep(POLL_INTERVAL);
    }
}
