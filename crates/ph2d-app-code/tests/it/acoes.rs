//! I7 (Onda 2, `docs/CodeEditor/03_onda2.md`): as acções de contexto — Alt+Enter. O `self.<campo>` que
//! ninguém dá é acusado por NÓS; o global errado pede as candidatas à completion do servidor; a acção
//! aplica-se num só passo de undo. O servidor é o FALSO de `assistente.rs`, a responder como o
//! `luau-lsp` 1.70.1 na sonda do I7 (`ferramentas/lspprobe_i7.py`), e o último teste corre o FIXADO.
//!
//! ⛔ Toda asserção de «NÃO abre / NÃO aplica» dá tempo REAL a uma resposta chegar ([`nunca`]).

use crate::Mesa;
use crate::assistente::{
    Partilhado, Plano, acusa_pirnt, com_falso, converge, letra, posicao, tecla,
};
use crate::completar::{T0, quadro_em};
use crate::definicao::{clica_em, cursor};
use crate::dicas::{ate, nunca};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;
use serde_json::{Value, json};
use std::time::Duration;

const TEXTO: &str = "ph2d.property(\"amplitude\", 1.0)\n\
function init(self)\n  \
self.base_x = ph2d.get(self.id, \"x\")\n\
end\n\
function update(self, dt)\n  \
local d = self.ampiltude * dt\n  \
ph2d.set(self.id, \"x\", self.base_x + d)\n  \
pirnt(d)\n\
end\n";

const SEG: u128 = 1_000_000_000;

fn alt() -> Modifiers {
    Modifiers {
        alt: true,
        ..Modifiers::default()
    }
}

fn alt_enter(m: &mut Mesa, id: ScriptId) {
    tecla(m, id, Key::Enter, alt(), None, "");
}

fn texto(m: &mut Mesa, id: ScriptId) -> String {
    m.docs().text_of(id).expect("aberto")
}

/// Os avisos de campo do `self` que a aba mostra (as mensagens).
fn campos(m: &mut Mesa, id: ScriptId) -> Vec<String> {
    m.docs()
        .problems_of(id)
        .into_iter()
        .map(|p| p.message)
        .filter(|s| s.contains("is never set"))
        .collect()
}

fn sem_assistente(texto: &str) -> (Mesa, ScriptId) {
    let (mut m, ids) = Mesa::new(&[("a.luau", texto)]);
    let id = ids[0];
    m.pede(id);
    quadro_em(&mut m, T0);
    m.pinta(id);
    (m, id)
}

/// ⭐ A prova do I7: `self.ampiltude` é acusado («did you mean amplitude?»), Alt+Enter mostra a acção
/// com a prévia, Enter corrige, e UM Ctrl+Z volta ao texto de antes. Controlo: `self.base_x` (dado no
/// `init`), `self.id` (do motor) e a `ph2d.property` lida noutro sítio não são acusados.
#[test]
fn ampiltude_pede_amplitude_e_alt_enter_corrige_num_passo() {
    let (mut m, id) = sem_assistente(TEXTO);
    let avisos = campos(&mut m, id);
    assert_eq!(avisos.len(), 1, "só o campo errado: {avisos:?}");
    assert!(
        avisos[0].contains("self.ampiltude") && avisos[0].contains("did you mean amplitude"),
        "{avisos:?}"
    );
    let p = m
        .docs()
        .problems_of(id)
        .into_iter()
        .find(|p| p.message.contains("is never set"))
        .unwrap();
    let errado = TEXTO.find("ampiltude").unwrap();
    assert_eq!(p.culprit, Some(errado..errado + "ampiltude".len()));
    assert_eq!(p.severity, ph2d_panel_code::Severity::Warning);

    clica_em(&mut m, id, errado + 3, Modifiers::default());
    alt_enter(&mut m, id);
    quadro_em(&mut m, T0);
    m.pinta(id);
    let (titulos, sel, previa) = m.docs().actions_of(id).expect("as acções abriram");
    assert_eq!(titulos, ["Did you mean amplitude?"]);
    assert_eq!(sel, 0);
    assert_eq!(previa, ["local d = self.amplitude * dt"]);
    let pintadas = m
        .docs()
        .completion_painted(id)
        .expect("a janelinha pintou-se");
    assert_eq!(pintadas[0].1, "Did you mean amplitude?");

    tecla(&mut m, id, Key::Enter, Modifiers::default(), None, "");
    let certo = TEXTO.replace("ampiltude", "amplitude");
    assert_eq!(texto(&mut m, id), certo);
    assert_eq!(cursor(&mut m, id), errado + "amplitude".len());
    assert_eq!(m.docs().actions_of(id), None, "aplicada, fecha");
    // O aviso larga-se na edição (o nome mudou) e não volta na pausa.
    assert!(campos(&mut m, id).is_empty());
    quadro_em(&mut m, T0 + 2 * SEG);
    assert!(campos(&mut m, id).is_empty());

    tecla(&mut m, id, Key::Char('z'), crate::ctrl(), None, "");
    assert_eq!(texto(&mut m, id), TEXTO, "um Ctrl+Z desfaz a acção inteira");
    tecla(&mut m, id, Key::Char('y'), crate::ctrl(), None, "");
    assert_eq!(texto(&mut m, id), certo);
}

