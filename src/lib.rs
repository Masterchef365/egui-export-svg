pub use svg;

use egui::{epaint::ClippedShape, Color32, LayerId, Shape as EguiShape, Ui};
use svg::node::element::{path::Data, Group, Path as SvgPath};

pub enum TextMode {
    /// Use SVG's builtin fonts
    Native,
    /// Convert fonts to meshes before export. EXPENSIVE!
    Meshed {
        /// Generate (invisible) selectable text?
        copyable: bool,
    },
}

pub enum MeshMode {
    /// Only approximate support opaque, textureless meshes
    Basic,
    /// Embed textures
    Textures,
}

pub struct ConversionOptions {
    pub text: TextMode,
    pub mesh: MeshMode,
}

impl ConversionOptions {
    /// Lowest detail, lowest file size, highest performance
    pub fn minimal() -> Self {
        Self {
            text: TextMode::Native,
            mesh: MeshMode::Basic,
        }
    }

    /// Highest detail, largest file size, lowest performance
    pub fn full() -> Self {
        Self {
            text: TextMode::Meshed { copyable: true },
            mesh: MeshMode::Textures,
        }
    }
}


pub fn shape_to_path(shape: &egui::Shape) -> Box<dyn svg::Node> {
    match shape {
        egui::Shape::Mesh(mesh) => {
            let mut group = Group::new();
            // TODO: Fast special case for vertices with of all the same color!
            let mut tri: [usize; 3] = [0; 3];
            for tri_indices in mesh.indices.chunks_exact(3) {
                tri.iter_mut()
                    .zip(tri_indices)
                    .for_each(|(o, i)| *o = *i as usize);

                // Draw the shape
                let mut data = Data::new();
                let first_pt = mesh.vertices[tri[0]].pos;
                data = data.move_to((first_pt.x, first_pt.y));
                for idx in &tri[1..] {
                    let pt = mesh.vertices[*idx as usize].pos;
                    data = data.line_to((pt.x, pt.y));
                }
                data = data.close();

                let color = mesh.vertices[tri[0] as usize].color;
                let path = svg::node::element::Path::new().fill(color).set("d", data);

                group = group.add(path);
            }
            Box::new(group)
        }
        egui::Shape::Noop => Box::new(SvgPath::default()),
        egui::Shape::Vec(children) => {
            let mut group = Group::new();
            for child in children {
                group = group.add(shape_to_path(child));
            }
            Box::new(group)
        }
        egui::Shape::Path(path) => {
            let mut data = Data::new();
            if let Some(pt) = path.points.first() {
                data = data.move_to((pt.x, pt.y));
            }
            for pt in &path.points[1..] {
                data = data.line_to((pt.x, pt.y));
            }
            if path.closed {
                data = data.close();
            }

            Box::new(
                svg::node::element::Path::new()
                    .fill(path.fill)
                    .stroke(path.stroke)
                    .set("d", data),
            )
        }
        egui::Shape::Circle(circle) => Box::new(
            svg::node::element::Circle::new()
                .set("cx", circle.center.x)
                .set("cy", circle.center.y)
                .set("r", circle.radius)
                .fill(circle.fill)
                .stroke(circle.stroke),
        ),
        egui::Shape::LineSegment { points, stroke } => Box::new(
            svg::node::element::Line::new()
                .set("x1", points[0].x)
                .set("y1", points[0].y)
                .set("x2", points[1].x)
                .set("y2", points[1].y)
                .stroke(*stroke),
        ),
        EguiShape::Rect(rectangle) => {
            if !rectangle.rounding.is_same() {
                eprintln!("TODO: Implement per-edge rounding ...")
            }

            let rounding = 0_f32
                .max(rectangle.rounding.nw)
                .max(rectangle.rounding.ne)
                .max(rectangle.rounding.sw)
                .max(rectangle.rounding.se);

            Box::new(
                svg::node::element::Rectangle::new()
                    .set("x", rectangle.rect.min.x)
                    .set("y", rectangle.rect.min.y)
                    .set("rx", rounding)
                    .set("ry", rounding)
                    .set("width", rectangle.rect.width())
                    .set("height", rectangle.rect.height())
                    .fill(rectangle.fill)
                    .stroke(rectangle.stroke),
            )
        }
        EguiShape::Text(text) => {
            let mut group = Group::new();

            let n_rows = text.galley.rows.len();
            for (row_idx, row) in text.galley.rows.iter().enumerate() {
                let Some(last_section_idx_in_row) = row.glyphs.last().map(|s| s.section_index)
                else {
                    continue;
                };

                for sec_idx in row.section_index_at_start..=last_section_idx_in_row {
                    let sec = &text.galley.job.sections[sec_idx as usize];

                    let stretch: f32;
                    if text.galley.job.justify
                        && !row.ends_with_newline
                        && row_idx + 1 != n_rows
                        && sec_idx == last_section_idx_in_row
                    {
                        // If justified, stretch until the end of the line
                        stretch = text.galley.rect.width();
                    } else {
                        // Otherwise, only go as far as you need to
                        let trailing_space = row
                            .glyphs
                            .iter()
                            .filter(|glyph| glyph.section_index == sec_idx)
                            .filter(|glyph| glyph.chr.is_whitespace())
                            .last()
                            .map(|glyph| glyph.size.x)
                            .unwrap_or(0.0);

                        stretch = row
                            .glyphs
                            .iter()
                            .filter(|glyph| glyph.section_index == sec_idx)
                            .map(|glyph| glyph.size.x)
                            .sum::<f32>()
                            - trailing_space;
                    }

                    let substring: String = row
                        .glyphs
                        .iter()
                        .filter(|glyph| glyph.section_index == sec_idx)
                        .map(|glyph| glyph.chr)
                        .collect();

                    let first_glyph_pos = row
                        .glyphs
                        .iter()
                        .find(|glyph| glyph.section_index == sec_idx)
                        .map(|glyph| glyph.pos)
                        .unwrap_or(row.rect.min);

                    let tl_pos = text.pos + first_glyph_pos.to_vec2();

                    let font_family = match &sec.format.font_id.family {
                        egui::FontFamily::Proportional => "sans-serif",
                        egui::FontFamily::Monospace => "monospace",
                        egui::FontFamily::Name(fam) => {
                            eprintln!("Font family {} unsupported!", fam);
                            "sans-serif"
                        }
                    };

                    let font_size = sec.format.font_id.size;
                    let mut color = text.override_text_color.unwrap_or(sec.format.color);
                    if color == Color32::PLACEHOLDER {
                        color = text.fallback_color;
                    }

                    group = group.add(
                        svg::node::element::Text::new(substring)
                            .set("x", tl_pos.x)
                            .set("y", tl_pos.y)
                            .set("font-size", font_size)
                            .set("font-family", font_family)
                            .set("textLength", stretch)
                            .fill(color),
                    );
                }
            }

            Box::new(group)
        }
        other => {
            println!("{:?}", other);
            Box::new(SvgPath::default())
        }
    }
}

