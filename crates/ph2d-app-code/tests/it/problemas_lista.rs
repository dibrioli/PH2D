//! Acabamento (Onda 2, item 4 de `docs/CodeEditor/03_onda2.md`): a LISTA DE PROBLEMAS — o 2.º
//! separador da faixa de baixo, com os MESMOS problemas da calha, por gravidade e depois por linha
//! (VS Code 1.142); o contador da barra e o Ctrl+Shift+M abrem/fecham-na; clicar numa linha ou o
//! F8 / Shift+F8 seleccionam o culpado; o leitor de ecrã lê-a.

use crate::assistente::{Plano, com_falso, converge, letra, posicao};
use crate::completar::T0;
use crate::dicas::ate;
use crate::{Mesa, centro, ctrl};
use ph2d_a11y::{Action, Role};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key, KeyInput, PointerLook};
use ph2d_host::Modifiers;
use ph2d_panel_code::{BandTab, Severity};
use ph2d_script::ScriptId;
use ph2d_text::TextSystem;
use serde_json::{Value, json};

/// Um aviso na linha 1, uma informação na 2, um erro (com explicação, I8) na 3 e outro aviso na 4.
const TEXTO: &str =
    "local aviso = 1\nlocal nota = 2\npirnt(aviso)\nlocal outro = 3\nprint(nota, outro)\n";
const PIRNT: &str = "TypeError: Unknown global 'pirnt'; consider assigning to it first";

fn acusa(s: &str) -> Vec<Value> {
    [
        ("aviso", 2, "LintTodo: aviso por usar"),
        ("nota", 3, "Uma nota sobre a nota"),
        ("pirnt", 1, PIRNT),
        ("outro", 2, "LintTodo: outro por usar"),
    ]
    .into_iter()
    .filter_map(|(w, sev, msg)| {
        let b = s.find(w)?;
        Some(
            json!({ "range": { "start": posicao(s, b), "end": posicao(s, b + w.len()) },
                     "severity": sev, "code": 1001, "message": msg }),
        )
    })
    .collect()
}

fn palavra(w: &str) -> std::ops::Range<usize> {
    let b = TEXTO.find(w).unwrap();
    b..b + w.len()
}

fn ctrl_shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..ctrl()
    }
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

/// Uma tecla direto à família, com o que ela respondeu.
fn tecla(m: &mut Mesa, id: ScriptId, key: Key, mods: Modifiers) -> bool {
    let mut ts = TextSystem::without_system_fonts();
    let input = KeyInput {
        key,
        mods,
        pressed: true,
        text: None,
        paste: None,
    };
    m.docs().key(id.0, &mut ts, input).consumed
}

/// A faixa: (aberta, separador) — o estado da aba.
fn faixa(m: &mut Mesa, id: ScriptId) -> (bool, BandTab) {
    let d = m.docs();
    (d.console_open(id), d.band_tab_of(id))
}

/// O texto que a lista mostra de um problema: o título da explicação, senão a mensagem crua.
fn texto_de(p: &ph2d_panel_code::Problem) -> String {
    p.explanation
        .as_ref()
        .map_or(p.message.clone(), |e| e.title.clone())
}

