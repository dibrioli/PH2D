//! I4 (Onda 2, `docs/CodeEditor/03_onda2.md`): a lista de completação. A lei do filtro e da ordem é a
//! do `CodeEdit` do Godot, corrido sobre entradas nossas (`fixtures/godot_completar.txt`); o resto
//! corre com o servidor FALSO de `assistente.rs` a responder `textDocument/completion` como o
//! `luau-lsp` 1.70.1 respondeu na sonda de 16/10 (snippets, `sortText`, sem filtrar).

use crate::assistente::{Partilhado, Plano, com_falso, letra, tecla};
use crate::{Mesa, ctrl};
use ph2d_app_code::complete::{Item, completable, order};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;
use ph2d_script::api_table::API;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

/// Como o `luau-lsp`: depois de `ph2d.` os membros da API (snippets, `sortText` 1); depois de
/// `self.` só os campos ATRIBUÍDOS (aqui, `t`); depois de uma aspa, nada; senão globais e palavras.
pub(crate) fn responde(texto: &str, b: usize) -> Value {
    let antes = &texto[..b.min(texto.len())];
    let ini = antes
        .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .map_or(0, |i| i + 1);
    let lead = &antes[..ini];
    let itens: Vec<Value> = if lead.ends_with("ph2d.") {
        API.iter()
            .filter(|e| e.table == "ph2d")
            .map(|e| {
                json!({ "label": e.ident, "kind": 3, "sortText": "1", "insertTextFormat": 2,
                        "insertText": format!("{}(${{1:a}})$0", e.ident), "detail": e.luau_type,
                        "documentation": { "kind": "markdown", "value": e.doc },
                        "command": { "title": "", "command": "editor.action.triggerParameterHints" } })
            })
            .collect()
    } else if lead.ends_with("self.") {
        vec![json!({ "label": "t", "kind": 5, "sortText": "1", "detail": "number" })]
    } else if lead.ends_with(['"', '\'']) {
        Vec::new()
    } else {
        vec![
            json!({ "label": "print", "kind": 3, "sortText": "4", "insertTextFormat": 2,
                    "insertText": "print($1)$0" }),
            json!({ "label": "pairs", "kind": 3, "sortText": "4" }),
            json!({ "label": "if", "kind": 14, "sortText": "8" }),
            json!({ "label": "local", "kind": 14, "sortText": "8" }),
        ]
    };
    json!({ "isIncomplete": false, "items": itens })
}

fn mesa(scripts: &[(&str, &str)]) -> (Mesa, Vec<ScriptId>, Partilhado) {
    com_falso(
        scripts,
        Plano {
            completar: Some(responde),
            ..Plano::default()
        },
    )
}

pub(crate) const T0: u128 = 1_000_000_000_000;
/// A pausa do `code_complete_delay` do Godot (I0b §7).
const PAUSA: u128 = 300_000_000;

pub(crate) fn escreve(m: &mut Mesa, id: ScriptId, s: &str) {
    for c in s.chars() {
        letra(m, id, c);
    }
}

pub(crate) fn quadro_em(m: &mut Mesa, t: u128) {
    m.hero.store.note_clock(t);
    m.quadro();
}

