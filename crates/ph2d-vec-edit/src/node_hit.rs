//! **O que um press do modo Node EDITARIA aqui** — a pergunta sem efeito colateral (`impl PenTool`
//! irmão, teto de LOC).
//!
//! Existe pelo mesmo motivo que o [`crate::corner_tool::PenTool::corner_hit_at`]: a shell precisa
//! decidir, **antes** do press, se a geometria daquele caminho é DERIVADA — e para isso ela tem de
//! saber se o press vai de fato mexer num vértice, ou se é só um clique que seleciona a forma.
//!
//! ⚠️ **`on_press_node` devolve `Grabbed` nos DOIS casos** (agarrou um vértice · clicou no
//! preenchimento e selecionou), então ler o retorno não distingue um do outro — e congelar a receita
//! de uma forma viva num clique que só a selecionou a expandiria sem o artista pedir. É a mesma
//! armadilha que o doc do `corner_hit_at` nomeia, com o mesmo remédio: **uma constante de raio só, e
//! uma BUSCA só**, perguntada pelas duas portas.

use crate::{Grab, Part, PenClick, PenTool};
use ph2d_vec_scene::{VecPathId, VecScene};

/// Raio de captura do nó, em PIXELS de tela. **Uma constante só** — o `on_press_node` e o
/// [`PenTool::node_edit_hit_at`] têm de concordar, senão a shell decide sobre uma edição que o press
/// depois não faz (ou o contrário).
pub(crate) const NODE_HIT_PX: f64 = 10.0;

/// Amostras por segmento no hit-test de "inserir vértice perto do traço".
pub(crate) const INSERT_SAMPLES: u32 = 24;

impl PenTool {
    /// **Que caminho um press do modo Node editaria em `p`?** — a MESMA busca do
    /// [`PenTool::on_press_node`], sem efeito colateral.
    ///
    /// `Some(id)` = o press vai agarrar um vértice/handle daquele caminho, ou inserir um vértice num
    /// segmento dele. `None` = o press vai apenas selecionar (ou desselecionar), e portanto **não
    /// edita geometria nenhuma**.
    ///
    /// A shell a consulta para congelar a receita de uma forma VIVA antes do gesto: sem isto o
    /// arrasto de nó é aceito e depois **descartado em silêncio** pelo `recook_into` do primeiro
    /// slider de parâmetro que o artista tocar.
    #[must_use]
    pub fn node_edit_hit_at(
        &self,
        scene: &VecScene,
        p: [f64; 2],
        px_to_world: f64,
    ) -> Option<VecPathId> {
        let hit_r = NODE_HIT_PX * px_to_world;
        if let Some(g) = self.hit_test(scene, p, hit_r) {
            return Some(g.path);
        }
        self.insert_hit(scene, p, hit_r).map(|(id, _, _)| id)
    }

    /// A busca do INSERT, **sem efeito colateral**: qual segmento do caminho selecionado o cursor
    /// alcança, e onde. Porta única — o [`PenTool::insert_on_selected_segment`] (que corta) e o
    /// [`PenTool::node_edit_hit_at`] (que só pergunta) leem daqui.
    ///
    /// Só o caminho SELECIONADO, e é previsível: é onde as âncoras aparecem.
    pub(crate) fn insert_hit(
        &self,
        scene: &VecScene,
        p: [f64; 2],
        hit_r: f64,
    ) -> Option<(VecPathId, usize, f64)> {
        let sel = self.selected?;
        let path = scene.paths().iter().find(|pp| pp.id == sel)?;
        // A curva é local; a distância que o usuário enxerga é mundo.
        let pl = self.to_local(sel, p);
        let (seg, t, d2) = ph2d_vec_scene::nearest_point_on_path(path, pl, INSERT_SAMPLES)?;
        (d2.sqrt() * self.xf(sel).mean_scale() <= hit_r).then_some((sel, seg, t))
    }

    /// Perto de um SEGMENTO do caminho selecionado → **agarra o segmento** para o reformar. A
    /// topologia não muda: nenhum vértice nasce, e o ponto que o dedo pegou segue o dedo.
    ///
    /// ⚠️ **Isto era um INSERT** (plano 25 §6): pressionar sobre a curva partia o segmento em dois
    /// e agarrava o vértice novo, então **não havia como reformar uma curva sem alterar a
    /// topologia dela** — o gesto que o Illustrator (Direct Selection) e o Inkscape documentam
    /// como o normal. A inserção não se perdeu: ela é da **CANETA**, que insere no mesmo hit-test
    /// (`on_press`, uma linha abaixo do dela) — é a divisão do Illustrator, onde a seta branca
    /// reforma e a Pen/Add-Anchor acrescenta.
    pub(crate) fn grab_segment(
        &mut self,
        scene: &VecScene,
        p: [f64; 2],
        hit_r: f64,
    ) -> Option<PenClick> {
        let (sel, seg, t) = self.insert_hit(scene, p, hit_r)?;
        self.selected_paths = vec![sel];
        // Nenhum vértice é selecionado: o que está agarrado é o TRECHO, e acender as âncoras
        // vizinhas diria ao artista que ele está a mover pontos — o Delete seguinte apagaria dois
        // nós que ele não escolheu.
        self.selected_verts.clear();
        self.grab = Some(Grab {
            path: sel,
            // ⚠️ No `Part::Segment` o `vert` guarda o índice do SEGMENTO (ver o doc do variant).
            vert: seg,
            part: Part::Segment,
            radius_offset: 0.0,
            seg_t: t,
            chamfer: None,
        });
        Some(PenClick::Grabbed)
    }

    /// Perto de um SEGMENTO do caminho selecionado → insere um vértice (split de Bézier, forma
    /// preservada) e o agarra pra arrastar de imediato. `None` quando o cursor não está perto de
    /// segmento nenhum.
    pub(crate) fn insert_on_selected_segment(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        hit_r: f64,
    ) -> Option<PenClick> {
        let (sel, seg, t) = self.insert_hit(scene, p, hit_r)?;
        let ni = ph2d_vec_scene::split_segment(scene.path_mut(sel)?, seg, t)?;
        self.selected_paths = vec![sel];
        self.selected_verts = vec![ni];
        self.grab = Some(Grab {
            path: sel,
            vert: ni,
            part: Part::Anchor,
            radius_offset: 0.0,
            seg_t: 0.0,
            chamfer: None,
        });
        Some(PenClick::Inserted)
    }
}

#[cfg(test)]
#[path = "node_hit_tests.rs"]
mod tests;

/// Gates do **alcance do nó** (plano 25 §6) — a costura dos dois gestos, irmã pelo assunto.
#[cfg(test)]
#[path = "node_reach_tests.rs"]
mod node_reach_tests;