fn sorted_layer_ids(ctx: &egui::Context) -> Vec<LayerId> {
    let mut layer_ids: Vec<LayerId> = ctx.memory(|mem| mem.layer_ids().collect());
    layer_ids.sort_by_key(|id| id.order);
    layer_ids
}

fn color32_rgba(color: Color32) -> String {
    format!(
        "rgb({}, {}, {})",
        color.r(),
        color.g(),
        color.b(),
    )
}

trait EguiColorable: svg::Node + Sized {
    fn fill(mut self, color: Color32) -> Self {
        self.assign("fill", color32_rgba(color));
        if !color.is_opaque() {
            self.assign("fill-opacity", color.r() as f32 / 255.0);
        }
        self
    }

    fn stroke(mut self, stroke: egui::Stroke) -> Self {
        if stroke != egui::Stroke::NONE {
            self.assign("stroke-width", stroke.width);
            self.assign("stroke", color32_rgba(stroke.color));
        }
        self
    }
}

impl<T: svg::Node + Sized> EguiColorable for T {}

/*
pub fn wrap(ui: &mut Ui, f: impl FnOnce(&mut Ui) -> egui::InnerResponse<bool>) -> egui::Response {

}
*/

fn rect_to_viewbox(rect: egui::Rect) -> (f32, f32, f32, f32) {
    (rect.min.x, rect.min.y, rect.width(), rect.height())
}

