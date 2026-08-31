//! ⭐⭐⭐ **A OPACIDADE ALCANÇA AS CÓPIAS do pincel** (plano 36, W6) — os gates do motor.
//!
//! # A lei, numa frase: *uma opacidade, uma casa* — e a casa de um pincel é a alfa da sua cor de
//! recurso
//!
//! Cada espécie de tinta tem **um** sítio onde a opacidade vive, e nunca dois:
//!
//! | tinta | a casa | quem a lê |
//! |---|---|---|
//! | `Solid` | a alfa da cor | o desenho, directamente |
//! | `Pattern` | `PatternFill::alpha` (`f32`), com `fallback.a` em sincronia | o amostrador do Vello |
//! | **`Brush`** | **`fallback.a`**, e mais nada | [`brush_copies`], que a aplica às cópias |
//!
//! ⚠️ **Por que o pincel NÃO ganha um campo `alpha` próprio, ao contrário do padrão.** O `alpha` do
//! padrão existe porque o amostrador quer um `f32` — a `fallback` é `Rgba8` e serve o instante
//! pré-resolução. Um pincel não tem amostrador: as cópias são **geometria com a tinta delas**, e
//! desvanecê-las é a mesma operação `Rgba8` que a `fallback` já sofre. ⇒ um campo novo seria uma
//! **segunda casa** para o mesmo número, com o preço do `VEC_SCENE_SCHEMA_VERSION` a subir e o
//! benefício de nada. *A assimetria com o padrão é a resposta certa, e está escrita aqui para não
//! ser "arrumada" depois.*
//!
//! ⭐⭐ **E foi por isso que a dívida declarada no [`crate::paint_bind`] fechou sem uma linha lá:**
//! ela dizia *"um pincel desvanece a cor de recurso, e as CÓPIAS ainda não"*, e escalar a
//! `fallback` passou a **ser** escalar as cópias. *Quando a cura de um buraco apaga o comentário
//! que o declarava sem lhe tocar, o desenho está no sítio certo.*

use super::fixtures::*;
use super::*;
use crate::{Paint, Rgba8, StrokeSpec};

/// A arte **pintada** — sem isto os gates desta folha não têm sujeito: a [`arte`] partilhada nasce
/// sem `fill` e sem `stroke`, e desvanecer uma forma sem tinta nenhuma é um no-op que se lê como
/// aprovado ([[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]]).
fn arte_pintada(fill_a: u8, stroke_a: u8) -> VecPath {
    let mut p = arte(1.0, 1.0);
    p.fill = Some(Paint::Solid(Rgba8::new(200, 100, 50, fill_a)));
    p.stroke = Some(StrokeSpec::new(Rgba8::new(10, 20, 30, stroke_a), 0.05));
    p
}

/// O pincel com a cor de recurso a `a` de alfa — que é onde a barra *Opacity* escreve.
fn pincel_a(a: u8) -> BrushStroke {
    BrushStroke {
        fallback: Rgba8::new(1, 2, 3, a),
        ..pincel()
    }
}

/// As alfas de preenchimento das cópias que o produto emite.
fn alfas_de_fill(copias: &[VecPath]) -> Vec<u8> {
    copias
        .iter()
        .map(|c| match c.fill.as_ref() {
            Some(Paint::Solid(k)) => k.a,
            _ => panic!("a copia perdeu o preenchimento da arte"),
        })
        .collect()
}

/// ⭐⭐⭐ **A BARRA *OPACITY* ALCANÇA AS CÓPIAS.** O buraco inteiro, numa afirmação.
///
/// Até 2026-08-30 a barra escrevia a alfa na `fallback` — e a `fallback` só se vê **enquanto a
/// arte não resolve**. Com arte, o artista arrastava a barra de ponta a ponta e não mudava um
/// pixel: o segundo tipo de knob morto do `CLAUDE.md` §5, *o consumidor que projecta o valor fora*.
/// ⛔ Nenhuma sonda de *"quem lê este campo?"* o via: ele **era** lido, para pintar a cor de
/// recurso.
#[test]
fn the_opacity_slider_reaches_the_brush_copies() {
    let copias = brush_along_path(
        &quadrado(4.0),
        &[arte_pintada(255, 255)],
        &traco(&pincel_a(128), 1.0, None),
    );
    assert!(!copias.is_empty(), "sem copias nao ha' o que medir");
    for a in alfas_de_fill(&copias) {
        assert_eq!(
            a, 128,
            "a copia saiu OPACA com a barra a meio - a barra anda e nao muda um pixel"
        );
    }
}

