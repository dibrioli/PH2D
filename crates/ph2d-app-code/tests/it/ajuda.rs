//! ⭐ **A página de ajuda** (Onda 2, item 5, `docs/CodeEditor/04_pagina_de_ajuda.md` §6, G6–G8): o
//! Ctrl+clique/F12 na API abre a página numa aba da família, rolada ao membro; o histórico ‹ › e o
//! Ctrl+W seguem o roteiro do oráculo do Godot 4.7.2 (`ferramentas/oraculo_ajuda/saida/amostras.txt`);
//! o teclado, a busca, a cópia e o leitor de ecrã da página. O servidor é o FALSO de `assistente.rs`,
//! a responder o balão que o `luau-lsp` 1.70.1 respondeu na sonda (`medicoes_onda2/ajuda_bruto.json`,
//! sobre o MESMO texto) e a definição como ele (um `local <nome>` do script; a API `[]`).

use crate::assistente::{Partilhado, Plano, com_falso, converge, tecla};
use crate::completar::{T0, quadro_em};
use crate::definicao::{clica_em, cursor};
use crate::{AREA, Mesa, centro, ctrl};
use ph2d_a11y::Role;
use ph2d_app_code::{dead_links, link_count, member_id};
use ph2d_code_help::Anchor;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key, KeyInput, KeyOutcome, PointerLook};
use ph2d_host::Modifiers;
use ph2d_panel_code::HelpHit;
use ph2d_script::ScriptId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use serde_json::Value;
use std::ops::Range;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// A sonda gravada: o texto e o balão de cada sítio.
fn bruto() -> &'static Value {
    static B: OnceLock<Value> = OnceLock::new();
    B.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../docs/CodeEditor/ferramentas/medicoes_onda2/ajuda_bruto.json"
        ))
        .expect("a sonda gravada")
    })
}

pub(crate) fn texto() -> &'static str {
    bruto()["texto"].as_str().expect("texto")
}

const SOMBRA: &str = "local print = function(...) end\nprint(1)\nlocal x = 1\n";

