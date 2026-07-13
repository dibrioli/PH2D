//! O GESTO do Shape Builder (`impl App`) — módulo irmão de [`crate::shape_build`], que é a
//! regra pura. A separação é a do conector: o que decide fica testável sem `App`, sem
//! câmera e sem mundo; aqui mora só a ponte com o frame.

use crate::App;
use crate::shape_build::BuildSession;

impl App {
    /// Abre (ou renova) a sessão de Shape Builder para a seleção atual. Chamado por frame,
    /// no modo Build.
    ///
    /// **Só reconstrói quando a SELEÇÃO muda.** Reconstruir por frame jogaria fora o memo do
    /// arranjo a cada quadro, e cada hover voltaria a pagar a booleana (o custo frio, ~140 µs
    /// por região) em vez de ler o cache (~20 µs).
    pub(crate) fn build_session_upkeep(&mut self) {
        let Some(gfx) = self.gfx.as_ref() else { return };
        if self.vec_draw_config.mode != ph2d_tool_vector::DrawMode::Build {
            self.vec_build = None;
            return;
        }
        let sel: Vec<u64> = self.vec_pen.selected_paths().to_vec();
        if self
            .vec_build
            .as_ref()
            .is_some_and(|s| s.sources == sel && !s.dragging)
        {
            return; // a seleção é a mesma: o arranjo (e o memo) valem
        }
        if self.vec_build.as_ref().is_some_and(|s| s.dragging) {
            return; // no meio de um arrasto o arranjo é sagrado
        }
        let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
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
    /// Os originais são CONSUMIDOS e o resultado nasce na fatia de z do mais de trás —
    /// exatamente como a booleana normal faz (a forma não salta para o topo).
    pub(crate) fn build_up(&mut self) {
        let Some(session) = self.vec_build.as_mut() else {
            return;
        };
        session.dragging = false;
        let results = session.resolve();
        let sources = session.sources.clone();
        session.marked.clear();
        if results.is_empty() && !sources.is_empty() {
            // Nada pintado: no-op silencioso (o hover sozinho não destrói nada).
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else { return };
        let scene = &mut gfx.vec_scene;

        // A fatia de z do operando mais de trás — o resultado ocupa o lugar dele.
        let at = sources
            .iter()
            .filter_map(|id| scene.paths().iter().position(|p| p.id == *id))
            .min()
            .unwrap_or(0);

        let pre = scene.clone();
        for id in &sources {
            scene.remove_path(*id);
        }
        let new_ids: Vec<u64> = results
            .into_iter()
            .enumerate()
            .map(|(k, r)| scene.insert_path(at + k, r))
            .collect();
        self.vec_history.push_undo(pre);
        self.vec_pen.select_many(&new_ids);
        eprintln!("[ph2d-vec] build: ok ({} path[s])", new_ids.len());
        // A sessão morre com o gesto: a seleção mudou, e o `upkeep` do próximo frame
        // reabre o arranjo sobre o que sobrou.
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
