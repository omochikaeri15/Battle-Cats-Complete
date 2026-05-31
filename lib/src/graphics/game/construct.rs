use crate::graphics::animation::FrameData;
use crate::graphics::game::transform::WorldTransform;
use crate::graphics::data::imgcut::SpriteSheet;

pub fn build_geometry(
    parts: &[WorldTransform],
    sheet: &SpriteSheet,
) -> Vec<FrameData> {
    let mut frame_output = Vec::with_capacity(parts.len());

    for part in parts {
        if part.hidden || part.opacity < 0.005 { continue; }
        let Some(cut) = sheet.cuts_map.get(&part.sprite_index) else { continue; };

        let sprite_width = cut.original_size.x;
        let sprite_height = cut.original_size.y;
        let pivot_x = part.pivot.x;
        let pivot_y = part.pivot.y;

        let vertices: [f32; 12] = [
            -pivot_x,               -pivot_y,
            sprite_width - pivot_x, -pivot_y,
            -pivot_x,               sprite_height - pivot_y,

            -pivot_x,               sprite_height - pivot_y,
            sprite_width - pivot_x, -pivot_y,
            sprite_width - pivot_x, sprite_height - pivot_y,
        ];

        let uv_coordinates = cut.uv_coordinates;
        let uvs: [f32; 12] = [
            uv_coordinates.min.x, uv_coordinates.min.y,
            uv_coordinates.max.x, uv_coordinates.min.y,
            uv_coordinates.min.x, uv_coordinates.max.y,

            uv_coordinates.min.x, uv_coordinates.max.y,
            uv_coordinates.max.x, uv_coordinates.min.y,
            uv_coordinates.max.x, uv_coordinates.max.y,
        ];

        frame_output.push(FrameData {
            sprite_index: part.sprite_index,
            final_matrix: part.matrix,
            vertices,
            uvs,
            opacity: part.opacity,
            glow: part.glow,
        });
    }

    frame_output
}