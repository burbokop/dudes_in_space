use crate::logger::MemLogger;
use crate::render::render_models::person_render_model::PersonRenderModel;
use crate::render::renderer::Renderer;
use crate::render::{ColumnLayout, LayoutElement, LazyVesselRenderModel, RenderError, RowLayout};
use dudes_in_space_api::item::{ItemSafe, ItemStorage};
use dudes_in_space_api::module::{Module, ModuleStorage};
use dudes_in_space_api::person::Person;
use dudes_in_space_api::recipe::{AssemblyRecipe, InputItemRecipe, ItemRecipe, OutputItemRecipe};
use dudes_in_space_api::trade::{BuyCustomVesselOffer, BuyOffer, BuyVesselOffer, SellOffer};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::Rect;
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector};
use std::fmt::Alignment;

fn draw_top_info<T: sdl2::render::RenderTarget>(
    renderer: &mut Renderer<T>,
    module: &dyn Module,
    bounding_box: Rect<Float>,
) {
    let text_box_height = 0.5;

    let text_box: Rect<Float> = (
        *bounding_box.x(),
        bounding_box.y() - text_box_height,
        *bounding_box.w(),
        text_box_height,
    )
        .into();

    renderer.draw_confined_text(
        &format!(
            "{} ({}:{})",
            module.id(),
            module.package_id(),
            module.type_id()
        ),
        text_box,
        Alignment::Center,
        Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
    );
    draw_bounding_box(renderer, bounding_box);
}

fn draw_bottom_info<T: sdl2::render::RenderTarget>(
    renderer: &mut Renderer<T>,
    module: &dyn Module,
    bounding_box: Rect<Float>,
) {
    let text_box_height = 0.5;

    let text_box: Rect<Float> = (
        *bounding_box.x(),
        bounding_box.bottom(),
        *bounding_box.w(),
        text_box_height,
    )
        .into();

    renderer.draw_confined_text(
        &format!(
            "{:?} <=> {:?}",
            module.capabilities(),
            module.primary_capabilities()
        ),
        text_box,
        Alignment::Center,
        Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
    );
    draw_bounding_box(renderer, bounding_box);
}

fn draw_bounding_box<T: sdl2::render::RenderTarget>(
    renderer: &mut Renderer<T>,
    bounding_box: Rect<Float>,
) {
    renderer.draw_rect(
        bounding_box,
        Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
    );
}

struct DrawPerson<'a, 'b, 'c> {
    person: Option<(&'a Person, &'b MemLogger, &'c PersonRenderModel)>,
}

impl<'a, 'b, 'c> DrawPerson<'a, 'b, 'c> {
    pub fn new(
        person: &'a Person,
        logger: &'b MemLogger,
        person_render_model: &'c PersonRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            person: Some((person, logger, person_render_model)),
        })
    }
    pub fn new_empty() -> Box<Self> {
        Box::new(Self { person: None })
    }
}

impl<'a, 'b, 'c, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawPerson<'a, 'b, 'c> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        match self.person {
            None => renderer.draw_confined_text(
                "Vacant",
                bounding_box,
                Alignment::Center,
                Color {
                    r: 0.,
                    g: 0.,
                    b: 0.,
                    a: 1.,
                },
            ),
            Some((person, logger, render_model)) => render_model
                .render(renderer, person, logger, bounding_box)
                .unwrap(),
        }
        draw_bounding_box(renderer, bounding_box)
    }
}

struct DrawPersons<'a, 'b, 'c> {
    module: &'a dyn Module,
    logger: &'b MemLogger,
    person_render_model: &'c PersonRenderModel,
}

impl<'a, 'b, 'c> DrawPersons<'a, 'b, 'c> {
    pub fn new(
        module: &'a dyn Module,
        logger: &'b MemLogger,
        person_render_model: &'c PersonRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            module,
            logger,
            person_render_model,
        })
    }
}

