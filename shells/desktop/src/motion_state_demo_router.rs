//! **Qual cena de demonstração o `PH2D_GPU_COOK_DEMO` nomeia** — a tabela, e só ela.
//!
//! Ela saiu do `MotionState::new` por teto de LOC, e o corte é por RESPONSABILIDADE: o
//! construtor responde *como um `MotionState` nasce* (registry, documento, bomba de cook,
//! anéis de sonda) e esta função responde *que documento o ambiente pediu*. Uma cresce a
//! cada wave que acrescenta uma cena; o outro, quase nunca.
//!
//! ⚠️ **O roteador é uma lista de braços e o PRIMEIRO vence** — dois braços com o mesmo
//! número deixam o segundo inalcançável **em silêncio**, que foi como a cena dos tokens da
//! `line/Vector` sumiu em 2026-08-02. O gate `no_two_smoke_scenes_claim_the_same_level`
//! existe por isso.

use super::*;

/// Os sinks da cena que o ambiente pediu — vazio quando ele não pediu nada, que é a TELA
/// VAZIA com que o editor abre.
pub(super) fn demo_sinks(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    // GPU/M5 ready-to-smoke documents (opt-in; the regular boot document is
    // untouched): `PH2D_GPU_COOK_DEMO=1` = the F1.1 1250×1600 (2.000.000)
    // chain that is 100% GPU under `PH2D_GPU_COOK=1`; `=2` = the F1.2 HYBRID
    // chain whose first node (an oscillator on the uncovered Rotation channel)
    // has no kernel, so the CPU cooks the prefix and the GPU runs the suffix;
    // `=3` = the Fase 3 SIMULATION (490.000 particles in a force loop whose
    // state ping-pongs on the device, ADR-0127); `=4` = the SEA (490.000
    // raining onto a travelling wave); `=5` = the emitter FOUNTAIN (ADR-0130,
    // the id-gather: particles born/killed across a sliding window, paired by
    // arithmetic — the scene the fixed-grid demos could not be).
    match std::env::var("PH2D_GPU_COOK_DEMO").as_deref() {
        Ok("1") => build_gpu_demo_document(doc, registry).unwrap_or_default(),
        Ok("2") => build_gpu_hybrid_demo_document(doc, registry).unwrap_or_default(),
        Ok("3") => build_gpu_sim_demo_document(doc, registry).unwrap_or_default(),
        Ok("4") => build_gpu_sea_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0130: the emitter FOUNTAIN — the id-gather (particles born/killed
        // across a sliding window, paired by arithmetic; fatia 5 tunes it live).
        Ok("5") => build_gpu_emitter_demo_document(doc, registry).unwrap_or_default(),
        // The PANEL scene: 262.144 instances through both domains, so every
        // kind of card reading is on screen at once — the smoke for the GPU
        // path becoming the default.
        Ok("6") => build_gpu_panel_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0140: the MURMURATION — the interacting sim (each agent reads its
        // neighbours) that the spatial grid lifts from a few-hundred-agent toy
        // to a swarm on the device (524.288, sized so it never stutters when the
        // flock gathers; the ceiling is millions).
        Ok("7") => build_gpu_boids_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0140 Fase 5: the breathing PACKING — the second grid client,
        // and the first ITERATED kernel (the grid is rebuilt per sweep).
        Ok("8") => build_gpu_collide_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0140 Fase 5: the spread SWEEP — the diagnostic scene. A slow,
        // linear triangle sweep of `spread` so the GPU meter shows a smooth
        // mountain (the cost is a function of the packing, no reach-boundary
        // step) rather than a staircase.
        Ok("9") => build_gpu_sweep_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0135: the SIM-ZONE family — a fixed-population snow globe, the
        // state-loop container (`sim.zone` + `sim.step` + `sim.collide`) 100% on
        // the device. The boot snow's physics minus birth/death (which are
        // count-changing and still cook on the pump).
        Ok("10") => build_gpu_zone_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0139: the breathing HONEYCOMB — the first engine ALGORITHM
        // (Lloyd relaxation via jump flooding), and the cap that fell with
        // it: 20.000 points where the CPU-era node capped at 600.
        Ok("11") => build_gpu_voronoi_demo_document(doc, registry).unwrap_or_default(),
        // The DEFORMER family: the whole-stream reduction channel. Two
        // deformers CHAINED, so the second one's fold must measure what the
        // first one produced (see the scene's own note).
        Ok("12") => build_gpu_deform_demo_document(doc, registry).unwrap_or_default(),
        // The `Sum` half of the deformer channel: the centroid lens (two
        // reductions on one node).
        Ok("13") => build_gpu_spherize_demo_document(doc, registry).unwrap_or_default(),
        // The widest reduction consumer: the bounding-box corner-pin (four
        // reductions, the first use of Min).
        Ok("14") => build_gpu_four_point_warp_demo_document(doc, registry).unwrap_or_default(),
        // The count-changing deformer: the mandala fan-out (StreamOp
        // SourceRows, the first kernel to READ its template).
        Ok("15") => build_gpu_kaleidoscope_demo_document(doc, registry).unwrap_or_default(),
        // O ORGANISMO: the whole reduction channel end to end — count-changing
        // fan (SourceRead) then the four count-preserving deformers, each
        // folding its reduction over the live stream the previous one produced.
        Ok("16") => build_gpu_deform_organism_demo_document(doc, registry).unwrap_or_default(),
        // The FIELD family: `field.index_range` writes the `falloff` mask keyed
        // by ORDINAL (not position), coloured by a Solid tint — the middle band
        // of 262k rows glowing red, a mask no spatial falloff can draw.
        Ok("17") => build_gpu_field_index_range_demo_document(doc, registry).unwrap_or_default(),
        // The spatial sibling: `field.box` masks by POSITION — a wide, thin box
        // is the razor-horizontal band (flat by y) that the ordinal index field
        // cannot draw. Blue, to read against `=17`'s red ordinal band.
        Ok("18") => build_gpu_field_box_demo_document(doc, registry).unwrap_or_default(),
        // Composition: two fields (ordinal band + spatial vertical band) fanned
        // off one grid and unioned by `field.combine` into a red cross — the
        // whole fan-out on the device (the field family's thesis).
        Ok("19") => build_gpu_field_combine_demo_document(doc, registry).unwrap_or_default(),
        // The ANGULAR field: `field.radial_sweep` — a 30° wedge repeated 6× into a
        // six-pointed blue star (a fan / radar). The shape a rectangle cannot make,
        // the HR-5 pseudo-angle sector on the device, and the 2nd field the canvas
        // gizmo drives (D9).
        Ok("20") => build_gpu_field_radial_sweep_demo_document(doc, registry).unwrap_or_default(),
        // The REMAPPER: `field.box` paints a soft ramp, `field.remap` Quantizes it
        // into three topographic bands — the D1 factoring (every field defers its
        // remap here), the C4D Remapping tab as a downstream node.
        Ok("21") => build_gpu_field_remap_demo_document(doc, registry).unwrap_or_default(),
        // The CURVE contour (A1): the same box ramp, remapped through a tent curve
        // authored in the text param — a blue RING no ramp or Quantize can make. The
        // kernel declines mode 4, so the remap cooks on the CPU (A1-gpu bakes the LUT).
        Ok("22") => build_gpu_field_curve_demo_document(doc, registry).unwrap_or_default(),
        // O PORTÃO ESPACIAL (doc 89, folha 12): um metrônomo, um losango, e só quem está
        // DENTRO dele escuta o beat — `pulse.level` (o pulso vira número) + o canal
        // Falloff do `value.attribute` (o peso do campo vira legível), os dois elos que
        // faltavam para eventos e campos se encontrarem.
        Ok("23") => build_gpu_pulse_gate_demo_document(doc, registry).unwrap_or_default(),
        // AS CINCO FONTES (doc 89, folha 12): a `=23` mostra um campo decidindo QUEM escuta
        // um evento; esta mostra um evento decidindo O QUE passa a existir. `rate = 0`, então
        // o pulso é o ÚNICO autor da população — se a porta não estivesse ligada a tela
        // ficaria vazia para sempre, e não meio cheia.
        Ok("24") => gpu_spawn_pulse_demo::build_gpu_spawn_pulse_demo_document(doc, registry)
            .unwrap_or_default(),
        // O COMPASSO: o `carry` do contador divide o metrônomo por quatro e o
        // `pulse.adsr` transforma esse disparo instantâneo numa curva — as duas
        // features só se veem JUNTAS (ver o doc do módulo).
        Ok("25") => gpu_adsr_demo::build_gpu_adsr_demo_document(doc, registry).unwrap_or_default(),
        // O GRAFO GRITA: a MESMA cena `=25` com uma `pulse.signal` em cada relógio. Ela é a
        // fronteira `pulse.* -> ph2d-runtime` na direção grafo→runtime, e o que ela prova só é
        // visível com `PH2D_SIGNAL_LOG=1` ao lado — o terminal conta a mesma razão que o olho.
        Ok("26") => {
            let sinks =
                gpu_adsr_demo::build_gpu_signal_demo_document(doc, registry).unwrap_or_default();
            // ⚠️ **A cena se ANUNCIA, e é aqui que ela o faz** — no roteador, que é quem sabe
            // que o ambiente a pediu, e não no construtor, que os gates chamam às dezenas.
            // Sem a linha, um smoke sem `PH2D_SIGNAL_LOG=1` mostra a MESMA imagem da `=25` e
            // nada mais: o artista julgaria uma feature que ele não pode ver.
            eprintln!(
                "[signal-demo] O GRAFO GRITA: '{}' a cada batida ({} s) e '{}' a cada {}.\n  \
                 (!) Rode com PH2D_SIGNAL_LOG=1: os nomes saem no terminal, na MESMA razao que\n  \
                 o olho conta na tela (4 pulos por crescimento). Arrastar a regua nao imprime\n  \
                 nada -- um sinal e' travessia de play para a frente, nunca estar num tique.",
                gpu_adsr_demo::TIC,
                gpu_adsr_demo::BEAT,
                gpu_adsr_demo::COMPASSO,
                gpu_adsr_demo::DIVIDE_BY,
            );
            sinks
        }
        // **Sem env: a TELA VAZIA** (Enio, 2026-08-07: *"tire a cena da cachoeira"*). O
        // editor abria com a neve caindo no mar — um sistema de partículas inteiro que o
        // artista tinha de apagar antes de começar. Quem quiser um grafo o traz pelo
        // command-palette (`A`); as cenas de demonstração seguem todas acessíveis pelo
        // `PH2D_GPU_COOK_DEMO` acima, e a neve pelo censo/gates (`strobe`, `cfg(test)`).
        _ => Vec::new(),
    }
}
