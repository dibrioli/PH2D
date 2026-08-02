//! **QUEM segura os planos** — a pergunta que reescreveu o plano da porta única de escrita.
//!
//! O irmão [`super::measure_commit_cost`] responde a outra metade (*de que é feito o CUSTO*); a linha
//! de corte é POSSE contra PREÇO.
//!
//! A decomposição anterior (doc 28 §5.13) disse que o pen-up custa 38,9 ms a 4096² **porque o
//! `commit_stroke_height` forka os três planos de relevo**, e fechou a conta somando `Vec::clone` dos
//! três: 0,40 + 9,94 + 18,13 = 28,47 contra 32,8 medidos. As sondas deste arquivo derrubaram os dois
//! lados dessa frase.
//!
//! ## 1. O dono extra é PERMANENTE, e é o histórico
//!
//! Em repouso, com dois traços commitados e nenhum gesto aberto, **os quatro planos canvas-shaped têm
//! DOIS donos**; limpar o histórico leva a contagem a **um**. Ou seja o `cursor` que a U1 instalou como
//! base de todo delta é um segundo dono permanente — e a hipótese fácil (*"o dono extra é o snapshot de
//! pen-down"*) descrevia só metade. Consequência de projeto: um journal que substituísse apenas o
//! `paint.stroke_undo` deixaria a contagem em 2 e **não mudaria um milissegundo**.
//!
//! ### 1b. DENTRO do gesto são TRÊS, e cada um tem nome (doc 28 §5.20)
//!
//! `tool` (irredutível) · `cursor` · `paint.stroke_undo` — este último porque `snapshot_model` clona os
//! `BTreeMap`, o que bumpa um `Arc` por plano. O **quarto** que aparece logo após o PRIMEIRO traço é a
//! entrada dele: um traço que **cria** os planos de relevo não tem lado `before` a diferenciar, então o
//! delta grava `Whole { before, after }` e segura o plano vivo. ⚠️ **`make_mut` copia com qualquer
//! coisa acima de um** ⇒ remover UMA das três não compra milissegundo nenhum: o S3 é tudo-ou-nada, e o
//! alvo são as três.
//!
//! ### 1c. E a entrada de um traço GRANDE também segura — para sempre (doc 28 §5.67)
//!
//! Esta seção dizia *"do segundo traço em diante a entrada é `Patch` e não segura nada"*. A frase foi
//! medida com a [`stroke`] de 200 px — **0,1% da área a 4096²** — e é falsa para o traço que o artista
//! de fato dá: a janela declarada é o **BBOX**, um traço diagonal tem bbox de ~90% do plano, e os dois
//! construtores de delta mandam para `Whole` toda janela ≥ 50% (*"o delta guarda DOIS lados, então só
//! compensa abaixo de metade"*).
//!
//! ⚠️ **São DUAS portas com o MESMO limiar** — [`StoredPlane::from_window`](crate::undo_delta) e
//! `from_journal` — e foi isso que atrasou a bisseção: uma ablação que mirou só a primeira curou o
//! canvas e deixou o relevo intacto, o que eu li como *"o relevo entra por outro lugar"*. O braço
//! `Whole` do journal faz `after: Arc::clone(live)` ⇒ **o `after` É o segundo dono**; o `before` dele já
//! é material próprio (`par_clone` + patch) e não segura nada.
//!
//! (As seções sobre o CUSTO — o fork, o pen-up, o commit — mudaram-se para o irmão junto com as sondas
//! que as mediram; o que segue é a atribuição de posse, que é o assunto deste arquivo.)
//!
//! ## 2. O fork custa um TERÇO do que a aritmética dizia
//!
//! `Vec::clone` é um memcpy **serial**; o produto usa [`super::plane_fork::fork_par`], que é paralelo.
//! Medido pela porta do produto a 4096²: **0,29 + 3,16 + 5,79 = 9,25 ms**, não 28,47. A soma que
//! "fechava" fechava por **coincidência** — e uma atribuição que casa com o total por acidente é pior
//! que nenhuma, porque encerra a investigação no lugar errado.
//!
//! ## 3. O pen-up é o COMMIT DE UNDO, e o `commit_stroke_height` é footprint-bound
//!
//! Ablacionando pela entrada (`paint.stroke_undo = None` faz o `close_stroke` pular o commit
//! estrutural), a 4096² com impasto: **40,20 ms completo contra 3,49 sem o commit** — e o que sobra é
//! **plano na tela** (3,35 / 3,43 / 3,49 a 1024²/2048²/4096²), que é a forma correta para trabalho
//! limitado pela pegada. **91% do pen-up era o histórico**, cujo `PlaneDeltas::split` varre
//! `diff_window` sobre os quatro planos: ~256 MB de comparação por traço.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;

