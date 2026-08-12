//! SONDA (`--ignored`): **quanto tempo cada carácter demora, e quanto custa integrá-lo.**
//!
//! Duas perguntas que o plano deixou por medir, e o número de uma delas está **cravado numa
//! constante do produto** (`DISCRETE.stiffness`) — logo esta sonda não é curiosidade, é a fonte.
//!
//! Rode: `cargo test -p ph2d-editor-core --release --test measure_ui_motion -- --ignored --nocapture`

use ph2d_a11y::NodeId;
use ph2d_editor_core::motion::{Role, UiCharacter, UiMotion};

const DT: f64 = 1.0 / 60.0;

/// Quantos segundos até a mola assentar, e qual o pico — dirigido pela porta do PRODUTO
/// (`animate` + `advance`), nunca por um integrador próprio da sonda.
fn settle(character: UiCharacter, role: Role) -> (f64, f32) {
    let mut m = UiMotion::default();
    m.set_character(character);
    let a = NodeId(1);
    m.animate(a, 0.0, role);
    m.animate(a, 1.0, role);
    let (mut t, mut peak) = (0.0_f64, 0.0_f32);
    for _ in 0..600 {
        m.advance(DT);
        peak = peak.max(m.animate(a, 1.0, role));
        t += DT;
        if m.in_flight() == 0 {
            break;
        }
    }
    (t, peak)
}

#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn measure_settle_time_of_each_character() {
    println!("[ui-motion] tempo até ASSENTAR, pela porta do produto (60 Hz)");
    println!(
        "[ui-motion] {:<28} {:>10} {:>10}",
        "carácter / papel", "assenta", "pico"
    );
    for (name, ch, role) in [
        ("Discreto · Travel", UiCharacter::Discrete, Role::Travel),
        ("Discreto · Fade", UiCharacter::Discrete, Role::Fade),
        ("Expressivo · Travel", UiCharacter::Expressive, Role::Travel),
        ("Expressivo · Fade", UiCharacter::Expressive, Role::Fade),
    ] {
        let (t, peak) = settle(ch, role);
        println!("[ui-motion] {name:<28} {:>9.3}s {peak:>10.4}", t);
    }
    println!(
        "[ui-motion] ⚠️ o Discreto NÃO pode ter pico > 1,0 — o contrato dele é estrutural (ζ = 1)"
    );
}

/// ⚠️ **O custo tem DUAS grandezas e elas não são a mesma.** Integrar é `O(em voo)`; lembrar é
/// `O(tocados recentemente)`. Uma sonda que só medisse a segunda diria que um app parado custa
/// caro, e um app parado não tem nada em voo.
#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn measure_what_a_frame_of_ui_motion_costs() {
    println!("[ui-motion] custo de UM `advance`, por número de coisas EM VOO");
    println!(
        "[ui-motion] {:>8} {:>12} {:>14}",
        "em voo", "us/quadro", "% de 16,6ms"
    );
    for n in [0_u64, 1, 3, 40, 400] {
        let mut m = UiMotion::default();
        m.set_character(UiCharacter::Expressive);
        for i in 0..n {
            m.animate(NodeId(i + 1), 0.0, Role::Travel);
            m.animate(NodeId(i + 1), 1.0, Role::Travel);
        }
        // Mediana de 200 quadros; o primeiro paga o aquecimento do mapa.
        let mut ms = Vec::with_capacity(200);
        for _ in 0..200 {
            let t0 = std::time::Instant::now();
            m.advance(DT);
            ms.push(t0.elapsed().as_secs_f64() * 1e6);
            // Re-alveja para manter tudo em voo durante a medição inteira.
            for i in 0..n {
                m.animate(
                    NodeId(i + 1),
                    if ms.len() % 2 == 0 { 1.0 } else { 0.0 },
                    Role::Travel,
                );
            }
        }
        ms.sort_by(f64::total_cmp);
        let med = ms[ms.len() / 2];
        println!(
            "[ui-motion] {n:>8} {med:>11.3}us {:>13.4}",
            med / 16_600.0 * 100.0
        );
    }
    println!("[ui-motion] ⚠️ o caso REAL é 0-3; o 400 existe só para mostrar a inclinação");
}

/// A varredura que **escolheu a rigidez do Discreto**. O plano pedia "uma transição discreta, na
/// ordem dos 120 ms"; o número que shipa é o desta tabela, não um palpite.
#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn measure_which_stiffness_a_discrete_character_wants() {
    println!("[ui-motion] ζ = 1 (criticamente amortecido), por rigidez");
    println!(
        "[ui-motion] ⚠️ `assenta` é a CAUDA (|x-1|<1e-3); o olho vê o JOELHO — mede-se t95 e t99"
    );
    println!(
        "[ui-motion] {:>10} {:>10} {:>10} {:>12}",
        "rigidez", "t95", "t99", "assenta"
    );
    for w in [12.0_f64, 18.0, 24.0, 28.0, 34.0, 40.0, 60.0, 90.0] {
        // O mesmo integrador do produto, com a rigidez varrida.
        let s = ph2d_spring::Spring {
            stiffness: w,
            damping: 1.0,
        };
        let mut st = ph2d_spring::SpringState::at_rest();
        let (mut t, mut done) = (0.0_f64, false);
        let (mut t95, mut t99) = (f64::NAN, f64::NAN);
        for _ in 0..600 {
            done = st.advance(DT, s);
            t += DT;
            if t95.is_nan() && st.x >= 0.95 {
                t95 = t;
            }
            if t99.is_nan() && st.x >= 0.99 {
                t99 = t;
            }
            if done {
                break;
            }
        }
        println!(
            "[ui-motion] {w:>10.1} {t95:>9.3}s {t99:>9.3}s {t:>11.3}s{}",
            if done { "" } else { "  (não assentou)" }
        );
    }
}
