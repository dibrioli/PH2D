//! [`Asset`] — the decoded payload behind an [`crate::AssetId`].
//!
//! M6 ships only `ImageRgba8`. Audio, font, vector, and binary blob
//! variants land as their respective milestones (M7+ as needed). The
//! enum is intentionally non-exhaustive so adding variants doesn't
//! break downstream `match`es.
//!
//! `pixels` is wrapped in `Arc<[u8]>` (not `Vec<u8>`) so two
//! consumers — e.g. the renderer's atlas builder + an MCP tool that
//! wants to introspect the data — can share the same allocation.

use crate::prefab::PrefabDoc;
use crate::scene::SceneDoc;
use std::sync::Arc;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Asset {
    ImageRgba8 {
        width: u32,
        height: u32,
        /// Tight-packed RGBA8: `len == width * height * 4`.
        pixels: Arc<[u8]>,
    },
    /// Cooked prefab — postcard-decoded once at insert time.
    /// `Arc<PrefabDoc>` so multiple spawns share one allocation
    /// without cloning the component blobs each time.
    Prefab(Arc<PrefabDoc>),
    /// Cooked scene — same Arc-sharing strategy as Prefab.
    Scene(Arc<SceneDoc>),
}

impl Asset {
    /// Convenience: rough byte cost of the decoded payload.  Used
    /// later for HR-13 budget accounting.
    pub fn byte_size(&self) -> usize {
        match self {
            Self::ImageRgba8 { pixels, .. } => pixels.len(),
            Self::Prefab(p) => {
                p.components
                    .iter()
                    .map(|c| c.data.len() + std::mem::size_of_val(c))
                    .sum::<usize>()
                    + p.children.len() * std::mem::size_of_val(&p.children[0])
                    + std::mem::size_of_val(&**p)
            }
            Self::Scene(s) => {
                s.instances
                    .iter()
                    .map(|i| {
                        i.overrides
                            .iter()
                            .map(|c| c.data.len() + std::mem::size_of_val(c))
                            .sum::<usize>()
                            + std::mem::size_of_val(i)
                    })
                    .sum::<usize>()
                    + s.relations.len() * std::mem::size_of::<crate::scene::ChildOfPair>()
                    + std::mem::size_of_val(&**s)
            }
        }
    }
}
