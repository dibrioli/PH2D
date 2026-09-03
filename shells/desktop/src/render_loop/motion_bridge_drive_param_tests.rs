//! **CONDUZIR UM PARAM POR FIO** — os gates e as sondas do report do Enio de 2026-09-01
//! (*«lfo nem oscillator conseguem atuar sobre Angle de Rotate»*), cortados do
//! `motion_bridge_rewire_duplicator_tests.rs` no teto de LOC do HR-18 (600).
//!
//! ⚠️ **O corte é por RESPONSABILIDADE:** o irmão responde *«o que atravessa um nó de duas
//! entradas?»* (as colunas, a porta em que o fio cai) e este *«que fio pode CONDUZIR um
//! param, e o que acontece quando ele não pode?»* — um param não tem porta (o
//! `NodeManifest.inputs` está congelado, ADR-0039), então esta é uma maquinaria inteira à
//! parte, com a sua própria recusa.
//!
//! Mecanismo e tabelas: [doc 98 §4.4c](../../../../docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md).

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::Edge;

/// ⛔⛔⛔ **«LFO NEM OSCILLATOR CONSEGUEM ATUAR SOBRE ANGLE DE ROTATE»** — report do Enio,
/// 2026-09-01.
///
/// A sonda liga cada candidato ao param `angle` do `motion.rotate` por `Graph::drive_param`
/// (a MESMA porta que o menu do gesto chama) e mede o `rot` da corrente ao longo do tempo:
/// um condutor que age faz o ângulo VARIAR entre instantes.
///
/// `cargo test -p ph2d-host-desktop --release --bins -- --ignored --nocapture what_can_drive_the_rotate_angle`
#[test]
#[ignore = "sonda de reproducao, nao um gate"]
fn what_can_drive_the_rotate_angle() {
    use ph2d_nodegraph::cook::Cook;
    for (fonte, mexer) in [
        ("value.lfo", false),
        ("value.lfo", true),
        ("motion.oscillator", true),
        ("value.number", true),
    ] {
        let mut motion = MotionState::new();
        let g = &mut motion.doc.graph;
        let grid = g.add_node("motion.grid".to_string());
        g.set_param(grid, "rows", 2.0);
        g.set_param(grid, "cols", 2.0);
        let rot = g.add_node("motion.rotate".to_string());
        let out = g.add_node("motion.output".to_string());
        g.connect(Edge {
            from: (grid, 0),
            to: (rot, 0),
            delayed: false,
        })
        .expect("grid -> rotate");
        g.connect(Edge {
            from: (rot, 0),
            to: (out, 0),
            delayed: false,
        })
        .expect("rotate -> out");
        let s = g.add_node(fonte.to_string());
        // ⚠️ **A 1.a corrida do `value.lfo` NAO mexe em nada** -- e' o que o artista tem
        // depois de largar o fio. A 2.a poe a amplitude longe do neutro.
        if mexer {
            for (k, v) in [("amplitude", 90.0f32), ("value", 45.0)] {
                g.set_param(s, k, v);
            }
        }
        let ligou = g.drive_param(rot, "angle", (s, 0));
        if let Err(e) = &ligou {
            eprintln!("  {fonte:<20} │ drive_param RECUSOU: {e:?}");
            continue;
        }
        let mut cook = Cook::new();
        let mut vistos: Vec<f32> = Vec::new();
        for t in 0..8u64 {
            let ph = t as f64 * 0.1;
            if cook
                .cook(&motion.doc.graph, &motion.registry, out, ph)
                .is_err()
            {
                eprintln!("  {fonte:<20} │ a cadeia NAO COZINHA");
                break;
            }
            if let Some(v) = cook.peek(out) {
                if let Some(ph2d_nodegraph::attr::Column::Scalar(r)) = v[0].as_stream().get("rot") {
                    vistos.push(r.first().copied().unwrap_or(0.0));
                }
            }
        }
        let lo = vistos.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = vistos.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let veredito = if !lo.is_finite() {
            "sem leitura".to_string()
        } else if (hi - lo).abs() > 1e-6 {
            format!("VARIA {lo:.2}..{hi:.2}")
        } else if lo.abs() > 1e-6 {
            format!("constante em {lo:.2} (age, mas nao anima)")
        } else {
            "⛔ INERTE (rot fica em 0)".to_string()
        };
        let como = if mexer {
            "amplitude = 90"
        } else {
            "TAL COMO SAI DA CAIXA"
        };
        eprintln!("  {fonte:<20} │ {como:<22} │ {veredito}");
    }
}

