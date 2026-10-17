//! I5 (Onda 2, `docs/CodeEditor/03_onda2.md`): o balão do rato e a ajuda de assinatura. O servidor é
//! o FALSO de `assistente.rs`, a responder como o `luau-lsp` 1.70.1 respondeu na sonda do I0b (§4:
//! uma assinatura, `activeParameter`, os parâmetros em offsets UTF-16 do rótulo; §5: o hover em
//! Markdown com o bloco ```` ```luau ````).
//!
//! ⛔ Toda asserção de «NÃO abre» dá tempo REAL a uma resposta assíncrona chegar ([`nunca`]): olhar
//! logo a seguir ao quadro deixou quatro mutantes vivos no I4.

use crate::Mesa;
use crate::assistente::{Partilhado, Plano, com_falso, converge, tecla};
use crate::completar::{T0, abre, escreve, quadro_em, responde};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

/// O `tooltip_delay_sec` do Godot (I0b §7).
const PAUSA_BALAO: u128 = 500_000_000;
const PASSO: u128 = 100_000_000;

fn e_palavra(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Como o `luau-lsp`: o tipo do nome sob o cursor num bloco `luau`, e a régua antes da documentação.
fn balao(texto: &str, b: usize) -> Value {
    let ini = texto[..b].rfind(|c| !e_palavra(c)).map_or(0, |i| i + 1);
    let fim = texto[b..]
        .find(|c| !e_palavra(c))
        .map_or(texto.len(), |i| b + i);
    let nome = &texto[ini..fim];
    if nome.is_empty() {
        return Value::Null;
    }
    json!({ "contents": { "kind": "markdown",
        "value": format!("```luau\nlocal {nome}: number\n```\n----------\nThe {nome} doc.") } })
}

/// Como o `luau-lsp`: a chamada aberta mais próxima antes do cursor; o parâmetro activo conta as
/// vírgulas de fora de strings e de parênteses de dentro. `ph2d.set` tem o rótulo real (I0b §4).
fn assinatura(texto: &str, b: usize) -> Value {
    let antes: Vec<char> = texto[..b].chars().collect();
    let (mut fundo, mut virgulas, mut aspa) = (0i32, 0u64, None);
    let mut abre = None;
    for i in (0..antes.len()).rev() {
        let c = antes[i];
        if let Some(q) = aspa {
            if c == q {
                aspa = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => aspa = Some(c),
            ')' => fundo += 1,
            '(' if fundo == 0 => {
                abre = Some(i);
                break;
            }
            '(' => fundo -= 1,
            ',' if fundo == 0 => virgulas += 1,
            '\n' => return Value::Null,
            _ => {}
        }
    }
    let Some(i) = abre else { return Value::Null };
    let nome: String = antes[..i]
        .iter()
        .rev()
        .take_while(|c| e_palavra(**c) || **c == '.')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let (label, params): (String, Vec<&str>) = if nome == "ph2d.set" {
        (
            "function ph2d.set(entity: number, field: \"rotation\" | \"x\", value: number): ()"
                .into(),
            vec![
                "entity: number",
                "field: \"rotation\" | \"x\"",
                "value: number",
            ],
        )
    } else {
        (
            format!("function {nome}(a: any, b: any)"),
            vec!["a: any", "b: any"],
        )
    };
    let ps: Vec<Value> = params
        .iter()
        .map(|p| {
            let s = label.find(p).unwrap() as u64;
            json!({ "label": [s, s + p.len() as u64],
                    "documentation": { "kind": "markdown", "value": format!("The {p} doc.") } })
        })
        .collect();
    json!({ "activeSignature": 0, "activeParameter": virgulas,
            "signatures": [{ "label": label, "activeParameter": virgulas, "parameters": ps,
                             "documentation": { "kind": "markdown", "value": "Writes a pose field." } }] })
}

fn mesa(scripts: &[(&str, &str)]) -> (Mesa, Vec<ScriptId>, Partilhado) {
    com_falso(
        scripts,
        Plano {
            completar: Some(responde),
            pairar: Some(balao),
            assinar: Some(assinatura),
            ..Plano::default()
        },
    )
}

/// Quadros no instante `t` (o tempo real a correr) até `feito`; reprova em 10 s.
fn ate(m: &mut Mesa, id: ScriptId, t: u128, onde: &str, mut feito: impl FnMut(&mut Mesa) -> bool) {
    let fim = Instant::now() + Duration::from_secs(10);
    loop {
        quadro_em(m, t);
        m.pinta(id);
        if feito(m) {
            return;
        }
        assert!(Instant::now() < fim, "{onde}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// 25 quadros no instante `t`, com tempo real para uma resposta chegar, e `nada` em cada um.
fn nunca(m: &mut Mesa, id: ScriptId, t: u128, onde: &str, mut nada: impl FnMut(&mut Mesa) -> bool) {
    for _ in 0..25 {
        quadro_em(m, t);
        m.pinta(id);
        assert!(nada(m), "{onde}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn aponta(m: &mut Mesa, id: ScriptId, byte: usize) {
    let (x, y) = m
        .docs()
        .glyph_center(id, byte)
        .unwrap_or_else(|| panic!("o byte {byte} não está à vista"));
    m.rato(id, x, y);
}

fn activo(m: &mut Mesa, id: ScriptId) -> Option<String> {
    m.docs().signature_of(id).and_then(|s| s.1)
}

fn sem_tecla() -> Modifiers {
    Modifiers::default()
}

/// ⭐ Rato em movimento contínuo (seis segundos, um movimento a cada 0,1 s, entre palavras e dentro
/// da MESMA) ⇒ `0` pedidos. Parado sobre uma palavra: nada antes da pausa de 0,5 s, o balão depois,
/// com UM pedido; sobre o balão fica; no vazio depois do fim da linha sai, e lá não pergunta.
#[test]
fn o_rato_a_andar_nunca_pergunta_e_parado_abre_o_balao() {
    let texto = "local brilho = 1\nprint(brilho)\n";
    let (mut m, ids, visto) = mesa(&[("a.luau", texto)]);
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    quadro_em(&mut m, T0);
    m.pinta(id);
    let b = texto.rfind("brilho").unwrap();
    let pontos = [
        b,
        b + 2,
        b + 5,
        texto.find("print").unwrap(),
        7,
        b + 1,
        b + 3,
    ];
    let baloes = |v: &Partilhado| v.lock().unwrap().baloes;
    for k in 0..60u128 {
        aponta(&mut m, id, pontos[k as usize % pontos.len()]);
        quadro_em(&mut m, T0 + k * PASSO);
        m.pinta(id);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(baloes(&visto), 0, "o rato a andar perguntou");
    assert_eq!(m.docs().hover_of(id), None);
    let t1 = T0 + 60 * PASSO;
    quadro_em(&mut m, t1);
    aponta(&mut m, id, b + 2);
    nunca(
        &mut m,
        id,
        t1 + PAUSA_BALAO - 1,
        "abriu antes da pausa",
        |m| m.docs().hover_of(id).is_none() && baloes(&visto) == 0,
    );
    ate(&mut m, id, t1 + PAUSA_BALAO, "o balão não abriu", |m| {
        m.docs().hover_of(id).is_some()
    });
    assert!(
        m.docs()
            .hover_of(id)
            .unwrap()
            .contains("local brilho: number")
    );
    let (pintado, _) = m.docs().hints_painted(id);
    let pintado = pintado.expect("o balão pintou-se");
    assert!(
        pintado.lines.iter().any(|l| l == "local brilho: number"),
        "sem a marcação do Markdown: {:?}",
        pintado.lines
    );
    nunca(
        &mut m,
        id,
        t1 + PAUSA_BALAO * 4,
        "um pedido por palavra",
        |m| m.docs().hover_of(id).is_some() && baloes(&visto) == 1,
    );
    // Sobre o balão fica (lê-se sem fugir).
    let r = pintado.rect;
    m.rato(id, r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(m.docs().hover_of(id).is_some(), "o balão fugiu do rato");
    // No vazio à direita do fim da linha (a 1.ª, que acaba numa palavra: o `1`): sai, e ali não
    // pergunta.
    let (_, y) = m.docs().glyph_center(id, 7).unwrap();
    m.rato(id, 900.0, y);
    assert_eq!(m.docs().hover_of(id), None, "o balão ficou");
    nunca(
        &mut m,
        id,
        t1 + PAUSA_BALAO * 10,
        "perguntou no vazio",
        |m| m.docs().hover_of(id).is_none() && baloes(&visto) == 1,
    );
}

/// O balão só pergunta sobre CÓDIGO (num comentário ou numa string nem pergunta) e sai com uma tecla.
#[test]
fn o_balao_so_pergunta_sobre_codigo_e_sai_com_uma_tecla() {
    let texto = "-- brilho no comentario\nlocal s = \"brilho\"\nlocal brilho = 1\n";
    let (mut m, ids, visto) = mesa(&[("a.luau", texto)]);
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    quadro_em(&mut m, T0);
    m.pinta(id);
    for (k, at) in [
        texto.find("brilho").unwrap() + 1,
        texto.find("\"brilho").unwrap() + 2,
    ]
    .into_iter()
    .enumerate()
    {
        let t = T0 + PAUSA_BALAO * 10 * k as u128;
        quadro_em(&mut m, t);
        aponta(&mut m, id, at);
        nunca(
            &mut m,
            id,
            t + PAUSA_BALAO * 2,
            "perguntou fora do código",
            |m| m.docs().hover_of(id).is_none() && visto.lock().unwrap().baloes == 0,
        );
    }
    let t = T0 + PAUSA_BALAO * 30;
    quadro_em(&mut m, t);
    aponta(&mut m, id, texto.rfind("brilho").unwrap());
    ate(
        &mut m,
        id,
        t + PAUSA_BALAO,
        "o balão do código não abriu",
        |m| m.docs().hover_of(id).is_some(),
    );
    tecla(&mut m, id, Key::Right, sem_tecla(), None, "");
    assert_eq!(m.docs().hover_of(id), None, "a tecla não fechou o balão");
}

/// ⭐ A assinatura abre em `(` e em `,` (sem pausa) e o argumento activo de `ph2d.set` fica MARCADO,
/// na pintura também; fechar o `)` sai da chamada e fecha-a; Esc fecha-a.
#[test]
fn a_assinatura_abre_em_parentese_e_virgula_e_marca_o_argumento() {
    let (mut m, ids, _) = mesa(&[("a.luau", "")]);
    let id = ids[0];
    m.pede(id);
    quadro_em(&mut m, T0);
    for (escrito, esperado) in [
        ("ph2d.set(", "entity: number"),
        ("self.id,", "field: \"rotation\" | \"x\""),
        (" \"x\", 1", "value: number"),
    ] {
        escreve(&mut m, id, escrito);
        ate(
            &mut m,
            id,
            T0,
            &format!("«{escrito}»: não marcou {esperado}"),
            |m| activo(m, id).as_deref() == Some(esperado),
        );
        let (_, s) = m.docs().hints_painted(id);
        let s = s.expect("a assinatura pintou-se");
        assert_eq!(s.marked.join(" "), esperado, "«{escrito}»: a marca pintada");
        assert!(s.lines.iter().any(|l| l.starts_with("function ph2d.set(")));
    }
    escreve(&mut m, id, ")");
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some("ph2d.set(self.id, \"x\", 1)")
    );
    ate(&mut m, id, T0, "fora da chamada a assinatura ficou", |m| {
        m.docs().signature_of(id).is_none()
    });
    tecla(&mut m, id, Key::Enter, sem_tecla(), Some("\r"), "");
    escreve(&mut m, id, "ph2d.set(");
    ate(&mut m, id, T0, "não abriu", |m| {
        m.docs().signature_of(id).is_some()
    });
    tecla(&mut m, id, Key::Escape, sem_tecla(), None, "");
    assert_eq!(m.docs().signature_of(id), None, "Esc não fechou");
    nunca(&mut m, id, T0, "uma resposta em voo reabriu-a", |m| {
        m.docs().signature_of(id).is_none() && m.docs().hints_painted(id).1.is_none()
    });
    // A vírgula volta a abri-la, já no 2.º argumento.
    escreve(&mut m, id, "e,");
    ate(&mut m, id, T0, "a vírgula não reabriu", |m| {
        activo(m, id).is_some_and(|a| a.starts_with("field"))
    });
}

/// O assistente que parou DE VEZ não deixa a assinatura aberta a envelhecer na aba (mesmo sem um
/// pedido dela em voo).
#[test]
fn um_assistente_parado_nao_deixa_a_assinatura() {
    use crate::assistente::falso;
    use ph2d_code_intel::lsp::Connect;
    let mut primeiro = Some(falso(
        Plano {
            assinar: Some(assinatura),
            morre_apos: Some(2),
            ..Plano::default()
        },
        Partilhado::default(),
        false,
    ));
    let liga: Connect = Box::new(move |wake| match primeiro.take() {
        Some(mut c) => c(wake),
        None => Err(ph2d_code_lsp::LspError::Spawn(std::io::Error::other(
            "não volta",
        ))),
    });
    let (m, ids) = Mesa::new(&[("a.luau", ""), ("b.luau", "")]);
    let mut m = m.com_assistente(ph2d_app_code::IntelServer::Custom(liga));
    let (a, b) = (ids[0], ids[1]);
    m.pede(a);
    quadro_em(&mut m, T0);
    escreve(&mut m, a, "f(");
    ate(&mut m, a, T0, "não abriu", |m| {
        m.docs().signature_of(a).is_some()
    });
    // O servidor morre com uma edição NOUTRO script: na aba da assinatura não há pedido em voo
    // que falhe, só o estado do assistente a fecha.
    m.pede(b);
    escreve(&mut m, b, "x");
    ate(
        &mut m,
        a,
        T0,
        "o assistente parou e a assinatura ficou",
        |m| m.docs().assist_text().is_some() && m.docs().signature_of(a).is_none(),
    );
}

/// Numa string ou num comentário, `(` e `,` não abrem (nem perguntam); o controlo, em código, abre.
#[test]
fn numa_string_ou_comentario_a_virgula_nao_abre_a_assinatura() {
    let (mut m, ids, visto) = mesa(&[("a.luau", "-- \nlocal s = \"\"\n\n")]);
    let id = ids[0];
    m.pede(id);
    quadro_em(&mut m, T0);
    tecla(&mut m, id, Key::End, sem_tecla(), None, "");
    escreve(&mut m, id, "f(a,");
    tecla(&mut m, id, Key::Down, sem_tecla(), None, "");
    tecla(&mut m, id, Key::End, sem_tecla(), None, "");
    tecla(&mut m, id, Key::Left, sem_tecla(), None, "");
    escreve(&mut m, id, "f(a,");
    nunca(
        &mut m,
        id,
        T0,
        "abriu numa string ou num comentário",
        |m| m.docs().signature_of(id).is_none() && visto.lock().unwrap().assinaturas == 0,
    );
    tecla(&mut m, id, Key::End, crate::ctrl(), None, "");
    escreve(&mut m, id, "f(");
    ate(&mut m, id, T0, "em código não abriu", |m| {
        activo(m, id).as_deref() == Some("a: any")
    });
}

/// ⭐ Aceitar uma FUNÇÃO da lista abre a assinatura (o `command` do item); aceitar um campo, não.
#[test]
fn aceitar_uma_funcao_da_lista_abre_a_assinatura() {
    let (mut m, ids, visto) = mesa(&[("a.luau", ""), ("b.luau", "")]);
    let (a, b) = (ids[0], ids[1]);
    m.pede(a);
    let _ = abre(&mut m, a, "ph2d.se");
    tecla(&mut m, a, Key::Tab, sem_tecla(), None, "");
    assert_eq!(m.docs().text_of(a).as_deref(), Some("ph2d.set()"));
    ate(
        &mut m,
        a,
        T0,
        "aceitar `set` não abriu a assinatura",
        |m| activo(m, a).as_deref() == Some("entity: number"),
    );
    m.pede(b);
    let _ = abre(&mut m, b, "self.");
    let antes = visto.lock().unwrap().assinaturas;
    tecla(&mut m, b, Key::Tab, sem_tecla(), None, "");
    assert_eq!(m.docs().text_of(b).as_deref(), Some("self.t"));
    nunca(&mut m, b, T0, "aceitar um campo abriu a assinatura", |m| {
        m.docs().signature_of(b).is_none() && visto.lock().unwrap().assinaturas == antes
    });
}

/// O leitor de ecrã: a assinatura e o balão são `Tooltip`s filhos do documento; o da assinatura diz
/// o parâmetro activo no valor.
#[test]
fn a_assinatura_e_o_balao_falam_ao_leitor_de_ecra() {
    use ph2d_a11y::Role;
    let (mut m, ids, _) = mesa(&[("a.luau", "local brilho = 1\n\n")]);
    let id = ids[0];
    m.pede(id);
    quadro_em(&mut m, T0);
    m.pinta(id);
    aponta(&mut m, id, 8);
    tecla(&mut m, id, Key::End, crate::ctrl(), None, "");
    escreve(&mut m, id, "ph2d.set(");
    // As teclas largaram o balão: o rato volta a apontar.
    m.pinta(id);
    aponta(&mut m, id, 8);
    ate(&mut m, id, T0 + PAUSA_BALAO, "os dois não abriram", |m| {
        m.docs().signature_of(id).is_some() && m.docs().hover_of(id).is_some()
    });
    let nos = m.docs().a11y(id.0);
    let doc = &nos[0].1;
    assert_eq!(doc.role(), Role::Document);
    let dicas: Vec<_> = nos
        .iter()
        .filter(|(_, n)| n.role() == Role::Tooltip)
        .collect();
    assert_eq!(dicas.len(), 2);
    for (nid, _) in &dicas {
        assert!(doc.children().contains(&(*nid).into()));
    }
    assert!(
        dicas
            .iter()
            .any(|(_, n)| n.value() == Some("entity: number")),
        "o parâmetro activo não foi dito"
    );
}

/// Sem assistente, nem o balão nem a assinatura abrem.
#[test]
fn sem_assistente_nem_balao_nem_assinatura() {
    let (mut m, ids) = Mesa::new(&[("a.luau", "local brilho = 1\n")]);
    let id = ids[0];
    m.pede(id);
    quadro_em(&mut m, T0);
    m.pinta(id);
    aponta(&mut m, id, 8);
    tecla(&mut m, id, Key::End, crate::ctrl(), None, "");
    escreve(&mut m, id, "f(");
    nunca(
        &mut m,
        id,
        T0 + PAUSA_BALAO * 2,
        "abriu sem assistente",
        |m| m.docs().signature_of(id).is_none() && m.docs().hover_of(id).is_none(),
    );
}

/// A assinatura e a lista de completação abertas juntas nunca se tapam: com a lista por baixo da
/// linha, a assinatura sobe; na 1.ª linha (sem espaço por cima), vai por baixo da lista.
#[test]
fn a_assinatura_e_a_lista_nunca_se_tapam() {
    let (mut m, ids, _) = mesa(&[("a.luau", "\n\n\n\n\n"), ("b.luau", "")]);
    for (id, fim) in [(ids[0], true), (ids[1], false)] {
        m.pede(id);
        if fim {
            tecla(&mut m, id, Key::End, crate::ctrl(), None, "");
        }
        quadro_em(&mut m, T0);
        escreve(&mut m, id, "ph2d.set(self.");
        ate(
            &mut m,
            id,
            T0 + 300_000_000,
            "a lista e a assinatura não abriram",
            |m| m.docs().completion_painted(id).is_some() && m.docs().hints_painted(id).1.is_some(),
        );
        let s = m.docs().hints_painted(id).1.unwrap().rect;
        let linhas = m.docs().completion_painted(id).unwrap();
        let (lado, ok) = if fim {
            ("por cima", linhas.iter().all(|l| l.3.1 > s.y + s.h))
        } else {
            ("por baixo da lista", linhas.iter().all(|l| l.3.1 < s.y))
        };
        assert!(ok, "a assinatura não ficou {lado}: {s:?} × {linhas:?}");
    }
}
