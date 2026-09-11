//! **O OPERADOR SOBRE O QUAL O `l-mode` DO SMOOTH CORRE** — os gates do
//! laplaciano por cotangentes (Meyer/Desbrun/Schröder/Barr 2003).
//!
//! Irmão do `taubin_pair.rs`, cortado por ASSUNTO: lá mora *quantas vezes o dab
//! roda e com que fator*; aqui, *sobre que operador o alvo do anel é
//! construído*. As duas perguntas têm a MESMA chave — `(Smooth, L)` — e é
//! exatamente essa coincidência que o primeiro gate pina, porque ela é o que
//! impede o chip de anunciar um paper e rodar outro.

use ph2d_mesh::{Mesh, shapes, shapes_open};
use ph2d_sculpt3d::{Brush, Dab, Falloff, RefMode, RingOperator, SculptStroke, Symmetry, Verb};

fn smooth_brush(mode: RefMode, radius: f32) -> Brush {
    Brush {
        verb: Verb::Smooth,
        mode,
        radius,
        strength: 1.0,
        // ⚠️ **`Constant` de propósito:** com uma curva macia o peso cairia com
        // a distância e o número falaria do FALLOFF junto. O que se mede aqui é
        // o operador.
        falloff: Falloff::Constant,
        ..Brush::default()
    }
}

/// Um dab que cobre a malha INTEIRA — o *Filter Layer*, onde o operador é o
/// efeito e não um detalhe de borda de pegada.
fn whole_dab(radius: f32) -> Dab {
    Dab::at([0.0, 0.0, 0.0], radius, [0.0, 0.0, -1.0])
}

fn len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Quanto os vértices escorregaram **ao longo** da superfície — a grandeza que o
/// paper existe para reduzir, e a única que separa os dois operadores num
/// número.
fn tangential_drift(before: &[[f32; 3]], nrm: &[[f32; 3]], after: &[[f32; 3]]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..before.len() {
        let d = [
            after[i][0] - before[i][0],
            after[i][1] - before[i][1],
            after[i][2] - before[i][2],
        ];
        let n = nrm[i];
        let along = d[0] * n[0] + d[1] * n[1] + d[2] * n[2];
        sum += len([
            d[0] - along * n[0],
            d[1] - along * n[1],
            d[2] - along * n[2],
        ]);
    }
    #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: contagem de vértices.
    let n = before.len() as f32;
    sum / n
}

/// N dabs do mesmo pincel sobre a malha inteira.
fn run(mesh: &mut Mesh, b: &Brush, dabs: usize, radius: f32) {
    for _ in 0..dabs {
        let mut s = SculptStroke::default();
        s.begin(mesh);
        s.dab(mesh, b, &whole_dab(radius), Symmetry::default());
    }
}

/// **AS DUAS PERGUNTAS TÊM A MESMA CHAVE** — o par λ|μ e o operador geométrico
/// entram e saem JUNTOS.
///
/// ⚠️ **É a bi-implicação que importa, e ela é escrita nos dois sentidos:** um
/// `(Smooth, L)` com o par e sem o operador rodaria o Taubin sobre o laplaciano
/// que o paper não nomeia (o mundo de antes desta wave, com o chip a prometer
/// literatura); e um operador sem o par encolheria a malha com pesos
/// geométricos, que é o defeito que o Taubin existe para curar.
#[test]
fn the_taubin_pair_runs_on_the_cotangent_operator() {
    for verb in Verb::ALL {
        for mode in RefMode::ALL {
            let b = Brush {
                verb,
                mode,
                ..Brush::default()
            };
            let pair = b.passes().len() > 1;
            let cot = b.ring_operator() == RingOperator::Cotangent;
            assert_eq!(
                pair, cot,
                "{verb:?}/{mode:?}: par={pair} operador-cotangente={cot} — as duas \
                 metades do mesmo l-mode têm de entrar juntas"
            );
            assert_eq!(
                cot,
                verb == Verb::Smooth && mode == RefMode::L,
                "{verb:?}/{mode:?}: o operador geométrico é do l-mode do Smooth e de mais ninguém"
            );
        }
    }
}