/// ⭐⭐⭐ **UM FIO QUE NÃO PODE CONDUZIR É RECUSADO, EM VOZ ALTA** — o gate do report do Enio
/// de 2026-09-01 (*«lfo nem oscillator conseguem atuar sobre Angle de Rotate»*).
///
/// ⛔⛔ **As duas metades mordem.** O `motion.oscillator` emite `Instances/Vec2` — uma
/// corrente por-elemento — e o `Graph::drive_param` **aceitava-o**, porque ele não tem
/// registry e só sabe verificar existência e ciclo. O param ficava com um fio VISÍVEL e o
/// valor nunca mudava (`rot` em `0` para sempre, medido em `what_can_drive_the_rotate_angle`).
/// E o `value.lfo`, que emite `Instances/Scalar`, **tem** de continuar a passar — uma recusa
/// larga demais tiraria o único condutor que funciona.
#[test]
fn only_a_value_output_may_drive_a_param() {
    use crate::render_loop::motion_bridge::subgraph::source_can_drive;
    let mut motion = MotionState::new();
    let g = &mut motion.doc.graph;
    let lfo = g.add_node("value.lfo".to_string());
    let osc = g.add_node("motion.oscillator".to_string());
    let grid = g.add_node("motion.grid".to_string());
    assert!(
        source_can_drive(&motion, (lfo, 0)),
        "o `value.lfo` emite Instances/Scalar -- e' O condutor, e recusa-lo mataria a feature"
    );
    assert!(
        !source_can_drive(&motion, (osc, 0)),
        "o `motion.oscillator` emite Instances/Vec2: ligar o fio era aceite e nao fazia NADA"
    );
    assert!(
        !source_can_drive(&motion, (grid, 0)),
        "uma fonte de instancias tambem nao conduz"
    );
    assert!(
        !source_can_drive(&motion, (lfo, 7)),
        "uma porta que nao existe recusa -- o default e' o lado seguro"
    );
}

/// ⛔⛔⛔ **QUANTA INFORMAÇÃO OS DOIS CANAIS DO SOCKET CARREGAM** — report do Enio, 2026-09-02:
/// *«uma deficiência do nosso sistema de nós [comparado] ao MiniCavalryV2 é que não temos
/// sistema de outputs evidentes nos nós»*.
///
/// Um socket desta casa codifica o tipo em **dois** canais: a **COR** é o
/// [`ph2d_nodegraph::port::Domain`] e a **FORMA** é o [`ph2d_nodegraph::port::Dim`] (círculo =
/// escalar · losango = vector). A pergunta que decide a avaliação é *quantos valores distintos
/// cada canal de facto toma no catálogo* — um canal com um valor só é tinta gasta.
///
/// `cargo test -p ph2d-host-desktop --release --bins -- --ignored --nocapture what_the_socket_encoding_carries`
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn what_the_socket_encoding_carries() {
    use ph2d_nodegraph::port::{Dim, Domain};
    use std::collections::BTreeMap;
    let motion = MotionState::new();
    let mut por_dominio: BTreeMap<String, usize> = BTreeMap::new();
    let mut por_forma: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut pares: BTreeMap<String, usize> = BTreeMap::new();
    let mut saidas = 0usize;
    let mut com_varias_saidas = 0usize;
    let mut tipos = 0usize;
    for m in motion.registry.manifests() {
        tipos += 1;
        if m.outputs.len() > 1 {
            com_varias_saidas += 1;
        }
        for p in m.outputs {
            saidas += 1;
            let dom = format!("{:?}", p.ty.domain);
            let forma = match p.ty.dim {
                Dim::Scalar => "circulo (escalar)",
                _ => "losango (vector)",
            };
            *por_dominio.entry(dom.clone()).or_default() += 1;
            *por_forma.entry(forma).or_default() += 1;
            *pares.entry(format!("{dom}/{:?}", p.ty.dim)).or_default() += 1;
        }
    }
    eprintln!(
        "  {tipos} tipos de no' · {saidas} portas de saida · {com_varias_saidas} tipos com MAIS DE UMA saida"
    );
    eprintln!("  --- canal COR (o dominio) ---");
    for (k, v) in &por_dominio {
        eprintln!(
            "    {k:<12} │ {v:>4} portas ({:>5.1}%)",
            *v as f64 * 100.0 / saidas as f64
        );
    }
    eprintln!("  --- canal FORMA (a dimensao) ---");
    for (k, v) in &por_forma {
        eprintln!(
            "    {k:<20} │ {v:>4} portas ({:>5.1}%)",
            *v as f64 * 100.0 / saidas as f64
        );
    }
    eprintln!("  --- os pares que de facto existem ---");
    for (k, v) in &pares {
        eprintln!("    {k:<24} │ {v:>4}");
    }
    eprintln!("  (um canal que toma UM valor so' e' tinta gasta: nao distingue nada)");
    let _ = Domain::Instances;
}

