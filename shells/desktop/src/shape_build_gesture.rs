//! O GESTO do Shape Builder (`impl App`) — módulo irmão de [`crate::shape_build`], que é a
//! regra pura. A separação é a do conector: o que decide fica testável sem `App`, sem
//! câmera e sem mundo; aqui mora só a ponte com o frame.

use crate::App;
use crate::shape_build::{BuildSession, source_key};
use ph2d_vec_scene::VecPathId;

impl App {
    /// Abre (ou renova) a sessão de Shape Builder para a seleção atual. Chamado por frame,
    /// no modo Build.
    ///
    /// **Reconstrói quando a seleção, a GEOMETRIA ou a POSE mudam** — e não só quando a
    /// seleção muda. O arranjo é assado em MUNDO: se a forma se move (ou volta de um undo)
    /// e o arranjo não é refeito, o véu segue descrevendo a forma onde ela *estava*. É a
    /// mesma família do [[feedback_derived_coordinate_seed_must_match_sample]].
    ///
    /// Fora disso ele é preservado, e isso importa: refazê-lo por frame jogaria fora o memo
    /// do arranjo e cada hover voltaria a pagar a booleana (~140 µs por região, contra ~20 µs
    /// de cache).
    pub(crate) fn build_session_upkeep(&mut self) {
        let Some(gfx) = self.gfx.as_ref() else { return };
        if self.vec_draw_config.mode != ph2d_tool_vector::DrawMode::Build {
            self.vec_build = None;
            return;
        }
        if self.vec_build.as_ref().is_some_and(|s| s.dragging) {
            return; // no meio de um arrasto o arranjo é sagrado
        }
        // **Ordem de z (fundo → topo), não a ordem de clique.** O `selected_paths` guarda a
        // ordem em que o artista clicou; o arranjo promete z, e é o z que decide de quem a
        // forma nova herda o estilo (a do topo) e onde ela nasce na pilha.
        let mut sel: Vec<VecPathId> = self.vec_pen.selected_paths().to_vec();
        sel.sort_by_key(|id| {
            gfx.vec_scene
                .paths()
                .iter()
                .position(|p| p.id == *id)
                .unwrap_or(usize::MAX)
        });
        let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
        let key = source_key(&gfx.vec_scene, &xf, &sel);
        if self.vec_build.as_ref().is_some_and(|s| s.opened_for == key) {
            return; // mesma arte, mesma pose: o arranjo (e o memo) valem
        }
        self.vec_build = BuildSession::open(&gfx.vec_scene, &xf, &sel);
    }

    /// Pressão no canvas em modo Build. `alt` = subtrai; `shift` = soma à seleção.
    ///
    /// **Sem arranjo ainda (menos de 2 formas fechadas selecionadas), o clique SELECIONA.**
    /// É a diferença entre um modo utilizável e um modo que parece quebrado: o artista entra
    /// no Build logo depois de desenhar, com uma forma selecionada ou nenhuma, e se o modo
    /// engolisse o clique para não fazer nada ele acharia que a ferramenta está morta. Aqui
    /// ele clica as formas que quer combinar (Shift para somar), o arranjo abre sozinho no
    /// frame seguinte, e o canvas vira a mesa de trabalho.
    pub(crate) fn build_down(&mut self, world: [f64; 2], alt: bool, shift: bool) {
        if self.vec_build.is_none() {
            self.build_select(world, shift);
            return;
        }
        let Some(s) = self.vec_build.as_mut() else {
            return;
        };
        s.subtract = alt; // fixado no press: o mesmo gesto não pode mudar de significado
        s.dragging = true;
        s.touch(world);
    }

    /// O clique de SELEÇÃO do modo Build (enquanto ainda não há 2 formas). Shift soma;
    /// clique no vazio limpa.
    fn build_select(&mut self, world: [f64; 2], shift: bool) {
        let px = self.vec_px_to_world();
        let Some(gfx) = self.gfx.as_ref() else { return };
        let hit = self.vec_pen.path_at(&gfx.vec_scene, world, 10.0 * px);
        match (hit, shift) {
            (Some(id), true) => self.vec_pen.toggle_path(id),
            (Some(id), false) => self.vec_pen.select(Some(id)),
            (None, false) => self.vec_pen.select(None),
            (None, true) => {}
        }
    }

    /// O cursor andou em modo Build. Realça a face sob ele e, se estiver arrastando, pinta.
    /// Devolve `true` se consumiu o movimento.
    pub(crate) fn build_move(&mut self, world: [f64; 2]) -> bool {
        let Some(s) = self.vec_build.as_mut() else {
            return false;
        };
        s.touch(world);
        true
    }

    /// Soltou: as faces pintadas viram forma (ou somem). UM passo de undo.
    ///
    /// **Só as formas TOCADAS são consumidas.** A que o gesto não atravessou continua sendo
    /// o mesmo path — mesmo id, mesma entidade, mesmo `Transform`, mesmos params de Live
    /// Shape. A 1ª versão dissolvia TODAS as fontes numa sobra única e o artista via a arte
    /// dele desaparecer num blob (o smoke do Enio).
    ///
    /// O undo é o GLOBAL, por diff (`App::post_frame_undo`): a `VecScene` está na captura,
    /// e o `held_button` já foi limpo quando o Up chega aqui. Não há passo a empurrar à mão
    /// — o `vec_history` é uma fila morta (ADR-0110+, "populado mas não lido").
    pub(crate) fn build_up(&mut self) {
        let Some(session) = self.vec_build.as_mut() else {
            return;
        };
        session.dragging = false;
        let result = session.resolve();
        session.marked.clear();
        if result.is_empty() {
            return; // nada pintado: no-op silencioso (o hover sozinho não destrói nada)
        }
        let sources: Vec<VecPathId> = session.sources.clone();
        let Some(gfx) = self.gfx.as_mut() else { return };
        // A regra do que morre e do que fica vive em `shape_build::commit` — provável sem
        // `App`, e é o que os gates exercem.
        let sel = crate::shape_build::commit(&mut gfx.vec_scene, &sources, result);
        self.vec_pen.select_many(&sel);
        // A sessão morre com o gesto: a arte mudou, e o `upkeep` do próximo frame reabre o
        // arranjo sobre o que ficou.
        self.vec_build = None;
    }

    /// Esc em modo Build: desmarca tudo, sem tocar na arte.
    pub(crate) fn build_cancel(&mut self) -> bool {
        let Some(s) = self.vec_build.as_mut() else {
            return false;
        };
        if !s.dragging && s.marked.is_empty() {
            return false;
        }
        s.dragging = false;
        s.marked.clear();
        true
    }
}
