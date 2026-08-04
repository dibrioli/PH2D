//! **Quanto custa a camada de override** (plano UI/UX W6, degrau 1).
//!
//! ⚠️ O doc do `overrides.rs` afirma que o caminho comum custa *"uma leitura de bool"*, e isso é
//! uma afirmação que se MEDE. A sonda percorre a tabela inteira nos quatro modos, como um frame
//! percorre os widgets, e imprime o custo por `resolve` em três regimes.
//!
//! Rodar: `cargo test -p ph2d-tokens --release --test measure_override_layer -- --ignored
//! --nocapture`.

use std::time::Instant;

use ph2d_tokens::color::{Color, ColorToken};
use ph2d_tokens::overrides::{clear_color_overrides, set_color_override};
use ph2d_tokens::theme::Theme;

const THEMES: [Theme; 4] = [
    Theme::Forge,
    Theme::Workshop,
    Theme::Sunstone,
    Theme::Blueprint,
];
/// Passagens sobre a tabela inteira. Um frame faz uma fracção disto; o laço existe para o relógio
/// ter o que medir.
const PASSES: u32 = 200;

fn one_sweep() -> (f64, usize) {
    let t = Instant::now();
    let mut sink = 0u64;
    let mut n = 0usize;
    for _ in 0..PASSES {
        for theme in THEMES {
            for &token in ColorToken::ALL {
                let c = token.resolve(theme);
                sink = sink.wrapping_add(u64::from(c.r));
                n += 1;
            }
        }
    }
    assert!(sink > 0);
    (t.elapsed().as_secs_f64() * 1e9 / n as f64, n)
}

/// O **MÍNIMO** de três varreduras — o redutor certo quando toda amostra faz o mesmo trabalho, e
/// máquina carregada só sabe deixar mais lento.
///
/// ⚠️ A primeira versão desta sonda media UMA varredura por regime, e as razões saíam entre
/// **0,97× e 1,45×** entre corridas — uma delas *abaixo de 1*, que é impossível como custo real.
/// O número absoluto era estável (58-62 ns) e só as razões eram ruído: o sinal que se procurava é
/// **menor que o ruído da própria medida**, e é isso que o resultado diz.
fn sweep() -> (f64, usize) {
    let mut best = (f64::MAX, 0);
    for _ in 0..3 {
        let s = one_sweep();
        if s.0 < best.0 {
            best = s;
        }
    }
    best
}

#[test]
#[ignore = "sonda: imprime o custo por resolve"]
fn measure_the_cost_of_the_layer() {
    // Aquecimento — a primeira passagem paga o first-touch das tabelas geradas.
    let _ = sweep();

    clear_color_overrides();
    let (empty, n) = sweep();

    // **Um** override: o regime em que o artista acabou de mexer num token.
    set_color_override(
        Theme::Forge,
        ColorToken::Accent,
        Some(Color::from_hex(0x00FF00)),
    );
    let (one, _) = sweep();

    // **Vinte**: uma re-vestida de verdade.
    for (i, &token) in ColorToken::ALL.iter().take(20).enumerate() {
        set_color_override(
            Theme::Forge,
            token,
            Some(Color::from_hex(0x0010_0000 * (i as u32 + 1))),
        );
    }
    let (twenty, _) = sweep();
    clear_color_overrides();

    eprintln!("  resolves por passagem: {}", n / PASSES as usize);
    eprintln!("  camada VAZIA .......: {empty:.2} ns/resolve");
    eprintln!(
        "  UM override ........: {one:.2} ns/resolve  ({:.2}x)",
        one / empty
    );
    eprintln!(
        "  VINTE overrides ....: {twenty:.2} ns/resolve  ({:.2}x)",
        twenty / empty
    );
}
