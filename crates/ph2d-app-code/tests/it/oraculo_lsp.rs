//! ⭐ **E7 — o ORÁCULO de posição do símbolo culpado**: o `luau-lsp` fixado corre sobre o script
//! quebrado do smoke (`docs/CodeEditor/ferramentas/smoke/script_quebrado.luau`) e o `range` do
//! diagnóstico dele tem de ser EXACTAMENTE o culpado que a aba sublinha (vindo do tree-sitter). A
//! saída dele está gravada em `fixtures/lsp_script_quebrado.txt` (com cabeçalho); a corrida de hoje
//! tem de bater com ela — um `luau-lsp` que mude de opinião reprova aqui, não no ecrã.
//!
//! Regravar a fixtura (só depois de ler a diferença): `PH2D_GRAVA_ORACULO=1` neste teste.

use crate::Mesa;
use ph2d_code_lsp::{
    LspClient, ServerConfig, Solver, pinned_version, position_to_byte, resolve_binary,
};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("raiz do repo")
}

const FIXTURA: &str = "tests/it/fixtures/lsp_script_quebrado.txt";

#[test]
fn o_culpado_do_tree_sitter_e_o_range_do_luau_lsp() {
    let script = repo().join("docs/CodeEditor/ferramentas/smoke/script_quebrado.luau");
    let texto = std::fs::read_to_string(&script).expect("script quebrado do smoke");
    // O lado do PH2D: o problema que a aba mostra.
    let (mut m, ids) = Mesa::new(&[("script_quebrado.luau", &texto)]);
    m.pede(ids[0]);
    let problemas = m.docs().problems_of(ids[0]);
    let p = problemas.first().expect("a aba vê o erro");
    let culpado = p.culprit.clone().expect("com culpado");
    let inicio_da_linha: usize = texto.split_inclusive('\n').take(p.line).map(str::len).sum();
    assert_eq!(
        (
            p.line,
            culpado.start - inicio_da_linha,
            culpado.end - inicio_da_linha
        ),
        (2, 10, 11),
        "linha 3, o segundo `=`"
    );

    // O oráculo.
    let bin = match resolve_binary() {
        Ok((p, _)) => p,
        Err(e) => panic!(
            "⛔ {e}\n   Este gate precisa do luau-lsp fixado: `bash scripts/luau-lsp-fetch.sh`."
        ),
    };
    let mut cfg =
        ServerConfig::new(bin).with_definitions("@ph2d", repo().join("runtime/luau/ph2d.d.luau"));
    cfg.solver = Solver::Old;
    let mut c = LspClient::start(cfg).expect("luau-lsp arranca");
    let uri = "file:///ph2d-gate/script_quebrado.luau";
    c.pump();
    c.did_open(uri, &texto).expect("didOpen");
    let diags = c
        .wait_diagnostics(uri, Duration::from_secs(30))
        .expect("diagnósticos")
        .diagnostics;
    c.shutdown().expect("shutdown");
    // O luau-lsp 1.70.1 dá o erro de sintaxe com o código `1014` e a mensagem prefixada
    // («SyntaxError: …»); os outros dois são lints (aviso) sobre a mesma linha.
    let sintaxe: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == Some(1) && d.message.starts_with("SyntaxError"))
        .collect();
    assert_eq!(sintaxe.len(), 1, "um erro de sintaxe: {diags:?}");
    let d = sintaxe[0];
    let (a, b) = (
        position_to_byte(&texto, d.range.start),
        position_to_byte(&texto, d.range.end),
    );
    assert_eq!(
        a..b,
        culpado,
        "o range do luau-lsp ({:?}) ≠ o culpado do tree-sitter",
        d.range
    );

    // A fixtura gravada.
    let corrida: String = sintaxe
        .iter()
        .map(|d| {
            format!(
                "{}:{}-{}:{} [{}] {}\n",
                d.range.start.line,
                d.range.start.character,
                d.range.end.line,
                d.range.end.character,
                d.code.as_deref().unwrap_or("-"),
                d.message
            )
        })
        .collect();
    let caminho = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURA);
    if std::env::var_os("PH2D_GRAVA_ORACULO").is_some() {
        let cabecalho = format!(
            "# Oráculo do E7 — luau-lsp {} (fixado, ph2d-code-lsp), solver antigo, definições runtime/luau/ph2d.d.luau.\n\
             # Entrada: docs/CodeEditor/ferramentas/smoke/script_quebrado.luau. Uma linha por diagnóstico de ERRO de sintaxe:\n\
             # <linha>:<carácter UTF-16>-<linha>:<carácter> [código] mensagem (0-based, como o LSP os dá).\n\
             # Gravado por crates/ph2d-app-code/tests/it/oraculo_lsp.rs com PH2D_GRAVA_ORACULO=1.\n",
            pinned_version()
        );
        std::fs::write(&caminho, cabecalho + &corrida).expect("grava a fixtura");
    }
    let gravada = std::fs::read_to_string(&caminho).expect("fixtura do oráculo");
    let corpo: String = gravada
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_eq!(
        corrida, corpo,
        "o luau-lsp mudou de opinião sobre o script quebrado"
    );
}

