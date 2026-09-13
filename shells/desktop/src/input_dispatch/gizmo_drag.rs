//! Gizmo drag advance — per-CursorMoved update of an open gizmo drag.
//!
//! Extracted from `input_dispatch.rs` (HR-18 LOC cap): the MovePivot
//! (TOOL_PIVOT) and the scale/rotate/translate advance paths are large
//! enough that keeping them inline tipped the window-event dispatch hub
//! past 600 LOC. The begin/end of a drag still live in the MouseInput
//! arm; only the per-move advance moved here.
// O tecto de LOC numerado que este ficheiro tinha (~735 LOC em `tests/it/file_loc_caps.rs`) SAIU na
// `line/input-dispatch` (2026-09-13): a divisão por caminho que esta nota adiava foi feita — o
// `advance_gizmo_drag` ficou com o prelúdio e os dois braços, e os corpos vivem em `gizmo_drag_calculo.rs`
// (pivô, `new_t`, factores) e `gizmo_drag_escrita.rs` (grupo, moldura, fluxo, simples). O histórico abaixo
// descreve braços que moram agora no segundo.
// +12 (gold-standard joint anchor): the joint tail — REMOVED again by W-J2; the
// anchor dots open `ph2d_app_physics::joint_anchor_drag`, which writes one side's local.
// +43 (frame resize): the snapshot install + the `else if` arm that resizes a frame's
// box instead of writing its pose. Both DELEGATE (`crate::vec_frame_resize`), so the
// arm is the branch and the ratio, nothing else — the same shape as the flow-reorder
// arm below it.

use crate::App;

/// A escrita do arrasto (grupo, moldura, fluxo, simples) — ramos do `advance_gizmo_drag`.
#[path = "gizmo_drag_escrita.rs"]
mod escrita;

/// O cálculo do arrasto (pivô, `new_t`, factores) — ramos do `advance_gizmo_drag`.
#[path = "gizmo_drag_calculo.rs"]
mod calculo;

impl App {
    /// Advance an in-progress gizmo drag against the latest cursor
    /// position. No-op when no drag is open. Two paths:
    ///
    /// - **MovePivot** (TOOL_PIVOT): relocate the pivot to the cursor
    ///   while the sprite's quad stays world-fixed (writes a
    ///   compensating `Sprite.anchor`); CTRL snaps the pivot to the quad
    ///   center / corners / edge midpoints AND the content-bbox center.
    /// - **Scale / Rotate / Translate**: the pure
    ///   `compute_gizmo_transform` math, with the grid-snap closure on
    ///   the dragged corner for Scale, written back to the entity
    ///   `Transform`.
    ///
    /// Called from `on_cursor_moved` after the pointer is forwarded to
    /// the hero. The next frame's extract + paint mirror the change.
    pub(crate) fn advance_gizmo_drag(&mut self) {
        // Peek the open drag (immutable; released before the mutable
        // pass below). `GizmoDragState` is Copy.
        let open = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.gizmo.drag);
        let Some(drag) = open else {
            // No drag in flight → drop any cached content-bbox center so
            // the next MovePivot drag recomputes it for ITS sprite.
            self.pivot_content_center = None;
            // O mesmo para o instantâneo da moldura: ele descreve um gesto que acabou.
            self.frame_resize_start = None;
            return;
        };
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        // MovePivot + CTRL: compute the content-bbox center ONCE per drag
        // (lazy — first CTRL-held move triggers the readback) and cache
        // it on `self`. Done in its own borrow so the readback doesn't
        // alias the mutable pass below.
        if matches!(drag.kind, ph2d_editor_core::GizmoDragKind::MovePivot)
            && ctrl
            && self.pivot_content_center.is_none()
        {
            self.pivot_content_center = self.compute_pivot_content_center(&drag);
        }
        let content_center = self.pivot_content_center;

