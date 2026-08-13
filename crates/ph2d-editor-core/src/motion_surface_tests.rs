//! **A SUPERFÍCIE que o dedo COMANDA** — os gates da rolagem suave (E2) e do [`super::Role::Surface`].
//!
//! Irmão do `motion_tests.rs` pela mesma linha que separa os dois assuntos: aquele prova o que o
//! **substrato É** (neutro quando vazio, herda velocidade, poda, os dois eixos); este prova o que
//! **UM consumidor lhe pede** — e o consumidor aqui é o único cujo lugar o utilizador nomeia
//! directamente, o que lhe dá uma lei própria (nunca ultrapassar).

use super::*;

/// Os ids desta suíte: o substrato não olha para o valor.
fn id(n: u64) -> NodeId {
    NodeId(n + 1)
}

// ─────────────────────────────────────────────────────────────────────────────
// A ROLAGEM SUAVE (E2) — a roda mexe um ALVO, a superfície desliza até lá.
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐ **A roda acumula no ALVO, e é isso que faz girar depressa cobrir a distância toda.**
///
/// Somando ao valor EM VOO, cada nova volta parte de uma posição que ainda não chegou e a
/// superfície anda **menos** do que o dedo pediu — o defeito clássico de toda rolagem suave
/// escrita à pressa. *Mutação: a roda a ler `panel_scroll` (vivo) ⇒ o total encolhe.*
#[test]
fn spinning_the_wheel_fast_covers_the_whole_distance() {
    use crate::interaction::WidgetStore;
    let mut store = WidgetStore::with_capacity(4);
    let p = NodeId(7);
    // ⚠️ **A fixture TEM de conter o fenómeno.** Sem um valor VIVO atrasado, `panel_scroll` cai no
    //    alvo e ler um ou outro é a mesma coisa — a primeira versão deste gate ficou VERDE com a
    //    roda a ler o vivo, porque não havia vivo. Aqui a superfície fica sempre 40% para trás, que
    //    é o que a mola faz entre duas voltas de roda seguidas.
    for _ in 0..5 {
        let cur = store.panel_scroll_target(p);
        store.set_panel_scroll(p, cur + 100.0);
        let t = store.panel_scroll_target(p);
        store.set_panel_scroll_live(p, t * 0.6);
    }
    assert!(
        (store.panel_scroll_target(p) - 500.0).abs() < 1e-3,
        "o alvo tem de somar as cinco voltas, e somou {}",
        store.panel_scroll_target(p)
    );
}

/// ⚠️ **Sem substrato, `panel_scroll` É o alvo — o mundo pré-wave, byte-idêntico.**
///
/// É a neutralidade que torna esta wave segura: um store que ninguém tiquou desenha exactamente
/// onde desenhava. *Mutação: o `or_else` a devolver 0.0 ⇒ toda superfície salta para o topo.*
#[test]
fn an_untocked_store_paints_where_it_always_painted() {
    use crate::interaction::WidgetStore;
    let mut store = WidgetStore::with_capacity(4);
    let p = NodeId(7);
    store.set_panel_scroll(p, 240.0);
    assert!((store.panel_scroll(p) - 240.0).abs() < 1e-3);
}

/// **Quanto uma superfície ULTRAPASSA o sítio para onde a roda a mandou, e quanto demora.**
///
/// ⚠️ A sonda que nomeou o report *«o balanço das labels ficou bem artificial e pouco suave»*: o
/// `~/.ph2d/prefs.txt` do Enio diz `motion_character=expressive`, e ali o `Role::Travel` **passa do
/// alvo e volta**. Numa entrada de cartão isso é o carácter; numa superfície é a régua a mentir.
#[test]
#[ignore = "sonda: imprime o que cada Role faz a uma volta de roda"]
fn probe_the_scroll_glide() {
    fn run(c: UiCharacter, role: Role) -> (f32, f64) {
        const TARGET: f32 = 200.0;
        let dt = 1.0 / 60.0;
        let (mut m, a) = (UiMotion::default(), id(1));
        m.set_character(c);
        m.animate(a, 0.0, role);
        // ⚠️ O assentamento é o ÚLTIMO quadro fora da tolerância, +1 — nunca o primeiro DENTRO
        //    dela: quem ultrapassa **atravessa** o alvo na volta, e uma condição de paragem
        //    ingénua declara-o assente a meio do balanço (a 1ª versão desta sonda imprimiu
        //    `3,000 s` para o Expressivo, que é o laço a esgotar-se, não uma medição).
        let (mut peak, mut last_out) = (0.0_f32, 0usize);
        for f in 0..180 {
            let v = m.animate(a, TARGET, role);
            peak = peak.max(v);
            if (v - TARGET).abs() >= 1.0 {
                last_out = f;
            }
            m.advance(dt);
        }
        #[allow(clippy::cast_precision_loss)]
        (peak - TARGET, (last_out + 1) as f64 * dt)
    }
    println!("\n caracter   | role    | ultrapassa | assenta");
    println!("------------|---------|------------|--------");
    for c in [UiCharacter::Discrete, UiCharacter::Expressive] {
        for role in [Role::Travel, Role::Surface] {
            let (over, secs) = run(c, role);
            println!(
                " {:<10} | {role:<7?} | {over:7.2} px | {secs:.3} s",
                c.wire()
            );
        }
    }
}

