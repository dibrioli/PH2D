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

use super::demo_conferencia as conferencia;
use super::demo_conferencia_animadores as animadores;
use super::*;

/// O maior nível que o roteador conhece — o teto da varredura do gate abaixo.
/// ⚠️ **Sobe com a cena nova**, e se alguém esquecer, o controle de contagem do gate
/// não acusa (ele mede o piso). O que acusa é a cena nova nunca ser diagnosticada —
/// então esta linha anda junto com o braço novo do `match`.
#[cfg(test)]
const MAX_DEMO_LEVEL: u32 = 76;

/// Os sinks da cena que o ambiente pediu — vazio quando ele não pediu nada, que é a TELA
/// VAZIA com que o editor abre.
pub(super) fn demo_sinks(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    build_level(
        std::env::var("PH2D_GPU_COOK_DEMO").ok().as_deref(),
        doc,
        registry,
    )
}

/// **O roteador, sem o ambiente** — a MESMA lista de níveis, chamável por um teste.
///
/// ⚠️ **Extraído em 2026-08-20 porque nenhuma cena era testável de fora**, e um
/// smoke reprovou por um fio que faltava numa delas (a `=71`, banda 6). O
/// diagnoser da casa (`ph2d_motion_diagnose`) sabia apontar aquele buraco desde
/// sempre — o que não existia era uma porta por onde um gate pudesse MONTAR uma
/// cena e perguntar. *Um instrumento que nenhum passo consegue invocar não protege
/// nada.*
///
/// ⚠️ Continua a ser o ÚNICO `match` de níveis (ver o cabeçalho): quem acrescenta
/// uma cena acrescenta um braço aqui, e a sonda passa a vê-la de graça.
pub(crate) fn build_level(
    level: Option<&str>,
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Vec<NodeId> {
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
    match level {
        Some("1") => build_gpu_demo_document(doc, registry).unwrap_or_default(),
        Some("2") => build_gpu_hybrid_demo_document(doc, registry).unwrap_or_default(),
        Some("3") => build_gpu_sim_demo_document(doc, registry).unwrap_or_default(),
        Some("4") => build_gpu_sea_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0130: the emitter FOUNTAIN — the id-gather (particles born/killed
        // across a sliding window, paired by arithmetic; fatia 5 tunes it live).
        Some("5") => build_gpu_emitter_demo_document(doc, registry).unwrap_or_default(),
        // The PANEL scene: 262.144 instances through both domains, so every
        // kind of card reading is on screen at once — the smoke for the GPU
        // path becoming the default.
        Some("6") => build_gpu_panel_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0140: the MURMURATION — the interacting sim (each agent reads its
        // neighbours) that the spatial grid lifts from a few-hundred-agent toy
        // to a swarm on the device (524.288, sized so it never stutters when the
        // flock gathers; the ceiling is millions).
        Some("7") => build_gpu_boids_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0140 Fase 5: the breathing PACKING — the second grid client,
        // and the first ITERATED kernel (the grid is rebuilt per sweep).
        Some("8") => build_gpu_collide_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0140 Fase 5: the spread SWEEP — the diagnostic scene. A slow,
        // linear triangle sweep of `spread` so the GPU meter shows a smooth
        // mountain (the cost is a function of the packing, no reach-boundary
        // step) rather than a staircase.
        Some("9") => build_gpu_sweep_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0135: the SIM-ZONE family — a fixed-population snow globe, the
        // state-loop container (`sim.zone` + `sim.step` + `sim.collide`) 100% on
        // the device. The boot snow's physics minus birth/death (which are
        // count-changing and still cook on the pump).
        Some("10") => build_gpu_zone_demo_document(doc, registry).unwrap_or_default(),
        // ADR-0139: the breathing HONEYCOMB — the first engine ALGORITHM
        // (Lloyd relaxation via jump flooding), and the cap that fell with
        // it: 20.000 points where the CPU-era node capped at 600.
        Some("11") => build_gpu_voronoi_demo_document(doc, registry).unwrap_or_default(),
        // The DEFORMER family: the whole-stream reduction channel. Two
        // deformers CHAINED, so the second one's fold must measure what the
        // first one produced (see the scene's own note).
        Some("12") => build_gpu_deform_demo_document(doc, registry).unwrap_or_default(),
        // The `Sum` half of the deformer channel: the centroid lens (two
        // reductions on one node).
        Some("13") => build_gpu_spherize_demo_document(doc, registry).unwrap_or_default(),
        // The widest reduction consumer: the bounding-box corner-pin (four
        // reductions, the first use of Min).
        Some("14") => build_gpu_four_point_warp_demo_document(doc, registry).unwrap_or_default(),
        // The count-changing deformer: the mandala fan-out (StreamOp
        // SourceRows, the first kernel to READ its template).
        Some("15") => build_gpu_kaleidoscope_demo_document(doc, registry).unwrap_or_default(),
        // O ORGANISMO: the whole reduction channel end to end — count-changing
        // fan (SourceRead) then the four count-preserving deformers, each
        // folding its reduction over the live stream the previous one produced.
        Some("16") => build_gpu_deform_organism_demo_document(doc, registry).unwrap_or_default(),
        // The FIELD family: `field.index_range` writes the `falloff` mask keyed
        // by ORDINAL (not position), coloured by a Solid tint — the middle band
        // of 262k rows glowing red, a mask no spatial falloff can draw.
        Some("17") => build_gpu_field_index_range_demo_document(doc, registry).unwrap_or_default(),
        // The spatial sibling: `field.box` masks by POSITION — a wide, thin box
        // is the razor-horizontal band (flat by y) that the ordinal index field
        // cannot draw. Blue, to read against `=17`'s red ordinal band.
        Some("18") => build_gpu_field_box_demo_document(doc, registry).unwrap_or_default(),
        // Composition: two fields (ordinal band + spatial vertical band) fanned
        // off one grid and unioned by `field.combine` into a red cross — the
        // whole fan-out on the device (the field family's thesis).
        Some("19") => build_gpu_field_combine_demo_document(doc, registry).unwrap_or_default(),
        // The ANGULAR field: `field.radial_sweep` — a 30° wedge repeated 6× into a
        // six-pointed blue star (a fan / radar). The shape a rectangle cannot make,
        // the HR-5 pseudo-angle sector on the device, and the 2nd field the canvas
        // gizmo drives (D9).
        Some("20") => build_gpu_field_radial_sweep_demo_document(doc, registry).unwrap_or_default(),
        // The REMAPPER: `field.box` paints a soft ramp, `field.remap` Quantizes it
        // into three topographic bands — the D1 factoring (every field defers its
        // remap here), the C4D Remapping tab as a downstream node.
        Some("21") => build_gpu_field_remap_demo_document(doc, registry).unwrap_or_default(),
        // The CURVE contour (A1): the same box ramp, remapped through a tent curve
        // authored in the text param — a blue RING no ramp or Quantize can make. The
        // kernel declines mode 4, so the remap cooks on the CPU (A1-gpu bakes the LUT).
        Some("22") => build_gpu_field_curve_demo_document(doc, registry).unwrap_or_default(),
        // O PORTÃO ESPACIAL (doc 89, folha 12): um metrônomo, um losango, e só quem está
        // DENTRO dele escuta o beat — `pulse.level` (o pulso vira número) + o canal
        // Falloff do `value.attribute` (o peso do campo vira legível), os dois elos que
        // faltavam para eventos e campos se encontrarem.
        Some("23") => build_gpu_pulse_gate_demo_document(doc, registry).unwrap_or_default(),
        // AS CINCO FONTES (doc 89, folha 12): a `=23` mostra um campo decidindo QUEM escuta
        // um evento; esta mostra um evento decidindo O QUE passa a existir. `rate = 0`, então
        // o pulso é o ÚNICO autor da população — se a porta não estivesse ligada a tela
        // ficaria vazia para sempre, e não meio cheia.
        Some("24") => gpu_spawn_pulse_demo::build_gpu_spawn_pulse_demo_document(doc, registry)
            .unwrap_or_default(),
        // O COMPASSO: o `carry` do contador divide o metrônomo por quatro e o
        // `pulse.adsr` transforma esse disparo instantâneo numa curva — as duas
        // features só se veem JUNTAS (ver o doc do módulo).
        Some("25") => {
            gpu_adsr_demo::build_gpu_adsr_demo_document(doc, registry).unwrap_or_default()
        }
        // O GRAFO GRITA: a MESMA cena `=25` com uma `pulse.signal` em cada relógio. Ela é a
        // fronteira `pulse.* -> ph2d-runtime` na direção grafo→runtime, e o que ela prova só é
        // visível com `PH2D_SIGNAL_LOG=1` ao lado — o terminal conta a mesma razão que o olho.
        Some("26") => {
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
        // A CENTELHA QUE ESTOURA: a MESMA pergunta da `=24` com o gatilho sendo a própria
        // MORTE — o `sim.replicate` da referência, que aqui é uma FIAÇÃO e não um nó.
        Some("27") => {
            let sinks =
                gpu_death_demo::build_gpu_death_demo_document(doc, registry).unwrap_or_default();
            // ⚠️ A cena se ANUNCIA, e é aqui que ela o faz — no roteador, que é quem sabe
            // que o ambiente a pediu. Os números são MEDIDOS (`probe_population`), não
            // escolhidos.
            eprintln!(
                "[death-demo] A CENTELHA QUE ESTOURA: {} sementes, vida {} s, e cada MORTE da a \
                 luz {} filhos NO LUGAR em que ela aconteceu.\n  \
                 (!) A taxa de nascimento e' ZERO: se a fiacao da morte nao funcionasse, a tela \
                 ficaria VAZIA para sempre\n  ao fim da primeira vida. Conte a cascata: {} -> {} \
                 -> {} -> ... uma geracao a cada {} s.",
                gpu_death_demo::SEEDS as u32,
                gpu_death_demo::LIFE,
                gpu_death_demo::BURST as u32,
                gpu_death_demo::SEEDS as u32,
                (gpu_death_demo::SEEDS * gpu_death_demo::BURST) as u32,
                (gpu_death_demo::SEEDS * gpu_death_demo::BURST * gpu_death_demo::BURST) as u32,
                gpu_death_demo::LIFE,
            );
            sinks
        }
        Some("28") => {
            let sinks =
                gpu_radius_demo::build_gpu_radius_demo_document(doc, registry).unwrap_or_default();
            // Os números são MEDIDOS (`probe_radius_rest`), não escolhidos.
            eprintln!(
                "[radius-demo] A TINTA POUSA SOBRE O CHAO: duas fileiras de {} discos, de {} a                  {} de tamanho,
  caindo no MESMO chao (y = {}). A ESQUERDA colide um PONTO:                  todo CENTRO pousa na linha,
  entao cada sprite afunda pela propria metade                  (0,15 a 0,60 unidade). A DIREITA colide o SPRITE:
  as BORDAS DE BAIXO                  alinham e os centros e' que ficam em cinco alturas.
                   (!) Os tamanhos VARIAM de proposito: com discos iguais um `height` subido a                  mao daria
  o mesmo desenho, e a cena nao provaria nada.",
                gpu_radius_demo::COLS as u32,
                gpu_radius_demo::SIZE_MIN,
                gpu_radius_demo::SIZE_MAX,
                gpu_radius_demo::FLOOR,
            );
            sinks
        }
        Some("29") => {
            let sinks =
                gpu_ramp_demo::build_gpu_ramp_demo_document(doc, registry, gpu_ramp_demo::RAMP_DEG)
                    .unwrap_or_default();
            // Os números são MEDIDOS (`probe_ramp_chute`), não escolhidos.
            eprintln!(
                "[ramp-demo] A CALHA: {} discos caem sobre uma RAMPA de {} graus (a mesma \
                 `sim.collide`\n  de sempre, com o ANGULO novo), deslizam para a direita e sao \
                 parados por uma PAREDE\n  em x = {} -- que e' o MESMO no', um quarto de volta. \
                 Medido: o centroide sai de\n  -1,50 no chao horizontal para +2,27 na rampa,\n  e o disco da frente encosta na parede.\n\
                   (!) Encadear colisores sempre funcionou; o que nao existia era uma RAMPA para \
                 encadear.",
                (gpu_ramp_demo::ROWS * gpu_ramp_demo::COLS) as u32,
                gpu_ramp_demo::RAMP_DEG,
                gpu_ramp_demo::WALL_X,
            );
            sinks
        }
        Some("30") => {
            let sinks =
                gpu_hit_demo::build_gpu_hit_demo_document(doc, registry, gpu_hit_demo::MARK)
                    .unwrap_or_default();
            // Os números são MEDIDOS (`probe_hit_mark`), não escolhidos.
            eprintln!(
                "[hit-demo] QUEM ESTA ENCOSTADO: {} discos caem sobre um obstaculo e um chao. O \n                   `sim.collide` escreve a coluna `hit` (quao fundo a colisao deste tique empurrou),\n                   o `value.attribute(Hit)` a LE, o `value.map_range` a satura e o\n                   `motion.drive(Size, Set)` a mostra -- quem esta' encostado e' 3,3x maior.\n\
                   (!) `Set`, nunca `Add`: `hit` e' um INSTANTE, e um corpo em repouso continua em\n                   contato -- somar isso a cada tique crescia para sempre (0,455 -> 3,021 aos 8 s\n                   com a chuva ja' parada). Agora o quadro PARA: size 0,720 aos 3, 5 e 8 s.",
                (gpu_hit_demo::ROWS * gpu_hit_demo::COLS) as u32,
            );
            sinks
        }
        Some("31") => {
            let sinks =
                gpu_speed_demo::build_gpu_speed_demo_document(doc, registry, gpu_speed_demo::LIMIT)
                    .unwrap_or_default();
            // Os números são MEDIDOS (`probe_speed_ceiling`), não escolhidos.
            eprintln!(
                "[speed-demo] O TETO DE VELOCIDADE: um atrator forte puxa {} elementos, e perto\n                   do centro a velocidade explode. O `sim.step` ganhou um teto ({} u/s) que capa a\n                   DISTANCIA que cada um anda no tique -- entre a velocidade e a posicao, nao depois.\n\
                   (!) O A/B e' o proprio controle: selecione o `Simulation Step` e ponha\n                   Speed Limit em 0 (zero e' DESLIGADO, nao 'congele'). A nuvem volta a se\n                   estilingar para fora de quadro.",
                (gpu_speed_demo::ROWS * gpu_speed_demo::COLS) as u32,
                gpu_speed_demo::LIMIT,
            );
            sinks
        }
        // **Sem env: a TELA VAZIA** (Enio, 2026-08-07: *"tire a cena da cachoeira"*). O
        // editor abria com a neve caindo no mar — um sistema de partículas inteiro que o
        // artista tinha de apagar antes de começar. Quem quiser um grafo o traz pelo
        // command-palette (`A`); as cenas de demonstração seguem todas acessíveis pelo
        // `PH2D_GPU_COOK_DEMO` acima, e a neve pelo censo/gates (`strobe`, `cfg(test)`).
        // ⬛ AS CENAS DA CONFERÊNCIA (doc 89). Cada uma responde UMA pergunta que
        // um gate não sabe fazer, e duas delas pedem um GESTO em vez de uma foto.
        //
        // A CURVA REVELA: a fila de cima gira com a tangente, a de baixo é o
        // CONTROLE com o toggle desligado. ⚠️ Arraste o **To** de 0 a 1 no painel
        // — o write-on é uma animação, e nenhum nó do grafo anima um param.
        Some("32") => {
            conferencia_demos::build_write_on_demo_document(doc, registry).unwrap_or_default()
        }
        // O PIVÔ: duas grades idênticas com `scale = 2`; a de cima pivota na
        // origem do mundo e FOGE, a de baixo pivota no centroide e só se espalha.
        Some("33") => {
            conferencia_demos::build_pivot_demo_document(doc, registry).unwrap_or_default()
        }
        // QUALQUER FÓRMULA É UMA FORÇA: nenhuma `force.*` na cena — duas fórmulas
        // escrevem `accel` e o integrador as consome. ⚠️ Se a nuvem NÃO girar,
        // ou as lanes `x`/`y` da fórmula ou o alvo do `make_point` estão mortos.
        Some("34") => {
            conferencia_demos::build_formula_force_demo_document(doc, registry).unwrap_or_default()
        }
        // QUEM MIRA, E QUANTO: só a faixa do campo mira, e as bordas macias miram
        // PELA METADE — o contrato de família de que o `look_at` era a exceção.
        Some("35") => {
            conferencia_demos::build_partial_aim_demo_document(doc, registry).unwrap_or_default()
        }
        // O MODO É DO SINK: três `motion.output` sobre nuvens SOBREPOSTAS, em
        // Normal / Add / Multiply. ⚠️ A pergunta é de OLHO — os três têm de
        // parecer três coisas diferentes; se os três forem iguais, o param não
        // está chegando à rota que este build usa (bissecte com `PH2D_GPU_COOK=0`).
        Some("36") => {
            conferencia_demos::build_sink_blend_demo_document(doc, registry).unwrap_or_default()
        }
        // O CATÁLOGO de kernels do `value.noise` (doc 89 folha 15): Value /
        // Perlin / Cellular-Cells / Cellular-Cracks, o mesmo campo espacial a
        // dirigir o TAMANHO. ⚠️ A pergunta é de OLHO — as quatro grades têm de
        // desenhar quatro coisas; se forem iguais, o `kernel` não chegou.
        Some("37") => {
            conferencia_demos::build_noise_kernel_demo_document(doc, registry).unwrap_or_default()
        }
        // A DIRECAO: o `value.attribute(Direction) -> motion.drive(Rotation)` que fecha a
        // linha da doc 89 §10.0 citada por CINCO familias.
        Some("38") => {
            let sinks = conferencia_demos_direction::build_direction_demo_document(doc, registry)
                .unwrap_or_default();
            // ⚠️ A cena se ANUNCIA aqui, no roteador, que e' quem sabe que o ambiente a pediu
            // (o construtor os gates chamam as dezenas). Sem a linha, quem nao souber o que
            // olhar julga duas nuvens iguais.
            eprintln!(
                "[direction-demo] VIRE PARA ONDE ESTA INDO: o MESMO redemoinho duas vezes,                  {} pecas cada.
  ESQUERDA sem o canal: os tracos ficam HORIZONTAIS e                  derrapam de lado.
  DIREITA com                  `value.attribute(Direction) -> motion.drive(Rotation)`: cada traco aponta ao                  longo do proprio caminho.
  (!) As pecas sao TRACOS de proposito -- um                  quadrado rodado 90 graus e' o mesmo quadrado, e a cena nao provaria nada.",
                (conferencia_demos_direction::SIDE * conferencia_demos_direction::SIDE) as u32,
            );
            sinks
        }
        Some("39") => {
            let sinks =
                conferencia_demos_text::build_text_demo_document(doc, registry).unwrap_or_default();
            eprintln!(
                "[text-demo] O TEXTO E' UMA LETRA POR INSTANCIA: a palavra \"{}\" duas vezes, {} letras cada.
  EM CIMA sem o canal: a palavra assenta na baseline, reta.
  EM BAIXO com `value.instance_field(Ramp) -> motion.drive(Rotation)`: cada letra roda um pouco mais que a anterior, abrindo em leque ate {} graus.
  (!) E' o LEQUE que prova a wave, nao o texto aparecer -- um bloco emitido como UMA instancia giraria RIGIDAMENTE, com todas as letras no mesmo angulo.",
                conferencia_demos_text::WORD,
                conferencia_demos_text::WORD.chars().count(),
                conferencia_demos_text::FAN_DEG as i32,
            );
            sinks
        }
        Some("40") => {
            let sinks = conferencia_demos_audio::build_audio_demo_document(doc, registry)
                .unwrap_or_default();
            eprintln!(
                "[audio-demo] O SOM DIRIGE A GEOMETRIA: a MESMA fileira de {} barras duas vezes.
  EM CIMA sem o canal: todas do mesmo tamanho, o CONTROLE.
  EM BAIXO com `audio.bands -> motion.drive(Size)`: cada barra respira com a banda dela.
  (!) E' o movimento COM O TEMPO que prova a wave, nao as barras diferirem -- um campo por-INDICE tambem as deixaria desiguais, e ficaria PARADO.
  A cena escreveu um VARRIMENTO de {:.0}s (60 Hz -> 12 kHz) em disco, entao a figura e' uma ONDA correndo da esquerda para a direita. De PLAY.",
                conferencia_demos_audio::BANDS,
                conferencia_demos_audio::SWEEP_SECS,
            );
            sinks
        }
        // As cenas de GRUPO da conferência (doc 89, a segunda volta) — o documento
        // e a prosa de cada uma moram no irmão `motion_state_demo_conferencia.rs`.
        Some("41") => conferencia::arith(doc, registry),
        Some("42") => conferencia::noise_clock(doc, registry),
        Some("43") => conferencia::stats(doc, registry),
        Some("44") => conferencia::table_seed(doc, registry),
        Some("45") => conferencia::compare(doc, registry),
        Some("46") => conferencia::envelope(doc, registry),
        Some("47") => conferencia::velocity(doc, registry),
        Some("48") => conferencia::collide(doc, registry),
        Some("49") => conferencia::proximity(doc, registry),
        Some("50") => conferencia::pin(doc, registry),
        Some("51") => conferencia::space(doc, registry),
        Some("52") => conferencia::weight(doc, registry),
        Some("53") => conferencia::rate(doc, registry),
        Some("54") => animadores::octave(doc, registry),
        Some("55") => animadores::shape(doc, registry),
        Some("56") => animadores::column(doc, registry),
        Some("57") => animadores::wave(doc, registry),
        Some("58") => animadores::axes(doc, registry),
        Some("59") => animadores::clock(doc, registry),
        Some("60") => animadores::field_space(doc, registry),
        Some("61") => conferencia::substep(doc, registry),
        Some("62") => conferencia::join(doc, registry),
        Some("63") => conferencia::sortkey(doc, registry),
        Some("64") => conferencia::taper(doc, registry),
        Some("65") => conferencia::cursor(doc, registry),
        Some("66") => conferencia::field_family(doc, registry),
        Some("67") => conferencia::drizzle(doc, registry),
        Some("68") => conferencia::deform(doc, registry),
        Some("69") => conferencia::transform_family(doc, registry),
        Some("70") => conferencia::fx_family(doc, registry),
        Some("71") => conferencia::force_family(doc, registry),
        Some("72") => conferencia::color_family(doc, registry),
        Some("73") => conferencia::rank_family(doc, registry),
        Some("74") => conferencia::goal_family(doc, registry),
        Some("75") => conferencia::sim_family(doc, registry),
        Some("76") => conferencia::style_family(doc, registry),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **NENHUMA CENA DA CONFERÊNCIA MONTA UM GRAFO COM BURACO DE SETUP.**
    ///
    /// ⚠️ **Este portão nasceu de um smoke reprovado** (Enio, 2026-08-20: *"6.
    /// EMPUXO com a coluna `density`. Todas as peças paradas"*). A causa era um fio
    /// que faltava na cena `=71`: o `value.instance_field` que alimentava a
    /// densidade estava **solto**, e o doc dele diz *"unconnected → one degenerate
    /// value"* — ele dava UM valor, o `motion.drive` transmitia-o a todos, e ele era
    /// ZERO. Densidade zero é empuxo nenhum.
    ///
    /// ⚠️ **A casa já sabia diagnosticar isto, e ninguém perguntava.** O
    /// `ph2d_motion_diagnose` reporta `MissingSource`/`MissingInput` exactamente para
    /// um nó sem nada ligado; o que não existia era uma porta por onde um gate
    /// pudesse MONTAR uma cena e perguntar — daí o [`build_level`].
    /// *Um instrumento que nenhum passo invoca não protege coisa nenhuma.*
    ///
    /// ⚠️ **A barra é ZERO, e ela foi MEDIDA antes de virar barra.** A sonda achou
    /// **seis** cenas marcadas (`=3`, `=31`, `=38`, `=57`, `=61`, `=71`) — e as seis
    /// estavam CERTAS: o falso positivo era do diagnoser, cuja isenção exigia que a
    /// aresta atrasada viesse do PRÓPRIO nó, quando o laço canónico de força a
    /// recebe do integrador. Curado lá; aqui sobra o zero.
    ///
    /// ⛔ **Se uma cena futura encenar um defeito DE PROPÓSITO**, ela não entra numa
    /// allowlist muda: ou o defeito não é de SETUP (a `=45` encena um nome que não
    /// resolve, e passa), ou o gate ganha o nível NOMEADO com o motivo ao lado.
    /// **O QUE UMA CENA DE FACTO DESENHA** — a caixa de cada banda, medida.
    ///
    /// ⚠️ **Este instrumento nasceu de um smoke reprovado** (Enio, 2026-08-21: *"esses
    /// exemplos não são compreensíveis. tudo misturado e bagunçado"*), e a causa não era
    /// nenhuma das features: era eu a **autorar cenas às cegas**. Um `motion.move(dx, dy)`
    /// diz onde o CENTRO de uma banda vai; ele não diz nada sobre a LARGURA dela, e a
    /// largura sai de `(cols − 1) · gap`, três nós acima. Duas bandas cujos centros
    /// distam 12 unidades sobrepõem-se alegremente se cada uma medir 8.
    ///
    /// ⚠️ **Não há como ver isto sem cozinhar.** O grafo é o que eu escrevo; a IMAGEM é o
    /// que o cook devolve — e até esta sonda existir, o único instrumento que media a
    /// diferença entre os dois era o olho do Enio, depois de compilar em release.
    ///
    /// `PH2D_LAYOUT_LEVEL=73 cargo test -p ph2d-host-desktop --bins
    /// measure_scene_layout -- --ignored --nocapture` (sem a env, varre tudo).
    #[test]
    #[ignore = "sonda de layout, não um gate — `-- --ignored --nocapture`"]
    fn measure_scene_layout() {
        let only = std::env::var("PH2D_LAYOUT_LEVEL").ok();
        for level in 1..=MAX_DEMO_LEVEL {
            if only.as_deref().is_some_and(|w| w != level.to_string()) {
                continue;
            }
            // ⚠️ **Um `MotionState` inteiro, e não um `MotionDoc` solto.** Uma cena de FORMA
            // ou de TEXTO lê a geometria por CANAL EXTERNO, que só o shell publica — um cook
            // virgem não tem external nenhum e devolve **zero instâncias**, que esta sonda
            // imprimiria como `VAZIA`. Seria ela a acusar a cena de um defeito dela própria,
            // pela terceira vez nesta linha (a sonda de movimento e o harness do texto
            // pagaram as outras duas).
            let mut state = MotionState::new();
            let sinks = build_level(Some(&level.to_string()), &mut state.doc, &state.registry);
            if sinks.is_empty() {
                continue;
            }
            crate::render_loop::motion_externals::publish_all(&mut state, 0.0);
            println!("--- cena =`{level}` · {} bandas", sinks.len());
            for (k, sink) in sinks.iter().enumerate() {
                match band_box(&mut state, *sink) {
                    Some((n, lo, hi)) => println!(
                        "  banda {:>2}: n={n:<6} x [{:>7.2} .. {:>7.2}]  y [{:>7.2} .. {:>7.2}]  ({:.2} x {:.2})",
                        k + 1,
                        lo[0],
                        hi[0],
                        lo[1],
                        hi[1],
                        hi[0] - lo[0],
                        hi[1] - lo[1]
                    ),
                    None => println!("  banda {:>2}: VAZIA", k + 1),
                }
            }
        }
    }

    /// A contagem e a caixa envolvente de uma banda, cozinhada em `t = 0`.
    ///
    /// ⚠️ **A caixa é a das POSIÇÕES, não a da tinta.** Uma banda de uma instância só —
    /// típica de uma cena de FORMA, em que a arte inteira é um `geometry_id` — mede
    /// `0.00 x 0.00`, e isso não quer dizer *vazia*: quer dizer *um ponto*. A extensão
    /// desenhada ali é a do `VecPath` vezes a coluna `size`, e vive no store, não no stream.
    fn band_box(
        state: &mut MotionState,
        sink: ph2d_nodegraph::graph::NodeId,
    ) -> Option<(usize, [f32; 2], [f32; 2])> {
        use ph2d_nodegraph::attr::Column;
        // ⚠️ O cook do PRÓPRIO estado — é nele que o `publish_all` escreveu os externals.
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, 0.0)
            .ok()?;
        let s = out.first()?.as_stream();
        let Some(Column::Vec2(p)) = s.get("P") else {
            return None;
        };
        if p.is_empty() {
            return None;
        }
        let mut lo = [f32::INFINITY; 2];
        let mut hi = [f32::NEG_INFINITY; 2];
        for q in p {
            for a in 0..2 {
                lo[a] = lo[a].min(q[a]);
                hi[a] = hi[a].max(q[a]);
            }
        }
        Some((p.len(), lo, hi))
    }

    /// **QUANTO UMA CENA DE FACTO ANDA** — a irmã temporal da [`measure_scene_layout`].
    ///
    /// ⚠️ **Ela nasceu de um smoke reprovado E provou-se contra uma cena APROVADA**
    /// (Enio, 2026-08-21: *"tudo foi levado pelo vento. nada rasgou"*). A primeira
    /// versão desta sonda dizia que a `=75` não andava — e dizia o mesmo da **`=71`**,
    /// que o Enio já tinha aprovado. Foi isso que provou que o erro era do HARNESS: o
    /// `pre` de um circuito sequencial só avança quando o quadro FECHA
    /// ([`Cook::advance_tick`]), e um laço que só `cook`a lê o mesmo tique N vezes.
    ///
    /// *Uma sonda que acusa a cena boa está a acusar-se a si própria.*
    ///
    /// `PH2D_LAYOUT_LEVEL=75 cargo test -p ph2d-host-desktop --bins
    /// measure_scene_motion -- --ignored --nocapture`
    #[test]
    #[ignore = "sonda de movimento, não um gate — `-- --ignored --nocapture`"]
    fn measure_scene_motion() {
        /// Quantos tiques a sonda corre — **cinco** segundos a 60 fps.
        ///
        /// ⚠️ **Dois segundos não chegavam, e isso é uma medição:** a cena `=75` faz o
        /// vento SUBIR ao longo de 4 s, e o pano só solta lá pelos 2. Com a janela curta
        /// as duas metades saíam com o mesmo número — a sonda dizia *"não há diferença"*
        /// sobre uma cena que a tem, cinco décimos de segundo mais tarde.
        const TICKS: u32 = 300;
        use ph2d_nodegraph::attr::Column;
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registra");
        let only = std::env::var("PH2D_LAYOUT_LEVEL").ok();
        for level in (1..=MAX_DEMO_LEVEL).map(|l| l.to_string()) {
            if only.as_deref().is_some_and(|w| *w != level) {
                continue;
            }
            let level = level.as_str();
            let mut doc = MotionDoc::default();
            let sinks = build_level(Some(level), &mut doc, &reg);
            if sinks.is_empty() {
                continue;
            }
            for (k, sink) in sinks.iter().enumerate() {
                let mut cook = ph2d_nodegraph::cook::Cook::new();
                let (mut a, mut b) = (Vec::new(), Vec::new());
                for t in 0..TICKS {
                    let ph = f64::from(t) / 60.0;
                    let out = cook.cook(&doc.graph, &reg, *sink, ph).expect("cozinha");
                    if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                        if t == 0 {
                            a = p.clone();
                        }
                        b = p.clone();
                    }
                    cook.advance_tick(&doc.graph, &reg, ph).expect("avança");
                }
                if a.is_empty() || b.is_empty() {
                    continue;
                }
                // O MAIOR percurso da banda, não o do elemento 0: uma banda cujo
                // primeiro elemento esteja pinado andaria zero e leria como parada.
                let d = a
                    .iter()
                    .zip(&b)
                    .map(|(p, q)| (q[0] - p[0]).abs() + (q[1] - p[1]).abs())
                    .fold(0.0_f32, f32::max);
                println!("  cena ={level} banda {:>2}: maior percurso {d:.4}", k + 1);
            }
        }
    }

    #[test]
    fn no_conference_scene_ships_a_setup_hole() {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
        let (mut built, mut bad) = (0usize, Vec::new());
        for level in 1..=MAX_DEMO_LEVEL {
            let mut doc = MotionDoc::default();
            if build_level(Some(&level.to_string()), &mut doc, &reg).is_empty() {
                continue;
            }
            built += 1;
            let d = ph2d_motion_diagnose::diagnose(&doc.graph, &reg);
            if !d.is_empty() {
                bad.push(format!("=`{level}`: {d:?}"));
            }
        }
        assert!(
            bad.is_empty(),
            "cenas com buraco de setup:\n{}",
            bad.join("\n")
        );
        // ⚠️ CONTROLE: sem isto o portão passa por VÁCUO no dia em que o
        // `build_level` deixar de montar (um `_ =>` que engula tudo, um refactor do
        // env). Uma varredura que não acha nada não prova nada.
        assert!(built >= 60, "controle: a varredura montou só {built} cenas");
    }
}