fn fixado() -> PathBuf {
    match resolve_binary() {
        Ok((p, _)) => p,
        Err(e) => panic!(
            "⛔ {e}\n   Este gate precisa do luau-lsp fixado: `bash scripts/luau-lsp-fetch.sh`."
        ),
    }
}

const USA_A_API: &str =
    "function update(self, dt)\n  ph2d.set(self.id, \"x\", ph2d.get(self.id, \"x\") + dt)\nend\n";

/// ⭐ I2 de ponta a ponta com o `luau-lsp` FIXADO, pelo caminho do produto (`IntelServer::Pinned`:
/// `resolve_binary`, as definições e a documentação geradas da API, a sessão, o espelho): o script
/// que usa a API não dá «Unknown global» (as definições chegaram), o vizinho com um global inventado
/// dá, e o que se ESCREVE na aba — sem salvar — chega ao servidor e volta como diagnóstico.
#[test]
fn o_assistente_fixado_ve_a_api_e_o_que_se_escreve_sem_salvar() {
    let _ = fixado();
    let (m, ids) = Mesa::new(&[("bom.luau", USA_A_API), ("mau.luau", "nao_existe()\n")]);
    let mut m = m.com_assistente(ph2d_app_code::IntelServer::Pinned);
    let (bom, mau) = (ids[0], ids[1]);
    m.pede(bom);
    let mensagens = |m: &mut Mesa, id| -> Option<Vec<String>> {
        let (_, d) = m.docs().assist_diagnostics(id);
        d.map(|d| d.into_iter().map(|x| x.message).collect())
    };
    let espera = |m: &mut Mesa, onde: &str, ok: &dyn Fn(&mut Mesa) -> bool| {
        let fim = std::time::Instant::now() + Duration::from_secs(30);
        while !ok(m) {
            assert!(
                std::time::Instant::now() < fim,
                "{onde}: sem resposta do luau-lsp ({:?})",
                m.docs().assist_reason()
            );
            m.quadro();
            std::thread::sleep(Duration::from_millis(10));
        }
    };
    espera(&mut m, "abrir", &|m| {
        mensagens(m, bom).is_some() && mensagens(m, mau).is_some()
    });
    let desconhecido = |ms: &[String]| ms.iter().any(|s| s.contains("Unknown global"));
    let b = mensagens(&mut m, bom).unwrap();
    assert!(
        !desconhecido(&b),
        "as definições da API não chegaram: {b:?}"
    );
    let ma = mensagens(&mut m, mau).unwrap();
    assert!(
        ma.iter().any(|s| s.contains("Unknown global 'nao_existe'")),
        "{ma:?}"
    );
    // Escrever na aba (sem salvar) muda o que o servidor vê.
    m.tecla(
        bom,
        ph2d_editor_core::doc_surface::Key::End,
        crate::ctrl(),
        None,
    );
    m.escreve(bom, "outro_inventado()\n");
    espera(&mut m, "escrever", &|m| {
        mensagens(m, bom).is_some_and(|ms| ms.iter().any(|s| s.contains("outro_inventado")))
    });
    assert_ne!(
        m.texto_salvo(bom),
        m.docs().text_of(bom).unwrap(),
        "nada se salvou"
    );
    assert_eq!(m.docs().assist_text(), None);
}