/// **O QUE O CHIP COMPRA, pela porta do PRODUTO** — o `l-mode` desliza muito
/// menos que o `b-mode`.
///
/// ⚠️ **Isto NÃO isola o operador, e a distinção me custou uma mutação.** Sob o
/// `L` entram DUAS coisas ao mesmo tempo — o par λ|μ e o operador geométrico —
/// e o par sozinho já reduz a deriva: com o `ring_operator` forçado a `Uniform`
/// este gate fica **VERDE**. Ele é honesto como afirmação sobre o CHIP (é o que
/// o artista vê ao trocá-lo), e a isolação mora onde as duas portas são públicas
/// e não há terceira variável: `ph2d-mesh`,
/// `cotangent_tests::the_geometric_ring_slides_far_less_than_the_uniform_one`.
///
/// ⚠️ **O CONTROLE é o `b-mode`, e a minha primeira versão usava o `s-mode` —
/// que conflacionava uma diferença a mais.** O `L` recua para o `B` naquilo que
/// não declara (`kernel_for`), então `S` contra `L` mede a lei de kernel junto.
/// Os quatro números que separam as leituras:
///
/// | verbo | `S` | `B` | `L` |
/// |---|---|---|---|
/// | Smooth | 0,01227704 | 0,00272686 | **0,00004865** |
/// | Sharpen | 0,01314242 | 0,00266994 | 0,00266994 |
///
/// O `S/B` de ~4,8× aparece nos DOIS verbos e é a lei de kernel, que precede
/// esta wave; o `B/L` de **56,1×** é o que o chip compra. A barra pede **8×**.
///
/// ⚠️ **A coluna `L` era `0,00002088` (130×) e MOVEU com o `λ`** — ele saiu de
/// `0,33` para `0,5` quando o smoke mediu o modo como *"quase imperceptível"*
/// (ver [`ph2d_sculpt3d::TAUBIN_LAMBDA`]). Um filtro mais forte anda mais, logo
/// desliza mais: é o preço, e ele está medido. A tabela é reproduzida pela sonda
/// `measure_smoothing_power.rs::the_drift_table_the_gate_cites`, que roda a
/// fixture DESTE gate exactamente para ela não envelhecer sozinha.
#[test]
fn the_literature_chip_slides_the_surface_far_less_than_the_blender_one() {
    let radius = 4.0;
    let base = shapes::uv_sphere(24, 32, 1.0);
    let before = base.positions().to_vec();
    let nrm = base.normals().to_vec();

    let mut uniform = base.clone();
    run(&mut uniform, &smooth_brush(RefMode::B, radius), 4, radius);
    let du = tangential_drift(&before, &nrm, uniform.positions());

    let mut cot = base.clone();
    run(&mut cot, &smooth_brush(RefMode::L, radius), 4, radius);
    let dc = tangential_drift(&before, &nrm, cot.positions());

    assert!(
        du > dc * 8.0,
        "deriva tangencial: uniforme {du:.8} contra cotangente {dc:.8}"
    );
    // ⚠️ **O CONTROLE.** Sem ele um operador que não movesse NADA passaria — e
    // *não suavizar* não é *suavizar sem deslizar*.
    assert!(
        dc > 0.0 && len(cot.positions()[0]) < len(before[0]) + 1e-3,
        "o l-mode tem de ter de facto suavizado (deriva {dc})"
    );
}

/// **A BEIRA RECUA PARA O UNIFORME, e o Smooth não congela numa malha aberta.**
///
/// O operador geométrico devolve `None` numa borda — a construção pede dois
/// ângulos por aresta e uma aresta de beira tem um só. Se o recuo não existisse,
/// o `l-mode` deixaria a boca de uma peça extraída intacta enquanto o `s-mode` a
/// alisa, e o artista leria *"o Smooth parou de funcionar nesta malha"*.
#[test]
fn the_open_mesh_still_smooths_under_the_literature_mode() {
    let radius = 4.0;
    let base = shapes_open::open_tube3();
    for mode in [RefMode::S, RefMode::L] {
        let mut m = base.clone();
        run(&mut m, &smooth_brush(mode, radius), 3, radius);
        let moved = (0..base.positions().len())
            .filter(|&i| {
                let (a, b) = (base.positions()[i], m.positions()[i]);
                len([a[0] - b[0], a[1] - b[1], a[2] - b[2]]) > 1e-5
            })
            .count();
        assert!(
            moved > base.positions().len() / 2,
            "{mode:?}: só {moved} de {} vértices se moveram numa malha aberta",
            base.positions().len()
        );
    }
}

/// **O OPERADOR NÃO VAZA PARA O VERBO QUE PARTILHA A PORTA**, e o oráculo é
/// BYTE A BYTE entre dois pincéis DIFERENTES.
///
/// ⚠️ **O verbo de controle é o [`Verb::Sharpen`], e não um qualquer:** ele chama
/// a MESMA `neighbour_average` e é o outro que a lê no caminho quente — se a
/// troca fosse chaveada por MODO em vez de pelo PAR, é nele que apareceria.
///
/// ⚠️ **E o par que ele compara é `B` contra `L`, porque o `L` não DECLARA o
/// Sharpen** (o paper do Taubin descreve um passa-baixa e não diz nada sobre
/// afiar), então `kernel_for` recua para o `B` — e os dois têm de sair
/// idênticos ao bit. É um oráculo real: **dois pincéis distintos** obrigados ao
/// mesmo resultado, e não a função sob teste comparada consigo mesma.
///
/// ⚠️ **A minha primeira versão deste gate era AUTO-REFERENTE:** ela rodava o
/// mesmo pincel duas vezes e comparava os dois resultados, o que testa
/// determinismo e **não pode falhar** sob mutação nenhuma do operador — a forma
/// exata que este repo já documenta como o oráculo sempre-verde. A segunda usava
/// o `s-mode` como controle e reprovou em `4,92×` **sobre produto correto**,
/// porque ali a lei de kernel entra junto.
#[test]
fn the_geometric_operator_does_not_leak_into_the_verb_next_door() {
    let radius = 4.0;
    let base = shapes::uv_sphere(24, 32, 1.0);

    let sculpted = |verb: Verb, mode: RefMode| {
        let b = Brush {
            verb,
            mode,
            radius,
            strength: 1.0,
            falloff: Falloff::Constant,
            ..Brush::default()
        };
        let mut m = base.clone();
        run(&mut m, &b, 4, radius);
        m.positions().to_vec()
    };

    assert_eq!(
        sculpted(Verb::Sharpen, RefMode::B),
        sculpted(Verb::Sharpen, RefMode::L),
        "o Sharpen não é declarado pelo l-mode, logo o `L` dele TEM de recuar \
         para o `B` — ao bit, sem o operador geométrico"
    );
    // ⚠️ **O CONTROLE POSITIVO.** Sem ele, um `ring_operator` que devolvesse
    // `Uniform` para TUDO passaria na asserção acima: ela ficaria verde sobre o
    // mundo em que esta wave nunca existiu.
    assert_ne!(
        sculpted(Verb::Smooth, RefMode::B),
        sculpted(Verb::Smooth, RefMode::L),
        "o l-mode do Smooth TEM de divergir do b-mode — é a entrega da wave"
    );
}
