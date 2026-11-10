use super::{CellSource, CellUvInput, sprite_cell_uv};
use ph2d_ecs::{SpriteGrid, SpriteRegion};

const ATLAS: u32 = 2048;
const REGION: crate::AtlasRegion = crate::AtlasRegion {
    x: 37,
    y: 5,
    w: 128,
    h: 32,
};

fn frame(k: u32, region: Option<SpriteRegion>, wraps: bool) -> [f32; 4] {
    let grid = SpriteGrid {
        hframes: 4,
        vframes: 1,
        frame: k,
        sheet: 0,
    };
    let source = CellSource::Atlas {
        size_px: ATLAS,
        region: Some(REGION),
    };
    let input = CellUvInput {
        source,
        region: region.map(|r| (r, (128, 32))),
        grid: Some(grid),
        wraps,
    };
    sprite_cell_uv(input).cell.map(|c| c * ATLAS as f32)
}

/// The extract's own law: an atlas sheet frame `k` lands on ITS texels (cut from the exact rect, not the inset one),
/// and the half-texel clip moves only the final cell, the same for every frame.
#[test]
fn the_extract_cuts_every_atlas_sheet_frame_on_its_own_texels() {
    let whole = SpriteRegion {
        rect: [0.0, 0.0, 128.0, 32.0],
        filter_clip: true,
    };
    for k in 0..4 {
        let start = 37.0 + 32.0 * k as f32;
        let [u0, v0, u1, v1] = frame(k, None, false);
        assert!(
            (u0 - start).abs() < 1e-3 && (u1 - start - 32.0).abs() < 1e-3,
            "frame {k}: {u0}..{u1}, its texels {start}..{}",
            start + 32.0
        );
        assert!((v0 - 5.0).abs() < 1e-3 && (v1 - 37.0).abs() < 1e-3);
        for (why, uv) in [
            ("filter_clip", frame(k, Some(whole), false)),
            ("wraps", frame(k, None, true)),
        ] {
            assert!(
                (uv[0] - start - 0.5).abs() < 1e-3 && (uv[2] - start - 31.5).abs() < 1e-3,
                "{why} frame {k}: {}..{}",
                uv[0],
                uv[2]
            );
        }
    }
}

/// An own texture is sampled whole, and never takes the atlas's wrap inset.
#[test]
fn an_own_texture_is_whole_and_takes_no_atlas_inset() {
    let input = CellUvInput {
        source: CellSource::Own,
        region: None,
        grid: None,
        wraps: true,
    };
    let uv = sprite_cell_uv(input);
    assert_eq!(uv.cell, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(uv.sheet, uv.cell);
}
