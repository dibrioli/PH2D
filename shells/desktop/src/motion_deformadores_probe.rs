//! ⭐⭐ **A AUDITORIA DO GRUPO DOS TRANSFORMES & DEFORMADORES** (ciclo 3, passo 2 — doc 106).
//!
//! Mesma régua dos ciclos 1 e 2: a auditoria começa por **medir o que existe**, nunca por uma
//! lista do que eu acho que falta.
//!
//! ⚠️ **A coluna do dispositivo lê-se do `register_gpu_kernel`, NUNCA do `lowerings`** — o
//! ciclo 2 pagou essa: o `lowerings` é o que o nó declara saber baixar **sozinho**, e imprimir
//! essa coluna fabrica uma tabela de dívida com oito kernels que já existem.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture audit_the_deformer_group
//! ```

use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Edge, NodeId};

/// Os treze do ciclo 3 (doc 103 §5).
pub(crate) const GRUPO: [&str; 13] = [
    "motion.move",
    "motion.rotate",
    "motion.scale",
    "motion.transform",
    "motion.mirror",
    "motion.look_at",
    "motion.bend",
    "motion.twist",
    "motion.spherize",
    "motion.four_point_warp",
    "motion.bezier_warp",
    "motion.kaleidoscope",
    "motion.spline_wrap",
];

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_deformer_group() {
    let base = MotionState::new();
    eprintln!("\n  nó                     | params | no cartão | device | portas | efeito");
    eprintln!("  -----------------------|--------|-----------|--------|--------|--------");
    for nome in GRUPO {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.to_string());
        let tid = m.doc.graph.node(id).expect("no'").type_id();
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            base.registry.resolve(tid).map(|op| op.manifest())
        };
        let Some(man) = man else {
            eprintln!("  {nome:<23} | (nao registado)");
            continue;
        };
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::render_loop::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        let no_cartao = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map_or(0, |v| v.params.len());
        let device = {
            use ph2d_nodegraph::gpu::KernelResolver;
            m.registry.gpu_kernel(tid).is_some()
        };
        eprintln!(
            "  {nome:<23} | {:>6} | {no_cartao:>9} | {:^6} | {}->{:<4} | {:?}",
            man.params.len(),
            if device { "sim" } else { "NAO" },
            man.inputs.len(),
            man.outputs.len(),
            man.effect,
        );
    }
    eprintln!();
}

