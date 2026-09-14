//! **O APAGADOR DE DESLOCAMENTO, na CENA** — o que só uma pilha de
//! multiresolução a sério pode afirmar.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é o mesmo dos vizinhos: a **LEI**
//! dele (a fracção, a colinearidade, o `Ctrl`) é medida sem GPU na
//! `ph2d-sculpt3d`; aqui fica o que depende da **pilha** — a recusa quando não
//! há, e a prova de que a referência é o LIMITE e não a previsão.

use super::*;

/// Uma peça com `níveis` níveis e umas bossas esculpidas no topo.
fn cena_com_pilha(device: &wgpu::Device, niveis: usize) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, ph2d_mesh::shapes::uv_sphere(16, 24, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.radius_px = 160.0;
    for _ in 1..niveis {
        assert!(s.subdivide(), "a fixtura não conseguiu subdividir");
    }
    s
}

/// A extensão da caixa da peça — a régua de *«ele comeu a forma?»*.
fn extensao(s: &Sculpt3dScene) -> f32 {
    let b = s.mesh().bounds();
    (b.max[0] - b.min[0])
        .max(b.max[1] - b.min[1])
        .max(b.max[2] - b.min[2])
}

/// ⛔⛔ **SEM PILHA ELE RECUSA — e a recusa é o produto** (espec §4.3).
///
/// ⚠️ **O gate desta fronteira é a RECUSA e não o resultado**, e a razão é
/// pública: *o irmão-filtro do alvo ESTOIROU por não fazer esta verificação*.
/// Sem pilha o dado de entrada **não existe** — não é que o resultado seja mau.
///
/// ⭐ **O controlo positivo está dentro:** o MESMO gesto, com pilha, move.
#[test]
#[ignore]
fn sem_pilha_o_apagador_nao_move_e_com_pilha_move() {
    let gpu = gpu_or_skip!();

    // (1) UM nível só: nada a apagar.
    let mut s = cena_com_pilha(&gpu.device, 1);
    s.brush.verb = Verb::EraseMultires;
    let antes = s.mesh().positions().to_vec();
    um_dab(&mut s);
    assert_eq!(
        s.mesh().positions(),
        &antes[..],
        "o apagador moveu barro numa peça sem pilha — ali não existe \\
         deslocamento nenhum a apagar"
    );

    // ⭐ (2) O CONTROLO: com pilha e com detalhe esculpido, ele move.
    let mut s = cena_com_pilha(&gpu.device, 3);
    s.brush.verb = Verb::Draw;
    s.brush.strength = 1.0;
    for _ in 0..4 {
        um_dab(&mut s);
    }
    let esculpida = s.mesh().positions().to_vec();
    s.brush.verb = Verb::EraseMultires;
    um_dab(&mut s);
    let movidos = s
        .mesh()
        .positions()
        .iter()
        .zip(&esculpida)
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        movidos > 20,
        "o apagador moveu só {movidos} vértices com a pilha montada e detalhe \\
         esculpido — sem isto a metade (1) não afirma nada"
    );
}

/// ⭐⭐⭐ **ELE NÃO COME A FORMA — a referência é o LIMITE e não a PREVISÃO**
/// (espec §2.1).
///
/// ⛔⛔ **É a metade que um gate de *«o vértice mexeu-se»* não vê.** Um apagador
/// ancorado em `subdivide(base)` funcionaria, mexeria e **encolheria a peça a
/// cada passagem** — medido no canto de um cubo, `11 %` de resíduo. O artista
/// leria isso como *«o apagador comeu a forma»*.
///
/// ⚠️ **A régua é a CAIXA da peça inteira**, e não o vértice: o que o dono vê é
/// a silhueta. E o gate exige as duas metades — a peça **não encolhe** e o
/// detalhe **desaparece** —, porque um apagador inerte também não encolheria
/// nada.
#[test]
#[ignore]
fn o_apagador_tira_o_detalhe_e_nao_encolhe_a_peca() {
    let gpu = gpu_or_skip!();
    let mut s = cena_com_pilha(&gpu.device, 3);
    let lisa = extensao(&s);

    // Esculpe, e depois apaga várias vezes no mesmo sítio.
    s.brush.verb = Verb::Draw;
    s.brush.strength = 1.0;
    for _ in 0..6 {
        um_dab(&mut s);
    }
    let esculpida = extensao(&s);
    assert!(
        esculpida > lisa + 1e-3,
        "a fixtura não esculpiu nada ({lisa:.4} -> {esculpida:.4}) — o gate \\
         abaixo mediria o nada"
    );

    s.brush.verb = Verb::EraseMultires;
    s.brush.strength = 1.0;
    for _ in 0..8 {
        um_dab(&mut s);
    }
    let apagada = extensao(&s);

    // (1) O detalhe SAIU: a peça voltou para perto da lisa.
    assert!(
        apagada < esculpida - 1e-3,
        "o apagador não tirou o relevo ({esculpida:.4} -> {apagada:.4})"
    );
    // ⭐ (2) **E ela NÃO ENCOLHEU abaixo da lisa.** É aqui que a previsão
    // falharia: cada passagem deixaria o resíduo do esquema, e a caixa desceria
    // por baixo do ponto de partida.
    assert!(
        apagada > lisa - 2e-3,
        "a peça ENCOLHEU abaixo da lisa ({lisa:.4} -> {apagada:.4}) — a \\
         referência deixou de ser o LIMITE e passou a ser a previsão de um \\
         passo, que é literalmente outro pincel (espec §2.1 e §4.4)"
    );
}

