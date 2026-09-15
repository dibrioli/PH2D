//! ⭐⭐⭐ **UM RÓTULO QUE CABE NÃO É PINTADO COM RETICÊNCIAS** — o irmão exacto do
//! [`super::a_label_that_does_not_fit_is_elided_not_wrapped`], pelo outro lado.
//!
//! ⛔⛔ **Report do dono, 2026-09-14, foto do cartão JUMP:** *«… mesmo com folga»* — seis dos nove
//! rótulos cortados enquanto o campo ao lado mostrava `2 m`, `0` e `1` com metade da caixa vazia.
//! ⚠️ **E o conjunto cortado não tinha relação nenhuma com o comprimento:** `Takeoff Gravity`
//! (`94,97 px`) saía inteiro e `Air Jumps` (`62,92`) saía cortado, na MESMA coluna. *Quando o
//! defeito não ordena pelo tamanho, a causa não é o tamanho.*
//!
//! # A causa: uma largura RE-DERIVADA por diferença
//!
//! O [`ph2d_editor_core::widget::paint_property_label`] alinha o rótulo à direita — o texto começa
//! em `recuo = x + (col_w − largura)` — e depois dava ao pintor o orçamento `x + col_w − recuo`,
//! *pretendendo* dizer `largura`. Em `f32` essa soma-e-subtracção cancela com erro: quando o
//! resultado cai um ULP abaixo, o pintor conclui que o texto não cabe e **volta a cortá-lo**.
//!
//! Medido antes da cura, com a coluna a `140 px` e o rótulo mais largo a `100,3`:
//! **310 de 3 600** células (nove rótulos × 400 posições de `x`).
//!
//! ⚠️⚠️ **Por que nenhum gate o via:** os que existiam mediam a DECISÃO (`prefix_width > coluna`)
//! ou a PORTA (`property_label_origin`), e a decisão estava certa nas 3 600 células — quem errava
//! era o segundo corte, dentro do pintor. *Um gate que mede a decisão é cego ao que a tinta faz com
//! ela* ⇒ a régua deste ficheiro é a **cena emitida**, contada em glifos.

use ph2d_editor_core::widget::paint_property_label;
use ph2d_text::TextSystem;
use ph2d_vector::{Color, VectorScene};

/// Quantos glifos a cena leva — a única régua que distingue `Air Jumps` de `Air Jum…`.
///
/// ⚠️ **Não é `n_paths` nem `n_path_segments`:** o Vello encaminha texto por `draw_glyphs`, e as
/// duas contagens dão **zero** para qualquer rótulo (lição já paga em
/// [`super::a_longer_suffix_is_never_shadowed_by_a_shorter_one`] e no L-System).
fn glifos(cena: &VectorScene) -> usize {
    cena.inner()
        .encoding()
        .resources
        .glyph_runs
        .iter()
        .map(|r| r.glyphs.end - r.glyphs.start)
        .sum()
}

/// Os nove rótulos do cartão da foto, verbatim.
///
/// ⚠️ **Fixtura do REPORT, não inventada** — e a ordem é a do `JUMP_ROWS` do Inspector. ⛔ Uma lista
/// de nomes curtos e iguais mediria silêncio: o que faz o defeito aparecer é a **dispersão** de
/// larguras (de `62,9` a `100,3`) contra um `x` que varre.
const ROTULOS: &[&str] = &[
    "Jump Height",
    "Air Jumps",
    "Air Jump Height",
    "Takeoff Gravity",
    "Takeoff Above",
    "Peak Gravity",
    "Peak Window",
    "Fall Gravity",
    "Cut Gravity",
];

/// O `x` de uma linha de cartão não é redondo (o cartão recua `Spacing::Sm` de um painel que o
/// artista arrasta), então a varredura anda em passos **fraccionários** — é aí que o cancelamento
/// aparece. ⛔ Com `x` inteiro o defeito reproduz-se em muito menos células.
const PASSO_X: f32 = 0.37;
const CELULAS_X: u16 = 400;

fn pintado(ts: &mut TextSystem, texto: &str, x: f32, col_w: f32, fonte: f32) -> usize {
    let mut cena = VectorScene::new();
    paint_property_label(
        ts,
        &mut cena,
        texto,
        x,
        0.0,
        fonte,
        col_w,
        Color::from_rgba8(255, 255, 255, 255),
    );
    glifos(&cena)
}

/// Quantos glifos o texto tem quando é pintado sem orçamento nenhum — o oráculo do «inteiro».
fn inteiro(ts: &mut TextSystem, texto: &str, fonte: f32) -> usize {
    let mut cena = VectorScene::new();
    ph2d_editor_core::paint::paint_text_elided(
        ts,
        &mut cena,
        texto,
        0.0,
        0.0,
        fonte,
        f32::INFINITY,
        Color::from_rgba8(255, 255, 255, 255),
    );
    glifos(&cena)
}