/// Escreve `s` no instante `T0`: a lista não abre antes da pausa e abre (com a resposta) depois.
pub(crate) fn abre(m: &mut Mesa, id: ScriptId, s: &str) -> Vec<String> {
    quadro_em(m, T0);
    escreve(m, id, s);
    // Um instante antes da pausa, com tempo de sobra para uma resposta chegar: fechada.
    for _ in 0..25 {
        quadro_em(m, T0 + PAUSA - 1);
        assert_eq!(
            m.docs().completion_of(id),
            None,
            "«{s}»: abriu antes da pausa"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    let fim = Instant::now() + Duration::from_secs(10);
    loop {
        quadro_em(m, T0 + PAUSA);
        if let Some((l, _)) = m.docs().completion_of(id) {
            return l;
        }
        assert!(Instant::now() < fim, "«{s}»: a lista não abriu");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn api_ph2d() -> Vec<&'static str> {
    API.iter()
        .filter(|e| e.table == "ph2d")
        .map(|e| e.ident)
        .collect()
}

/// ⭐ Oráculo: para cada linha da fixtura do Godot, a nossa lista é a dele, pela mesma ordem.
#[test]
fn a_ordem_da_lista_e_a_do_godot() {
    let fx = include_str!("fixtures/godot_completar.txt");
    let nomes: Vec<&str> = fx
        .lines()
        .find_map(|l| l.strip_prefix("# nomes: "))
        .expect("cabeçalho com os nomes")
        .split_whitespace()
        .collect();
    let items: Vec<Item> = nomes
        .iter()
        .map(|n| Item {
            label: (*n).to_owned(),
            kind: Some(3),
            detail: None,
            documentation: None,
            sort_text: String::new(),
            filter: None,
            insert: (*n).to_owned(),
            snippet: false,
            signature: false,
            usage: 0,
        })
        .collect();
    let mut casos = 0;
    for l in fx.lines().filter(|l| !l.starts_with('#')) {
        let (escrito, godot) = l.split_once('\t').expect("<escrito>\\t<lista>");
        let godot: Vec<&str> = godot.split_whitespace().collect();
        let nossa: Vec<&str> = if completable(escrito) {
            order(escrito, &items)
                .iter()
                .map(|(i, _)| items[*i].label.as_str())
                .collect()
        } else {
            Vec::new()
        };
        assert_eq!(nossa, godot, "«{escrito}»");
        casos += 1;
    }
    assert!(casos >= 20, "a fixtura perdeu casos: {casos}");
}

/// ⭐ A lista abre na PAUSA depois de `ph2d.`, com os membros da API; escrever filtra no cliente
/// (um só pedido ao servidor), e quando nada casa a lista fecha.
#[test]
fn a_lista_abre_na_pausa_e_filtra_enquanto_se_escreve() {
    let (mut m, ids, visto) = mesa(&[("a.luau", "")]);
    let id = ids[0];
    m.pede(id);
    let l = abre(&mut m, id, "ph2d.");
    let mut esperado = api_ph2d();
    let mut visto_l: Vec<&str> = l.iter().map(String::as_str).collect();
    esperado.sort_unstable();
    visto_l.sort_unstable();
    assert_eq!(visto_l, esperado);
    escreve(&mut m, id, "s");
    m.quadro();
    let (l, sel) = m.docs().completion_of(id).expect("continua aberta");
    assert_eq!(sel, 0);
    assert!(l.iter().all(|x| x.contains('s')), "{l:?}");
    escreve(&mut m, id, "et");
    m.quadro();
    let (l, _) = m.docs().completion_of(id).expect("continua aberta");
    assert_eq!(l[0], "set", "o prefixo exacto vem primeiro: {l:?}");
    assert_eq!(visto.lock().unwrap().pedidos, 1, "o filtro é do cliente");
    escreve(&mut m, id, "zz");
    m.quadro();
    assert_eq!(m.docs().completion_of(id), None, "nada casa: fecha");
}

/// Tab aceita: a chamada com os buracos do esqueleto da tabela (I12), o 1.º seleccionado, num passo
/// de undo; Enter também.
#[test]
fn tab_e_enter_aceitam_num_passo_de_undo() {
    let (mut m, ids, _) = mesa(&[("a.luau", ""), ("b.luau", "")]);
    for (id, k) in [(ids[0], Key::Tab), (ids[1], Key::Enter)] {
        m.pede(id);
        let _ = abre(&mut m, id, "ph2d.se");
        tecla(&mut m, id, k, Modifiers::default(), None, "");
        assert_eq!(
            m.docs().text_of(id).as_deref(),
            Some("ph2d.set(entity, \"x\", value)"),
            "{k:?}"
        );
        assert_eq!(m.docs().completion_of(id), None, "{k:?}: fechou");
        // O 1.º buraco seleccionado: o que se escreve a seguir entra lá.
        escreve(&mut m, id, "1");
        assert_eq!(
            m.docs().text_of(id).as_deref(),
            Some("ph2d.set(1, \"x\", value)"),
            "{k:?}"
        );
        tecla(&mut m, id, Key::Char('z'), ctrl(), Some("z"), "");
        tecla(&mut m, id, Key::Char('z'), ctrl(), Some("z"), "");
        assert_eq!(
            m.docs().text_of(id).as_deref(),
            Some("ph2d.se"),
            "{k:?}: um passo"
        );
    }
}

/// As setas andam e dão a volta nas pontas; Esc fecha sem mexer no texto.
#[test]
fn as_setas_andam_e_esc_fecha() {
    let (mut m, ids, _) = mesa(&[("a.luau", "")]);
    let id = ids[0];
    m.pede(id);
    let l = abre(&mut m, id, "ph2d.");
    let n = l.len();
    let sel = |m: &mut Mesa| m.docs().completion_of(id).map(|c| c.1);
    tecla(&mut m, id, Key::Down, Modifiers::default(), None, "");
    assert_eq!(sel(&mut m), Some(1));
    tecla(&mut m, id, Key::Up, Modifiers::default(), None, "");
    tecla(&mut m, id, Key::Up, Modifiers::default(), None, "");
    assert_eq!(sel(&mut m), Some(n - 1), "dá a volta");
    tecla(&mut m, id, Key::PageUp, Modifiers::default(), None, "");
    assert_eq!(sel(&mut m), Some(n - 1 - ph2d_panel_code::COMPLETION_ROWS));
    tecla(&mut m, id, Key::Escape, Modifiers::default(), None, "");
    assert_eq!(sel(&mut m), None);
    assert_eq!(m.docs().text_of(id).as_deref(), Some("ph2d."));
    // Uma tecla que não escreve nem anda na palavra fecha-a, mesmo com o cursor ainda dentro dela.
    let _ = abre(&mut m, id, "se");
    tecla(
        &mut m,
        id,
        Key::Left,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        None,
        "",
    );
    assert_eq!(sel(&mut m), None, "Shift+← (seleccionar) fecha");
}

/// Numa string e num comentário a lista não abre sozinha — nem com `.` (o servidor proporia a API)
/// nem com uma letra (proporia os globais).
#[test]
fn numa_string_ou_comentario_a_lista_nao_abre() {
    let (mut m, ids, visto) = mesa(&[("a.luau", "-- \nlocal s = \"\"\n")]);
    let id = ids[0];
    m.pede(id);
    let espera_fechada = |m: &mut Mesa, t: u128, onde: &str| {
        for _ in 0..25 {
            quadro_em(m, t);
            assert_eq!(m.docs().completion_of(id), None, "{onde}");
            std::thread::sleep(Duration::from_millis(2));
        }
    };
    // No fim do comentário: um `.` e depois só letras.
    tecla(&mut m, id, Key::Home, ctrl(), None, "");
    tecla(&mut m, id, Key::End, Modifiers::default(), None, "");
    for (k, s) in ["ph2d.", " pr"].iter().enumerate() {
        let t = T0 + PAUSA * 10 * k as u128;
        quadro_em(&mut m, t);
        escreve(&mut m, id, s);
        espera_fechada(&mut m, t + PAUSA * 2, &format!("comentário «{s}»"));
    }
    // Dentro da string, depois de um espaço (não logo a seguir à aspa).
    tecla(&mut m, id, Key::Down, Modifiers::default(), None, "");
    tecla(&mut m, id, Key::End, Modifiers::default(), None, "");
    tecla(&mut m, id, Key::Left, Modifiers::default(), None, "");
    for (k, s) in ["a ph2d.", " pr"].iter().enumerate() {
        let t = T0 + PAUSA * 10 * (2 + k as u128);
        quadro_em(&mut m, t);
        escreve(&mut m, id, s);
        espera_fechada(&mut m, t + PAUSA * 2, &format!("string «{s}»"));
    }
    assert_eq!(visto.lock().unwrap().pedidos, 0, "nem chegou a perguntar");
}

/// Depois de `self.` entram as propriedades `ph2d.property` do script (o servidor só conhece os
/// campos atribuídos).
#[test]
fn depois_de_self_entram_as_propriedades() {
    let texto = "ph2d.property(\"amplitude\", 1)\nph2d.property(\"speed\", 2)\n";
    let (mut m, ids, _) = mesa(&[("a.luau", texto)]);
    let id = ids[0];
    m.pede(id);
    tecla(&mut m, id, Key::End, ctrl(), None, "");
    let mut l = abre(&mut m, id, "self.");
    l.sort_unstable();
    assert_eq!(l, ["amplitude", "speed", "t"]);
}

/// A ordem segue o USO no projeto: os membros mais usados nos scripts vêm primeiro.
#[test]
fn a_ordem_segue_o_uso_no_projeto() {
    let outro = "ph2d.emit(\"a\")\nph2d.get(1, \"x\")\nph2d.get(1, \"y\")\nph2d.set(1, \"x\", 0)\nph2d.set(1, \"y\", 0)\nph2d.set(1, \"x\", 2)\n";
    let (mut m, ids, _) = mesa(&[("a.luau", ""), ("b.luau", outro)]);
    m.pede(ids[0]);
    let l = abre(&mut m, ids[0], "ph2d.");
    assert_eq!(&l[..3], ["set", "get", "emit"], "{l:?}");
}

/// Pintada: a lista desenha as sete primeiras linhas, um clique numa aceita-a, e a roda rola-a sem
/// mexer na seleção.
#[test]
fn a_lista_pinta_se_clica_se_e_rola() {
    let (mut m, ids, _) = mesa(&[("a.luau", "")]);
    let id = ids[0];
    m.pede(id);
    let l = abre(&mut m, id, "ph2d.");
    m.pinta(id);
    let p = m.docs().completion_painted(id).expect("pintada");
    assert_eq!(p.len(), ph2d_panel_code::COMPLETION_ROWS.min(l.len()));
    assert_eq!(
        p.iter().map(|r| r.1.as_str()).collect::<Vec<_>>(),
        l[..p.len()].iter().map(String::as_str).collect::<Vec<_>>()
    );
    assert!(p[0].2, "a 1.ª é a seleccionada");
    let (x, y) = p[0].3;
    let lh = 40.0;
    assert!(m.docs().wheel(id.0, x, y, -lh, false));
    m.pinta(id);
    let p2 = m.docs().completion_painted(id).expect("pintada");
    assert!(p2[0].0 > 0, "a roda rolou a lista");
    assert_eq!(
        m.docs().completion_of(id).map(|c| c.1),
        Some(0),
        "a seleção ficou"
    );
    let (alvo, nome) = (p2[1].3, p2[1].1.clone());
    m.clica(id, alvo.0, alvo.1, Modifiers::default());
    // A chamada entra com os buracos do esqueleto da tabela (I12): `ph2d.<nome>(…)`.
    let t = m.docs().text_of(id).unwrap_or_default();
    assert!(
        t.starts_with(&format!("ph2d.{nome}(")) && t.ends_with(')'),
        "o clique aceita a linha: {t}"
    );
}

/// O leitor de ecrã: a lista aberta é uma `ListBox` com uma opção por linha à vista, e a
/// seleccionada é o `active_descendant` do documento.
#[test]
fn a_lista_aberta_fala_ao_leitor_de_ecra() {
    use ph2d_a11y::Role;
    let (mut m, ids, _) = mesa(&[("a.luau", "")]);
    let id = ids[0];
    m.pede(id);
    let _ = abre(&mut m, id, "ph2d.");
    tecla(&mut m, id, Key::Down, Modifiers::default(), None, "");
    m.pinta(id);
    let nos = m.docs().a11y(id.0);
    let (lista, no) = nos
        .iter()
        .find(|(_, n)| n.role() == Role::ListBox)
        .expect("a ListBox");
    let opcoes: Vec<_> = nos
        .iter()
        .filter(|(_, n)| n.role() == Role::ListBoxOption)
        .collect();
    assert_eq!(no.children().len(), opcoes.len());
    assert_eq!(opcoes.len(), ph2d_panel_code::COMPLETION_ROWS);
    let sel: Vec<_> = opcoes
        .iter()
        .filter(|(_, n)| n.is_selected() == Some(true))
        .collect();
    assert_eq!(sel.len(), 1);
    let doc = &nos[0].1;
    assert_eq!(doc.role(), Role::Document);
    assert!(doc.children().contains(&(*lista).into()));
    assert_eq!(doc.active_descendant(), Some(sel[0].0.into()));
}

/// Sem assistente, a lista nunca abre.
#[test]
fn sem_assistente_a_lista_nao_abre() {
    let (mut m, ids) = Mesa::new(&[("a.luau", "")]);
    let id = ids[0];
    m.pede(id);
    quadro_em(&mut m, T0);
    escreve(&mut m, id, "ph2d.");
    for k in 1..5 {
        quadro_em(&mut m, T0 + PAUSA * k);
        assert_eq!(m.docs().completion_of(id), None);
    }
}

/// Ctrl+Espaço abre a lista NA HORA (sem a pausa), e também num comentário (pedida à mão).
#[test]
fn ctrl_espaco_abre_a_lista_na_hora() {
    let (mut m, ids, _) = mesa(&[("a.luau", ""), ("b.luau", "-- \n")]);
    for (id, antes) in [(ids[0], ""), (ids[1], "")] {
        m.pede(id);
        tecla(&mut m, id, Key::End, Modifiers::default(), None, antes);
        quadro_em(&mut m, T0);
        escreve(&mut m, id, "ph2d.se");
        tecla(&mut m, id, Key::Space, ctrl(), Some(" "), "");
        assert_eq!(
            m.docs().text_of(id).map(|t| t.contains("ph2d.se ")),
            Some(false),
            "não escreve o espaço"
        );
        let fim = Instant::now() + Duration::from_secs(10);
        loop {
            quadro_em(&mut m, T0);
            if let Some((l, _)) = m.docs().completion_of(id) {
                assert_eq!(l[0], "set");
                break;
            }
            assert!(
                Instant::now() < fim,
                "Ctrl+Espaço não abriu a lista sem a pausa"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
