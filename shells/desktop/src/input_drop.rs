//! Drag-and-drop de arquivos no canvas — o import que nasce de um gesto do sistema.
//!
//! Módulo IRMÃO de [`crate::input_handlers`] (que cuida do TECLADO). O corte é por
//! responsabilidade, não por contagem de linhas: soltar arquivos e apertar teclas não
//! compartilham estado nenhum. Ele saiu daqui quando o `input_handlers` cruzou o cap de
//! 600 LOC do HR-18 na árvore integrada — duas linhas paralelas (Motion: teclas do grafo;
//! Áudio: Ctrl+X/C/V) cabiam sozinhas e não cabiam somadas. Split, nunca allowlist.

use crate::App;
use crate::cursor_pos::live_cursor_in_window;
use crate::image_import::{ImportItemResult, import_images_grid};
use ph2d_editor::Toast;

impl App {
    /// Imports each dropped path that resolves to an image, anchoring
    /// the spawn at the cursor's world position (CoreGraphics
    /// override on macOS where winit 0.30 doesn't update `last_cursor`
    /// during drag). Falls back to `self.last_cursor` on other
    /// platforms. Honors `grid_snap_state` so multi-drop forms a
    /// tidy grid.
    pub(crate) fn handle_dropped_files(&mut self, paths: &[std::path::PathBuf]) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return;
        };
        let pixels_per_meter = hero.project.pixels_per_meter;
        let win = gfx.surface.size();
        // macOS-only: winit 0.30 does NOT emit `CursorMoved` during
        // external file drag operations (see
        // winit-0.30.13/src/platform_impl/macos/window_delegate.rs —
        // `draggingEntered:` doesn't extract `draggingLocation`, no
        // `draggingUpdated:` is implemented at all). So `last_cursor`
        // is whatever it was BEFORE the drag started, not where the
        // file was actually dropped. Query the live cursor from
        // CoreGraphics (`macos_cursor_query`) to override; fall back
        // to `last_cursor` on other platforms.
        let cursor_px = self
            .host
            .as_ref()
            .and_then(|h| live_cursor_in_window(h.window()))
            .unwrap_or(self.last_cursor);
        let drop_world_raw = gfx.camera.screen_to_world(cursor_px, win);
        // Grid-snap apply (drag-drop site). When snap is enabled in
        // `grid_snap_state`, align the drop position to the active
        // grid before spawning so a multi-sprite drop forms a tidy
        // grid rather than scattering at sub-pixel offsets. No-op
        // when snap_enabled = false or active kind has no snap target.
        let drop_world: [f32; 2] = if let Some(hero) = gfx.hero_screen.as_mut() {
            // Drag-drop: sprite hasn't been imported yet, so half-size
            // is unknown. Pass [0.0, 0.0] — Corner-family modes
            // degenerate to point-Intersection snap in that case.
            hero.grid.snap_state.snap_world(drop_world_raw, [0.0, 0.0])
        } else {
            drop_world_raw
        };
        // Filter to image files up front (warning toast per skip), then
        // hand the survivors to the batch importer, which lays them out
        // in a near-square grid anchored at the drop point (`drop_world`
        // = first cell's center; the grid grows right + down). This
        // replaces the old per-file `camera.center` shuffle — the
        // importer takes the world anchor directly.
        let mut valid_paths: Vec<std::path::PathBuf> = Vec::new();
        for path in paths {
            if ph2d_asset::is_supported_image_extension(path) {
                valid_paths.push(path.clone());
            } else {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("(unnamed)");
                gfx.toasts
                    .push(Toast::warning(format!("Skipped non-image: {name}")));
                self.title_dirty = true;
            }
        }
        if valid_paths.is_empty() {
            return;
        }
        let results = import_images_grid(
            &mut gfx.sim,
            &mut gfx.renderer,
            &gfx.asset_db,
            drop_world,
            &mut gfx.next_import_cell,
            &valid_paths,
            pixels_per_meter,
            &mut gfx.atlas_asset_map,
        );
        // Seat the selection: the first imported sprite replaces the
        // selection, the rest join it as extras so a multi-file drop
        // ends up fully selected (same shape as Shift-clicking each on
        // the canvas). The per-frame snapshot sync
        // (render_loop/snapshots.rs) derives both the canvas gizmo view
        // and the Hierarchy row highlight from `hero.gizmo`, so this one
        // write covers both surfaces.
        let mut selected_any = false;
        for r in results {
            match r {
                ImportItemResult::Ok { label, bits } => {
                    if let Some(hero) = gfx.hero_screen.as_mut() {
                        if selected_any {
                            hero.gizmo.add_to_selection(bits);
                        } else {
                            hero.gizmo.replace_selection(Some(bits));
                            selected_any = true;
                        }
                    }
                    gfx.toasts.push(Toast::success(format!("Imported {label}")));
                    self.title_dirty = true;
                }
                ImportItemResult::Err { name, error } => {
                    eprintln!("M14.4e drop failed ({name}): {error}");
                    gfx.toasts
                        .push(Toast::error(format!("Drop failed: {error}")));
                    self.title_dirty = true;
                }
            }
        }
    }
}
