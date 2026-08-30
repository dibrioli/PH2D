//! **ONDE OS DOIS KNOBS DA CURVA CHEGAM AO BARRO — e onde eles NÃO chegam.**
//!
//! Esta suíte responde a UMA pergunta, e ela é a terceira da costura: *o valor
//! que o painel escreve alcança um consumidor?* Os `seam_*` provam que o clique
//! chega à ferramenta; o `architecture_panel_wiring_parity` prova que o controle
//! é focalizável. Nenhum dos dois olha para o barro.
//!
//! ⚠️ **A régua é o PRODUTO** (`SculptStroke::dab`), nunca as funções soltas: um
//! `Falloff::weight` correto não prova um pincel que o consome, e é exatamente
//! essa distância que a caça aos knobs mortos de 2026-08-30 mede. O que cada
//! gate faz é trivial de propósito — muda UM knob, roda o MESMO gesto, e compara
//! as posições (ou o canal) **bit a bit**.
//!
//! # O mapa que estes gates congelam
//!
//! O único consumidor dos dois knobs no motor inteiro é o `stroke_dab_core`, e
//! ele tem TRÊS regimes:
//!
//! | regime | a curva do dab | `falloff` | `hardness` |
//! |---|---|---|---|
//! | geometria (`Draw`, `Clay`, …) | `falloff.weight(shaped_distance(t))` | **chega** | **chega** |
//! | máscara (`Verb::Mask`) | `mask_weight(shaped_distance(t))` | ⛔ inerte | **chega** |
//! | campo elástico (`RefMode::L` + verbo com `elastic_field`) | `kelvinlet::rim_landing(t)` | ⛔ inerte | ⛔ inerte |
//!
//! # ⛔ As duas RECUSAS MEDIDAS que estes gates guardam
//!
//! A cura de primeira escolha para um knob inerte é **fazer o consumidor usá-lo**.
//! Ela foi tentada nos dois regimes e as duas vezes ela parte uma lei que já tem
//! gate, com a referência escrita ao lado:
//!
//! - **Máscara.** A curva do canal é a SEGUNDA curva da referência —
//!   `(1 − d)^{2(1 − hardness)}`, `Masking.js:66-69` — e não a quártica da
//!   geometria. Pedir a curva da geometria aqui (ou multiplicá-la) faz o
//!   `the_mask_channel_reproduces_the_reference_kernel` e o
//!   `the_mask_kernel_is_bit_identical` sangrarem, porque o `Verb::Mask` nasce
//!   com `Falloff::Plateau` (a quártica) e a referência não a aplica. Ver
//!   também o campo [`ph2d_sculpt3d::Brush::mask_hardness`], onde a decisão está
//!   escrita com a fonte: *o `Falloff` — o nosso seletor, que é um superconjunto
//!   do que a referência oferece à geometria — **não** o alcança*.
//! - **Campo elástico.** Ali a curva é o SUPORTE do campo: o perfil inteiro
//!   (quanto cada vértice anda e para que lado) é do
//!   [`ph2d_sculpt3d::kelvinlet`], e o que sobra à curva é onde o campo é
//!   avaliado e como ele aterrissa na borda. Multiplicar as duas aplicaria o
//!   perfil **duas vezes**, e a queda do `falloff` a partir do centro comeria o
//!   agarre que o `the_elastic_grab_lands_its_target_exactly` mede.
//!
//! ⇒ Os dois knobs ficam inertes nesses regimes **por lei**, e o que muda é que
//! a inércia deixa de ser um comentário e passa a ser MEDIDA. O painel resolve o
//! resto pelo lado dele: a **Dureza** some onde o campo corre (a fileira segue a
//! mesma porta, `RefMode::field`, que a largura do campo já segue), e o
//! **Falloff** é pintado sempre, por uma cerca com motivo escrito —
//! `the_basic_level_never_hides_the_curve_that_shapes_the_dab`, que porta a
//! decisão do Blender (*o `FalloffPanel` é dobrado, nunca ausente*).
//!
//! Rodar: `bash scripts/cargo-test-narrow.sh ph2d-sculpt3d`

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, Falloff, RefMode, SculptStroke, Symmetry, Verb};

fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(24, 32, 1.0)
}

const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.5;

/// **AS DUAS CURVAS QUE SEPARAM TUDO.** `Constant` é o disco chapado e `Sharper`
/// é a quártica apertada — se um regime lê o seletor, estas duas não podem dar o
/// mesmo barro em lado nenhum.
const A: Falloff = Falloff::Constant;
const B: Falloff = Falloff::Sharper;

