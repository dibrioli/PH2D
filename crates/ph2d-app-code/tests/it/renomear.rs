//! I10 (Onda 2, `docs/CodeEditor/03_onda2.md`): renomear (F2), as referências, e o marcador do undo
//! global. Medição: `ferramentas/medicoes_onda2/I10_rename_e_undo.md`.
//!
//! A fila global é a da shell em pequeno ([`Fila`]): um degrau por diff, o `own` de cada objecto de
//! antes, e as decisões do elo pelas MESMAS portas que a shell chama (`take_ask`, `begin_undo`,
//! `end_undo`, `publish` de `ph2d_editor_core::undo_link`). A shell sem janela não desfaz (o `gfx` é
//! `None`): a parte dela é só chamar estas portas.

use crate::Mesa;
use crate::assistente::{Partilhado, Plano, com_falso, converge, tecla};
use crate::completar::{T0, quadro_em};
use crate::ctrl;
use crate::definicao::{clica_em, cursor};
use crate::dicas::{ate, nunca};
use ph2d_ecs::StableId;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_editor_core::undo_link::UndoLink;
use ph2d_host::Modifiers;
use ph2d_script::props::{Origin, resolve};
use ph2d_script::{LuauScript, ScriptId, ScriptValue};
use std::collections::BTreeMap;
use std::ops::Range;

const TEXTO: &str = "ph2d.property(\"amplitude\", 1.0)\n\
ph2d.property(\"speed\", 2.0)\n\
local brilho = 2\n\
function update(self, dt)\n  \
local amplitude = \"amplitude\"\n  \
ph2d.set(self.id, \"x\", self.amplitude + self[\"amplitude\"] + self.speed * brilho)\n\
end\n";

const RENOMEADO: &str = "ph2d.property(\"altura\", 1.0)\n\
ph2d.property(\"speed\", 2.0)\n\
local brilho = 2\n\
function update(self, dt)\n  \
local amplitude = \"amplitude\"\n  \
ph2d.set(self.id, \"x\", self.altura + self[\"altura\"] + self.speed * brilho)\n\
end\n";

type Owns = Vec<(u64, BTreeMap<String, ScriptValue>)>;

fn owns(m: &mut Mesa) -> Owns {
    let w = m.sim.world_mut();
    let mut q = w.query::<(&StableId, &LuauScript)>();
    let mut out: Owns = q.iter(w).map(|(s, c)| (s.0, c.own.clone())).collect();
    out.sort_by_key(|(s, _)| *s);
    out
}

fn set_owns(m: &mut Mesa, o: &Owns) {
    let w = m.sim.world_mut();
    let mut q = w.query::<(&StableId, &mut LuauScript)>();
    for (s, mut c) in q.iter_mut(w) {
        if let Some((_, own)) = o.iter().find(|(k, _)| *k == s.0) {
            c.own.clone_from(own);
        }
    }
}

/// A fila global da shell, em pequeno (ver o cabeçalho).
struct Fila {
    undo: Vec<(Owns, Option<UndoLink>)>,
    redo: Vec<(Owns, Option<UndoLink>)>,
    base: Owns,
}

