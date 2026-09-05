//! ⭐⭐⭐ **O PEN-DOWN TOMA A ÂNCORA QUE O ARRASTO VAI PEDIR** — as duas metades
//! do gesto, conferidas uma contra a outra.
//!
//! # ⛔⛔ O report que o obrigou a existir
//!
//! Enio, 2026-09-05, sobre o pincel de tecido acabado de shipar: ***«não
//! funciona. nada aconteceu ao pintar»***. E não era o solver, nem o pincel, nem
//! o chip: era **uma linha**.
//!
//! O gesto de escultura tem duas metades escritas em sítios diferentes:
//!
//! 1. **o pen-down** decide entre *tomar a âncora* (`take_hold`) e *carimbar*
//!    (`sculpt_at`), e a pergunta é [`Verb::anchors`];
//! 2. **o arrasto** roteia por `Grip`, e três dos ramos (`hook_step`,
//!    `pending_grab`, `turn_at`) começam com `let Some(..) = self.grab else {
//!    return; }`.
//!
//! O `Grip::Simulate` entrou na metade 2 e **não** na metade 1 ⇒ o pen-down
//! carimbava, a âncora ficava `None`, e todo evento de arrasto saía no primeiro
//! `if`. Silêncio absoluto: sem erro, sem log, sem um vértice movido.
//!
//! ⚠️⚠️ **E o `anchors()` JÁ TINHA o parágrafo que previa esta classe** — ele
//! diz, por escrito, que a pergunta é feita pelo lado positivo *«que é a forma
//! que sobrevive ao sexto grip em vez de o adotar em silêncio»*. O sexto grip
//! chegou e a lista não foi lida. *Uma lista pelo lado positivo obriga o grip
//! novo a declarar-se, e nada obriga quem o escreve a ler a lista — o que obriga
//! é um gate.*
//!
//! # ⚠️ Por que ele lê o FONTE, e o que isso custa
//!
//! A rota do ponteiro não é alcançável de um teste (a `Sculpt3dScene` segura
//! device e superfície de janela real — a mesma cerca que o handoff do undo do
//! filtro já declarou). O que ESTE gate pode fazer é prender as duas listas uma
//! à outra, e é o que ele faz. ⛔ Ele não prova que o gesto funciona; prova que
//! as duas metades **concordam sobre quem precisa de âncora**, que é a classe
//! inteira do defeito.

use ph2d_sculpt3d::{Grip, Verb};

/// O roteador do gesto.
const SRC: &str = include_str!("../src/sculpt3d_input.rs");

/// O nome de um grip, como ele aparece no fonte.
fn nome(g: Grip) -> &'static str {
    match g {
        Grip::Stamp => "Stamp",
        Grip::Hold => "Hold",
        Grip::Hook => "Hook",
        Grip::Turn(_) => "Turn",
        Grip::Paint => "Paint",
        Grip::Simulate => "Simulate",
    }
}

/// **AS TRÊS PORTAS QUE COMEÇAM PELA ÂNCORA.**
///
/// ⚠️ Cada uma abre com `let Some(..) = self.grab else { return; }` — sem a
/// âncora tomada no pen-down, as três são no-ops silenciosos.
const PEDEM_ANCORA: [&str; 3] = ["hook_step", "pending_grab", "turn_at"];

/// O corpo do `match` do arrasto, arm a arm: `(cabeça, corpo)`.
fn arms() -> Vec<(String, String)> {
    let (_, resto) = SRC
        .split_once("Drag::Sculpt => match scene.brush.verb.grip() {")
        .expect("⛔ o roteador do arrasto mudou de forma");
    // O `match` acaba onde o braço seguinte do `Drag` começa.
    let corpo = resto
        .split_once("\n            Drag::")
        .map_or(resto, |(a, _)| a);
    let mut out = Vec::new();
    let mut it = corpo.split("Grip::").skip(1).peekable();
    let mut pendente: Vec<String> = Vec::new();
    while let Some(pedaco) = it.next() {
        let (cabeca, cauda) = pedaco
            .split_once("=>")
            .map_or((pedaco, ""), |(a, b)| (a, b));
        // `Grip::Stamp | Grip::Paint =>` chega como dois pedaços: o primeiro sem
        // `=>` na cabeça dele. Ele fica pendente até o que fecha o arm.
        let nome: String = cabeca
            .trim()
            .trim_end_matches(|c: char| !c.is_alphanumeric())
            .to_string();
        pendente.push(nome);
        if pedaco.contains("=>") {
            let corpo = cauda.to_string();
            for n in pendente.drain(..) {
                out.push((n, corpo.clone()));
            }
        }
        if it.peek().is_none() {
            assert!(pendente.is_empty(), "⛔ um arm ficou sem `=>`");
        }
    }
    out
}

/// ⭐⭐⭐ **GATE — quem o arrasto conduz a partir da âncora, o pen-down ancora.**
#[test]
fn the_sculpt_pen_down_arms_what_the_drag_needs() {
    let arms = arms();
    let mut visto: Vec<&str> = Vec::new();
    let (mut com, mut sem) = (0usize, 0usize);

    for verb in Verb::ALL {
        let n = nome(verb.grip());
        let (_, corpo) = arms
            .iter()
            .find(|(cabeca, _)| cabeca.starts_with(n))
            .unwrap_or_else(|| {
                panic!("⛔ o grip `{n}` (verbo {verb:?}) não é roteado pelo arrasto")
            });
        // ⚠️ O corpo do arm vai até o próximo `Grip::`, então ele pode arrastar o
        // início do seguinte — o que importa é se ALGUMA das três portas aparece
        // antes disso, e o `split` já corta ali.
        let precisa = PEDEM_ANCORA.iter().any(|porta| corpo.contains(porta));
        assert_eq!(
            verb.anchors(),
            precisa,
            "⛔ as duas metades do gesto discordam sobre `{verb:?}` (grip `{n}`): o \
             arrasto {} a âncora e o pen-down {}. Foi exatamente isto que deixou o \
             pincel de tecido MUDO no smoke de 2026-09-05.",
            if precisa { "PEDE" } else { "não pede" },
            if verb.anchors() {
                "a toma"
            } else {
                "NÃO a toma"
            }
        );
        if precisa {
            com += 1;
        } else {
            sem += 1;
        }
        if !visto.contains(&n) {
            visto.push(n);
        }
    }

    // ⚠️ **Os dois controles, e sem eles o gate fica verde por vácuo:** se
    // nenhum arm pedisse âncora (uma porta renomeada), a igualdade valeria com
    // `false == false` em todo verbo.
    assert!(
        com > 0,
        "⛔ nenhum verbo pede âncora: as portas mudaram de nome"
    );
    assert!(
        sem > 0,
        "⛔ todo verbo pede âncora: o censo perdeu o outro lado"
    );
    assert_eq!(
        visto.len(),
        6,
        "⛔ o roteador não cobre os seis grips — um grip novo entrou sem ser \
         roteado, que é a metade 2 do mesmo defeito"
    );
}
