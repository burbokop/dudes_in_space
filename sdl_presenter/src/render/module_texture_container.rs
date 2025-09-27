use dudes_in_space_api::module::ModuleTypeId;
use sdl2::image::LoadTexture;
use std::collections::BTreeMap;

pub(crate) struct ModuleTextureContainerBuilder<'texture, T: 'texture> {
    texture_creator: &'texture sdl2::render::TextureCreator<T>,
    data: BTreeMap<ModuleTypeId, sdl2::render::Texture<'texture>>,
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
            self.texture_creator.load_texture_bytes(bytes).unwrap(),
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
    data: BTreeMap<ModuleTypeId, sdl2::render::Texture<'texture>>,
}

impl<'texture> ModuleTextureContainer<'texture> {
    pub(crate) fn get_ref<'a>(&'a self) -> ModuleTextureContainerRef<'a> {
        ModuleTextureContainerRef { data: &self.data }
    }
}

#[derive(Clone)]
pub(crate) struct ModuleTextureContainerRef<'texture> {
    data: &'texture BTreeMap<ModuleTypeId, sdl2::render::Texture<'texture>>,
}

impl<'texture> ModuleTextureContainerRef<'texture> {
    pub fn get(&self, type_id: ModuleTypeId) -> Option<&sdl2::render::Texture<'texture>> {
        self.data.get(&type_id)
    }
}
