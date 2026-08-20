//! **OS GATES DA ESCALA** — a asserção **A6** do ADR-0160 (*a densidade responde
//! à curvatura*), e o CONTROLE que a torna afirmável.

use ph2d_mesh::shapes;

use super::{MAX_ADAPTIVE_RATIO, ScaleField};

/// Uma malha cuja curvatura VARIA muito entre regiões — sem isso, um campo
/// adaptativo e um uniforme dão a mesma coisa e o gate não discrimina nada.
///
/// ⚠️ **Um toro fino:** o tubo tem curvatura alta e o buraco interno tem a
/// curvatura do raio maior. Uma esfera teria curvatura CONSTANTE — a fixture não
/// conteria o fenômeno.
fn fixture() -> ph2d_mesh::Mesh {
    shapes::torus(64, 24, 1.0, 0.22)
}

/// ⭐ **A6 — O UNIFORME É EXATAMENTE UNIFORME.**
///
/// ⚠️ **É o CONTROLE, e ele vem antes do gate adaptativo de propósito:** um modo
/// "uniforme" que variasse um por cento seria um adaptativo fraco a fingir-se de
/// uniforme, e nenhuma medição de aparência veria a diferença. A razão tem de sair
/// `1,0` **exato**.
#[test]
fn the_uniform_scale_does_not_vary_at_all() {
    let mesh = fixture();
    let f = ScaleField::uniform(&mesh, 0.1);
    let (lo, hi) = f.range();
    assert_eq!(
        lo, hi,
        "o campo uniforme variou entre {lo} e {hi}: ele nao e' uniforme"
    );
    assert_eq!(f.len(), mesh.vert_count(), "o campo nao cobre a malha");
}

/// ⭐ **A6 — O ADAPTATIVO ENCOLHE ONDE A CURVATURA APERTA.**
///
/// O oráculo não é a fórmula: é a **correlação de ordem**. O vértice de maior
/// curvatura tem de receber um lado ESTRITAMENTE menor que o de menor curvatura,
/// e a razão entre os extremos tem de passar de **2×** — que é o número que o
/// ADR-0160 §4 congelou.
#[test]
fn the_adaptive_scale_shrinks_where_the_curvature_bites() {
    let mesh = fixture();
    let f = ScaleField::adaptive(&mesh, 0.1, 1.0);

    let curv = mesh.curvatures();
    let (mut lo_i, mut hi_i) = (0usize, 0usize);
    for v in 0..mesh.vert_count() {
        if curv[v].abs() < curv[lo_i].abs() {
            lo_i = v;
        }
        if curv[v].abs() > curv[hi_i].abs() {
            hi_i = v;
        }
    }
    assert!(
        f.at(hi_i) < f.at(lo_i),
        "o vertice de curvatura ALTA ({:.4}) recebeu lado {:.4}, e o de curvatura BAIXA ({:.4}) \
         recebeu {:.4}: a lei esta' invertida ou inerte",
        curv[hi_i],
        f.at(hi_i),
        curv[lo_i],
        f.at(lo_i)
    );

    let (lo, hi) = f.range();
    eprintln!(
        "[quadflow] escala adaptativa: {lo:.5}..{hi:.5} (razao {:.3}x)",
        hi / lo
    );
    assert!(
        hi / lo >= 2.0,
        "a faixa adaptativa e' de so' {:.3}x ({lo:.5}..{hi:.5}) -- o ADR-0160 §4 congelou 2x",
        hi / lo
    );
}

/// **A FAIXA É LIMITADA** — e o limite diz de que recurso ele é.
///
/// ⚠️ A extração liga células vizinhas da retícula; duas células de escalas
/// incompatíveis deixam de ter aresta comum e a grade **rasga**. O teto não é
/// conforto: ele é a condição de a transição existir.
#[test]
fn the_adaptive_range_is_bounded() {
    let mesh = fixture();
    let f = ScaleField::adaptive(&mesh, 0.1, 1.0);
    let (lo, hi) = f.range();
    assert!(
        hi / lo <= MAX_ADAPTIVE_RATIO + 1.0e-4,
        "a faixa {lo:.5}..{hi:.5} ({:.3}x) passou o teto de {MAX_ADAPTIVE_RATIO}x",
        hi / lo
    );
}

/// **FORÇA ZERO É O UNIFORME, AO BIT.**
///
/// ⚠️ **A saída antecipada é o que garante isto**, e não uma interpolação com
/// `t = 0`: uma curvatura `NaN` numa malha importada atravessaria a aritmética e
/// envenenaria o modo uniforme por um caminho que ninguém suspeitaria.
#[test]
fn zero_strength_is_the_uniform_field_bit_for_bit() {
    let mesh = fixture();
    let a = ScaleField::adaptive(&mesh, 0.1, 0.0);
    let b = ScaleField::uniform(&mesh, 0.1);
    assert_eq!(a, b, "forca zero deixou de ser o campo uniforme");
}

