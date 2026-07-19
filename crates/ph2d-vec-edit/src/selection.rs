//! Seleção do `PenTool` (paths + vértices) e as operações de documento que agem
//! sobre ela: nudge, box-select, retype/delete de vértice. Extraído de `lib.rs`
//! (teto de 700 LOC de produção); é um `impl PenTool` inerente, então a API
//! pública fica idêntica.
//!
//! Índices de vértice são **planos** entre contornos (ver `ph2d_vec_scene`
//! `compound`): um buraco de compound path seleciona e edita como qualquer outro.

use crate::{INSERT_SAMPLES, Part, PenTool};
use ph2d_vec_scene::{VecPathId, VecScene, VecVertex, VertexKind};

impl PenTool {
    /// **Shift+clique num vértice: alterna-o na seleção de pontos** (a multi-seleção do modo Node —
    /// Enio 2026-07-15). Hit-testa a ÂNCORA sob `p` (raio `hit_r` em MUNDO, como o resto do pen) e a
    /// acrescenta/remove de [`Self::selected_verts`], **mantendo o resto** — é o que permite retipar
    /// vários pontos de uma vez ([`Self::set_selected_vertex_kind`] já age sobre todos).
    ///
    /// Devolve `true` se havia uma âncora ali (o chamador então NÃO trata o clique como seleção de
    /// objeto nem abre marquee). Só âncora: um HANDLE não entra na seleção de pontos — ele pertence
    /// a uma âncora, e retipar/apagar agem sobre âncoras.
    ///
    /// Um vértice de OUTRO path TROCA o alvo (a seleção de pontos é de um path só, como o resto do
    /// pen), começando a multi-seleção nele.
    pub fn toggle_vert_at(&mut self, scene: &VecScene, p: [f64; 2], hit_r: f64) -> bool {
        let Some(g) = self.hit_test(scene, p, hit_r) else {
            return false;
        };
        if g.part != Part::Anchor {
            return false;
        }
        if self.selected != Some(g.path) {
            self.selected = Some(g.path);
            self.selected_paths = vec![g.path];
            self.selected_verts = vec![g.vert];
            return true;
        }
        if let Some(i) = self.selected_verts.iter().position(|&v| v == g.vert) {
            self.selected_verts.remove(i);
        } else {
            self.selected_verts.push(g.vert);
        }
        true
    }

    /// Path selecionado (o shell mostra seus gizmos de handle).
    pub fn selected(&self) -> Option<VecPathId> {
        self.selected
    }

    /// A seleção de objeto multi-path (para overlay + Align/Distribute + move em
    /// grupo). Sempre inclui o primário; vazia quando nada está selecionado.
    pub fn selected_paths(&self) -> &[VecPathId] {
        &self.selected_paths
    }

    /// Shift+clique: alterna `id` na seleção de objeto. Ao adicionar, vira o primário;
    /// ao remover, o primário passa a ser o último remanescente (ou `None`). Limpa a
    /// seleção de vértice (a edição de ponto é do primário de clique simples).
    pub fn toggle_path(&mut self, id: VecPathId) {
        self.selected_verts.clear();
        if let Some(pos) = self.selected_paths.iter().position(|&p| p == id) {
            self.selected_paths.remove(pos);
            self.selected = self.selected_paths.last().copied();
        } else {
            self.selected_paths.push(id);
            self.selected = Some(id);
        }
    }

    /// Hit-test de OBJETO: o path cujo âncora/handle OU contorno está a `hit_r` de
    /// `p` (o mais próximo). Para o Shift+clique de multi-seleção. `None` = vazio.
    pub fn path_at(&self, scene: &VecScene, p: [f64; 2], hit_r: f64) -> Option<VecPathId> {
        if let Some(g) = self.hit_test(scene, p, hit_r) {
            return Some(g.path);
        }
        let mut best: Option<(VecPathId, f64)> = None;
        for path in scene.paths() {
            if !self.view.is_pickable(path.id) {
                continue;
            }
            // A curva é local; o raio de captura é world (ADR-0111). `best` compara
            // paths de escalas diferentes, então guarda a distância JÁ no mundo.
            let pl = self.to_local(path.id, p);
            let scale = self.xf(path.id).mean_scale();
            if let Some((_, _, d2)) =
                ph2d_vec_scene::nearest_point_on_path(path, pl, INSERT_SAMPLES)
                && let d2_world = d2 * scale * scale
                && d2_world.sqrt() <= hit_r
                && best.is_none_or(|(_, b)| d2_world < b)
            {
                best = Some((path.id, d2_world));
            }
        }
        if let Some((id, _)) = best {
            return Some(id);
        }
        // Nenhum traço perto: o clique pode ter caído no PREENCHIMENTO. Vale a forma
        // mais ao topo que contenha o ponto — é como se seleciona uma forma cheia em
        // qualquer editor, e sem isto a seta branca só pegaria a borda.
        // `paths` é fundo → topo, então varre-se ao contrário.
        for path in scene.paths().iter().rev() {
            if !self.view.is_pickable(path.id) {
                continue;
            }
            if scene.path_contains_point(path.id, self.to_local(path.id, p)) {
                return Some(path.id);
            }
        }
        None
    }