fn e_palavra(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn palavra(s: &str, b: usize) -> Range<usize> {
    let ini = s[..b].rfind(|c| !e_palavra(c)).map_or(0, |i| i + 1);
    let fim = s[b..].find(|c| !e_palavra(c)).map_or(s.len(), |i| b + i);
    ini..fim
}

fn byte_de(s: &str, linha: u64, col: u64) -> usize {
    s.split_inclusive('\n')
        .take(linha as usize)
        .map(str::len)
        .sum::<usize>()
        + col as usize
}

/// Como o `luau-lsp` 1.70.1 na sonda: o balão gravado do sítio (só sobre o texto da sonda).
fn pairar(s: &str, b: usize) -> Value {
    if s != texto() {
        return Value::Null;
    }
    let w = palavra(s, b);
    let casos = bruto()["casos"].as_object().expect("casos");
    casos
        .values()
        .find(|c| {
            let p = &c["pos"];
            let at = byte_de(
                s,
                p["line"].as_u64().unwrap(),
                p["character"].as_u64().unwrap(),
            );
            w.contains(&at) || (w.is_empty() && at == b)
        })
        .map_or(Value::Null, |c| c["hover"].clone())
}

/// Como o `luau-lsp`: um nome com `local <nome>` no script ⇒ a declaração; o resto (a API) ⇒ `[]`.
fn definir(s: &str, b: usize) -> Vec<Range<usize>> {
    let w = palavra(s, b);
    let nome = &s[w.clone()];
    if nome.is_empty() || s[..w.start].ends_with('.') || s[..w.start].ends_with(':') {
        return Vec::new();
    }
    s.find(&format!("local {nome} "))
        .map(|i| i + 6..i + 6 + nome.len())
        .into_iter()
        .collect()
}

pub(crate) fn mesa(scripts: &[(&str, &str)]) -> (Mesa, Vec<ScriptId>, Partilhado) {
    let (mut m, ids, visto) = com_falso(
        scripts,
        Plano {
            atraso: Duration::ZERO,
            pairar: Some(pairar),
            definir: Some(definir),
            ..Plano::default()
        },
    );
    for id in &ids {
        m.pede(*id);
    }
    m.pede(ids[0]);
    converge(&mut m, &visto, "abrir");
    quadro_em(&mut m, T0);
    m.pinta(ids[0]);
    (m, ids, visto)
}

pub(crate) fn ativa(m: &Mesa) -> Option<u64> {
    m.hero.documents.surface_active().map(|t| t.id)
}

/// Pinta a aba `id` da família (um script ou uma página).
pub(crate) fn pinta(m: &mut Mesa, id: u64) {
    let mut scene = VectorScene::new();
    let mut ts = std::mem::replace(&mut m.ts, TextSystem::without_system_fonts());
    m.docs()
        .paint(id, AREA, &mut scene, &mut ts, &Theme::default());
    m.ts = ts;
}

/// Um quadro como a shell: a família, e a pintura do script e da aba activa.
pub(crate) fn quadro(m: &mut Mesa, id: ScriptId) {
    quadro_em(m, T0);
    m.pinta(id);
    if let Some(a) = ativa(m) {
        pinta(m, a);
    }
}

/// Quadros até `feito`; reprova em 10 s.
fn ate(m: &mut Mesa, id: ScriptId, onde: &str, mut feito: impl FnMut(&mut Mesa) -> bool) {
    let fim = Instant::now() + Duration::from_secs(10);
    loop {
        quadro(m, id);
        if feito(m) {
            return;
        }
        assert!(Instant::now() < fim, "{onde}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// 25 quadros com tempo real para uma resposta chegar, e `nada` em cada um.
fn nunca(m: &mut Mesa, id: ScriptId, onde: &str, mut nada: impl FnMut(&mut Mesa) -> bool) {
    for _ in 0..25 {
        quadro(m, id);
        assert!(nada(m), "{onde}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

pub(crate) fn tecla_em(
    m: &mut Mesa,
    tab: u64,
    key: Key,
    mods: Modifiers,
    text: Option<&str>,
) -> KeyOutcome {
    let mut ts = TextSystem::without_system_fonts();
    let input = KeyInput {
        key,
        mods,
        pressed: true,
        text,
        paste: None,
    };
    m.docs().key(tab, &mut ts, input)
}

fn alt() -> Modifiers {
    Modifiers {
        alt: true,
        ..Modifiers::default()
    }
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

/// Um clique real (carregar + largar) na aba `tab` em `(x, y)`.
fn clica_aba(m: &mut Mesa, tab: u64, (x, y): (f32, f32)) {
    use ph2d_editor_core::doc_surface::PointerInput;
    use ph2d_host::{PointerButton, PointerKind};
    let mut ts = std::mem::replace(&mut m.ts, TextSystem::without_system_fonts());
    for kind in [PointerKind::Down, PointerKind::Up] {
        let input = PointerInput {
            text: &mut ts,
            mods: Modifiers::default(),
            now_ns: 1,
        };
        assert!(
            m.docs()
                .pointer(tab, input, kind, PointerButton::Primary, x, y),
            "o clique em ({x}, {y}) não foi tomado"
        );
    }
    m.ts = ts;
}

/// O byte do `n`-ésimo (0-based) `agulha` do texto da sonda, deslocado `off`.
pub(crate) fn em(agulha: &str, off: usize) -> usize {
    texto().find(agulha).unwrap_or_else(|| panic!("{agulha:?}")) + off
}

/// A página `page` mostra o membro `member` no topo — ou, no fim da página (a rolagem presa ao
/// máximo), a assinatura dele à vista.
fn no_membro(m: &mut Mesa, page: &str, member: &str) -> bool {
    let Some(p) = m.docs().help_probe(page) else {
        return false;
    };
    let id = member_id(page, member).expect("membro do catálogo");
    let lib = if page == "globals" {
        String::new()
    } else {
        format!("{page}.")
    };
    let sig = format!("{lib}{member}");
    // A vista é a aba menos o cabeçalho (‹ › e o título; a busca fechada).
    let head = (ph2d_tokens::ROW_H_PX + 2.0 * ph2d_tokens::Spacing::Sm.px()).min(AREA.h);
    let fim = p.scroll + 0.5 >= (p.height - (AREA.h - head)).max(0.0);
    p.top_member == Some(id)
        || (fim
            && p.lines
                .iter()
                .any(|l| l.starts_with(&sig) && !l[sig.len()..].starts_with(e_palavra)))
}

pub(crate) fn na_pagina(m: &mut Mesa, page: &str, member: &str) -> bool {
    let tab = m.docs().help_tab_id(page);
    tab.is_some() && ativa(m) == tab && no_membro(m, page, member)
}

fn definicoes(v: &Partilhado) -> usize {
    v.lock().unwrap().definicoes
}

/// ⭐⭐ G6 — o Ctrl+clique (e o F12) na API abre a página CERTA rolada ao membro, numa aba que se
/// reaproveita; o pseudónimo (`m.cos`), o método (`s:upper()`), a função sem nome no balão
/// (`table.insert`), a constante (`math.pi`) e a nossa (`ph2d.set`) chegam onde o Godot chegaria.
#[test]
fn ctrl_clique_abre_a_pagina_no_membro() {
    let (mut m, ids, _visto) = mesa(&[("ajuda.luau", texto())]);
    let id = ids[0];
    clica_em(&mut m, id, em("math.sin", 6), ctrl());
    ate(&mut m, id, "math.sin: a página não abriu no sin", |m| {
        na_pagina(m, "math", "sin")
    });
    let math = m.docs().help_tab_id("math");
    assert!(
        m.docs()
            .tabs()
            .iter()
            .any(|t| Some(t.id) == math && t.title == "math"),
        "a aba aparece na fila da família"
    );
    for (onde, byte, page, member) in [
        (
            "m.cos (o pseudónimo: só o balão o sabe)",
            em("m.cos", 3),
            "math",
            "cos",
        ),
        ("s:upper()", em("s:upper", 3), "string", "upper"),
        (
            "table.insert (o balão sem nome)",
            em("table.insert", 7),
            "table",
            "insert",
        ),
        ("math.pi (constante)", em("math.pi", 6), "math", "pi"),
        ("ph2d.set", em("ph2d.set", 6), "ph2d", "set"),
    ] {
        clica_em(&mut m, id, byte, ctrl());
        ate(
            &mut m,
            id,
            &format!("{onde}: não abriu {page}.{member}"),
            |m| na_pagina(m, page, member),
        );
    }
    let abertas: Vec<&str> = m.docs().help_tabs().iter().map(|t| t.0).collect();
    assert_eq!(
        abertas,
        ["math", "ph2d", "string", "table"],
        "uma aba por página (a de math reaproveitada), por título, depois dos scripts"
    );
    assert_eq!(m.docs().help_tab_id("math"), math, "a mesma aba");
    // F12 = o mesmo que o clique.
    let b = em("buffer.create", 8);
    clica_em(&mut m, id, b, Modifiers::default());
    let c = cursor(&mut m, id);
    tecla(&mut m, id, Key::F(12), Modifiers::default(), None, "");
    ate(&mut m, id, "F12 em buffer.create", |m| {
        na_pagina(m, "buffer", "create")
    });
    assert_eq!(cursor(&mut m, id), c, "o F12 não mexe no cursor do script");
}

/// G6 — o nome declarado NESTE script vai à declaração (um `local print` sombreia a global) e não
/// abre página; uma palavra reservada não faz nada e o ponteiro não vira mão.
#[test]
fn o_local_sombreia_e_a_palavra_reservada_nao_abre_nada() {
    let (mut m, ids, visto) = mesa(&[("sombra.luau", SOMBRA)]);
    let id = ids[0];
    let uso = SOMBRA.find("print(1)").unwrap() + 2;
    clica_em(&mut m, id, uso, ctrl());
    ate(&mut m, id, "não foi à declaração do local", |m| {
        cursor(m, id) == 6
    });
    nunca(&mut m, id, "o local abriu uma página", |m| {
        m.docs().help_tabs().is_empty()
    });
    // `local` com o Ctrl carregado: nem mão, nem pergunta, nem página.
    let kw = SOMBRA.find("local x").unwrap() + 2;
    m.hero.store.set_cmd_held(true);
    crate::dicas::aponta(&mut m, id, kw);
    let antes = definicoes(&visto);
    nunca(&mut m, id, "a palavra reservada deu a mão", |m| {
        let (x, y) = m.docs().glyph_center(id, kw).expect("à vista");
        m.docs().pointer_look(id.0, x, y) != Some(PointerLook::Hand)
    });
    m.hero.store.set_cmd_held(false);
    clica_em(&mut m, id, kw, ctrl());
    nunca(&mut m, id, "a palavra reservada abriu algo", |m| {
        m.docs().help_tabs().is_empty() && ativa(m) == Some(id.0)
    });
    assert_eq!(
        definicoes(&visto),
        antes,
        "perguntou por uma palavra reservada"
    );
}

/// O link à vista da página `page` cujo destino satisfaz `quer`: o número, o destino e o centro do
/// retângulo pintado.
fn link_a_vista(
    m: &mut Mesa,
    page: &str,
    quer: impl Fn(Anchor) -> bool,
) -> (u32, Anchor, (f32, f32)) {
    let p = m.docs().help_probe(page).expect("pintada");
    p.links
        .iter()
        .filter_map(|(n, r, _)| Some((*n, m.docs().help_link_target(page, *n)?, centro(*r))))
        .find(|(_, a, _)| quer(*a))
        .unwrap_or_else(|| panic!("nenhum link assim à vista em {page}: {:?}", p.links))
}

/// ⭐⭐ G7 — o roteiro do oráculo: o link na MESMA página rola e não empilha (o ‹ seguinte volta ao
/// SCRIPT, no cursor do Ctrl+clique); o link para OUTRA página abre-a e empilha; ‹ volta à página
/// anterior na MESMA rolagem e › avança; o Ctrl+W numa página fecha-a e volta ao script; numa aba
/// de script fecha-a.
#[test]
fn navegacao() {
    let (mut m, ids, _visto) = mesa(&[("ajuda.luau", texto())]);
    let id = ids[0];
    let set = em("ph2d.set", 6);
    clica_em(&mut m, id, set, ctrl());
    ate(&mut m, id, "ph2d.set", |m| na_pagina(m, "ph2d", "set"));
    let c0 = cursor(&mut m, id);
    let ph2d = m.docs().help_tab_id("ph2d").unwrap();
    assert_eq!(m.docs().nav_can(ph2d), (true, false), "‹ aceso, › apagado");
    // Um link para outro membro da MESMA página: rola, a aba é a mesma.
    let (_, a, p) = link_a_vista(&mut m, "ph2d", |a| {
        a.page == "ph2d" && a.member != Some("set")
    });
    clica_aba(&mut m, ph2d, p);
    ate(&mut m, id, "o link na mesma página não rolou", |m| {
        na_pagina(m, "ph2d", a.member.unwrap())
    });
    assert_eq!(m.docs().help_tabs().len(), 1);
    // ‹ (Alt+←): o link na mesma página não entrou no histórico — volta ao SCRIPT, no cursor.
    let _ = tecla_em(&mut m, ph2d, Key::Left, alt(), None);
    ate(&mut m, id, "‹ não voltou ao script", |m| {
        ativa(m) == Some(id.0)
    });
    assert_eq!(cursor(&mut m, id), c0, "o cursor do Ctrl+clique");
    // De novo (noutro byte: o mesmo seria um duplo clique, a mesa clica sempre no mesmo instante);
    // e um link para OUTRA página (o tipo da assinatura).
    clica_em(&mut m, id, set + 1, ctrl());
    let c0 = cursor(&mut m, id);
    ate(&mut m, id, "ph2d.set outra vez", |m| {
        na_pagina(m, "ph2d", "set")
    });
    let s1 = m.docs().help_probe("ph2d").unwrap().scroll;
    let (_, a, p) = link_a_vista(&mut m, "ph2d", |a| a.page != "ph2d");
    clica_aba(&mut m, ph2d, p);
    ate(
        &mut m,
        id,
        "o link para outra página não a abriu",
        |m| match a.member {
            Some(mb) => na_pagina(m, a.page, mb),
            None => {
                let t = m.docs().help_tab_id(a.page);
                t.is_some() && ativa(m) == t
            }
        },
    );
    let outra = m.docs().help_tab_id(a.page).unwrap();
    // ‹ pelo botão do cabeçalho: a página anterior, na MESMA rolagem.
    let r = m.docs().help_target(a.page, HelpHit::Back).unwrap();
    clica_aba(&mut m, outra, centro(r));
    ate(&mut m, id, "‹ não voltou à página anterior", |m| {
        ativa(m) == Some(ph2d)
            && m.docs()
                .help_probe("ph2d")
                .is_some_and(|p| (p.scroll - s1).abs() < 0.5)
    });
    // › (Alt+→): a outra página.
    let _ = tecla_em(&mut m, ph2d, Key::Right, alt(), None);
    ate(&mut m, id, "› não avançou", |m| ativa(m) == Some(outra));
    let _ = tecla_em(&mut m, outra, Key::Left, alt(), None);
    ate(&mut m, id, "‹ outra vez", |m| ativa(m) == Some(ph2d));
    // Ctrl+W na página: fecha-a e volta ao script (o sítio anterior do histórico).
    let out = tecla_em(&mut m, ph2d, Key::Char('w'), ctrl(), Some("w"));
    assert!(out.consumed, "o Ctrl+W é da família");
    ate(&mut m, id, "o Ctrl+W não voltou ao script", |m| {
        ativa(m) == Some(id.0) && m.docs().help_tab_id("ph2d").is_none()
    });
    assert_eq!(cursor(&mut m, id), c0);
    // Ctrl+W num script fecha-o.
    let out = tecla_em(&mut m, id.0, Key::Char('w'), ctrl(), Some("w"));
    assert!(out.consumed);
    quadro(&mut m, id);
    assert!(
        !m.docs().open_scripts().contains(&id),
        "o Ctrl+W não fechou o script"
    );
}

/// ⭐ G8 — PgDn rola; Tab dá o foco ao 1.º link e Enter segue-o; Ctrl+F «sin» conta «1 of N» e o
/// Enter passa à seguinte; arrastar seleciona e o Ctrl+C copia esse texto; o «Copy» copia o exemplo;
/// o leitor de ecrã vê um `Document` com `Link`s, em ids que não são os do script.
#[test]
fn teclado_e_leitor() {
    let (mut m, ids, _visto) = mesa(&[("ajuda.luau", texto())]);
    let id = ids[0];
    clica_em(&mut m, id, em("math.sin", 6), ctrl());
    ate(&mut m, id, "math.sin", |m| na_pagina(m, "math", "sin"));
    let math = m.docs().help_tab_id("math").unwrap();
    // O «Copy» do exemplo do sin (à vista, logo abaixo da assinatura).
    let n = m
        .docs()
        .help_copy_of("math", "sin")
        .expect("o sin tem exemplo");
    let r = m
        .docs()
        .help_target("math", HelpHit::Copy(n))
        .expect("o botão");
    clica_aba(&mut m, math, centro(r));
    quadro(&mut m, id);
    let exemplo = ph2d_code_help::page("math").unwrap().members
        [member_id("math", "sin").unwrap() as usize]
        .example
        .unwrap();
    assert_eq!(
        m.hero.store.take_clipboard_copy().as_deref(),
        Some(exemplo),
        "o «Copy» copia o exemplo, pelo caminho da área de transferência"
    );
    // Arrastar seleciona; Ctrl+C copia.
    let (a, b) = dois_pontos_de_texto(&mut m, math);
    {
        use ph2d_host::{PointerButton, PointerKind};
        let mut ts = std::mem::replace(&mut m.ts, TextSystem::without_system_fonts());
        let d = m.docs();
        assert!(d.pointer(
            math,
            mk(&mut ts),
            PointerKind::Down,
            PointerButton::Primary,
            a.0,
            a.1
        ));
        let _ = d.pointer_move(math, mk(&mut ts), b.0, b.1);
        assert!(d.pointer(
            math,
            mk(&mut ts),
            PointerKind::Up,
            PointerButton::Primary,
            b.0,
            b.1
        ));
        m.ts = ts;
    }
    let sel = m
        .docs()
        .help_state("math")
        .unwrap()
        .0
        .expect("arrastar seleciona");
    assert!(!sel.is_empty());
    let plano = m.docs().help_plain("math").unwrap();
    let out = tecla_em(&mut m, math, Key::Char('c'), ctrl(), Some("c"));
    assert_eq!(
        out.copy.as_deref(),
        Some(&plano[sel]),
        "Ctrl+C copia a seleção"
    );
    // PgDn rola.
    let antes = m.docs().help_probe("math").unwrap().scroll;
    let _ = tecla_em(&mut m, math, Key::PageDown, Modifiers::default(), None);
    quadro(&mut m, id);
    let depois = m.docs().help_probe("math").unwrap().scroll;
    assert!(depois > antes + 100.0, "PgDn: {antes} → {depois}");
    // Home; Tab dá o foco ao 1.º link; Enter segue-o.
    let _ = tecla_em(&mut m, math, Key::Home, Modifiers::default(), None);
    let _ = tecla_em(&mut m, math, Key::Tab, Modifiers::default(), None);
    quadro(&mut m, id);
    let foco = m
        .docs()
        .help_state("math")
        .unwrap()
        .1
        .expect("o Tab focou um link");
    let alvo = m.docs().help_link_target("math", foco).expect("um link");
    let _ = tecla_em(&mut m, math, Key::Tab, shift(), None);
    let _ = tecla_em(&mut m, math, Key::Tab, Modifiers::default(), None);
    assert_eq!(
        m.docs().help_state("math").unwrap().1,
        Some(foco),
        "Shift+Tab volta"
    );
    let _ = tecla_em(&mut m, math, Key::Enter, Modifiers::default(), None);
    ate(
        &mut m,
        id,
        "o Enter não seguiu o link com o foco",
        |m| match alvo.member {
            Some(mb) => na_pagina(m, alvo.page, mb),
            None => ativa(m) == m.docs().help_tab_id(alvo.page),
        },
    );
    // Ctrl+F «sin»: a contagem; Enter: a seguinte.
    let tab = ativa(&m).unwrap();
    let page = ph2d_code_help::catalog()[(tab & !ph2d_app_code::HELP_TAG) as usize].id;
    let _ = tecla_em(&mut m, tab, Key::Char('f'), ctrl(), Some("f"));
    for c in "sin".chars() {
        let s = c.to_string();
        let _ = tecla_em(&mut m, tab, Key::Other, Modifiers::default(), Some(&s));
    }
    quadro(&mut m, id);
    let conta = m.docs().help_find_count_text(page).expect("a barra aberta");
    assert!(conta.starts_with("1 of "), "{conta}");
    let _ = tecla_em(&mut m, tab, Key::Enter, Modifiers::default(), None);
    let conta = m.docs().help_find_count_text(page).unwrap();
    assert!(conta.starts_with("2 of "), "Enter: a seguinte — {conta}");
    let _ = tecla_em(&mut m, tab, Key::Escape, Modifiers::default(), None);
    assert_eq!(m.docs().help_find_count_text(page), None, "Esc fecha");
    // O leitor de ecrã.
    quadro(&mut m, id);
    let nos = m.docs().a11y(tab);
    assert_eq!(nos[0].1.role(), Role::Document);
    assert_eq!(
        nos[0].1.label(),
        Some(ph2d_code_help::page(page).unwrap().title.as_str())
    );
    assert!(nos.iter().any(|(_, n)| n.role() == Role::Link), "os links");
    let do_script: Vec<_> = m.docs().a11y(id.0).into_iter().map(|n| n.0).collect();
    assert!(
        nos.iter().all(|(n, _)| !do_script.contains(n)),
        "ids do leitor partilhados entre a página e o script"
    );
}

/// Dois pontos da página `tab` sobre TEXTO (não links), na mesma linha e com bytes diferentes.
fn dois_pontos_de_texto(m: &mut Mesa, tab: u64) -> ((f32, f32), (f32, f32)) {
    let page = ph2d_code_help::catalog()[(tab & !ph2d_app_code::HELP_TAG) as usize].id;
    let mut ts = std::mem::replace(&mut m.ts, TextSystem::without_system_fonts());
    let mut found = None;
    'y: for k in 0..200 {
        let y = AREA.y + 120.0 + k as f32 * 3.0;
        let mut first = None;
        for j in 0..80 {
            let x = AREA.x + 400.0 + j as f32 * 4.0;
            if let Some(HelpHit::Text(b)) = m.docs().help_hit(page, &mut ts, x, y) {
                match first {
                    None => first = Some((x, y, b)),
                    Some((x0, y0, b0)) if b > b0 + 8 => {
                        found = Some(((x0, y0), (x, y)));
                        break 'y;
                    }
                    _ => {}
                }
            } else if first.is_some() {
                first = None;
            }
        }
    }
    m.ts = ts;
    found.expect("dois pontos de texto na página")
}

/// ⭐ Todo `[…]` de todo texto do catálogo (`[math.floor]`, `[hooks.on_signal]`, `[PoseField]`)
/// resolve pela MESMA porta do Ctrl+clique — um link morto é vermelho aqui, nunca texto calado.
#[test]
fn todo_link_do_catalogo_resolve() {
    let mortos = dead_links();
    assert!(mortos.is_empty(), "links que não resolvem: {mortos:#?}");
    let n = link_count();
    assert!(n > 100, "controlo: o catálogo tem links ({n})");
}

fn mk(ts: &mut TextSystem) -> ph2d_editor_core::doc_surface::PointerInput<'_> {
    ph2d_editor_core::doc_surface::PointerInput {
        text: ts,
        mods: Modifiers::default(),
        now_ns: 1,
    }
}