/// ⭐ **Uma SUPERFÍCIE nunca passa do sítio que a roda nomeou — em carácter NENHUM.**
///
/// Medido pela [`probe_the_scroll_glide`], numa volta de 200 px:
///
/// | carácter | role | ultrapassa | assenta |
/// |---|---|---|---|
/// | discreto | Travel | 0,00 px | 0,217 s |
/// | **expressivo** | **Travel** | **31,08 px** | **0,500 s** |
/// | expressivo | **Surface** | **0,00 px** | **0,217 s** |
///
/// Os 31,08 px são os 15,5% do Expressivo, e são o *«balanço … artificial»* do report; os 0,500 s
/// contra 0,217 são o *«pouco suave»*. **Uma troca de `Role` responde às duas metades.**
///
/// *Mutação: `Role::Surface` a cair no braço do carácter ⇒ 31,08 px ⇒ sangra no Expressivo.*
#[test]
fn a_surface_never_passes_the_place_the_wheel_named() {
    const TARGET: f32 = 200.0;
    for c in [UiCharacter::Discrete, UiCharacter::Expressive] {
        let (mut m, a) = (UiMotion::default(), id(1));
        m.set_character(c);
        m.animate(a, 0.0, Role::Surface);
        let mut peak = 0.0_f32;
        for _ in 0..120 {
            peak = peak.max(m.animate(a, TARGET, Role::Surface));
            m.advance(1.0 / 60.0);
        }
        assert!(
            peak <= TARGET + 0.01,
            "{}: a superfície passou {:.2} px do alvo",
            c.wire(),
            peak - TARGET
        );
        assert!(
            m.get(a).is_some_and(|v| (v - TARGET).abs() < 1.0),
            "{}: e tem de CHEGAR — uma superfície que não chega é pior que uma que balança",
            c.wire()
        );
    }
}

/// ⚠️ **O CONTROLE, e sem ele o gate acima é satisfeito por castrar o Expressivo.**
///
/// A ultrapassagem **é** o carácter onde ele é o produto: o cartão da paleta passa do sítio e volta,
/// e foi assim que o Enio o aprovou. O que a wave da superfície afirma é que os dois papéis
/// respondem ao carácter de maneiras diferentes — não que o carácter deixou de existir.
#[test]
fn the_character_still_overshoots_where_that_is_the_product() {
    let (mut m, a) = (UiMotion::default(), id(1));
    m.set_character(UiCharacter::Expressive);
    m.animate(a, 0.0, Role::Travel);
    let mut peak = 0.0_f32;
    for _ in 0..120 {
        peak = peak.max(m.animate(a, 1.0, Role::Travel));
        m.advance(1.0 / 60.0);
    }
    assert!(peak > 1.05, "o Expressivo ultrapassa, e mediu {peak:.3}");
}

/// E o *reduced motion* continua a levar a superfície — ela é PERCURSO, a classe que o interruptor
/// existe para matar. *Mutação: o braço `Role::Travel | Role::Surface if self.reduced` a perder o
/// `Surface` ⇒ a superfície desliza para quem pediu que nada deslizasse.*
#[test]
fn reduced_motion_still_takes_the_surface() {
    let mut m = UiMotion::default();
    m.set_reduced_motion(true);
    for c in [UiCharacter::Discrete, UiCharacter::Expressive] {
        m.set_character(c);
        assert!(m.law(Role::Surface).is_none(), "{}", c.wire());
    }
}

/// E o vivo, quando existe, é o que o pintor vê — enquanto o alvo fica onde a roda o pôs.
#[test]
fn the_painter_sees_the_live_offset_and_the_wheel_sees_the_target() {
    use crate::interaction::WidgetStore;
    let mut store = WidgetStore::with_capacity(4);
    let p = NodeId(7);
    store.set_panel_scroll(p, 300.0);
    store.set_panel_scroll_live(p, 120.0);
    assert!(
        (store.panel_scroll(p) - 120.0).abs() < 1e-3,
        "o pintor vê o vivo"
    );
    assert!(
        (store.panel_scroll_target(p) - 300.0).abs() < 1e-3,
        "a roda vê o alvo"
    );
}
