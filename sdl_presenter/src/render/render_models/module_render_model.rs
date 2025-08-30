use crate::logger::MemLogger;
use crate::person_table::PersonTable;
use crate::render::render_models::person_render_model::PersonRenderModel;
use crate::render::renderer::Renderer;
use crate::render::scene_graph::{ColumnLayout, GraphicsNode, GridLayout, RowLayout};
use crate::render::{
    HorisontalAlignment, ItemStorageRenderModel, LazyVesselRenderModel, RenderError,
};
use dudes_in_space_api::item::{ItemSafe, ItemStorage};
use dudes_in_space_api::module::{Module, ModuleStorage};
use dudes_in_space_api::person::Person;
use dudes_in_space_api::recipe::{AssemblyRecipe, InputItemRecipe, ItemRecipe, OutputItemRecipe};
use dudes_in_space_api::trade::{BuyCustomVesselOffer, BuyOffer, BuyVesselOffer, SellOffer};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::Rect;
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector};

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
        HorisontalAlignment::Center,
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
        HorisontalAlignment::Center,
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

impl<'a, 'b, 'c, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawPerson<'a, 'b, 'c> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        match self.person {
            None => renderer.draw_confined_text(
                "Vacant",
                bounding_box,
                HorisontalAlignment::Center,
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

impl<'a, 'b, 'c, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawPersons<'a, 'b, 'c> {
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
                .map(|x| x as Box<dyn GraphicsNode<_>>)
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawItemRecipe<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{} -> {}", self.recipe.input, self.recipe.output),
            bounding_box,
            HorisontalAlignment::Center,
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawItemRecipes<'a> {
    fn visible(&self) -> bool {
        !self.recipes.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = ColumnLayout::new(
            self.recipes
                .iter()
                .map(DrawItemRecipe::new)
                .map(|x| x as Box<dyn GraphicsNode<_>>)
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawInputItemRecipe<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{} ->", self.recipe),
            bounding_box,
            HorisontalAlignment::Center,
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawInputItemRecipes<'a> {
    fn visible(&self) -> bool {
        !self.recipes.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = ColumnLayout::new(
            self.recipes
                .iter()
                .map(DrawInputItemRecipe::new)
                .map(|x| x as Box<dyn GraphicsNode<_>>)
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawOutputItemRecipe<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("-> {}", self.recipe),
            bounding_box,
            HorisontalAlignment::Center,
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawOutputItemRecipes<'a> {
    fn visible(&self) -> bool {
        !self.recipes.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = ColumnLayout::new(
            self.recipes
                .iter()
                .map(DrawOutputItemRecipe::new)
                .map(|x| x as Box<dyn GraphicsNode<_>>)
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawAssemblyRecipe<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!(
                "{} -> {}",
                self.recipe.input(),
                self.recipe.output_description().type_id(),
            ),
            bounding_box,
            HorisontalAlignment::Center,
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawAssemblyRecipes<'a> {
    fn visible(&self) -> bool {
        !self.recipes.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = ColumnLayout::new(
            self.recipes
                .iter()
                .map(DrawAssemblyRecipe::new)
                .map(|x| x as Box<dyn GraphicsNode<_>>)
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawRecipes<'a> {
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

struct DrawItemStorage<'a, 'b> {
    storage: &'a ItemStorage,
    model: &'b ItemStorageRenderModel,
}

impl<'a, 'b> DrawItemStorage<'a, 'b> {
    pub fn new(storage: &'a ItemStorage, model: &'b ItemStorageRenderModel) -> Box<Self> {
        Box::new(Self { storage, model })
    }
}

impl<'a, 'b, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawItemStorage<'a, 'b> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        self.model
            .render(renderer, self.storage, bounding_box)
            .unwrap();
    }
}

struct DrawItemStorages<'a, 'b> {
    storages: Vec<&'a ItemStorage>,
    model: &'b ItemStorageRenderModel,
}

impl<'a, 'b> DrawItemStorages<'a, 'b> {
    pub fn new(storages: Vec<&'a ItemStorage>, model: &'b ItemStorageRenderModel) -> Box<Self> {
        Box::new(Self { storages, model })
    }
}

impl<'a, 'b, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawItemStorages<'a, 'b> {
    fn visible(&self) -> bool {
        !self.storages.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(
            self.storages
                .iter()
                .cloned()
                .map(|x| DrawItemStorage::new(x, self.model))
                .map(|x| x as Box<dyn GraphicsNode<_>>)
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawItemSafe<'a> {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{:?}", self.safe,),
            bounding_box,
            HorisontalAlignment::Center,
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawItemSafes<'a> {
    fn visible(&self) -> bool {
        !self.safes.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(
            self.safes
                .iter()
                .map(DrawItemSafe::new)
                .map(|x| x as Box<dyn GraphicsNode<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawModuleStorage<'a, 'b> {
    storage: &'a ModuleStorage,
    module_render_model: &'b ModuleRenderModel,
}

impl<'a, 'b> DrawModuleStorage<'a, 'b> {
    pub fn new(
        storage: &'a ModuleStorage,
        module_render_model: &'b ModuleRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            storage,
            module_render_model,
        })
    }
}

impl<'a, 'b, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawModuleStorage<'a, 'b> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout: GridLayout<_> = self
            .storage
            .content()
            .iter()
            .map(|module| {
                |renderer: &mut Renderer<T>, bounding_box| {
                    renderer.draw_confined_text(
                        &format!("{}", module.type_id()),
                        bounding_box,
                        HorisontalAlignment::Center,
                        Color::black(),
                    );
                    renderer.draw_rect(bounding_box, Color::black());
                }
            })
            .collect();

        layout.draw(renderer, bounding_box);
        renderer.draw_rect(bounding_box, Color::black());
    }
}

struct DrawModuleStorages<'a, 'b> {
    storages: &'a [ModuleStorage],
    module_render_model: &'b ModuleRenderModel,
}

impl<'a, 'b> DrawModuleStorages<'a, 'b> {
    pub fn new(
        storages: &'a [ModuleStorage],
        module_render_model: &'b ModuleRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            storages,
            module_render_model,
        })
    }
}

impl<'a, 'b, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawModuleStorages<'a, 'b> {
    fn visible(&self) -> bool {
        !self.storages.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(
            self.storages
                .iter()
                .map(|storage| DrawModuleStorage::new(storage, self.module_render_model))
                .map(|x| x as Box<dyn GraphicsNode<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawStorages<'a, 'b, 'c> {
    module: &'a dyn Module,
    item_storage_render_model: &'b ItemStorageRenderModel,
    module_render_model: &'c ModuleRenderModel,
}

impl<'a, 'b, 'c> DrawStorages<'a, 'b, 'c> {
    pub fn new(
        module: &'a dyn Module,
        item_storage_render_model: &'b ItemStorageRenderModel,
        module_render_model: &'c ModuleRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            module,
            item_storage_render_model,
            module_render_model,
        })
    }
}

impl<'a, 'b, 'c, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawStorages<'a, 'b, 'c> {
    fn visible(&self) -> bool {
        !self.module.storages().is_empty()
            || !self.module.safes().is_empty()
            || !self.module.module_storages().is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(vec![
            DrawItemStorages::new(self.module.storages(), self.item_storage_render_model),
            DrawItemSafes::new(self.module.safes()),
            DrawModuleStorages::new(self.module.module_storages(), self.module_render_model),
        ]);

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawDockingClamp<'a, 'b, 'c, 'd> {
    clamp: &'a DockingClamp,
    logger: &'b MemLogger,
    person_table: &'c PersonTable,
    vessel_render_model: &'d LazyVesselRenderModel,
}

impl<'a, 'b, 'c, 'd> DrawDockingClamp<'a, 'b, 'c, 'd> {
    pub fn new(
        clamp: &'a DockingClamp,
        logger: &'b MemLogger,
        person_table: &'c PersonTable,
        vessel_render_model: &'d LazyVesselRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            clamp,
            logger,
            person_table,
            vessel_render_model,
        })
    }
}

impl<'a, 'b, 'c, 'd, T: sdl2::render::RenderTarget> GraphicsNode<T>
    for DrawDockingClamp<'a, 'b, 'c, 'd>
{
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
                        self.person_table,
                        Some(bounding_box),
                    )
                    .unwrap();
            }
        }
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawDockingClamps<'a, 'b, 'c, 'd> {
    clamps: &'a [DockingClamp],
    logger: &'b MemLogger,
    person_table: &'c PersonTable,
    vessel_render_model: &'d LazyVesselRenderModel,
}

impl<'a, 'b, 'c, 'd> DrawDockingClamps<'a, 'b, 'c, 'd> {
    pub fn new(
        clamps: &'a [DockingClamp],
        logger: &'b MemLogger,
        person_table: &'c PersonTable,
        vessel_render_model: &'d LazyVesselRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            clamps,
            logger,
            person_table,
            vessel_render_model,
        })
    }
}

impl<'a, 'b, 'c, 'd, T: sdl2::render::RenderTarget> GraphicsNode<T>
    for DrawDockingClamps<'a, 'b, 'c, 'd>
{
    fn visible(&self) -> bool {
        !self.clamps.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(
            self.clamps
                .iter()
                .map(|clamp| {
                    DrawDockingClamp::new(
                        clamp,
                        self.logger,
                        self.person_table,
                        self.vessel_render_model,
                    )
                })
                .map(|x| x as Box<dyn GraphicsNode<_>>)
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawDockingConnector<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{:?}", self.connector),
            bounding_box,
            HorisontalAlignment::Center,
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawDockingConnectors<'a> {
    fn visible(&self) -> bool {
        !self.connectors.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(
            self.connectors
                .iter()
                .map(DrawDockingConnector::new)
                .map(|x| x as Box<dyn GraphicsNode<_>>)
                .collect(),
        );

        layout.draw(renderer, bounding_box);
        draw_bounding_box(renderer, bounding_box);
    }
}

struct DrawDockingStuff<'a, 'b, 'c, 'd> {
    module: &'a dyn Module,
    logger: &'b MemLogger,
    person_table: &'c PersonTable,
    vessel_render_model: &'d LazyVesselRenderModel,
}

impl<'a, 'b, 'c, 'd> DrawDockingStuff<'a, 'b, 'c, 'd> {
    pub fn new(
        module: &'a dyn Module,
        logger: &'b MemLogger,
        person_table: &'c PersonTable,
        vessel_render_model: &'d LazyVesselRenderModel,
    ) -> Box<Self> {
        Box::new(Self {
            module,
            logger,
            person_table,
            vessel_render_model,
        })
    }
}

impl<'a, 'b, 'c, 'd, T: sdl2::render::RenderTarget> GraphicsNode<T>
    for DrawDockingStuff<'a, 'b, 'c, 'd>
{
    fn visible(&self) -> bool {
        !self.module.docking_clamps().is_empty() || !self.module.docking_connectors().is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let layout = RowLayout::new(vec![
            DrawDockingClamps::new(
                self.module.docking_clamps(),
                self.logger,
                self.person_table,
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawBuyOffer<'a> {
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawBuyOffers<'a> {
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawSellOffer<'a> {
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawSellOffers<'a> {
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawBuyVesselOffer<'a> {
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawBuyVesselOffers<'a> {
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawBuyCustomVesselOffer<'a> {
    fn visible(&self) -> bool {
        self.offer.is_some()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &format!("{:?}", self.offer),
            bounding_box,
            HorisontalAlignment::Left,
            Color::black(),
        )
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

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawTradingInfo<'a> {
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
    item_storage_render_model: ItemStorageRenderModel,
}

impl ModuleRenderModel {
    pub fn new() -> Self {
        Self {
            person_render_model: PersonRenderModel::new(),
            vessel_render_model: LazyVesselRenderModel::new(),
            item_storage_render_model: ItemStorageRenderModel::new(),
        }
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        renderer: &mut Renderer<T>,
        module: &dyn Module,
        logger: &MemLogger,
        person_table: &PersonTable,
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
            DrawStorages::new(module, &self.item_storage_render_model, self),
            DrawDockingStuff::new(module, logger, person_table, &self.vessel_render_model),
            DrawTradingInfo::new(module),
        ]);

        column.draw(renderer, bounding_box);

        Ok(())
    }
}