impl Fila {
    fn new(m: &mut Mesa) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            base: owns(m),
        }
    }

    /// O fim do quadro da shell (`post_frame_undo`): o pedido da aba, ou um degrau se o documento
    /// mudou ou há um elo pendente; depois publica.
    fn fim(&mut self, m: &mut Mesa) {
        if let Some(Ok(redo)) = m.hero.undo_links.take_ask() {
            self.aplica(m, redo);
        } else {
            let agora = owns(m);
            let pending = m.hero.undo_links.pending.take();
            if agora != self.base || pending.is_some() {
                self.undo
                    .push((std::mem::replace(&mut self.base, agora), pending));
                self.redo.clear();
            }
        }
        self.publica(m);
    }

    fn publica(&self, m: &mut Mesa) {
        let top = [
            self.undo.last().and_then(|s| s.1),
            self.redo.last().and_then(|s| s.1),
        ];
        let alive: Vec<UndoLink> = self
            .undo
            .iter()
            .chain(&self.redo)
            .filter_map(|s| s.1)
            .collect();
        m.hero.undo_links.publish(top, alive);
    }

    /// Ctrl+Z (ou Ctrl+Shift+Z) com a CENA em foco.
    fn desfaz(&mut self, m: &mut Mesa, redo: bool) {
        self.aplica(m, redo);
        self.publica(m);
    }

    fn aplica(&mut self, m: &mut Mesa, redo: bool) {
        let side = if redo { &self.redo } else { &self.undo };
        let top = side.last().and_then(|s| s.1);
        if side.is_empty() || !m.hero.undo_links.begin_undo(top, redo) {
            return;
        }
        let agora = owns(m);
        let (estado, link) = if redo {
            let s = self.redo.pop().unwrap();
            self.undo.push((agora, s.1));
            s
        } else {
            let s = self.undo.pop().unwrap();
            self.redo.push((agora, s.1));
            s
        };
        set_owns(m, &estado);
        self.base = estado;
        m.hero.undo_links.end_undo(link, redo);
    }

    /// Mexer na cena (um valor do Inspector noutro objecto): um degrau sem elo.
    fn mexe(&mut self, m: &mut Mesa) {
        let mut o = owns(m);
        o[1].1.insert("speed".into(), ScriptValue::Number(9.0));
        set_owns(m, &o);
        self.fim(m);
    }
}

/// Um quadro da aplicação: a família, e o fim do quadro da shell.
fn passo(m: &mut Mesa, f: &mut Fila) {
    m.quadro();
    f.fim(m);
}

/// Dois objectos a correr o script: o 1.º com `amplitude = 3`, o 2.º com `speed = 5`.
fn mesa() -> (Mesa, ScriptId, Fila) {
    let (m, ids) = Mesa::new(&[("bob.luau", TEXTO)]);
    let id = ids[0];
    let mut m = m.com_cena(id, 2);
    let mut o = owns(&mut m);
    o[0].1.insert("amplitude".into(), ScriptValue::Number(3.0));
    o[1].1.insert("speed".into(), ScriptValue::Number(5.0));
    set_owns(&mut m, &o);
    m.pede(id);
    m.quadro();
    m.pinta(id);
    let f = Fila::new(&mut m);
    (m, id, f)
}

fn f2(m: &mut Mesa, id: ScriptId) {
    tecla(m, id, Key::F(2), Modifiers::default(), None, "");
}

fn enter(m: &mut Mesa, id: ScriptId) {
    tecla(m, id, Key::Enter, Modifiers::default(), Some("\r"), "");
}

fn ctrl_z(m: &mut Mesa, id: ScriptId, redo: bool) {
    let mods = Modifiers {
        shift: redo,
        ..ctrl()
    };
    tecla(m, id, Key::Char('z'), mods, None, "");
}

fn amplitude(o: &Owns) -> (Option<f64>, Option<f64>) {
    let n = |k: &str| match o[0].1.get(k) {
        Some(ScriptValue::Number(v)) => Some(*v),
        _ => None,
    };
    (n("amplitude"), n("altura"))
}

/// O Inspector: as declarações da biblioteca (pela VM, como o cartão) contra o `own` do 1.º objecto.
fn inspector(m: &mut Mesa, id: ScriptId) -> Vec<(String, Origin)> {
    m.cena();
    let o = owns(m);
    let decls = m
        .host
        .as_ref()
        .and_then(|h| h.scene_info(id))
        .and_then(ph2d_script::ScriptInfo::decls)
        .expect("o script compila")
        .to_vec();
    resolve(Some(&decls), &o[0].1)
        .values
        .into_iter()
        .map(|v| (v.name, v.origin))
        .collect()
}

