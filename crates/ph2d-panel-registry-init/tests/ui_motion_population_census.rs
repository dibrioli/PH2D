//! **SONDA (`--ignored`): o custo de um quadro da UI viva com a POPULAÇÃO que o app tem.**
//!
//! ## Por que ela existe, e o que corrige numa afirmação minha
//!
//! O [`PLANO_UI_viva_2026-08-12.md`] §1.4 afirma **`O(vivos)`, com `vivos` ≈ 0-3**, e o doc do
//! `ph2d_editor_core::motion` nomeia **duas** grandezas — *integrar é `O(em voo)`* contra *lembrar
//! é `O(tocados recentemente)`*. As duas frases descrevem o mesmo laço, e **nenhuma delas governa
//! o custo**: o `UiMotion::advance` percorre `tracks` INTEIRO (e aloca um `Vec` do tamanho dele em
//! cada quadro), e o `tick_hover` chama `animate` para **todo widget de estado macio do store** —
//! não só para os que se movem. ⇒ o custo por quadro é `O(lembrado)`, e `lembrado` é a população
//! da UI, não o punhado que se mexe.
//!
//! ⚠️ **E a sonda irmã não consegue ver isto, por FIXTURE.** O
//! `measure_what_a_frame_of_ui_motion_costs` re-alveja tudo a cada quadro, então lá dentro
//! `lembrado == em voo`: as duas grandezas que o doc do módulo separa são, naquela fixture, **o
//! mesmo número**. Ela reporta *0 em voo ⇒ 0,020 µs* sobre um mapa **VAZIO**, e o app em repouso
//! tem 0 em voo sobre um mapa de milhares. Uma sonda cuja fixture colapsa os dois eixos não pode
//! dizer qual deles manda — a doença que este repositório já pagou no `soft_body`, no `boids` e no
//! censo do Wet Paint.
//!
//! ## O que esta sonda mede que a outra não pode
//!
//! Ela mede `HeroScreen::tick_motion` — a porta do PRODUTO — em quatro regimes, e o eixo que ela
//! varre é a **POPULAÇÃO**, nunca o número de coisas em voo:
//!
//! | regime | o que é |
//! |---|---|
//! | **chrome** | `HeroScreen::new` ANTES de registar painel nenhum — o piso da população |
//! | **⭐ repouso** | o app inteiro aberto, rato parado: **o regime em que ele vive** |
//! | hover | o mesmo, com widgets genuinamente em voo |
//! | irmã | tudo em voo — a fixture da outra sonda, para as duas serem comparáveis |
//!
//! ⚠️ **A ordem das duas primeiras linhas é load-bearing:** `HeroScreen::new` **já popula todos os
//! painéis do registo** (`pre_populate_store`), então um «controlo» construído depois do
//! `register_all_panels` é idêntico ao caso cheio — foi o que a primeira versão desta sonda mediu,
//! e ela reportou dois números iguais achando que comparava alguma coisa.
//!
//! ⚠️ **E os voadores têm de nascer FRIOS.** A lei do substrato é *a primeira vista de um id CHEGA
//! ao alvo*; registá-los já quentes faz o tique assentá-los no primeiro quadro e a coluna `em voo`
//! imprime **zero** — a segunda fixture desta sonda que não continha o fenómeno que ela mede.
//!
//! ⚠️ **Sonda, não gate.** Ela imprime e não afirma: quantos widgets um app tem é consequência de
//! quantos painéis existem, e um tecto aqui seria um número inventado. O que ela entrega é a coluna
//! `% de 16,6ms`, que é o que decide se há wave ou se há só uma nota a corrigir.
//!
//! Rode: `cargo test -p ph2d-panel-registry-init --release --test ui_motion_population_census -- --ignored --nocapture`

use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::widget::ButtonState;

const DT: f64 = 1.0 / 60.0;

/// Ids sintéticos para o regime «em voo», fora de qualquer intervalo que um painel use.
const FLYER_BASE: u64 = 0x00ff_0000_0000_0000;