/// ⛔⛔⛔ **O CENSO DOS CANAIS VISUAIS DO GRAFO** — ordem do Enio, 2026-09-02: *«MiniCavalry é
/// nosso MVP referência, mais belo e fácil de usar que nosso produto. Faça um estudo sério»*.
///
/// O `visual-tokens.js` dele codifica em **cinco** canais (e o cabeçalho declara que os tirou
/// de um doc NOSSO, «Doc PH2D §6»): cor de cabeçalho por CATEGORIA · silhueta por PAPEL · cor
/// de pino por TIPO · forma de pino por CARDINALIDADE · **espessura de fio** por tipo.
///
/// ⚠️ **Ter o vocabulário não é usá-lo.** Esta sonda conta quantos valores distintos cada canal
/// de facto toma no nosso catálogo — um canal com um valor só não distingue nada, esteja ou não
/// declarado. É a mesma régua do `what_the_socket_encoding_carries`.
///
/// `cargo test -p ph2d-host-desktop --release --bins -- --ignored --nocapture what_our_visual_channels_carry`
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn what_our_visual_channels_carry() {
    use std::collections::BTreeMap;
    let motion = MotionState::new();
    let mut sil: BTreeMap<String, usize> = BTreeMap::new();
    let mut cat: BTreeMap<String, usize> = BTreeMap::new();
    let (mut com_ui, mut sem_ui) = (0usize, 0usize);
    for m in motion.registry.manifests() {
        match motion.registry.ui_manifest(m.id) {
            Some(u) => {
                com_ui += 1;
                *sil.entry(format!("{:?}", u.silhouette)).or_default() += 1;
                *cat.entry(format!("{:?}", u.category)).or_default() += 1;
            }
            None => sem_ui += 1,
        }
    }
    eprintln!("  {com_ui} tipos com metadados de UI · {sem_ui} SEM (caem no default)");
    eprintln!("  --- canal SILHUETA (o papel do no' no grafo) — 7 valores possiveis ---");
    for (k, v) in &sil {
        eprintln!(
            "    {k:<16} │ {v:>4} ({:>5.1}%)",
            *v as f64 * 100.0 / com_ui as f64
        );
    }
    eprintln!("  --- canal CATEGORIA (a familia) ---");
    for (k, v) in &cat {
        eprintln!(
            "    {k:<20} │ {v:>4} ({:>5.1}%)",
            *v as f64 * 100.0 / com_ui as f64
        );
    }
    eprintln!("  (um canal que toma UM valor so' nao distingue nada, esteja ou nao declarado)");
}

/// ⭐⭐⭐ **QUANTO DO «LÊ / ESCREVE» NÓS CONSEGUIMOS DERIVAR** — o estudo do MiniCavalry
/// (Enio, 2026-09-02).
///
/// O cartão dele mostra, por nó, os atributos que **lê** (cinza) e que **escreve** (dourado)
/// — `src/editor/chips.js`, a partir de `def.reads_attrs`/`def.writes_attrs`, **declarados à
/// mão em cada nó**. É exactamente a informação cuja ausência custou os reports de 01/09 (o
/// duplicator a deitar fora `id`/`vel`/`age`/`life`).
///
/// ⭐ **A nossa pode ser DERIVADA e não declarada:** o `GpuKernel::bindings` diz, por coluna, o
/// [`ph2d_nodegraph::gpu::ColumnAccess`] — e é a MESMA lista de que o gerador de código vive,
/// logo não pode divergir do que o nó faz. Uma lista à mão pode.
///
/// Esta sonda mede a **cobertura**: para quantos dos 134 tipos a derivação existe hoje.
///
/// `cargo test -p ph2d-host-desktop --release --bins -- --ignored --nocapture how_much_reads_writes_we_can_derive`
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn how_much_reads_writes_we_can_derive() {
    use ph2d_nodegraph::gpu::KernelResolver;
    let motion = MotionState::new();
    let (mut com_kernel, mut com_bindings, mut sem, mut com_variante) = (0usize, 0, 0, 0);
    let mut total_cols = 0usize;
    let mut mudos: Vec<&str> = Vec::new();
    for m in motion.registry.manifests() {
        match motion.registry.gpu_kernel(m.id) {
            Some(k) => {
                com_kernel += 1;
                if k.variant_by_param.is_some() {
                    com_variante += 1;
                }
                if k.bindings.is_empty() {
                    mudos.push(m.name);
                } else {
                    com_bindings += 1;
                    total_cols += k.bindings.len();
                }
            }
            None => {
                sem += 1;
                mudos.push(m.name);
            }
        }
    }
    let n = com_kernel + sem;
    eprintln!("  {n} tipos de no'");
    eprintln!(
        "    com kernel de device        │ {com_kernel:>4} ({:>5.1}%)",
        com_kernel as f64 * 100.0 / n as f64
    );
    eprintln!(
        "    ⭐ com BINDINGS derivaveis  │ {com_bindings:>4} ({:>5.1}%) · {total_cols} colunas declaradas",
        com_bindings as f64 * 100.0 / n as f64
    );
    eprintln!("    ⚠️  cuja forma depende de PARAM (variant) │ {com_variante:>4}");
    eprintln!(
        "    ⛔ SEM nenhuma declaracao    │ {:>4} ({:>5.1}%)",
        mudos.len(),
        mudos.len() as f64 * 100.0 / n as f64
    );
    eprintln!("  os mudos: {}", mudos.join(" "));
}
