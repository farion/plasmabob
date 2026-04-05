use bevy::asset::io::AssetSourceId;
use bevy::prelude::*;

/// Read a text asset from the AssetServer via its underlying source and return UTF-8 text.
/// Returns a std::io::Error describing failures (not found, invalid data, etc.).
pub fn read_asset_text(asset_server: &AssetServer, asset_path: &str) -> Result<String, std::io::Error> {
    let source = asset_server.get_source(AssetSourceId::Default).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Asset source error: {error}"),
        )
    })?;

    let mut bytes = Vec::new();
    pollster::block_on(async {
        let mut reader = source
            .reader()
            .read(asset_path.as_ref())
            .await
            .map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Could not read asset '{asset_path}': {error}"),
                )
            })?;

        reader.read_to_end(&mut bytes).await.map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Could not read asset bytes for '{asset_path}': {error}"),
            )
        })?;

        Ok::<(), std::io::Error>(())
    })?;

    String::from_utf8(bytes).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Asset '{asset_path}' is not valid UTF-8: {error}"),
        )
    })
}