    /// Troca a seleção de OBJETO por `ids`, **preservando** o vértice primário e a
    /// seleção de vértice. É como a shell expande um clique para o grupo inteiro
    /// (a árvore é a Hierarquia, ADR-0110) sem estragar a edição de ponto.
    pub fn set_object_selection(&mut self, ids: &[VecPathId]) {
        if ids.is_empty() {
            return;
        }
        self.selected_paths = ids.to_vec();
        if self.selected.is_none_or(|s| !ids.contains(&s)) {
            self.selected = ids.last().copied();
            self.selected_verts.clear();
        }
    }

    /// Alterna `ids` (um objeto, ou as folhas de um grupo) na seleção de OBJETO
    /// (Shift+clique): entra e sai inteiro. Limpa a seleção de vértice.
    pub fn toggle_object_members(&mut self, ids: &[VecPathId]) {
        if ids.is_empty() {
            return;
        }
        self.selected_verts.clear();
        if ids.iter().all(|m| self.selected_paths.contains(m)) {
            self.selected_paths.retain(|p| !ids.contains(p));
        } else {
            for m in ids {
                if !self.selected_paths.contains(m) {
                    self.selected_paths.push(*m);
                }
            }
        }
        self.selected = self.selected_paths.last().copied();
    }

    /// Vértice "primário" (último tocado) — o do destaque do painel; `None` se
    /// nada selecionado.
    pub fn selected_vert(&self) -> Option<usize> {
        self.selected_verts.last().copied()
    }

    /// Todos os vértices selecionados (para o overlay destacá-los).
    pub fn selected_verts(&self) -> &[usize] {
        &self.selected_verts
    }

    /// Define a seleção de PATH (ex.: selecionar o resultado de uma booleana).
    /// Reduz a seleção de objeto a `[id]` (ou vazia) e limpa a seleção de vértice.
    pub fn select(&mut self, id: Option<VecPathId>) {
        self.selected = id;
        self.selected_paths = id.map(|i| vec![i]).unwrap_or_default();
        self.selected_verts.clear();
    }

    /// Seleciona um CONJUNTO de paths de uma vez (uma booleana pode devolver
    /// várias regiões disjuntas). O primário vira o último — como no clique a
    /// clique. Limpa a seleção de vértice.
    pub fn select_many(&mut self, ids: &[VecPathId]) {
        self.selected = ids.last().copied();
        self.selected_paths = ids.to_vec();
        self.selected_verts.clear();
    }

    /// Nudge por teclado: desloca a seleção por `(dx, dy)` world-units. Se há
    /// vértices selecionados, translada só eles (âncora + handles); senão, o path
    /// inteiro. Devolve `true` se moveu algo (nada selecionado ⇒ `false`).
    pub fn nudge(&mut self, scene: &mut VecScene, dx: f64, dy: f64) -> bool {
        // Multi-path OBJECT selection (no specific vertices) → move every selected
        // path wholesale (Align/Distribute companion).
        // `(dx, dy)` é um delta de MUNDO (uma seta do teclado move o mesmo tanto na
        // tela, esteja o path onde estiver); a geometria é local (ADR-0111).
        if self.selected_verts.is_empty() && self.selected_paths.len() > 1 {
            let mut moved = false;
            for &id in &self.selected_paths {
                let d = self.delta_to_local(id, [dx, dy]);
                moved |= scene.translate_path(id, d[0], d[1]);
            }
            return moved;
        }
        let Some(sel) = self.selected else {
            return false;
        };
        let [dx, dy] = self.delta_to_local(sel, [dx, dy]);
        let Some(path) = scene.path_mut(sel) else {
            return false;
        };
        let shift = |v: &mut VecVertex| {
            v.anchor = [v.anchor[0] + dx, v.anchor[1] + dy];
            v.in_handle = [v.in_handle[0] + dx, v.in_handle[1] + dy];
            v.out_handle = [v.out_handle[0] + dx, v.out_handle[1] + dy];
        };
        if self.selected_verts.is_empty() {
            path.for_each_vert_mut(shift);
        } else {
            for &i in &self.selected_verts {
                if let Some(v) = path.vert_mut(i) {
                    shift(v);
                }
            }
        }
        true
    }