/// Um pincel do verbo/modo pedidos, com a curva e a dureza escolhidas À MÃO.
///
/// ⚠️ **`falloff` e `hardness` são escritos DEPOIS do `..Brush::default()`**, e é
/// isso que faz a fixture conter o fenômeno: o default deles é derivado do verbo
/// (`Verb::default_falloff`), então herdar seria medir duas vezes o mesmo pincel.
fn brush(verb: Verb, mode: RefMode, falloff: Falloff, hardness: f32) -> Brush {
    Brush {
        verb,
        mode,
        radius: R,
        strength: 1.0,
        falloff,
        hardness,
        ..Brush::default()
    }
}

/// O gesto: um traço de quatro eventos, sempre o MESMO — o que varia entre as
/// duas corridas é só o pincel.
///
/// ⚠️ **Um PUXÃO e não um carimbo**, porque o `Verb::Move` precisa dele e os
/// verbos de carimbo o ignoram: um gesto só mantém as três colunas comparáveis.
fn run(b: &Brush) -> Mesh {
    let mut mesh = sphere();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 1..=4 {
        let t = k as f32 / 4.0;
        s.dab(
            &mut mesh,
            b,
            &Dab::pulling(TIP, R, EYE, [0.25 * t, 0.0, 0.0]),
            Symmetry::default(),
        );
    }
    mesh
}

