//! Registers SpaceMono as the global default font for the whole game.
//!
//! ## How it works
//! `load_internal_binary_asset!` embeds the TTF bytes at compile time and
//! inserts them into `Assets<Font>` at a specific handle UUID.  We use
//! `TextFont::default().font` as the handle for SpaceMono Regular – the same
//! UUID that Bevy's built-in FiraMono occupies – so every existing
//! `TextFont { font_size: ..., ..default() }` automatically renders in
//! SpaceMono with **zero changes elsewhere**.
//!
//! ## Variant selection
//! Add one or both marker components to a text entity and the correct variant
//! is applied automatically on the same frame (change-detection, zero cost on
//! frames with no new text):
//!
//! | components               | variant             |
//! |--------------------------|---------------------|
//! | *(none)*                 | SpaceMono Regular   |
//! | [`BoldText`]             | SpaceMono Bold      |
//! | [`ItalicText`]           | SpaceMono Italic    |
//! | [`BoldText`]+[`ItalicText`] | SpaceMono BoldItalic |

use bevy::asset::{load_internal_binary_asset, uuid_handle};
use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Stable weak handles for the three non-Regular variants.
// These numbers are arbitrary but MUST NOT be changed after shipping.
// ---------------------------------------------------------------------------

pub(crate) const FONT_BOLD_HANDLE: Handle<Font> =
    uuid_handle!("50534d5f-424f-4c44-0000-000000000001");

pub(crate) const FONT_ITALIC_HANDLE: Handle<Font> =
    uuid_handle!("50534d5f-4954-414c-0000-000000000001");

pub(crate) const FONT_BOLD_ITALIC_HANDLE: Handle<Font> =
    uuid_handle!("50534d5f-4249-5441-0000-000000000001");

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Apply SpaceMono **Bold** to a text entity (or **BoldItalic** if
/// [`ItalicText`] is also present).
#[derive(Component)]
pub(crate) struct BoldText;

/// Apply SpaceMono *Italic* to a text entity (or **BoldItalic** if
/// [`BoldText`] is also present).
#[derive(Component)]
pub(crate) struct ItalicText;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers all four SpaceMono variants as compile-time binary assets and
/// wires up the automatic variant-selection system.
///
/// **Must be added *after* `DefaultPlugins`** so that `Assets<Font>` already
/// exists when `build()` runs.
pub struct FontsPlugin;

impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        // Override Bevy's built-in default font (FiraMono) with SpaceMono
        // Regular.  Every TextFont that uses ..default() picks this up.
        load_internal_binary_asset!(
            app,
            TextFont::default().font,
            "../assets/fonts/spacemono/SpaceMono-Regular.ttf",
            |bytes: &[u8], _path: String| {
                Font::try_from_bytes(bytes.to_vec()).unwrap()
            }
        );

        load_internal_binary_asset!(
            app,
            FONT_BOLD_HANDLE,
            "../assets/fonts/spacemono/SpaceMono-Bold.ttf",
            |bytes: &[u8], _path: String| {
                Font::try_from_bytes(bytes.to_vec()).unwrap()
            }
        );

        load_internal_binary_asset!(
            app,
            FONT_ITALIC_HANDLE,
            "../assets/fonts/spacemono/SpaceMono-Italic.ttf",
            |bytes: &[u8], _path: String| {
                Font::try_from_bytes(bytes.to_vec()).unwrap()
            }
        );

        load_internal_binary_asset!(
            app,
            FONT_BOLD_ITALIC_HANDLE,
            "../assets/fonts/spacemono/SpaceMono-BoldItalic.ttf",
            |bytes: &[u8], _path: String| {
                Font::try_from_bytes(bytes.to_vec()).unwrap()
            }
        );

        app.add_systems(Update, apply_font_variants);
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Assigns the correct SpaceMono variant to any [`TextFont`] entity whose
/// [`BoldText`] or [`ItalicText`] marker was *just added*.
///
/// Uses Bevy change-detection (`Added<T>`) – cost is essentially zero on
/// frames where no new styled text is spawned.
fn apply_font_variants(
    mut query: Query<
        (&mut TextFont, Has<BoldText>, Has<ItalicText>),
        Or<(Added<BoldText>, Added<ItalicText>)>,
    >,
) {
    for (mut tf, is_bold, is_italic) in &mut query {
        tf.font = match (is_bold, is_italic) {
            (true, true) => FONT_BOLD_ITALIC_HANDLE,
            (true, false) => FONT_BOLD_HANDLE,
            (false, true) => FONT_ITALIC_HANDLE,
            (false, false) => TextFont::default().font, // back to Regular
        };
    }
}