/// ⭐⭐ Renomear uma `ph2d.property` (F2 sobre `self.amplitude`): a declaração, `self.amplitude` e
/// `self["amplitude"]` mudam; o local `amplitude` e a string `"amplitude"` não (zero falso positivo).
/// A biblioteca muda com a aba (a aba fica limpa), e o valor que o 1.º objecto guardava vai para o
/// nome novo: o Inspector continua a mostrá-lo como PRÓPRIO. Um passo só, ligado à fila global.
#[test]
fn renomear_uma_propriedade_leva_o_valor_de_cada_objecto() {
    let (mut m, id, mut f) = mesa();
    assert!(inspector(&mut m, id).contains(&("amplitude".into(), Origin::Own)));
    let at = TEXTO.find("self.amplitude").unwrap() + 7;
    clica_em(&mut m, id, at, Modifiers::default());
    f2(&mut m, id);
    passo(&mut m, &mut f);
    let (campo, ajuda, razao) = m.docs().rename_of(id).expect("a caixinha abriu");
    assert_eq!(
        campo.as_deref(),
        Some("amplitude"),
        "vem com o nome escrito"
    );
    assert!(
        ajuda.contains("1 object"),
        "diz quantos objectos guardam um valor: {ajuda}"
    );
    assert_eq!(razao, None);
    // Escrever substitui o nome todo (nasce seleccionado).
    m.escreve(id, "altura");
    enter(&mut m, id);
    passo(&mut m, &mut f);
    assert_eq!(m.docs().rename_of(id), None, "Enter fecha a caixinha");
    assert_eq!(m.docs().text_of(id).unwrap(), RENOMEADO);
    assert_eq!(m.texto_salvo(id), RENOMEADO, "a biblioteca muda com a aba");
    assert!(!m.docs().is_dirty(id), "a aba e a biblioteca concordam");
    let o = owns(&mut m);
    assert_eq!(amplitude(&o), (None, Some(3.0)), "o valor vai com o nome");
    assert_eq!(
        o[1].1.get("speed"),
        Some(&ScriptValue::Number(5.0)),
        "o resto fica"
    );
    assert!(inspector(&mut m, id).contains(&("altura".into(), Origin::Own)));
    assert_eq!(f.undo.len(), 1, "UM degrau global");
    let link = f.undo[0].1.expect("com elo");
    assert_eq!(
        m.docs().top_link_of(id, false),
        Some(link.0),
        "o grupo do texto leva o mesmo elo"
    );
    // O cursor fica no nome renomeado.
    let c = cursor(&mut m, id);
    let s = RENOMEADO.find("self.altura").unwrap() + 5;
    assert!((s + 2..=s + 3).contains(&c), "cursor em {c}, o nome em {s}");
}

fn renomeado(m: &mut Mesa, id: ScriptId, f: &mut Fila) {
    let at = TEXTO.find("ph2d.property(\"amplitude\"").unwrap() + 17;
    clica_em(m, id, at, Modifiers::default());
    f2(m, id);
    passo(m, f);
    m.escreve(id, "altura");
    enter(m, id);
    passo(m, f);
    assert_eq!(m.docs().text_of(id).unwrap(), RENOMEADO);
}

fn estado(m: &mut Mesa, id: ScriptId) -> (String, String, bool, (Option<f64>, Option<f64>)) {
    let o = owns(m);
    (
        m.docs().text_of(id).unwrap(),
        m.texto_salvo(id),
        m.docs().is_dirty(id),
        amplitude(&o),
    )
}

/// ⭐⭐⭐ O marcador: Ctrl+Z NA ABA desfaz o texto, a biblioteca e as chaves `own` juntos; Ctrl+Shift+Z
/// refá-los; e Ctrl+Z NA CENA (a fila global) faz o mesmo. Nunca um passo a mais: depois de desfeito,
/// o Ctrl+Z seguinte da aba já não toca no rename.
#[test]
fn ctrl_z_na_aba_e_na_cena_desfaz_o_texto_e_os_objectos_juntos() {
    let antes = (TEXTO.to_owned(), TEXTO.to_owned(), false, (Some(3.0), None));
    let depois = (
        RENOMEADO.to_owned(),
        RENOMEADO.to_owned(),
        false,
        (None, Some(3.0)),
    );
    let (mut m, id, mut f) = mesa();
    renomeado(&mut m, id, &mut f);
    assert_eq!(estado(&mut m, id), depois);
    // Na aba.
    ctrl_z(&mut m, id, false);
    passo(&mut m, &mut f); // o pedido chega à fila, que desfaz a cena
    passo(&mut m, &mut f); // a família lê o `done` e desfaz o texto
    assert_eq!(estado(&mut m, id), antes, "Ctrl+Z na aba");
    assert_eq!((f.undo.len(), f.redo.len()), (0, 1));
    ctrl_z(&mut m, id, true);
    passo(&mut m, &mut f);
    passo(&mut m, &mut f);
    assert_eq!(estado(&mut m, id), depois, "Ctrl+Shift+Z na aba");
    // Na cena.
    f.desfaz(&mut m, false);
    passo(&mut m, &mut f);
    assert_eq!(estado(&mut m, id), antes, "Ctrl+Z na cena");
    f.desfaz(&mut m, true);
    passo(&mut m, &mut f);
    assert_eq!(estado(&mut m, id), depois, "Ctrl+Shift+Z na cena");
    // Sem passo duplicado: desfeito, a aba já não tem o grupo do rename para desfazer.
    f.desfaz(&mut m, false);
    passo(&mut m, &mut f);
    ctrl_z(&mut m, id, false);
    passo(&mut m, &mut f);
    passo(&mut m, &mut f);
    assert_eq!(
        estado(&mut m, id),
        antes,
        "o 2.º Ctrl+Z não desfaz o rename outra vez"
    );
    assert_eq!(
        (f.undo.len(), f.redo.len()),
        (0, 1),
        "nem a fila global anda"
    );
}

