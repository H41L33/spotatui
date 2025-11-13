use clap::ArgMatches;
use rspotify::{
  model::enums::RepeatState,
  model::idtypes::Id,
  model::{
    album::SimplifiedAlbum, artist::FullArtist, artist::SimplifiedArtist,
    playlist::SimplifiedPlaylist, show::FullEpisode, show::SimplifiedShow, track::FullTrack,
  },
};
use std::time::Duration;

use crate::user_config::UserConfig;

// Helper function to extract URI from typed IDs or external URLs
fn get_uri_or_fallback<T: Id>(
  id: &Option<T>,
  external_urls: &std::collections::HashMap<String, String>,
) -> String {
  if let Some(id) = id {
    id.uri()
  } else {
    external_urls
      .get("spotify")
      .cloned()
      .unwrap_or_else(|| "N/A".to_string())
  }
}

// Possible types to list or search
#[derive(Debug)]
pub enum Type {
  Playlist,
  Track,
  Artist,
  Album,
  Show,
  Device,
  Liked,
}

impl Type {
  pub fn play_from_matches(m: &ArgMatches<'_>) -> Self {
    if m.is_present("playlist") {
      Self::Playlist
    } else if m.is_present("track") {
      Self::Track
    } else if m.is_present("artist") {
      Self::Artist
    } else if m.is_present("album") {
      Self::Album
    } else if m.is_present("show") {
      Self::Show
    }
    // Enforced by clap
    else {
      unreachable!()
    }
  }

  pub fn search_from_matches(m: &ArgMatches<'_>) -> Self {
    if m.is_present("playlists") {
      Self::Playlist
    } else if m.is_present("tracks") {
      Self::Track
    } else if m.is_present("artists") {
      Self::Artist
    } else if m.is_present("albums") {
      Self::Album
    } else if m.is_present("shows") {
      Self::Show
    }
    // Enforced by clap
    else {
      unreachable!()
    }
  }

  pub fn list_from_matches(m: &ArgMatches<'_>) -> Self {
    if m.is_present("playlists") {
      Self::Playlist
    } else if m.is_present("devices") {
      Self::Device
    } else if m.is_present("liked") {
      Self::Liked
    }
    // Enforced by clap
    else {
      unreachable!()
    }
  }
}

//
// Possible flags to set
//

pub enum Flag {
  // Does not get toggled
  // * User chooses like -> Flag::Like(true)
  // * User chooses dislike -> Flag::Like(false)
  Like(bool),
  Shuffle,
  Repeat,
}

impl Flag {
  pub fn from_matches(m: &ArgMatches<'_>) -> Vec<Self> {
    // Multiple flags are possible
    let mut flags = Vec::new();

    // Only one of these two
    if m.is_present("like") {
      flags.push(Self::Like(true));
    } else if m.is_present("dislike") {
      flags.push(Self::Like(false));
    }

    if m.is_present("shuffle") {
      flags.push(Self::Shuffle);
    }
    if m.is_present("repeat") {
      flags.push(Self::Repeat);
    }
    flags
  }
}

// Possible directions to jump to
pub enum JumpDirection {
  Next,
  Previous,
}

impl JumpDirection {
  pub fn from_matches(m: &ArgMatches<'_>) -> (Self, u64) {
    if m.is_present("next") {
      (Self::Next, m.occurrences_of("next"))
    } else if m.is_present("previous") {
      (Self::Previous, m.occurrences_of("previous"))
    // Enforced by clap
    } else {
      unreachable!()
    }
  }
}

// For fomatting (-f / --format flag)

// Types to create a Format enum from
// Boxing was proposed by cargo clippy
// to reduce the size of this enum
pub enum FormatType {
  Album(Box<SimplifiedAlbum>),
  Artist(Box<FullArtist>),
  Playlist(Box<SimplifiedPlaylist>),
  Track(Box<FullTrack>),
  Episode(Box<FullEpisode>),
  Show(Box<SimplifiedShow>),
}

