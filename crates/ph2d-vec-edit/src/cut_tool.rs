//! **A TESOURA** (plano 25 §7, W4) — clicar num caminho e ele abre ali.
//!
//! É o primeiro consumidor do [`ph2d_vec_scene::VecScene::cut_path_at_vertex`], e o corpo dela é
//! literalmente uma pergunta: **de onde vem o vértice?** A resposta são três linhas —
//! `path_at` (que caminho), `insert_hit` (onde nele) e `split_segment` (o vértice, se ainda não
//! houver um ali) — todas portas que já existiam para o insert da caneta.
//!
//! # Clicar EM CIMA de um vértice não insere um segundo
//!
//! Se o cursor cai a `hit_r` de uma das âncoras do segmento, a tesoura corta NELA. Sem isso, um
//! clique sobre um nó existente inseriria um vértice coincidente e o caminho ficaria com um
//! segmento de comprimento zero — invisível no desenho, e um degrau em todo Simplify, Average e
//! Delete seguinte.
//!
//! ⚠️ **A comparação é feita em MUNDO**, como todo hit-test desta crate: a curva é local, o raio de
//! captura é o que o artista vê, e a ponte entre os dois é o `mean_scale` do afim — a mesma
//! conversão que o `insert_hit` faz uma linha acima.

use crate::PenTool;
use ph2d_vec_scene::{VecPathId, VecScene};

impl PenTool {
    /// **Corta** o caminho sob o cursor no ponto `p` (MUNDO). Devolve o id do caminho cortado —
    /// que continua a ser o SELECIONADO — ou `None` se o cursor não alcança caminho nenhum, ou se
    /// o ponto cai numa PONTA de caminho aberto (não há ali o que abrir).
    ///
    /// O gesto vale sem pré-selecionar: a tesoura (re)seleciona o que está sob o cursor, como as
    /// ferramentas de quina e a de largura.
    pub fn scissors_cut(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        hit_r: f64,
    ) -> Option<VecPathId> {
        let id = self.path_at(scene, p, hit_r)?;
        self.select(Some(id));
        let (_, seg, t) = self.insert_hit(scene, p, hit_r)?;

        let vert = self.cut_vertex_at(scene, id, seg, t, hit_r)?;
        // ⚠️ A seleção de nó já morreu no `select` acima, e uma limpeza EXPLÍCITA aqui seria
        // redundante (mutação-provado: apagá-la não sangra gate nenhum). Ela importa e está
        // gateada — os índices planos a jusante do corte andam, e um Delete a seguir apagaria o
        // nó errado —, mas quem responde é a porta de seleção, não uma segunda linha aqui.
        scene.cut_path_at_vertex(id, vert)?;
        Some(id)
    }

    /// **Onde a tesoura corta**: o índice PLANO do vértice — o que já estava lá, se o clique caiu
    /// sobre uma âncora, senão um recém-inserido pelo `split_segment`.
    ///
    /// Extraído por ser a única decisão do gesto: o resto é encanamento, e uma decisão testável
    /// separada é o padrão `hit_plan` do repo.
    fn cut_vertex_at(
        &self,
        scene: &mut VecScene,
        id: VecPathId,
        seg: usize,
        t: f64,
        hit_r: f64,
    ) -> Option<usize> {
        let scale = self.xf(id).mean_scale();
        let path = scene.paths().iter().find(|pp| pp.id == id)?;
        let (c, local) = path.locate_segment(seg)?;
        let (verts, closed) = path.contour(c)?;
        let n = verts.len();
        // O segmento `local` do contorno vai do vértice `local` ao seguinte (que dá a volta num
        // contorno fechado).
        let next = if closed && local + 1 == n {
            0
        } else {
            local + 1
        };
        let ends = [path.flat_vert(c, local)?, path.flat_vert(c, next)?];

        // O ponto que a tesoura tocou, em LOCAL — derivado de `(seg, t)`, e não re-projetado do
        // cursor: as duas respostas divergiriam, e a que manda é a que o `insert_hit` deu.
        let touch = ph2d_vec_scene::point_on_segment(path, seg, t)?;
        for &e in &ends {
            let a = path.vert(e)?.anchor;
            let d = ((touch[0] - a[0]).powi(2) + (touch[1] - a[1]).powi(2)).sqrt() * scale;
            if d <= hit_r {
                return Some(e);
            }
        }
        ph2d_vec_scene::split_segment(scene.path_mut(id)?, seg, t)
    }
}

#[cfg(test)]
#[path = "cut_tool_tests.rs"]
mod tests;