/// ⭐⭐ Recusa, nunca meio passo (o VS Code desfaz só um lado e parte o projecto, medição §2): com uma
/// letra escrita na aba DEPOIS do rename, o Ctrl+Z na cena não anda e diz porquê; desfeita a letra,
/// anda. Com a cena mexida depois do rename, o Ctrl+Z da aba não anda; desfeita a cena, anda.
#[test]
fn com_a_outra_metade_atras_o_desfazer_recusa_por_inteiro() {
    let (mut m, id, mut f) = mesa();
    renomeado(&mut m, id, &mut f);
    let fim = RENOMEADO.len() - 1;
    clica_em(&mut m, id, fim - 4, Modifiers::default());
    crate::assistente::letra(&mut m, id, ' ');
    passo(&mut m, &mut f);
    let escrito = m.docs().text_of(id).unwrap();
    f.desfaz(&mut m, false);
    passo(&mut m, &mut f);
    assert_eq!(m.docs().text_of(id).unwrap(), escrito, "nada andou");
    assert_eq!(amplitude(&owns(&mut m)), (None, Some(3.0)));
    assert_eq!(f.undo.len(), 1, "o degrau continua lá");
    assert!(
        m.toasts.iter().any(|t| t.message.contains("script edits")),
        "o aviso diz porquê"
    );
    ctrl_z(&mut m, id, false); // a letra (um passo só da aba)
    passo(&mut m, &mut f);
    assert_eq!(m.docs().text_of(id).unwrap(), RENOMEADO);
    f.desfaz(&mut m, false);
    passo(&mut m, &mut f);
    assert_eq!(estado(&mut m, id).3, (Some(3.0), None), "agora anda");
    assert_eq!(m.docs().text_of(id).unwrap(), TEXTO);

    let (mut m, id, mut f) = mesa();
    renomeado(&mut m, id, &mut f);
    f.mexe(&mut m);
    ctrl_z(&mut m, id, false);
    passo(&mut m, &mut f);
    passo(&mut m, &mut f);
    assert_eq!(m.docs().text_of(id).unwrap(), RENOMEADO, "a aba não andou");
    assert!(
        m.toasts.iter().any(|t| t.message.contains("scene changes")),
        "o aviso diz porquê"
    );
    f.desfaz(&mut m, false); // a cena mexida
    passo(&mut m, &mut f);
    ctrl_z(&mut m, id, false);
    passo(&mut m, &mut f);
    passo(&mut m, &mut f);
    assert_eq!(m.docs().text_of(id).unwrap(), TEXTO, "agora anda");
    assert_eq!(amplitude(&owns(&mut m)), (Some(3.0), None));
}

/// Um elo que a fila global já não guarda (saiu pelo tecto, ou outro projecto) deixa o grupo como um
/// passo de texto: a aba desfá-lo sozinha.
#[test]
fn um_elo_morto_deixa_o_texto_desfazer_se_sozinho() {
    let (mut m, id, mut f) = mesa();
    renomeado(&mut m, id, &mut f);
    f.undo.clear();
    f.publica(&mut m);
    passo(&mut m, &mut f);
    assert_eq!(
        m.docs().top_link_of(id, false),
        None,
        "o grupo largou o elo"
    );
    ctrl_z(&mut m, id, false);
    passo(&mut m, &mut f);
    assert_eq!(m.docs().text_of(id).unwrap(), TEXTO);
}