pub(super) fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um canvas com pincel de impasto — o modo cujo pen-up paga os três planos de relevo.
pub(super) fn armed(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 40.0,
        hardness: 0.5,
        falloff: Falloff::Sphere,
        strength: 1.0,
        color: [0.2, 0.3, 0.8],
        impasto: true,
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t
}

pub(super) fn stroke(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
    for k in 1..=6u8 {
        let x = 60.0 + f32::from(k) * 30.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([260.0, y], PointerPhase::Up));
}

fn plane_owners<T>(
    m: &std::collections::BTreeMap<crate::layers::LayerId, std::sync::Arc<Vec<T>>>,
    active: Option<crate::layers::LayerId>,
) -> usize {
    active
        .and_then(|a| m.get(&a).map(std::sync::Arc::strong_count))
        .unwrap_or(0)
}

fn owners(t: &PainterTool) -> (usize, usize, usize, usize) {
    let a = t.layers.active();
    (
        std::sync::Arc::strong_count(&t.canvas_rgba),
        plane_owners(&t.heights, a),
        plane_owners(&t.covers, a),
        plane_owners(&t.mats, a),
    )
}

/// **A contagem de donos em REGIME, e de quem é cada um.**
///
/// Roda dois traços (o primeiro instala o histórico) e conta os donos de cada plano canvas-shaped no
/// repouso entre eles — depois derruba os suspeitos, um a um, para atribuir.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn who_holds_the_planes_when_a_stroke_begins() {
    let mut t = armed(1024);
    stroke(&mut t, 200.0);
    stroke(&mut t, 300.0);

    let (c, h, cv, m) = owners(&t);
    eprintln!("\n[donos] REGIME (2 tracos commitados, nenhum gesto aberto)");
    eprintln!("  canvas_rgba {c} · heights {h} · covers {cv} · mats {m}");
    eprintln!("  (1 = so o tool ⇒ a proxima escrita e IN PLACE; ≥2 ⇒ ela COPIA o plano inteiro)");

    // Suspeito 1: o snapshot de pen-down. Em repouso ele nem existe.
    eprintln!("  stroke_undo vivo? {}", t.paint.stroke_undo.is_some());

    // Suspeito 2: o histórico (o `cursor` da U1 + as entradas).
    t.undo.clear();
    let (c2, h2, cv2, m2) = owners(&t);
    eprintln!("[donos] depois de LIMPAR o historico");
    eprintln!("  canvas_rgba {c2} · heights {h2} · covers {cv2} · mats {m2}");

    // E dentro do gesto: quantos donos o primeiro dab encontra?
    //
    // ⚠️ A contagem depende de QUANTOS traços vieram antes, e não por acaso: o PRIMEIRO traço numa
    // camada CRIA os planos de relevo, então o `split` não tem janela a diferenciar (o lado `before`
    // é vazio) e a entrada guarda `Whole { before, after }` — um `Arc` do plano VIVO, para sempre.
    // Do segundo em diante a entrada é `Patch` e não segura plano nenhum.
    for warm in [1usize, 2, 4] {
        let mut t2 = armed(1024);
        for k in 0..warm {
            stroke(&mut t2, 200.0 + (k as f32) * 40.0);
        }
        t2.on_canvas_pointer(cp([60.0, 600.0], PointerPhase::Down));
        let (c, h, cv, m) = owners(&t2);
        eprintln!(
            "[donos] apos {warm} traco(s), dentro do gesto: canvas {c} · heights {h} · covers {cv} · mats {m}"
        );
    }

    let mut t2 = armed(1024);
    stroke(&mut t2, 200.0);
    t2.on_canvas_pointer(cp([60.0, 300.0], PointerPhase::Down));
    let (c3, h3, cv3, m3) = owners(&t2);
    eprintln!("[donos] DENTRO do gesto (logo apos o pen-down)");
    eprintln!("  canvas_rgba {c3} · heights {h3} · covers {cv3} · mats {m3}");

    // …e de QUEM são, um a um. A ordem é a das duas referências que o S3 removeria (§7 do doc 28):
    // primeiro o snapshot de pen-down, depois o `cursor` do histórico. O que sobrar depois das duas
    // é o que decide se a wave é viável — `make_mut` copia com qualquer coisa acima de um.
    t2.paint.stroke_undo = None;
    let (c4, h4, cv4, m4) = owners(&t2);
    eprintln!(
        "  - sem o snapshot de pen-down:  canvas {c4} · heights {h4} · covers {cv4} · mats {m4}"
    );
    t2.undo.clear();
    let (c5, h5, cv5, m5) = owners(&t2);
    eprintln!(
        "  - …e sem o historico:          canvas {c5} · heights {h5} · covers {cv5} · mats {m5}"
    );
    eprintln!("  (sobra 1 = o TOOL, irredutivel ⇒ o S3 chega la, mas so alcancando as tres)\n");
}

