//! **A pressão da caneta** — o corpo do `on_press` e a variante da lâmina do Corte (F29). Irmão do
//! `lib.rs` pelo tecto de LOC; o corte é por GESTO.

use super::*;

impl PenTool {
    /// A pressão da caneta com a ferramenta **Corte** na mão: o clique só desenha a LÂMINA — nunca
    /// agarra um nó, reabre uma ponta, insere sobre um segmento ou emenda a lâmina noutro caminho
    /// (dono 25/10, F29: *«não consigo cortar nada com ela»* — o clique sobre a forma editava-a).
    pub fn on_press_blade(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        px_to_world: f64,
        snap: &mut dyn FnMut([f64; 2]) -> [f64; 2],
    ) -> PenClick {
        self.press(scene, p, px_to_world, false, snap, true)
    }

    pub(crate) fn press(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        px_to_world: f64,
        alt: bool,
        snap: &mut dyn FnMut([f64; 2]) -> [f64; 2],
        lamina: bool,
    ) -> PenClick {
        let close_dist = 12.0 * px_to_world;
        let hit_r = 10.0 * px_to_world;
        let stroke_w = self.style.stroke_w;

        // Desenhando → continua o pen (adiciona/fecha).
        if let Some(id) = self.active {
            // O ponto de fechamento é comparado no MUNDO (o cursor está lá).
            let first_world = scene
                .paths()
                .iter()
                .find(|pp| pp.id == id)
                .and_then(|pp| pp.verts.first())
                .map(|v| self.to_world(id, v.anchor));
            let snapped = snap(p);
            let anchor_local = self.to_local(id, snapped);
            // Fechar: o cursor voltou ao PRIMEIRO ponto do path ativo (≥3 vértices).
            let close = scene
                .paths()
                .iter()
                .find(|pp| pp.id == id)
                .is_some_and(|pp| pp.verts.len() >= 3)
                && first_world.is_some_and(|fw| {
                    let (dx, dy) = (p[0] - fw[0], p[1] - fw[1]);
                    (dx * dx + dy * dy).sqrt() <= close_dist
                });
            if close {
                let Some(path) = scene.path_mut(id) else {
                    return PenClick::Ignored;
                };
                path.closed = true;
                path.fill = Some(Paint::solid(self.style.fill));
                self.active = None;
                self.dragging = false;
                return PenClick::Closed;
            }
            // Clicar sobre o endpoint de OUTRO path aberto FUNDE os dois num só
            // objeto (Enio 2026-07-09) — a via de construir uma forma fechada com
            // várias linhas/arcos/traços.
            if !lamina && let Some(click) = self.join_open_path(scene, id, snapped, close_dist) {
                return click;
            }
            let Some(path) = scene.path_mut(id) else {
                self.active = None;
                self.dragging = false;
                return PenClick::Ignored;
            };
            path.verts.push(VecVertex::corner(anchor_local));
            self.selected_verts = vec![(id, path.verts.len() - 1)];
            self.dragging = true;
            return PenClick::Added;
        }

        // Parado → CONTINUAR um path aberto pelo seu endpoint tem prioridade: clicar
        // na ponta de uma linha/arco a reativa para seguir desenhando (e, fechando no
        // outro extremo, vira uma forma fechada). Convenção da Pen do Illustrator.
        if !lamina && self.reopen_endpoint(scene, p, hit_r) {
            return PenClick::Grabbed;
        }
        // hit-test para EDITAR um ponto existente. **Nenhum modo agarra alça de raio aqui:** ela
        // saiu do Node e virou o par de ferramentas Fillet / Chamfer (`corner_tool`), então o
        // arredondar/chanfrar quina tem press próprio. (Este comentário dizia *"ela é do modo
        // NODE"* e contradizia o código shipado 35 linhas abaixo — a auditoria do plano 25 o pegou.)
        if !lamina && let Some(g) = self.hit_test(scene, p, hit_r) {
            self.grab_vertex(scene, g, alt);
            return PenClick::Grabbed;
        }
        if !lamina && let Some(click) = self.insert_on_selected_segment(scene, p, hit_r) {
            return click;
        }

        // Nada agarrado → começa um path novo. É a ÚNICA coisa que a caneta faz e a
        // edição de nós não (ADR-0112): `on_press_node` para antes desta linha.
        let id = scene.push_path(VecPath {
            verts: vec![VecVertex::corner(snap(p))],
            stroke: Some(self.style.stroke_spec(stroke_w)),
            ..VecPath::default()
        });
        self.active = Some(id);
        self.selected = Some(id);
        self.selected_paths = vec![id];
        self.selected_verts = vec![(id, 0)];
        self.dragging = true;
        PenClick::Started
    }
}