    /// Box-select: seleciona as âncoras do path (selecionado; senão o que tiver
    /// mais âncoras na caixa) dentro do retângulo world `[min,max]`. Substitui a
    /// seleção. Só muda estado de seleção — não muta a cena, não gera undo.
    pub fn box_select(&mut self, scene: &VecScene, min: [f64; 2], max: [f64; 2]) {
        let (x0, x1) = (min[0].min(max[0]), min[0].max(max[0]));
        let (y0, y1) = (min[1].min(max[1]), min[1].max(max[1]));
        let inside = |a: [f64; 2]| a[0] >= x0 && a[0] <= x1 && a[1] >= y0 && a[1] <= y1;
        // A caixa é world; as âncoras são locais (ADR-0111). Captura só `xforms`
        // (não `self`) para não travar `self.selected` no borrow-check.
        let xforms = &self.xforms;
        let in_world =
            |id: VecPathId, a: [f64; 2]| inside(ph2d_vec_scene::xform_of(xforms, id).apply(a));
        let target = self.selected.or_else(|| {
            scene
                .paths()
                .iter()
                .map(|p| {
                    (
                        p.id,
                        p.verts_all().filter(|v| in_world(p.id, v.anchor)).count(),
                    )
                })
                .filter(|&(_, c)| c > 0)
                .max_by_key(|&(_, c)| c)
                .map(|(id, _)| id)
        });
        let Some(id) = target else {
            self.selected_verts.clear();
            return;
        };
        self.selected = Some(id);
        self.selected_paths = vec![id];
        self.selected_verts = scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .map(|p| {
                p.verts_all()
                    .enumerate()
                    .filter(|(_, v)| in_world(id, v.anchor))
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default();
    }

    /// Tipo do vértice primário (para o painel destacar Corner/Smooth/Symmetric).
    /// `None` se não há vértice selecionado ou o índice não existe mais.
    pub fn selected_vertex_kind(&self, scene: &VecScene) -> Option<VertexKind> {
        let i = self.selected_vert()?;
        let sel = self.selected?;
        let path = scene.paths().iter().find(|p| p.id == sel)?;
        path.vert(i).map(|v| v.kind)
    }

    /// Retipa TODOS os vértices selecionados (botões Corner/Smooth/Symmetric).
    /// Devolve `true` se algo mudou (o shell empurra um passo de undo nesse caso).
    pub fn set_selected_vertex_kind(&mut self, scene: &mut VecScene, kind: VertexKind) -> bool {
        let Some(id) = self.selected else {
            return false;
        };
        let Some(path) = scene.path_mut(id) else {
            return false;
        };
        let mut changed = false;
        for &i in &self.selected_verts {
            changed |= ph2d_vec_scene::retype_vertex(path, i, kind);
        }
        changed
    }

    /// Apaga TODOS os vértices selecionados (Delete / botão), re-costurando os
    /// vizinhos. Um contorno que fique com < 2 vértices é descartado — o buraco de
    /// um compound some; se o path inteiro esvaziar, ele é removido e a seleção
    /// limpa. A seleção segue no vizinho do 1º apagado (delete encadeado de um só).
    /// Devolve `true` se apagou algo.
    pub fn delete_selected_vertex(&mut self, scene: &mut VecScene) -> bool {
        let Some(id) = self.selected else {
            return false;
        };
        if self.selected_verts.is_empty() {
            return false;
        }
        let Some(path) = scene.path_mut(id) else {
            return false;
        };
        let mut idxs = self.selected_verts.clone();
        idxs.sort_unstable();
        idxs.dedup();
        let lowest = idxs.first().copied().unwrap_or(0);
        let single = idxs.len() == 1;
        // Resolve os índices PLANOS em (contorno, local) ANTES de remover: remover
        // encurta um contorno e reescreve o mapa plano dos seguintes.
        let mut located: Vec<(usize, usize)> =
            idxs.iter().filter_map(|&i| path.locate_vert(i)).collect();
        // Do maior pro menor: assim nenhum índice local ainda pendente desliza.
        located.sort_unstable();
        for &(c, local) in located.iter().rev() {
            if let Some((verts, _)) = path.contour_mut(c)
                && local < verts.len()
            {
                verts.remove(local);
            }
        }
        // Descarta contornos degenerados (do último pro primeiro).
        for c in (0..path.contour_count()).rev() {
            if path.contour(c).is_some_and(|(v, _)| v.len() < 2) && !path.remove_contour(c) {
                // `remove_contour` recusa o contorno único ⇒ o path inteiro morre.
                scene.remove_path(id);
                self.selected_verts.clear();
                self.selected = None;
                self.active = None;
                return true;
            }
        }
        let remaining = path.total_verts();
        self.selected_verts.clear();
        if single && remaining > 0 {
            // Delete de um só: seleção segue no vizinho (delete encadeado).
            self.selected_verts = vec![lowest.min(remaining - 1)];
        }
        true
    }
}