/// **O QUE O JOURNAL RETÉM NUM TRAÇO REAL — o número que decide se ele pode sair do `cfg(debug)`.**
///
/// A troca do S3 substitui, em cada gesto, um **fork do plano inteiro** (que só existe porque há um
/// segundo dono) por uma **captura da região**. A §5.25 mediu a região contra o fork no primitivo —
/// 15–73× mais barata —, mas o primitivo não diz quanto um TRAÇO retém, e é o traço que paga.
///
/// ⚠️ **A comparação certa não é contra o documento, é contra o que o modelo de HOJE já retém.** O fork
/// aloca um plano inteiro por gesto e o histórico guarda um delta por passo; se a captura for da mesma
/// ordem do delta, ela é gratuita em memória — ela substitui trabalho, não o acrescenta.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn what_the_journal_retains_for_one_real_stroke() {
    println!(
        "\n{:<8} {:>10} {:>12} {:>12} {:>10}",
        "tela", "doc (MB)", "journal MB", "delta MB", "j/doc"
    );
    for side in [1024u32, 2048, 4096] {
        let mut t = armed(side);
        stroke(&mut t, 40.0); // o 1º instala o histórico (e o cursor)
        let before = t.undo.retained_bytes();

        // O passo seguinte, medido com o journal ancorado nele.
        // ⚠️ **Lido ANTES do pen-up**, e a 1ª versão desta sonda não era: o pen-up commita, o commit
        // move o cursor e `set_cursor` **zera o journal** — a tabela saía com 0,00 MB em toda tela, que
        // é o journal *depois* de ele ter cumprido o seu papel, não o pico que ele retém.
        t.begin_undo_step();
        t.on_canvas_pointer(cp([60.0, 80.0], PointerPhase::Down));
        for k in 1..=6u8 {
            t.on_canvas_pointer(cp([60.0 + f32::from(k) * 30.0, 80.0], PointerPhase::Move));
        }
        let journal = t.undo.write_state.journal_heap_bytes();
        t.on_canvas_pointer(cp([260.0, 80.0], PointerPhase::Up));

        let n = (side as usize) * (side as usize);
        // Os quatro planos canvas-shaped de uma camada tocada: rgba(4) + heights(4) + covers(1) + mats(7).
        let doc = (n * 16) as f64 / 1e6;
        let delta = (t.undo.retained_bytes() - before) as f64 / 1e6;
        println!(
            "{side:<8} {doc:>10.1} {:>12.2} {delta:>12.2} {:>9.1}%",
            journal as f64 / 1e6,
            100.0 * journal as f64 / (doc * 1e6)
        );
    }
    println!();
}