/// Take a snapshot of the entire screen as SVG
pub fn snapshot(ctx: &egui::Context) -> svg::Document {
    // Steal graphics data from context
    let layer_ids = sorted_layer_ids(ctx);
    let clipped_shapes: Vec<ClippedShape> = ctx.graphics(|gfx| {
        layer_ids
            .into_iter()
            .filter_map(|id| {
                gfx.get(id)
                    .map(|paint| paint.all_entries().cloned().collect::<Vec<_>>())
            })
            .flatten()
            .collect()
    });

    // Set viewbox to screen rect.
    // TODO: Is this what we want?
    let viewbox = rect_to_viewbox(ctx.screen_rect());

    svg::Document::new()
        .set("viewBox", viewbox)
        .add(clipped_shapes_to_group(&clipped_shapes))
}

fn clipped_shapes_to_group(shapes: &[ClippedShape]) -> Group {
    let mut output_group = Group::new();

    let mut current_clip_id = 0;
    let mut current_clipgroup = None;
    let mut current_clip_rect = None;

    for clip_shape in shapes {
        // Clip rectangles must be each assigned an ID
        // TODO: Make this more efficient- re-use IDs!

        if Some(clip_shape.clip_rect) != current_clip_rect {
            let clip_id_name = format!("clip_rect_{current_clip_id}");
            current_clip_id += 1;

            let clip_path = svg::node::element::ClipPath::new()
                .set("id", clip_id_name.clone())
                .add(
                    svg::node::element::Rectangle::new()
                        .set("x", clip_shape.clip_rect.min.x)
                        .set("y", clip_shape.clip_rect.min.y)
                        .set("width", clip_shape.clip_rect.width())
                        .set("height", clip_shape.clip_rect.height()),
                );

            let new_clip_group = Group::new()
                .set("clip-path", format!("url(#{clip_id_name})"))
                .add(clip_path);

            if let Some(old) = std::mem::replace(&mut current_clipgroup, Some(new_clip_group)) {
                output_group = output_group.add(old);
            }

            current_clip_rect = Some(clip_shape.clip_rect);
        }

        current_clipgroup = Some(
            current_clipgroup
                .take()
                .unwrap()
                .add(shape_to_path(&clip_shape.shape)),
        );
    }

    if let Some(last_clipgroup) = current_clipgroup {
        output_group = output_group.add(last_clipgroup);
    }

    output_group
}

/// Runs the given function and, if it returns `true`, returns an SVG document
pub fn capture_scope(ui: &mut Ui, f: impl FnOnce(&mut Ui) -> bool) -> Option<svg::Document> {
    let layer_ids = sorted_layer_ids(ui.ctx());

    let lengths_before: Vec<usize> = ui.ctx().graphics(|gfx| {
        layer_ids
            .iter()
            .copied()
            .filter_map(|id| gfx.get(id).map(|paint| paint.all_entries().len()))
            .collect()
    });

    let do_capture = f(ui);

    do_capture.then(|| {
        // Find the difference between the old and new shape vectors
        let mut new_clipped_shapes: Vec<ClippedShape> = ui.ctx().graphics(|gfx| {
            layer_ids
                .into_iter()
                .zip(lengths_before)
                .filter_map(|(id, idx_before)| {
                    gfx.get(id).map(|paint| {
                        paint
                            .all_entries()
                            .skip(idx_before)
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                })
                .flatten()
                .collect()
        });

        let total_rect = new_clipped_shapes
            .iter()
            .fold(egui::Rect::NOTHING, |acc, x| {
                acc.union(x.shape.visual_bounding_rect())
            });

        // Translate everything to the top left corner
        let to_tl = -total_rect.min.to_vec2();
        new_clipped_shapes.iter_mut().for_each(|clip_shape| {
            clip_shape.clip_rect = clip_shape.clip_rect.translate(to_tl);
            clip_shape.shape.translate(to_tl);
        });

        let viewbox = rect_to_viewbox(total_rect.translate(to_tl));

        svg::Document::new()
            .set("viewBox", viewbox)
            .add(clipped_shapes_to_group(&new_clipped_shapes))
    })
}