/// O maior deslocamento entre duas malhas do mesmo repouso, em unidades de
/// mundo. `0.0` **exato** significa byte a byte.
fn spread(a: &Mesh, b: &Mesh) -> f32 {
    assert_eq!(
        a.vert_count(),
        b.vert_count(),
        "malhas de tamanhos diferentes"
    );
    (0..a.vert_count())
        .map(|i| {
            let (p, q) = (a.positions()[i], b.positions()[i]);
            ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// O maior desvio do canal de máscara entre duas malhas.
fn channel_spread(a: &Mesh, b: &Mesh) -> f32 {
    let (x, y) = (
        a.masks().expect("o canal foi pintado"),
        b.masks().expect("o canal foi pintado"),
    );
    x.iter()
        .zip(y.iter())
        .map(|(u, v)| (u - v).abs())
        .fold(0.0f32, f32::max)
}

/// **O CONTROLE POSITIVO: com um verbo de GEOMETRIA os dois knobs chegam.**
///
/// ⚠️ Sem esta metade os três gates de inércia abaixo seriam satisfeitos por um
/// pincel que ignora os dois knobs em toda parte — verde sobre nada, que é a
/// forma de gate que esta casa varre a cada wave.
#[test]
fn both_curve_knobs_reach_the_clay_of_a_geometry_verb() {
    let flat = run(&brush(Verb::Draw, RefMode::S, A, 0.0));
    let sharp = run(&brush(Verb::Draw, RefMode::S, B, 0.0));
    let by_curve = spread(&flat, &sharp);
    assert!(
        by_curve > 1e-4,
        "trocar a CURVA não moveu o barro do Draw ({by_curve:.3e}) — o seletor \
         está morto no regime em que ele deveria mandar"
    );

    let soft = run(&brush(Verb::Draw, RefMode::S, B, 0.0));
    let hard = run(&brush(Verb::Draw, RefMode::S, B, 0.75));
    let by_hardness = spread(&soft, &hard);
    assert!(
        by_hardness > 1e-4,
        "trocar a DUREZA não moveu o barro do Draw ({by_hardness:.3e})"
    );
}

/// **A CURVA NÃO ALCANÇA O CANAL DE MÁSCARA — e a DUREZA alcança.**
///
/// ⚠️ **As duas metades num gate só, porque é a assimetria que é a lei.** Os dois
/// knobs vivem lado a lado no painel e um deles morre aqui: o `mask_weight` é a
/// curva PRÓPRIA do canal (`Masking.js:66-69`) e o `shaped_distance` roda ANTES
/// dela, exatamente como o `apply_hardness_to_distances` roda antes do
/// `BKE_brush_calc_curve_factors` no original. Um gate que só medisse a metade
/// morta deixaria a viva livre para morrer no dia seguinte, sem sangrar.
///
/// ⛔ **Não conserte isto ligando o `falloff` ao canal.** Ver as recusas medidas
/// no cabeçalho do módulo: o `the_mask_channel_reproduces_the_reference_kernel`
/// e o `the_mask_kernel_is_bit_identical` reprovam, e a decisão está escrita no
/// [`ph2d_sculpt3d::Brush::mask_hardness`] com a linha da referência.
#[test]
fn the_curve_never_reaches_the_mask_channel_but_the_hardness_does() {
    let flat = run(&brush(Verb::Mask, RefMode::S, A, 0.0));
    let sharp = run(&brush(Verb::Mask, RefMode::S, B, 0.0));
    let by_curve = channel_spread(&flat, &sharp);
    assert_eq!(
        by_curve, 0.0,
        "o seletor de curva mexeu no canal de máscara ({by_curve:.3e}): o canal \
         tem curva PRÓPRIA e ligá-lo à da geometria parte a paridade com o \
         `Masking.paint` da referência"
    );
    // ⚠️ E a GEOMETRIA também não se mexe — a metade que prova que o Mask não é
    // um verbo de carimbo disfarçado, e sem a qual `by_curve == 0` seria
    // satisfeito por um traço que não pintou nada.
    assert_eq!(
        spread(&flat, &sharp),
        0.0,
        "o Mask moveu geometria ao trocar de curva"
    );

    let soft = run(&brush(Verb::Mask, RefMode::S, A, 0.0));
    let hard = run(&brush(Verb::Mask, RefMode::S, A, 0.75));
    let by_hardness = channel_spread(&soft, &hard);
    assert!(
        by_hardness > 1e-4,
        "a DUREZA não alcançou o canal de máscara ({by_hardness:.3e}) — ela \
         remapeia a distância que QUALQUER curva lê, o canal incluído"
    );
}

/// **COM UM CAMPO ELÁSTICO ARMADO, NENHUM DOS DOIS ALCANÇA — e o mesmo verbo no
/// `s-mode` é o CONTROLE.**
///
/// ⚠️ **O controle é o que separa *"o knob está morto"* de *"este verbo ignora
/// tudo"*.** O `Verb::Move` em `S` lê os dois; o MESMO verbo em `L` não lê
/// nenhum, porque a curva ali é o `kelvinlet::rim_landing` — uma indicadora com
/// aterrissagem, e não uma escolha do artista. Um gate sem a metade `S` ficaria
/// verde se o Move parasse de esculpir.
///
/// ⛔ **Não conserte isto multiplicando as duas curvas:** o perfil do campo já é
/// o falloff dele, e aplicá-lo duas vezes come o agarre que o
/// `the_elastic_grab_lands_its_target_exactly` mede.
#[test]
fn neither_curve_knob_reaches_an_elastic_field() {
    // A fixture tem de conter o fenômeno: o Move em `L` declara campo, em `S`
    // não. A pergunta é feita à porta do motor, nunca a uma lista de nomes.
    assert!(
        brush(Verb::Move, RefMode::L, A, 0.0)
            .mode
            .field(Verb::Move)
            .is_some(),
        "a fixture perdeu a premissa: o Move em L tem de declarar campo"
    );
    assert!(
        brush(Verb::Move, RefMode::S, A, 0.0)
            .mode
            .field(Verb::Move)
            .is_none(),
        "a fixture perdeu o controle: o Move em S não pode declarar campo"
    );

    for (knob, a, b) in [
        (
            "CURVA",
            brush(Verb::Move, RefMode::L, A, 0.0),
            brush(Verb::Move, RefMode::L, B, 0.0),
        ),
        (
            "DUREZA",
            brush(Verb::Move, RefMode::L, B, 0.0),
            brush(Verb::Move, RefMode::L, B, 0.75),
        ),
    ] {
        let d = spread(&run(&a), &run(&b));
        assert_eq!(
            d, 0.0,
            "a {knob} mexeu no barro sob um campo elástico ({d:.3e}) — ali quem \
             manda é o perfil do kelvinlet, e compor os dois o aplicaria duas vezes"
        );
    }

    // CONTROLE — o MESMO verbo, o MESMO gesto, no `s-mode`: os dois chegam.
    for (knob, a, b) in [
        (
            "CURVA",
            brush(Verb::Move, RefMode::S, A, 0.0),
            brush(Verb::Move, RefMode::S, B, 0.0),
        ),
        (
            "DUREZA",
            brush(Verb::Move, RefMode::S, B, 0.0),
            brush(Verb::Move, RefMode::S, B, 0.75),
        ),
    ] {
        let d = spread(&run(&a), &run(&b));
        assert!(
            d > 1e-4,
            "o controle falhou: a {knob} não alcança o Move nem no `s-mode` \
             ({d:.3e}) — o gate acima estaria medindo um verbo inerte"
        );
    }
}
