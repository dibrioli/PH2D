//! ⭐⭐⭐ **AS FORMAS PELA ROTA DO DISPOSITIVO** (doc 121 W3) — a MESMA cena de estrelas carimbadas
//! cozida pelas duas rotas da ponte e desenhada pela placa de formas: na da CPU a bomba baixa as
//! `VectorInstance` e a placa sobe-as; na do dispositivo o cozimento escreve as cópias no buffer
//! dele e a placa lê-o sem o trazer de volta. As duas camadas comparam-se pixel a pixel.
//!
//! ⚠️ **As duas metades que a paridade sozinha não diz:** a rota do dispositivo tem de ter sido
//! TOMADA (senão o gate mede a CPU contra si mesma — a cerca do TIPO recusava toda forma viva até
//! esta wave), e o buffer das SPRITES tem de ter calado as formas (senão a cena desenharia as
//! estrelas por cima de quadrados de átlas, o defeito que a `=108` já mostrou).
//!
//! `#[ignore]`: precisa de adaptador real.
//!   `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --lib --release \
//!        formas_tests -- --ignored --nocapture`

use super::{GpuOutcome, cook_gpu};
use crate::motion_shape_placa::PlacaDeFormas;
use crate::motion_shape_placa::gpu_tests::{LADO, camara, compara, le_a_camada};
use crate::motion_state::MotionState;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::Edge;

/// O tique a comparar.
const TIQUE: u64 = 3;
const DT: f64 = 1.0 / 60.0;

/// `grelha 6×6 → duplicador ← estrela`, e um `motion.rotate` depois — a pose de cada cópia tem
/// base rodada, que é onde o seno do dispositivo e o da CPU se podem separar.
fn estado() -> MotionState {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let grade = g.add_node("motion.grid");
    g.set_param(grade, "rows", 6.0);
    g.set_param(grade, "cols", 6.0);
    g.set_param(grade, "gap_x", 2.3);
    g.set_param(grade, "gap_y", 2.3);
    let forma = g.add_node("source.shape");
    let estrela = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Star)
        .expect("a estrela");
    #[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
    g.set_param(forma, ph2d_node_motion_shape::param::KIND, estrela as f32);
    g.set_param(forma, ph2d_node_motion_shape::param::SIZE, 0.9);
    let dup = g.add_node("motion.duplicator");
    let roda = g.add_node("motion.rotate");
    g.set_param(roda, "angle", 23.0);
    let out = g.add_node("motion.output");
    for (de, para, porta) in [
        (forma, dup, 0u16),
        (grade, dup, 1),
        (dup, roda, 0),
        (roda, out, 0),
    ] {
        g.connect(Edge {
            from: (de, 0),
            to: (para, porta),
            delayed: false,
        })
        .expect("fio");
    }
    m.sinks = vec![out];
    m
}

