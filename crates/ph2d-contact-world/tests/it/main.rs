//! ⭐⭐ **Os bytes que o mundo DECLARA cobrem a cópia dele** (doc 121 §9.20) — o anel do recuo cobra
//! por eles (ADR-0137), e um orçamento que acredita usar menos do que usa estoura calado. Medido
//! com o alocador que CONTA (`dhat`, como os gates HR-3 da casa) — num binário de teste próprio.

use ph2d_contact::obstaculo::{self, FormaFixa};
use ph2d_contact_world::{Estado, Mundo, Pedido, passo};
use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, Column, Stream};
use ph2d_nodegraph::cook::Memo;
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
fn the_declared_bytes_cover_the_copy() {
    const DT: f32 = 1.0 / 60.0;
    let lado = 20_usize;
    let n = lado * lado;
    #[expect(clippy::cast_precision_loss, reason = "índices pequenos")]
    let mut p: Vec<[f32; 2]> = (0..n)
        .map(|i| [(i % lado) as f32 * 0.23, 0.2 + (i / lado) as f32 * 0.23])
        .collect();
    let mut v = vec![[0.0_f32, 0.0]; n];
    let mut mundo: Option<Mundo> = None;
    for k in 1..=30_u16 {
        let t = f32::from(k) * DT;
        #[expect(clippy::cast_precision_loss, reason = "um id pequeno")]
        let mut s = Stream::new(n)
            .with("P", Column::Vec2(p.clone()))
            .with("vel", Column::Vec2(v.clone()))
            .with("id", Column::Scalar((0..n).map(|i| i as f32).collect()))
            .with("sim_t", Column::Scalar(vec![t - DT; n]))
            .with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![[0.11, 0.11]; n]));
        obstaculo::declara(
            &mut s,
            1,
            FormaFixa::Plano {
                normal: [0.0, 1.0],
                altura: 0.0,
            },
            0.6,
            &vec![0.0; n],
        );
        let antes = p.clone();
        for x in &mut v {
            x[1] -= 4.0 * DT;
        }
        let rot0 = vec![0.0_f32; n];
        let (mut rot, mut spin) = (rot0.clone(), vec![0.0; n]);
        let _ = passo(
            &mut mundo,
            &Pedido {
                state: &s,
                pesos: &vec![1.0; n],
                dt: &vec![DT; n],
                sim_t: Some(&vec![t - DT; n]),
                playhead: t,
            },
            &mut Estado {
                antes: &antes,
                rot_antes: &rot0,
                p: &mut p,
                vel: &mut v,
                rot: &mut rot,
                spin: &mut spin,
            },
        );
    }
    let mundo = mundo.expect("há mundo");
    let _perfil = dhat::Profiler::builder().testing().build();
    let antes = dhat::HeapStats::get();
    let copia = mundo.clone();
    let depois = dhat::HeapStats::get();
    drop(copia);
    let medidos = (depois.total_bytes - antes.total_bytes) as usize;
    let declarados = mundo.approx_bytes();
    eprintln!("  {n} pecas: declarados {declarados} B · a copia alocou {medidos} B");
    assert!(
        declarados >= medidos,
        "o mundo declara {declarados} bytes e a cópia alocou {medidos}"
    );
    assert!(
        declarados <= medidos * 4,
        "e não pode inflar sem medida: {declarados} contra {medidos}"
    );
}
