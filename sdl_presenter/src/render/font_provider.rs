use font_loader::system_fonts;
use sdl2::ttf::Sdl2TtfContext;

pub struct FontProvider {
    bytes: Vec<u8>,
    ctx: Sdl2TtfContext,
}

impl FontProvider {
    pub fn new() -> Self {
        let mut property = system_fonts::FontPropertyBuilder::new().monospace().build();
        let sys_fonts = system_fonts::query_specific(&mut property);
        let font_bytes = system_fonts::get(
            &system_fonts::FontPropertyBuilder::new()
                .family(sys_fonts.first().unwrap())
                .build(),
        )
        .unwrap();

        Self {
            bytes: font_bytes.0,
            ctx: sdl2::ttf::init().unwrap(),
        }
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