/// **O PRÊMIO DO DEGRAU 4 É CONDICIONAL À ROTA DO JOURNAL, e isto é FORMA — não relógio**
/// (doc 28 §5.60).
///
/// A elisão do relevo no `before` do pen-down parecia remover um dono. Ela **não remove: troca de
/// lugar.** Com o mapa `before` vazio o `split` cai em [`StoredEntry::OnlyAfter`], que guarda um `Arc`
/// **forte do plano VIVO** — antes o fork copiava porque o `before` segurava o plano ANTIGO, depois
/// copiaria porque a ENTRADA segura o de agora. O único delta que não segura `Arc` nenhum é o `Patch`
/// da rota do journal (ele extrai uma JANELA em `Vec`), e por isso a rota deixa de ser uma otimização
/// e passa a ser **pré-requisito** da elisão.
///
/// ⚠️ **Roda nos DOIS perfis, e sem cronômetro.** Este parágrafo dizia *"roda em DEBUG de propósito —
/// o journal é `cfg(debug_assertions)`, em release esta pergunta não existe ainda"*, e isso é **FALSO
/// desde o degrau 4**: quem é `cfg(any(test, debug_assertions))` é o journal do CANVAS; o do RELEVO
/// **shipa** (veja o cabeçalho de `undo_delta_journal.rs`). O único `#[cfg(test)]` no caminho é o
/// INCREMENTO do contador — a testemunha, nunca a rota. E a resposta é uma CONTAGEM, que uma máquina
/// carregada não sabe distorcer (a §5.49: nenhum relógio desta máquina significa nada com load > ~5).
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture --test-threads=1"]
fn the_journal_route_is_what_makes_the_elision_worth_anything() {
    let mut t = armed(1024);
    stroke(&mut t, 200.0);
    let fired_first = crate::undo_planes::RELIEF_FROM_JOURNAL.with(std::cell::Cell::get);
    stroke(&mut t, 260.0);
    let fired = crate::undo_planes::RELIEF_FROM_JOURNAL.with(std::cell::Cell::get);
    let (c, h, cv, m) = owners(&t);
    println!(
        "\n[degrau 4] a rota do journal disparou {fired} vez(es) em 2 tracos (1a: {fired_first})"
    );
    println!(
        "[degrau 4] em REPOUSO, apos 2 tracos: canvas {c} · heights {h} · covers {cv} · mats {m}"
    );

    // A elisão simulada: o `before` do pen-down larga o relevo, e conta-se o que sobra.
    t.on_canvas_pointer(cp([60.0, 320.0], PointerPhase::Down));
    for k in 1..=6u8 {
        t.on_canvas_pointer(cp([60.0 + f32::from(k) * 30.0, 320.0], PointerPhase::Move));
    }
    let (_, h_in, _, _) = owners(&t);
    if let Some(s) = t.paint.stroke_undo.as_mut() {
        s.heights.clear();
        s.covers.clear();
        s.mats.clear();
    }
    let (_, h_elided, _, _) = owners(&t);
    t.on_canvas_pointer(cp([260.0, 320.0], PointerPhase::Up));
    let (_, h_after, _, _) = owners(&t);
    println!(
        "[degrau 4] heights: {h_in} donos no gesto · {h_elided} com o `before` elidido · \
         {h_after} depois do commit"
    );
    println!(
        "[degrau 4] (1 = so o tool. Se o numero DEPOIS do commit nao cair, a entrada assumiu a \
         posse — `OnlyAfter` — e a elisao nao comprou nada.)\n"
    );
}