/// **A MEDIANA, e não o MÁXIMO** — um pico não pode esmagar o modelo inteiro.
///
/// ⚠️ **O modo de falha que este gate mata é silencioso e caro:** um único
/// vértice patológico (o polo de uma esfera UV, um pico de uma importação) tem
/// curvatura ordens de grandeza acima do resto. Normalizar pelo MÁXIMO poria
/// todo o resto do modelo contra o piso da faixa — a adaptação inteira a servir
/// um vértice —, e a malha sairia uniforme outra vez, agora com o nome errado.
#[test]
fn a_single_spike_does_not_flatten_the_whole_field() {
    let mesh = fixture();
    let calm = ScaleField::adaptive(&mesh, 0.1, 1.0);

    // A mesma malha com UM vértice puxado para fora — curvatura enorme, num sítio
    // só.
    let mut spiked = fixture();
    let p0 = spiked.positions()[0];
    spiked.positions_mut()[0] = [p0[0] * 4.0, p0[1] * 4.0, p0[2] * 4.0];
    spiked.rebuild();
    let spiky = ScaleField::adaptive(&spiked, 0.1, 1.0);

    // A MEDIANA das duas tem de continuar parecida: o pico move um vértice, não o
    // campo.
    let median = |f: &ScaleField| {
        let mut v: Vec<f32> = (0..f.len()).map(|i| f.at(i)).collect();
        v.sort_by(f32::total_cmp);
        v[v.len() / 2]
    };
    let (a, b) = (median(&calm), median(&spiky));
    assert!(
        (a - b).abs() / a < 0.25,
        "um pico num vertice moveu a mediana do campo de {a:.5} para {b:.5}: a normalizacao esta' \
         a seguir o MAXIMO em vez da mediana"
    );
}

/// ⭐ **TODO PONTO DO CURSO É LEGAL** — o gate que a foto do Enio pediu.
///
/// ⚠️ **O painel oferecia `0,02 … 1,00` em unidades de OBJETO, e as duas pontas
/// destruíam a peça** (2026-08-19): abaixo do que a entrada resolve a extração
/// devolve **malha vazia**, e a `1,5×` a aresta de entrada ela devolve um ciclo de
/// 352 lados com **58 % do volume perdido**. A faixa não estava errada por pouco
/// — ela não era **da malha**.
///
/// Este gate afirma a propriedade que substitui aquela faixa: para qualquer
/// `detail` em `0..1`, o lado do quad que sai **nunca** cai abaixo do piso
/// medido, e a ordem do curso é monótona.
#[test]
fn every_point_of_the_detail_slider_is_legal() {
    for (name, mesh) in [
        ("esfera 48x64", shapes::uv_sphere(48, 64, 1.0)),
        ("toro", shapes::torus(64, 32, 1.0, 0.35)),
        ("uv 96x144", shapes::uv_sphere(96, 144, 1.0)),
    ] {
        let (floor, ceiling) = super::resolvable_edge_range(&mesh);
        let e = super::mean_edge(&mesh);
        assert!(
            floor >= super::FLOOR_IN_INPUT_EDGES * e * 0.999,
            "{name}: o piso saiu {floor:.4}, abaixo das {} arestas de entrada que a medicao pede",
            super::FLOOR_IN_INPUT_EDGES
        );
        assert!(
            ceiling >= floor,
            "{name}: a faixa saiu INVERTIDA ([{floor:.4}, {ceiling:.4}]) -- numa malha grossa os \
             dois extremos cruzam-se, e uma faixa invertida devolve um quad abaixo do piso por \
             aritmetica"
        );

        let mut last = f32::MAX;
        for step in 0..=20 {
            let d = step as f32 / 20.0;
            let s = super::edge_for_detail(&mesh, d);
            assert!(
                s >= floor * 0.999 && s <= ceiling * 1.001,
                "{name}: detail={d:.2} pediu um quad de {s:.4}, fora da faixa legal \
                 [{floor:.4}, {ceiling:.4}] -- e' exatamente o ponto do slider que destroi a peca"
            );
            assert!(
                s <= last,
                "{name}: detail={d:.2} devolveu {s:.4} DEPOIS de {last:.4} -- o curso tem de ir do \
                 grosso ao fino sem voltar"
            );
            last = s;
        }
        // ⚠️ **E fora do curso também**: um `detail` que escape do `0..1` (um
        // arredondamento de slider, um projeto de outra versão) tem de saturar,
        // nunca extrapolar para fora da faixa.
        for d in [-1.0f32, -0.001, 1.001, 7.0, f32::NAN] {
            let s = super::edge_for_detail(&mesh, d);
            assert!(
                s >= floor * 0.999 && s <= ceiling * 1.001,
                "{name}: detail={d} escapou da faixa e devolveu {s:.4}"
            );
        }
    }
}
