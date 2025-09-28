use crate::render::spritesheet::Spritesheet;
use dudes_in_space_api::module::ModuleTypeId;
use sdl2::image::LoadTexture;
use std::collections::BTreeMap;

pub(crate) enum ModuleTexture<'texture> {
    Texture(sdl2::render::Texture<'texture>),
    Spritesheet(Spritesheet<'texture>),
}

pub(crate) struct ModuleTextureContainerBuilder<'texture, T: 'texture> {
    texture_creator: &'texture sdl2::render::TextureCreator<T>,
    data: BTreeMap<ModuleTypeId, ModuleTexture<'texture>>,
}

impl<'texture, T: 'texture> ModuleTextureContainerBuilder<'texture, T> {
    pub(crate) fn new(texture_creator: &'texture sdl2::render::TextureCreator<T>) -> Self {
        Self {
            texture_creator,
            data: BTreeMap::new(),
        }
    }

    pub(crate) fn with(mut self, type_id: ModuleTypeId, bytes: &[u8]) -> Self {
        match self.data.try_insert(
            type_id,
            ModuleTexture::Texture(self.texture_creator.load_texture_bytes(bytes).unwrap()),
        ) {
            Ok(_) => {}
            Err(_) => todo!(),
        }

        self
    }

    pub(crate) fn with_aseprite_spritesheet(
        mut self,
        type_id: ModuleTypeId,
        png_bytes: &[u8],
        json_bytes: &[u8],
    ) -> Self {
        let spritesheet: aseprite::SpritesheetData = serde_json::from_slice(json_bytes).unwrap();

        match self.data.try_insert(
            type_id,
            ModuleTexture::Spritesheet(Spritesheet::new(
                self.texture_creator.load_texture_bytes(png_bytes).unwrap(),
                spritesheet,
            )),
        ) {
            Ok(_) => {}
            Err(_) => todo!(),
        }

        self
    }

    pub(crate) fn build(self) -> ModuleTextureContainer<'texture> {
        ModuleTextureContainer { data: self.data }
    }
}

pub(crate) struct ModuleTextureContainer<'texture> {
    data: BTreeMap<ModuleTypeId, ModuleTexture<'texture>>,
}

impl<'texture> ModuleTextureContainer<'texture> {
    pub(crate) fn get_ref<'a>(&'a self) -> ModuleTextureContainerRef<'a> {
        ModuleTextureContainerRef { data: &self.data }
    }
}

#[derive(Clone)]
pub(crate) struct ModuleTextureContainerRef<'texture> {
    data: &'texture BTreeMap<ModuleTypeId, ModuleTexture<'texture>>,
}

impl<'texture> ModuleTextureContainerRef<'texture> {
    pub fn get(&self, type_id: ModuleTypeId) -> Option<&ModuleTexture<'texture>> {
        self.data.get(&type_id)
    }
}
