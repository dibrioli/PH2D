//! **A GRADE DE PIXELS** — os gates de [`super::on_pixel_grid`].
//!
//! ⚠️ **O oráculo aqui NÃO é *«o número publicado é inteiro»*.** Isso é o mecanismo, e um gate que
//! o afirma passa a proibir a cura no dia em que ela mudar de forma. O que o artista vê é a label
//! a **desencontrar-se da própria linha**, então é isso que se mede: o offset entre o retângulo
//! (contínuo, como o Vello o recebe) e a baseline do texto (arredondada, como o `paint_text` a
//! encaixa) tem de ficar **constante** durante a rolagem inteira.
//!
//! Um gate escrito assim falha por **qualquer** caminho que reintroduza o tremor — publicar cru,
//! arredondar só metade dos consumidores, ou um pintor futuro que decida deslocar-se por conta.

use super::*;
use crate::screens::hero::HeroScreen;

const DT: f64 = 1.0 / 60.0;

/// A ordenada em que uma linha de painel desenha o seu retângulo, dado o scroll publicado.
///
/// ⚠️ **O topo é FRACCIONÁRIO de propósito** — layout real sai de somas de tokens e paddings, e
/// um topo inteiro tornaria o gate verde por vácuo (com tudo alinhado, arredondar não faz nada).
const ROW_TOP: f32 = 12.37;

fn scroll_a_panel(target: f32, frames: usize) -> Vec<f32> {
    let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
    hero.motion.set_character(UiCharacter::Expressive);
    let panel = ph2d_a11y::NodeId(4242);
    hero.store.set_panel_scroll(panel, 0.0);
    hero.tick_motion(DT);
    hero.store.set_panel_scroll(panel, target);
    (0..frames)
        .map(|_| {
            hero.tick_motion(DT);
            hero.store.panel_scroll(panel)
        })
        .collect()
}

/// ⭐ **A label anda no MESMO passo que a linha em que mora.**
///
/// Este é o report do Enio (*«as labels ainda têm um movimento incómodo ao rolar os painéis»*)
/// escrito como propriedade. Medido antes da cura: a label afastava-se **0,481 px**, o passo dela
/// desencontrava-se do da linha em **0,820 px**, e ela ficava **parada 3 quadros seguidos** com a
/// linha a deslizar por baixo.
///
/// *Mutação: publicar `live` cru em vez de `on_pixel_grid(live)` ⇒ o offset varia ~0,5 px e este
/// gate sangra.*
#[test]
fn a_label_moves_in_lockstep_with_the_row_it_lives_in() {
    let scrolls = scroll_a_panel(40.0, 60);
    assert!(scrolls.len() == 60, "a fixture tem de conter o percurso");

    // O CONTROLO: a superfície tem mesmo de se mover, senão "constante" é trivialmente verdade.
    let travelled = scrolls.last().unwrap() - scrolls.first().unwrap();
    assert!(
        travelled > 30.0,
        "a fixture não contém o fenómeno — a superfície andou só {travelled:.2} px"
    );

    let offsets: Vec<f32> = scrolls
        .iter()
        .map(|s| {
            let row_y = ROW_TOP - s; // o retângulo, como o Vello o recebe
            let label_y = row_y.round(); // a baseline, como o `paint_text` a encaixa
            label_y - row_y
        })
        .collect();

    let (lo, hi) = offsets
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), o| (lo.min(*o), hi.max(*o)));
    assert!(
        hi - lo < 1e-3,
        "a label desencontra-se da linha em {:.3} px durante a rolagem \
         (offsets de {lo:.3} a {hi:.3}) — é isto que se lê como tremor",
        hi - lo
    );
}

