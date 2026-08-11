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

/// **A MARCA NÃO É GLOBAL: quem não tocou continua EXATAMENTE do tamanho com que nasceu.**
///
/// Medido em 1,0 s, o instante em que a chuva está partida ao meio (23 dos 56 já alcançaram o
/// obstáculo). O oráculo não é uma razão entre dois grupos que eu escolhi por uma linha de `y` —
/// é a **igualdade exata** com o braço de controle: um elemento que nunca tocou tem de sair da
/// cena com o tamanho que o `motion.scale` lhe deu, ao bit. É o que separa *"o canal descreve os
/// contatos"* de *"o canal descreve o tempo"*.
///
/// E a segunda metade amarra a marca ao TEMPO em vez do lugar: a conta de marcados só CRESCE, e
/// no fim da cena é a chuva inteira. Um canal que descrevesse qualquer outra coisa não teria
/// como produzir uma frente que avança elemento a elemento.
///
/// ⚠️ **A correlação com a ALTURA foi tentada e é FALSA, e a cena diz por quê:** o topo do disco
/// está em `DISC_Y + DISC_R = 2,1`, e com `restitution` um elemento marcado QUICA de volta para
/// cima — medido, um deles está em `y = 2,08` carregando `0,4545` de marca. *"Marcado ⇒ está
/// embaixo"* é uma afirmação sobre a POSIÇÃO num instante, e uma marca é sobre a HISTÓRIA; a
/// linha de `y` que eu tinha escolhido nem era da cena, era um número meu.
///
/// FALSIFICADO por um canal escrito fora do ramo de contato: todo mundo cresceria junto, o
/// mínimo sairia do tamanho de nascimento e a frente apareceria completa no primeiro tique.
#[test]
fn the_mark_is_not_global_and_the_front_advances() {
    let (_, control) = settled(0.0, 1.0);
    let born = control[0][0];
    let count = |secs: f64| -> (usize, usize) {
        let (_, sz) = settled(MARK, secs);
        (
            sz.iter().filter(|s| s[0] == born).count(),
            sz.iter().filter(|s| s[0] > born).count(),
        )
    };

    let (untouched, marked) = count(1.0);
    assert!(untouched > 0, "alguém tem de continuar intocado");
    assert!(marked > 0, "e alguém tem de estar marcado");
    assert_eq!(
        untouched + marked,
        (ROWS * COLS) as usize,
        "ninguém ENCOLHEU"
    );

    // A frente avança e nunca recua: a marca é um registro, não uma leitura do instante.
    let (_, early) = count(0.6);
    let (rest, late) = count(3.0);
    assert!(early < marked, "a frente avança: {early} então {marked}");
    assert!(marked < late, "…e continua: {marked} então {late}");
    assert_eq!(rest, 0, "no fim a chuva inteira encostou em alguma coisa");
}

/// **A SONDA** — de onde saem os números do anúncio e do doc.
///
/// `cargo test -p ph2d-host-desktop --lib hit_demo::tests::probe -- --ignored --nocapture`
#[test]
#[ignore]
fn probe_hit_mark() {
    for (mark, secs) in [
        (2.0f32, 3.0f64),
        (MARK, 3.0),
        (8.0, 3.0),
        (MARK, 1.0),
        (MARK, 5.0),
    ] {
        let (p, sz) = settled(mark, secs);
        let (_, ctl) = settled(0.0, secs);
        let big = sz.iter().fold(0.0f32, |m, s| m.max(s[0]));
        let small = sz.iter().fold(f32::MAX, |m, s| m.min(s[0]));
        let landed = p.iter().filter(|q| q[1] < 2.0).count();
        eprintln!(
            "mark {mark} @ {secs:.1}s: size [{small:.3}, {big:.3}]  controle {:.3}  \
             {landed}/{} abaixo de y=2",
            ctl[0][0],
            p.len()
        );
    }
}
