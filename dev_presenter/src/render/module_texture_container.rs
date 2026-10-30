use crate::render::spritesheet::Spritesheet;
use dudes_in_space_api::module::ModuleTypeId;
use sdl2::image::ImageRWops;
use std::collections::BTreeMap;

pub(crate) enum ModuleTexture<'surface> {
    Texture(sdl2::surface::Surface<'surface>),
    Spritesheet(Spritesheet<'surface>),
}

pub(crate) struct ModuleTextureContainerBuilder {
    data: BTreeMap<ModuleTypeId, ModuleTexture<'static>>,
}

impl ModuleTextureContainerBuilder {
    pub(crate) fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub(crate) fn with(mut self, type_id: ModuleTypeId, bytes: &[u8]) -> Self {
        let surface: sdl2::surface::Surface<'static> = sdl2::rwops::RWops::from_bytes(bytes)
            .unwrap()
            .load_png()
            .unwrap();

        match self
            .data
            .try_insert(type_id, ModuleTexture::Texture(surface))
        {
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
        let surface: sdl2::surface::Surface<'static> = sdl2::rwops::RWops::from_bytes(png_bytes)
            .unwrap()
            .load_png()
            .unwrap();

        match self.data.try_insert(
            type_id,
            ModuleTexture::Spritesheet(Spritesheet::new(surface, spritesheet)),
        ) {
            Ok(_) => {}
            Err(_) => todo!(),
        }

        self
    }

    pub(crate) fn build(self) -> ModuleTextureContainer {
        ModuleTextureContainer { data: self.data }
    }
}

pub(crate) struct ModuleTextureContainer {
    data: BTreeMap<ModuleTypeId, ModuleTexture<'static>>,
}

impl ModuleTextureContainer {
    pub(crate) fn get_ref<'a>(&'a self) -> ModuleTextureContainerRef<'a> {
        ModuleTextureContainerRef { data: &self.data }
    }

    pub(crate) fn get(&self, type_id: ModuleTypeId) -> Option<&ModuleTexture<'static>> {
        self.data.get(&type_id)
    }
}

#[derive(Clone)]
pub(crate) struct ModuleTextureContainerRef<'texture> {
    data: &'texture BTreeMap<ModuleTypeId, ModuleTexture<'texture>>,
}

impl<'texture> ModuleTextureContainerRef<'texture> {
    pub(crate) fn get(&self, type_id: ModuleTypeId) -> Option<&ModuleTexture<'texture>> {
        self.data.get(&type_id)
    }
}
