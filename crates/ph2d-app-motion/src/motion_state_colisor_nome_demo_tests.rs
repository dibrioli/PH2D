//! Os gates da cena `=137` (doc 131 §3, D.14): a coluna `collider` separa os discos da borda, a do
//! nome errado não; e as duas vão à placa.

use super::*;
use crate::motion_state::MotionState;

/// A fração dos discos DESENHADOS da borda que se sobrepõem a um vizinho, numa coluna (só ela cozida), ao fim de `ate`
/// tiques pela CPU — o que o passe do `Output` entrega ao desenho.
fn borda_sobreposta(k: usize, ate: u64) -> f32 {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let (saidas, _) = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = vec![saidas[k]];
    m.pump.define_a_lei(true);
    m.pump.mark_dirty();
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    let sinks = m.sinks.clone();
    for t in 0..=ate {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    }
    let p: Vec<[f32; 2]> = m.pump.vector_instances.iter().map(|v| v.world_pos).collect();
    assert!(
        p.len() > 100,
        "a coluna desenha os discos: {} ({:?}, {} sprites)",
        p.len(),
        m.pump.last_error(),
        m.pump.instances.len()
    );
    // A BORDA: os discos a mais de 2/3 do raio da galáxia, à volta do centro dela.
    #[expect(clippy::cast_precision_loss, reason = "centenas de discos")]
    let n = p.len() as f32;
    let c = p
        .iter()
        .fold([0.0f32; 2], |s, q| [s[0] + q[0] / n, s[1] + q[1] / n]);
    let d = |q: &[f32; 2]| (q[0] - c[0]).hypot(q[1] - c[1]);
    let longe = p.iter().map(d).fold(0.0f32, f32::max);
    let borda: Vec<&[f32; 2]> = p.iter().filter(|q| d(q) > longe * 2.0 / 3.0).collect();
    let sobrepostos = borda
        .iter()
        .filter(|a| {
            p.iter()
                .any(|b| !std::ptr::eq(**a, b) && (a[0] - b[0]).hypot(a[1] - b[1]) < 1.9 * DISCO)
        })
        .count();
    #[expect(clippy::cast_precision_loss, reason = "uma fração")]
    let f = sobrepostos as f32 / borda.len() as f32;
    f
}

/// ⭐ **O nome `collider` separa; o nome errado não** — o controlo é a mesma galáxia: sem leitor,
/// os discos da borda passam uns por cima dos outros.
#[test]
fn o_nome_collider_separa_os_discos_e_o_nome_errado_nao() {
    let (com, sem) = (borda_sobreposta(0, 120), borda_sobreposta(1, 120));
    eprintln!("cena 137: discos da borda sobrepostos com `collider` {com:.3} · com `colisor` {sem:.3}");
    assert!(sem > 0.5, "controlo: sem leitor a borda sobrepõe-se ({sem})");
    assert!(com < 0.1, "o colisor pelo nome não separou a borda: {com}");
}

/// ⭐ **E as duas colunas cozem na PLACA** — antes a da esquerda caía na CPU (`RECUSA_COLISOR`).
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn a_cena_do_colisor_pelo_nome_coze_na_placa() {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("adapter");
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let (saidas, _) = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = saidas.to_vec();
    let mut playhead = ph2d_core::Playhead::new(1.0 / 60.0);
    for k in 1..=5u64 {
        #[expect(clippy::cast_precision_loss, reason = "um quadro")]
        playhead.seek(k as f64 / 60.0);
        crate::motion_bridge::quadro::coze_o_quadro(&mut m, &gpu, &playhead, 1.0 / 60.0);
    }
    let rota = m.route_said.unwrap_or("");
    eprintln!("cena 137: {rota}");
    assert!(rota.starts_with("device"), "a cena caiu na CPU: {rota}");
}