/// E a superfície **CHEGA**: arredondar no publicar não pode deixá-la a um pixel do alvo para
/// sempre.
///
/// ⚠️ **GUARDA DE REGRESSÃO, e a distinção é honesta: não encontrei mutação que o mate sozinho.**
/// A candidata óbvia — arredondar DENTRO da [`super::Track::value`] — foi armada e **não o toca**
/// (o `SpringState` é independente do valor devolvido, então quantizar a saída não realimenta a
/// mola); ela mata **quatro** gates do substrato, entre eles o
/// `the_character_still_overshoots_where_that_is_the_product`. Este gate vale pelo que apanha no
/// futuro — uma cura que faça a superfície parar a um pixel do sítio para onde a roda a mandou —,
/// não por uma mutação de hoje.
#[test]
fn rounding_the_published_value_does_not_stall_the_surface() {
    let scrolls = scroll_a_panel(40.0, 120);
    let landed = *scrolls.last().unwrap();
    assert!(
        (landed - 40.0).abs() < f32::EPSILON,
        "a superfície tinha de pousar exatamente no alvo e parou em {landed}"
    );
}

/// ⚠️ **Um toque de TRACKPAD acumula EXATO — no alvo, que ninguém arredonda.**
///
/// É a razão de a grade morar no publicar e não no acumulador: dez nudges de 0,3 px têm de somar
/// 3,0 px. Arredondados na entrada somariam **zero**, e o painel não responderia a um trackpad.
///
/// *Mutação: a roda a acumular sobre `panel_scroll` (o vivo, arredondado) em vez de
/// `panel_scroll_target` ⇒ o total colapsa e este gate sangra.*
#[test]
fn a_trackpad_nudge_accumulates_exactly_in_the_target() {
    use crate::interaction::WidgetStore;
    let mut store = WidgetStore::with_capacity(4);
    let p = ph2d_a11y::NodeId(7);
    for _ in 0..10 {
        let cur = store.panel_scroll_target(p);
        store.set_panel_scroll(p, cur + 0.3);
        // o tique publica o vivo já na grade — e não pode contaminar o alvo
        store.set_panel_scroll_live(p, on_pixel_grid(store.panel_scroll_target(p)));
    }
    let target = store.panel_scroll_target(p);
    assert!(
        (target - 3.0).abs() < 1e-4,
        "dez toques de 0,3 px têm de somar 3,0 e somaram {target}"
    );
}

/// O cartão da cascata **translada e leva a própria label** ⇒ mesma lei, mesmo pouso.
/// *Mutação: `cascade_rise` a devolver o valor cru ⇒ sangra.*
#[test]
fn the_cascade_card_lands_on_the_grid_because_it_carries_its_label() {
    let mut saw_fraction = false;
    for i in 0..=20 {
        let t = i as f32 / 20.0;
        let rise = cascade_rise(t, true);
        assert!(
            (rise - rise.round()).abs() < f32::EPSILON,
            "o cartão pousou em {rise} px, fora da grade"
        );
        // CONTROLO: sem a grade, esta varredura produziria fracções — se ela não as produzisse,
        // o gate acima seria verde por vácuo.
        let raw = CASCADE_RISE_PX * (1.0 - t);
        saw_fraction |= (raw - raw.round()).abs() > 1e-3;
    }
    assert!(
        saw_fraction,
        "a varredura não contém um valor fraccionário — este gate não pode falhar"
    );
}

/// ⚠️ **E o crescimento de um chip NÃO pousa na grade** — a isenção, pinada para ninguém
/// «completar» a wave. O [`hover_lift`] cresce o retângulo por igual nos quatro lados, então o
/// CENTRO fica onde está e o glifo centrado não se move; quantizá-lo só tornaria o crescimento
/// aos degraus, sem texto nenhum a ganhar com isso.
#[test]
fn a_growing_chip_is_exempt_because_its_centre_never_moves() {
    let r = crate::zones::Rect::new(10.0, 20.0, 36.0, 36.0);
    let mut saw_fraction = false;
    for i in 0..=10 {
        let t = i as f32 / 10.0;
        let g = hover_lift(r, t, true);
        let cx = g.x + g.w * 0.5;
        let cy = g.y + g.h * 0.5;
        assert!(
            (cx - 28.0).abs() < 1e-4 && (cy - 38.0).abs() < 1e-4,
            "o centro do chip andou para ({cx}, {cy}) — o glifo passaria a viajar"
        );
        saw_fraction |= (g.x - g.x.round()).abs() > 1e-3;
    }
    assert!(
        saw_fraction,
        "o chip nunca pousou fora da grade — a isenção que este gate descreve não existe"
    );
}
