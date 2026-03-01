use sdl2::ttf::Sdl2TtfContext;

pub struct FontProvider {
    bytes: Vec<u8>,
    ctx: Sdl2TtfContext,
}

fn choose_font(fonts: Vec<String>) -> String {
    // Fonts with symbol ₴
    let preferred_fonts = ["DejaVu Sans Mono", "FreeMono", "Hack", "Liberation Mono"];

    for font in preferred_fonts {
        if fonts.contains(&font.into()) {
            return font.into();
        }
    }

    fonts.first().unwrap().into()
}

impl FontProvider {
    pub fn new() -> Self {
        let mut collection = fontique::Collection::new(fontique::CollectionOptions::default());

        let ids = collection
            .generic_families(fontique::GenericFamily::Monospace)
            .collect::<Vec<_>>();

        for id in ids {
            if let Some(info) = collection.family(id) {
                if let Some(font) = info.default_font() {
                    if let Some(blob) = font.load(None) {
                        let bytes: &[u8] = blob.as_ref();

                        return Self {
                            bytes: bytes.into(),
                            ctx: sdl2::ttf::init().unwrap(),
                        };
                    }
                }
            }
        }

        panic!("No suitable font found")
    }

    pub fn font<'a>(&'a self, point_size: u16) -> sdl2::ttf::Font<'a, 'a> {
        self.ctx
            .load_font_from_rwops(
                sdl2::rwops::RWops::from_bytes(&self.bytes).unwrap(),
                point_size,
            )
            .unwrap()
    }
}