/// O aviso espera a pausa da escrita (o `idle_parse_delay` do Godot, 1,5 s); apagar o nome errado
/// larga-o já.
#[test]
fn o_aviso_espera_a_pausa_da_escrita() {
    let certo = TEXTO.replace("ampiltude", "amplitude");
    let (mut m, id) = sem_assistente(&certo);
    assert!(campos(&mut m, id).is_empty(), "o texto certo não tem aviso");
    let fim = certo.find("amplitude *").unwrap() + "amplitude".len();
    // O centro do «e» final põe o cursor de um lado ou do outro dele.
    clica_em(&mut m, id, fim - 1, Modifiers::default());
    let c = cursor(&mut m, id);
    assert!((fim - 1..=fim).contains(&c), "{c}");
    if c < fim {
        tecla(&mut m, id, Key::Right, Modifiers::default(), None, "");
    }
    letra(&mut m, id, 'x');
    quadro_em(&mut m, T0 + SEG);
    // A árvore do texto novo já chegou: só a pausa o segura.
    m.docs().wait_grammar();
    quadro_em(&mut m, T0 + SEG + 1_400_000_000);
    assert!(campos(&mut m, id).is_empty(), "o aviso não espera a pausa");
    quadro_em(&mut m, T0 + SEG + 1_600_000_000);
    let avisos = campos(&mut m, id);
    assert!(
        avisos.len() == 1 && avisos[0].contains("self.amplitudex"),
        "{avisos:?}"
    );
    tecla(&mut m, id, Key::Backspace, Modifiers::default(), None, "");
    assert!(campos(&mut m, id).is_empty(), "apagar o nome larga o aviso");
}

/// Escrever ENCOSTADO ao nome errado (antes dele) também o larga: o aviso não fica agarrado às
/// letras velhas, e o Alt+Enter nunca troca metade de uma palavra.
#[test]
fn escrever_encostado_ao_nome_larga_o_aviso() {
    let (mut m, id) = sem_assistente(TEXTO);
    assert_eq!(campos(&mut m, id).len(), 1);
    let errado = TEXTO.find("ampiltude").unwrap();
    // O centro do «a» inicial: o cursor fica antes ou depois dele.
    clica_em(&mut m, id, errado, Modifiers::default());
    if cursor(&mut m, id) > errado {
        tecla(&mut m, id, Key::Left, Modifiers::default(), None, "");
    }
    letra(&mut m, id, 'x');
    assert!(
        campos(&mut m, id).is_empty(),
        "o aviso ficou agarrado a `xampiltude`"
    );
    alt_enter(&mut m, id);
    quadro_em(&mut m, T0);
    let (titulos, ..) = m.docs().actions_of(id).expect("abriu");
    assert_eq!(titulos, ["No quick fixes here"]);
}

/// Sem nada para corrigir, a janelinha di-lo e o Enter fecha-a sem mexer no texto; uma tecla de
/// fora fecha as acções e escreve.
#[test]
fn sem_nada_a_corrigir_a_janelinha_diz_e_uma_tecla_fecha() {
    let (mut m, id) = sem_assistente(TEXTO);
    let dt = TEXTO.find("dt)").unwrap();
    clica_em(&mut m, id, dt, Modifiers::default());
    alt_enter(&mut m, id);
    quadro_em(&mut m, T0);
    let (titulos, _, previa) = m.docs().actions_of(id).expect("abriu");
    assert_eq!(titulos, ["No quick fixes here"]);
    assert!(previa.is_empty());
    tecla(&mut m, id, Key::Enter, Modifiers::default(), None, "");
    assert_eq!(texto(&mut m, id), TEXTO);
    assert_eq!(m.docs().actions_of(id), None);
    // Abertas outra vez sobre o campo errado: uma letra fecha-as e é escrita.
    let errado = TEXTO.find("ampiltude").unwrap();
    clica_em(&mut m, id, errado + 3, Modifiers::default());
    alt_enter(&mut m, id);
    quadro_em(&mut m, T0);
    assert!(m.docs().actions_of(id).is_some());
    let antes = cursor(&mut m, id);
    letra(&mut m, id, 'q');
    assert_eq!(m.docs().actions_of(id), None, "a letra fecha-as");
    assert_eq!(cursor(&mut m, id), antes + 1, "e é escrita");
    tecla(&mut m, id, Key::Escape, Modifiers::default(), None, "");
    alt_enter(&mut m, id);
    tecla(&mut m, id, Key::Escape, Modifiers::default(), None, "");
    assert_eq!(m.docs().actions_of(id), None, "Esc fecha-as");
}