/// **OS PARAMS DE CADA NÓ DO GRUPO, um a um** — o que a auditoria compara contra as
/// referências. ⚠️ Sem esta lista, «falta X» é um palpite.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_each_deformer_offers
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_deformer_offers() {
    let m = MotionState::new();
    for nome in GRUPO {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            let Some(op) = m.registry.resolve(tid) else {
                continue;
            };
            op.manifest()
        };
        eprintln!("\n  === {nome} ===");
        eprintln!(
            "  portas: [{}] -> [{}]",
            man.inputs
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", "),
            man.outputs
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
        let hints = m.registry.param_ui(tid).unwrap_or(&[]);
        for spec in man.params {
            let h = hints.iter().find(|h| h.param == spec.name);
            let w = h.map_or("(sem hint)".to_string(), |h| match h.widget {
                ph2d_node_registry::ParamWidget::Enum { labels } => {
                    format!("Enum[{}]", labels.join("|"))
                }
                outro => format!("{outro:?}"),
            });
            eprintln!(
                "    {:<18} default {:>9.3}  {:<16}  {}",
                spec.name,
                spec.default,
                h.map_or("—".to_string(), |h| format!("{}..{}", h.min, h.max)),
                w
            );
        }
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// ⭐⭐⭐ O PREÇO DE CADA NÓ DO GRUPO — e, sobretudo, o preço de ele NÃO chegar ao dispositivo.
// ---------------------------------------------------------------------------------------------

fn wire(m: &mut MotionState, from: NodeId, fp: u16, to: NodeId, tp: u16) {
    m.doc
        .graph
        .connect(Edge {
            from: (from, fp),
            to: (to, tp),
            delayed: false,
        })
        .expect("liga");
}

/// Monta `motion.grid(lado × lado) [→ <nó>] → motion.output` e devolve o sink.
fn build_com(m: &mut MotionState, lado: f32, no: Option<&str>, aceso: bool) -> NodeId {
    let grid = m.doc.graph.add_node("motion.grid");
    m.doc.graph.set_param(grid, "rows", lado);
    m.doc.graph.set_param(grid, "cols", lado);
    let out = m.doc.graph.add_node("motion.output");
    match no {
        Some(nome) => {
            let d = m.doc.graph.add_node(nome.to_string());
            if aceso {
                acordar(m, d, nome);
            }
            wire(m, grid, 0, d, 0);
            wire(m, d, 0, out, 0);
        }
        None => wire(m, grid, 0, out, 0),
    }
    out
}

/// ⛔⛔ **TIRA O NÓ DO PONTO NEUTRO ANTES DE O CRONOMETRAR.**
///
/// ⚠️ **A primeira redacção desta sonda media cada nó nos DEFAULTS dele**, e o
/// `motion.bezier_warp` denunciou-a assim que chegou ao dispositivo: ele leu **`0,34×` o custo
/// da grelha sozinha** — *mais rápido que não estar lá* —, porque com os 24 offsets a zero o
/// `eval` toma o atalho da identidade e devolve o stream clonado. A tabela dizia que o patch de
/// Coons é barato e o que ela cronometrava era um `clone`.
///
/// ⚠️ **Não é um caso especial dele:** metade deste grupo nasce na identidade (o `motion.move`
/// em `(0,0)`, o `rotate` a `0°`, os dois warps com os cantos parados). *Um corpus no ponto
/// neutro de um knob não testa esse knob*, e uma tabela de PREÇO medida no neutro mede o preço
/// de não fazer nada.
///
/// ⚠️ **Os knobs movidos são DERIVADOS dos hints, nunca uma tabela à mão** — uma lista escrita
/// aqui envelheceria em silêncio a cada param novo, e um param renomeado deixaria de ser
/// movido sem nada acusar.
///
/// ⛔ **E só os SLIDERS se movem** — mexer num `Enum` ou num `Toggle` mudaria o MODO do nó, que
/// é outra medição.
///
/// ⚠️⚠️ **Isto NÃO é suficiente para deixar a coluna do dispositivo em paz, e eu escrevi aqui
/// que era.** A cláusula `applicable` do `motion.look_at` lê `target_x`/`target_y`, que são
/// **sliders**: a primeira corrida com o despertar acendeu-os e o nó apareceu 🔴 — a minha
/// perturbação com cara de regressão. ⇒ a coluna do dispositivo passou a ser lida num grafo
/// SEPARADO, nos defaults (ver [`cook_com`]), e o desacordo entre os dois virou uma leitura
/// própria (🟡). *Não há subconjunto de knobs seguro: a única forma de não perturbar uma
/// medição é não a fazer no mesmo grafo.*
fn acordar(m: &mut MotionState, no: NodeId, nome: &str) {
    let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
    for h in m.registry.param_ui(tid).unwrap_or(&[]) {
        if !matches!(h.widget, ph2d_node_registry::ParamWidget::Slider) {
            continue;
        }
        // Um quarto do caminho até ao topo da faixa — longe do default sem ser o extremo,
        // que é onde uma cerca degenerada moraria.
        let alvo = h.min + (h.max - h.min) * 0.25;
        let v = if (alvo - h.min).abs() < f32::EPSILON {
            h.min + (h.max - h.min) * 0.5
        } else {
            alvo
        };
        m.doc.graph.set_param(no, h.param, v);
    }
}

/// A mediana de 3 cozimentos **FRIOS** — um `MotionState` novo por corrida, senão o memo do cook
/// responde à segunda e a sonda mede a tabela de hash. Devolve `(ms, n, no_device)`.
/// O que uma linha da tabela diz.
struct Medida {
    /// A mediana de 3 cozimentos FRIOS, em ms — com o nó **acordado** (ver [`acordar`]).
    ms: f64,
    /// Quantos objectos saíram.
    n: usize,
    /// ⛔⛔ **O planeador reivindica a cadeia com o nó nos DEFAULTS dele?** É esta a pergunta
    /// do produto: é o que o artista recebe ao largar o nó.
    gpu_neutro: bool,
    /// E com o nó acordado. ⚠️ **Quando as duas diferem, a residência do nó DEPENDE de um
    /// knob** — não é ruído da sonda, é uma propriedade do nó que vale a pena ler.
    gpu_aceso: bool,
}

/// ⛔⛔ **DUAS PERGUNTAS, DOIS GRAFOS — e a primeira redacção respondia-as com um só.**
///
/// O relógio quer o nó a **trabalhar** ([`acordar`]); a coluna do dispositivo quer o nó como o
/// artista o **recebe**. Misturá-las custou uma leitura errada na primeira corrida: acordar o
/// `motion.look_at` pôs `target_x`/`target_y` fora de zero, que é exactamente o que a cláusula
/// `applicable` dele lê, e a coluna virou 🔴 — *a MINHA perturbação, lida como uma regressão do
/// produto*. ⚠️ E o comentário que eu tinha escrito ao lado do `acordar` dizia que só `Enum` e
/// `Toggle` alimentam um `applicable`: **falso**, e a tabela desmentiu-o na corrida seguinte.
fn cook_com(lado: f32, no: Option<&str>) -> Medida {
    let mut ms: Vec<f64> = Vec::new();
    let (mut n, mut gpu_aceso) = (0usize, false);
    for _ in 0..3 {
        let mut m = MotionState::new();
        let sink = build_com(&mut m, lado, no, true);
        gpu_aceso =
            ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink).is_fully_gpu();
        let t = std::time::Instant::now();
        let out = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, sink, 0.0)
            .expect("coze");
        ms.push(t.elapsed().as_secs_f64() * 1000.0);
        n = out[0].as_stream().count();
    }
    ms.sort_by(f64::total_cmp);
    let gpu_neutro = {
        let mut m = MotionState::new();
        let sink = build_com(&mut m, lado, no, false);
        ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink).is_fully_gpu()
    };
    Medida {
        ms: ms[1],
        n,
        gpu_neutro,
        gpu_aceso,
    }
}

