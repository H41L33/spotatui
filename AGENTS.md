# Spotify-TUI Modernization Project

## Project Overview

**spotify-tui** is a Spotify client for the terminal written in Rust. It provides a full-featured TUI (Terminal User Interface) for controlling Spotify playback, browsing libraries, searching music, and managing playlists.

### Current Status
- **Original Version**: 0.25.0 (Last updated ~2 years ago)
- **Main Issue**: Outdated dependencies causing backend API failures
- **Goal**: Update dependencies and fix breaking API changes for personal use

### Tech Stack
- **Language**: Rust (Edition 2018)
- **UI Library**: Originally `tui-rs`, migrating to `ratatui`
- **Spotify API**: Originally `rspotify 0.10.0`, migrating to `0.12.x`
- **Terminal**: `crossterm`
- **Async Runtime**: `tokio`

### Key Features
- Browse and play Spotify playlists
- Search for tracks, albums, artists, and podcasts
- Control playback (play/pause, skip, seek, volume)
- View saved tracks, albums, and followed artists
- Audio analysis visualization
- Device selection
- CLI interface alongside TUI

---

## Migration Strategy

### Dependency Updates Required

| Dependency   | Original | Target           | Reason                                       |
| ------------ | -------- | ---------------- | -------------------------------------------- |
| `rspotify`   | 0.10.0   | 0.12.x           | Spotify API wrapper (major breaking changes) |
| `tui`        | 0.16.0   | N/A (deprecated) | Renamed to `ratatui`                         |
| `ratatui`    | N/A      | 0.26.x           | Successor to `tui-rs`                        |
| `tokio`      | 0.2      | 1.40.x           | Async runtime (major version upgrade)        |
| `crossterm`  | 0.20     | 0.27.x           | Terminal manipulation                        |
| `arboard`    | 1.2.0    | 3.4.x            | Clipboard support                            |
| `dirs`       | 3.0.2    | 5.0.x            | Directory utilities                          |
| `serde_yaml` | 0.8      | 0.9.x            | YAML parsing                                 |

### Breaking Changes in rspotify 0.10 → 0.12

#### Module Structure
- `rspotify::client::Spotify` → `rspotify::AuthCodeSpotify`
- `rspotify::oauth2` → `rspotify::OAuth` + `rspotify::Credentials`
- `rspotify::senum` → `rspotify::model::enums`

#### Type Renames
- `CurrentlyPlaybackContext` → `CurrentPlaybackContext`
- `PlayingItem` → `PlayableItem`
- `PlaylistTrack` → `PlaylistItem`
- `TokenInfo` → `Token`
- `SpotifyOAuth` → `OAuth`
- `SpotifyClientCredentials` → (integrated into client)

#### API Changes
- `for_position(u32)` → `Offset::Position(u32)`
- Track/Artist/Album IDs changed from `String` to typed IDs (`TrackId`, `ArtistId`, etc.)
- OAuth flow completely redesigned
- `util::get_token()`, `util::process_token()`, `util::request_token()` removed
- Many API methods have new signatures

#### Tokio Changes
- `tokio::time::delay_for()` → `tokio::time::sleep()`

---

## Changes Completed ✅

### Dependency Updates
- ✅ Updated `Cargo.toml` with modern dependency versions
- ✅ Changed `tui` to `ratatui` in dependencies
- ✅ Updated `rspotify` to 0.12 with required features (`cli`, `env-file`, `client-reqwest`)
- ✅ Updated `tokio` to 1.40
- ✅ Updated `crossterm` to 0.27
- ✅ Updated `arboard` to 3.4
- ✅ Updated `dirs` to 5.0
- ✅ Updated `serde_yaml` to 0.9

### Global Type Renames (All `.rs` files)
- ✅ Replaced all `use tui::` → `use ratatui::` imports
- ✅ Renamed `CurrentlyPlaybackContext` → `CurrentPlaybackContext`
- ✅ Renamed `PlayingItem` → `PlayableItem`
- ✅ Renamed `PlaylistTrack` → `PlaylistItem`
- ✅ Renamed `senum::` → `model::enums::`

### Import Updates
- ✅ Updated `src/network.rs` imports to use new rspotify structure
  - Added `prelude::*`, `AuthCodeSpotify`, `Token`, `OAuth`, `Credentials`, `Config`
  - Replaced leftover `for_position()` usages with `Offset::Position()`
  - Updated enum imports to use `model::enums::`

-### Core Functionality
- ✅ **src/main.rs**: Async bootstrap + OAuth flow fully modernized for rspotify 0.12.
  - ✅ Token cache now handled via `spotify.token.lock().await`, with graceful fallback when the cache file is missing.
  - ✅ `start_tokio` runs inside `tokio::spawn`, so queued `IoEvent`s can `.await` network calls without lifetime hacks.
  - ✅ Manual and web-based auth paths both work, and CLI/UI entry now happens even when no cached token exists.