/// A validação é NOSSA (o servidor aceita `end` e nomes que já existem, medição §1): a razão aparece
/// na caixinha, e o Enter não faz nada até o nome servir.
#[test]
fn um_nome_que_nao_serve_diz_porque_e_o_enter_nao_aplica() {
    let (mut m, id, mut f) = mesa();
    let at = TEXTO.find("self.amplitude").unwrap() + 7;
    clica_em(&mut m, id, at, Modifiers::default());
    f2(&mut m, id);
    passo(&mut m, &mut f);
    for (nome, razao) in [
        ("end", "reserved"),
        ("id", "reserved"),
        ("1x", "not a valid name"),
        ("speed", "already exists"),
        ("dt", ""),
    ] {
        tecla(&mut m, id, Key::Char('a'), ctrl(), None, "");
        m.escreve(id, nome);
        let (_, _, r) = m.docs().rename_of(id).unwrap();
        if razao.is_empty() {
            assert_eq!(r, None, "{nome}: um campo pode chamar-se como um parâmetro");
        } else {
            assert!(
                r.as_deref().is_some_and(|r| r.contains(razao)),
                "{nome}: {r:?}"
            );
            enter(&mut m, id);
            passo(&mut m, &mut f);
            assert_eq!(
                m.docs().text_of(id).unwrap(),
                TEXTO,
                "{nome}: o Enter não aplica"
            );
            assert!(m.docs().rename_of(id).is_some(), "{nome}: a caixinha fica");
        }
    }
    tecla(&mut m, id, Key::Escape, Modifiers::default(), None, "");
    assert_eq!(m.docs().rename_of(id), None, "Esc fecha");
    assert_eq!(m.docs().text_of(id).unwrap(), TEXTO);
}

/// Um valor órfão com o nome novo (de uma propriedade que já saiu do script) seria apagado pela
/// mudança: o nome não serve.
#[test]
fn um_objecto_que_guarda_o_nome_novo_recusa_o_rename() {
    let (mut m, id, mut f) = mesa();
    let mut o = owns(&mut m);
    o[1].1.insert("altura".into(), ScriptValue::Number(7.0));
    set_owns(&mut m, &o);
    let at = TEXTO.find("self.amplitude").unwrap() + 7;
    clica_em(&mut m, id, at, Modifiers::default());
    f2(&mut m, id);
    passo(&mut m, &mut f);
    m.escreve(id, "altura");
    let (_, _, r) = m.docs().rename_of(id).unwrap();
    assert!(r.is_some_and(|r| r.contains("already exists")));
}

/// O que não se renomeia diz porquê, e nunca pergunta ao servidor (um membro MATA-O, medição §1).
#[test]
fn um_membro_um_gancho_e_o_vazio_dizem_porque_e_nao_perguntam() {
    let (mut m, id, visto) = falso();
    for (agulha, off, razao) in [
        ("ph2d.set", 6, "another table"),
        ("function update", 10, "engine calls"),
        ("\"x\"", 1, "Nothing to rename"),
    ] {
        let at = TEXTO.find(agulha).unwrap() + off;
        clica_em(&mut m, id, at, Modifiers::default());
        m.quadro(); // o realce pergunta (ou não) pelo nome sob o cursor
        f2(&mut m, id);
        let (campo, _, r) = m.docs().rename_of(id).expect("a caixinha diz porquê");
        assert_eq!(campo, None, "{agulha}: sem campo");
        assert!(r.is_some_and(|r| r.contains(razao)), "{agulha}");
        tecla(&mut m, id, Key::Escape, Modifiers::default(), None, "");
        assert_eq!(m.docs().rename_of(id), None, "qualquer tecla a fecha");
    }
    nunca(&mut m, id, T0, "perguntou sobre um membro", |_| {
        let v = visto.lock().unwrap();
        v.renomes == 0 && !v.palavras.iter().any(|p| p == "set")
    });
    ate(
        &mut m,
        id,
        T0,
        "controlo: o realce do gancho pergunta (só o membro não)",
        |_| visto.lock().unwrap().palavras.iter().any(|p| p == "update"),
    );
}