/// ⭐ A base de documentação gerada da API (`--docs`, I0b §3) chega à ajuda de assinatura do
/// `luau-lsp` fixado: a linha da função, e a do parâmetro `entity` é a do tipo `Entity`.
#[test]
fn a_documentacao_gerada_chega_a_assinatura() {
    use ph2d_script::api_table::{
        API, API_TYPES, DEFINITIONS_PACKAGE, luau_definitions, luau_docs,
    };
    let dir = std::env::temp_dir().join(format!("ph2d-gate-docs-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("pasta");
    let (defs, docs) = (dir.join("ph2d.d.luau"), dir.join("ph2d-docs.json"));
    std::fs::write(&defs, luau_definitions()).expect("defs");
    std::fs::write(&docs, luau_docs()).expect("docs");
    let mut cfg = ServerConfig::new(fixado()).with_definitions(DEFINITIONS_PACKAGE, defs);
    cfg.docs = Some(docs);
    let mut c = LspClient::start(cfg).expect("luau-lsp arranca");
    let uri = "file:///ph2d-gate/assinatura.luau";
    let texto = "local e = 1\nph2d.set(e, ";
    c.pump();
    c.did_open(uri, texto).expect("didOpen");
    let _ = c.wait_diagnostics(uri, Duration::from_secs(30));
    let r = c
        .request(
            "textDocument/signatureHelp",
            serde_json::json!({ "textDocument": { "uri": uri },
                                "position": { "line": 1, "character": 12 } }),
        )
        .expect("signatureHelp");
    c.shutdown().expect("shutdown");
    let _ = std::fs::remove_dir_all(&dir);
    let texto_de = |v: &serde_json::Value| {
        v.as_str()
            .or_else(|| v["value"].as_str())
            .unwrap_or_default()
            .to_owned()
    };
    let sig = &r["signatures"][0];
    let set = API
        .iter()
        .find(|e| e.table == "ph2d" && e.ident == "set")
        .unwrap();
    let entity = API_TYPES.iter().find(|t| t.ident == "Entity").unwrap();
    assert!(texto_de(&sig["documentation"]).contains(set.doc), "{sig:#}");
    assert!(
        texto_de(&sig["parameters"][0]["documentation"]).contains(entity.doc),
        "{sig:#}"
    );
}

/// ⭐ I3 de ponta a ponta com o `luau-lsp` FIXADO: os erros do assistente aparecem na aba ENQUANTO se
/// escreve, sem salvar, um por erro e no sítio.
/// - a fixtura `erros_de_proposito` colada numa aba vazia: exactamente os dois `-- ERR` sublinhados;
/// - o `bob.luau` canónico: nada (os ganchos `init`/`update` são do motor, não «nunca usados»);
/// - `if x = 1 then x = 2 end`: UM problema (dois `1014`, o global e o lint fundem-se), com a
///   mensagem do Luau no lugar do tree-sitter; o `end` em falta: UM;
/// - `--!strict` num ficheiro acrescenta o erro de tipo só a ele.
#[test]
fn o_assistente_fixado_sublinha_o_que_se_escreve() {
    use ph2d_editor_core::doc_surface::DocumentSurface;
    let _ = fixado();
    let erros = std::fs::read_to_string(
        repo().join("crates/ph2d-code-lsp/tests/it/fixtures/erros_de_proposito.luau"),
    )
    .expect("fixtura");
    let bob =
        std::fs::read_to_string(repo().join("crates/ph2d-code-syntax/tests/it/fixtures/bob.luau"))
            .expect("bob");
    let sem_end = "function update(self, dt)\n  if dt then\n    print(1)\nend\n";
    let (m, ids) = Mesa::new(&[
        ("erros.luau", ""),
        ("bob.luau", &bob),
        ("sintaxe.luau", "if x = 1 then x = 2 end\n"),
        ("sem_end.luau", sem_end),
        ("estrito.luau", "--!strict\nlocal k = 5\nk()\n"),
        ("frouxo.luau", "local k = 5\nk()\n"),
    ]);
    let mut m = m.com_assistente(ph2d_app_code::IntelServer::Pinned);
    for id in &ids {
        m.pede(*id);
    }
    let e = ids[0];
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let _ = m.docs().key(
        e.0,
        &mut ts,
        ph2d_editor_core::doc_surface::KeyInput {
            key: ph2d_editor_core::doc_surface::Key::Char('v'),
            mods: crate::ctrl(),
            pressed: true,
            text: Some("v"),
            paste: Some(erros.clone()),
        },
    );
    // Cada aba com a resposta ao texto DELA e a árvore do worker assente.
    let fim = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        m.quadro();
        let pronto = ids.iter().all(|id| {
            let d = m.docs();
            d.assist_diagnostics(*id).1.is_some() && d.grammar_of(*id).is_some_and(|g| !g.stale)
        });
        if pronto {
            break;
        }
        assert!(
            std::time::Instant::now() < fim,
            "sem resposta do luau-lsp ({:?})",
            m.docs().assist_reason()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(m.docs().text_of(e).as_deref(), Some(erros.as_str()));
    assert_eq!(m.texto_salvo(e), "", "nada se salvou");
    let resumo = |m: &mut Mesa, id| -> Vec<(usize, String)> {
        m.docs()
            .problems_of(id)
            .into_iter()
            .map(|p| (p.line, p.message))
            .collect()
    };

    // A fixtura: as linhas `-- ERR <code> <agulha>`, e só elas.
    let esperado: Vec<(usize, &str)> = erros
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.starts_with("--"))
        .filter_map(|(i, l)| {
            let (_, marca) = l.split_once("-- ERR ")?;
            Some((i, marca.split_once(' ')?.1))
        })
        .collect();
    assert_eq!(esperado.len(), 2, "a fixtura perdeu as marcas");
    let visto = resumo(&mut m, e);
    assert_eq!(visto.len(), esperado.len(), "{visto:#?}");
    for ((linha, msg), (l, agulha)) in visto.iter().zip(&esperado) {
        assert_eq!(linha, l, "{visto:#?}");
        assert!(msg.contains(agulha), "linha {l}: «{msg}» sem {agulha}");
    }
    m.pinta(e);
    let sublinhado: Vec<usize> = m.docs().underlines_of(e).iter().map(|u| u.line).collect();
    assert_eq!(
        sublinhado,
        esperado.iter().map(|(l, _)| *l).collect::<Vec<_>>(),
        "a pintura sublinha cada erro, no ecrã"
    );

    assert_eq!(resumo(&mut m, ids[1]), [], "o bob canónico está limpo");

    let s = resumo(&mut m, ids[2]);
    assert_eq!(s.len(), 1, "{s:#?}");
    assert!(s[0].1.starts_with("SyntaxError: Expected 'then'"), "{s:#?}");
    let s = resumo(&mut m, ids[3]);
    assert_eq!(s.len(), 1, "{s:#?}");
    assert!(s[0].1.starts_with("SyntaxError: Expected 'end'"), "{s:#?}");

    let s = resumo(&mut m, ids[4]);
    assert_eq!(s.len(), 1, "{s:#?}");
    assert_eq!(s[0].0, 2);
    assert!(
        s[0].1.contains("Cannot call a value of type number"),
        "{s:#?}"
    );
    assert_eq!(resumo(&mut m, ids[5]), [], "sem --!strict, o vizinho não");
}

/// ⭐ I4 com o `luau-lsp` FIXADO, pelo caminho do produto: depois de `ph2d.` a lista é EXACTAMENTE a
/// `api_table` (um membro a mais ou a menos reprova aqui); depois de `self.` entram as propriedades
/// declaradas e os campos atribuídos; dentro de `ph2d.set(self.id, "|")`, os cinco campos que o
/// tipo aceita; e aceitar `set` deixa o cursor nos parênteses.
#[test]
fn o_assistente_fixado_completa_a_api() {
    use crate::completar::abre;
    let _ = fixado();
    let props =
        "ph2d.property(\"amplitude\", 1)\nfunction update(self, dt)\n  self.t = 0\n  \nend\n";
    let (m, ids) = Mesa::new(&[("a.luau", ""), ("b.luau", props), ("c.luau", "")]);
    let mut m = m.com_assistente(ph2d_app_code::IntelServer::Pinned);
    for id in &ids {
        m.pede(*id);
    }
    let mut l = abre(&mut m, ids[0], "ph2d.");
    let mut api: Vec<&str> = ph2d_script::api_table::API
        .iter()
        .filter(|e| e.table == "ph2d")
        .map(|e| e.ident)
        .collect();
    l.sort_unstable();
    api.sort_unstable();
    assert_eq!(l, api);
    // `self.` na linha vazia do `update`.
    let b = ids[1];
    let k = |m: &mut Mesa, key, mods| crate::assistente::tecla(m, b, key, mods, None, "");
    use ph2d_editor_core::doc_surface::Key;
    k(&mut m, Key::Home, crate::ctrl());
    for _ in 0..3 {
        k(&mut m, Key::Down, ph2d_host::Modifiers::default());
    }
    k(&mut m, Key::End, ph2d_host::Modifiers::default());
    let mut l = abre(&mut m, b, "self.");
    l.sort_unstable();
    assert_eq!(l, ["amplitude", "t"]);
    // O literal: os campos de `ph2d.set`.
    let c = ids[2];
    let mut l = abre(&mut m, c, "ph2d.set(self.id, \"");
    l.sort_unstable();
    assert_eq!(l, ["rotation", "scale_x", "scale_y", "x", "y"]);
}

/// ⭐ I5 com o `luau-lsp` FIXADO, pelo caminho do produto: dentro de `ph2d.set(e, |` a assinatura
/// marca o 2.º parâmetro (`field`) e traz a documentação GERADA da `api_table` (a da função e a do
/// tipo do parâmetro, pelo `--docs`); o rato parado sobre `set` mostra a documentação da função.
#[test]
fn o_assistente_fixado_marca_o_argumento_e_mostra_o_balao() {
    use crate::assistente::tecla;
    use crate::completar::{T0, escreve, quadro_em};
    use ph2d_editor_core::doc_surface::Key;
    use ph2d_script::api_table::{API, API_TYPES};
    let _ = fixado();
    let (m, ids) = Mesa::new(&[("a.luau", "local e = 1\n")]);
    let mut m = m.com_assistente(ph2d_app_code::IntelServer::Pinned);
    let id = ids[0];
    m.pede(id);
    tecla(&mut m, id, Key::End, crate::ctrl(), None, "");
    quadro_em(&mut m, T0);
    escreve(&mut m, id, "ph2d.set(e, ");
    let fim = Instant::now() + Duration::from_secs(30);
    let (label, activo) = loop {
        quadro_em(&mut m, T0);
        m.pinta(id);
        if let Some((l, Some(a))) = m.docs().signature_of(id)
            && a.starts_with("field")
        {
            break (l, a);
        }
        assert!(
            Instant::now() < fim,
            "a assinatura de ph2d.set não marcou `field`"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    assert!(label.contains("ph2d.set(entity"), "{label}");
    let (_, s) = m.docs().hints_painted(id);
    let s = s.expect("a assinatura pintou-se");
    assert_eq!(
        s.marked.join(" "),
        activo,
        "a marca pintada é o parâmetro activo"
    );
    let dito = s.lines.join(" ");
    let set = API
        .iter()
        .find(|e| e.table == "ph2d" && e.ident == "set")
        .unwrap();
    let campo = API_TYPES.iter().find(|t| t.ident == "PoseField").unwrap();
    for doc in [set.doc, campo.doc] {
        assert!(dito.contains(doc), "falta «{doc}» em {:?}", s.lines);
    }
    // O balão sobre `set`.
    let t = m.docs().text_of(id).unwrap();
    let b = t.find("set(").unwrap() + 1;
    let (x, y) = m.docs().glyph_center(id, b).expect("à vista");
    m.rato(id, x, y);
    let fim = Instant::now() + Duration::from_secs(30);
    loop {
        quadro_em(&mut m, T0 + 500_000_000);
        if m.docs().hover_of(id).is_some_and(|h| h.contains(set.doc)) {
            break;
        }
        assert!(
            Instant::now() < fim,
            "o balão de `set` não trouxe a documentação"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// ⭐ I6 e I11 com o `luau-lsp` FIXADO, pelo caminho do produto: Ctrl+clique num uso de um local vai à
/// declaração; F12 em `ph2d.set` (o servidor dá `[]`) abre a documentação GERADA no balão; as cores
/// do servidor pintam `ph2d` e `set` na cor da biblioteca (o tree-sitter dá-lhes variável e função),
/// e `pirnt` — que não tem token nenhum — na de alerta, pelo diagnóstico.
#[test]
fn o_assistente_fixado_vai_a_definicao_e_pinta_as_cores() {
    use crate::assistente::tecla;
    use crate::completar::{T0, quadro_em};
    use ph2d_editor_core::doc_surface::Key;
    use ph2d_host::Modifiers;
    use ph2d_panel_code::Ink;
    use ph2d_script::api_table::API;
    let _ = fixado();
    let texto = "local brilho = 1\nfunction update(self, dt)\n  ph2d.set(self.id, \"x\", brilho)\n  pirnt(dt)\nend\n";
    let (m, ids) = Mesa::new(&[("a.luau", texto)]);
    let mut m = m.com_assistente(ph2d_app_code::IntelServer::Pinned);
    let id = ids[0];
    m.pede(id);
    let espera = |m: &mut Mesa, onde: &str, feito: &dyn Fn(&mut Mesa) -> bool| {
        let fim = Instant::now() + Duration::from_secs(30);
        loop {
            quadro_em(m, T0);
            m.pinta(id);
            if feito(m) {
                return;
            }
            assert!(Instant::now() < fim, "{onde}");
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    let ph2d = texto.find("ph2d").unwrap();
    let pirnt = texto.find("pirnt").unwrap();
    espera(&mut m, "as cores do servidor não pintaram `ph2d.set`", &|m| {
        m.docs().ink_painted(id, ph2d) == Ink::Builtin
            && m.docs().ink_painted(id, ph2d + 6) == Ink::Builtin
    });
    espera(&mut m, "`pirnt` não pintou de alerta", &|m| {
        m.docs().ink_painted(id, pirnt + 1) == Ink::Unresolved
    });
    assert_ne!(
        m.docs().ink_painted(id, texto.find("brilho)").unwrap()),
        Ink::Builtin,
        "um local não é da biblioteca"
    );
    // Ctrl+clique no `brilho` da chamada → a declaração da 1.ª linha.
    let uso = texto.find("brilho)").unwrap() + 2;
    let (x, y) = m.docs().glyph_center(id, uso).expect("à vista");
    m.clica(id, x, y, crate::ctrl());
    espera(&mut m, "o Ctrl+clique não foi à declaração", &|m| {
        m.docs().cursor_of(id).map(|c| c.1) == Some(6)
    });
    // F12 em `set` → a documentação da função, e o cursor fica.
    let set = ph2d + 6;
    let (x, y) = m.docs().glyph_center(id, set).expect("à vista");
    m.clica(id, x, y, Modifiers::default());
    let antes = m.docs().cursor_of(id);
    tecla(&mut m, id, Key::F(12), Modifiers::default(), None, "");
    let doc = API
        .iter()
        .find(|e| e.table == "ph2d" && e.ident == "set")
        .unwrap()
        .doc;
    espera(&mut m, "o F12 em `set` não abriu a documentação", &|m| {
        m.docs().hover_of(id).is_some_and(|h| h.contains(doc))
    });
    assert_eq!(m.docs().cursor_of(id), antes, "o F12 na API não mexe no cursor");
}
