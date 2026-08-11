//! **Arch-gate: o overlay desenha a leitura que a PONTE publicou** (`W-Probes`).
//!
//! ## O que este gate protege
//!
//! Os sensores do player só podem ser desenhados a partir da leitura que a
//! ponte grava no tique (`PhysicsBridge::player_probe_marks`), preenchida pelas
//! MESMAS portas que lançam os raios. Um segundo cálculo do lado do desenho —
//! *"onde este raio nasce, mais ou menos"* — divergiria no primeiro dia em que
//! alguém mexesse numa das duas, e o sintoma seria um overlay que **mente sobre
//! o que o produto mede**, que é a única coisa pior do que overlay nenhum. É a
//! regra que o `scaled_shape` (W6) impôs ao contorno e a `zone_force_world_at`
//! (W-AreaFrame) impôs à seta da zona.
//!
//! ## Por que um gate de TEXTO
//!
//! A fiação mora no corpo do `render_frame`, que exige janela + GPU — **nenhum
//! unit test a alcança** (irmão do `the_node_overlay_is_suppressed_under_an_envelope`).
//! O desenho em si está gateado no `physics_overlay_probes::tests`, e a leitura
//! no `ph2d-physics-ecs::tests/player_probe_view`; o que só aqui é observável é
//! a COSTURA: passar `&[]` compila, apaga os sensores da tela e deixa as duas
//! outras famílias de gate **verdes**.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

fn at(needle: &str) -> usize {
    SRC.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu do render_loop — se foi renomeado, atualize este gate (e confira que \
             os sensores do player continuam na tela: `PH2D_PHYSICS_SMOKE=108`)"
        )
    })
}

/// **A lista de marcas sai da PONTE, e não de uma reconstrução local.**
#[test]
fn the_marks_come_from_the_bridge() {
    let bind = at("let probes = ");
    let end = SRC[bind..].find(';').expect("fim do binding") + bind;
    let body = &SRC[bind..end];
    assert!(
        body.contains("player_probe_marks"),
        "`probes` tem de sair de `physics.player_probe_marks()` — qualquer outra fonte é uma \
         SEGUNDA resposta a *onde este sensor olha*:\n{body}"
    );
}

/// **E ela é ENTREGUE ao `draw`** — sem isto, o passe de sensores recebe uma
/// lista vazia todo quadro e não desenha um pixel, com tudo verde.
#[test]
fn the_marks_are_handed_to_the_draw() {
    let call = at("physics_overlay::draw(");
    let end = SRC[call..].find("\n            );").unwrap_or_else(|| {
        panic!("nao achei o fim da chamada de `physics_overlay::draw` — atualize este gate")
    }) + call;
    let args = &SRC[call..end];
    assert!(
        args.contains("&probes"),
        "o `draw` tem de receber `&probes`; entregar `&[]` compila e apaga os sensores da tela \
         sem nenhum gate de unidade notar"
    );
}

/// **CONTROLE POSITIVO:** o arquivo lido é o do produto e contém a chamada que
/// os dois gates acima inspecionam.
///
/// ⚠️ Sem ele, um `render_loop` que se mudasse de lugar deixaria os `find`
/// acima a varrer o vazio — e uma varredura vazia é a forma como um arch-gate
/// passa a ser verde sobre um produto que ninguém está a olhar.
#[test]
fn the_file_this_gate_reads_is_the_one_that_draws() {
    assert!(
        SRC.contains("physics_overlay::draw("),
        "este gate lê o arquivo errado"
    );
    assert!(
        SRC.len() > 100_000,
        "o `render_loop/mod.rs` encolheu de forma suspeita ({} bytes) — se ele foi partido, os \
         dois gates acima podem estar a ler o lado sem a chamada",
        SRC.len()
    );
}
