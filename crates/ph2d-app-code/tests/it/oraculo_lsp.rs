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
use std::time::Duration;

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
