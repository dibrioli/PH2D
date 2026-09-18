//! **A metade que precisa da `App`** do traço do Flip (W2/L5, 2026-09-11).
//!
//! O módulo [`ph2d_app_flip::draw`] guarda a LEI do traço (amostragem, largura, a cor); aqui
//! fica o que toca os agregados da shell — o `AppGfx` (o `FlipDoc` vivo), o relógio e a
//! captura de undo. É a lista que o substrato da `ph2d-app-host` tem de cobrir (Fase B).
//!
//! ⚠️ **O corte foi por TECTO, e o tecto subiu sozinho:** ao reapontar os caminhos da W2 as
//! linhas ficaram mais longas e o `rustfmt` reexpandiu-as — `600` (exactamente no tecto)
//! passou a `605`. A cura de um tecto é corte por responsabilidade, nunca uma entrada nova
//! no `FILE_OVERAGE_OK` (CLAUDE.md §5.0).

#[allow(unused_imports)]
use ph2d_app_flip::draw::*;
use ph2d_core::Vec2;
use ph2d_flip::FlipDrawing;
use ph2d_flip_render::{FlipGpuData, pack_drawing};
use ph2d_tool_flip::FlipMode;
use ph2d_vec_scene::Xform;

impl crate::App {
    /// A tool Flip quer capturar o canvas AGORA? (ativa + modo Draw). Lê o cache
    /// publicado pelo `flip_bridge` — sem downcast (o `input_dispatch` é livre).
    #[must_use]
    pub(crate) fn flip_wants_canvas(&self) -> bool {
        self.flip_state.active
            && matches!(self.flip_state.style.map(|s| s.mode), Some(FlipMode::Draw))
    }

    /// O afim MUNDO→LOCAL do objeto Flip ativo (o 1º). ADR-0111: o gizmo pode ter
    /// movido o objeto (geometria LOCAL + `Transform`); a mão desenha/apaga em
    /// MUNDO, então converte-se na fronteira. Identidade se o objeto nunca foi
    /// movido, sumiu, ou colapsou — caminho comum (desenho normal), no-op.
    #[must_use]
    pub(crate) fn flip_active_world_to_local(&self) -> Xform {
        // **A POSE DA CHAVE ativa entra no funil** (W7.2). A cadeia da arte é
        // `objeto ∘ pose_da_chave`, então o inverso dela é o que leva o cursor ao espaço
        // do DESENHO — onde a geometria vive. Sem isto, desenhar/esculpir/preencher numa
        // chave deslocada erraria pelo tanto do deslocamento: o usuário aponta para o que
        // VÊ, e o que ele vê já está posado.
        //
        // A pose sai do MESMO amostrador que o render usa (`offset_at_cycled`) — seed e
        // sample são a mesma função (`feedback_derived_coordinate_seed_must_match_sample`).
        ph2d_flip_entities::transform::world_to_art(
            &self.flip_active_object_xform(),
            self.flip_active_pose(),
        )
    }

    /// O afim LOCAL(objeto)→MUNDO do objeto Flip ativo — **sem a pose da chave**. É a
    /// cadeia do `Transform` do ECS (o gizmo), e nada mais.
    #[must_use]
    fn flip_active_object_xform(&self) -> Xform {
        let Some(gfx) = self.gfx.as_ref() else {
            return Xform::IDENTITY;
        };
        let Some(oid) = gfx.flip.objects().first().map(|o| o.id) else {
            return Xform::IDENTITY;
        };
        self.flip_state
            .entities
            .get(&oid)
            .map(|&bits| ph2d_ecs::Entity::from_bits(bits))
            .filter(|e| gfx.sim.world().get_entity(*e).is_ok())
            .map_or(Xform::IDENTITY, |e| {
                ph2d_flip_entities::transform::object_xform(&gfx.sim, e)
            })
    }

    /// A pose (afim) da chave que está NA TELA agora — a MESMA que o render dobra
    /// (`pose_at_cycled`, W7.2). Delega à função livre `flip_transform::active_pose`
    /// (o overlay dos helpers do Gap Closure chama a MESMA, com os campos soltos —
    /// uma 2ª derivação desenharia o helper fora do desenho posado).
    #[must_use]
    fn flip_active_pose(&self) -> ph2d_flip::Pose {
        let Some(gfx) = self.gfx.as_ref() else {
            return ph2d_flip::Pose::IDENTITY;
        };
        ph2d_flip_entities::transform::active_pose(
            &gfx.flip,
            self.flip_state.active_layer,
            &self.playhead,
        )
    }

