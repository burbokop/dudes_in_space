pub(crate) struct Spritesheet<'texture> {
    pub(crate) texture: sdl2::render::Texture<'texture>,
    pub(crate) data: aseprite::SpritesheetData,
}

impl<'texture> Spritesheet<'texture> {
    pub(super) fn new(
        texture: sdl2::render::Texture<'texture>,
        data: aseprite::SpritesheetData,
    ) -> Self {
        Self { texture, data }
    }
}

pub(crate) struct SpritesheetRef<'texture> {
    texture: &'texture sdl2::render::Texture<'texture>,
    data: &'texture aseprite::SpritesheetData,
}