/// ⭐⭐ **CONTROLO: opaca deixa a arte EXACTAMENTE como ela é.**
///
/// Sem isto o gate acima ficaria verde sobre uma porta que escurece tudo por sistema. ⚠️ E a
/// afirmação é **byte-a-byte** e não "parecido": `255 * 255 + 127) / 255` é `255`, então a conta de
/// desvanecimento é a identidade no topo — e é por isso que ela pode correr **sem guarda**, em vez
/// de um `if` que teria de ser mantido em sincronia com a conta.
#[test]
fn a_fully_opaque_brush_leaves_the_art_byte_identical() {
    let art = arte_pintada(255, 200);
    let opaco = brush_along_path(
        &quadrado(4.0),
        std::slice::from_ref(&art),
        &traco(&pincel_a(255), 1.0, None),
    );
    assert!(!opaco.is_empty());
    for c in &opaco {
        assert_eq!(
            c.fill, art.fill,
            "o preenchimento mudou com a barra no topo"
        );
        assert_eq!(
            c.stroke.as_ref().map(StrokeSpec::color),
            art.stroke.as_ref().map(StrokeSpec::color),
            "o traco da arte mudou com a barra no topo"
        );
    }
}

/// ⭐⭐ **A opacidade MULTIPLICA a da arte, nunca a substitui** — a mesma lei que as paradas de um
/// gradiente já obedecem no [`crate::paint_bind::fade`].
///
/// Uma arte autorada a `200` de alfa sob uma barra a `128` sai a `round(200·128/255) = 100`.
/// ⛔ Substituir daria `128` e **subiria** a opacidade de uma arte que o artista fez translúcida —
/// arrastar a barra para baixo tornaria a arte mais opaca, que é o contrário do rótulo.
#[test]
fn the_opacity_scales_the_arts_own_alpha_it_does_not_replace_it() {
    let copias = brush_along_path(
        &quadrado(4.0),
        &[arte_pintada(200, 255)],
        &traco(&pincel_a(128), 1.0, None),
    );
    for a in alfas_de_fill(&copias) {
        assert_eq!(
            a, 100,
            "a alfa autorada da arte foi SUBSTITUIDA em vez de escalada"
        );
    }
}

/// ⚠️ **O TRAÇO da arte desvanece com o preenchimento dela.** Uma cópia que perdesse metade do
/// preenchimento e mantivesse o contorno a cheio desenharia uma silhueta a saltar à vista.
#[test]
fn the_arts_own_stroke_fades_with_its_fill() {
    let copias = brush_along_path(
        &quadrado(4.0),
        &[arte_pintada(255, 255)],
        &traco(&pincel_a(64), 1.0, None),
    );
    assert!(!copias.is_empty());
    for c in &copias {
        assert_eq!(
            c.stroke.as_ref().map(|s| s.color().a),
            Some(64),
            "o contorno da arte ficou a cheio enquanto o preenchimento desvaneceu"
        );
    }
}

/// ⭐⭐⭐ **A SOBREPOSIÇÃO VIVA (tokens / opacidade de vista) chega às cópias PELA MESMA PORTA.**
///
/// [`VecPath::painted`] desvanece a `fallback` do pincel; o motor lê a `fallback`; ⇒ as cópias
/// desvanecem. É a dívida que o [`crate::paint_bind`] declarava, fechada **sem uma linha lá**.
///
/// ⚠️ **A composição é multiplicativa e tem de o ser:** uma forma autorada a meia opacidade sob um
/// desvanecimento de vista a meio sai a um quarto. Somar, ou deixar a vista ganhar, faria a
/// opacidade autorada evaporar assim que alguém tocasse no slider de vista.
#[test]
fn the_live_fade_reaches_the_copies_through_the_fallback() {
    let art = arte_pintada(255, 255);
    let mut forma = quadrado(4.0);
    forma.stroke = Some(traco(&pincel_a(128), 1.0, None));
    let bound = crate::paint_bind::BoundStyle {
        path: forma.id,
        alpha: Some(128),
        ..crate::paint_bind::BoundStyle::default()
    };
    let vista = forma.painted(Some(&bound));
    let copias = brush_along_path(
        &vista,
        std::slice::from_ref(&art),
        vista.stroke.as_ref().expect("o traco sobrevive"),
    );
    assert!(!copias.is_empty());
    for a in alfas_de_fill(&copias) {
        // 128 (autorada) x 128 (vista) = 64 a menos de arredondamento.
        assert_eq!(a, 64, "a opacidade de vista nao compos com a autorada");
    }
    // CONTROLO: sem sobreposição, a autorada sozinha manda.
    let sem = brush_along_path(
        &forma,
        std::slice::from_ref(&art),
        forma.stroke.as_ref().unwrap(),
    );
    for a in alfas_de_fill(&sem) {
        assert_eq!(a, 128);
    }
}
