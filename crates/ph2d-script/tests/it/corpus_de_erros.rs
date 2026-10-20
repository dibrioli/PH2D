//! 📏 **A via do MOTOR do corpus de erros** (CodeEditor Onda 2, I0c) — corre à mão:
//! `PH2D_CORPUS_DIR=… cargo test -p ph2d-script --test it corpus_de_erros -- --ignored --nocapture`
//! (o `docs/CodeEditor/ferramentas/corpus_erros/corre.sh` faz tudo por ordem).
//!
//! Cada mutante entra na VM da casa como um script da cena (`ScriptHost` + `ProjectScripts`): o
//! topo corre ao `scene_sync`, depois `init` + `update` em [`QUADROS`] tiques e um `on_signal`. Sai
//! uma linha JSON por mutante com TODOS os `ScriptDiagnostic` (fase, tipo, linha, mensagem).
//! O doc com as tabelas: `docs/CodeEditor/ferramentas/medicoes_onda2/I0c_erros_frequentes.md`.

use std::fmt::Write as _;
use std::path::PathBuf;

use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_script::{
    DiagnosticKind, LuauScript, ProjectScripts, ScriptDiagnostic, ScriptHost, ScriptInfo,
};

/// Tiques depois do nascimento — o 1.º corre o `init`; os seguintes apanham o que só falha quando o
/// estado mudou (um `self.t` que cresce, um sinal no topo da onda).
const QUADROS: usize = 5;

fn kind(d: &ScriptDiagnostic) -> String {
    match &d.kind {
        DiagnosticKind::Syntax => "Syntax".into(),
        DiagnosticKind::Runtime => "Runtime".into(),
        DiagnosticKind::Budget => "Budget".into(),
        DiagnosticKind::Decl(e) => {
            let s = format!("{e:?}");
            format!(
                "Decl:{}",
                s.split(['(', ' ', '{']).next().unwrap_or_default()
            )
        }
    }
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Todos os diagnósticos de UM mutante, pela ordem em que aparecem.
fn corre(chunk: &str, source: &str) -> Vec<(String, ScriptDiagnostic)> {
    let mut sim = SimWorld::new();
    let id = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert(chunk, source);
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, StableId(1), LuauScript::library(id)))
        .id();
    let mut h = ScriptHost::new().expect("vm");
    let mut out = Vec::new();
    h.scene_sync(sim.world_mut());
    match h.scene_info(id) {
        Some(ScriptInfo::Broken(d)) => {
            out.push(("topo".to_owned(), d.clone()));
            return out;
        }
        Some(ScriptInfo::Missing(m)) => panic!("o script sumiu da biblioteca: {m}"),
        _ => {}
    }
    for q in 0..QUADROS {
        h.scene_sync(sim.world_mut());
        let r = h.scene_tick(&mut sim, 1.0 / 60.0);
        out.extend(r.failed.into_iter().map(|(_, d)| (format!("tique{q}"), d)));
    }
    if h.scene_failure(e).is_none() {
        let r = h.scene_hear(&mut sim, &["ping"]);
        out.extend(r.failed.into_iter().map(|(_, d)| ("sinal".to_owned(), d)));
    }
    out
}

#[test]
#[ignore = "medição: corre à mão (PH2D_CORPUS_DIR; ver corre.sh)"]
fn corpus_de_erros() {
    let dir = PathBuf::from(
        std::env::var("PH2D_CORPUS_DIR")
            .expect("PH2D_CORPUS_DIR = a pasta do corpus_erros.py gera"),
    );
    let saida = std::env::var("PH2D_CORPUS_SAIDA").expect("PH2D_CORPUS_SAIDA = o runtime.jsonl");
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir.join("mutantes"))
        .expect("mutantes/")
        .map(|e| e.expect("entrada").path())
        .filter(|p| p.extension().is_some_and(|x| x == "luau"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "nenhum mutante — correu o `gera`?");
    let mut out = String::new();
    let mut com_diag = 0usize;
    let t0 = std::time::Instant::now();
    for p in &files {
        let stem = p.file_stem().expect("nome").to_string_lossy().into_owned();
        // O nome do chunk é o do ficheiro original (`bob.luau`), como na cena.
        let chunk = format!("{}.luau", stem.split("__").next().expect("id"));
        let src = std::fs::read_to_string(p).expect("ler mutante");
        let diags = corre(&chunk, &src);
        com_diag += usize::from(!diags.is_empty());
        let _ = write!(out, "{{\"mutant\": {}, \"diags\": [", json_str(&stem));
        for (i, (fase, d)) in diags.iter().enumerate() {
            let line = d.line.map_or_else(|| "null".to_owned(), |l| l.to_string());
            let _ = write!(
                out,
                "{}{{\"fase\": {}, \"kind\": {}, \"line\": {line}, \"message\": {}}}",
                if i == 0 { "" } else { ", " },
                json_str(fase),
                json_str(&kind(d)),
                json_str(&d.message)
            );
        }
        out.push_str("]}\n");
    }
    std::fs::write(&saida, out).expect("escrever runtime.jsonl");
    println!(
        "corpus_de_erros: {} mutantes, {com_diag} com diagnóstico, {:.1} s → {saida}",
        files.len(),
        t0.elapsed().as_secs_f64()
    );
}
