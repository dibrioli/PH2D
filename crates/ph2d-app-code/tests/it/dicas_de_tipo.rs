//! I9 (Onda 2, `docs/CodeEditor/03_onda2.md`): as dicas de tipo — desligadas por omissão, um comando
//! liga; só a dica que o Alt+Enter inseriria; e a lei medida no VS Code 1.142.0
//! (`ferramentas/medicoes_onda2/I9_I12_I14.md` §I9): a dica não tem coluna — o cursor pinta-se antes
//! dela, a selecção que começa ali começa depois dela, um clique nela dá a posição dela, escrever ali
//! empurra-a.

use crate::assistente::{Partilhado, Plano, com_falso, converge, letra, tecla};
use crate::completar::T0;
use crate::dicas::{ate, nunca};
use crate::Mesa;
use ph2d_app_code::commands::CMD_TYPE_HINTS;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;

const TEXTO: &str = "local s = 1\nlocal b = s\nlocal t = f\n";

/// Como o `luau-lsp` 1.70.1 (I0b §6, I7 §2): `: number` inserível, `: any`, o genérico solto e o
/// `self` truncado sem `textEdits` — só a 1.ª passa a lei do Alt+Enter.
fn tipos(s: &str) -> Vec<(usize, String, bool)> {
    let fim = |agulha: &str| s.find(agulha).map(|i| i + agulha.len());
    let mut out = Vec::new();
    if let Some(b) = fim("local s") {
        out.push((b, ": number".to_owned(), true));
    }
    if let Some(b) = fim("local b") {
        out.push((b, ": any".to_owned(), true));
    }
    if let Some(b) = fim("local t") {
        out.push((b, ": T".to_owned(), true));
    }
    out.push((0, ": {+ base_x: number?, ... 3 more }".to_owned(), false));
    out
}

fn mesa() -> (Mesa, ScriptId, Partilhado) {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", TEXTO)],
        Plano {
            tipar: Some(tipos),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    (m, id, visto)
}

fn liga(m: &mut Mesa, id: ScriptId) {
    assert!(m.docs().command(id.0, CMD_TYPE_HINTS));
}

fn rotulo(m: &mut Mesa, id: ScriptId) -> String {
    m.docs()
        .commands(id.0)
        .into_iter()
        .find(|c| c.id == CMD_TYPE_HINTS)
        .expect("o comando está na aba")
        .label
}

#[test]
fn desligadas_por_omissao_e_o_comando_liga_e_desliga() {
    let (mut m, id, visto) = mesa();
    nunca(&mut m, id, T0 + 10_000_000_000, "desligadas: nada se pergunta", |m| {
        m.docs().inlays_of(id).is_empty()
    });
    assert_eq!(visto.lock().unwrap().tipos, 0);
    assert_eq!(rotulo(&mut m, id), "Show Type Hints");
    liga(&mut m, id);
    assert_eq!(rotulo(&mut m, id), "Hide Type Hints");
    ate(&mut m, id, T0 + 20_000_000_000, "as dicas não chegaram", |m| {
        !m.docs().inlays_of(id).is_empty()
    });
    assert_eq!(
        m.docs().inlays_of(id),
        [(7, ": number".to_owned())],
        "só a que o Alt+Enter inseriria (nem `any`, nem o genérico, nem a truncada)"
    );
    liga(&mut m, id);
    assert!(m.docs().inlays_of(id).is_empty(), "desligar larga-as");
    assert_eq!(rotulo(&mut m, id), "Show Type Hints");
}

#[test]
fn a_dica_nao_tem_coluna_o_cursor_a_seleccao_e_o_clique() {
    let (mut m, id, _) = mesa();
    liga(&mut m, id);
    ate(&mut m, id, T0 + 20_000_000_000, "as dicas não chegaram", |m| {
        !m.docs().inlays_of(id).is_empty()
    });
    m.pinta(id);
    let caret = |m: &mut Mesa, at: usize| m.docs().caret_rect_at(id, at).expect("à vista");
    let adv = caret(&mut m, 6).x - caret(&mut m, 5).x;
    let (c7, c8) = (caret(&mut m, 7), caret(&mut m, 8));
    // Entre o `s` e o espaço estão os 8 caracteres de `: number`: o cursor no 7 pinta-se ANTES deles.
    assert!(
        c8.x - c7.x > 8.0 * adv,
        "o cursor do byte 7 antes da dica: {} → {} ({adv} por coluna)",
        c7.x,
        c8.x
    );
    // A selecção do espaço (7..8) começa DEPOIS da dica: um carácter de largura.
    let seg = m.docs().segments_of(id, 7, 8);
    assert_eq!(seg.len(), 1);
    assert!(
        (seg[0].w - adv).abs() < 0.5 && (seg[0].x + seg[0].w - c8.x).abs() < 0.5,
        "a selecção cobre só o espaço: {:?}",
        seg[0]
    );
    // Um clique em qualquer sítio da dica dá a posição dela; no espaço, o meio decide.
    let y = c7.y + c7.h * 0.5;
    let fim_da_dica = seg[0].x;
    for f in [0.1, 0.5, 0.9] {
        m.clica(id, c7.x + (fim_da_dica - c7.x) * f, y, Modifiers::default());
        assert_eq!(m.docs().cursor_of(id), Some((0, 7)), "clique a {f} da dica");
    }
    m.clica(id, fim_da_dica + adv * 0.25, y, Modifiers::default());
    assert_eq!(m.docs().cursor_of(id), Some((0, 7)), "o 1.º quarto do espaço");
    m.clica(id, fim_da_dica + adv * 0.75, y, Modifiers::default());
    assert_eq!(m.docs().cursor_of(id), Some((0, 8)), "o último quarto do espaço");
    // A seta salta a dica inteira: do 7 vai ao 8.
    m.clica(id, c7.x + 1.0, y, Modifiers::default());
    tecla(&mut m, id, Key::Right, Modifiers::default(), None, "");
    assert_eq!(m.docs().cursor_of(id), Some((0, 8)));
}

#[test]
fn escrever_na_posicao_da_dica_empurra_a() {
    let (mut m, id, _) = mesa();
    liga(&mut m, id);
    ate(&mut m, id, T0 + 20_000_000_000, "as dicas não chegaram", |m| {
        !m.docs().inlays_of(id).is_empty()
    });
    m.pinta(id);
    let c = m.docs().caret_rect_at(id, 7).expect("à vista");
    m.clica(id, c.x + 1.0, c.y + c.h * 0.5, Modifiers::default());
    letra(&mut m, id, 'x');
    assert_eq!(
        m.docs().inlays_of(id),
        [(8, ": number".to_owned())],
        "a dica anda para depois do escrito (o VS Code)"
    );
    // Apagado o sítio dela (uma faixa que o contém), sai até a pausa a refazer.
    tecla(&mut m, id, Key::Home, Modifiers::default(), None, "");
    tecla(
        &mut m,
        id,
        Key::End,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        None,
        "",
    );
    tecla(&mut m, id, Key::Backspace, Modifiers::default(), None, "");
    assert!(m.docs().inlays_of(id).iter().all(|(at, _)| *at != 8));
}
