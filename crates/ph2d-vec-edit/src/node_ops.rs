//! **As três operações de NÓ da W4** — Join · Average · Reverse (plano 25 §7).
//!
//! Irmão de `selection.rs` pelo teto de LOC, e o corte é: lá mora *quem está selecionado*, aqui
//! *o que se faz com a seleção* quando a resposta muda a TOPOLOGIA (ou a direção) do caminho.
//!
//! As três são roteadores finos: a geometria inteira vive nas portas do `ph2d-vec-scene`
//! (`join_paths` / `close_path` / `reverse_path`). O que este arquivo decide é **a que objetos o
//! gesto se aplica**, e é aí que a pergunta é de PRODUTO:
//!
//! - **Join** solda 2+ caminhos numa cadeia. ⚠️ **Não fecha um caminho só**, embora o `Ctrl+J` do
//!   Illustrator faça as duas coisas: o botão **`Close Path`** da seção PATH já é essa metade, e
//!   um segundo caminho para "fechar" divergiria dele no primeiro refino (foi a auditoria desta
//!   wave que achou o botão já lá). O que o Join acrescentou ao `Close Path` foi a **solda** — ele
//!   passou a chamar a mesma porta, e deixou de deixar dois vértices sobrepostos.
//! - **Average** colapsa os nós selecionados no CENTROIDE deles. Compõe com o Join: *Average +
//!   Join* é a solda exata de duas pontas, o par canônico do Illustrator.
//! - **Reverse** inverte o sentido do caminho — quem decide de que lado uma ponta de seta aponta,
//!   para onde um texto-em-caminho corre e qual contorno é buraco.
//!
//! ⚠️ **As três invalidam `selected_verts`** (índices planos que andam, ou que descrevem um
//! caminho que deixou de existir), então as três a limpam. Deixá-la de pé apontaria para nós que
//! o artista não escolheu, e o Delete seguinte apagaria os errados.

use crate::PenTool;
use ph2d_vec_scene::{VecPathId, VecScene};

/// Tolerância de solda, em unidades de MUNDO: pontas mais perto que isto fundem num vértice só;
/// mais longe, o segmento entre elas aparece.
///
/// Não é um palpite de conforto — é a granularidade do que o artista consegue *afirmar* que
/// coincide. O `Average` (que existe justamente para tornar duas pontas coincidentes) as põe no
/// MESMO ponto, com erro zero; um clique à mão nunca chega perto disto. Quem quer a linha de
/// ligação afasta as pontas; quem quer a solda usa o Average antes, que é o par documentado.
pub const WELD_TOL: f64 = 1e-6;

