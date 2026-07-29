//! **Arch-gate: ninguém drena o arrasto de `CurvePoint` com uma pergunta que não pode dizer NÃO.**
//!
//! ## O bug que este arquivo existe para impedir (medido 2026-07-29)
//!
//! O `curve_point_drag` do `WidgetStore` é um canal **GLOBAL** — um `Option` só, com muitos donos
//! possíveis (as curvas de ajuste do Painter, as duas barras de rampa dele, o editor de falloff, o
//! gizmo de dab, o dial de tilt do Wet Paint, as curvas do motion-params, o trilho de gradiente do
//! FX vetorial). E o `HeroScreen::apply_event` pergunta a **TODO** painel do registry, visível ou
//! não, parando no primeiro `Consumed`.
//!
//! O painel de camadas drenava para **qualquer** `ValueChanged` — tomava o stash e *só então*
//! procurava a camada dona — e devolvia `Consumed` mesmo sem achar nenhuma. Preço medido: os punhos
//! do trilho de rampa do **painel de vetor** não se moviam. O painel do FX pintava os punhos, o
//! dispatch calculava a posição nova, e o gesto era engolido por um painel que nem estava na tela —
//! sem erro, sem warning, com os gates isolados dos dois painéis **verdes**.
//!
//! ## A cura estrutural, e o que sobra para este gate
//!
//! A porta sem pergunta **não existe mais**: `take_curve_point_drag_if(|parent| …)` é a única, e o
//! compilador recusa quem não responde. O que o compilador **não** vê é uma resposta que é sempre
//! `true` — um `|_| true` reabre o buraco exatamente como antes, com a assinatura nova.
//!
//! ⚠️ **O escopo é preciso de propósito:** ele recusa a TAUTOLOGIA LITERAL na chamada. Um predicado
//! nomeado cujo corpo é `true` passaria — e a defesa contra esse é a de COMPORTAMENTO
//! (`ph2d-panel-painter-layers/tests/seam_curve_drag_ownership.rs`, que prova que o arrasto de outro
//! painel sobrevive à travessia). Um gate que tentasse ler o corpo de closures nomeadas seria fuzzy,
//! e um gate fuzzy é o que se silencia em vez de acreditar.
//!
//! Dep-free (std only).

use std::path::{Path, PathBuf};

/// O número de sítios de drenagem em painéis quando este gate foi escrito (2026-07-29): 5 no
/// Painter, 2 no motion-params (produção + o seu próprio teste), 1 no vetor, 1 no roteador de
/// `ValueChanged` do Painter (que passa um predicado nomeado).
///
/// ⚠️ É **controle POSITIVO**, não um teto: ele existe porque um scanner que deixasse de casar com
/// a chamada (renomeada, reformatada, movida) passaria **verde sobre zero leitura**, que é a falha
/// clássica de um gate de busca negativa. Cresce livremente; encolher a zero é o que ele pega.
const MIN_CALL_SITES: usize = 6;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Todos os `.rs` sob `src/` de cada crate `ph2d-panel-*`.
fn panel_sources() -> Vec<(PathBuf, String)> {
    let crates = repo_root().join("crates");
    let mut out = Vec::new();
    let entries = std::fs::read_dir(&crates).expect("cannot read crates/");
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("ph2d-panel-") {
            continue;
        }
        walk(&entry.path().join("src"), &mut out);
    }
    out
}

fn walk(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs")
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            out.push((path, text));
        }
    }
}

/// **Nenhum painel responde "é meu" com uma tautologia.**
///
/// Mutação que tem de sangrar: trocar qualquer `|p| p == <id>` por `|_| true` (é literalmente o
/// código que shipava antes da cura, vestindo a assinatura nova).
#[test]
fn no_panel_drains_a_curve_drag_with_a_predicate_that_cannot_say_no() {
    const CALL: &str = "take_curve_point_drag_if(";
    // As formas em que uma tautologia é ESCRITA na chamada. `|_|` cobre o caso comum; os nomeados
    // (`|_p|`, `|_parent|`) são a variação que alguém escreve para silenciar o `unused`.
    const TAUTOLOGIES: &[&str] = &["|_| true", "|_p| true", "|_parent| true", "|_id| true"];

    let mut sites = 0usize;
    let mut offenders: Vec<String> = Vec::new();
    for (path, text) in panel_sources() {
        let mut from = 0usize;
        while let Some(hit) = text[from..].find(CALL) {
            let start = from + hit + CALL.len();
            sites += 1;
            // A janela é o argumento: até o fecha-parênteses da própria chamada, com folga para o
            // `rustfmt` a ter quebrado em linhas.
            let window: String = text[start..].chars().take(120).collect();
            if TAUTOLOGIES.iter().any(|t| window.contains(t)) {
                offenders.push(format!("{}", path.display()));
            }
            from = start;
        }
    }

    assert!(
        offenders.is_empty(),
        "estes paineis drenam o arrasto de `CurvePoint` com um predicado que nunca diz NAO: \
         {offenders:?}\n\
         O stash e um canal GLOBAL e o `take` e irreversivel: um predicado tautologico rouba o \
         gesto de outro painel (o dono nao tem o que drenar) e reintroduz, com a assinatura nova, \
         exatamente o defeito medido em 2026-07-29 — os punhos do trilho de rampa do painel de \
         vetor nao se moviam.\n\
         Fix: responda com o id que o SEU painel registrou como `parent` do `CurvePoint`."
    );
    assert!(
        sites >= MIN_CALL_SITES,
        "o scanner achou {sites} sitios de drenagem (esperado >= {MIN_CALL_SITES}): ele parou de \
         casar com a chamada, e um gate de busca NEGATIVA sem controle positivo fica verde sobre \
         zero leitura"
    );
}
