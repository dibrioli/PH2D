//! **O CORTE** — abrir um contorno num vértice (plano 25 §7, a W4).
//!
//! Este módulo tem UMA função de produto, e ela é o primitivo de que **as quatro** ferramentas de
//! corte são feitas: a tesoura, a faca, a borracha de caminho e o "break path" do modo Node. A
//! diferença entre elas não é geométrica — é só *de onde vem o vértice*:
//!
//! | ferramenta | de onde vem o vértice |
//! |---|---|
//! | Tesoura | um clique: o vértice sob o cursor, senão [`crate::split_segment`] no ponto mais próximo |
//! | Faca | cada cruzamento de uma lâmina reta com a curva |
//! | Borracha de caminho | as duas pontas de um arrasto AO LONGO da curva (dois cortes + um delete) |
//!
//! ⚠️ **Nenhuma delas tem aritmética de arco própria.** A borracha, em particular, não reimplementa
//! o `fx_trim`: dois cortes deixam o trecho do meio como um contorno inteiro, e apagá-lo é
//! [`VecPath::remove_contour`] ou [`VecScene::remove_path`] — que já existem. Uma segunda resposta
//! a *"onde este caminho termina?"* divergiria da primeira no dia em que uma quina viva ou um
//! efeito entrasse no meio.
//!
//! # As três respostas, e por que a última é uma pergunta de FILL RULE
//!
//! - **Contorno FECHADO** → vira ABERTO, re-enraizado no vértice do corte, que passa a aparecer
//!   nas DUAS pontas. Um objeto, um id, um contorno. (É o que o Illustrator faz com a tesoura.)
//! - **Contorno ABERTO, vértice INTERIOR** → parte em dois, e o vértice do corte fica nos dois
//!   lados (as duas metades encostam onde a tesoura passou).
//! - **Ponta de contorno aberto** → `None`. Não há o que cortar ali, e o Illustrator também recusa.
//!
//! A segunda metade vira um **path novo** (arrastável para longe) num path de contorno único, e um
//! **contorno irmão** num compound. A pergunta é feita UMA vez, aqui: separar um contorno de um
//! compound em dois OBJETOS mudaria o que a [`crate::FillRule`] significa — o buraco deixaria de
//! ser buraco no clique que era para ser um corte.
//!
//! # O que o corte NÃO faz
//!
//! Não toca `selected_verts` de ninguém: os índices planos a jusante do corte andam, e quem tem
//! seleção é quem sabe o que ela significa. O chamador limpa.

use crate::{Contour, VecPath, VecPathId, VecScene};

/// O que um corte produziu — o suficiente para o chamador achar as duas metades sem re-derivar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cut {
    /// O contorno estava FECHADO e apenas ABRIU (um objeto, nada nasceu).
    pub opened: bool,
    /// O path NOVO que recebeu a segunda metade (path de contorno único partido em dois).
    pub new_path: Option<VecPathId>,
    /// O índice do contorno NOVO que recebeu a segunda metade (um contorno de compound partido).
    pub new_contour: Option<usize>,
}

impl VecScene {
    /// **Corta o path `id` no vértice de índice PLANO `vert`** — o primitivo de todas as
    /// ferramentas de corte (ver o doc do módulo). `None` quando não há corte a fazer: id
    /// inexistente, índice fora do alcance, contorno degenerado (< 2 vértices) ou o vértice é uma
    /// PONTA de contorno aberto.
    ///
    /// ⚠️ **Os handles do vértice cortado são preservados nas duas cópias**, e isso é deliberado:
    /// um contorno aberto só consome o `out_handle` do primeiro e o `in_handle` do último, então
    /// os handles "mortos" ficam guardados e **re-fechar devolve a curva original ao bit**. Zerá-los
    /// pareceria higiene e destruiria a tangente que o artista autorou.
    pub fn cut_path_at_vertex(&mut self, id: VecPathId, vert: usize) -> Option<Cut> {
        let path = self.paths.iter().find(|p| p.id == id)?;
        let (c, local) = path.locate_vert(vert)?;
        let (verts, closed) = path.contour(c)?;
        if verts.len() < 2 {
            return None;
        }

        if closed {
            let path = self.paths.iter_mut().find(|p| p.id == id)?;
            let (verts, closed) = path.contour_mut(c)?;
            verts.rotate_left(local);
            let seam = verts[0];
            verts.push(seam);
            *closed = false;
            return Some(Cut {
                opened: true,
                new_path: None,
                new_contour: None,
            });
        }

        // Aberto: só um vértice INTERIOR parte o contorno.
        if local == 0 || local + 1 >= verts.len() {
            return None;
        }
        let compound = path.is_compound();
        let z = self.paths.iter().position(|p| p.id == id)?;

        let tail: Vec<crate::VecVertex> = {
            let path = &mut self.paths[z];
            let (verts, _) = path.contour_mut(c)?;
            let tail = verts[local..].to_vec();
            verts.truncate(local + 1);
            tail
        };

        if compound {
            let path = &mut self.paths[z];
            path.subpaths.push(Contour {
                verts: tail,
                closed: false,
            });
            return Some(Cut {
                opened: false,
                new_path: None,
                new_contour: Some(path.contour_count() - 1),
            });
        }

        // Contorno único: a segunda metade vira um OBJETO, logo acima da fonte na pilha de z (as
        // duas metades ficam vizinhas, e nenhuma salta para a frente da cena).
        let src = &self.paths[z];
        let half = VecPath {
            verts: tail,
            closed: false,
            fill: src.fill.clone(),
            stroke: src.stroke,
            fill_rule: src.fill_rule,
            // A pilha de efeitos é APARÊNCIA, e as duas metades continuam a mesma aparência.
            effects: src.effects.clone(),
            ..VecPath::default()
        };
        let new_path = self.insert_path(z + 1, half);
        Some(Cut {
            opened: false,
            new_path: Some(new_path),
            new_contour: None,
        })
    }
}