impl PenTool {
    /// **Average** — colapsa os vértices selecionados no CENTROIDE das âncoras deles.
    ///
    /// Move o vértice INTEIRO (âncora e os dois handles) por uma translação, então a tangente de
    /// cada nó sobrevive intacta: o que muda é onde ele está, não como a curva chega nele.
    /// `false` com menos de 2 vértices selecionados, ou quando eles já estão todos no mesmo ponto
    /// (nada a mover ⇒ nenhum passo de undo espúrio).
    ///
    /// ⚠️ **Os dois eixos de uma vez.** O Illustrator oferece H / V / ambos; as três são a MESMA
    /// função com um eixo mascarado, e o caso que compõe com o Join — o único que *tem* de
    /// existir para soldar — é o de ambos. As variantes por-eixo entram quando o smoke as pedir,
    /// e não como um terceiro botão nascido de simetria.
    /// ⚠️ **O centroide é tomado no MUNDO, e isto é CORREÇÃO, não generalização gratuita:** dois
    /// nós de formas diferentes vivem em espaços locais diferentes (ADR-0111), e mediar as
    /// coordenadas locais de frames distintos não significa nada — o "centro" cairia num lugar que
    /// não é o meio de coisa nenhuma. Com **uma** forma o resultado é o MESMO que sempre foi, e
    /// não por sorte: um mapa afim comuta com o centroide (`xf⁻¹(média(xf(aᵢ))) = média(aᵢ)`),
    /// então a rota nova reduz exatamente à antiga onde a antiga era válida.
    pub fn average_selected_verts(&mut self, scene: &mut VecScene) -> bool {
        // ⚠️ A contagem que decide é a dos índices que RESOLVEM, não a dos selecionados: uma
        // guarda em `selected_verts().len()` aqui em cima seria puro atalho (o `n == 0` abaixo a
        // subsume inteira), e uma mutação prova isso — dois guards para uma pergunta são um a mais.
        let mut sum = [0.0f64; 2];
        let mut n = 0usize;
        for &(pid, i) in self.selected_verts() {
            let Some(a) = scene
                .paths()
                .iter()
                .find(|p| p.id == pid)
                .and_then(|p| p.vert(i))
                .map(|v| v.anchor)
            else {
                continue;
            };
            let w = self.to_world(pid, a);
            sum[0] += w[0];
            sum[1] += w[1];
            n += 1;
        }
        // ⚠️ **Esta guarda é HIGIENE, e a mutação prova** (nenhum gate sangra ao trocá-la por
        // `false`). Quem protege de verdade é o laço abaixo: um índice estale não resolve, então
        // um centroide NaN não é escrito em lugar nenhum. Ela fica porque um `0/0` guardado num
        // local é a forma de defeito que o próximo leitor "conserta" na direção errada — e
        // afirmar que ela é load-bearing seria a mentira que este comentário existe para evitar.
        //
        // Pela mesma medição, pedir `n < 2` aqui seria inerte com aparência de regra: com um só
        // nó o centroide é ele mesmo, o deslocamento é zero e o `moved` já devolve `false`.
        if n == 0 {
            return false;
        }
        let c = [sum[0] / n as f64, sum[1] / n as f64];
        let mut moved = false;
        for id in self.vert_paths() {
            // O centroide DESCE ao espaço desta forma — um número de mundo escrito num vértice
            // local poria o nó onde a moldura da forma o levasse, não onde o artista viu.
            let cl = self.to_local(id, c);
            let idxs: Vec<usize> = self.verts_in(id).collect();
            let Some(path) = scene.path_mut(id) else {
                continue;
            };
            for i in idxs {
                let Some(v) = path.vert_mut(i) else { continue };
                let d = [cl[0] - v.anchor[0], cl[1] - v.anchor[1]];
                if d[0] == 0.0 && d[1] == 0.0 {
                    continue;
                }
                crate::selection::shift_vert(v, d);
                moved = true;
            }
        }
        moved
    }

    /// **Join** — solda os caminhos selecionados numa CADEIA. `false` com menos de dois
    /// selecionados ou quando nenhum par é soldável (todos fechados / vazios).
    ///
    /// ⚠️ **Fechar um caminho só NÃO é daqui** — é o `Close Path` da seção PATH, que já existia.
    ///
    /// A cadeia é dobrada da esquerda para a direita e **cada solda re-lê o sobrevivente**: soldar
    /// A com B e depois o RESULTADO com C é o que "juntar três pedaços" significa; guardar `A`
    /// como alvo fixo tentaria soldar em C um id que já não existe.
    pub fn join_selection(&mut self, scene: &mut VecScene) -> bool {
        let ids: Vec<VecPathId> = self.selected_paths().to_vec();
        if ids.len() < 2 {
            return false;
        }
        self.selected_verts.clear();
        let mut kept = ids[0];
        let mut any = false;
        for &next in &ids[1..] {
            match scene.join_paths(kept, next, &self.xforms, WELD_TOL) {
                Some(id) => {
                    kept = id;
                    any = true;
                }
                // Um par que não solda (fechado, vazio) não pode matar a cadeia: o artista pediu
                // para juntar o que dá, e parar aqui esconderia as soldas ainda possíveis.
                None => continue,
            }
        }
        if any {
            self.select(Some(kept));
        }
        any
    }

    /// **Reverse** — inverte o sentido de cada caminho selecionado. `false` se nada foi tocado.
    pub fn reverse_selected_paths(&mut self, scene: &mut VecScene) -> bool {
        let ids: Vec<VecPathId> = self.selected_paths().to_vec();
        if ids.is_empty() {
            return false;
        }
        self.selected_verts.clear();
        let mut any = false;
        for id in ids {
            any |= scene.reverse_path(id);
        }
        any
    }
}

#[cfg(test)]
#[path = "node_ops_tests.rs"]
mod tests;
