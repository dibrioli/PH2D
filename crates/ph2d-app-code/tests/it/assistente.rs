//! I2 no app (Onda 2, `docs/CodeEditor/03_onda2.md`): o assistente de código vê o texto que o editor
//! MOSTRA, de todos os scripts do projeto, e nunca faz a tecla esperar. O servidor aqui é FALSO, no
//! processo: reconstrói cada documento pelo que recebe (`didOpen` + `didChange` incremental em
//! UTF-16, com a conversão dele, não a do editor), publica um diagnóstico por versão, e pode ser lento
//! ou morrer.

use crate::{Mesa, ctrl};
use ph2d_app_code::IntelServer;
use ph2d_app_code::commands::CMD_RESTART_ASSIST;
use ph2d_code_intel::lsp::Connect;
use ph2d_code_intel::uri_of;
use ph2d_code_lsp::framing::{read_message, write_message};
use ph2d_code_lsp::{LspClient, LspError, ServerConfig};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key, KeyInput};
use ph2d_host::Modifiers;
use ph2d_script::{ProjectScripts, ScriptId};
use ph2d_text::TextSystem;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct Visto {
    ligacoes: usize,
    /// uri → (versão, texto), do servidor VIVO (um arranque novo começa vazio).
    textos: BTreeMap<String, (i64, String)>,
    /// `didOpen` por uri, em todos os arranques.
    aberturas: BTreeMap<String, usize>,
    mudancas: usize,
    /// Uma versão que não subiu (uri, antes, depois).
    para_tras: Vec<(String, i64, i64)>,
    /// Os pedidos `textDocument/completion` recebidos.
    pub(crate) pedidos: usize,
    /// Os pedidos `textDocument/hover` e `textDocument/signatureHelp` recebidos.
    pub(crate) baloes: usize,
    pub(crate) assinaturas: usize,
    /// Os pedidos `textDocument/definition` e `semanticTokens/full` recebidos.
    pub(crate) definicoes: usize,
    pub(crate) cores: usize,
}

pub(crate) type Partilhado = Arc<Mutex<Visto>>;

/// Os sítios de uma definição (bytes no documento do pedido).
pub(crate) type Sitios = Vec<std::ops::Range<usize>>;
/// Os tokens de um texto: `(bytes, tipo, modificadores)`.
pub(crate) type Tokens = Vec<(std::ops::Range<usize>, u64, u64)>;

#[derive(Clone, Copy, Default)]
pub(crate) struct Plano {
    /// Antes de ler cada mensagem.
    pub(crate) atraso: Duration,
    /// O 1.º arranque morre depois deste número de `didChange`.
    pub(crate) morre_apos: Option<usize>,
    /// Os diagnósticos que o servidor publica para um texto (`None` = um só, `v<versão>`, em 0:0).
    pub(crate) regra: Option<fn(&str) -> Vec<Value>>,
    /// A resposta a `textDocument/completion` sobre o texto e o byte do pedido (`None` = `null`).
    pub(crate) completar: Option<fn(&str, usize) -> Value>,
    /// As respostas a `textDocument/hover` e a `textDocument/signatureHelp` (`None` = `null`).
    pub(crate) pairar: Option<fn(&str, usize) -> Value>,
    pub(crate) assinar: Option<fn(&str, usize) -> Value>,
    /// `textDocument/definition`: os bytes, NESTE documento, de onde o nome no byte do pedido está
    /// definido (`None` = `null`).
    pub(crate) definir: Option<fn(&str, usize) -> Sitios>,
    /// `semanticTokens/full`: `(bytes, tipo, modificadores)` de cada nome, pela ordem do texto, na
    /// legenda do `luau-lsp` 1.70.1 (`None` = `null`).
    pub(crate) colorir: Option<fn(&str) -> Tokens>,
}

/// As capacidades que o `luau-lsp` 1.70.1 anunciou (a legenda dos tokens vem daqui).
fn capacidades() -> Value {
    let c: Value = serde_json::from_str(include_str!(
        "../../../../docs/CodeEditor/ferramentas/medicoes_onda2/caps_luau_lsp_1.70.1.json"
    ))
    .expect("as capacidades gravadas");
    c["capabilities"].clone()
}

