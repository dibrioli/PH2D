//! I1 (Onda 2, `docs/CodeEditor/03_onda2.md`): o reparse do tree-sitter sai da tecla. A tecla só
//! desloca a árvore; o worker reparsa e o quadro seguinte traz a árvore nova. Gate SEM relógio: um
//! contador de reparses síncronos.

use crate::{Mesa, ctrl};
use ph2d_code_syntax::HighlightKind;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key, KeyInput};
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;
use ph2d_text::TextSystem;

const BOB: &str = include_str!("../../../ph2d-code-syntax/tests/it/fixtures/bob.luau");

fn tecla(m: &mut Mesa, id: ScriptId, key: Key, mods: Modifiers, text: Option<&str>, paste: &str) {
    let mut ts = TextSystem::without_system_fonts();
    let input = KeyInput {
        key,
        mods,
        pressed: true,
        text,
        paste: (!paste.is_empty()).then(|| paste.to_owned()),
    };
    let _ = m.docs().key(id.0, &mut ts, input);
}

fn letra(m: &mut Mesa, id: ScriptId, c: char) {
    tecla(m, id, Key::Char(c), Modifiers::default(), Some(&c.to_string()), "");
}

/// O cursor no fim da linha que contém `agulha` (pelo teclado: Ctrl+Home, ↓, End).
fn fim_da_linha(m: &mut Mesa, id: ScriptId, agulha: &str) {
    let t = m.docs().text_of(id).expect("aberto");
    let linha = t.lines().position(|l| l.contains(agulha)).expect("agulha");
    tecla(m, id, Key::Home, ctrl(), None, "");
    for _ in 0..linha {
        tecla(m, id, Key::Down, Modifiers::default(), None, "");
    }
    tecla(m, id, Key::End, Modifiers::default(), None, "");
}

/// O quadro da shell: a família e a pintura da aba.
fn quadro(m: &mut Mesa, id: ScriptId) {
    m.quadro();
    m.pinta(id);
}

fn sonda(m: &mut Mesa, id: ScriptId) -> ph2d_app_code::GrammarProbe {
    m.docs().grammar_of(id).expect("aberto")
}

/// Espera o worker e dá quadros até a árvore ter todas as edições.
fn assenta(m: &mut Mesa, id: ScriptId) {
    for _ in 0..8 {
        m.docs().wait_grammar();
        quadro(m, id);
        if !sonda(m, id).stale {
            return;
        }
    }
    panic!("a árvore não assentou em 8 quadros com o worker parado");
}

/// ⛔ Gate: nenhuma tecla que não precisa da árvore reparsa NA tecla — letras, Backspace, Delete,
/// Ctrl+Backspace, Tab, colar, desfazer, refazer — com quadros pelo meio (o worker a trazer árvores
/// entre as teclas) e sem eles. Controlo: o Enter sobre a árvore velha conta (o contador está vivo).
/// E a árvore que o worker traz é a do texto: o realce, assente, é o de um documento aberto do zero.
#[test]
fn as_teclas_sem_arvore_nunca_reparsam_na_tecla() {
    let grande: String = BOB.repeat(20);
    let (mut m, ids) = Mesa::new(&[("bob.luau", &grande)]);
    let id = ids[0];
    m.pede(id);
    quadro(&mut m, id);
    fim_da_linha(&mut m, id, "self.t = self.t + dt");
    let mut rng = fastrand::Rng::with_seed(0x1_0F0A);
    for i in 0..300 {
        match rng.u8(0..12) {
            0 => tecla(&mut m, id, Key::Backspace, Modifiers::default(), None, ""),
            1 => tecla(&mut m, id, Key::Backspace, ctrl(), None, ""),
            2 => tecla(&mut m, id, Key::Delete, Modifiers::default(), None, ""),
            3 => tecla(&mut m, id, Key::Char('v'), ctrl(), Some("v"), " + 1"),
            4 => tecla(&mut m, id, Key::Char('z'), ctrl(), Some("z"), ""),
            5 => tecla(&mut m, id, Key::Char('y'), ctrl(), Some("y"), ""),
            6 => tecla(&mut m, id, Key::Tab, Modifiers::default(), None, ""),
            _ => letra(&mut m, id, char::from(b'a' + rng.u8(0..26))),
        }
        match i % 5 {
            0 => quadro(&mut m, id),
            1 => {
                m.docs().wait_grammar();
                quadro(&mut m, id);
            }
            _ => {}
        }
    }
    assert_eq!(
        sonda(&mut m, id).sync_reparses,
        0,
        "uma tecla que não precisa da árvore reparsou NA tecla"
    );
    // De volta ao texto (válido) do início, desfazendo com quadros pelo meio: com ERROR a forma
    // depende da história (`ph2d-code-syntax` `incremental.rs`); com o texto válido tem de ser a do
    // parse do zero.
    let mut desfeitos = 0;
    while m.docs().text_of(id).as_deref() != Some(grande.as_str()) {
        assert!(desfeitos < 400, "o desfazer não voltou ao texto do início");
        tecla(&mut m, id, Key::Char('z'), ctrl(), Some("z"), "");
        if desfeitos % 3 == 0 {
            quadro(&mut m, id);
        }
        desfeitos += 1;
    }
    assenta(&mut m, id);
    assert_eq!(sonda(&mut m, id).sync_reparses, 0, "o desfazer reparsou na tecla");
    let (mut zero, ids0) = Mesa::new(&[("bob.luau", &grande)]);
    zero.pede(ids0[0]);
    assert!(zero.docs().problems_of(ids0[0]).is_empty(), "o texto do início é válido");
    let alvo = grande.find("self.t = self.t + dt").unwrap_or(0);
    let faixa = alvo.saturating_sub(3_000)..(alvo + 3_000).min(grande.len());
    let mut lidos = 0;
    let diferentes: Vec<usize> = faixa
        .step_by(5)
        .filter(|&b| grande.is_char_boundary(b))
        .inspect(|_| lidos += 1)
        .filter(|&b| m.docs().highlight_at(id, b) != zero.docs().highlight_at(ids0[0], b))
        .collect();
    assert!(lidos > 500);
    assert!(
        diferentes.is_empty(),
        "o realce assente difere do de um documento aberto do zero em {} bytes (1.º {:?})",
        diferentes.len(),
        diferentes.first()
    );
    // Controlo: uma letra (árvore velha) e o Enter logo a seguir — esse reparse é na tecla.
    letra(&mut m, id, 'q');
    assert!(sonda(&mut m, id).stale, "a letra devia deixar a árvore velha");
    tecla(&mut m, id, Key::Enter, Modifiers::default(), Some("\r"), "");
    assert_eq!(
        sonda(&mut m, id).sync_reparses,
        1,
        "o Enter sobre a árvore velha tem de reparsar (o contador está morto?)"
    );
}