#[cfg(test)]
#[path = "path_cut_tests.rs"]
mod tests;

/// Quantas amostras por SEGMENTO a busca de cruzamentos toma antes de refinar. O refino é por
/// bisseção, então isto não decide a precisão — decide **quantos cruzamentos são encontrados**:
/// dois cruzamentos dentro da mesma amostra cancelam-se em sinal e passam despercebidos. 32 por
/// segmento resolve uma cúbica a atravessar uma reta três vezes, que é o pior caso de uma cúbica.
const BLADE_SAMPLES: usize = 32;

/// Passos de bisseção depois de a amostragem cercar o cruzamento. 40 leva o intervalo a
/// `2⁻⁴⁰` do de partida — muito abaixo da precisão de qualquer coisa a jusante.
const BLADE_REFINE: usize = 40;

/// Quão perto de uma PONTA de segmento um cruzamento ainda conta como sendo **nela**.
///
/// ⚠️ **O número é MEDIDO, não escolhido:** junto de uma tangência o `side` é ~0 com ruído de
/// `f64`, e a bisseção estagna em vez de convergir — uma lâmina a passar exactamente por um
/// vértice do quadrado da fixture devolvia `t = 4,3e-9`, que um limiar de `1e-9` deixava passar.
/// `1e-6` de um segmento é `1e-4` unidades num de 100: abaixo de tudo o que se vê, e uma ordem de
/// grandeza acima do ruído medido.
const BLADE_END_EPS: f64 = 1e-6;

/// **Onde uma LÂMINA reta `a→b` cruza a curva de `path`** — tudo em coordenadas LOCAIS do path.
/// Devolve `(segmento PLANO, t)` em ordem crescente.
///
/// ⚠️ **É a geometria AUTORADA, não a cozida**, e é deliberado: quem corta é o `split_segment`, que
/// opera na autorada — a mesma escolha que o insert da caneta faz. Num caminho com quina viva ou
/// pilha de efeitos, o que se vê na tela e o que se corta divergem, e essa divergência é do modelo
/// fonte≠cozido (ADR-0121), não desta função.
///
/// ⚠️ **Só cruzamentos DENTRO do segmento da lâmina** — não da reta infinita que a contém. Uma
/// faca é um traço com começo e fim, e cortar o que está para lá da ponta seria a ferramenta a
/// fazer o que ninguém desenhou.
#[must_use]
pub fn blade_crossings(path: &VecPath, a: [f64; 2], b: [f64; 2]) -> Vec<(usize, f64)> {
    let d = [b[0] - a[0], b[1] - a[1]];
    let len2 = d[0] * d[0] + d[1] * d[1];
    if len2 <= f64::EPSILON {
        return Vec::new();
    }
    // Lado da reta (produto vetorial) e posição AO LONGO dela, normalizada.
    let side = |p: [f64; 2]| d[0] * (p[1] - a[1]) - d[1] * (p[0] - a[0]);
    let along = |p: [f64; 2]| ((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / len2;

    let mut out = Vec::new();
    for s in 0..path.total_segments() {
        let at = |t: f64| crate::point_on_segment(path, s, t);
        let Some(p0_first) = at(0.0) else { continue };
        let mut prev = (0.0_f64, side(p0_first));
        for k in 1..=BLADE_SAMPLES {
            let t1 = k as f64 / BLADE_SAMPLES as f64;
            let Some(p1) = at(t1) else { continue };
            let s1 = side(p1);
            // ⚠️ **Um zero exato é UM LADO, não um cruzamento próprio.** Tratá-lo à parte
            // (`prev.1 == 0.0 || …`) contava a mesma travessia duas vezes sempre que uma amostra
            // caía em cima da lâmina — medido: uma aresta vertical cortada ao meio devolvia t=0,50
            // *e* t=0,53. Com o zero dobrado no lado positivo, uma TANGÊNCIA (que toca e volta)
            // deixa de contar, o que é o certo: ali a lâmina não atravessa nada.
            if (prev.1 < 0.0) != (s1 < 0.0) {
                // Cerca o zero e refina por bisseção — sem transcendental, sem solver.
                let (mut lo, mut hi) = (prev.0, t1);
                let (mut slo, _) = (prev.1, s1);
                for _ in 0..BLADE_REFINE {
                    let mid = 0.5 * (lo + hi);
                    let Some(pm) = at(mid) else { break };
                    let sm = side(pm);
                    if (slo < 0.0) != (sm < 0.0) {
                        hi = mid;
                    } else {
                        lo = mid;
                        slo = sm;
                    }
                }
                let t = 0.5 * (lo + hi);
                if let Some(p) = at(t) {
                    let u = along(p);
                    // ⚠️ `t` estritamente dentro: um cruzamento na fronteira de amostra é o mesmo
                    // que o do segmento vizinho, e cortar duas vezes no mesmo ponto deixaria um
                    // segmento de comprimento zero.
                    if (0.0..=1.0).contains(&u) && t > BLADE_END_EPS && t < 1.0 - BLADE_END_EPS {
                        out.push((s, t));
                    }
                }
            }
            prev = (t1, s1);
        }
    }
    out.sort_by(|x, y| x.0.cmp(&y.0).then(x.1.total_cmp(&y.1)));
    out.dedup_by(|x, y| x.0 == y.0 && (x.1 - y.1).abs() < 1e-7);
    out
}