/// `SemanticTokens.data`: (linha, início) relativos ao anterior, comprimento em UTF-16.
fn codifica(s: &str, toks: &[(std::ops::Range<usize>, u64, u64)]) -> Value {
    let (mut linha, mut col) = (0u64, 0u64);
    let mut data = Vec::new();
    for (r, tipo, mods) in toks {
        let p = posicao(s, r.start);
        let (l, c) = pos(&p);
        let n = s[r.clone()].encode_utf16().count() as u64;
        let (dl, dc) = if l == linha {
            (0, c - col)
        } else {
            (l - linha, c)
        };
        data.extend([dl, dc, n, *tipo, *mods]);
        (linha, col) = (l, c);
    }
    json!({ "data": data })
}

/// O byte da posição LSP `(linha, unidades UTF-16)`; além do fim da linha = o fim da linha.
fn byte_de(s: &str, linha: u64, unidades: u64) -> usize {
    let inicio = s
        .split_inclusive('\n')
        .take(linha as usize)
        .map(str::len)
        .sum::<usize>();
    let resto = &s[inicio.min(s.len())..];
    let mut u = 0;
    for (i, c) in resto.char_indices() {
        if c == '\n' || u >= unidades {
            return inicio + i;
        }
        u += c.len_utf16() as u64;
    }
    s.len()
}

/// A posição LSP `{line, character}` (unidades UTF-16) do byte `b` de `s`.
fn posicao(s: &str, b: usize) -> Value {
    let inicio = s[..b].rfind('\n').map_or(0, |i| i + 1);
    json!({ "line": s[..b].matches('\n').count(),
            "character": s[inicio..b].encode_utf16().count() })
}

/// O servidor falso acusa cada `pirnt` do texto (um «Unknown global», como o `luau-lsp`).
pub(crate) fn acusa_pirnt(s: &str) -> Vec<Value> {
    s.match_indices("pirnt")
        .map(|(b, w)| {
            json!({ "range": { "start": posicao(s, b), "end": posicao(s, b + w.len()) },
                    "severity": 1, "code": 1001,
                    "message": "TypeError: Unknown global 'pirnt'; consider assigning to it first" })
        })
        .collect()
}

fn pos(v: &Value) -> (u64, u64) {
    (
        v["line"].as_u64().unwrap(),
        v["character"].as_u64().unwrap(),
    )
}

