use crate::render::scene_graph::GraphicsNode;
use crate::render::{Distance, Margins, OLD_DEFAULT_MARGIN, Pix, Renderer};
use burbomath::math::{Rect, Size};
use dudes_in_space_api::utils::utils::Float;

pub struct ColumnLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<Box<dyn GraphicsNode<T> + 'a>>,
}

impl<'a, T: sdl2::render::RenderTarget + 'a> ColumnLayout<'a, T> {
    pub fn new(elems: Vec<Box<dyn GraphicsNode<T> + 'a>>) -> Self {
        Self { elems }
    }

    pub fn boxed(elems: Vec<Box<dyn GraphicsNode<T> + 'a>>) -> Box<dyn GraphicsNode<T> + 'a> {
        Box::new(Self { elems })
    }
}

impl<'a, T: sdl2::render::RenderTarget + 'a> From<ColumnLayout<'a, T>>
    for Box<dyn GraphicsNode<T> + 'a>
{
    fn from(value: ColumnLayout<'a, T>) -> Self {
        Box::new(value)
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for ColumnLayout<'a, T> {
    fn visible(&self) -> bool {
        self.elems.iter().any(|x| x.visible())
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let (bounding_box, margin) =
            bounding_box.homogeneous_mul(OLD_DEFAULT_MARGIN.assume_relative().unwrap().value());

        if !renderer.intersects_with_view_port(&bounding_box) {
            return;
        }

        let count = self.elems.iter().filter(|x| x.visible()).count();
        if count == 0 {
            return;
        }

        let margin = margin / 2.;
        let sum_margin = margin.abs() * (count - 1) as Float;

        let x = *bounding_box.x();
        let y = *bounding_box.y();
        let w = *bounding_box.w();
        let h = (bounding_box.h() - sum_margin) / count as Float;

        let mut i = 0;

        for elem in &self.elems {
            if elem.visible() {
                elem.draw(
                    renderer,
                    (x, i as Float * (h + margin.abs()) + y, w, h).into(),
                );
                i += 1;
            }
        }
    }

    fn implicit_size(&self) -> Option<Size<Float>> {
        let v: Vec<_> = self
            .elems
            .iter()
            .filter_map(|elem| elem.visible().then(|| elem.implicit_size()).flatten())
            .collect();

        if v.is_empty() {
            return None;
        }

        let mut result: Size<_> = (0., 0.).into();

        for size in v {
            result = (Float::max(*result.w(), *size.w()), *result.h() + *size.h()).into();
        }

        Some(result)
    }
}

#[derive(Default)]
pub(crate) struct ExtColumnLayoutOptions {
    preserve_aspect_ratio: bool,
    relative_height: Option<Float>,
    margins: Margins,
    fill_height: bool,
}

impl ExtColumnLayoutOptions {
    pub fn relative_height(h: Float) -> Self {
        Self {
            preserve_aspect_ratio: false,
            relative_height: Some(h),
            margins: Default::default(),
            fill_height: false,
        }
    }

    pub fn margins(margins: Margins) -> Self {
        Self {
            preserve_aspect_ratio: false,
            relative_height: None,
            margins,
            fill_height: false,
        }
    }

    pub fn fill_height() -> Self {
        Self {
            preserve_aspect_ratio: false,
            relative_height: None,
            margins: Default::default(),
            fill_height: true,
        }
    }
}

pub struct ExtColumnLayout<'a, T: sdl2::render::RenderTarget> {
    spacing: Distance,
    elems: Vec<(ExtColumnLayoutOptions, Box<dyn GraphicsNode<T> + 'a>)>,
}

impl<'a, T: sdl2::render::RenderTarget + 'a> ExtColumnLayout<'a, T> {
    pub(crate) fn new(
        spacing: Distance,
        elems: Vec<(ExtColumnLayoutOptions, Box<dyn GraphicsNode<T> + 'a>)>,
    ) -> Self {
        Self { spacing, elems }
    }

    pub fn boxed(
        spacing: Distance,

        elems: Vec<(ExtColumnLayoutOptions, Box<dyn GraphicsNode<T> + 'a>)>,
    ) -> Box<dyn GraphicsNode<T> + 'a> {
        Box::new(Self { spacing, elems })
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for ExtColumnLayout<'a, T> {
    fn visible(&self) -> bool {
        self.elems.iter().any(|(_, x)| x.visible())
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let (bounding_box, margin) =
            bounding_box.homogeneous_mul(OLD_DEFAULT_MARGIN.assume_relative().unwrap().value());

        if !renderer.intersects_with_view_port(&bounding_box) {
            return;
        }

        let count = self.elems.iter().filter(|(_, x)| x.visible()).count();
        if count == 0 {
            return;
        }

        let margin = margin / 2.;
        let sum_margin = margin.abs() * (count - 1) as Float;

        let x = *bounding_box.x();
        let mut y = *bounding_box.y();
        let w = *bounding_box.w();

        let elems_with_relative_height_count = self
            .elems
            .iter()
            .filter(|(options, elem)| elem.visible() && options.relative_height.is_some())
            .count();
        let sum_relative_height = self
            .elems
            .iter()
            .filter_map(|(options, elem)| {
                if elem.visible() {
                    options.relative_height
                } else {
                    None
                }
            })
            .sum::<Float>();
        assert!(sum_relative_height >= 0.);
        assert!(sum_relative_height <= 1.);

        let rest_relative_height = (1. - sum_relative_height)
            / (self.elems.len() - elems_with_relative_height_count) as Float;

        for (options, elem) in &self.elems {
            if elem.visible() {
                let h = options.relative_height.unwrap_or(rest_relative_height) * bounding_box.h();

                elem.draw(renderer, (x, y, w, h).into());
                y += h;
            }
        }
    }

    // fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
    //     let (bounding_box, margin) =
    //         bounding_box.homogeneous_mul(OLD_DEFAULT_MARGIN.assume_relative().unwrap().value());
    //
    //     if !renderer.intersects_with_view_port(&bounding_box) {
    //         return;
    //     }
    //
    //     let visible_elems: Vec<_> = self.elems.iter().filter(|(_, x)| x.visible()).collect();
    //     if visible_elems.len() == 0 {
    //         return;
    //     }
    //
    //     let sum_implicit_size = self.implicit_size().unwrap();
    //
    //     let static_elements_sum_size: Size<Float> = sum_sizes_with_spacing(
    //         visible_elems
    //             .iter()
    //             .filter(|(options, _)| !options.fill_height)
    //             .map(|(options, elem)| {
    //                 implicit_size_with_margins(elem.implicit_size().unwrap_or((0.,0.).into()), &options.margins)
    //             })
    //             .collect(),
    //         self.spacing,
    //     )
    //     .unwrap_or((*bounding_box.w(),0.).into());
    //
    //     let remaining_implicit_height_for_dynamic_sized_elems =
    //         sum_implicit_size.h() - static_elements_sum_size.h();
    //
    //     let remaining_height_for_dynamic_sized_elems =
    //         bounding_box.h() - static_elements_sum_size.h();
    //
    //     let f = remaining_height_for_dynamic_sized_elems
    //         / remaining_implicit_height_for_dynamic_sized_elems;
    //
    //
    //
    //     renderer.draw_text( &format!("r: {:.2}, f: {:.2}",remaining_height_for_dynamic_sized_elems, f),bounding_box.left_top(), bounding_box.w() / 10., Alignment::left_top(), Color::red()).unwrap();
    //
    //     let all_without_implicit_size = visible_elems.iter().all(|(_, elem)| elem.implicit_size().is_none());
    //
    //     let mut y_offset = 0.;
    //     for (i, (options, elem)) in visible_elems.iter().enumerate() {
    //         // let implicit_size =
    //         //     implicit_size_with_margins(elem.implicit_size().unwrap(), &options.margins);
    //         let implicit_size =
    //             elem.implicit_size().unwrap_or((0., 0.).into());
    //
    //         let left_margin = options.margins.left.to_pix(Pix(*implicit_size.w())).value();
    //         let right_margin = options.margins.right.to_pix(Pix(*implicit_size.w())).value();
    //         let top_margin = options.margins.top.to_pix(Pix(*implicit_size.h())).value();
    //         let bottom_margin = options.margins.bottom.to_pix(Pix(*implicit_size.h())).value();
    //
    //         println!("implicit_s: {:?}", implicit_size);
    //         println!("left_margin: {}, right_margin: {}, top_margin: {}, bottom_margin: {}", left_margin, right_margin, top_margin, bottom_margin);
    //
    //         y_offset += top_margin;
    //         let (w, mut h) = implicit_size.into();
    //         let x = *bounding_box.x() + left_margin;
    //         let x = w - left_margin - right_margin;
    //         let y=  bounding_box.y() + y_offset;
    //
    //         let elem_bb = if remaining_height_for_dynamic_sized_elems < 0. {
    //             if options.fill_height {
    //                 (x, y, w, 0.).into()
    //             } else {
    //                 h = f * h;
    //                 y_offset += h;
    //                 (x, y, w, h).into()
    //             }
    //         } else {
    //             if options.fill_height {
    //                 // if all_without_implicit_size {
    //                 //     h = f * bounding_box.h();
    //                 // } else {
    //                     h = f * h;
    //                 // }
    //             }
    //             y_offset += h;
    //             (x, y, w, h).into()
    //         };
    //
    //         y_offset += bottom_margin;
    //
    //         if i < visible_elems.len() - 1 {
    //             y_offset += self.spacing.assume_pix().unwrap().value();
    //         }
    //
    //         elem.draw(renderer, elem_bb);
    //         renderer.draw_rect(elem_bb, Color::red());
    //     }
    // }

    fn implicit_size(&self) -> Option<Size<Float>> {
        let v: Vec<_> = self
            .elems
            .iter()
            .filter_map(|(options, elem)| {
                elem.visible()
                    .then(|| {
                        elem.implicit_size().map(|implicit_size| {
                            implicit_size_with_margins(implicit_size, &options.margins)
                        })
                    })
                    .flatten()
            })
            .collect();

        sum_sizes_with_spacing(v, self.spacing)
    }
}

fn implicit_size_with_margins(implicit_size: Size<Float>, margins: &Margins) -> Size<Float> {
    let (w, h) = implicit_size.into();
    (
        w + margins.left.to_pix(Pix(w)).value() + margins.right.to_pix(Pix(w)).value(),
        h + margins.top.to_pix(Pix(h)).value() + margins.bottom.to_pix(Pix(h)).value(),
    )
        .into()
}

fn sum_sizes_with_spacing(v: Vec<Size<Float>>, spacing: Distance) -> Option<Size<Float>> {
    if v.is_empty() {
        return None;
    }

    let mut result: Size<_> = (0., 0.).into();

    for (i, size) in v.iter().enumerate() {
        let spacing = if i < v.len() - 1 {
            spacing.assume_pix().unwrap().value()
        } else {
            0.
        };
        result = (
            Float::max(*result.w(), *size.w()),
            *result.h() + *size.h() + spacing,
        )
            .into();
    }

    Some(result)
}
