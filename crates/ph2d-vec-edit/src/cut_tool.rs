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

/// Teto de cortes por caminho numa passagem de faca. Uma lâmina reta cruza uma cúbica no máximo
/// três vezes por segmento, então este número é folga larga sobre qualquer forma que um artista
/// desenhe — e existe para que um defeito de convergência vire uma faca que corta de menos, nunca
/// um laço infinito com a janela congelada.
const MAX_KNIFE_CUTS: usize = 256;

impl PenTool {
    /// **A FACA** — uma lâmina reta de `a` a `b` (MUNDO) corta TODO caminho que ela atravesse.
    /// Devolve quantos cortes fez.
    ///
    /// ⚠️ **Não há geometria nova aqui, e é o desenho inteiro:** a faca é a tesoura repetida em
    /// cada cruzamento. Uma origem FECHADA cortada em dois pontos dá duas fitas cujas pontas
    /// assentam na lâmina — e como a lâmina é RETA, a corda que fecharia cada peça **é** a lâmina,
    /// então o resultado coincide com o do Illustrator sem um motor de arranjo no caminho.
    ///
    /// ⚠️ **As peças ficam ABERTAS.** É a escolha do Affinity, e a razão é a que aquele produto
    /// documenta: fechar em silêncio destrói informação (a peça deixa de poder ser reaberta como
    /// estava), enquanto fechar é um clique — o `Close Path`, que a mesma wave ensinou a soldar.
    ///
    /// O laço re-deriva os cruzamentos a cada corte em vez de os pré-calcular, e isso é
    /// deliberado: cortar rota e re-indexa o contorno inteiro, então uma lista feita antes
    /// descreveria vértices que já não existem.
    ///
    /// ⚠️ **A costura recém-criada assenta EXACTAMENTE sobre a lâmina**, e duas camadas
    /// independentes impedem que ela seja reencontrada para sempre — **medido**, cada uma basta
    /// sozinha (com as duas removidas, três gates ficam vermelhos; com qualquer uma delas, verdes):
    ///
    /// 1. o `blade_crossings` **exclui `t` nas pontas** de cada segmento, e um vértice de costura é
    ///    sempre uma ponta (é assim que o `cut_path_at_vertex` o deixa) — esta é a camada
    ///    SEMÂNTICA, e tem gate próprio na `ph2d-vec-scene`;
    /// 2. o conjunto `done` de pontos já cortados, em MUNDO — o cinto deste laço.
    ///
    /// A 2ª não é hoje observável sozinha, e fica registada em vez de removida: a 1ª mora noutra
    /// crate e existe por outro motivo (não reportar o mesmo cruzamento por dois segmentos
    /// vizinhos), então relaxá-la é uma mudança legítima que não sabe que esta faca depende dela.
    pub fn knife_cut(
        &mut self,
        scene: &mut VecScene,
        a: [f64; 2],
        b: [f64; 2],
        hit_r: f64,
    ) -> usize {
        let mut work: Vec<VecPathId> = scene
            .paths()
            .iter()
            .map(|p| p.id)
            .filter(|id| self.view.is_pickable(*id))
            .collect();
        let mut done: Vec<[f64; 2]> = Vec::new();
        let mut cuts = 0usize;

        while let Some(pid) = work.pop() {
            for _ in 0..MAX_KNIFE_CUTS {
                let Some((seg, t, at_world)) = self.next_blade_crossing(scene, pid, a, b, &done)
                else {
                    break;
                };
                done.push(at_world);
                let Some(vert) = self.cut_vertex_at(scene, pid, seg, t, hit_r) else {
                    continue;
                };
                let Some(cut) = scene.cut_path_at_vertex(pid, vert) else {
                    continue;
                };
                cuts += 1;
                if let Some(new_id) = cut.new_path {
                    // ⚠️ **Só a metade NOVA volta à fila.** A fonte fica com tudo até ao corte, e o
                    // corte foi tomado no PRIMEIRO cruzamento que restava — logo ela não pode ter
                    // outro lá dentro, e as duas pontas dela são as costuras, que o
                    // `blade_crossings` exclui. Re-enfileirá-la seria uma volta que nunca acha
                    // nada (mutação-provado: com ela, os números não mudam).
                    work.push(new_id);
                    break;
                }
            }
        }
        if cuts > 0 {
            self.selected_verts.clear();
        }
        cuts
    }

    /// O 1º cruzamento da lâmina com `id` que ainda não foi cortado — `(segmento, t, ponto MUNDO)`.
    fn next_blade_crossing(
        &self,
        scene: &VecScene,
        id: VecPathId,
        a: [f64; 2],
        b: [f64; 2],
        done: &[[f64; 2]],
    ) -> Option<(usize, f64, [f64; 2])> {
        let path = scene.paths().iter().find(|p| p.id == id)?;
        let xf = self.xf(id);
        // A lâmina é MUNDO; a curva é LOCAL (ADR-0111). Um afim degenerado não tem inversa, e um
        // caminho nesse estado não é cortável — recusar é a resposta honesta.
        let inv = xf.inverse()?;
        let (la, lb) = (inv.apply(a), inv.apply(b));
        // O raio do "já cortei aqui" acompanha a ESCALA do caminho: o `done` fala mundo, e num
        // caminho encolhido um raio local fixo apanharia cruzamentos legítimos vizinhos.
        let eps = 1e-6_f64.max(hit_epsilon(xf.mean_scale()));
        for (seg, t) in ph2d_vec_scene::blade_crossings(path, la, lb) {
            let Some(pl) = ph2d_vec_scene::point_on_segment(path, seg, t) else {
                continue;
            };
            let pw = xf.apply(pl);
            if done
                .iter()
                .any(|d| (d[0] - pw[0]).powi(2) + (d[1] - pw[1]).powi(2) <= eps * eps)
            {
                continue;
            }
            return Some((seg, t, pw));
        }
        None
    }
}

/// O raio, em MUNDO, dentro do qual dois cruzamentos são "o mesmo ponto" para a faca. Deriva da
/// escala do caminho para que a resposta não mude quando a forma é ampliada ou reduzida.
fn hit_epsilon(mean_scale: f64) -> f64 {
    (mean_scale.abs() * 1e-9).max(1e-9)
}
