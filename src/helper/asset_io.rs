use bevy::asset::io::AssetSourceId;
use bevy::prelude::*;
use std::path::Path;

use crate::helper::active_character::ActiveCharacter;

/// Resolve an asset path for the active character.
///
/// Resolution order:
/// 1) Use the original path if it exists.
/// 2) Otherwise try `{stem}.{character}.{ext}` (e.g. `foo.betty.png`).
/// 3) If neither exists, return the original path so Bevy reports it as missing.
pub(crate) fn resolve_character_asset_path(
    asset_server: &AssetServer,
    asset_path: &str,
    active_character: ActiveCharacter,
) -> Result<String, std::io::Error> {
    if asset_exists(asset_server, asset_path)? {
        return Ok(asset_path.to_string());
    }

    let candidate = append_character_suffix_before_extension(asset_path, active_character);
    if asset_exists(asset_server, &candidate)? {
        return Ok(candidate);
    }

    Ok(asset_path.to_string())
}

/// Central helper for loading any Bevy asset with character-aware fallback.
pub(crate) fn load_character_asset<A: Asset>(
    asset_server: &AssetServer,
    asset_path: &str,
    active_character: ActiveCharacter,
) -> Handle<A> {
    let resolved = match resolve_character_asset_path(asset_server, asset_path, active_character) {
        Ok(path) => path,
        Err(error) => {
            tracing::warn!(
                path = %asset_path,
                error = %error,
                "asset resolver failed; falling back to original path"
            );
            asset_path.to_string()
        }
    };

    asset_server.load::<A>(resolved)
}

fn asset_exists(asset_server: &AssetServer, asset_path: &str) -> Result<bool, std::io::Error> {
    let source = asset_server
        .get_source(AssetSourceId::Default)
        .map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Asset source error: {error}"),
            )
        })?;

    pollster::block_on(async {
        let result = source.reader().read(asset_path.as_ref()).await;
        Ok::<bool, std::io::Error>(result.is_ok())
    })
}

fn append_character_suffix_before_extension(
    asset_path: &str,
    active_character: ActiveCharacter,
) -> String {
    let suffix = active_character.asset_suffix();
    let path = Path::new(asset_path);

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return asset_path.to_string();
    };

    let candidate_file_name = if let Some((stem, ext)) = file_name.rsplit_once('.') {
        if stem.ends_with(".bob")
            || stem.ends_with(".betty")
            || stem.ends_with("_bob")
            || stem.ends_with("_betty")
        {
            file_name.to_string()
        } else {
            format!("{stem}.{suffix}.{ext}")
        }
    } else {
        format!("{file_name}.{suffix}")
    };

    match path.parent().and_then(|parent| parent.to_str()) {
        Some(parent) if !parent.is_empty() => format!("{parent}/{candidate_file_name}"),
        _ => candidate_file_name,
    }
}

/// Read a text asset from the AssetServer via its underlying source and return UTF-8 text.
/// Returns a std::io::Error describing failures (not found, invalid data, etc.).
pub fn read_asset_text(
    asset_server: &AssetServer,
    asset_path: &str,
    active_character: ActiveCharacter,
) -> Result<String, std::io::Error> {
    let resolved_asset_path = resolve_character_asset_path(asset_server, asset_path, active_character)?;

    let source = asset_server
        .get_source(AssetSourceId::Default)
        .map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Asset source error: {error}"),
            )
        })?;

    let mut bytes = Vec::new();
    pollster::block_on(async {
        let mut reader = source
            .reader()
            .read(resolved_asset_path.as_ref())
            .await
            .map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Could not read asset '{resolved_asset_path}': {error}"),
                )
            })?;

        reader.read_to_end(&mut bytes).await.map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Could not read asset bytes for '{resolved_asset_path}': {error}"
                ),
            )
        })?;

        Ok::<(), std::io::Error>(())
    })?;

    String::from_utf8(bytes).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Asset '{resolved_asset_path}' is not valid UTF-8: {error}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::append_character_suffix_before_extension;
    use crate::helper::active_character::ActiveCharacter;

    #[test]
    fn inserts_suffix_before_extension() {
        let path = "sprites/player/player-default.png";
        assert_eq!(
            append_character_suffix_before_extension(path, ActiveCharacter::Betty),
            "sprites/player/player-default.betty.png"
        );
    }

    #[test]
    fn keeps_existing_character_suffix_unchanged() {
        let path = "music/start.ogg";
        assert_eq!(
            append_character_suffix_before_extension(path, ActiveCharacter::Bob),
            "music/start.ogg"
        );
    }

    #[test]
    fn supports_paths_without_extension() {
        let path = "story/world_start";
        assert_eq!(
            append_character_suffix_before_extension(path, ActiveCharacter::Bob),
            "story/world_start.bob"
        );
    }
}