/// A completion do servidor no início do nome errado: as candidatas do «você quis dizer».
fn candidatas(_texto: &str, _b: usize) -> Value {
    let item = |l: &str, k: u32| json!({ "label": l, "kind": k });
    json!({ "items": [item("pairs", 3), item("print", 3), item("if", 14), item("dt", 6), item("d", 6)] })
}

/// Como o `luau-lsp`: uma quickfix que ECOA o diagnóstico mandado e outra (a do `FunctionUnused`
/// num gancho) que vem com `diagnostics: []` — essa a aba não mostra.
fn agir(uri: &str, texto: &str, b: usize, ctx: &[Value]) -> Value {
    let edit = |s: &str| {
        let at = posicao(texto, b);
        let mut changes = serde_json::Map::new();
        changes.insert(
            uri.to_owned(),
            json!([{ "range": { "start": at, "end": at }, "newText": s }]),
        );
        json!({ "changes": changes })
    };
    json!([
        { "title": "Prefix 'init' with '_' to silence", "kind": "quickfix", "diagnostics": [],
          "edit": edit("_") },
        { "title": "Do server", "kind": "quickfix", "diagnostics": ctx, "edit": edit("--") },
        { "title": "Extract to function", "kind": "refactor.extract", "diagnostics": ctx }
    ])
}