fn servir(
    entrada: std::io::PipeReader,
    mut saida: std::io::PipeWriter,
    plano: Plano,
    visto: Partilhado,
    arranque: usize,
) {
    let mut r = BufReader::new(entrada);
    let mut mudancas = 0usize;
    loop {
        std::thread::sleep(plano.atraso);
        let Ok(Some(msg)) = read_message(&mut r) else {
            return;
        };
        let method = msg["method"].as_str().unwrap_or_default().to_owned();
        let p = &msg["params"];
        let mut publica = None;
        {
            let mut v = visto.lock().unwrap();
            match method.as_str() {
                "textDocument/didOpen" => {
                    let d = &p["textDocument"];
                    let uri = d["uri"].as_str().unwrap().to_owned();
                    let ver = d["version"].as_i64().unwrap();
                    *v.aberturas.entry(uri.clone()).or_default() += 1;
                    let texto = d["text"].as_str().unwrap().to_owned();
                    v.textos.insert(uri.clone(), (ver, texto.clone()));
                    publica = Some((uri, ver, texto));
                }
                "textDocument/didChange" => {
                    let uri = p["textDocument"]["uri"].as_str().unwrap().to_owned();
                    let ver = p["textDocument"]["version"].as_i64().unwrap();
                    v.mudancas += 1;
                    let (antes, texto) =
                        v.textos.get(&uri).cloned().expect("didChange sem didOpen");
                    if ver <= antes {
                        v.para_tras.push((uri.clone(), antes, ver));
                    }
                    let mut texto = texto;
                    for c in p["contentChanges"].as_array().unwrap() {
                        let novo = c["text"].as_str().unwrap();
                        match c.get("range") {
                            Some(rg) => {
                                let (l0, c0) = pos(&rg["start"]);
                                let (l1, c1) = pos(&rg["end"]);
                                let a = byte_de(&texto, l0, c0);
                                let b = byte_de(&texto, l1, c1);
                                texto.replace_range(a..b, novo);
                            }
                            None => texto = novo.to_owned(),
                        }
                    }
                    v.textos.insert(uri.clone(), (ver, texto.clone()));
                    publica = Some((uri, ver, texto));
                    mudancas += 1;
                }
                "textDocument/didClose" => {
                    let uri = p["textDocument"]["uri"].as_str().unwrap();
                    v.textos.remove(uri);
                }
                _ => {}
            }
        }
        if let Some(id) = msg.get("id") {
            let uri = p["textDocument"]["uri"]
                .as_str()
                .unwrap_or_default()
                .to_owned();
            let texto_de = |conta: fn(&mut Visto)| {
                let mut v = visto.lock().unwrap();
                conta(&mut v);
                v.textos.get(&uri).map(|t| t.1.clone()).unwrap_or_default()
            };
            let result = if method == "initialize" {
                json!({ "capabilities": capacidades() })
            } else if let Some(f) = plano
                .definir
                .filter(|_| method == "textDocument/definition")
            {
                let texto = texto_de(|v| v.definicoes += 1);
                let (l, c) = pos(&p["position"]);
                let sitios = f(&texto, byte_de(&texto, l, c));
                Value::Array(
                    sitios
                        .into_iter()
                        .map(|r| {
                            json!({ "uri": uri, "range": {
                                "start": posicao(&texto, r.start), "end": posicao(&texto, r.end) } })
                        })
                        .collect(),
                )
            } else if let Some(f) = plano
                .colorir
                .filter(|_| method == "textDocument/semanticTokens/full")
            {
                let texto = texto_de(|v| v.cores += 1);
                codifica(&texto, &f(&texto))
            } else if let Some(f) = match method.as_str() {
                "textDocument/completion" => plano.completar,
                "textDocument/hover" => plano.pairar,
                "textDocument/signatureHelp" => plano.assinar,
                _ => None,
            } {
                let uri = p["textDocument"]["uri"].as_str().unwrap_or_default();
                let texto = {
                    let mut v = visto.lock().unwrap();
                    match method.as_str() {
                        "textDocument/completion" => v.pedidos += 1,
                        "textDocument/hover" => v.baloes += 1,
                        _ => v.assinaturas += 1,
                    }
                    v.textos.get(uri).map(|t| t.1.clone())
                };
                let texto = texto.unwrap_or_default();
                let (l, c) = pos(&p["position"]);
                f(&texto, byte_de(&texto, l, c))
            } else {
                Value::Null
            };
            if write_message(
                &mut saida,
                &json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            )
            .is_err()
            {
                return;
            }
        }
        if let Some((uri, ver, texto)) = publica {
            let diagnostics = plano.regra.map_or_else(
                || {
                    vec![json!({
                        "range": { "start": { "line": 0, "character": 0 },
                                   "end": { "line": 0, "character": 0 } },
                        "severity": 1, "message": format!("v{ver}") })]
                },
                |f| f(&texto),
            );
            let n = json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                "params": { "uri": uri, "version": ver, "diagnostics": diagnostics } });
            if write_message(&mut saida, &n).is_err() {
                return;
            }
        }
        if arranque == 0 && plano.morre_apos == Some(mudancas) {
            return;
        }
    }
}

/// Liga um servidor falso novo a cada chamada; `falha_primeiro` = o 1.º arranque não chega a ligar.
pub(crate) fn falso(plano: Plano, visto: Partilhado, falha_primeiro: bool) -> Connect {
    Box::new(move |wake| {
        let arranque = {
            let mut v = visto.lock().unwrap();
            v.ligacoes += 1;
            v.textos.clear();
            v.ligacoes - 1
        };
        if falha_primeiro && arranque == 0 {
            return Err(LspError::Spawn(std::io::Error::other("arranque partido")));
        }
        let (srv_in, cli_out) = std::io::pipe().expect("cano");
        let (cli_in, srv_out) = std::io::pipe().expect("cano");
        let v = visto.clone();
        std::thread::spawn(move || servir(srv_in, srv_out, plano, v, arranque));
        LspClient::connect(
            ServerConfig::new(PathBuf::from("falso")),
            Box::new(cli_out),
            Box::new(cli_in),
            Some(wake),
        )
    })
}

