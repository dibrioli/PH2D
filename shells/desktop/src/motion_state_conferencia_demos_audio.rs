//! **O ÁUDIO** (`PH2D_GPU_COOK_DEMO=40`) — a cena do último P0 aberto da
//! conferência: o `audio.bands` do [doc 63 §3](docs/Motion%20Nodes/63_pesquisa_industria_2026_e_plano_estado_da_arte.md),
//! que a folha SOURCE §3 item 6 nomeava *"só para o consolidador não o perder na
//! fronteira entre famílias"*.
//!
//! ## O que a cena põe lado a lado
//!
//! **A MESMA fileira de barras duas vezes**, com o mesmo tamanho e o mesmo
//! arquivo. A de baixo tem dois nós a mais: `audio.bands → motion.drive(Size)`.
//!
//! - **EM CIMA** as barras ficam todas do mesmo tamanho — o CONTROLE.
//! - **EM BAIXO** cada barra respira com a banda dela, e o desenho MUDA com o
//!   playhead.
//!
//! ⚠️ **É o movimento COM O TEMPO que prova a wave, não as barras diferirem.**
//! Um campo por-índice qualquer (o `value.instance_field`, que já existia) também
//! deixaria as barras com alturas distintas — e ficaria PARADO. O que só o áudio
//! entrega é a fileira mudando quando a régua anda, e cada barra respondendo à
//! própria faixa de frequência.
//!
//! ## O arquivo é ESCRITO pela cena, e isso é deliberado
//!
//! Não há asset de áudio no repo, e um smoke que dependesse de o artista ter um
//! arquivo à mão testaria a coleção dele. A cena sintetiza um **varrimento** —
//! um tom que sobe de grave a agudo — e o grava em disco, então o que se vê é uma
//! **onda correndo pelas barras**, da esquerda para a direita, ao longo do tempo.
//! É a figura que torna o defeito óbvio: se as bandas estivessem trocadas, a onda
//! correria ao contrário; se o eixo de frequência estivesse errado, ela saltaria.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas barras — e, por construção, quantas bandas.
pub(crate) const BANDS: usize = 24;
/// Quanto o varrimento demora a atravessar o espectro (segundos).
pub(crate) const SWEEP_SECS: f32 = 6.0;
/// A amplitude do respiro no canal de tamanho.
pub(crate) const SIZE_GAIN: f32 = 2.5;

/// `bands → drive(Size)` sobre uma fileira, duas vezes. Devolve os DOIS sinks (o
/// de cima é o controle).
pub(crate) fn build_audio_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::Pos;
    let file = write_sweep()?;
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for k in 0..2u8 {
        let driven = k == 1;
        let row = 120.0 + f32::from(k) * 320.0;

        // Uma fileira de barras: a grade dá as posições, o transform as espalha.
        let grid = g.add_node("motion.lattice");
        g.set_param(grid, "cols", BANDS as f32);
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "spacing", 0.42);

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", if driven { -2.4 } else { 1.6 });
        let out = g.add_node("motion.output");

        for (i, n) in [grid, place, out].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 240.0,
                    y: row,
                },
            );
        }

        let tail = if driven {
            let bands = g.add_node("audio.bands");
            g.set_text_param(bands, ph2d_node_audio_bands::FILE_KEY, &file);
            g.set_param(bands, ph2d_node_audio_bands::param::COUNT, BANDS as f32);
            // Log é o default e é o certo aqui: o varrimento cruza o espectro em
            // oitavas, então bandas log-espaçadas o veem como velocidade constante.
            let drive = g.add_node("motion.drive");
            // ⚠️ **3, e o rótulo é a prova**: a lista do `motion.drive` é
            // `X · Y · Rotation · Size · …`, e `1` seria **Y** — o índice errado aqui
            // move as barras de lado em vez de as fazer crescer, e a fileira
            // continuaria "respondendo ao áudio" de um jeito que o gate de variação
            // aceitaria. O gate da cena mede a coluna `size` por nome.
            g.set_param(drive, "channel", 3.0); // Size
            g.set_param(drive, "mode", 0.0); // Add — as barras crescem do tamanho base
            g.set_param(drive, "scale", SIZE_GAIN);
            g.set_pos(
                bands,
                Pos {
                    x: 320.0,
                    y: row + 150.0,
                },
            );
            g.set_pos(
                drive,
                Pos {
                    x: 540.0,
                    y: row + 150.0,
                },
            );
            wire(g, grid, 0, bands, 0)?;
            wire(g, grid, 0, drive, 0)?;
            wire(g, bands, 0, drive, 1)?;
            drive
        } else {
            grid
        };
        wire(g, tail, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// Escreve o varrimento em disco e devolve o caminho.
///
/// ⚠️ **Um varrimento e não música:** a pergunta da cena é *"cada barra responde à
/// FAIXA dela?"*, e só uma fonte cuja frequência é conhecida a cada instante
/// responde isso de olho. Com música, uma fileira desalinhada e uma fileira certa
/// parecem as duas plausíveis.
pub(crate) fn write_sweep() -> Option<String> {
    let path = std::env::temp_dir().join("ph2d_audio_bands_sweep.wav");
    let sr = 48_000usize;
    let n = (SWEEP_SECS * sr as f32) as usize;
    // Varrimento EXPONENCIAL entre 60 Hz e 12 kHz: a fase é a integral da
    // frequência, senão o tom salta em vez de deslizar.
    let (f0, f1) = (60.0f64, 12_000.0f64);
    let secs = f64::from(SWEEP_SECS);
    let k = (f1 / f0).ln() / secs;
    let samples: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f64 / sr as f64;
            let phase = std::f64::consts::TAU * f0 * ((k * t).exp() - 1.0) / k;
            (phase.sin() * 0.9) as f32
        })
        .collect();

    let bytes = samples.len() * 4;
    let mut w: Vec<u8> = Vec::with_capacity(44 + bytes);
    w.extend(b"RIFF");
    w.extend(((36 + bytes) as u32).to_le_bytes());
    w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes());
    w.extend(3u16.to_le_bytes()); // IEEE float
    w.extend(1u16.to_le_bytes()); // mono
    w.extend((sr as u32).to_le_bytes());
    w.extend(((sr * 4) as u32).to_le_bytes());
    w.extend(4u16.to_le_bytes());
    w.extend(32u16.to_le_bytes());
    w.extend(b"data");
    w.extend((bytes as u32).to_le_bytes());
    for v in &samples {
        w.extend(v.to_le_bytes());
    }
    std::fs::write(&path, w).ok()?;
    Some(path.to_string_lossy().into_owned())
}

/// Uma aresta. Função LIVRE e não closure: uma closure que captura `g` o empresta
/// até ao fim do escopo, e o corpo do laço ainda precisa dele.
fn wire(
    g: &mut ph2d_nodegraph::graph::Graph,
    a: NodeId,
    ap: u16,
    b: NodeId,
    bp: u16,
) -> Option<()> {
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .ok()
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_audio_tests.rs"]
mod tests;
