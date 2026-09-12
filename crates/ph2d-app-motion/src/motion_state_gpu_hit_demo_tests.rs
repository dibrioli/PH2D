//! Gates da cena `=30` — e a SONDA de onde saíram os números do anúncio.
//!
//! A regra do plano 89: *toda wave ganha cena com números MEDIDOS, e a sonda headless roda ANTES
//! de a mensagem ser escrita*.

use super::*;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Cozinha até assentar e devolve `(P, size)` de cada elemento.
fn settled(mark: f32, secs: f64) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_hit_demo_document(&mut doc, &reg, mark).expect("a cena é bem tipada");
    let mut cook = Cook::new();
    let last = (secs * 60.0) as u64;
    let (mut p, mut sz) = (vec![], vec![]);
    for k in 0..=last {
        let t = k as f64 / 60.0;
        let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
            .as_stream()
            .clone();
        if k == last {
            let g = |name| match s.get(name) {
                Some(Column::Vec2(v)) => v.clone(),
                _ => vec![],
            };
            p = g("P");
            sz = g("size");
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    (p, sz)
}

/// **A cena é bem tipada e a chuva inteira está lá.**
#[test]
fn the_scene_builds_with_the_whole_shower() {
    let (p, _) = settled(MARK, 3.0);
    assert_eq!(p.len(), (ROWS * COLS) as usize);
}

/// **O CONTATO DEIXA MARCA, e a marca é o contato** — a capacidade inteira numa comparação.
///
/// A MESMA cena, o mesmo tudo, e um único número diferente: com `mark = 0` a cadeia continua
/// inteira (o canal é lido, o valor viaja, o drive roda) e nada engorda. É esse braço que impede
/// o gate de estar medindo alguma outra coisa da cena que também mude o tamanho.
///
/// FALSIFICADO por um canal que não chega ao `size`: os dois braços mediriam igual.
#[test]
fn what_touched_is_marked_and_what_did_not_is_not() {
    let (_, marked) = settled(MARK, 3.0);
    let (_, control) = settled(0.0, 3.0);
    let biggest = marked.iter().fold(0.0f32, |m, s| m.max(s[0]));
    let flat = control.iter().fold(0.0f32, |m, s| m.max(s[0]));
    assert!(
        biggest > flat * 1.5,
        "o contato tem de engordar quem tocou: {biggest} contra {flat} sem marca"
    );
    // E o controle tem de estar VIVO: `mark = 0` não pode ser uma cena que nem roda.
    assert_eq!(control.len(), (ROWS * COLS) as usize);
}

/// **O TAMANHO É O CONTATO DE AGORA — E O QUADRO CONVERGE.**
///
/// ⚠️ Este gate substitui o `the_mark_is_not_global_and_the_front_advances`, que afirmava uma
/// **frente monotônica** — propriedade de uma marca HISTÓRICA, e o canal `hit` não é história: é
/// a profundidade que ESTE tique empurrou para fora. A cena somava esse instantâneo com
/// `motion.drive(Add)` e crescia para sempre (medido: maior `size` 0,455 · 0,892 · 1,197 · 1,963
/// · **3,021** aos 8 s, com a chuva já PARADA) — *"cada Play um resultado diferente"*, o report
/// do Enio de 2026-08-11. Um valor instantâneo consumido como acumulador não tem ponto fixo.
///
/// As duas metades que sobrevivem, e que são o que a cena de fato diz:
///
/// - **quem não toca fica EXATAMENTE no tamanho de nascimento** (igualdade ao bit contra o braço
///   de controle — é isto que separa *"o canal descreve contatos"* de *"o canal descreve tempo"*);
/// - **e a foto PARA**: o conjunto de tamanhos aos 5 s e aos 8 s é o mesmo, então dois Plays
///   mostram a mesma coisa. Sem esta metade o gate voltaria a ser verde sobre o defeito.
#[test]
fn the_size_is_the_contact_now_and_the_frame_converges() {
    let (_, control) = settled(0.0, 1.0);
    let born = control[0][0];
    let (_, early) = settled(MARK, 1.0);
    let untouched = early.iter().filter(|s| s[0] == born).count();
    assert!(untouched > 0, "alguém tem de continuar intocado, ao bit");
    assert!(
        early.iter().all(|s| s[0] >= born - 1e-6),
        "ninguém pode ENCOLHER abaixo do tamanho de nascimento"
    );

    // A foto para: o mesmo quadro aos 5 s e aos 8 s.
    let (p5, s5) = settled(MARK, 5.0);
    let (p8, s8) = settled(MARK, 8.0);
    let dp = p5
        .iter()
        .zip(&p8)
        .flat_map(|(a, b)| (0..2).map(move |k| (a[k] - b[k]).abs()))
        .fold(0.0f32, f32::max);
    let ds = s5
        .iter()
        .zip(&s8)
        .map(|(a, b)| (a[0] - b[0]).abs())
        .fold(0.0f32, f32::max);
    assert!(dp < 0.01, "a chuva tem de estar parada: {dp}");
    assert!(
        ds < 0.02,
        "o tamanho tem de PARAR de crescer: {ds} entre 5 s e 8 s"
    );
    // E o contato tem de estar de fato marcando alguém no quadro final.
    assert!(
        s8.iter().fold(0.0f32, |m, s| m.max(s[0])) > born * 1.5,
        "no fim alguém tem de estar visivelmente encostado"
    );
}

/// **A SONDA** — de onde saem os números do anúncio e do doc.
///
/// `cargo test -p ph2d-host-desktop --lib hit_demo::tests::probe -- --ignored --nocapture`
#[test]
#[ignore]
fn probe_hit_mark() {
    for (mark, secs) in [
        (0.2f32, 3.0f64),
        (MARK, 3.0),
        (1.0, 3.0),
        (MARK, 1.0),
        (MARK, 5.0),
        (MARK, 8.0),
    ] {
        let (p, sz) = settled(mark, secs);
        let (_, ctl) = settled(0.0, secs);
        let big = sz.iter().fold(0.0f32, |m, s| m.max(s[0]));
        let small = sz.iter().fold(f32::MAX, |m, s| m.min(s[0]));
        let touching = sz.iter().filter(|s| s[0] > ctl[0][0] + 1e-4).count();
        eprintln!(
            "mark {mark} @ {secs:.1}s: size [{small:.3}, {big:.3}]  base {:.3}  \
             {touching}/{} encostados",
            ctl[0][0],
            p.len()
        );
    }
}

/// **SONDA — A CHUVA ASSENTA?** (o irmao do `probe_does_the_scene_settle` do `=31`.)
#[test]
#[ignore]
fn probe_does_the_shower_settle() {
    for secs in [1.0f64, 2.0, 3.0, 5.0, 8.0] {
        let (p, sz) = settled(MARK, secs);
        let (q, _) = settled(MARK, secs + 1.0 / 60.0);
        let mov = p
            .iter()
            .zip(&q)
            .flat_map(|(a, b)| (0..2).map(move |k| (a[k] - b[k]).abs()))
            .fold(0.0f32, f32::max);
        let big = sz.iter().fold(0.0f32, |m, s| m.max(s[0]));
        eprintln!("t={secs:>5.2}s  maior size {big:.3}  movimento no tique seguinte {mov:.6}");
    }
}