/// ⭐⭐⭐ **A rota do dispositivo desenha as mesmas estrelas que a da CPU**, e cala-as nas sprites.
///
/// **A barra, MEDIDA** (2026-09-29, RTX, `36` estrelas rodadas a `23°`): as duas rotas usam o
/// MESMO passe e as MESMAS geometrias, e só as cópias diferem — no seno do `rot` (hardware contra
/// `sin_cos`) e na ordem das somas da posição.
///
/// | alfa máx. | cor máx. | pixels (CPU / dispositivo) | pixels `> 16` |
/// |---:|---:|---|---:|
/// | `1` | `0` | `45 172` / `45 172` | `0` |
///
/// ⇒ a barra é `4` no alfa e na cor (quatro vezes o ULP de cobertura medido) e **zero** pixels que
/// se vejam — um erro de pose (o `basis` com o sinal trocado, um `anchor` sem pivô) move a borda
/// inteira de uma estrela, que são dezenas de pixels acima de `16`.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn as_formas_pela_rota_do_dispositivo_desenham_o_que_a_cpu_desenha() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    // ── A CPU: a bomba baixa as cópias, a placa sobe-as.
    let mut cpu = estado();
    cpu.gpu_enabled = false;
    crate::motion_externals::publish_all(&mut cpu, TIQUE as f64 * DT);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&cpu.doc.graph, &cpu.registry);
    for tick in 0..=TIQUE {
        cpu.pump.advance_or_scrub_scoped(
            &cpu.doc.graph,
            &cpu.registry,
            &cpu.sinks,
            tick,
            |t| t as f64 * DT,
            cpu.default_uv_rect,
            cpu.default_size,
            &scopes,
        );
    }
    let n_cpu = cpu.pump.vector_instances.len();
    assert_eq!(n_cpu, 36, "a CPU baixa as 36 estrelas");
    let mut placa_cpu = PlacaDeFormas::default();
    assert!(placa_cpu.decide(
        true,
        &cpu.pump.vector_instances,
        &cpu.shape_store,
        &mut cpu.placa_geometrias,
        camara(),
    ));
    placa_cpu
        .desenha(&gpu, (LADO, LADO), &cpu.placa_geometrias, None)
        .expect("a placa da CPU desenha");
    let da_cpu = le_a_camada(&gpu, &placa_cpu);

    // ── O DISPOSITIVO: a ponte coze, e a placa lê o buffer do cozimento.
    let mut m = estado();
    crate::motion_externals::publish_all(&mut m, TIQUE as f64 * DT);
    let saida = cook_gpu(&mut m, &gpu, TIQUE, DT, &scopes);
    assert!(
        matches!(saida, GpuOutcome::Handled) && m.gpu_live,
        "a cena de formas tem de ir à placa — a rota disse {:?}",
        m.route_said
    );
    assert!(
        !m.formas_no_dispositivo.is_empty(),
        "a ponte anota as formas vivas"
    );
    let formas = m.gpu_cook.formas().expect("o cozimento escreveu as formas");
    assert_eq!(formas.len(), 36, "uma cópia por estrela");
    // ⚠️ As sprites CALARAM as formas: todo quad do buffer é degenerado.
    let sprites = ph2d_gpu_cook::read_instances(&gpu, m.gpu_cook.instances().expect("sprites"));
    assert!(
        sprites
            .iter()
            .all(|s| s.size == [0.0, 0.0] && s.opacity == 0.0),
        "uma estrela no buffer das sprites é um quadrado de átlas por baixo dela"
    );
    let mut placa_dev = PlacaDeFormas::default();
    assert!(placa_dev.decide_do_dispositivo(
        true,
        formas.len(),
        &m.formas_no_dispositivo,
        &m.shape_store,
        &mut m.placa_geometrias,
        camara(),
    ));
    let buffer = m.gpu_cook.formas().map(|f| f.buffer());
    placa_dev
        .desenha(&gpu, (LADO, LADO), &m.placa_geometrias, buffer)
        .expect("a placa do dispositivo desenha");
    let do_dev = le_a_camada(&gpu, &placa_dev);

    let (alfa, cor, nc, nd, fora) = compara(&da_cpu, &do_dev);
    eprintln!(
        "formas pelo dispositivo: alfa max {alfa} · cor max {cor} · pixels cpu {nc} / dispositivo \
         {nd} · pixels > 16: {fora}"
    );
    assert!(
        nc > 5_000,
        "a cena pinta estrelas a sério (pintou {nc} pixels)"
    );
    assert!(alfa <= ALFA_MAX && cor <= COR_MAX, "alfa {alfa} cor {cor}");
    // A barra é ZERO: com `usize` um `<=` seria um `==` disfarçado, e o clippy di-lo.
    assert_eq!(fora, FORA_MAX, "{fora} pixels desviam mais de 16");
}

/// As barras do veredito — ver a tabela no doc do gate.
const ALFA_MAX: u8 = 4;
const COR_MAX: u8 = 4;
const FORA_MAX: usize = 0;