/// A resposta do worker para a aba FECHADA nunca entra na aba que a reabre (a gramática nasce de
/// novo, `CodeDoc::unpark`): os números dos trabalhos são do processo, não da gramática. Traço: um
/// `-` na aba velha (trabalho A pronto, por trazer), fechar com ● e reabrir, outro `-` (trabalho B).
/// Se A entrasse, a árvore teria só um `-` e a linha não seria comentário.
#[test]
fn a_resposta_da_aba_fechada_nunca_entra_na_reaberta() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    let id = ids[0];
    m.pede(id);
    quadro(&mut m, id);
    let t = m.docs().text_of(id).expect("aberto");
    let linha = t
        .lines()
        .position(|l| l.contains("self.t = self.t + dt"))
        .expect("linha");
    let dt = t.find("self.t + dt").expect("dt") + "self.t + ".len() + 2;
    tecla(&mut m, id, Key::Home, ctrl(), None, "");
    for _ in 0..linha {
        tecla(&mut m, id, Key::Down, Modifiers::default(), None, "");
    }
    tecla(&mut m, id, Key::Home, Modifiers::default(), None, "");
    letra(&mut m, id, '-');
    m.docs().wait_grammar();
    m.docs().close(id.0);
    assert!(m.docs().is_parked(id), "a aba com ● estaciona");
    // Reabre SEM quadro (o quadro traria A antes de B existir): A e B chegam juntos ao dreno.
    m.docs().open(id, "bob.luau", "");
    letra(&mut m, id, '-');
    assert_eq!(
        m.docs().text_of(id).map(|s| s[dt..dt + 2].to_owned()),
        Some("dt".to_owned())
    );
    assenta(&mut m, id);
    assert_eq!(
        m.docs().highlight_at(id, dt),
        Some(HighlightKind::Comment),
        "a árvore da aba reaberta não tem o 2.º `-`: entrou a resposta da aba fechada"
    );
}

/// O realce alcança a tecla no quadro SEGUINTE ao worker acabar — e é o quadro que traz a árvore,
/// nunca a espera (o quadro nunca espera o worker).
#[test]
fn o_realce_alcanca_a_tecla_no_quadro_seguinte() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    let id = ids[0];
    m.pede(id);
    quadro(&mut m, id);
    let t = m.docs().text_of(id).expect("aberto");
    let linha = t
        .lines()
        .position(|l| l.contains("self.t = self.t + dt"))
        .expect("linha");
    let byte_dt = t.find("self.t + dt").expect("dt") + "self.t + ".len();
    assert_ne!(
        m.docs().highlight_at(id, byte_dt),
        Some(HighlightKind::Comment)
    );
    tecla(&mut m, id, Key::Home, ctrl(), None, "");
    for _ in 0..linha {
        tecla(&mut m, id, Key::Down, Modifiers::default(), None, "");
    }
    tecla(&mut m, id, Key::Home, Modifiers::default(), None, "");
    letra(&mut m, id, '-');
    letra(&mut m, id, '-');
    let dt = byte_dt + 2;
    assert_eq!(
        m.docs().text_of(id).map(|s| s[dt..dt + 2].to_owned()),
        Some("dt".to_owned())
    );
    m.docs().wait_grammar();
    assert!(
        sonda(&mut m, id).stale,
        "a espera trouxe a árvore: quem a traz é o quadro"
    );
    assert_ne!(
        m.docs().highlight_at(id, dt),
        Some(HighlightKind::Comment),
        "sem quadro a árvore ainda é a velha"
    );
    m.pinta(id);
    assert!(!sonda(&mut m, id).stale);
    assert_eq!(
        m.docs().highlight_at(id, dt),
        Some(HighlightKind::Comment),
        "um quadro depois do worker, a linha comentada pinta-se como comentário"
    );
    assert_eq!(sonda(&mut m, id).sync_reparses, 0);
}
