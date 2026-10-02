//! ⏱️⭐⭐⭐⭐ **AS SONDAS VELHAS A MEXER — quanto erram, contra ir sem ricochete** — a varredura que
//! fixa o [`ph2d_field_gpu::sondas_na_placa::TOLERANCIA_EM_CELULAS`].
//!
//! Um quadro que não pode esperar não assa as sondas (report do dono, 2026-10-01: *«ainda com
//! delay»*; um arrasto re-assava-as a cada quadro, `129 ms` de placa). Ele tem DUAS saídas: ler as
//! guardadas (de antes da edição) ou ir sem ricochete. A sonda arrasta o toro do meio cada vez mais
//! longe e, contra a imagem do MESMO quadro com as sondas acabadas de assar, mede o erro de cada
//! saída — com o deslocamento da grade em células ao lado, que é a grandeza que o produto lê.
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-field3d --lib --release \
//!   -- --ignored diag_as_sondas_velhas_a_mexer --nocapture --test-threads=1
//! ```

use super::*;

fn peca(dy: f32) -> ph2d_field::FieldDoc {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let n = 5usize;
    #[allow(clippy::cast_precision_loss)]
    let mut nos: Vec<ph2d_field::Node> = (0..n)
        .map(|i| {
            ph2d_field_eval::leaf(
                Primitive::Torus {
                    major: 0.16,
                    minor: 0.06,
                },
                Xform::at(i as f32 * 0.3 - 0.6, if i == 2 { dy } else { 0.0 }, 0.0),
            )
        })
        .collect();
    #[allow(clippy::cast_possible_truncation)]
    nos.push(ph2d_field::Node {
        xform: Xform::IDENTITY,
        kind: ph2d_field::NodeKind::Combine {
            op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
            children: (0..n as u32).map(NodeId).collect(),
        },
        mods: Vec::new(),
        verb: None,
    });
    #[allow(clippy::cast_possible_truncation)]
    FieldDoc::new(nos, NodeId(n as u32)).expect("a peça")
}

/// Média e p99 do erro por canal, em bytes, só nos píxeis em que alguma das duas imagens tem peça.
fn erro(a: &[u8], b: &[u8]) -> (f32, u8) {
    let mut v: Vec<u8> = Vec::new();
    for (x, y) in a.chunks(4).zip(b.chunks(4)) {
        if x[3] == 0 && y[3] == 0 {
            continue;
        }
        for c in 0..3 {
            v.push(x[c].abs_diff(y[c]));
        }
    }
    if v.is_empty() {
        return (0.0, 0);
    }
    #[allow(clippy::cast_precision_loss)]
    let media = v.iter().map(|&e| f32::from(e)).sum::<f32>() / v.len() as f32;
    v.sort_unstable();
    (media, v[v.len() * 99 / 100])
}

#[test]
#[ignore = "sonda de diagnóstico: o erro das sondas velhas a mexer"]
fn diag_as_sondas_velhas_a_mexer() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let mats = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &mats,
        owners: None,
    };
    let quadro = |doc: &ph2d_field::FieldDoc, assente: bool| -> Vec<u8> {
        crate::gpu_frame::paint(
            t,
            doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
            [0, 0, 0, 0],
            None,
            LW,
            LH,
            assente,
        )
        .expect("o dispositivo pinta")
        .rgba
    };
    let a = peca(0.0);
    let ba = ph2d_field_eval::bounds::bounding_ball(&a, &reg).expect("a bola");
    let celula = 2.0 * ba.radius * ph2d_field_render::probes::PROBE_MARGIN
        / (ph2d_field_render::probes::PROBE_GRID - 1) as f32;
    println!("\n  {}", contexto());
    println!("  dy     · desloc. (células) · sondas VELHAS média/p99 · SEM ricochete média/p99");
    for dy in [0.01f32, 0.03, 0.06, 0.1, 0.15, 0.2, 0.3] {
        let b = peca(dy);
        let bb = ph2d_field_eval::bounds::bounding_ball(&b, &reg).expect("a bola");
        let dc = (0..3)
            .map(|i| (ba.center[i] - bb.center[i]).abs())
            .fold(0.0f32, f32::max);
        let celulas =
            (dc + (ba.radius - bb.radius).abs() * ph2d_field_render::probes::PROBE_MARGIN) / celula;
        // As VELHAS: as guardadas são as de A, e a tolerância deixa-as servir a B.
        let _ = quadro(&a, false);
        let _ = quadro(&a, true);
        t.lock()
            .expect("o traçador")
            .tolerancia_das_sondas(f32::INFINITY);
        let velhas = quadro(&b, false);
        t.lock()
            .expect("o traçador")
            .tolerancia_das_sondas(ph2d_field_gpu::sondas_na_placa::TOLERANCIA_EM_CELULAS);
        // SEM ricochete: nada guardado.
        t.lock().expect("o traçador").esquece_as_sondas();
        let sem = quadro(&b, false);
        // A VERDADE: as de B acabadas de assar, lidas pelo mesmo quadro de movimento.
        let _ = quadro(&b, true);
        let frescas = quadro(&b, false);
        let (mv, pv) = erro(&velhas, &frescas);
        let (ms, ps) = erro(&sem, &frescas);
        println!("  {dy:<6} · {celulas:17.3} · {mv:10.3} / {pv:3}           · {ms:10.3} / {ps:3}");
    }
    println!();
}
