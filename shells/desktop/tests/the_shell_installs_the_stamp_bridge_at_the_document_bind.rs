//! **Arch-gate: a ponte do carimbo é instalada no BIND, ao lado do pré-aquecimento dos shaders.**
//!
//! ## Por que um gate de TEXTO
//!
//! A instalação mora dentro de `painter_bridge::dispatch`, que exige `hero`/`sim`/`renderer`/`camera`
//! mais uma **janela** — nenhum teste de unidade a alcança. Os gates de COMPORTAMENTO do outro lado
//! da costura vivem no tool (`ph2d-tool-painter`, `tool::paint::stamp_device::tests`) e provam que a
//! ponte é usada, que ela escreve exatamente a região declarada e que declinar devolve o lote à CPU
//! byte a byte. Este aqui prova que a shell **entrega** a ponte, e no momento certo.
//!
//! ## O momento é a metade que importa
//!
//! Construir o `StampPass` **compila um shader**. É a mesma classe de custo que fez o `prewarm`
//! existir: os 28 ms de criação de pipeline do preview caíam no primeiro traço, o gesto em que o
//! artista está esperando a tinta aparecer (doc 28 §4.8, medido). O bind é o vão HUMANO entre
//! escolher o sprite e levar o mouse à tela.
//!
//! ⚠️ **A afirmação é POSICIONAL de propósito** — *depois do `bind_document`, junto do `prewarm`* —
//! e não uma distância em bytes. Um gate ancorado em distância é um proxy que expira: esta linha já
//! perdeu dois gates assim quando um terceiro consumidor entrou no meio
//! ([[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]]).

const SRC: &str = include_str!("../src/render_loop/painter_bridge.rs");
const WIRE: &str = include_str!("../src/render_loop/painter_stamp_device.rs");

/// A ponte é instalada no bind, DEPOIS de o documento existir e junto do pré-aquecimento.
///
/// **Mutação que deve sangrar:** apagar a chamada, ou movê-la para antes do `bind_document`.
#[test]
fn the_bind_installs_the_stamp_bridge_after_the_document_exists() {
    let bind = SRC
        .find("painter.bind_document(")
        .expect("o bloco de bind sumiu do painter_bridge — atualize este gate");
    let install = SRC
        .find("painter_stamp_device::install(painter, renderer)")
        .expect(
            "a shell não instala mais a ponte do carimbo. Sem ela o tool nunca publica um lote e a \
             rota do device fica INERTE no produto — verde em toda suíte, porque os gates do tool \
             instalam a própria ponte falsa.",
        );
    assert!(
        install > bind,
        "a ponte é instalada ANTES do `bind_document`: o passe seria construído para um documento \
         que ainda não existe, e o custo de compilar o shader voltaria a cair no primeiro traço."
    );
    let prewarm = SRC
        .find("painter_gpu_preview::prewarm(")
        .expect("o pré-aquecimento do preview sumiu — os dois pagam a mesma compilação no bind");
    assert!(
        prewarm > bind && install > prewarm,
        "a instalação da ponte saiu de perto do pré-aquecimento. Os dois existem pela MESMA razão \
         (compilar shader fora do gesto do artista) e no MESMO vão humano; separá-los é como um \
         deles volta a cair no primeiro traço sem ninguém notar."
    );
}

/// **A ponte é idempotente**, e isso não é higiene: o bind acontece a cada troca de sprite, e
/// recriar o passe por bind pagaria a compilação de novo — exatamente o custo que ele evita.
///
/// **Mutação que deve sangrar:** apagar o `if painter.has_device_stamp() { return; }`.
#[test]
fn installing_twice_does_not_recompile_the_shader() {
    assert!(
        WIRE.contains("if painter.has_device_stamp() {") && WIRE.contains("return;"),
        "a instalação perdeu a guarda de idempotência: o bind acontece a cada troca de sprite, e \
         sem ela cada troca recompila o shader — o custo que o pré-aquecimento existe para evitar."
    );
}

/// **A contenção é estrutural: `wgpu` não atravessa para o tool.**
///
/// O que cruza a fronteira é dado simples (bytes, uma tabela, discos). Se este arquivo — a ÚNICA
/// tradução entre os dois vocabulários — deixasse de ser o único lugar onde `GpuDab` e `DeviceDab`
/// se encontram, o tool teria de aprender o que WGSL alinha.
///
/// **Mutação que deve sangrar:** fazer a `ph2d-tool-painter` depender da `ph2d-paint-gpu`.
#[test]
fn the_tool_crate_never_learns_what_wgsl_aligns() {
    const TOOL_MANIFEST: &str = include_str!("../../../crates/ph2d-tool-painter/Cargo.toml");
    assert!(
        !TOOL_MANIFEST.contains("ph2d-paint-gpu"),
        "a `ph2d-tool-painter` passou a depender da crate do device. A contenção corta nos DOIS \
         sentidos: nada de `wgpu` entra no tool, e a `ph2d-paint-gpu` não alcança o \
         `falloff_weight` — é isso que a impede de ter opinião sobre a lei que carimba \
         (doc 33 §2). A tradução mora em `painter_stamp_device.rs`, no shell, e só lá."
    );
    assert!(
        WIRE.contains("GpuDab") && WIRE.contains("DeviceDab") || WIRE.contains("job.dabs"),
        "a tradução entre o dado do tool e o layout do buffer saiu deste arquivo"
    );
}
