mod editor;
mod io;
mod model;
mod dashboard;
mod entity_types;
use serde_json::Value;

fn main() {
    let mut args = std::env::args().skip(1);
    if let Some(arg) = args.next() {
        if arg == "--sync-entity-types" {
            println!("Update Entity Types: starting...");
            match io::sync_entity_types_with_sprites() {
                Ok(report) => {
                    println!("Update Entity Types: finished. created={}, updated={}, deleted={}", report.created, report.updated, report.deleted);
                    return;
                }
                Err(e) => {
                    eprintln!("Update Entity Types: failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        if arg == "--print-entity" {
            if let Some(entity_name) = args.next() {
                let sprites_dir = io::assets_dir().join("sprites");
                let entity_dir = sprites_dir.join(&entity_name);
                if !entity_dir.is_dir() {
                    eprintln!("entity sprites directory not found: {}", entity_dir.display());
                    std::process::exit(2);
                }
                match io::collect_sprite_frames(&entity_name, &entity_dir) {
                                Ok(_grouped) => {
                                let root = Value::Object(serde_json::Map::new());
                                match io::build_entity_type_json(&entity_name, &entity_dir, root) {
                            Ok(val) => {
                                println!("{}", serde_json::to_string_pretty(&val).unwrap());
                                return;
                            }
                            Err(e) => {
                                eprintln!("build failed: {}", e);
                                std::process::exit(3);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("collect frames failed: {}", e);
                        std::process::exit(4);
                    }
                }
            }
        }
    }

    dashboard::run();
}