/// ⭐⭐⭐ **A REFERÊNCIA É O LIMITE, E NÃO A PREVISÃO — medido directamente.**
///
/// ⛔⛔ **Este gate nasceu de uma mutação SOBREVIVENTE.** O irmão de cima mede a
/// escolha pela **caixa da peça**, e numa esfera UV subdividida isso não
/// discrimina: quase todo vértice é regular (valência `4`), e ali a previsão e o
/// limite estão a menos do que a tolerância da silhueta. Trocar um pelo outro
/// passava. *Uma régua indirecta pode ser cega exactamente onde a escolha
/// acontece.*
///
/// ⇒ a régua passa a ser a **própria tabela de referência**, sobre uma base de
/// **CUBO**: ali todo canto tem valência `3`, que é onde o resíduo do esquema é
/// grande. As duas metades:
///
/// 1. a referência **é** o ponto-limite, ao bit da aritmética;
/// 2. ela **difere** da previsão por muito mais que o ruído — é o `11 %` que a
///    espec §2.1 nomeia, e é a diferença inteira entre este pincel e o outro
///    que a §4.4 manda não construir com este nome.
#[test]
#[ignore]
fn a_referencia_e_o_limite_e_nao_a_previsao() {
    let gpu = gpu_or_skip!();
    // ⚠️ **CUBO e não esfera:** os cantos têm valência `3`, onde o resíduo do
    // esquema é grande. Numa esfera UV quase tudo é regular e as duas
    // superfícies quase coincidem — foi assim que a mutação sobreviveu.
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::cube(1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    assert!(s.subdivide(), "a fixtura não conseguiu subdividir");

    let referencia = s
        .superficie_de_referencia()
        .expect("com dois níveis há referência");
    let baixo = s.objects[s.active]
        .stack
        .level_mesh(0)
        .expect("o nível de baixo existe")
        .clone();
    let previsto = ph2d_mesh::subdivide(&baixo);

    let mut pior_contra_limite = 0.0f32;
    let mut pior_contra_previsao = 0.0f32;
    for (v, &r) in referencia.iter().enumerate() {
        let prev = previsto.positions()[v];
        if let ph2d_mesh::LimitPoint::At(q) = ph2d_mesh::limit_point(&previsto, v) {
            let d = |a: [f32; 3], b: [f32; 3]| {
                (a[0] - b[0])
                    .abs()
                    .max((a[1] - b[1]).abs())
                    .max((a[2] - b[2]).abs())
            };
            pior_contra_limite = pior_contra_limite.max(d(r, q));
            pior_contra_previsao = pior_contra_previsao.max(d(r, prev));
        }
    }
    assert!(
        pior_contra_limite < 1e-6,
        "a referência afasta-se do ponto-limite em {pior_contra_limite:e} — ela \
         deixou de ser a superfície que a espec §2 define"
    );
    // ⭐ **O controlo que dá sentido ao de cima:** se as duas superfícies
    // coincidissem, a metade anterior seria trivial.
    assert!(
        pior_contra_previsao > 1e-2,
        "a referência está a {pior_contra_previsao:e} da PREVISÃO — nesta \
         fixtura as duas superfícies coincidem, logo o gate acima não afirma \
         nada. Troque a peça por uma com vértices irregulares."
    );
}

/// ⭐ **E ele NUNCA ULTRAPASSA a referência** (espec §4.3, o tecto `min(f, 1)`).
///
/// ⚠️ **A régua é a ESTABILIDADE e não uma distância:** insistir no mesmo sítio
/// tem de **assentar**. Um pincel sem tecto passaria do liso e começaria a cavar
/// para o outro lado, e a assinatura disso é a caixa a **crescer** outra vez
/// depois de ter descido.
#[test]
#[ignore]
fn insistir_com_o_apagador_assenta_e_nao_cava() {
    let gpu = gpu_or_skip!();
    let mut s = cena_com_pilha(&gpu.device, 3);
    s.brush.verb = Verb::Draw;
    s.brush.strength = 1.0;
    for _ in 0..6 {
        um_dab(&mut s);
    }
    s.brush.verb = Verb::EraseMultires;
    let mut serie = Vec::new();
    for _ in 0..12 {
        um_dab(&mut s);
        serie.push(extensao(&s));
    }
    // Da metade em diante ela tem de estar parada.
    let cauda = &serie[6..];
    let (lo, hi) = (
        cauda.iter().copied().fold(f32::MAX, f32::min),
        cauda.iter().copied().fold(0.0f32, f32::max),
    );
    assert!(
        hi - lo < 2e-3,
        "a peça não assentou ao insistir ({lo:.4}..{hi:.4}) — sem o tecto \\
         `min(f, 1)` ele passa do liso e começa a cavar"
    );
}
