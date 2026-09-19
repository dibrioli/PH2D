//! **OS DOIS CAMPOS SOBRE UMA MANCHA** — a orientação semeada pelo traço e a
//! retícula que sai dela.
//!
//! Filho (`#[path]`) do [`super`], e o corte é por responsabilidade: no pai mora
//! *quem é a mancha e como a lei chega ao barro*, aqui *as duas leis sobre ela*.
//! ⚠️ Ele saiu de lá por **TECTO DE LOC** (`703` contra `700`) no dia em que a
//! memória do traço entrou, e o tamanho é o sintoma — o assunto é a razão.
//!
//! ⛔ **Isto continua a NÃO ser um segundo motor:** as duas funções centrais são
//! a [`crate::orientation::smooth_on_fixed`] e a
//! [`crate::position::smooth_on_fixed`], que a [`crate::hierarchy`] já usava; o
//! que vive aqui é a SEMENTE de cada uma e a redução ao domínio local.

use core::cmp::Ordering;

use super::{Mancha, norm};

/// **O CAMPO DE ORIENTAÇÃO da mancha, semeado pelo TRAÇO.**
///
/// Cada vértice nasce com a direcção do traço projectada no plano tangente
/// dele, e a suavização 4-RoSy torna o campo **consistente** sobre a curvatura
/// — que é o trabalho que a projecção sozinha não faz: dois vértices vizinhos
/// com normais diferentes recebem tangentes que, comparadas, podem estar a 90°
/// uma da outra sem que nada esteja errado, e é a compatibilização extrínseca
/// que escolhe o representante certo de cada uma.
///
/// ⚠️ **A franja é semeada como o miolo e depois PREGADA.** Ela é a direcção
/// que o artista pediu, não a que a superfície tinha — o dab IMPÕE, e a
/// atenuação do pincel é que decide quanto do que o campo diz chega ao barro.
///
/// ⛔ **Direcção nula devolve o campo VAZIO**, nunca um eixo inventado: é a
/// mesma degenerescência que o `ph2d_rake::pentear` declara — com menos de dois
/// carimbos não há direcção, e não há lei que dizer.
#[must_use]
pub fn orientacao_semeada(m: &Mancha, direccao: [f32; 3], iteracoes: usize) -> Vec<[f32; 3]> {
    if m.is_empty() || norm(direccao) <= 0.0 {
        return Vec::new();
    }
    let mut dirs: Vec<[f32; 3]> = m
        .nrm
        .iter()
        .map(|&n| crate::orientation::project_tangent(direccao, n))
        .collect();
    debug_assert_eq!(dirs.len(), m.len());
    crate::orientation::smooth_on_fixed(&mut dirs, &m.nrm, &m.adj, &m.fronteira, iteracoes);
    dirs
}

/// ⭐⭐⭐ **O CAMPO DE POSIÇÃO da mancha — a RETÍCULA de lado `passo`.**
///
/// Devolve, para cada vértice, **o ponto da grelha quadrada onde ele devia
/// estar**: uma retícula de lado `passo` alinhada com [`orientacao_semeada`],
/// consensual entre vizinhos, e reduzida ao ponto mais perto do vértice.
///
/// # Porque esta é a classe certa e a troca de diagonais não era
///
/// Trocar diagonais escolhe **que vértices se ligam**; ela não tem como dizer
/// *«e a que distância»*. Uma retícula diz as duas coisas de uma vez — o
/// alinhamento e o espaçamento **igual nas duas direcções** são a mesma
/// construção —, e é por isso que ela não compra uma à custa da outra.
///
/// # ⚠️ O deslocamento é TANGENCIAL e LIMITADO, e as duas coisas são por
/// construção
///
/// A `position_round_4` monta o ponto a partir de `q` e `n×q`, que são tangentes
/// ao vértice, e devolve o **mais perto** da posição dele ⇒ o alvo vive no plano
/// tangente e nunca está a mais de meia diagonal (`0,71 · passo`). *A malha
/// desliza; ela não incha nem encolhe.*
///
/// ⛔ **`dirs` tem de ser o campo desta mancha** (mesmo comprimento). Com
/// `passo` não positivo, ou sem campo, devolve vazio — não há retícula que
/// dizer.
#[must_use]
pub fn posicao_da_mancha(
    m: &Mancha,
    dirs: &[[f32; 3]],
    passo: f32,
    iteracoes: usize,
) -> Vec<[f32; 3]> {
    posicao_da_mancha_com(m, dirs, passo, iteracoes, false)
}

/// A mesma, com a **SEMENTE** por parâmetro — a variável que a sonda do arame
/// varre. `false` é o que a referência faz (cada vértice é a própria origem).
#[must_use]
pub fn posicao_da_mancha_com(
    m: &Mancha,
    dirs: &[[f32; 3]],
    passo: f32,
    iteracoes: usize,
    semente_unica: bool,
) -> Vec<[f32; 3]> {
    if m.is_empty() || dirs.len() != m.len() || passo.partial_cmp(&0.0) != Some(Ordering::Greater) {
        return Vec::new();
    }
    // ⚠️ **A SEMENTE decide a COERÊNCIA**, e é ela que a sonda varre: com cada
    // vértice a nascer como a própria origem, a suavização só faz consenso
    // LOCAL; com uma origem só, a mancha inteira partilha uma retícula.
    let semente = if semente_unica {
        let n = m.len() as f32;
        let c = m.pos.iter().fold([0.0f32; 3], |a, p| {
            [a[0] + p[0] / n, a[1] + p[1] / n, a[2] + p[2] / n]
        });
        vec![c; m.len()]
    } else {
        m.pos.clone()
    };
    posicao_da_mancha_de(m, dirs, passo, iteracoes, &semente)
}

/// ⭐⭐⭐⭐ **A MESMA lei, com a SEMENTE dada de fora** — a porta por onde a
/// memória do traço entra ([`CampoDoTraco`]).
///
/// ⚠️ **É o ÚNICO solver**: as duas irmãs acima constroem uma semente e chamam
/// esta. *A semente é a única coisa que distingue as três, e escrever o laço
/// três vezes seria três leis que divergem na primeira wave que mexesse numa.*
///
/// ⛔ Com `semente == m.pos` o resultado é **byte-idêntico** ao que a
/// [`posicao_da_mancha`] devolve — é essa a propriedade que faz a memória vazia
/// não mudar o produto, e há gate a afirmá-la.
#[must_use]
pub fn posicao_da_mancha_de(
    m: &Mancha,
    dirs: &[[f32; 3]],
    passo: f32,
    iteracoes: usize,
    semente: &[[f32; 3]],
) -> Vec<[f32; 3]> {
    if m.is_empty()
        || dirs.len() != m.len()
        || semente.len() != m.len()
        || passo.partial_cmp(&0.0) != Some(Ordering::Greater)
    {
        return Vec::new();
    }
    let escalas = vec![passo; m.len()];
    let mut pos = semente.to_vec();
    crate::position::smooth_on_fixed(
        &mut pos,
        &m.pos,
        &m.nrm,
        dirs,
        &escalas,
        &m.adj,
        &m.fronteira,
        iteracoes,
    );
    pos
}