fn e_palavra(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Como o `luau-lsp`: todos os sítios do nome no byte (palavra inteira, fora de `self.`/`ph2d.`).
fn sitios(texto: &str, b: usize) -> Vec<Range<usize>> {
    let ini = texto[..b].rfind(|c| !e_palavra(c)).map_or(0, |i| i + 1);
    let fim = texto[b..]
        .find(|c| !e_palavra(c))
        .map_or(texto.len(), |i| b + i);
    let nome = &texto[ini..fim];
    if nome.is_empty() {
        return Vec::new();
    }
    texto
        .match_indices(nome)
        .map(|(i, _)| i..i + nome.len())
        .filter(|r| {
            let antes = texto[..r.start].chars().next_back();
            let depois = texto[r.end..].chars().next();
            !antes.is_some_and(|c| e_palavra(c) || c == '.' || c == '"')
                && !depois.is_some_and(e_palavra)
        })
        .collect()
}

fn falso() -> (Mesa, ScriptId, Partilhado) {
    let (mut m, ids, visto) = com_falso(
        &[("bob.luau", TEXTO)],
        Plano {
            referir: Some(sitios),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    quadro_em(&mut m, T0);
    m.pinta(id);
    (m, id, visto)
}

/// ⭐ Renomear um local: o pedido vai ao servidor com o nome novo, e a resposta aplica-se num passo de
/// undo (um Ctrl+Z repõe tudo).
#[test]
fn renomear_um_local_pelo_servidor_num_passo_de_undo() {
    let (mut m, id, visto) = falso();
    let at = TEXTO.find("local brilho").unwrap() + 8;
    clica_em(&mut m, id, at, Modifiers::default());
    f2(&mut m, id);
    m.escreve(id, "luz");
    enter(&mut m, id);
    let esperado = TEXTO.replace("brilho", "luz");
    ate(&mut m, id, T0, "o rename não chegou", |m| {
        m.docs().text_of(id).as_deref() == Some(esperado.as_str())
    });
    assert_eq!(visto.lock().unwrap().renomes, 1);
    assert_eq!(m.docs().rename_of(id), None);
    ctrl_z(&mut m, id, false);
    assert_eq!(m.docs().text_of(id).unwrap(), TEXTO, "um Ctrl+Z repõe tudo");
    assert_eq!(
        m.docs().top_link_of(id, true),
        None,
        "um local não leva elo"
    );
}

/// ⭐ As referências: o cursor num local realça os sítios que o servidor dá; num campo do `self`, os
/// nossos (a declaração e os campos, sem perguntar); num membro, nada e nenhuma pergunta.
#[test]
fn as_ocorrencias_seguem_o_cursor() {
    let (mut m, id, visto) = falso();
    let uso = TEXTO.rfind("brilho").unwrap();
    clica_em(&mut m, id, uso + 2, Modifiers::default());
    let todos = sitios(TEXTO, uso);
    ate(&mut m, id, T0, "o realce do local não chegou", |m| {
        m.docs().occurrences_of(id) == todos
    });
    assert_eq!(todos.len(), 2);
    let pedidos = visto.lock().unwrap().referencias;
    clica_em(
        &mut m,
        id,
        TEXTO.find("self.amplitude").unwrap() + 7,
        Modifiers::default(),
    );
    m.quadro();
    let campo: Vec<Range<usize>> = [
        "\"amplitude\", 1.0",
        "self.amplitude",
        "self[\"amplitude\"]",
    ]
    .iter()
    .map(|a| {
        let off = if a.starts_with('"') {
            1
        } else if a.contains('[') {
            6
        } else {
            5
        };
        let s = TEXTO.find(a).unwrap() + off;
        s..s + "amplitude".len()
    })
    .collect();
    assert_eq!(
        m.docs().occurrences_of(id),
        campo,
        "a declaração e os campos"
    );
    clica_em(
        &mut m,
        id,
        TEXTO.find("ph2d.set").unwrap() + 6,
        Modifiers::default(),
    );
    m.quadro();
    assert!(m.docs().occurrences_of(id).is_empty());
    nunca(
        &mut m,
        id,
        T0,
        "perguntou sobre um campo ou um membro",
        |_| visto.lock().unwrap().referencias == pedidos,
    );
}

/// A caixinha pinta-se e fala ao leitor de ecrã (um `TextInput` com a ajuda no rótulo); um clique
/// dentro dela é dela, um clique fora fecha-a.
#[test]
fn a_caixinha_pinta_se_fala_e_um_clique_fora_fecha() {
    let (mut m, id, mut f) = mesa();
    let at = TEXTO.find("self.amplitude").unwrap() + 7;
    clica_em(&mut m, id, at, Modifiers::default());
    f2(&mut m, id);
    passo(&mut m, &mut f);
    m.pinta(id);
    let nodes = m.docs().a11y(id.0);
    let campo = nodes
        .iter()
        .find(|(_, n)| n.role() == ph2d_a11y::Role::TextInput)
        .expect("o campo do rename no leitor de ecrã");
    assert_eq!(campo.1.value(), Some("amplitude"));
    assert!(
        campo
            .1
            .label()
            .is_some_and(|l| l.contains("Enter to rename"))
    );
    assert!(
        nodes[0].1.children().contains(&campo.0.into()),
        "filho do Document"
    );
    let caixa = m.docs().rename_rect_of(id).expect("a caixinha pintou-se");
    let (_, y) = m.docs().glyph_center(id, at).unwrap();
    assert!(caixa.y > y, "por baixo do nome");
    let (x, y) = crate::centro(caixa);
    m.clica(id, x, y, Modifiers::default());
    assert!(m.docs().rename_of(id).is_some(), "um clique dentro é dela");
    let (x0, y0) = m.docs().glyph_center(id, 0).unwrap();
    m.clica(id, x0, y0, Modifiers::default());
    assert_eq!(m.docs().rename_of(id), None, "um clique fora fecha");
}

/// ⭐ Shift+F12: a lista dos sítios do nome (os do servidor, num local; os nossos, num campo do
/// `self`), cada linha «<linha>: <código>», nascida no sítio do cursor; as setas andam, Enter leva o
/// cursor lá; um clique numa linha também; o leitor de ecrã ouve «References».
#[test]
fn shift_f12_lista_os_sitios_e_enter_leva_la() {
    let (mut m, id, _visto) = falso();
    let uso = TEXTO.rfind("brilho").unwrap();
    clica_em(&mut m, id, uso + 2, Modifiers::default());
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    tecla(&mut m, id, Key::F(12), shift, None, "");
    ate(&mut m, id, T0, "a lista não abriu", |m| {
        m.docs().actions_of(id).is_none() && m.docs().references_of(id).is_some()
    });
    let (linhas, sel) = m.docs().references_of(id).unwrap();
    assert_eq!(
        linhas,
        [
            "3: local brilho = 2",
            &format!("6: {}", TEXTO.lines().nth(5).unwrap().trim())
        ]
    );
    assert_eq!(sel, 1, "nasce no sítio do cursor");
    m.pinta(id);
    let nos = m.docs().a11y(id.0);
    assert!(
        nos.iter()
            .any(|(_, n)| n.role() == ph2d_a11y::Role::ListBox && n.label() == Some("References"))
    );
    tecla(&mut m, id, Key::Down, Modifiers::default(), None, "");
    assert_eq!(
        m.docs().references_of(id).unwrap().1,
        0,
        "a seta dá a volta"
    );
    tecla(&mut m, id, Key::Enter, Modifiers::default(), Some("\r"), "");
    assert_eq!(m.docs().references_of(id), None, "Enter fecha");
    assert_eq!(cursor(&mut m, id), TEXTO.find("local brilho").unwrap() + 6);
    assert_eq!(m.docs().text_of(id).unwrap(), TEXTO, "nada se escreveu");
    // Num campo do `self`: os nossos, já; um clique na 1.ª linha vai à declaração.
    clica_em(
        &mut m,
        id,
        TEXTO.find("self[").unwrap() + 8,
        Modifiers::default(),
    );
    m.quadro();
    tecla(&mut m, id, Key::F(12), shift, None, "");
    let (linhas, sel) = m.docs().references_of(id).expect("abre na hora");
    assert_eq!((linhas.len(), sel), (3, 2));
    m.pinta(id);
    let rows = m.docs().completion_painted(id).expect("a lista pintou-se");
    let (_, _, _, (x, y)) = rows[0].clone();
    m.clica(id, x, y, Modifiers::default());
    assert_eq!(cursor(&mut m, id), TEXTO.find("amplitude").unwrap());
}
