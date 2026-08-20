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
/// ⚠️ **O `edge` sai da PORTA DO PRODUTO, e não de um `0.1` escrito à mão.**
/// Aquele literal era **menor que o piso desta fixtura**, então depois de o piso
/// passar a valer no campo adaptativo (2026-08-19) o campo inteiro clampava
/// contra ele e a lei ficava inerte — o gate reprovava sobre código correto. Um
/// `edge` que a malha não consegue resolver não é um caso que o produto tenha:
/// o botão só chega aqui pela [`super::edge_for_detail`]. *A fixtura tem de
/// conter o fenômeno, e pedir o impossível não é o fenômeno.*
#[test]
fn the_adaptive_scale_shrinks_where_the_curvature_bites() {
    let mesh = fixture();
    // ⚠️ **`detail = 0,25` (grosso) e não o meio do curso**, porque a faixa que a
    // A6 exige precisa de FOLGA sob o piso: a `0,50` a folga desta fixtura dá
    // **1,938×** e a asserção é `2×`. Não é a lei que encolheu — é o piso a
    // dizer que abaixo dele não há adaptação nenhuma a fazer.
    let f = ScaleField::adaptive(&mesh, super::edge_for_detail(&mesh, 0.25), 1.0);

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

/// ⭐ **O CAMPO ADAPTATIVO NUNCA PEDE ABAIXO DO PISO** — o invariante que mora
/// na COMPOSIÇÃO, e que nenhum gate desta folha via.
///
/// ⚠️ **Os cinco gates acima que chamam `adaptive` passam `edge = 0.1` escrito à
/// mão**, e afirmam razão e faixa. Nenhum compõe `edge_for_detail → adaptive` —
/// e o defeito não morava em nenhuma das duas: morava em a segunda construir o
/// seu limite inferior sem consultar o piso que a primeira acabara de respeitar.
/// Medido antes do conserto: `lo` saía em **metade** do piso.
///
/// ⚠️ **E o `the_adaptive_range_is_bounded` não o via por construção:** ele afere
/// a RAZÃO entre os dois extremos, que continua `4,0` mesmo com os dois extremos
/// no fundo do poço. *Uma razão sobrevive a uma translação; um piso não.*
#[test]
fn the_adaptive_field_never_asks_below_the_resolvable_floor() {
    for (name, mesh) in [
        ("esfera 48x64", shapes::uv_sphere(48, 64, 1.0)),
        ("uv 96x144", shapes::uv_sphere(96, 144, 1.0)),
        ("toro", shapes::torus(64, 32, 1.0, 0.35)),
    ] {
        let floor = super::resolvable_edge_range(&mesh).0;
        for d in 0..=8u32 {
            let detail = f32::from(d as u16) / 8.0;
            let edge = super::edge_for_detail(&mesh, detail);
            for a in 0..=8u32 {
                let adapt = f32::from(a as u16) / 8.0;
                let field = super::ScaleField::adaptive(&mesh, edge, adapt);
                let (lo, hi) = field.range();
                assert!(
                    lo >= floor * 0.999,
                    "{name}: detail={detail:.3} adapt={adapt:.3} pede um quad de {lo:.5}, abaixo \
                     do piso {floor:.5} que a malha consegue resolver -- e' o canto que devolve a \
                     peca esburacada"
                );
                assert!(
                    hi >= lo,
                    "{name}: detail={detail:.3} adapt={adapt:.3} devolveu a faixa INVERTIDA \
                     [{lo:.5}, {hi:.5}]"
                );
            }
        }
    }
}

/// ⭐ **PERTO DO PISO A ADAPTAÇÃO PARA BAIXO PERDE O CURSO** — a consequência
/// honesta do conserto, afirmada em vez de descoberta.
///
/// ⚠️ **Este gate existe para que ninguém a leia como regressão.** Em
/// `detail = 1,00` o `edge` **é** o piso: não há folga por baixo, então a
/// adaptação só pode subir. O `range` colapsa contra o piso pelo lado de baixo, e
/// isso é o recurso a dizer o que é. *Uma propriedade que surpreende alguém daqui
/// a um mês é uma propriedade que devia ter um teste.*
#[test]
fn at_the_floor_the_adaptive_field_can_only_grow_upwards() {
    let mesh = fixture();
    let floor = super::resolvable_edge_range(&mesh).0;
    let f = ScaleField::adaptive(&mesh, super::edge_for_detail(&mesh, 1.0), 1.0);
    let (lo, hi) = f.range();
    eprintln!("[quadflow] no piso {floor:.5}: faixa [{lo:.5}, {hi:.5}]");
    assert!(
        (lo - floor).abs() <= 1.0e-4 * floor,
        "no piso o menor quad saiu {lo:.5} e o piso e' {floor:.5} -- se ele desceu, o conserto \
         de 2026-08-19 foi desfeito e a peca volta a esburacar"
    );
    assert!(
        hi > lo,
        "no piso a adaptacao ficou INERTE ({lo:.5}..{hi:.5}) -- ela ainda tem de poder CRESCER \
         onde a forma e' chapada"
    );
}
