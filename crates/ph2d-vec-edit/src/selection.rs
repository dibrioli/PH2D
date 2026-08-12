//! Seleção do `PenTool` (paths + vértices) e as operações de documento que agem
//! sobre ela: nudge, box-select, retype/delete de vértice. Extraído de `lib.rs`
//! (teto de 700 LOC de produção); é um `impl PenTool` inerente, então a API
//! pública fica idêntica.
//!
//! Índices de vértice são **planos** entre contornos (ver `ph2d_vec_scene`
//! `compound`): um buraco de compound path seleciona e edita como qualquer outro.

use crate::node_hit::INSERT_SAMPLES;
use crate::{Part, PenTool};
use ph2d_vec_scene::{VecPathId, VecScene, VecVertex, VertexKind};

/// **O que a seleção de vértices tem em comum** (ver [`PenTool::selected_vertex_kind`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SelectedKind {
    /// Todos os vértices selecionados são deste tipo.
    Uniform(VertexKind),
    /// A seleção MISTURA tipos — nenhum chip descreve o todo.
    Mixed,
}

/// Translada o vértice INTEIRO (âncora e os dois handles) por `d`, no espaço LOCAL do caminho a
/// que ele pertence — mover a âncora sem os handles mudaria a curva em vez de a deslocar.
pub(crate) fn shift_vert(v: &mut VecVertex, d: [f64; 2]) {
    v.anchor = [v.anchor[0] + d[0], v.anchor[1] + d[1]];
    v.in_handle = [v.in_handle[0] + d[0], v.in_handle[1] + d[1]];
    v.out_handle = [v.out_handle[0] + d[0], v.out_handle[1] + d[1]];
}

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
    /// ⚠️ **Um vértice de OUTRA forma SOMA — ele não troca o alvo.** Era o oposto (*"a seleção de
    /// pontos é de um path só"*), e não por escolha: sem dono no índice, somar era inexprimível.
    /// Medido antes da troca: Shift+clique num canto de A e depois num de B deixava **1** nó
    /// selecionado, não 2.
    ///
    /// O PRIMÁRIO segue o último tocado — é ele que o painel de estilo edita —, e a forma entra na
    /// seleção de OBJETO em vez de a substituir: quem editou um nó de duas formas está a olhar
    /// para as duas.
    pub fn toggle_vert_at(&mut self, scene: &VecScene, p: [f64; 2], hit_r: f64) -> bool {
        let Some(g) = self.hit_test(scene, p, hit_r) else {
            return false;
        };
        if g.part != Part::Anchor {
            return false;
        }
        if let Some(i) = self
            .selected_verts
            .iter()
            .position(|&v| v == (g.path, g.vert))
        {
            self.selected_verts.remove(i);
        } else {
            self.selected_verts.push((g.path, g.vert));
        }
        // O primário acompanha o último tocado, e a forma entra na seleção de objeto (sem
        // duplicar) — nunca a substitui, senão somar um nó de B DESSELECIONARIA A.
        self.selected = Some(g.path);
        if !self.selected_paths.contains(&g.path) {
            self.selected_paths.push(g.path);
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

    /// Vértice "primário" (último tocado) **com o dono** — o do destaque do painel; `None` se
    /// nada selecionado.
    pub fn selected_vert(&self) -> Option<(VecPathId, usize)> {
        self.selected_verts.last().copied()
    }

    /// Todos os vértices selecionados, **cada um com o seu dono** (para o overlay destacá-los).
    /// A ordem é a de toque: o último é o primário.
    pub fn selected_verts(&self) -> &[(VecPathId, usize)] {
        &self.selected_verts
    }

    /// Os índices selecionados **de UMA forma** — a pergunta que todo consumidor por-caminho faz
    /// (o overlay desenha path a path; uma operação de documento edita um `VecPath` de cada vez).
    ///
    /// Filtrar na porta em vez de em cada chamador é o que impede o próximo consumidor de nascer
    /// comparando só o índice e acender o nó certo da forma errada.
    pub fn verts_in(&self, path: VecPathId) -> impl Iterator<Item = usize> + '_ {
        self.selected_verts
            .iter()
            .filter(move |(p, _)| *p == path)
            .map(|(_, i)| *i)
    }

    /// As FORMAS que a seleção de nós toca, na ordem em que foram tocadas — o escopo de toda
    /// operação que age "sobre a seleção" e precisa editar um `VecPath` por vez.
    pub(crate) fn vert_paths(&self) -> Vec<VecPathId> {
        let mut out: Vec<VecPathId> = Vec::new();
        for &(p, _) in &self.selected_verts {
            if !out.contains(&p) {
                out.push(p);
            }
        }
        out
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
        // ⚠️ **Nós escolhidos: cada um anda no espaço local do SEU dono.** A seta do teclado é um
        // delta de MUNDO (move o mesmo tanto na tela, esteja a forma onde estiver) e a geometria é
        // local (ADR-0111), então a conversão é POR FORMA — duas formas com escalas diferentes
        // andariam distâncias diferentes se um único `delta_to_local` servisse as duas.
        if !self.selected_verts.is_empty() {
            let mut moved = false;
            for id in self.vert_paths() {
                let d = self.delta_to_local(id, [dx, dy]);
                let idxs: Vec<usize> = self.verts_in(id).collect();
                let Some(path) = scene.path_mut(id) else {
                    continue;
                };
                for i in idxs {
                    if let Some(v) = path.vert_mut(i) {
                        shift_vert(v, d);
                    }
                }
                moved = true;
            }
            return moved;
        }
        // Multi-path OBJECT selection (no specific vertices) → move every selected
        // path wholesale (Align/Distribute companion).
        if self.selected_paths.len() > 1 {
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
        let d = self.delta_to_local(sel, [dx, dy]);
        let Some(path) = scene.path_mut(sel) else {
            return false;
        };
        path.for_each_vert_mut(|v| shift_vert(v, d));
        true
    }

    /// Box-select: seleciona as âncoras do path (selecionado; senão o que tiver
    /// mais âncoras na caixa) dentro do retângulo world `[min,max]`. Substitui a
    /// seleção. Só muda estado de seleção — não muta a cena, não gera undo.
    pub fn box_select(&mut self, scene: &VecScene, min: [f64; 2], max: [f64; 2]) {
        self.box_select_with(scene, min, max, false);
    }

    /// O mesmo, **somando** à seleção quando `additive` (o Shift+retângulo de todo app).
    ///
    /// ⚠️ **A caixa apanha os nós de TODAS as formas que ela cobre**, e somar atravessa formas.
    /// Era o oposto — *"somar só vale dentro do MESMO caminho"* —, e não por preguiça: sem dono no
    /// índice, acumular pares de formas diferentes era inexprimível, então o retângulo tinha de
    /// eleger UM caminho e substituir. Medido antes da troca: uma caixa sobre duas formas apanhava
    /// **4 de 8** nós; somar B a A deixava **4**, não 8.
    ///
    /// Com isso morrem as três perguntas que a eleição obrigava a responder — *quem é o alvo?*,
    /// *a caixa apanhou o selecionado?*, *é o mesmo caminho?* — e o corpo passa a dizer só o que o
    /// gesto significa.
    pub fn box_select_with(
        &mut self,
        scene: &VecScene,
        min: [f64; 2],
        max: [f64; 2],
        additive: bool,
    ) {
        let (x0, x1) = (min[0].min(max[0]), min[0].max(max[0]));
        let (y0, y1) = (min[1].min(max[1]), min[1].max(max[1]));
        self.select_verts_where(scene, additive, |a| {
            a[0] >= x0 && a[0] <= x1 && a[1] >= y0 && a[1] <= y1
        });
    }

    /// **O LAÇO**: as âncoras dentro do polígono `poly` (fechado, em MUNDO) — a mesma seleção do
    /// retângulo, com a região desenhada à mão em vez de dois cantos.
    ///
    /// ⚠️ **O laço não é uma segunda seleção, é um segundo PREDICADO.** O corpo (o filtro de
    /// escondido/travado, o modo aditivo, o primário que segue, o `selected_paths`) é o MESMO do
    /// [`Self::box_select_with`], pela porta [`Self::select_verts_where`] — dois corpos divergiriam
    /// no dia em que um deles ganhasse um caso especial, e o artista veria o laço deixar de somar
    /// (ou de respeitar uma forma travada) sem nada na tela dizer porquê. O gate que prova isto
    /// não conhece a implementação: um laço cujo polígono É um retângulo tem de apanhar
    /// **exatamente** o que a caixa apanha.
    ///
    /// Polígono com menos de 3 pontos não delimita área: não apanha nada (e, não sendo aditivo,
    /// limpa a seleção de nós — é um gesto que falhou, como um retângulo de área zero).
    pub fn lasso_select_with(&mut self, scene: &VecScene, poly: &[[f64; 2]], additive: bool) {
        self.select_verts_where(scene, additive, |a| {
            ph2d_vec_scene::point_in_polygon(poly, a)
        });
    }

    /// **O corpo ÚNICO da seleção por região** — a caixa e o laço só trazem o predicado.
    ///
    /// `inside` decide em MUNDO: as âncoras são LOCAIS (ADR-0111) e sobem pelo afim da forma antes
    /// da pergunta, porque é o desenho na tela que o artista está a cercar.
    fn select_verts_where(
        &mut self,
        scene: &VecScene,
        additive: bool,
        inside: impl Fn([f64; 2]) -> bool,
    ) {
        // Captura só `xforms` (não `self`) para não travar `self.selected` no borrow-check.
        let xforms = &self.xforms;
        let in_world =
            |id: VecPathId, a: [f64; 2]| inside(ph2d_vec_scene::xform_of(xforms, id).apply(a));
        // ⚠️ **A região respeita ESCONDIDO e TRAVADO, e a exigência nasceu com a wave do dono.**
        // Antes a caixa elegia um caminho só, e o `is_pickable` faltava sem consequência visível
        // na maioria dos gestos; apanhando TODAS as formas cobertas, uma forma invisível entraria
        // na seleção em silêncio e o Delete seguinte apagaria nós que ninguém vê — exatamente o
        // modo de falha que o comentário antigo usava para justificar a eleição.
        let hits: Vec<(VecPathId, usize)> = scene
            .paths()
            .iter()
            .filter(|p| self.view.is_pickable(p.id))
            .flat_map(|p| {
                p.verts_all()
                    .enumerate()
                    .filter(|(_, v)| in_world(p.id, v.anchor))
                    .map(|(i, _)| (p.id, i))
            })
            .collect();
        if !additive {
            self.selected_verts.clear();
        }
        for h in hits {
            if !self.selected_verts.contains(&h) {
                self.selected_verts.push(h);
            }
        }
        // O objeto acompanha os nós: quem tem nó escolhido está selecionado. Uma região VAZIA não
        // desmancha a seleção de objeto (o gesto falhou; desselecionar seria uma segunda coisa que
        // o artista não pediu), e o PRIMÁRIO só se move se o antigo saiu de cena — senão o painel
        // de estilo saltaria de forma a cada retângulo.
        let touched = self.vert_paths();
        if touched.is_empty() {
            return;
        }
        if self.selected.is_none_or(|s| !touched.contains(&s)) {
            self.selected = touched.last().copied();
        }
        self.selected_paths = touched;
    }

    /// **Todos os nós** do caminho selecionado (o `Ctrl+A` do modo Node). `true` se selecionou
    /// algum — sem caminho selecionado não há o que selecionar, e dizer que sim faria o shell
    /// empurrar um passo de undo por nada.
    /// ⚠️ Percorre **todas as formas selecionadas**, não só a primária — com uma forma só ele é
    /// byte-idêntico ao que sempre foi, e com várias é o que o `Ctrl+A` do Inkscape faz no editor
    /// de nós. Ele deixou de poder mentir no dia em que a seleção passou a guardar donos.
    pub fn select_all_verts(&mut self, scene: &VecScene) -> bool {
        let mut out: Vec<(VecPathId, usize)> = Vec::new();
        for &id in &self.selected_paths {
            let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
                continue;
            };
            out.extend((0..path.total_verts()).map(|i| (id, i)));
        }
        if out.is_empty() {
            return false;
        }
        self.selected_verts = out;
        true
    }

    /// **Todos os nós dos CONTORNOS que a seleção toca** — o *select subpath*. Num compound (forma
    /// com furos) é o que separa "este buraco" de "a forma inteira", e o `Ctrl+A` não distingue.
    pub fn select_subpath_verts(&mut self, scene: &VecScene) -> bool {
        let mut out: Vec<(VecPathId, usize)> = Vec::new();
        for id in self.vert_paths() {
            let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
                continue;
            };
            // Os contornos tocados pela seleção atual NESTA forma. Um `BTreeSet` seria exagero:
            // são unidades.
            let mut cs: Vec<usize> = Vec::new();
            for i in self.verts_in(id) {
                if let Some((c, _)) = path.locate_vert(i)
                    && !cs.contains(&c)
                {
                    cs.push(c);
                }
            }
            for c in cs {
                let Some((verts, _)) = path.contour(c) else {
                    continue;
                };
                out.extend(
                    (0..verts.len())
                        .filter_map(|l| path.flat_vert(c, l))
                        .map(|f| (id, f)),
                );
            }
        }
        if out.is_empty() {
            return false;
        }
        self.selected_verts = out;
        true
    }

    /// **Todos os nós do MESMO TIPO** do primário (o *Select Same* do Inkscape) — o gesto que
    /// transforma "afiar as 12 quinas desta estrela" de doze cliques em dois.
    ///
    /// O tipo vem do vértice PRIMÁRIO (o último tocado); sem seleção não há tipo a igualar.
    pub fn select_verts_of_same_kind(&mut self, scene: &VecScene) -> bool {
        let Some((pid, vi)) = self.selected_vert() else {
            return false;
        };
        let Some(kind) = scene
            .paths()
            .iter()
            .find(|p| p.id == pid)
            .and_then(|p| p.vert(vi))
            .map(|v| v.kind)
        else {
            return false;
        };
        // O TIPO vem do primário; a VARREDURA cobre as formas que a seleção toca. Com uma forma
        // só é o que sempre foi; com duas, "afiar todas as quinas destas duas estrelas" também
        // deixa de ser doze cliques.
        let mut out: Vec<(VecPathId, usize)> = Vec::new();
        for id in self.vert_paths() {
            let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
                continue;
            };
            out.extend(
                path.verts_all()
                    .enumerate()
                    .filter(|(_, v)| v.kind == kind)
                    .map(|(i, _)| (id, i)),
            );
        }
        if out.is_empty() {
            return false;
        }
        self.selected_verts = out;
        true
    }

    /// **O nó SEGUINTE (ou anterior)** do caminho selecionado — o `Tab` do Inkscape.
    ///
    /// Substitui a seleção por UM nó: percorrer é olhar um de cada vez, e somar ao andar tornaria
    /// o Tab um segundo "select all" lento. Sem seleção nenhuma começa no primeiro (ou no último,
    /// andando para trás) — é o que faz o gesto ter uma porta de entrada.
    /// ⚠️ **Percorre a forma do nó PRIMÁRIO** (sem nó nenhum, a selecionada). Atravessar formas em
    /// silêncio ao chegar ao último nó seria o Tab a mudar o assunto sem o artista pedir — e com
    /// uma forma só ele é byte-idêntico ao que sempre foi.
    pub fn step_vert_selection(&mut self, scene: &VecScene, forward: bool) -> bool {
        let cur = self.selected_vert();
        let Some(id) = cur.map(|(p, _)| p).or(self.selected) else {
            return false;
        };
        let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
            return false;
        };
        let n = path.total_verts();
        if n == 0 {
            return false;
        }
        let next = match cur.map(|(_, i)| i) {
            Some(i) if forward => (i + 1) % n,
            Some(i) => (i + n - 1) % n,
            None if forward => 0,
            None => n - 1,
        };
        self.selected_verts = vec![(id, next)];
        true
    }

    /// **O que a SELEÇÃO de vértices tem em comum** — o que o painel precisa para destacar (ou não
    /// destacar) um chip Corner/Smooth/Symmetric. `None` se não há vértice selecionado (ou se
    /// nenhum dos índices existe mais).
    ///
    /// ⚠️ **Devolvia o tipo do vértice PRIMÁRIO**, e isso fazia o painel afirmar um tipo sobre uma
    /// seleção que tem três: com dois nós de tipos diferentes selecionados, um dos chips ficava
    /// aceso como se descrevesse o todo (auditoria do plano 25, item 5). O `set_selected_vertex_kind`
    /// sempre agiu sobre TODOS — era só a leitura que mentia.
    pub fn selected_vertex_kind(&self, scene: &VecScene) -> Option<SelectedKind> {
        let mut acc: Option<VertexKind> = None;
        for &(pid, i) in &self.selected_verts {
            // Índice que não existe mais é ignorado, não é "misto": ele não descreve vértice nenhum.
            let Some(k) = scene
                .paths()
                .iter()
                .find(|p| p.id == pid)
                .and_then(|p| p.vert(i))
                .map(|v| v.kind)
            else {
                continue;
            };
            match acc {
                None => acc = Some(k),
                Some(prev) if prev == k => {}
                Some(_) => return Some(SelectedKind::Mixed),
            }
        }
        acc.map(SelectedKind::Uniform)
    }

    /// **Onde a seleção de nós ESTÁ, em MUNDO** — a mediana das âncoras escolhidas.
    ///
    /// ⚠️ **Mundo, e não a coordenada guardada.** O documento guarda geometria LOCAL e a pose vive
    /// no `Transform` (ADR-0111); ler o vértice cru mostraria um número que **discorda da régua
    /// sob ele** em toda forma que já foi movida ou escalada. A lei do módulo é a mesma do pen:
    /// *o que se vê é MUNDO, o que o documento guarda é LOCAL*.
    ///
    /// ⚠️ **A MEDIANA, e não o primário** — é a lição que o irmão [`Self::selected_vertex_kind`]
    /// pagou: ele devolvia o tipo do vértice primário e, com três nós selecionados, afirmava sobre
    /// o todo uma verdade de um só. Uma mediana é uma afirmação verdadeira sobre o conjunto
    /// inteiro, e é ela que torna o campo utilizável com N > 1 sem colapsar a forma (o modelo do
    /// Blender; o do Inkscape põe todos os nós no mesmo X e destrói o desenho).
    ///
    /// Índice que não existe mais é ignorado, não conta para a mediana — o mesmo tratamento do
    /// irmão, e pela mesma razão: ele não descreve vértice nenhum.
    #[must_use]
    pub fn selected_anchor_world(&self, scene: &VecScene) -> Option<[f64; 2]> {
        let mut acc = [0.0f64; 2];
        let mut n = 0u32;
        for &(pid, i) in &self.selected_verts {
            let Some(a) = scene
                .paths()
                .iter()
                .find(|p| p.id == pid)
                .and_then(|p| p.vert(i))
                .map(|v| self.to_world(pid, v.anchor))
            else {
                continue;
            };
            acc[0] += a[0];
            acc[1] += a[1];
            n += 1;
        }
        (n > 0).then(|| {
            let k = f64::from(n);
            [acc[0] / k, acc[1] / k]
        })
    }

    /// Retipa TODOS os vértices selecionados (botões Corner/Smooth/Symmetric).
    /// Devolve `true` se algo mudou (o shell empurra um passo de undo nesse caso).
    pub fn set_selected_vertex_kind(&mut self, scene: &mut VecScene, kind: VertexKind) -> bool {
        let mut changed = false;
        for id in self.vert_paths() {
            let idxs: Vec<usize> = self.verts_in(id).collect();
            let Some(path) = scene.path_mut(id) else {
                continue;
            };
            for i in idxs {
                changed |= ph2d_vec_scene::retype_vertex(path, i, kind);
            }
        }
        changed
    }

    /// Apaga TODOS os vértices selecionados (Delete / botão), re-costurando os
    /// vizinhos. Um contorno que fique com < 2 vértices é descartado — o buraco de
    /// um compound some; se o path inteiro esvaziar, ele é removido e a seleção
    /// limpa. A seleção segue no vizinho do 1º apagado (delete encadeado de um só).
    /// Devolve `true` se apagou algo.
    /// ⚠️ **Apaga em TODAS as formas que a seleção toca**, cada uma tratando os próprios contornos
    /// — e uma forma que morre não leva as outras junto: antes, o caminho único que esvaziava
    /// zerava a seleção inteira e voltava, porque não havia outras para sobreviver.
    pub fn delete_selected_vertex(&mut self, scene: &mut VecScene) -> bool {
        if self.selected_verts.is_empty() {
            return false;
        }
        // O delete ENCADEADO (a seleção segue no vizinho) só faz sentido quando o gesto apagou UM
        // nó: com vários, não há "o vizinho" — e re-selecionar um deles escolheria por ele.
        let single = (self.selected_verts.len() == 1).then(|| self.selected_verts[0]);
        let mut any = false;
        let mut died: Vec<VecPathId> = Vec::new();
        for id in self.vert_paths() {
            let mut idxs: Vec<usize> = self.verts_in(id).collect();
            idxs.sort_unstable();
            idxs.dedup();
            let gone = {
                let Some(path) = scene.path_mut(id) else {
                    continue;
                };
                any = true;
                // Resolve os índices PLANOS em (contorno, local) ANTES de remover: remover
                // encurta um contorno e reescreve o mapa plano dos seguintes.
                let mut located: Vec<(usize, usize)> =
                    idxs.iter().filter_map(|&i| path.locate_vert(i)).collect();
                // Do maior pro menor: assim nenhum índice local ainda pendente desliza.
                located.sort_unstable();
                for &(c, local) in located.iter().rev() {
                    if let Some((verts, closed)) = path.contour_mut(c) {
                        // ⚠️ **PRESERVA A FORMA** (plano 25 §6): os handles dos vizinhos são
                        // re-ajustados para que a cúbica que sobra passe por onde as duas
                        // passavam. Antes disto era um `verts.remove(local)` cru — a curva morria
                        // com o ponto, e é a operação de nó mais usada em qualquer app de
                        // desenho. A porta é a MESMA do Simplify (`dissolve_vertex`): duas cópias
                        // divergiriam, e a divergência apareceria como *"o Simplify preserva a
                        // forma e o Delete não"*, que era o estado anterior.
                        ph2d_vec_scene::dissolve_vertex(verts, local, *closed);
                    }
                }
                // Descarta contornos degenerados (do último pro primeiro). `remove_contour` recusa
                // o contorno ÚNICO ⇒ a forma inteira morre.
                (0..path.contour_count()).rev().any(|c| {
                    path.contour(c).is_some_and(|(v, _)| v.len() < 2) && !path.remove_contour(c)
                })
            };
            if gone {
                scene.remove_path(id);
                died.push(id);
            }
        }
        if !any {
            return false;
        }
        self.selected_verts.clear();
        if let Some((pid, lowest)) = single
            && !died.contains(&pid)
            && let Some(path) = scene.paths().iter().find(|p| p.id == pid)
            && path.total_verts() > 0
        {
            // Delete de um só: seleção segue no vizinho (delete encadeado).
            self.selected_verts = vec![(pid, lowest.min(path.total_verts() - 1))];
        }
        if !died.is_empty() {
            self.selected_paths.retain(|p| !died.contains(p));
            if self.selected.is_some_and(|s| died.contains(&s)) {
                self.selected = self.selected_paths.last().copied();
            }
            if self.active.is_some_and(|a| died.contains(&a)) {
                self.active = None;
            }
        }
        true
    }
}

#[cfg(test)]
#[path = "selection_kind_tests.rs"]
mod kind_tests;

/// Gates da **escala da seleção** (plano 25 §6, W3b) — irmão pelo assunto.
#[cfg(test)]
#[path = "selection_scale_tests.rs"]
mod scale_tests;

/// Sondas do **alcance** da seleção de nós — o que o gesto apanha, de quantas formas.
#[cfg(test)]
#[path = "multi_probe.rs"]
mod multi_probe;

/// Gates da seleção de nós que **atravessa formas** — o irmão executável das sondas acima.
#[cfg(test)]
#[path = "multi_path_tests.rs"]
mod multi_path_tests;

/// Gates do **LAÇO** — a segunda forma da região, e a equivalência que prova o corpo partilhado.
#[cfg(test)]
#[path = "lasso_tests.rs"]
mod lasso_tests;

/// Gates de **ONDE a seleção de nós está** — a leitura em MUNDO que o campo X/Y do painel mostra.
#[cfg(test)]
#[path = "anchor_world_tests.rs"]
mod anchor_world_tests;
