use rust_embed::RustEmbed;

use super::mime::{guess_mime_type, MimeType};

#[derive(RustEmbed)]
#[folder = "src/static"]
struct StaticAssets;

pub struct StaticHandler;

impl StaticHandler {
    pub fn serve(path: &str) -> Option<(Vec<u8>, MimeType)> {
        StaticAssets::get(path).map(|f| {
            let mime = guess_mime_type(path);
            (f.data.into(), mime)
        })
    }
}