// Types that can be formatted
#[derive(Clone)]
pub enum Format {
  Album(String),
  Artist(String),
  Playlist(String),
  Track(String),
  Show(String),
  Uri(String),
  Device(String),
  Volume(u32),
  // Current position, duration
  Position((u32, u32)),
  // This is a bit long, should it be splitted up?
  Flags((RepeatState, bool, bool)),
  Playing(bool),
}

pub fn join_artists(a: Vec<SimplifiedArtist>) -> String {
  a.iter()
    .map(|l| l.name.clone())
    .collect::<Vec<String>>()
    .join(", ")
}

impl Format {
  // Extract important information from types
  pub fn from_type(t: FormatType) -> Vec<Self> {
    match t {
      FormatType::Album(a) => {
        let joined_artists = join_artists(a.artists.clone());
        let uri = get_uri_or_fallback(&a.id, &a.external_urls);
        vec![
          Self::Album(a.name),
          Self::Artist(joined_artists),
          Self::Uri(uri),
        ]
      }
      FormatType::Artist(a) => {
        let uri = a.id.uri();
        vec![Self::Artist(a.name), Self::Uri(uri)]
      }
      FormatType::Playlist(p) => {
        let uri = p.id.uri();
        vec![Self::Playlist(p.name), Self::Uri(uri)]
      }
      FormatType::Track(t) => {
        let joined_artists = join_artists(t.artists.clone());
        let uri = get_uri_or_fallback(&t.id, &t.external_urls);
        vec![
          Self::Album(t.album.name),
          Self::Artist(joined_artists),
          Self::Track(t.name),
          Self::Uri(uri),
        ]
      }
      FormatType::Show(r) => {
        let uri = r.id.uri();
        vec![
          Self::Artist(r.publisher),
          Self::Show(r.name),
          Self::Uri(uri),
        ]
      }
      FormatType::Episode(e) => {
        let uri = e.id.uri();
        vec![
          Self::Show(e.show.name),
          Self::Artist(e.show.publisher),
          Self::Track(e.name),
          Self::Uri(uri),
        ]
      }
    }
  }

  // Is there a better way?
  pub fn inner(&self, conf: UserConfig) -> String {
    match self {
      Self::Album(s) => s.clone(),
      Self::Artist(s) => s.clone(),
      Self::Playlist(s) => s.clone(),
      Self::Track(s) => s.clone(),
      Self::Show(s) => s.clone(),
      Self::Uri(s) => s.clone(),
      Self::Device(s) => s.clone(),
      // Because this match statements
      // needs to return a &String, I have to do it this way
      Self::Volume(s) => s.to_string(),
      Self::Position((curr, duration)) => {
        let current_progress_ms = *curr as u128;
        let duration = Duration::from_millis(*duration as u64);
        crate::ui::util::display_track_progress(current_progress_ms, duration)
      }
      Self::Flags((r, s, l)) => {
        let like = if *l {
          conf.behavior.liked_icon
        } else {
          String::new()
        };
        let shuffle = if *s {
          conf.behavior.shuffle_icon
        } else {
          String::new()
        };
        let repeat = match r {
          RepeatState::Off => String::new(),
          RepeatState::Track => conf.behavior.repeat_track_icon,
          RepeatState::Context => conf.behavior.repeat_context_icon,
        };

        // Add them together (only those that aren't empty)
        [shuffle, repeat, like]
          .iter()
          .filter(|a| !a.is_empty())
          // Convert &String to String to join them
          .map(|s| s.to_string())
          .collect::<Vec<String>>()
          .join(" ")
      }
      Self::Playing(s) => {
        if *s {
          conf.behavior.playing_icon
        } else {
          conf.behavior.paused_icon
        }
      }
    }
  }

  pub fn get_placeholder(&self) -> &str {
    match self {
      Self::Album(_) => "%b",
      Self::Artist(_) => "%a",
      Self::Playlist(_) => "%p",
      Self::Track(_) => "%t",
      Self::Show(_) => "%h",
      Self::Uri(_) => "%u",
      Self::Device(_) => "%d",
      Self::Volume(_) => "%v",
      Self::Position(_) => "%r",
      Self::Flags(_) => "%f",
      Self::Playing(_) => "%s",
    }
  }
}