/// Acrescenta `n` botões **frios**. Quem os põe em voo é o [`heat`], um quadro depois.
fn add_flyers(hero: &mut HeroScreen, n: u64) {
    for i in 0..n {
        hero.store.register(
            NodeId(FLYER_BASE + i),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// Acende os `n` voadores — só DEPOIS de o tique os ter visto frios é que isto é uma interrupção.
fn heat(hero: &mut HeroScreen, n: u64) {
    for i in 0..n {
        if let Some(InteractiveState::Button { state }) = hero.store.get_mut(NodeId(FLYER_BASE + i))
        {
            *state = ButtonState::Hovered;
        }
    }
}

/// A mediana de `tick_motion` sobre 200 quadros, com o aquecimento fora.
///
/// ⚠️ **Mediana e não mínimo.** O primeiro quadro instala o mapa inteiro e é estruturalmente
/// diferente dos outros — exactamente a armadilha que o censo do Wet Paint pagou (*«o mínimo era a
/// amostra SEM o fenómeno»*).
///
/// ⚠️ **Re-acende a cada quadro**, senão os voadores assentam a meio da medição e as últimas
/// amostras medem o regime de repouso com o rótulo do de voo.
fn median_us(hero: &mut HeroScreen, flyers: u64) -> (f64, usize, usize) {
    hero.tick_motion(DT);
    heat(hero, flyers);
    let mut ms = Vec::with_capacity(200);
    let (mut worst_flight, mut remembered) = (0usize, 0usize);
    for f in 0..200 {
        // Alterna o alvo para os manter em voo durante a medição inteira.
        if flyers > 0 {
            for i in 0..flyers {
                if let Some(InteractiveState::Button { state }) =
                    hero.store.get_mut(NodeId(FLYER_BASE + i))
                {
                    *state = if f % 2 == 0 {
                        ButtonState::Hovered
                    } else {
                        ButtonState::Normal
                    };
                }
            }
        }
        let t0 = std::time::Instant::now();
        hero.tick_motion(DT);
        ms.push(t0.elapsed().as_secs_f64() * 1e6);
        worst_flight = worst_flight.max(hero.motion.in_flight());
        remembered = hero.motion.remembered();
    }
    ms.sort_by(f64::total_cmp);
    (ms[ms.len() / 2], remembered, worst_flight)
}

fn row(label: &str, hero: &mut HeroScreen, flyers: u64) {
    let (us, remembered, in_flight) = median_us(hero, flyers);
    println!(
        "[ui-pop] {label:<10} {remembered:>10} {in_flight:>8} {us:>11.3}us {:>12.4}",
        us / 16_600.0 * 100.0
    );
}

#[test]
#[ignore = "sonda: rode com --release -- --ignored --nocapture"]
fn census_of_what_a_live_ui_frame_costs_with_the_population_the_app_has() {
    // ⚠️ ANTES do registo: é a única janela em que o piso da população é observável.
    let mut chrome = HeroScreen::new(NodeId(1));

    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut rest = HeroScreen::new(NodeId(1));
    let soft = rest.store.hover_targets().count();

    println!("[ui-pop] o tique da UI viva, pela porta do produto (`HeroScreen::tick_motion`)");
    println!("[ui-pop] widgets de estado MACIO no store do app inteiro: {soft}");
    println!(
        "[ui-pop] {:<10} {:>10} {:>8} {:>12} {:>13}",
        "regime", "lembrado", "em voo", "us/quadro", "% de 16,6ms"
    );

    row("chrome", &mut chrome, 0);
    row("repouso", &mut rest, 0);

    let mut hover = HeroScreen::new(NodeId(1));
    add_flyers(&mut hover, 3);
    row("hover", &mut hover, 3);

    let mut all = HeroScreen::new(NodeId(1));
    add_flyers(&mut all, 400);
    row("irma", &mut all, 400);

    println!(
        "[ui-pop] ⚠️ o eixo que manda e' LEMBRADO, nao EM VOO: o `advance` percorre `tracks` \
         inteiro e o `tick_hover` alveja todo widget macio do store"
    );
}