        // Scale-snap vetorial: se a forma arrastada é vetorial e o gesto é scale, o
        // canto arrastado encaixa nas OUTRAS formas (bordas/centros/vértices), como no
        // translate. Recolhe os alvos + cfg agora, fora do borrow mutável de gfx.
        let is_scale_drag = matches!(
            drag.kind,
            ph2d_editor_core::GizmoDragKind::ScaleCorner { .. }
                | ph2d_editor_core::GizmoDragKind::ScaleEdge { .. }
        );
        // **A moldura fotografa-se na primeira movida do arrasto** (corolário do W3).
        //
        // A alça de uma moldura muda a CAIXA dela e não a pose, e a razão que o gizmo entrega é
        // ABSOLUTA contra o pen-down — logo a geometria de partida tem de sobreviver ao gesto
        // inteiro. Preguiçoso e no `App`, como o `pivot_content_center` acima e pela mesma razão:
        // um instantâneo de geometria não cabe num `GizmoDragState`, que é `Copy`.
        //
        // ⚠️ A guarda é `is_for`, não `is_none`: soltar e voltar a pegar sem um `CursorMoved` pelo
        // meio não passa pela limpeza lá em cima, e reporia a geometria da moldura ANTERIOR por
        // cima desta.
        if is_scale_drag
            && !self
                .frame_resize_start
                .as_ref()
                .is_some_and(|s| s.is_for(drag.entity_bits))
        {
            self.frame_resize_start = self.begin_frame_resize(drag.entity_bits);
        }
        // ⛔⛔ **AQUI VIVIA O SEGUIMENTO DAS CÓPIAS, e ele DISSOLVEU-SE com o motor** (F4.6c,
        // 2026-09-07). O `vec_instance_follow` existia porque o modelo derivado perdia a translação
        // do mestre (`D(p) = (p − Tm)·I + Ti`): uma alça ancorada pagava a âncora numa translação
        // que a cópia não herdava, e era preciso reproduzi-la à mão sob rotação/escala.
        //
        // ⭐ No mecanismo geral **não há delta**: a cópia é uma sub-árvore REAL, a pose da raiz dela
        // é dela (`ROOT_IS_ITS_OWN`) e a de cada peça chega verbatim. *O substrato apaga a cura* —
        // não havia nada a portar, e a medição de 2026-08-27 já o dizia.
        let vec_scale_ids = if is_scale_drag {
            self.dragged_vec_path_ids(drag.entity_bits)
        } else {
            Vec::new()
        };
        let vec_scale_snap = !vec_scale_ids.is_empty();
        let vec_cfg = self.vec_snap_cfg(self.vec_px_to_world());
        if vec_scale_snap {
            self.vec_rebuild_snap_targets(&vec_scale_ids, &[]);
        }

        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
            && let Some(mut drag) = hero.gizmo.drag
        {
            // Advance the cursor THROUGH the drag, not around it: on a Rotate the
            // step also counts any revolution the cursor just completed, which is
            // the only record of it (`atan2` cannot see past one turn). Skipping
            // this and assigning `cursor_screen` directly is exactly the bug —
            // rotation would silently jump 2π at the branch cut.
            // ⚠️ A janela da CENA, não a da janela (report do Enio, 2026-08-25). Aqui ela
            // é montada a partir do `hero` que já está emprestado — `gfx.surface` é campo
            // DISJUNTO, então lê-se sem conflito; pedir `scene_window_of(gfx)` re-pediria
            // o `gfx` inteiro e colidiria com esse empréstimo.
            let size = ph2d_app_motion::field_gizmo::scene_camera_window(
                hero.view.center_split,
                gfx.surface.size(),
            );
            let cam = ph2d_editor_core::GizmoCamera {
                center: gfx.camera.center,
                height_world: gfx.camera.height_world,
                window_w: size.width as f32,
                window_h: size.height as f32,
            };
            drag.advance_cursor((self.last_pointer.0, self.last_pointer.1), &cam);
            hero.gizmo.drag = Some(drag);
            // **A âncora deste FRAME** — Ctrl/Cmd ancora no centro, sem ele no canto oposto.
            //
            // ⚠️ O `Shift` já era vivo (ele viaja no `GizmoModifiers`, reconstruído a cada
            // movimento logo abaixo) e a âncora não era: ela era decidida no pen-down e congelada.
            // Um gizmo em que um modificador é vivo e o outro não é a incoerência que o artista
            // descobre pelo dedo — e todo app do ramo (Photoshop, Illustrator, Figma) troca a
            // âncora ao vivo. A porta é ÚNICA porque o `frame_resize` abaixo pergunta o mesmo
            // pivô: duas respostas fariam a caixa da moldura ancorar num ponto e o gizmo noutro.
            //
            // ⚠️ **A derivação é LOCAL, e a ordem destas duas linhas é load-bearing.** O que fica
            // no `hero.gizmo.drag` é o estado AUTORADO — o canto que só o pen-down sabe qual é.
            // Guardar o derivado por cima dele apagaria o canto no primeiro frame com a tecla
            // premida, e soltá-la deixaria de devolver coisa nenhuma: o modificador seria vivo
            // só na ida.
            let drag = ph2d_editor_core::live_anchor(drag, ctrl);
            if matches!(drag.kind, ph2d_editor_core::GizmoDragKind::MovePivot) {
                self.ramo_gizmo_mover_pivo(drag, ctrl, content_center);
            } else {
                self.ramo_gizmo_calcular(drag, is_scale_drag, vec_scale_snap, vec_cfg);
            }
            // **As cópias seguem, DEPOIS de o mestre ter sido escrito.**
            //
            // ⚠️ A ordem é load-bearing: o seguimento lê a translação de mundo do mestre AGORA e a
            // compara com o instantâneo do pen-down, então corrê-lo antes das escritas mediria o
            // delta do frame anterior — a cópia ficaria um `CursorMoved` atrás do dedo.
            //
            // ⚠️ E fica FORA dos ramos de propósito: quem decide se há algo a seguir é o
            // instantâneo (que só nasce sob rotação/escala e só quando alguma cópia obedece ao que
            // se arrasta), nunca uma segunda enumeração dos ramos que escrevem pose. Um ramo novo
            // — a moldura foi o último — nasce coberto em vez de esquecido.
        }
        // ⚠️ **No joint-anchor tail here any more (W-J2).** A Translate on a
        // joint entity used to clear `PhysicsJoint::anchored`, because the dot
        // opened a generic Translate and the joint's `Transform` was its anchor.
        // Both halves of that are gone: the dot opens
        // `ph2d_app_physics::joint_anchor_drag` instead, and clearing the sentinel
        // re-derives BOTH locals from the seed policy — so dragging the A dot
        // would have thrown away a B anchor the artist had just placed. A
        // reposition knows its side and writes that local directly.
    }

    /// Compute the world-space center of the selected sprite's CONTENT
    /// bbox (the bounds of its non-transparent pixels) for the current
    /// MovePivot drag, or `None` if the source can't be read or the
    /// sprite is fully transparent.
    ///
    /// Reads the sprite source via the arch-gated `read_sprite_source`
    /// chokepoint (one GPU readback for an Individual texture — done once
    /// per drag, cached by the caller), scans the alpha channel for the
    /// opaque bounds, then maps the bbox center (texture px) through the
    /// quad's local frame to world: `quad_center + R·((u-0.5)·size·scale,
    /// −(v-0.5)·size·scale)`, where `quad_center = drag.pivot_world` and
    /// `R` is `start_transform.rotation`. The Y term is negated because
    /// image rows run top-down while world Y is up.
    fn compute_pivot_content_center(
        &mut self,
        drag: &ph2d_editor_core::GizmoDragState,
    ) -> Option<[f32; 2]> {
        let gfx = self.gfx.as_mut()?;
        let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
        let size = gfx
            .sim
            .world()
            .get::<ph2d_render::Sprite>(entity)
            .map(|s| s.size)?;
        // PRECISION-READONLY: MEDIÇÃO pura — só o canal alfa entra, e só para achar a caixa
        // opaca a que a âncora encaixa. Nenhum pixel volta para a sprite, por isso os 8 bits que
        // o `read_sprite_source` devolve não custam precisão nenhuma a ninguém.
        let src = crate::hero_intents::texture_edit::read_sprite_source(
            entity,
            &gfx.sim,
            &mut gfx.renderer,
            &gfx.asset_db,
            &gfx.atlas_asset_map,
        )?;
        let (w, h) = (src.image.width, src.image.height);
        if w == 0 || h == 0 {
            return None;
        }
        let px = &src.image.pixels;
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (w, h, 0u32, 0u32);
        let mut any = false;
        for y in 0..h {
            for x in 0..w {
                let a = px[((y * w + x) * 4 + 3) as usize];
                if a > 0 {
                    any = true;
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }
        if !any {
            return None;
        }
        // Pixel-center of the opaque bbox → normalized [0,1] → local quad
        // offset (world-scaled) → world (rotate about the quad center).
        let cx = (min_x as f32 + max_x as f32 + 1.0) * 0.5;
        let cy = (min_y as f32 + max_y as f32 + 1.0) * 0.5;
        let u = cx / w as f32 - 0.5;
        let v = cy / h as f32 - 0.5;
        let scale = drag.start_transform.scale;
        let local_x = u * size[0] * scale[0];
        let local_y = -v * size[1] * scale[1];
        // T1.3.5 cross-OS bit-identical.
        let (sin_r, cos_r) = libm::sincosf(drag.start_transform.rotation);
        let qc = drag.pivot_world;
        Some([
            qc[0] + local_x * cos_r - local_y * sin_r,
            qc[1] + local_x * sin_r + local_y * cos_r,
        ])
    }
}