/// O projecto com o servidor falso, a aba aberta e os quatro problemas chegados.
fn mesa() -> (Mesa, ScriptId) {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", TEXTO)],
        Plano {
            regra: Some(acusa),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    ate(&mut m, id, T0, "os quatro problemas não chegaram", |m| {
        m.docs().problems_of(id).len() == 4
    });
    (m, id)
}

/// ⭐ (a) As linhas são EXACTAMENTE os problemas da calha (`problems_of`), pela ordem gravidade →
/// linha: o erro da linha 3 vem antes do aviso da linha 1; o texto é o título da explicação (I8)
/// quando a há. A pintura mostra as mesmas, pela mesma ordem.
#[test]
fn a_lista_e_a_da_calha_por_gravidade_e_linha() {
    let (mut m, id) = mesa();
    let calha = m.docs().problems_of(id);
    let lista = m.docs().problem_list_of(id);
    assert_eq!(lista.len(), calha.len(), "{lista:?}");
    let de = |linha: usize| {
        let p = calha.iter().find(|p| p.line == linha).unwrap();
        (p.severity, texto_de(p), linha + 1)
    };
    assert_eq!(
        lista,
        [de(2), de(0), de(3), de(1)],
        "gravidade, depois linha"
    );
    assert_eq!(lista[0].0, Severity::Error);
    assert_eq!(
        lista[0].1, "\u{201c}pirnt\u{201d} doesn\u{2019}t exist",
        "o título da explicação, não o cru"
    );
    assert_eq!(lista[3].0, Severity::Info);
    // Fechada, a faixa não pinta linhas; aberta nos Problems, pinta estas, por esta ordem.
    m.pinta(id);
    assert!(m.docs().band_of(id).unwrap().problems.is_empty());
    assert!(tecla(&mut m, id, Key::Char('m'), ctrl_shift()));
    m.pinta(id);
    let banda = m.docs().band_of(id).unwrap();
    let pintadas: Vec<_> = banda
        .problems
        .iter()
        .map(|p| (p.severity, p.text.clone(), p.line))
        .collect();
    assert_eq!(pintadas, lista);
    assert_eq!(banda.tabs[1].0, "Problems (4)");
    assert_eq!(banda.empty, None);
}

/// (b) O contador da barra abre a faixa nos Problems e, com ela aberta nos Problems, fecha-a; o
/// Ctrl+Shift+M faz o mesmo (e é tomado: a cena não o vê); o Ctrl+M sem Shift segue para a shell.
#[test]
fn o_contador_e_o_ctrl_shift_m_abrem_e_fecham_a_lista() {
    let (mut m, id) = mesa();
    m.pinta(id);
    assert_eq!(faixa(&mut m, id), (false, BandTab::Console));
    let (x, y) = centro(m.docs().problems_counter_of(id).expect("o contador"));
    m.clica(id, x, y, Modifiers::default());
    assert_eq!(faixa(&mut m, id), (true, BandTab::Problems));
    m.pinta(id);
    let (x, y) = centro(m.docs().problems_counter_of(id).unwrap());
    m.clica(id, x, y, Modifiers::default());
    assert_eq!(faixa(&mut m, id), (false, BandTab::Problems));

    assert!(tecla(&mut m, id, Key::Char('m'), ctrl_shift()), "tomado");
    assert_eq!(faixa(&mut m, id), (true, BandTab::Problems));
    assert!(tecla(&mut m, id, Key::Char('m'), ctrl_shift()));
    assert_eq!(faixa(&mut m, id), (false, BandTab::Problems));
    assert!(
        !tecla(&mut m, id, Key::Char('m'), ctrl()),
        "Ctrl+M é da shell"
    );
    assert_eq!(faixa(&mut m, id), (false, BandTab::Problems));

    // Aberta no Console, o contador troca para os Problems (não fecha).
    m.pinta(id);
    let (x, y) = centro(m.docs().band_of(id).unwrap().tabs[0].1);
    m.clica(id, x, y, Modifiers::default());
    assert_eq!(faixa(&mut m, id), (true, BandTab::Console));
    m.pinta(id);
    let (x, y) = centro(m.docs().problems_counter_of(id).unwrap());
    m.clica(id, x, y, Modifiers::default());
    assert_eq!(faixa(&mut m, id), (true, BandTab::Problems));
}

/// ⭐ (c)+(e) Clicar na linha do erro selecciona o culpado e a letra seguinte vai para o TEXTO
/// (escrever «print» por cima do «pirnt»); a resposta do assistente ao texto corrigido tira o erro
/// da lista no quadro seguinte.
#[test]
fn clicar_numa_linha_selecciona_o_culpado_e_a_correccao_tira_o_da_lista() {
    let (mut m, id) = mesa();
    assert!(tecla(&mut m, id, Key::Char('m'), ctrl_shift()));
    m.pinta(id);
    let linha = m.docs().band_of(id).unwrap().problems[0].clone();
    assert_eq!(linha.line, 3);
    let (x, y) = centro(linha.rect);
    m.clica(id, x, y, Modifiers::default());
    assert_eq!(m.docs().selection_of(id), [palavra("pirnt")]);
    assert_eq!(m.docs().cursor_of(id).map(|c| c.0), Some(2));
    for c in "print".chars() {
        letra(&mut m, id, c);
    }
    let texto = m.docs().text_of(id).unwrap();
    assert!(texto.contains("\nprint(aviso)\n"), "{texto}");
    ate(&mut m, id, T0, "o erro corrigido ficou na lista", |m| {
        m.docs()
            .band_of(id)
            .is_some_and(|b| b.problems.len() == 3 && b.tabs[1].0 == "Problems (3)")
    });
    let lista = m.docs().problem_list_of(id);
    assert!(lista.iter().all(|p| p.0 != Severity::Error), "{lista:?}");
    assert_eq!(lista.len(), m.docs().problems_of(id).len());
}

/// ⭐ (d) F8 / Shift+F8 percorrem os problemas pela ORDEM DA LISTA e dão a volta; cada passo
/// selecciona o culpado, e a linha da lista dele fica realçada.
#[test]
fn f8_percorre_pela_ordem_da_lista_e_da_a_volta() {
    let (mut m, id) = mesa();
    assert!(tecla(&mut m, id, Key::Char('m'), ctrl_shift()));
    let ordem = ["pirnt", "aviso", "outro", "nota"];
    let passo = |m: &mut Mesa, mods: Modifiers, w: &str| {
        assert!(tecla(m, id, Key::F(8), mods));
        assert_eq!(m.docs().selection_of(id), [palavra(w)], "{w}");
        m.pinta(id);
        let realce: Vec<usize> = m
            .docs()
            .band_of(id)
            .unwrap()
            .problems
            .iter()
            .filter(|p| p.selected)
            .map(|p| p.index)
            .collect();
        let k = ordem.iter().position(|o| *o == w).unwrap();
        assert_eq!(realce, [k], "a linha realçada de {w}");
    };
    for w in ordem {
        passo(&mut m, Modifiers::default(), w);
    }
    passo(&mut m, Modifiers::default(), "pirnt");
    passo(&mut m, shift(), "nota");
    passo(&mut m, shift(), "outro");
}

/// (f) Sem problemas: a frase, e o separador sem contagem.
#[test]
fn a_lista_vazia_diz_que_nao_ha_problemas() {
    let (mut m, ids) = Mesa::new(&[("ok.luau", "local x = 1\nprint(x)\n")]);
    let id = ids[0];
    m.pede(id);
    assert!(tecla(&mut m, id, Key::Char('m'), ctrl_shift()));
    m.pinta(id);
    let banda = m.docs().band_of(id).unwrap();
    assert_eq!(banda.empty.as_deref(), Some("No problems in this script."));
    assert_eq!(banda.tabs[1].0, "Problems");
    assert!(banda.problems.is_empty());
    assert!(m.docs().problems_of(id).is_empty());
}

/// ⭐ (g) O leitor de ecrã: com a lista aberta, uma `List` «Problems: 4» filha do `Document` com um
/// `ListItem` clicável por linha (o molde do VS Code), o realçado pelo F8 seleccionado; os dois
/// separadores são `Tab`; o contador é um `Button` «Errors: 1, Warnings: 2».
#[test]
fn o_leitor_de_ecra_le_a_lista_os_separadores_e_o_contador() {
    let (mut m, id) = mesa();
    assert!(tecla(&mut m, id, Key::Char('m'), ctrl_shift()));
    assert!(tecla(&mut m, id, Key::F(8), Modifiers::default()));
    m.pinta(id);
    let nos = m.docs().a11y(id.0);
    let doc = &nos[0].1;
    assert_eq!(doc.role(), Role::Document);
    let filho = |r: Role| {
        nos.iter()
            .filter(|(n, x)| x.role() == r && doc.children().contains(&(*n).into()))
            .map(|(n, x)| (*n, x.clone()))
            .collect::<Vec<_>>()
    };
    let listas = filho(Role::List);
    assert_eq!(listas.len(), 1);
    let lista = &listas[0].1;
    assert_eq!(lista.label(), Some("Problems: 4"));
    let itens: Vec<_> = lista
        .children()
        .iter()
        .map(|c| {
            nos.iter()
                .find(|(n, _)| ph2d_a11y::AkNodeId::from(*n) == *c)
                .unwrap()
                .1
                .clone()
        })
        .collect();
    assert_eq!(itens.len(), 4);
    assert!(itens.iter().all(|n| n.role() == Role::ListItem));
    assert!(itens.iter().all(|n| n.supports_action(Action::Click)));
    assert_eq!(
        itens[0].label(),
        Some("Error: \u{201c}pirnt\u{201d} doesn\u{2019}t exist, line 3")
    );
    assert!(itens[1].label().unwrap().starts_with("Warning: "));
    assert!(itens[3].label().unwrap().starts_with("Info: "));
    assert_eq!(itens[0].is_selected(), Some(true), "o F8 está no 1.º");
    assert_eq!(itens[1].is_selected(), Some(false));

    let seps = filho(Role::Tab);
    let rotulos: Vec<_> = seps
        .iter()
        .map(|(_, n)| (n.label().unwrap().to_owned(), n.is_selected()))
        .collect();
    assert_eq!(
        rotulos,
        [
            ("Console".to_owned(), Some(false)),
            ("Problems (4)".to_owned(), Some(true))
        ]
    );
    let botoes: Vec<_> = filho(Role::Button)
        .into_iter()
        .filter(|(_, n)| n.label() == Some("Errors: 1, Warnings: 2"))
        .collect();
    assert_eq!(botoes.len(), 1, "o contador");
    assert!(botoes[0].1.supports_action(Action::Click));

    // Fechada a faixa, a lista sai da árvore (os separadores ficam).
    assert!(tecla(&mut m, id, Key::Char('m'), ctrl_shift()));
    m.pinta(id);
    let nos = m.docs().a11y(id.0);
    assert!(nos.iter().all(|(_, n)| n.role() != Role::List));
    assert_eq!(nos.iter().filter(|(_, n)| n.role() == Role::Tab).count(), 2);
}

/// (h) O ponteiro é a mão sobre o que um clique leva a algum sítio: uma linha da lista, o contador,
/// os separadores, e uma linha do console que tem número; a seta no resto da faixa.
#[test]
fn a_mao_sobre_as_linhas_o_contador_e_os_separadores() {
    let (mut m, id) = mesa();
    assert!(tecla(&mut m, id, Key::Char('m'), ctrl_shift()));
    m.pinta(id);
    let olha = |m: &mut Mesa, (x, y): (f32, f32)| m.docs().pointer_look(id.0, x, y);
    let banda = m.docs().band_of(id).unwrap();
    assert_eq!(
        olha(&mut m, centro(banda.problems[1].rect)),
        Some(PointerLook::Hand)
    );
    assert_eq!(
        olha(&mut m, centro(banda.tabs[0].1)),
        Some(PointerLook::Hand)
    );
    assert_eq!(
        olha(&mut m, centro(banda.tabs[1].1)),
        Some(PointerLook::Hand)
    );
    let c = m.docs().problems_counter_of(id).unwrap();
    assert_eq!(olha(&mut m, centro(c)), Some(PointerLook::Hand));
    // Por baixo da última linha da lista, ainda na faixa: a seta.
    let ult = banda.problems.last().unwrap().rect;
    assert_eq!(
        olha(&mut m, (ult.x + ult.w * 0.5, ult.y + ult.h * 1.5)),
        Some(PointerLook::Arrow)
    );
    // O texto continua a ser o I.
    let (x, y) = m.docs().glyph_center(id, palavra("aviso").start).unwrap();
    assert_eq!(olha(&mut m, (x, y)), Some(PointerLook::Text));
}

/// (h) Uma linha do console com número (o erro de runtime) é a mão; uma sem número é a seta.
#[test]
fn a_mao_sobre_a_linha_do_console_que_leva_ao_script() {
    const PARTE: &str =
        "print(\"oi\")\nfunction update(self, dt)\n  local t = nil\n  t.x = 1\nend\n";
    let (m, ids) = Mesa::new(&[("parte.luau", PARTE)]);
    let id = ids[0];
    let mut m = m.com_cena(id, 1);
    m.pede(id);
    m.cena();
    m.quadro();
    assert!(m.docs().console_open(id), "o erro abre a faixa");
    assert_eq!(m.docs().band_tab_of(id), BandTab::Console);
    m.pinta(id);
    let linhas = m.docs().console_texts(id);
    assert_eq!(linhas.len(), 2, "{linhas:?}");
    let sem = m.docs().console_row_rect(id, 0).expect("o print");
    let com = m.docs().console_row_rect(id, 1).expect("o erro");
    assert_eq!(
        m.docs().pointer_look(id.0, centro(sem).0, centro(sem).1),
        Some(PointerLook::Arrow)
    );
    assert_eq!(
        m.docs().pointer_look(id.0, centro(com).0, centro(com).1),
        Some(PointerLook::Hand)
    );
}
