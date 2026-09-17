//! ⭐⭐⭐ **O COLISOR QUE A FORMA DECLARA, honrado aqui** — ordem do dono (2026-09-17): *«O botão
//! Collide da Shape deve funcionar para todo e qualquer duplicador»* (doc 114 §12).
//!
//! ## O que faltava, e não era o motor
//!
//! O `source.shape` **declara** o colisor dela em colunas (`ph2d_collider` · `ph2d_collider_box` ·
//! `ph2d_collider_offset`), e a [`ph2d_contact`] — o motor de peça-contra-peça da casa, com grelha
//! espacial, caixas orientadas e rotação — **lê exactamente essas colunas**. Medido em 2026-09-17,
//! a declaração já chegava intacta a todas as peças dos cinco duplicadores verdadeiros
//! (`motion.clone` · `motion.mirror` · `motion.kaleidoscope` · `fx.drop_shadow` · `fx.rgb_split`).
//!
//! ⛔ **O que não existia era um LEITOR fora de uma zona de simulação.** As três crates que
//! consumiam aquelas colunas eram todas da família `sim.*`, logo o botão só fazia alguma coisa
//! dentro do laço da simulação — *um controlo vivo e inalcançável a partir de todo o resto do
//! catálogo*, que dá exactamente o mesmo report que um controlo morto.
//!
//! ## Três decisões, cada uma com o porquê
//!
//! **1 · A DECLARAÇÃO GANHA do `Radius` do cartão.** *Só quem desenha sabe o tamanho do que
//! desenha* (o cabeçalho do `collider.rs` da forma): com o mesmo `size = 1` um círculo desenha raio
//! `1` e uma sprite raio inscrito `0,5`, e nenhum consumidor a jusante sabe que mídia recebeu. Um
//! quadrado comprido declarado como caixa `4 × 1` tratado como disco deixaria o ar que o report de
//! 13/09 já tinha medido em `41 %`.
//!
//! **2 · Uma peça SEM declaração cai no `Radius`, como disco.** ⚠️ Sem isto uma corrente MISTA (a
//! forma clonada, fundida com uma grelha) deixaria metade das peças **inertes e caladas** — e um
//! nó que separa umas e não outras é pior que um que não separa nenhuma.
//!
//! **3 · ⛔⛔ O `Strength` e o `falloff` entram por MISTURA no fim, e NUNCA nos pesos.** A
//! tentação é multiplicar o `inv_mass` de cada peça por eles e deixar o motor fazer o resto —
//! e isso torna o `Strength` **INERTE**: a correcção do PBD é `λ = penetração / (k_a + k_b)` com
//! `Δp = n·λ·w`, logo escalar os DOIS pesos de um par divide `k` e multiplica `λ` pelo mesmo
//! factor, *que cancela*. É a armadilha que o `push_apart` deste mesmo ficheiro já documenta um
//! nível acima (*«pôr o peso do par no divisor além do numerador faz ele CANCELAR»*), e ela morde
//! outra vez aqui, noutra aritmética.

use ph2d_contact::{Colisor, Forma, Pecas, Saida};
use ph2d_nodegraph::attr::{Column, Stream};

/// A coluna do ângulo de cada peça, em graus.
const ROT_COL: &str = "rot";

/// Lê o `rot` de um stream, ou zeros.
fn rot_col(s: &Stream, n: usize) -> Vec<f32> {
    match s.get(ROT_COL) {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|r| if r.is_finite() { *r } else { 0.0 })
            .collect(),
        _ => vec![0.0; n],
    }
}

/// O resultado de uma separação pelo colisor DECLARADO: as posições e os ângulos novos.
pub(crate) struct Separado {
    pub pos: Vec<[f32; 2]>,
    pub rot: Vec<f32>,
}

/// **Separa pelo colisor que a corrente DECLARA.** `None` quando ela não declara nenhum — e é
/// isso que mantém toda cena sem colisor **byte-idêntica**: quem pergunta sai antes de tocar em
/// nada.
///
/// `raios` é o que o cartão pediria para cada peça (o `radius` já multiplicado pelo `spread` e
/// pelo `size` dela): ele é o **recuo** da decisão 2, e não um segundo multiplicador da caixa
/// declarada.
pub(crate) fn separa(
    input: &Stream,
    p: &[[f32; 2]],
    pesos: &[f32],
    raios: &[f32],
    falloff: &[f32],
    iteracoes: usize,
    strength: f32,
) -> Option<Separado> {
    let declarados = ph2d_contact::colisores(input)?;
    let n = p.len();
    if declarados.len() != n {
        return None;
    }
    // A decisão 2: quem não declara leva o disco do cartão. ⚠️ Um raio que não é positivo NÃO vira
    // um disco de zero — vira `None`, que o motor lê como *«esta peça não participa»*. Um disco de
    // raio zero participaria de todos os testes para nunca tocar em nada.
    let colisores: Vec<Option<Colisor>> = (0..n)
        .map(|i| {
            declarados[i].or_else(|| {
                let r = raios.get(i).copied().unwrap_or(0.0);
                (r.is_finite() && r > 0.0).then_some(Colisor {
                    forma: Forma::Disco(r),
                    desvio: [0.0, 0.0],
                })
            })
        })
        .collect();
    if !colisores.iter().any(Option::is_some) {
        return None;
    }

    let inv_i = ph2d_contact::inv_inercias(input, &colisores, pesos);
    let mut pos = p.to_vec();
    let mut giro = vec![0.0f32; n];
    ph2d_contact::separate(
        &mut pos,
        &mut Saida { giro: &mut giro },
        &Pecas::novas(&colisores, pesos, &inv_i),
        iteracoes,
    );

    // A decisão 3: a mistura, no fim. `k = 1` devolve o que o motor deu, termo a termo.
    let rot_in = rot_col(input, n);
    let mut out_pos = Vec::with_capacity(n);
    let mut out_rot = Vec::with_capacity(n);
    for i in 0..n {
        let k = (strength * falloff.get(i).copied().unwrap_or(1.0)).clamp(0.0, 1.0);
        out_pos.push([
            p[i][0] + (pos[i][0] - p[i][0]) * k,
            p[i][1] + (pos[i][1] - p[i][1]) * k,
        ]);
        out_rot.push(rot_in[i] + giro[i] * k);
    }
    Some(Separado {
        pos: out_pos,
        rot: out_rot,
    })
}

#[cfg(test)]
#[path = "declarado_tests.rs"]
mod tests;