- ✅ **src/network.rs**: Cleaned up authentication helpers.
  - ✅ Added `use anyhow::anyhow;` to fix macro usage.
  - ✅ `Network` now owns an `Arc<Mutex<App>>`, eliminating the old `'a` lifetime bound.
  - ✅ Corrected `refresh_authentication` to be a proper no-op.
-  - ✅ Removed unused `EpisodeId` and `SystemTime` imports.
-  - ⚠️ Stream API errors remain (artist_albums, playlists) - next priority after typed-ID dispatch fixes.
- ✅ **src/cli/cli_app.rs**: Stripped leftover `<'a>` lifetimes from `CliApp` struct and impl.
- ✅ **src/ui/**: ratatui migration completed for draw helpers
  - ✅ Converted all `Frame<B>` + `where B: Backend` to `Frame<'_>` in `src/ui/mod.rs`
  - ✅ Replaced `ResumePoint::resume_position_ms` with `resume_position`
  - ✅ Kept `std::time::Duration::as_millis()` where appropriate

---

## Work Remaining ❌

### High Priority - Core Functionality

#### Typed Spotify IDs (Network + App)
- ✅ `IoEvent` payloads now use typed IDs (`TrackId`, `AlbumId`, `ArtistId`, `ShowId`, `PlaylistId`, `PlayableId`) from `rspotify::model::idtypes`
- ✅ `network.rs` imports updated to use correct `idtypes` module instead of `id`
- ✅ `app.rs` imports updated to use `idtypes`
- ✅ `PlayContextId` replaces `ContextId` throughout  
- ✅ App continues to store IDs as `String`s in HashSets (simpler comparison logic)
- ❌ **Critical**: Handlers still dispatch String IDs; need conversion helpers in each handler file
  - Pattern: `if let Ok(id) = TrackId::from_id(&id_string) { app.dispatch(...) }`
  - Affects: track_table.rs, album_tracks.rs, recently_played.rs, playbar.rs, artist.rs, search_results.rs, input.rs, podcasts.rs, and many methods in app.rs

#### Playback & Queue helpers
- ❌ `start_playback`, queue additions still need typed ID conversions from handlers
- ❌ Recommendation helpers in app.rs need typed ID conversions

#### UI Ratatui Follow-ups
- ✅ All draw helpers now use `Frame<'_>`; Backend bounds removed.
- ✅ `resume_position_ms` replaced with `resume_position` in episode tables.
- ❌ Queue lookup and ID comparisons may still fail if they expect typed IDs instead of Strings; decide whether to store typed IDs or stringify at render time.

#### Tokio Updates
- ✅ `tokio::time::delay_for()` has been fully removed; remaining async waits use `tokio::time::sleep`.

### Medium Priority - Type Conversions

#### ID Type Conversions
- ❌ Fix `TrackId<'_>` to `String` conversions throughout codebase
- ❌ Fix `ArtistId<'_>` to `String` conversions
- ❌ Fix `AlbumId<'_>` to `String` conversions
- ❌ Update all code that stores/compares IDs as Strings
- ❌ Handle lifetime parameters in ID types

#### Model Field Access
- ❌ Update `PlaylistItem` field access (fields changed from `track` to different structure)
- ❌ Review and fix `PlayableItem` enum matching
- ❌ Update any code accessing changed model fields

### Low Priority - Additional Updates

#### CLI Module
- ❌ **src/cli/*.rs**: Review and test CLI functionality with new API
- ❌ Verify command-line interface still works correctly

#### Error Handling
- ❌ Update error handling for new rspotify error types
- ❌ Test error scenarios and ensure proper user feedback

#### Testing & Validation
- ❌ Test OAuth flow end-to-end
- ❌ Test playback controls
- ❌ Test library browsing
- ❌ Test search functionality
- ❌ Test device selection
- ❌ Test CLI commands
- ❌ Verify audio analysis feature
- ❌ Test with actual Spotify account

---

## Known Issues & Blockers

-### Compilation Errors (Current)
- **CLI compile issues**: `src/cli/handle.rs` still uses `Network<'_>`; must be `Network`. `src/cli/cli_app.rs` uses `unwrap_or_default()` on typed IDs and treats optional device fields as non-optional.
- **Typed-ID dispatch conversions incomplete**: Handlers (track_table.rs, album_tracks.rs, playbar.rs, artist.rs, search_results.rs, input.rs, podcasts.rs, recently_played.rs, playlist.rs) and app.rs helper methods still dispatch `IoEvent`s with `String` IDs instead of calling `.into_static()` on typed IDs. This causes "does not live long enough" errors.
- **Stream API incompatibility**: `rspotify 0.12` returns `Stream` types for paginated endpoints (artist_albums, playlists), but code tries to `.await` them directly. Need to collect/consume streams properly with `futures::StreamExt`.

### Design Decisions Needed
1. Do we store typed IDs (`TrackId`, `AlbumId`, …) inside `App`/UI state, or do we continue storing Strings and convert at the rspotify call sites?
2. How strict should we be about propagating typed IDs through every `IoEvent` vs. introducing helper conversion functions?
3. Are we keeping the `redirect_uri_web_server` helper even though it only needs the port (current signature still warns about unused `spotify`)?

---

## File-by-File Status

### Core Files
| File                  | Status          | Notes                                                                                           |
| --------------------- | --------------- | ----------------------------------------------------------------------------------------------- |
| `Cargo.toml`          | ✅ Updated       | Dependencies modernized                                                                         |
| `src/main.rs`         | ✅ Updated       | Async bootstrap, token cache handling, and UI/CLI dispatch now compile + run.                   |
| `src/network.rs`      | 🔶 Partial       | Owns `Arc<Mutex<App>>`, unused imports removed, but Stream APIs + typed-ID dispatch need fixes. |
| `src/redirect_uri.rs` | ✅ Updated       | Callback helper converted; unused `spotify` arg is the only warning.                            |
| `src/config.rs`       | ⚠️ Unknown       | May need updates for new OAuth                                                                  |
| `src/app.rs`          | ✅ Types updated | Model types renamed                                                                             |

### Handler Files
| File                | Status          | Notes                                                              |
| ------------------- | --------------- | ------------------------------------------------------------------ |
| `src/handlers/*.rs` | ✅ Types updated | Model types renamed globally; typed-ID dispatch conversions needed |

### UI Files
| File                       | Status      | Notes                                                                                 |
| -------------------------- | ----------- | ------------------------------------------------------------------------------------- |
| `src/ui/mod.rs`            | ✅ Complete  | All draw helpers migrated to `Frame<'_>`; replaced `resume_position_ms` appropriately |
| `src/ui/audio_analysis.rs` | ✅ Complete  | `Frame<B>` → `Frame<'_>`, Backend import removed.                                     |
| `src/ui/help.rs`           | ✅ Complete  | No generic signatures, no changes needed.                                             |

### CLI Files
| File                 | Status          | Notes                                     |
| -------------------- | --------------- | ----------------------------------------- |
| `src/cli/cli_app.rs` | ✅ Complete      | Lifetime `<'a>` stripped from struct/impl |
| `src/cli/*.rs`       | ✅ Types updated | Needs testing with new API                |

---

## Next Steps

### Immediate Actions (to get it compiling)
1. ❌ Fix CLI compile issues: remove lifetime from `Network<'_>` in `src/cli/handle.rs`; adjust `src/cli/cli_app.rs` to handle typed IDs via `.id().to_string()` and unwrap optional device fields safely; replace `.num_milliseconds()` with `as_millis()`.
2. ❌ Typed-ID dispatch conversions: Convert all `app.dispatch(IoEvent::...)` calls in `src/app.rs` + `src/handlers/*.rs` to build owned IDs via `.into_static()` (wrap tracks/episodes in `PlayableId` where required).
3. ❌ Stream API fixes: Replace `.await` on stream-returning rspotify calls (`playlist_items`, `artist_albums`, `current_user_playlists`, etc.) with `StreamExt` consumption in `src/network.rs`.

### Short Term (to get it working)
1. Re-test every `Network` API method once typed-ID dispatch & stream handling compile; ensure logging/error propagation is aligned with new APIs.
2. Retest CLI commands now that they share the async client/runtime.
3. Verify token refresh behavior in practice (currently relying on rspotify auto-refresh); remove redundant `RefreshAuthentication` IoEvent if unnecessary.
4. Update documentation/config templates (`client.yml`, README) with the new OAuth guidance.

### Long Term (for stability)
1. Comprehensive manual testing with a Spotify account (TUI + CLI flows, audio analysis, device switching).
2. Improve error handling and surface actionable messages to the TUI/CLI.
3. Consider migrating further to rspotify 0.13+ once 0.12 is stable.
4. Keep docs (`AGENTS.md`, `GEMINI.md`, `MIGRATION_NOTES.md`) updated as new fixes land.

---

## Resources

- [rspotify 0.12 Documentation](https://docs.rs/rspotify/0.12)
- [rspotify Migration Guide](https://github.com/ramsayleung/rspotify/blob/master/CHANGELOG.md)
- [ratatui Documentation](https://docs.rs/ratatui)
- [Tokio 1.x Migration Guide](https://tokio.rs/tokio/topics/bridging)

---

## Notes for Future Developers

- This is a **personal use** fork, not intended for upstream contribution
- Focus on getting it working rather than perfect code
- The original project is unmaintained, so we own the maintenance burden
- Consider switching to an actively maintained alternative if this becomes too difficult
- Main complexity is in the Spotify OAuth flow - once that works, the rest should follow
- Keep `AGENTS.md` and `GEMINI.md` in sync—if you mark work complete or add context in one, update the other in the same change

---

*Last Updated: 2025-11-11 by Codex (UI ratatui draw migration completed)*
*Status: Migration In Progress - Compilation Failing*
