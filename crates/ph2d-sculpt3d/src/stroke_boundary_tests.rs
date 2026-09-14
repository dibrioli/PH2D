//! ⭐⭐⭐ **O GATE QUE A CONTAGEM NÃO ERA** — a deformação é MÁXIMA na borda e
//! esmorece para dentro.
//!
//! # O defeito, medido (2026-09-14, report do dono: *«não é a borda que está
//! dobrando, mas a região interna»*)
//!
//! A ponte entregava a curva do pincel **com o argumento ao contrário**. As duas
//! casas usam convenções OPOSTAS e as duas estão certas em casa:
//!
//! | quem | o argumento é | vale `1` em |
//! |---|---|---|
//! | [`ph2d_boundary::Curva`] | **quanto FALTA** (`1 − anel/K`) | o argumento `1` |
//! | [`crate::Falloff::weight`] | **quanto já se ANDOU** (`d/R`) | o argumento `0` |
//!
//! Ligadas sem a inversão, o peso na borda lê `curva(1) = 0` e o máximo cai no
//! anel mais fundo: a boca ficava **parada** e o miolo dobrava. Medido na tigela
//! da cena `=42` (`K = 4`): pesos `0 · 0,156 · 0,5 · 0,844 · 0` e deslocamento
//! `0 · 0,096 · 0,197 · 0,152 · 0`.
//!
//! ⭐ **O oráculo diz o contrário, e não por pouco:** no `grade_agarrar_constante`
//! (`K = 5`) ele desloca `0,1000 · 0,0896 · 0,0648 · 0,0352 · 0,0104 · 0` — o
//! máximo **no anel 0** e monótono para dentro. A lei da crate estava **certa**;
//! o corpus corre-a **directamente**, com a convenção dela, e por isso `51 de 61`
//! fecharam sobre uma ponte partida.
//!
//! ⛔⛔ **E o gate da cena `=42` era VERDE:** ele contava **quantos** vértices se
//! movem (`> 30`), e `144` movem-se nas duas orientações. *Uma régua que conta
//! QUANTOS nunca vê QUAIS* — é a mesma forma do `edge_max` global cego ao quad
//! fino e do `χ` cego à almofada.

use ph2d_mesh::{Face, Mesh};

