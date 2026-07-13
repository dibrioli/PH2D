//! The breadcrumb (Motion Nodes doc 57) — *where am I, and how do I get out*.
//!
//! Blender puts it in the same corner (*"you can refer to the breadcrumbs in the top
//! left corner of the node editor to see where you are in the hierarchy"*) and
//! Houdini's path gadget is clickable the same way (*"go up a level: click the level
//! in the path above the network view"*). So is this: every crumb is a button.
//!
//! **It appears only when there is somewhere to go back to.** At the root there is no
//! hierarchy to show, and a permanent "Root" chip in the corner of every graph would
//! be a label that never says anything. Drawn OUTSIDE the canvas clip (it is chrome,
//! like the toolbar) and hit-registered on the `Chrome` ordinal channel, so it needs
//! no interaction vocabulary of its own.

use crate::geom;
use crate::paint::fnv_id;
use crate::paint_chrome::CHROME_CRUMB_BASE;
use crate::snapshot::GraphViewSnapshot;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::GraphHitKind;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_title, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

const CRUMB_RADIUS: f32 = 5.0; // LITERAL-PX-OK: crumb chip corner radius
const CRUMB_TEXT_PAD_Y: f32 = 3.0; // LITERAL-PX-OK: crumb label top inset
/// The separator drawn in the gap between crumbs. **Not an arrow**: `->` and `>` are
/// the same claim and one of them is a tofu box in this font stack (gate:
/// `no_tofu_glyphs`).
const SEPARATOR: &str = "/";

/// Draw the path from the root to the level being viewed, and register one hit rect
/// per crumb. A no-op at the root.
pub(crate) fn draw(
    ctx: &mut PaintCtx,
    rect: Rect,
    theme: Theme,
    snap: &GraphViewSnapshot,
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
) {
    if snap.level.is_none() || snap.breadcrumb.len() < 2 {
        return;
    }
    let titles: Vec<String> = snap.breadcrumb.iter().map(|c| c.title.clone()).collect();
    for (i, title) in titles.iter().enumerate() {
        let chip = geom::crumb_rect(rect, &titles, i);
        // The LAST crumb is where you are — it is not a place to go, so it reads as a
        // label (no border) while the ones above it read as buttons.
        let here = i + 1 == titles.len();
        fill_rounded_rect(
            ctx.scene,
            chip,
            CRUMB_RADIUS,
            resolve(
                if here {
                    ColorToken::Bg3
                } else {
                    ColorToken::Bg2
                },
                theme,
            ),
        );
        if !here {
            stroke_rounded_rect(
                ctx.scene,
                chip,
                CRUMB_RADIUS,
                1.0,
                resolve(ColorToken::Border, theme),
            );
        }
        paint_text_title(
            ctx.text_system,
            ctx.scene,
            title,
            chip.x + geom::CRUMB_PAD_X,
            chip.y + CRUMB_TEXT_PAD_Y,
            geom::CRUMB_TEXT_SIZE,
            chip.w - geom::CRUMB_PAD_X,
            resolve(
                if here {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                },
                theme,
            ),
        );
        if !here {
            // The separator lives in the gap AFTER this crumb.
            paint_text_title(
                ctx.text_system,
                ctx.scene,
                SEPARATOR,
                chip.x + chip.w,
                chip.y + CRUMB_TEXT_PAD_Y,
                geom::CRUMB_TEXT_SIZE,
                geom::CRUMB_GAP,
                resolve(ColorToken::Text2, theme),
            );
        }
        // Every crumb is registered, including the one you are standing on: clicking
        // it is a no-op the shell absorbs (`set_level` to the level you are already
        // at changes nothing), and a chip that is drawn but not clickable is the one
        // the artist will click first.
        hits.push((
            fnv_id(&format!("motion_graph/crumb/{i}")),
            GraphHitKind::Chrome {
                id: CHROME_CRUMB_BASE + i as u16,
            },
            chip,
        ));
    }
}