fn com_servidor(atraso: Duration) -> (Mesa, ScriptId, Partilhado) {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", TEXTO)],
        Plano {
            atraso,
            regra: Some(acusa_pirnt),
            completar: Some(candidatas),
            agir: Some(agir),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    ate(&mut m, id, T0, "o 1001 não chegou à aba", |m| {
        m.docs()
            .problems_of(id)
            .iter()
            .any(|p| p.message.contains("Unknown global 'pirnt'"))
    });
    (m, id, visto)
}

/// ⭐ O global errado (`1001`): as candidatas vêm da completion do servidor (sem as palavras
/// reservadas), a quickfix que ecoa o diagnóstico vem a seguir, a que não ecoa não vem. Enter aplica
/// a 1.ª num passo de undo.
#[test]
fn o_global_errado_pede_as_candidatas_ao_servidor() {
    let (mut m, id, visto) = com_servidor(Duration::ZERO);
    let errado = TEXTO.find("pirnt").unwrap();
    clica_em(&mut m, id, errado + 2, Modifiers::default());
    let c = cursor(&mut m, id) - errado;
    alt_enter(&mut m, id);
    ate(
        &mut m,
        id,
        T0,
        "as acções do servidor não chegaram",
        |m| m.docs().actions_of(id).is_some(),
    );
    let (titulos, _, previa) = m.docs().actions_of(id).unwrap();
    assert_eq!(titulos, ["Did you mean print?", "Do server"]);
    assert_eq!(previa, ["print(d)"]);
    {
        let v = visto.lock().unwrap();
        assert_eq!((v.acoes, v.pedidos), (1, 1), "uma pergunta de cada género");
    }
    tecla(&mut m, id, Key::Down, Modifiers::default(), None, "");
    let (_, sel, previa) = m.docs().actions_of(id).unwrap();
    let marcado = format!("{}--{}(d)", &"pirnt"[..c], &"pirnt"[c..]);
    assert_eq!(
        (sel, previa),
        (1, vec![marcado]),
        "a prévia segue a seleccionada"
    );
    tecla(&mut m, id, Key::Up, Modifiers::default(), None, "");
    tecla(&mut m, id, Key::Enter, Modifiers::default(), None, "");
    let certo = TEXTO.replace("pirnt", "print");
    assert_eq!(texto(&mut m, id), certo);
    tecla(&mut m, id, Key::Char('z'), crate::ctrl(), None, "");
    assert_eq!(texto(&mut m, id), TEXTO, "um Ctrl+Z desfaz a acção inteira");
}

/// O clique numa linha da janelinha aplica-a, como o Enter.
#[test]
fn o_clique_numa_accao_aplica_a() {
    let (mut m, id, _visto) = com_servidor(Duration::ZERO);
    let errado = TEXTO.find("pirnt").unwrap();
    clica_em(&mut m, id, errado + 2, Modifiers::default());
    alt_enter(&mut m, id);
    ate(&mut m, id, T0, "as acções não chegaram", |m| {
        m.docs()
            .completion_painted(id)
            .is_some_and(|r| r.len() == 2)
    });
    let linhas = m.docs().completion_painted(id).unwrap();
    let (x, y) = linhas[0].3;
    m.clica(id, x, y, Modifiers::default());
    assert_eq!(texto(&mut m, id), TEXTO.replace("pirnt", "print"));
}

/// Uma edição que NÃO é tecla (um acento composto) com as perguntas em voo fecha as acções: as
/// respostas que chegam depois não abrem nada nem mexem no texto.
#[test]
fn uma_composicao_com_as_accoes_em_voo_fecha_as() {
    use ph2d_editor_core::doc_surface::{Compose, DocumentSurface};
    let (mut m, id, visto) = com_servidor(Duration::from_millis(40));
    let errado = TEXTO.find("pirnt").unwrap();
    clica_em(&mut m, id, errado + 2, Modifiers::default());
    alt_enter(&mut m, id);
    // O quadro que pergunta; a composição logo a seguir, com as perguntas em voo.
    quadro_em(&mut m, T0);
    assert!(m.docs().compose(id.0, Compose::Commit('é')));
    let depois = texto(&mut m, id);
    nunca(
        &mut m,
        id,
        T0,
        "uma resposta velha abriu as acções",
        |m| m.docs().actions_of(id).is_none() && m.docs().completion_painted(id).is_none(),
    );
    assert_eq!(texto(&mut m, id), depois);
    assert!(visto.lock().unwrap().acoes <= 1);
}

/// ⭐ Com o `luau-lsp` FIXADO, pelo caminho do produto: `pirnt` → «Did you mean print?» e
/// `ph2d.proprety` → «Did you mean property?» (as candidatas da completion dele); num `local` com
/// tipo, «Add type: boolean».
#[test]
fn o_assistente_fixado_corrige_com_alt_enter() {
    let _ = crate::oraculo_lsp::fixado();
    let fonte = "ph2d.proprety(\"speed\", 2.0)\nfunction update(self, dt)\n  local up = dt > 0.5\n  pirnt(up)\nend\n";
    let (m, ids) = Mesa::new(&[("a.luau", fonte)]);
    let mut m = m.com_assistente(ph2d_app_code::IntelServer::Pinned);
    let id = ids[0];
    m.pede(id);
    let espera = |m: &mut Mesa, onde: &str, feito: &dyn Fn(&mut Mesa) -> bool| {
        let fim = std::time::Instant::now() + Duration::from_secs(30);
        loop {
            quadro_em(m, T0);
            m.pinta(id);
            if feito(m) {
                return;
            }
            assert!(std::time::Instant::now() < fim, "{onde}");
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    espera(&mut m, "os diagnósticos do luau-lsp não chegaram", &|m| {
        let ps = m.docs().problems_of(id);
        ps.iter()
            .any(|p| p.message.contains("Unknown global 'pirnt'"))
            && ps.iter().any(|p| p.message.contains("proprety"))
    });
    for (agulha, off, titulo) in [
        ("pirnt", 2, "Did you mean print?"),
        ("proprety", 3, "Did you mean property?"),
        ("up =", 1, "Add type: boolean"),
        // O gancho do motor: o servidor dá «Prefix 'update' with '_'» ao lint que a aba esconde.
        ("update", 2, "No quick fixes here"),
    ] {
        let b = fonte.find(agulha).unwrap() + off;
        clica_em(&mut m, id, b, Modifiers::default());
        alt_enter(&mut m, id);
        espera(
            &mut m,
            &format!("Alt+Enter em `{agulha}` não abriu"),
            &|m| m.docs().actions_of(id).is_some(),
        );
        let (titulos, ..) = m.docs().actions_of(id).unwrap();
        assert!(titulos.iter().any(|t| t == titulo), "{agulha}: {titulos:?}");
        assert!(
            !titulos.iter().any(|t| t.starts_with("Prefix")),
            "{agulha}: o aviso que a aba esconde (o gancho) não dá correcção: {titulos:?}"
        );
        tecla(&mut m, id, Key::Escape, Modifiers::default(), None, "");
    }
}