/// ⭐⭐⭐ **O PREÇO DO GRUPO, e o 🔴 é o achado — nunca a razão** (doc 103 §5.1).
///
/// Um nó que cai na CPU **no meio de uma cadeia** não custa o que ele custa: custa o
/// dispositivo inteiro ([doc 98](../../docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md)
/// mediu `50,9×`). Por isso a coluna que decide é *«a cadeia inteira é reivindicada?»*, e o
/// relógio está ao lado só para dizer quanto.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture measure_the_deformer_group
/// ```
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn measure_the_deformer_group() {
    let lado: f32 = std::env::var("PH2D_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(320.0);
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let so_grade = cook_com(lado, None);
    let nu = so_grade.ms;
    eprintln!(
        "\n  grade {lado}×{lado} = {} objectos · so' a grade: {nu:.2} ms {}",
        so_grade.n,
        if so_grade.gpu_neutro { "🟢" } else { "🔴" }
    );
    eprintln!(
        "\n  ⚠️ o nó é medido ACORDADO (cada slider a 1/4 da faixa) — nos defaults, metade
     deste grupo é a identidade e o relógio media um `clone`.\n"
    );
    eprintln!("  nó                     | objectos  | cozimento  | vs. so' a grade | no device?");
    eprintln!(
        "  -----------------------|-----------|------------|-----------------|------------------"
    );
    for nome in GRUPO {
        let d = cook_com(lado, Some(nome));
        let device = match (d.gpu_neutro, d.gpu_aceso) {
            (true, true) => "🟢 sim".to_string(),
            (false, false) => "🔴 NAO".to_string(),
            // A residência depende de um knob — a coluna diz qual dos dois lados é qual.
            (true, false) => "🟡 so' nos defaults".to_string(),
            (false, true) => "🟡 so' fora dos defaults".to_string(),
        };
        eprintln!(
            "  {nome:<23} | {:>9} | {:>7.2} ms | {:>14.2}× | {device}",
            d.n,
            d.ms,
            d.ms / nu.max(1e-9),
        );
    }
    eprintln!(
        "\n  🟢 = o planeador reivindica a cadeia inteira · 🔴 = ela cai na CPU
  🟡 = a residência DEPENDE de um knob (o `applicable` do kernel lê-o)
  (um quadro de 60 fps tem 16,67 ms)\n"
    );
}

/// ⭐⭐ **AS ROWS QUE O CARTÃO DE FACTO PINTA, com o RÓTULO que aparece na tela.**
///
/// ⚠️ **Um passo de smoke que manda clicar numa linha AFIRMA que ela está na lista** — e a casa
/// já pagou por escrever um passo impossível ([memória]). Esta sonda é o instrumento que
/// verifica a afirmação antes de a mensagem sair: ela imprime o que o `stamp_card_params`
/// produz, que é literalmente o que o pintor desenha.
///
/// [memória]: ../../../project-memory/feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list.md
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_the_card_shows
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_card_shows() {
    for nome in GRUPO {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.to_string());
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::render_loop::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        let view = snap.nodes.iter().find(|v| v.id == id.0);
        let rows: Vec<String> = view
            .map(|v| v.params.iter().map(|c| c.hint.label.to_string()).collect())
            .unwrap_or_default();
        // ⚠️ **As SECÇÕES fazem parte do que o cartão MOSTRA, e esta sonda não as lia.** O
        // plano do ciclo 3 acusou o `motion.bezier_warp` de pintar `In X · In Y · Out X ·
        // Out Y` **quatro vezes** sem dizer de que aresta — uma acusação construída sobre
        // esta lista de rótulos, que é metade da resposta. O cartão dobra as rows em
        // [`CardSection`], e um rótulo repetido debaixo de um cabeçalho que o nomeia **não é
        // ambíguo**. *Uma sonda que lê metade da superfície fabrica dívida.*
        let secs: Vec<String> = view
            .map(|v| {
                v.sections
                    .iter()
                    .map(|s| {
                        format!(
                            "{}@{}{}",
                            s.title,
                            s.at,
                            if s.open { "" } else { " (fechada)" }
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        eprintln!("  {nome:<23} | {}", rows.join(" · "));
        if !secs.is_empty() {
            eprintln!("  {:<23} > secções: {}", "", secs.join(" · "));
        }
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// ⛔⛔ ONDE UMA CENA ESTÁ — e se o artista consegue lá CHEGAR.
// ---------------------------------------------------------------------------------------------

/// ⛔⛔ **O ALCANCE DA CÂMARA, lido do código.** `ph2d_render::Camera2d` abre em
/// `height_world = 10` e o zoom-out **pára** em `ZOOM_MAX_HEIGHT_WORLD = 100` — metade disso é a
/// distância máxima a que um objecto ainda pode ser trazido ao ecrã, e ⚠️ **só com o zoom no
/// batente**. A barra fica em `50`, que é esse batente, e não num número escolhido.
const ALCANCE_DA_CAMARA: f32 = ph2d_render::Camera2d::ZOOM_MAX_HEIGHT_WORLD * 0.5;

/// A cena publica legenda? Construir é barato; **cozinhar não é** (a cena `=1` tem 2 M
/// elementos), e é por isso que a pergunta vem antes.
fn scene_has_legend(level: u32) -> bool {
    let mut m = MotionState::new();
    let nivel = level.to_string();
    let _ = crate::motion_state::demo_router::build_level(Some(&nivel), &mut m.doc, &m.registry);
    !crate::motion_demo_legend::captions().is_empty()
}

/// A caixa que os objectos de uma cena ocupam, em unidades de mundo.
fn scene_bounds(level: u32) -> Option<([f32; 2], [f32; 2], usize)> {
    let mut m = MotionState::new();
    let nivel = level.to_string();
    let sinks =
        crate::motion_state::demo_router::build_level(Some(&nivel), &mut m.doc, &m.registry);
    let sink = *sinks.first()?;
    crate::render_loop::motion_shape_gen::publish(&mut m, 0.0);
    let out = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, 0.0)
        .ok()?;
    let stream = out.first()?.as_stream();
    let ph2d_nodegraph::attr::Column::Vec2(p) = stream.get("P")? else {
        return None;
    };
    if p.is_empty() {
        return None;
    }
    let mut lo = [f32::INFINITY; 2];
    let mut hi = [f32::NEG_INFINITY; 2];
    for q in p {
        for k in 0..2 {
            lo[k] = lo[k].min(q[k]);
            hi[k] = hi[k].max(q[k]);
        }
    }
    Some((lo, hi, p.len()))
}

/// ⭐⭐⭐ **A CÂMARA NÃO CHEGA A TODA A PARTE, E O NÚMERO É DO CÓDIGO.**
///
/// `ph2d_render::Camera2d` abre em `height_world = 10` e o zoom-out **pára em 100**
/// (`ZOOM_MAX_HEIGHT_WORLD`). ⇒ uma cena cujos objectos vivam para lá disso não está «fora do
/// ecrã»: está **fora do alcance**, e o artista não tem gesto que a encontre.
///
/// ⚠️ **Isto não é uma regra para TODA cena.** As cenas de perf (`=12` é uma grelha de
/// `700 × 700` a `1` unidade de passo) existem para carregar o dispositivo, não para serem
/// lidas — e a esse tamanho o zoom máximo mostra um sétimo delas, de propósito.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture where_each_demo_scene_lives
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn where_each_demo_scene_lives() {
    eprintln!(
        "\n  cena | objectos | x                  | y                  | legenda | cabe em 100?"
    );
    eprintln!(
        "  -----|----------|--------------------|--------------------|---------|-------------"
    );
    for level in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let Some((lo, hi, n)) = scene_bounds(level) else {
            continue;
        };
        let legenda = !crate::motion_demo_legend::captions().is_empty();
        let alcance = hi[0]
            .abs()
            .max(lo[0].abs())
            .max(hi[1].abs())
            .max(lo[1].abs());
        let cabe = alcance <= 50.0;
        if legenda || !cabe {
            eprintln!(
                "  {level:>4} | {n:>8} | {:>8.1}..{:<8.1} | {:>8.1}..{:<8.1} | {:^7} | {}",
                lo[0],
                hi[0],
                lo[1],
                hi[1],
                if legenda { "sim" } else { "-" },
                if cabe { "sim" } else { "NAO" },
            );
        }
    }
    eprintln!();
}

/// ⭐⭐⭐ **UMA CENA QUE PÕE LEGENDA TEM DE CABER NO ALCANCE DA CÂMARA** (ciclo 3 — doc 106).
///
/// ⛔⛔ **O report que a fez existir:** *«a simulação funciona nos nós mas não aparece no canvas»*
/// (Enio, 2026-09-08). Os cartões cozinhavam `180 000` objectos, cada pré-visualização de nó
/// tinha pontos, e o canvas estava vazio — porque a cena `=111` nascia com o pano entre `300` e
/// `600` unidades de mundo, e **a câmara não chega lá**: ela abre em `10` e o zoom-out pára em
/// `100`. *Não estava fora do ecrã; estava fora do ALCANCE, e nenhum gesto a encontrava.*
///
/// ⚠️⚠️ **E o HUD não ajudava — dizia `0 inst`, o que quase mandou a investigação para o sítio
/// errado.** Aquele contador conta **entidades `RenderInstance` do ECS**, e uma cena de motion
/// residente no dispositivo não cria nenhuma: ele lê `0` em toda cena de motion que funciona.
///
/// ## A régua é DERIVADA, não uma lista
///
/// ⚠️ Isto **não** vale para toda cena, e a distinção não é de gosto: as cenas de perf
/// (`=1`, `=2`, `=6`, `=7`, `=12`..`=16`) vivem a `180`–`800` unidades **de propósito** — elas
/// existem para carregar o dispositivo, e ninguém as lê. O que separa umas das outras é a
/// **LEGENDA**: uma cena que pousa uma ficha no canvas está, por construção, a dizer *«alguém vai
/// ler isto»*.
///
/// Medido no dia em que este gate nasceu: **29 cenas com legenda, todas dentro de `±11`**
/// unidades; **9 sem legenda, todas entre `180` e `800`**; e a `=111` — esta — era a única com
/// legenda do lado errado. *A partição não foi escolhida: ela estava lá.*
///
/// A tabela inteira sai de `where_each_demo_scene_lives`.
#[test]
fn a_scene_with_a_legend_fits_inside_what_the_camera_can_reach() {
    let mut lidas = 0usize;
    let mut fora: Vec<String> = Vec::new();
    for level in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        if !scene_has_legend(level) {
            continue;
        }
        lidas += 1;
        let Some((lo, hi, _)) = scene_bounds(level) else {
            continue;
        };
        let alcance = hi[0]
            .abs()
            .max(lo[0].abs())
            .max(hi[1].abs())
            .max(lo[1].abs());
        if alcance > ALCANCE_DA_CAMARA {
            fora.push(format!(
                "cena {level}: x {:.1}..{:.1} · y {:.1}..{:.1} (alcance {alcance:.1} >                  {ALCANCE_DA_CAMARA})",
                lo[0], hi[0], lo[1], hi[1]
            ));
        }
    }
    // Controlo positivo: um censo que casasse zero passaria vaziamente, e é exactamente o que
    // aconteceria se o `build_level` deixasse de publicar legendas.
    assert!(
        lidas >= 25,
        "so' {lidas} cena(s) com legenda foram vistas — a varredura foi as cegas"
    );
    assert!(
        fora.is_empty(),
        "{} cena(s) com legenda vivem fora do alcance da camara — o artista NAO tem gesto que \
         as encontre:\n  {}",
        fora.len(),
        fora.join("\n  ")
    );
}
