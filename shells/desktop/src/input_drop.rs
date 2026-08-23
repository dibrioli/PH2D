//! Drag-and-drop de arquivos no canvas — o import que nasce de um gesto do sistema.
//!
//! Módulo IRMÃO de [`crate::input_handlers`] (que cuida do TECLADO). O corte é por
//! responsabilidade, não por contagem de linhas: soltar arquivos e apertar teclas não
//! compartilham estado nenhum. Ele saiu daqui quando o `input_handlers` cruzou o cap de
//! 600 LOC do HR-18 na árvore integrada — duas linhas paralelas (Motion: teclas do grafo;
//! Áudio: Ctrl+X/C/V) cabiam sozinhas e não cabiam somadas. Split, nunca allowlist.

use crate::App;
use crate::cursor_pos::live_cursor_in_window;
use crate::image_import::ImportItemResult;
use ph2d_editor::Toast;

impl App {
    /// Imports each dropped path that resolves to an image, anchoring
    /// the spawn at the cursor's world position (CoreGraphics
    /// override on macOS where winit 0.30 doesn't update `last_cursor`
    /// during drag). Falls back to `self.last_cursor` on other
    /// platforms. Honors `grid_snap_state` so multi-drop forms a
    /// tidy grid.
    pub(crate) fn handle_dropped_files(&mut self, paths: &[std::path::PathBuf]) {
        // ⚠️ **As malhas saem da fila ANTES do filtro de imagem**, e não é ordem
        // arbitrária: o roteador abaixo emite um toast *"Skipped …"* por
        // arquivo que não reconhece, então sem este desvio soltar um `.obj`
        // produziria um aviso de que ele foi ignorado — a resposta errada, com
        // a certeza da resposta certa.
        #[cfg(feature = "sculpt3d")]
        let paths: Vec<std::path::PathBuf> = {
            let (mesh, rest): (Vec<_>, Vec<_>) = paths
                .iter()
                .cloned()
                .partition(|p| crate::sculpt3d::is_mesh_file(p));
            self.sculpt3d_import_files(&mesh);
            rest
        };
        #[cfg(feature = "sculpt3d")]
        let paths: &[std::path::PathBuf] = &paths;
        if paths.is_empty() {
            return;
        }
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
        // **AS FOLHAS hand-packed saem da fila antes do filtro de imagem**, pela MESMA razão que
        // as malhas 3D acima: o roteador emite *"Skipped …"* por arquivo que não reconhece,
        // e um `.json` de folha largado com a folha produziria um aviso de que foi ignorado.
        //
        // ⚠️ E a imagem que a folha referencia é retirada da leva também — senão largar
        // `folha.png` + `folha.json` daria a folha **e** um sprite avulso com a folha inteira
        // desenhada nele, os dois no mesmo sítio, um por cima do outro.
        let (sheet_metas, rest) = crate::sheet_import::partition_sheet_metadata(paths);
        let mut sheet_bits: Vec<u64> = Vec::new();
        let paths: Vec<std::path::PathBuf> = if sheet_metas.is_empty() {
            rest
        } else {
            let consumed: Vec<std::path::PathBuf> = sheet_metas
                .iter()
                .filter_map(|j| crate::sheet_import::referenced_image(j))
                .collect();
            for json in &sheet_metas {
                match crate::sheet_import::import_sheet(
                    &mut gfx.sim,
                    &mut gfx.renderer,
                    &gfx.asset_db,
                    &mut gfx.sheets,
                    &mut gfx.sheet_textures,
                    &mut gfx.next_sheet_id,
                    json,
                    drop_world,
                    pixels_per_meter,
                ) {
                    crate::sheet_import::SheetImportResult::Ok {
                        name,
                        regions,
                        bits,
                    } => {
                        gfx.toasts
                            .push(Toast::success(format!("Sheet {name}: {regions} sprites")));
                        sheet_bits.extend(bits);
                    }
                    crate::sheet_import::SheetImportResult::Err { name, error } => {
                        gfx.toasts
                            .push(Toast::error(format!("Sheet {name}: {error}")));
                    }
                }
                self.title_dirty = true;
            }
            rest.into_iter()
                .filter(|p| !consumed.iter().any(|c| c == p))
                .collect()
        };
        let paths: &[std::path::PathBuf] = &paths;
        // ⚠️ **UMA porta para o que este app importa** (`crate::import_router`, Enio 2026-08-23:
        // *«.ase não aparece no dialog de import»*). Este sítio filtrava por
        // `is_supported_image_extension` e o botão «Import…» oferecia uma lista escrita à mão —
        // duas respostas à mesma pergunta, e a que o artista via era a que envelhecia. Hoje as
        // duas portas chamam esta função, e a única diferença entre elas é de onde vêm os
        // caminhos.
        let batch = crate::import_router::import_paths_grid(
            &mut gfx.sim,
            &mut gfx.renderer,
            &gfx.asset_db,
            drop_world,
            &mut gfx.next_import_cell,
            paths,
            pixels_per_meter,
            &mut gfx.atlas_asset_map,
        );
        for name in &batch.skipped {
            // ⚠️ A mensagem diz o que o app SABE ler. Ela dizia «Skipped non-image», e isso
            // virou mentira no dia em que um `.ase` — que não é uma imagem — passou a entrar.
            gfx.toasts.push(Toast::warning(format!(
                "Skipped {name}: not an image or an Aseprite file"
            )));
            self.title_dirty = true;
        }
        // ⚠️ A seleção nasce com os sprites da FOLHA (quando houve uma), e é por isso que ela é
        // semeada ANTES: largar só `folha.png` + `folha.json` não produz item nenhum aqui, e sem
        // isto devolveria N sprites novos sem nenhum selecionado, ao contrário de todo outro
        // import.
        let mut selected_any = false;
        let seat = |gfx: &mut crate::AppGfx, bits: u64, selected_any: &mut bool| {
            if let Some(hero) = gfx.hero_screen.as_mut() {
                if *selected_any {
                    hero.gizmo.add_to_selection(bits);
                } else {
                    hero.gizmo.replace_selection(Some(bits));
                    *selected_any = true;
                }
            }
        };
        for bits in std::mem::take(&mut sheet_bits) {
            seat(gfx, bits, &mut selected_any);
        }
        // Seat the selection: the first imported sprite replaces the selection, the rest join it
        // as extras so a multi-file drop ends up fully selected (same shape as Shift-clicking each
        // on the canvas). The per-frame snapshot sync (render_loop/snapshots.rs) derives both the
        // canvas gizmo view and the Hierarchy row highlight from `hero.gizmo`.
        for r in batch.items {
            match r {
                ImportItemResult::Ok { label, bits } => {
                    seat(gfx, bits, &mut selected_any);
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
        // ⚠️ **As notas do `.ase` são o ÚLTIMO a falar** — elas dizem o que ficou por trás, e uma
        // linha dessas escondida entre dez «Imported» não é lida.
        for note in batch.notes {
            gfx.toasts.push(Toast::warning(note));
        }
    }
}