#[test]
fn a_label_that_fits_is_never_painted_with_dots() {
    let mut ts = TextSystem::without_system_fonts();
    let fonte = 13.0_f32;
    // A coluna é larga de propósito: os nove cabem com `40` a `77 px` de folga. *A folga é o
    // assunto* — o dono não se queixou de um rótulo apertado, queixou-se de um com espaço de sobra.
    let col_w = 140.0_f32;
    assert!(
        ROTULOS.len() >= 9,
        "piso de populacao: a fixtura do report tem 9 rotulos, ficou com {}",
        ROTULOS.len()
    );

    let mut folga_minima = f32::MAX;
    let mut cortes = Vec::new();
    for texto in ROTULOS {
        let esperado = inteiro(&mut ts, texto, fonte);
        folga_minima = folga_minima.min(col_w - ts.prefix_width(texto, fonte));
        for k in 0..CELULAS_X {
            let x = 100.0 + f32::from(k) * PASSO_X;
            if pintado(&mut ts, texto, x, col_w, fonte) != esperado {
                cortes.push(format!("{texto:?} em x = {x}"));
            }
        }
    }
    assert!(
        folga_minima > 30.0,
        "a fixtura deixou de ter folga ({folga_minima:.1} px) — ela deixaria de medir o report"
    );
    assert!(
        cortes.is_empty(),
        "{} de {} celulas pintam reticencias num rotulo que cabe com {folga_minima:.1} px de \
         folga:\n  {}\n\nO orcamento do pintor tem de ser a largura MEDIDA do que coube, nunca \
         `x + col_w - recuo` (ver `property_label_origin`).",
        cortes.len(),
        ROTULOS.len() * CELULAS_X as usize,
        cortes
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// ⭐⭐ **E NUMA COLUNA MAIS ESTREITA QUE A PRÓPRIA RETICÊNCIA ele não pinta NADA.**
///
/// ⚠️ **É a outra metade do orçamento** (`largura.min(col_w)`): ali o
/// [`ph2d_editor_core::text_elide::fit`] devolve o texto **cru** — por desenho, *«melhor
/// transbordar visivelmente do que desaparecer»* —, e se o pintor recebesse essa largura como
/// orçamento pintaria o rótulo inteiro **por cima do campo numérico**. O `min` é o que faz o
/// orçamento voltar a ser a coluna, e o pintor recusar.
///
/// ⛔ Sem este teste a mutação `largura.min(col_w)` → `largura` **sobrevive**: nenhuma das duas
/// metades acima entra nesta faixa (as duas medem colunas largas), e a porta
/// [`ph2d_editor_core::widget::property_label_origin`] só responde pelo `x`, nunca pela tinta.
#[test]
fn a_column_narrower_than_the_ellipsis_paints_nothing() {
    let mut ts = TextSystem::without_system_fonts();
    let fonte = 13.0_f32;
    let reticencia = ts.prefix_width("…", fonte);
    assert!(
        reticencia > 1.0,
        "a reticencia mede {reticencia} — a fixtura perdeu o fenomeno"
    );
    let mut medidos = 0usize;
    for texto in ROTULOS {
        for fraccao in [0.1_f32, 0.5, 0.9] {
            let col_w = reticencia * fraccao;
            assert_eq!(
                pintado(&mut ts, texto, 100.0, col_w, fonte),
                0,
                "{texto:?} numa coluna de {col_w:.2} px (a reticencia mede {reticencia:.2}): \
                 o rotulo foi pintado e invade o controlo"
            );
            medidos += 1;
        }
    }
    assert_eq!(medidos, ROTULOS.len() * 3);
}

/// ⭐⭐ **O CONTROLO da régua acima** — sem ele, um contador de glifos que nunca vê uma reticência
/// passaria sobre um pintor que não pinta nada.
///
/// ⚠️ *Uma fixtura sem o fenómeno mede silêncio* (a lição do `what_fits_comes_back_untouched`):
/// aqui a coluna é apertada de propósito e o gate exige que a contagem **mude**.
#[test]
fn the_glyph_ruler_can_see_an_ellipsis_when_there_is_one() {
    let mut ts = TextSystem::without_system_fonts();
    let fonte = 13.0_f32;
    let mut vistos = 0usize;
    for texto in ROTULOS {
        let esperado = inteiro(&mut ts, texto, fonte);
        let apertada = ts.prefix_width(texto, fonte) * 0.6;
        assert_ne!(
            pintado(&mut ts, texto, 100.0, apertada, fonte),
            esperado,
            "{texto:?}: a coluna a 60 % nao cortou — a regua de glifos nao ve a reticencia"
        );
        vistos += 1;
    }
    assert_eq!(vistos, ROTULOS.len());
}