use crate::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// Uma grelha plana `N×N` em `y = 0` — a forma da própria família `grade_*` do
/// corpus, e a mais fácil de ler por anéis: a borda é o quadrado de fora.
fn grelha(n: usize) -> Mesh {
    let passo = 2.0 / (n as f32 - 1.0);
    let mut pos = Vec::with_capacity(n * n);
    for i in 0..n {
        for j in 0..n {
            pos.push([-1.0 + passo * j as f32, 0.0, -1.0 + passo * i as f32]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(i * n + j).expect("grelha pequena");
    let mut faces = Vec::with_capacity((n - 1) * (n - 1));
    for i in 0..n - 1 {
        for j in 0..n - 1 {
            faces.push(Face::quad(
                idx(i, j),
                idx(i + 1, j),
                idx(i + 1, j + 1),
                idx(i, j + 1),
            ));
        }
    }
    Mesh::from_parts(pos, faces).expect("a grelha é construída aqui")
}

/// O deslocamento MÁXIMO da **coluna do cursor**, vértice a vértice, do mais
/// raso (na borda) ao mais fundo.
///
/// ⚠️⚠️ **A profundidade é GEOMÉTRICA (`z`), e não o anel da lei** — de
/// propósito. Um gate da PONTE que perguntasse à estrutura pelo índice de anel
/// estaria a medir a lei com a régua da própria lei; `z` é o que o dono vê, e
/// numa grelha plana com a cadeia na aresta `z = −1` ele é exactamente *«quão
/// fundo na peça»*.
///
/// ⚠️ **E é UMA coluna, não a malha toda:** a borda desta fixtura é o quadrado
/// inteiro, e os cantos opostos também são borda — uma média por «distância à
/// borda mais próxima» misturaria a região deformada com o resto da peça e
/// leria um planalto onde há uma rampa (medido: `0,452` chapado nos quatro
/// primeiros anéis).
fn perfil() -> Vec<(f32, f32)> {
    perfil_com([0.0, 0.0, 0.4])
}

fn perfil_com(puxao: [f32; 3]) -> Vec<(f32, f32)> {
    let mut malha = grelha(21);
    let antes = malha.positions().to_vec();

    // O cursor no MEIO da aresta `z = −1`, para a cadeia nascer ali.
    let alvo = [0.0, 0.0, -1.0];
    let b = Brush {
        verb: Verb::Boundary,
        radius: 0.6,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&malha);
    s.dab(
        &mut malha,
        &b,
        &Dab::pulling(alvo, b.radius, [0.0, -1.0, 0.0], puxao),
        Symmetry::default(),
    );

    let mut saida: Vec<(f32, f32)> = Vec::new();
    for (a, d) in antes.iter().zip(malha.positions()) {
        if a[0].abs() > 1e-6 {
            continue; // só a coluna do cursor
        }
        let dist = ((d[0] - a[0]).powi(2) + (d[1] - a[1]).powi(2) + (d[2] - a[2]).powi(2)).sqrt();
        saida.push((a[2], dist));
    }
    saida.sort_by(|x, y| x.0.total_cmp(&y.0));
    saida
}

/// ⭐⭐⭐ **A BORDA é quem mais se move, e o efeito MORRE para dentro.**
///
/// É a frase inteira do pincel, e é o que o dono lê na tela. ⚠️ Ela é sobre
/// QUAIS vértices se movem, e não sobre quantos — que é exactamente o que o
/// gate da cena media quando este defeito shipou.
#[test]
fn a_borda_e_quem_mais_se_move_e_o_efeito_morre_para_dentro() {
    let p = perfil();
    assert!(
        p.len() >= 8,
        "a coluna tem de ter fundura para a pergunta ter sentido: {p:?}"
    );
    let (z0, d0) = p[0];
    assert!(
        (z0 + 1.0).abs() < 1e-6,
        "o primeiro ponto da coluna é a BORDA"
    );
    assert!(
        d0 > 0.0,
        "a BORDA não se moveu — é o defeito de 2026-09-14, com a curva do pincel \
         entregue ao contrário: perfil {p:?}"
    );
    // ⚠️ **Monótono, e não «o primeiro é o maior»**: com a curva invertida o
    // ponto da borda é o MENOR, mas um simples `max` deixaria passar o perfil em
    // CORCOVA que o defeito de facto produzia (`0 · 0,096 · 0,197 · 0,152 · 0`).
    let mut anterior = f32::INFINITY;
    for &(z, d) in &p {
        assert!(
            d <= anterior + 1e-6,
            "a `z = {z:.2}` a peça move MAIS que mais perto da borda \
             ({d:.6} > {anterior:.6}) — a deformação tem de esmorecer para \
             dentro: perfil {p:?}"
        );
        anterior = d;
    }
    // ⭐ E ela MORRE de facto: o fundo da coluna não se mexe.
    let (zf, df) = *p.last().expect("coluna não vazia");
    assert_eq!(
        df, 0.0,
        "o fundo da coluna (z = {zf:.2}) ainda se move {df:.6}"
    );
}

/// ⚠️⚠️ **A SONDA QUE ESCOLHEU O PUXÃO desta fixtura, e as duas que falharam.**
///
/// O avanço deste pincel é a **projecção** do arrasto na direcção do
/// ponto-origem (`n̂ = −unit(ponto_origem)`), e numa grelha plana essa direcção
/// vive **no plano**:
///
/// | puxão | deslocamento na coluna |
/// |---|---|
/// | `[0, 0,4, 0]` (perpendicular ao plano) | **tudo zero** — `s ≈ 0` |
/// | `[0,4, 0, 0]` (ao longo da borda) | **tudo zero** — `s ≈ 0`, e é a LEI |
/// | **`[0, 0, ±0,4]`** (para dentro/fora) | a rampa inteira |
/// | `[0, 0,3, 0,3]` | a mesma rampa, mais fraca |
///
/// *Uma fixtura que não contém o fenómeno reprova apontando para o verbo* — e
/// esta reprovou assim na primeira redacção, com `perfil` a ler zeros de ponta a
/// ponta.
#[test]
#[ignore]
fn diag_varre_o_puxao() {
    for puxao in [
        [0.0f32, 0.4, 0.0],
        [0.0, 0.0, 0.4],
        [0.0, 0.0, -0.4],
        [0.0, 0.3, 0.3],
        [0.4, 0.0, 0.0],
    ] {
        println!("puxao {puxao:?} -> {:?}", perfil_com(puxao));
    }
}