/// **E UM TRAÇO GRANDE DEIXA A ENTRADA SEGURANDO OS PLANOS — para sempre.**
///
/// É esta sonda que derrubou a §1b e escreveu a §1c do cabeçalho: a janela declarada é o **BBOX**, e um
/// traço diagonal tem bbox de ~90% do plano (medido: 78,5% a 2048², 88,9% a 4096²) ⇒ os dois
/// construtores de delta caem em `Whole`, que segura um `Arc` de cada endpoint ⇒ **segundo dono
/// permanente** ⇒ o `fork_par` do gesto seguinte copia os quatro planos canvas-sized.
///
/// ⚠️ **O traço curto NÃO é redundante — é o CONTROLE, na MESMA corrida e no MESMO canvas:** ele cai em
/// `Patch` pela mesma regra, então *a entrada grande segura* e *esta máquina/este canvas segura* deixam
/// de ser indistinguíveis. Sem ele a tabela não decide nada.
///
/// ⚠️ **O contador `(relevo pelo JOURNAL: Nx)` diz por qual rota o commit passou** — e a leitura dele
/// é a mesma nos dois perfis, porque **o journal do relevo SHIPA** (só o do canvas é `cfg`; veja o
/// cabeçalho de `undo_delta_journal.rs`).
///
/// ⚠️ **Este bloco afirmava o oposto até 2026-08-02** (*"esta sonda não vê o caminho do produto: o
/// journal do relevo é `cfg(any(test, debug_assertions))` e `cargo test --release` liga `cfg(test)`"*)
/// — a segunda metade é verdadeira e **irrelevante aqui**, porque o `cfg(test)` cobre o INCREMENTO do
/// contador e não a rota. Eu li dois doc-comments obsoletos do produto, não grepei o atributo, e
/// publiquei o veredito errado (doc 28 §5.67). *Um `cfg` se confere no atributo, nunca na prosa.*
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture --test-threads=1"]
fn who_holds_the_planes_after_a_canvas_wide_stroke() {
    fn diagonal(t: &mut PainterTool, side: f32, y0: f32) {
        let span = side - 200.0;
        t.on_canvas_pointer(cp([60.0, y0], PointerPhase::Down));
        for k in 1..=12u8 {
            let f = f32::from(k) / 12.0;
            t.on_canvas_pointer(cp([60.0 + span * f, y0 + span * f], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([60.0 + span, y0 + span], PointerPhase::Up));
    }

    for side in [2048u32, 4096] {
        eprintln!("\n[donos] {side}x{side}, em REPOUSO depois de N tracos");
        for (name, wide) in [("curto (200 px, o CONTROLE)", false), ("diagonal", true)] {
            let mut t = armed(side);
            let mut row = String::new();
            for k in 0..3u8 {
                let y = 200.0 + f32::from(k) * 9.0;
                if wide {
                    diagonal(&mut t, side as f32, y);
                } else {
                    stroke(&mut t, y);
                }
                let (c, h, cv, m) = owners(&t);
                row.push_str(&format!("  apos {}: {c}/{h}/{cv}/{m}", k + 1));
                if let Some(v) = t.undo.probe_top_variants() {
                    let j = crate::undo_planes::RELIEF_FROM_JOURNAL.with(std::cell::Cell::get);
                    eprintln!(
                        "      [{}] entrada {}: {v}  (relevo pelo JOURNAL: {j}x)",
                        if wide { "diag " } else { "curto" },
                        k + 1
                    );
                }
            }
            eprintln!("  {name:26}{row}");
            // Duas ablações, e a ordem separa os dois donos que um `clear()` remove de uma vez.
            t.undo.probe_drop_entries();
            let (c, h, cv, m) = owners(&t);
            eprintln!("  {:26}  so' as ENTRADAS fora: {c}/{h}/{cv}/{m}", "");
            t.undo.clear();
            let (c, h, cv, m) = owners(&t);
            eprintln!("  {:26}  …e o cursor tambem:   {c}/{h}/{cv}/{m}", "");
        }
        eprintln!("  (canvas/heights/covers/mats — 1 = so o tool ⇒ a proxima escrita e IN PLACE)");
    }
}

/// **A JANELA DECLARADA CONTRA A REGIÃO QUE DE FATO MUDA** — o número que decide onde a cura mora.
///
/// A entrada acima mostra a POSSE; esta pergunta *por quê*. `from_window` manda para `Whole` toda
/// janela ≥ 50% do plano, e a janela declarada é o **bbox** do que o passo escreveu. Mas o delta não
/// precisa de onde se escreveu — precisa de **onde o conteúdo DIFERE** (escrever o mesmo valor de volta
/// não é uma mudança a desfazer), e é isso que o `diff_window` deriva quando não há janela declarada.
///
/// Se as duas coincidirem, a cura tem de estar no motor de delta (o contrato do `Whole`); se a mudança
/// for uma fração, a janela declarada é que está larga demais.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture --test-threads=1"]
fn what_a_stroke_declares_against_what_it_changes() {
    fn changed_frac<T: PartialEq>(before: &[T], after: &[T], w: usize) -> (f64, usize, usize) {
        let (mut r0, mut r1, mut c0, mut c1) = (usize::MAX, 0usize, usize::MAX, 0usize);
        for (i, (a, b)) in before.iter().zip(after).enumerate() {
            if a != b {
                let (r, c) = (i / w, i % w);
                r0 = r0.min(r);
                r1 = r1.max(r);
                c0 = c0.min(c);
                c1 = c1.max(c);
            }
        }
        if r0 == usize::MAX {
            return (0.0, 0, 0);
        }
        let (rows, cols) = (r1 - r0 + 1, c1 - c0 + 1);
        let n = before.len() as f64;
        ((rows * cols) as f64 / n * 100.0, rows, cols)
    }

    for side in [2048u32, 4096] {
        let mut t = armed(side);
        let span = side as f32 - 200.0;
        // Um traço de aquecimento: o PRIMEIRO cria os planos e não tem lado `before`.
        t.on_canvas_pointer(cp([60.0, 200.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([260.0, 200.0], PointerPhase::Up));
        let a = t.layers.active().unwrap();
        let (h0, c0, m0) = (
            t.heights[&a].clone(),
            t.covers[&a].clone(),
            t.mats[&a].clone(),
        );
        let rgba0 = t.canvas_rgba.clone();

        t.on_canvas_pointer(cp([60.0, 300.0], PointerPhase::Down));
        for k in 1..=12u8 {
            let f = f32::from(k) / 12.0;
            t.on_canvas_pointer(cp([60.0 + span * f, 300.0 + span * f], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([60.0 + span, 300.0 + span], PointerPhase::Up));

        let w = side as usize;
        let (fh, rh, ch) = changed_frac(&h0, &t.heights[&a], w);
        let (fc, rc, cc) = changed_frac(&c0, &t.covers[&a], w);
        let (fm, rm, cm) = changed_frac(&m0, &t.mats[&a], w);
        let (fr, rr, cr) = changed_frac(&rgba0, &t.canvas_rgba, w * 4);
        eprintln!("\n[janela] {side}x{side}, traco DIAGONAL de canto a canto");
        eprintln!("  bbox da MUDANCA, em % do plano (>=50% ⇒ a entrada vira `Whole`)");
        eprintln!("  heights {fh:6.2}%  ({rh}x{ch})");
        eprintln!("  covers  {fc:6.2}%  ({rc}x{cc})");
        eprintln!("  mats    {fm:6.2}%  ({rm}x{cm})");
        eprintln!("  canvas  {fr:6.2}%  ({rr}x{cr} elementos)");
    }
}