pub(crate) fn com_falso(
    scripts: &[(&str, &str)],
    plano: Plano,
) -> (Mesa, Vec<ScriptId>, Partilhado) {
    let visto = Partilhado::default();
    let (m, ids) = Mesa::new(scripts);
    let m = m.com_assistente(IntelServer::Custom(falso(plano, visto.clone(), false)));
    (m, ids, visto)
}

pub(crate) fn tecla(
    m: &mut Mesa,
    id: ScriptId,
    key: Key,
    mods: Modifiers,
    text: Option<&str>,
    paste: &str,
) {
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

pub(crate) fn letra(m: &mut Mesa, id: ScriptId, c: char) {
    tecla(
        m,
        id,
        Key::Other,
        Modifiers::default(),
        Some(&c.to_string()),
        "",
    );
}

/// O que o editor MOSTRA de cada script: a aba aberta, a fechada com ●, senão a biblioteca.
fn mostrado(m: &mut Mesa) -> BTreeMap<String, String> {
    let lib: Vec<(ScriptId, String, String)> = m
        .sim
        .world()
        .resource::<ProjectScripts>()
        .iter()
        .map(|(id, d)| (id, d.name.clone(), d.text.clone()))
        .collect();
    lib.into_iter()
        .map(|(id, nome, texto)| {
            let d = m.docs();
            let t = d.text_of(id).or_else(|| d.parked_text(id)).unwrap_or(texto);
            (uri_of(id.0, &nome), t)
        })
        .collect()
}

fn do_servidor(visto: &Partilhado) -> BTreeMap<String, String> {
    visto
        .lock()
        .unwrap()
        .textos
        .iter()
        .map(|(u, (_, t))| (u.clone(), t.clone()))
        .collect()
}

/// Dá quadros até o servidor ter o que o editor mostra (o worker escreve fora do quadro).
pub(crate) fn converge(m: &mut Mesa, visto: &Partilhado, onde: &str) {
    let fim = Instant::now() + Duration::from_secs(20);
    loop {
        m.quadro();
        let (a, b) = (mostrado(m), do_servidor(visto));
        if a == b {
            return;
        }
        assert!(
            Instant::now() < fim,
            "{onde}: o servidor não alcançou o editor\neditor: {a:#?}\nservidor: {b:#?}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// ⛔ Gate: depois de qualquer sequência de gestos — letras com acento e emoji, Enter, apagar,
/// colar várias linhas, desfazer/refazer, salvar, fechar com ● e reabrir, abrir outro script, o
/// texto mudado por fora, renomear, um script novo e um apagado — o servidor tem EXACTAMENTE o texto
/// que o editor mostra de cada script. Só arranca na 1.ª aba; a escrita vai por `didChange` (a aba
/// que nunca se renomeia abre-se UMA vez); as versões só sobem.
#[test]
fn o_assistente_ve_o_texto_que_o_editor_mostra() {
    let (mut m, ids, visto) = com_falso(
        &[
            ("a.luau", "local x = 1\n-- ação 😀\n"),
            ("b.luau", "function update(self, dt)\nend\n"),
            ("c.luau", "print(\"c\")\n"),
        ],
        Plano::default(),
    );
    for _ in 0..3 {
        m.quadro();
    }
    assert_eq!(
        visto.lock().unwrap().ligacoes,
        0,
        "arrancou sem nenhuma aba aberta"
    );
    let (a, b, c) = (ids[0], ids[1], ids[2]);
    m.pede(a);
    converge(&mut m, &visto, "abrir a 1.ª aba");
    assert_eq!(
        do_servidor(&visto).len(),
        3,
        "todos os scripts do projeto, não só a aba"
    );
    let mut cur = a;
    let mut extra: Option<ScriptId> = None;
    let mut rng = fastrand::Rng::with_seed(0x1_2A55);
    let letras = ['a', 'é', '😀', ' ', 'z', '('];
    for i in 0..400 {
        if m.docs().text_of(cur).is_none() {
            m.pede(cur);
        }
        match rng.u8(0..24) {
            0..=7 => letra(&mut m, cur, letras[rng.usize(0..letras.len())]),
            8 => tecla(
                &mut m,
                cur,
                Key::Enter,
                Modifiers::default(),
                Some("\r"),
                "",
            ),
            9 => tecla(&mut m, cur, Key::Backspace, Modifiers::default(), None, ""),
            10 => tecla(&mut m, cur, Key::Delete, Modifiers::default(), None, ""),
            11 => tecla(&mut m, cur, Key::Char('v'), ctrl(), Some("v"), "x\n é😀\n"),
            12 => tecla(&mut m, cur, Key::Char('z'), ctrl(), Some("z"), ""),
            13 => tecla(&mut m, cur, Key::Char('y'), ctrl(), Some("y"), ""),
            14 => tecla(&mut m, cur, Key::Char('s'), ctrl(), Some("s"), ""),
            15 => tecla(&mut m, cur, Key::Up, Modifiers::default(), None, ""),
            16 => m.docs().close(cur.0),
            17 => cur = [a, b][rng.usize(0..2)],
            18 => {
                let mut lib = m.sim.world_mut().resource_mut::<ProjectScripts>();
                lib.set_text(c, &format!("print({i}) -- ç\n"));
            }
            19 => {
                let mut lib = m.sim.world_mut().resource_mut::<ProjectScripts>();
                lib.rename(c, &format!("c{}.luau", i % 3));
            }
            20 => {
                let mut lib = m.sim.world_mut().resource_mut::<ProjectScripts>();
                match extra.take() {
                    Some(e) => {
                        lib.remove(e);
                    }
                    None => extra = Some(lib.insert("extra.luau", "-- novo\n")),
                }
            }
            _ => tecla(&mut m, cur, Key::End, Modifiers::default(), None, ""),
        }
        if rng.bool() {
            m.quadro();
        }
        if i % 40 == 39 {
            converge(&mut m, &visto, &format!("gesto {i}"));
        }
    }
    converge(&mut m, &visto, "fim");
    // A aba fechada com ●: o servidor fica com o texto POR SALVAR (não o da biblioteca), e reabri-la
    // não lhe manda nada (é o mesmo texto). O laço acima reabre a aba logo a seguir — nunca o vê.
    if m.docs().text_of(a).is_none() {
        m.pede(a);
    }
    letra(&mut m, a, 'Q');
    m.quadro();
    m.docs().close(a.0);
    assert!(m.docs().is_parked(a));
    assert_ne!(m.docs().parked_text(a), Some(m.texto_salvo(a)));
    converge(&mut m, &visto, "fechada com ●");
    let mudancas = visto.lock().unwrap().mudancas;
    m.pede(a);
    converge(&mut m, &visto, "reaberta");
    assert_eq!(
        visto.lock().unwrap().mudancas,
        mudancas,
        "reabrir mandou texto ao servidor"
    );
    let v = visto.lock().unwrap();
    assert_eq!(v.ligacoes, 1);
    assert!(
        v.para_tras.is_empty(),
        "versões que não subiram: {:?}",
        v.para_tras
    );
    assert!(
        v.mudancas > 50,
        "a escrita não foi por didChange: {}",
        v.mudancas
    );
    let ua = uri_of(a.0, "a.luau");
    assert_eq!(
        v.aberturas.get(&ua),
        Some(&1),
        "a aba que se escreve reabriu-se: {:?}",
        v.aberturas
    );
    assert_eq!(
        m.docs().assist_text(),
        None,
        "pronto, a barra não fala do assistente"
    );
}

/// ⛔ Gate: um servidor que demora 1 s em CADA mensagem nunca segura a tecla nem o quadro — 60
/// teclas com quadros pelo meio custam muito menos que UMA espera dessas. E o texto chega-lhe.
#[test]
fn um_servidor_lento_nunca_segura_a_tecla() {
    let (mut m, ids, _visto) = com_falso(
        &[("a.luau", "local x = 1\n")],
        Plano {
            atraso: Duration::from_secs(1),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    let t0 = Instant::now();
    for i in 0..60 {
        letra(&mut m, id, char::from(b'a' + (i % 26) as u8));
        m.quadro();
    }
    let gasto = t0.elapsed();
    assert!(
        gasto < Duration::from_millis(800),
        "a tecla esperou o servidor: {gasto:?}"
    );
    assert_eq!(
        m.docs().text_of(id).unwrap().len(),
        "local x = 1\n".len() + 60
    );
    assert!(
        m.docs()
            .assist_text()
            .is_some_and(|t| t.starts_with("Starting")),
        "o `initialize` ainda não voltou: a barra diz que está a arrancar"
    );
}

/// ⛔ Gate: o servidor morre a meio da escrita; o editor continua a escrever, a sessão repõe-no
/// sozinha, e o servidor novo recebe o texto de AGORA de cada script.
#[test]
fn um_servidor_que_morre_a_meio_volta_com_o_texto_de_agora() {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", "local x = 1\n"), ("b.luau", "-- b\n")],
        Plano {
            morre_apos: Some(5),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    for i in 0..30 {
        letra(&mut m, id, char::from(b'a' + (i % 26) as u8));
        m.quadro();
        std::thread::sleep(Duration::from_millis(2));
    }
    converge(&mut m, &visto, "depois da morte");
    assert_eq!(
        visto.lock().unwrap().ligacoes,
        2,
        "a sessão não repôs o servidor (ou repô-lo duas vezes)"
    );
    assert_eq!(
        m.docs().text_of(id).unwrap().len(),
        "local x = 1\n".len() + 30
    );
    assert_eq!(m.docs().assist_text(), None);
}

/// ⛔ Gate: um diagnóstico de uma versão velha nunca sai para quem pinta — em cada quadro, o que a
/// sonda dá é o da versão que o servidor tem AGORA (o falso escreve `v<versão>`). Controlo: com o
/// servidor alcançado, o diagnóstico da última versão está lá.
#[test]
fn um_diagnostico_de_versao_velha_nunca_sai() {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", "local x = 1\n")],
        Plano {
            atraso: Duration::from_millis(3),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    for i in 0..80 {
        letra(&mut m, id, char::from(b'a' + (i % 26) as u8));
        m.quadro();
        if let (Some(v), Some(d)) = m.docs().assist_diagnostics(id) {
            assert_eq!(d.len(), 1);
            assert_eq!(
                d[0].message,
                format!("v{v}"),
                "quadro {i}: diagnóstico de outra versão"
            );
        }
    }
    converge(&mut m, &visto, "fim");
    let fim = Instant::now() + Duration::from_secs(10);
    let (v, d) = loop {
        m.quadro();
        if let (Some(v), Some(d)) = m.docs().assist_diagnostics(id) {
            break (v, d);
        }
        assert!(
            Instant::now() < fim,
            "o diagnóstico da última versão não chegou"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(d[0].message, format!("v{v}"));
    assert!(v > 1, "a versão não subiu com a escrita: {v}");
}

/// ⛔ Gate: o assistente não arranca — o editor escreve na mesma, a barra diz que está parado e
/// como o repor; o comando «Restart Code Assistant» liga-o, e o servidor recebe o texto de agora.
#[test]
fn sem_assistente_o_editor_escreve_e_o_restart_liga_o() {
    let visto = Partilhado::default();
    let (m, ids) = Mesa::new(&[("a.luau", "local x = 1\n")]);
    let mut m = m.com_assistente(IntelServer::Custom(falso(
        Plano::default(),
        visto.clone(),
        true,
    )));
    let id = ids[0];
    m.pede(id);
    let fim = Instant::now() + Duration::from_secs(10);
    while m.docs().assist_reason().is_none() {
        assert!(
            Instant::now() < fim,
            "o arranque partido não chegou à barra"
        );
        m.quadro();
        std::thread::sleep(Duration::from_millis(5));
    }
    let barra = m.docs().assist_text().expect("a barra fala do assistente");
    assert!(
        barra.contains("editing works") && barra.contains("Restart"),
        "{barra}"
    );
    assert!(
        m.docs()
            .assist_reason()
            .unwrap()
            .contains("arranque partido")
    );
    letra(&mut m, id, 'q');
    m.quadro();
    assert_eq!(m.docs().text_of(id).as_deref(), Some("qlocal x = 1\n"));
    assert!(m.docs().command(id.0, CMD_RESTART_ASSIST));
    converge(&mut m, &visto, "depois do Restart");
    assert_eq!(visto.lock().unwrap().ligacoes, 2);
    let fim = Instant::now() + Duration::from_secs(10);
    while m.docs().assist_text().is_some() {
        assert!(
            Instant::now() < fim,
            "a barra não se calou com o assistente pronto"
        );
        m.quadro();
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// O binário que não existe: a barra diz que parou (e a razão crua diz porquê), a escrita segue.
#[test]
fn um_binario_que_nao_existe_para_o_assistente_e_nao_o_editor() {
    let (m, ids) = Mesa::new(&[("a.luau", "x = 1\n")]);
    let mut m = m.com_assistente(IntelServer::Binary(PathBuf::from(
        "/ph2d-nao-existe/luau-lsp",
    )));
    let id = ids[0];
    m.pede(id);
    let fim = Instant::now() + Duration::from_secs(10);
    while m.docs().assist_reason().is_none() {
        assert!(Instant::now() < fim);
        m.quadro();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        m.docs()
            .assist_reason()
            .unwrap()
            .contains("could not start"),
        "{:?}",
        m.docs().assist_reason()
    );
    assert!(m.docs().assist_text().unwrap().contains("editing works"));
    letra(&mut m, id, 'q');
    assert_eq!(m.docs().text_of(id).as_deref(), Some("qx = 1\n"));
}

/// ⭐ I3: os sublinhados do assistente SEGURAM-SE entre respostas. Cada tecla avança a versão do
/// servidor, e um servidor lento responde sempre atrás; pintado só o da versão corrente, o erro
/// piscaria a cada tecla. Em todo quadro, a aba mostra o erro UMA vez e EXACTAMENTE sobre `pirnt`
/// (seguiu as edições — com acento e emoji antes, numa string, para a conversão UTF-16); apagado o
/// `pirnt`, o erro sai na MESMA tecla, sem esperar o servidor.
#[test]
fn os_sublinhados_seguram_se_entre_respostas() {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", "local s = \"\" pirnt(1)\n")],
        Plano {
            atraso: Duration::from_millis(15),
            regra: Some(acusa_pirnt),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    let sobre_pirnt = |m: &mut Mesa, onde: &str| -> bool {
        let ps = m.docs().problems_of(id);
        let t = m.docs().text_of(id).unwrap();
        match ps.as_slice() {
            [] => false,
            [p] => {
                let c = p.culprit.clone().expect("com culpado");
                assert_eq!(&t[c], "pirnt", "{onde}: o sublinhado saiu do sítio");
                assert!(
                    p.message.contains("Unknown global 'pirnt'"),
                    "{onde}: {p:?}"
                );
                true
            }
            _ => panic!("{onde}: o erro saiu em dobro: {ps:#?}"),
        }
    };
    let fim = Instant::now() + Duration::from_secs(10);
    while !sobre_pirnt(&mut m, "abrir") {
        assert!(Instant::now() < fim, "a 1.ª resposta não chegou");
        m.quadro();
        std::thread::sleep(Duration::from_millis(2));
    }
    // Dentro da string, antes do `pirnt` da mesma linha.
    tecla(&mut m, id, Key::Home, ctrl(), None, "");
    for _ in 0.."local s = \"".len() {
        tecla(&mut m, id, Key::Right, Modifiers::default(), None, "");
    }
    let mut atras = 0;
    for (i, c) in "local é = 1 🙂 ".chars().cycle().take(60).enumerate() {
        letra(&mut m, id, c);
        assert!(
            sobre_pirnt(&mut m, &format!("tecla {i}")),
            "tecla {i}: o erro piscou"
        );
        m.quadro();
        if m.docs().assist_diagnostics(id).1.is_none() {
            atras += 1;
        }
        assert!(
            sobre_pirnt(&mut m, &format!("quadro {i}")),
            "quadro {i}: o erro piscou"
        );
    }
    assert!(
        atras > 0,
        "o servidor nunca ficou atrás: o gate não prova o segurar"
    );
    // Apagar o `pirnt` (o fim da linha é `pirnt(1)`): o erro sai sem esperar a resposta.
    tecla(&mut m, id, Key::End, Modifiers::default(), None, "");
    for _ in 0.."pirnt(1)".len() {
        tecla(&mut m, id, Key::Backspace, Modifiers::default(), None, "");
    }
    assert!(m.docs().problems_of(id).is_empty(), "o erro apagado ficou");
    converge(&mut m, &visto, "fim");
    for _ in 0..20 {
        m.quadro();
        assert!(m.docs().problems_of(id).is_empty());
    }
}

/// O assistente que parou DE VEZ (morreu depois de responder e não voltou a arrancar) não deixa
/// sublinhados a envelhecer na aba: sem quem os confirme, saem; a barra diz que parou.
#[test]
fn um_assistente_parado_nao_deixa_sublinhados() {
    let visto = Partilhado::default();
    let mut primeiro = Some(falso(
        Plano {
            regra: Some(acusa_pirnt),
            morre_apos: Some(1),
            ..Plano::default()
        },
        visto,
        false,
    ));
    let liga: Connect = Box::new(move |wake| match primeiro.take() {
        Some(mut c) => c(wake),
        None => Err(LspError::Spawn(std::io::Error::other("não volta"))),
    });
    let (m, ids) = Mesa::new(&[("a.luau", "pirnt(1)\n")]);
    let mut m = m.com_assistente(IntelServer::Custom(liga));
    let id = ids[0];
    m.pede(id);
    let fim = Instant::now() + Duration::from_secs(10);
    while m.docs().problems_of(id).is_empty() {
        assert!(Instant::now() < fim, "a 1.ª resposta não chegou");
        m.quadro();
        std::thread::sleep(Duration::from_millis(2));
    }
    tecla(&mut m, id, Key::End, Modifiers::default(), None, "");
    letra(&mut m, id, ' ');
    let fim = Instant::now() + Duration::from_secs(10);
    while m.docs().assist_text().is_none() || !m.docs().problems_of(id).is_empty() {
        assert!(
            Instant::now() < fim,
            "o assistente parou e o sublinhado ficou: {:?} {:?}",
            m.docs().assist_reason(),
            m.docs().problems_of(id)
        );
        m.quadro();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(
        m.docs().assist_text().as_deref(),
        Some(
            ph2d_panel_code::assist_line(ph2d_panel_code::Assist::Stopped)
                .0
                .as_str()
        )
    );
}

/// A resposta que chega com teclas AINDA por enviar (escritas depois do último quadro) cai no sítio:
/// adoptada, desloca-se por elas. Cada volta: um quadro envia a versão `v`, o servidor responde-lhe
/// fora do quadro, uma tecla entra antes do quadro seguinte, e é nesse quadro que a resposta a `v`
/// é adoptada.
#[test]
fn uma_resposta_com_teclas_por_enviar_cai_no_sitio() {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", "local s = \"\" pirnt(1)\n")],
        Plano {
            regra: Some(acusa_pirnt),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    tecla(&mut m, id, Key::Home, ctrl(), None, "");
    for _ in 0.."local s = \"".len() {
        tecla(&mut m, id, Key::Right, Modifiers::default(), None, "");
    }
    let uri = uri_of(id.0, "a.luau");
    for (i, c) in "é🙂ab".chars().cycle().take(12).enumerate() {
        letra(&mut m, id, c);
        m.quadro();
        let v = m.docs().assist_diagnostics(id).0.expect("aberto");
        let fim = Instant::now() + Duration::from_secs(10);
        while visto.lock().unwrap().textos.get(&uri).map(|t| t.0) != Some(v as i64) {
            assert!(
                Instant::now() < fim,
                "volta {i}: o servidor não recebeu a versão {v}"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        std::thread::sleep(Duration::from_millis(20));
        letra(&mut m, id, c);
        m.quadro();
        let t = m.docs().text_of(id).unwrap();
        let ps = m.docs().problems_of(id);
        assert_eq!(ps.len(), 1, "volta {i}: {ps:#?}");
        let c = ps[0].culprit.clone().unwrap();
        assert_eq!(t.get(c.clone()), Some("pirnt"), "volta {i}: {c:?} em {t:?}");
    }
}
