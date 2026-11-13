pub(crate) struct Spritesheet<'surface> {
    pub(crate) surface: sdl2::surface::Surface<'surface>,
    pub(crate) data: aseprite::SpritesheetData,
}

impl<'surface> Spritesheet<'surface> {
    pub(super) fn new(
        surface: sdl2::surface::Surface<'surface>,
        data: aseprite::SpritesheetData,
    ) -> Self {
        Self { surface, data }
    }
}

pub(crate) struct SpritesheetRef<'surface> {
    texture: &'surface sdl2::surface::Surface<'surface>,
    data: &'surface aseprite::SpritesheetData,
}
