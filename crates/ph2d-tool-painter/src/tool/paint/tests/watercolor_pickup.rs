//! **O item 4 — o pincel gasto RECOLHE a própria tinta molhada ao voltar** (doc 40 §S2-C; ordem do
//! dono 2026-09-20: *«implemente o Item 4 como opção extra e não como substituto»*).
//!
//! ## A régua que decide, e porque ela é DUAS fixturas e não uma
//!
//! O risco desta feature não é ela não funcionar — é ela funcionar **de mais**: com o espaçamento
//! de fábrica (~10 % do raio) o dab `i−1` já cobriu ~95 % do disco do dab `i`, logo um pickup sem
//! cerca reabastece o pincel do próprio rasto e **o Charge deixa de gastar** (doc 40 §S2-C). A
//! cerca é a IDADE: o arco da PRIMEIRA cobertura de cada texel.
//!
//! ⭐ **Isso dá um par de fixturas que discrimina sozinho:**
//!
//! | gesto | a cerca abre? | o que o gate exige |
//! |---|---|---|
//! | traço **RECTO** | **não** — o texel mais velho sob o disco tem ~meio diâmetro de idade | saída **AO BIT** igual à de pickup `0` |
//! | traço em **U** que volta sobre si | **sim** — a perna de ida foi coberta centenas de px atrás | a volta sai **mais carregada** |
//!
//! *Uma fixtura só não separa «recolhe» de «reabastece-se»: o recto é o CONTROLO que prova que a
//! cerca existe, e o U é a metade positiva que prova que ela abre onde tem de abrir.*

use super::*;
// A fixtura em U e o arnes vivem no irmao — o item 4 mede-se no MESMO gesto do report.
use super::watercolor_selfseam::{SeamKnobs, UStroke, paint_u};

/// O traço RECTO: a mesma geometria do U, mas sem a volta — as pernas não se sobrepõem.
fn straight(r: f32, k: SeamKnobs) -> PainterTool {
    let u = UStroke::new(r);
    let line = UStroke {
        y1: 91.0,
        xb: u.xa + 260.0,
        ..u
    };
    paint_u(line, k)
}

fn worst_byte(a: &PainterTool, b: &PainterTool) -> i32 {
    a.canvas_rgba
        .iter()
        .zip(b.canvas_rgba.iter())
        .map(|(x, y)| (i32::from(*x) - i32::from(*y)).abs())
        .max()
        .unwrap_or(0)
}

/// **O CONTROLO, e a razão de a feature poder existir: um traço RECTO não se reabastece.**
///
/// ⛔ Se este gate ficar vermelho, o pickup está a ler o rasto imediato do próprio pincel e o
/// Charge deixou de gastar — que é exactamente o modo de falha que o doc 40 §S2-C nomeia como *«self
/// feeding literal»*. ⚠️ A barra é **ZERO**: não há tolerância a dar, porque a cerca ou está lá ou
/// não está.
#[test]
fn a_straight_stroke_never_feeds_on_its_own_trail() {
    let bare = SeamKnobs::default();
    let (off, on) = (
        straight(32.0, bare),
        straight(
            32.0,
            SeamKnobs {
                pickup: 1.0,
                ..bare
            },
        ),
    );
    let d = worst_byte(&off, &on);
    // CONTROLO POSITIVO: a fixtura tem de estar a pintar alguma coisa, senão isto é vácuo.
    let painted = off.canvas_rgba.chunks(4).filter(|p| p[1] < 250).count();
    assert!(painted > 5_000, "piso de população: {painted} px pintados");
    assert_eq!(
        d, 0,
        "o pickup mexeu num traço RECTO (Δ{d}): a cerca de idade não está a segurar"
    );
}

/// **A metade positiva: a volta sobre a perna de ida sai MAIS CARREGADA.**
///
/// ⚠️ A régua é a tinta na perna de VOLTA (`xb`), não a costura: o item 4 é sobre o que o pincel
/// leva, e o que ele leva aparece como pigmento a mais onde ele já estava gasto.
#[test]
fn the_return_leg_picks_up_the_pigment_it_crosses() {
    let bare = SeamKnobs::default();
    let (off, on) = (
        paint_u(UStroke::new(32.0), bare),
        paint_u(
            UStroke::new(32.0),
            SeamKnobs {
                pickup: 1.0,
                ..bare
            },
        ),
    );
    let u = UStroke::new(32.0);
    // A meia altura da perna de VOLTA, onde a carga já está gasta.
    let y = ((u.y0 + u.y1) * 0.5) as u32;
    let x = u.xb as u32;
    let (g_off, g_on) = (
        f32::from(px(&off, u.size, x, y)[1]),
        f32::from(px(&on, u.size, x, y)[1]),
    );
    assert!(
        g_on < g_off - 2.0,
        "a volta devia sair mais carregada: G {g_on} com pickup contra {g_off} sem"
    );
}

/// **Não-substituto: com o knob em `0` o traço é BYTE-IDÊNTICO ao de antes da feature.**
///
/// ⚠️ Este é o gate que a ordem do dono pede por escrito (*«opção extra e não substituto»*), e ele
/// mede o gesto que a feature MAIS muda — o U — para não ser vácuo.
#[test]
fn with_the_knob_at_zero_nothing_moves() {
    let bare = SeamKnobs::default();
    let a = paint_u(UStroke::new(32.0), bare);
    let b = paint_u(
        UStroke::new(32.0),
        SeamKnobs {
            pickup: 0.0,
            ..bare
        },
    );
    assert_eq!(worst_byte(&a, &b), 0, "o caminho de fábrica mexeu-se");
    // E o CONTROLO de que a fixtura contém o fenómeno: a `1` ela MUDA.
    let on = paint_u(
        UStroke::new(32.0),
        SeamKnobs {
            pickup: 1.0,
            ..bare
        },
    );
    assert!(
        worst_byte(&a, &on) > 2,
        "a fixtura não contém o fenómeno: pickup 1 não mudou nada"
    );
}