impl<'a, 'b, 'c, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawPersons<'a, 'b, 'c> {
    fn visible(&self) -> bool {
        !(self.module.persons().is_empty() && self.module.free_person_slots_count() == 0)
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let persons = self.module.persons();
        let free_person_slots_count = self.module.free_person_slots_count();

        let layout = RowLayout::new(
            persons
                .iter()
                .map(|person| DrawPerson::new(person, self.logger, self.person_render_model))
                .chain((0..free_person_slots_count).map(|_| DrawPerson::new_empty()))
                .map(|x| x as Box<dyn LayoutElement<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawItemRecipe<'a> {
    recipe: &'a ItemRecipe,
}

impl<'a> DrawItemRecipe<'a> {
    pub fn new(recipe: &'a ItemRecipe) -> Box<Self> {
        Box::new(Self { recipe })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawItemRecipe<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{:?} -> {:?}", self.recipe.input, self.recipe.output,),
            bounding_box,
            Alignment::Center,
            Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        );
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawItemRecipes<'a> {
    recipes: &'a [ItemRecipe],
}

impl<'a> DrawItemRecipes<'a> {
    pub fn new(recipes: &'a [ItemRecipe]) -> Box<Self> {
        Box::new(Self { recipes })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawItemRecipes<'a> {
    fn visible(&self) -> bool {
        !self.recipes.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = ColumnLayout::new(
            self.recipes
                .iter()
                .map(DrawItemRecipe::new)
                .map(|x| x as Box<dyn LayoutElement<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawInputItemRecipe<'a> {
    recipe: &'a InputItemRecipe,
}

impl<'a> DrawInputItemRecipe<'a> {
    pub fn new(recipe: &'a InputItemRecipe) -> Box<Self> {
        Box::new(Self { recipe })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawInputItemRecipe<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{:?} -> |", self.recipe,),
            bounding_box,
            Alignment::Center,
            Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        );
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawInputItemRecipes<'a> {
    recipes: &'a [InputItemRecipe],
}

impl<'a> DrawInputItemRecipes<'a> {
    pub fn new(recipes: &'a [InputItemRecipe]) -> Box<Self> {
        Box::new(Self { recipes })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawInputItemRecipes<'a> {
    fn visible(&self) -> bool {
        !self.recipes.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = ColumnLayout::new(
            self.recipes
                .iter()
                .map(DrawInputItemRecipe::new)
                .map(|x| x as Box<dyn LayoutElement<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawOutputItemRecipe<'a> {
    recipe: &'a OutputItemRecipe,
}

impl<'a> DrawOutputItemRecipe<'a> {
    pub fn new(recipe: &'a OutputItemRecipe) -> Box<Self> {
        Box::new(Self { recipe })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawOutputItemRecipe<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("| -> {:?}", self.recipe,),
            bounding_box,
            Alignment::Center,
            Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        );
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawOutputItemRecipes<'a> {
    recipes: &'a [OutputItemRecipe],
}

impl<'a> DrawOutputItemRecipes<'a> {
    pub fn new(recipes: &'a [OutputItemRecipe]) -> Box<Self> {
        Box::new(Self { recipes })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawOutputItemRecipes<'a> {
    fn visible(&self) -> bool {
        !self.recipes.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = ColumnLayout::new(
            self.recipes
                .iter()
                .map(DrawOutputItemRecipe::new)
                .map(|x| x as Box<dyn LayoutElement<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawAssemblyRecipe<'a> {
    recipe: &'a AssemblyRecipe,
}

impl<'a> DrawAssemblyRecipe<'a> {
    pub fn new(recipe: &'a AssemblyRecipe) -> Box<Self> {
        Box::new(Self { recipe })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawAssemblyRecipe<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!(
                "{:?} -> {}",
                self.recipe.input(),
                self.recipe.output_description().type_id(),
            ),
            bounding_box,
            Alignment::Center,
            Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        );
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawAssemblyRecipes<'a> {
    recipes: &'a [AssemblyRecipe],
}

impl<'a> DrawAssemblyRecipes<'a> {
    pub fn new(recipes: &'a [AssemblyRecipe]) -> Box<Self> {
        Box::new(Self { recipes })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawAssemblyRecipes<'a> {
    fn visible(&self) -> bool {
        !self.recipes.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = ColumnLayout::new(
            self.recipes
                .iter()
                .map(DrawAssemblyRecipe::new)
                .map(|x| x as Box<dyn LayoutElement<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawRecipes<'a> {
    module: &'a dyn Module,
}

impl<'a> DrawRecipes<'a> {
    pub fn new(module: &'a dyn Module) -> Box<Self> {
        Box::new(Self { module })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawRecipes<'a> {
    fn visible(&self) -> bool {
        !self.module.item_recipes().is_empty()
            || !self.module.input_item_recipes().is_empty()
            || !self.module.output_item_recipes().is_empty()
            || !self.module.assembly_recipes().is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(vec![
            DrawAssemblyRecipes::new(self.module.assembly_recipes()),
            DrawItemRecipes::new(self.module.item_recipes()),
            DrawInputItemRecipes::new(self.module.input_item_recipes()),
            DrawOutputItemRecipes::new(self.module.output_item_recipes()),
        ]);

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawItemStorage<'a> {
    storage: &'a ItemStorage,
}

impl<'a> DrawItemStorage<'a> {
    pub fn new(storage: &'a ItemStorage) -> Box<Self> {
        Box::new(Self { storage })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawItemStorage<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{:?}", self.storage,),
            bounding_box,
            Alignment::Center,
            Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        );
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawItemStorages<'a> {
    storages: Vec<&'a ItemStorage>,
}

impl<'a> DrawItemStorages<'a> {
    pub fn new(storages: Vec<&'a ItemStorage>) -> Box<Self> {
        Box::new(Self { storages })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawItemStorages<'a> {
    fn visible(&self) -> bool {
        !self.storages.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(
            self.storages
                .iter()
                .cloned()
                .map(DrawItemStorage::new)
                .map(|x| x as Box<dyn LayoutElement<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawItemSafe<'a> {
    safe: &'a ItemSafe,
}

impl<'a> DrawItemSafe<'a> {
    pub fn new(safe: &'a ItemSafe) -> Box<Self> {
        Box::new(Self { safe })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawItemSafe<'a> {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{:?}", self.safe,),
            bounding_box,
            Alignment::Center,
            Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        );
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawItemSafes<'a> {
    safes: &'a [ItemSafe],
}

impl<'a> DrawItemSafes<'a> {
    pub fn new(safes: &'a [ItemSafe]) -> Box<Self> {
        Box::new(Self { safes })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawItemSafes<'a> {
    fn visible(&self) -> bool {
        !self.safes.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(
            self.safes
                .iter()
                .map(DrawItemSafe::new)
                .map(|x| x as Box<dyn LayoutElement<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawModuleStorage<'a> {
    storage: &'a ModuleStorage,
}

impl<'a> DrawModuleStorage<'a> {
    pub fn new(storage: &'a ModuleStorage) -> Box<Self> {
        Box::new(Self { storage })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawModuleStorage<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{:?}", self.storage,),
            bounding_box,
            Alignment::Center,
            Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        );
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawModuleStorages<'a> {
    storages: &'a [ModuleStorage],
}

impl<'a> DrawModuleStorages<'a> {
    pub fn new(storages: &'a [ModuleStorage]) -> Box<Self> {
        Box::new(Self { storages })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawModuleStorages<'a> {
    fn visible(&self) -> bool {
        !self.storages.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(
            self.storages
                .iter()
                .map(DrawModuleStorage::new)
                .map(|x| x as Box<dyn LayoutElement<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawStorages<'a> {
    module: &'a dyn Module,
}

impl<'a> DrawStorages<'a> {
    pub fn new(module: &'a dyn Module) -> Box<Self> {
        Box::new(Self { module })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawStorages<'a> {
    fn visible(&self) -> bool {
        !self.module.storages().is_empty()
            || !self.module.safes().is_empty()
            || !self.module.module_storages().is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(vec![
            DrawItemStorages::new(self.module.storages()),
            DrawItemSafes::new(self.module.safes()),
            DrawModuleStorages::new(self.module.module_storages()),
        ]);

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawDockingClamp<'a, 'b, 'c> {
    clamp: &'a DockingClamp,
    logger: &'b MemLogger,
    vessel_render_model: &'c LazyVesselRenderModel,
}

impl<'a, 'b, 'c> DrawDockingClamp<'a, 'b, 'c> {
    pub fn new(
        clamp: &'a DockingClamp,
        logger: &'b MemLogger,
        vessel_render_model: &'c LazyVesselRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            clamp,
            logger,
            vessel_render_model,
        })
    }
}

impl<'a, 'b, 'c, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawDockingClamp<'a, 'b, 'c> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        match self.clamp.connection() {
            None => todo!(),
            Some(connection) => {
                self.vessel_render_model
                    .get()
                    .render(
                        renderer,
                        &connection.vessel,
                        self.logger,
                        Some(bounding_box),
                    )
                    .unwrap();
            }
        }
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawDockingClamps<'a, 'b, 'c> {
    clamps: &'a [DockingClamp],
    logger: &'b MemLogger,
    vessel_render_model: &'c LazyVesselRenderModel,
}

impl<'a, 'b, 'c> DrawDockingClamps<'a, 'b, 'c> {
    pub fn new(
        clamps: &'a [DockingClamp],
        logger: &'b MemLogger,
        vessel_render_model: &'c LazyVesselRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            clamps,
            logger,
            vessel_render_model,
        })
    }
}

impl<'a, 'b, 'c, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawDockingClamps<'a, 'b, 'c> {
    fn visible(&self) -> bool {
        !self.clamps.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(
            self.clamps
                .iter()
                .map(|clamp| DrawDockingClamp::new(clamp, self.logger, self.vessel_render_model))
                .map(|x| x as Box<dyn LayoutElement<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawDockingConnector<'a> {
    connector: &'a DockingConnector,
}

impl<'a> DrawDockingConnector<'a> {
    pub fn new(connector: &'a DockingConnector) -> Box<Self> {
        Box::new(Self { connector })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawDockingConnector<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{:?}", self.connector),
            bounding_box,
            Alignment::Center,
            Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        );
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawDockingConnectors<'a> {
    connectors: &'a [DockingConnector],
}

impl<'a> DrawDockingConnectors<'a> {
    pub fn new(connectors: &'a [DockingConnector]) -> Box<Self> {
        Box::new(Self { connectors })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawDockingConnectors<'a> {
    fn visible(&self) -> bool {
        !self.connectors.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(
            self.connectors
                .iter()
                .map(DrawDockingConnector::new)
                .map(|x| x as Box<dyn LayoutElement<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawDockingStuff<'a, 'b, 'c> {
    module: &'a dyn Module,
    logger: &'b MemLogger,
    vessel_render_model: &'c LazyVesselRenderModel,
}

impl<'a, 'b, 'c> DrawDockingStuff<'a, 'b, 'c> {
    pub fn new(
        module: &'a dyn Module,
        logger: &'b MemLogger,
        vessel_render_model: &'c LazyVesselRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            module,
            logger,
            vessel_render_model,
        })
    }
}

impl<'a, 'b, 'c, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawDockingStuff<'a, 'b, 'c> {
    fn visible(&self) -> bool {
        !self.module.docking_clamps().is_empty() || !self.module.docking_connectors().is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(vec![
            DrawDockingClamps::new(
                self.module.docking_clamps(),
                self.logger,
                self.vessel_render_model,
            ),
            DrawDockingConnectors::new(self.module.docking_connectors()),
        ]);

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawBuyOffer<'a> {
    offer: &'a BuyOffer,
}

impl<'a> DrawBuyOffer<'a> {
    pub fn new(offer: &'a BuyOffer) -> Box<Self> {
        Box::new(Self { offer })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawBuyOffer<'a> {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        todo!()
    }
}

struct DrawBuyOffers<'a> {
    offers: &'a [BuyOffer],
}

impl<'a> DrawBuyOffers<'a> {
    pub fn new(offers: &'a [BuyOffer]) -> Box<Self> {
        Box::new(Self { offers })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawBuyOffers<'a> {
    fn visible(&self) -> bool {
        !self.offers.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        todo!()
    }
}

struct DrawSellOffer<'a> {
    offer: &'a SellOffer,
}

impl<'a> DrawSellOffer<'a> {
    pub fn new(offer: &'a SellOffer) -> Box<Self> {
        Box::new(Self { offer })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawSellOffer<'a> {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        todo!()
    }
}

struct DrawSellOffers<'a> {
    offers: &'a [SellOffer],
}

impl<'a> DrawSellOffers<'a> {
    pub fn new(offers: &'a [SellOffer]) -> Box<Self> {
        Box::new(Self { offers })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawSellOffers<'a> {
    fn visible(&self) -> bool {
        !self.offers.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        todo!()
    }
}

struct DrawBuyVesselOffer<'a> {
    offer: &'a BuyVesselOffer,
}

impl<'a> DrawBuyVesselOffer<'a> {
    pub fn new(offer: &'a BuyVesselOffer) -> Box<Self> {
        Box::new(Self { offer })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawBuyVesselOffer<'a> {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        todo!()
    }
}

struct DrawBuyVesselOffers<'a> {
    offers: &'a [BuyVesselOffer],
}

impl<'a> DrawBuyVesselOffers<'a> {
    pub fn new(offers: &'a [BuyVesselOffer]) -> Box<Self> {
        Box::new(Self { offers })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawBuyVesselOffers<'a> {
    fn visible(&self) -> bool {
        !self.offers.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        todo!()
    }
}

struct DrawBuyCustomVesselOffer<'a> {
    offer: Option<&'a BuyCustomVesselOffer>,
}

impl<'a> DrawBuyCustomVesselOffer<'a> {
    pub fn new(offer: Option<&'a BuyCustomVesselOffer>) -> Box<Self> {
        Box::new(Self { offer })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawBuyCustomVesselOffer<'a> {
    fn visible(&self) -> bool {
        self.offer.is_some()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        todo!()
    }
}

struct DrawTradingInfo<'a> {
    module: &'a dyn Module,
}

impl<'a> DrawTradingInfo<'a> {
    pub fn new(module: &'a dyn Module) -> Box<Self> {
        Box::new(Self { module })
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for DrawTradingInfo<'a> {
    fn visible(&self) -> bool {
        self.module.trading_console().is_some()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let console = self.module.trading_console().unwrap();

        let layout = RowLayout::new(vec![
            DrawBuyOffers::new(console.buy_offers()),
            DrawSellOffers::new(console.sell_offers()),
            DrawBuyVesselOffers::new(console.buy_vessel_offers()),
            DrawBuyCustomVesselOffer::new(console.buy_custom_vessel_offer()),
        ]);

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

pub struct ModuleRenderModel {
    person_render_model: PersonRenderModel,
    vessel_render_model: LazyVesselRenderModel,
}

impl ModuleRenderModel {
    pub fn new() -> Self {
        Self {
            person_render_model: PersonRenderModel::new(),
            vessel_render_model: LazyVesselRenderModel::new(),
        }
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        renderer: &mut Renderer<T>,
        module: &dyn Module,
        logger: &MemLogger,
        bounding_box: Rect<Float>,
    ) -> Result<(), RenderError> {
        if !renderer.intersects_with_view_port(&bounding_box) {
            return Ok(());
        }

        draw_top_info(renderer, module, bounding_box);
        draw_bottom_info(renderer, module, bounding_box);
        draw_bounding_box(renderer, bounding_box);

        let column = ColumnLayout::new(vec![
            DrawPersons::new(module, logger, &self.person_render_model),
            DrawRecipes::new(module),
            DrawStorages::new(module),
            DrawDockingStuff::new(module, logger, &self.vessel_render_model),
            DrawTradingInfo::new(module),
        ]);

        column.draw(renderer, bounding_box);

        Ok(())
    }
}