    /// O afim MUNDO→LOCAL(objeto) **sem a pose da chave** — o funil do gesto de MOVER.
    ///
    /// Por que este não pode ter a pose: mover uma instância ESCREVE a pose, e a pose
    /// entra no [`Self::flip_active_world_to_local`]. Usar aquele funil aqui criaria um
    /// laço — cada amostra converte o cursor num referencial que a amostra anterior
    /// acabou de mover — e o desenho **treme** (smoke do Enio, 2026-07-14). O delta é um
    /// VETOR (uma diferença de dois pontos); a translação da pose se cancela nele, então
    /// tirar a pose não muda o resultado no caso comum e **elimina o laço** no instanciado.
    #[must_use]
    pub(crate) fn flip_active_world_to_object(&self) -> Xform {
        self.flip_active_object_xform()
            .inverse()
            .unwrap_or(Xform::IDENTITY)
    }

    /// Pen-down do desenho Flip: começa um traço na coord de mundo. Devolve
    /// `true` se consumiu (a tool está desenhando) — o caller não deixa cair no
    /// gizmo/pick.
    pub(crate) fn flip_canvas_down(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_wants_canvas() {
            return false;
        }
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let win = gfx.scene_window();
        let w = gfx.camera.screen_to_world((x, y), win);
        self.flip_state.draw.begin(Vec2::new(w[0], w[1]), 1.0);
        true
    }

    /// Move enquanto desenha: adiciona uma amostra (override a <2px). Devolve
    /// `true` se um traço está em curso (consome o move).
    pub(crate) fn flip_canvas_move(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_state.draw.is_active() {
            return false;
        }
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let win = gfx.scene_window();
        let w = gfx.camera.screen_to_world((x, y), win);
        let px_per_world = win.height.max(1) as f32 / gfx.camera.height_world.max(f32::EPSILON);
        self.flip_state
            .draw
            .extend(Vec2::new(w[0], w[1]), 1.0, px_per_world);
        true
    }

    /// GPU-data do traço em curso pro **preview ao vivo** (renderizado por cima do
    /// composite a cada frame; vira documento só no pen-up). `None` quando não há
    /// gesto ou < 2 amostras.
    #[must_use]
    pub(crate) fn flip_preview_data(&mut self) -> Option<FlipGpuData> {
        if !self.flip_state.draw.is_active() {
            return None;
        }
        let style = self.flip_state.style?;
        // O preview é dobrado na fatia da camada ativa (espaço LOCAL do objeto); as
        // amostras são MUNDO → converte, senão o preview folga do traço final. A
        // largura é px de tela ABSOLUTO (o render não escala pelo zoom) → sem câmera.
        let w2l = self.flip_active_world_to_local();
        let (pts, prs, fit) = self.flip_state.draw.preview_parts();
        if pts.len() < 2 {
            return None;
        }
        // **A MESMA porta do bake** — não "o mesmo smoothing", a mesma FUNÇÃO.
        //
        // Antes o preview repetia só o `active_smooth` e o bake acrescentava um RDP; os
        // dois só coincidiam porque esse RDP estava calibrado para não fazer nada
        // (tolerância de 0,05 px). Ou seja: o invariante *"o preview mostra o traço
        // final"* era mantido **castrando** um dos lados, e qualquer simplificação de
        // verdade o quebrava em silêncio — foi exatamente o que o Enio reportou em
        // 2026-07-11 (*"o desenho em tempo real está mais suave que o traço cosido"*).
        // Compartilhando a função, ele passa a valer por CONSTRUÇÃO.
        let mut d = FlipDrawing::default();
        d.strokes
            .push(stroke_from_samples_cached(&style, pts, prs, &w2l, fit));
        Some(pack_drawing(&d))
    }

    /// Pen-up: assa o traço acumulado no `FlipDoc`. Devolve `true` se um gesto
    /// estava em curso (consome o Up, mesmo que um toque simples não vire traço).
    pub(crate) fn flip_canvas_up(&mut self) -> bool {
        if !self.flip_state.draw.is_active() {
            return false;
        }
        let Some((points, pressures)) = self.flip_state.draw.take() else {
            return true; // toque simples (<2 pontos): consumido, sem traço
        };
        let style = self.flip_state.style;
        let active_layer = self.flip_state.active_layer;
        // Fronteira MUNDO→LOCAL (ADR-0111): num objeto já movido pelo gizmo o traço
        // é guardado no espaço local dele. Identidade num objeto novo (o comum).
        let w2l = self.flip_active_world_to_local();
        let playhead = self.playhead;
        let strip_ref = &mut self.flip_state.strip;
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(style) = style
        {
            // O traço é assado e ACABOU. (Ele já foi o "alvo vivo" — os controles do
            // painel continuavam reescrevendo o último traço até o usuário fazer outra
            // coisa. O Enio mandou parar com isso em 2026-07-18: um traço desenhado é um
            // FATO, não uma pré-visualização que os sliders continuam editando.)
            ph2d_app_flip::bake::bake_stroke(
                &mut gfx.flip,
                &playhead,
                &style,
                active_layer,
                strip_ref,
                &points,
                &pressures,
                &w2l,
            );
        }
        true
    }
}
