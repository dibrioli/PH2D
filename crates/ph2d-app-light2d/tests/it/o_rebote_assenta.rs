//! ⭐⭐ **O rebote assenta em ≤ 10 quadros** (K9 do relatório; o ABERTO 1b da W5c: 11–12). Um rebote
//! por quadro converge à razão de um rebote por quadro; o Baixo faz DOIS (`Degrau::rebotes_por_quadro`,
//! o 2.º repete no mesmo quadro a soma na emissão com a fonte nova, a cascata e a fonte).
//!
//! A régua (a da W5c): a cena 3 na vista de fábrica, o PIOR dos cinco pontos da sala — o quadro em que
//! fica a 1 % do convergido (120 quadros) —, com o controlo de um rebote por quadro.

use bevy_ecs::world::World;
use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::{
    captura_do_quadro, fina_do_quadro, grade_do_quadro, regiao_da_janela, smoke,
};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Solver};
use ph2d_render::Camera2d;

use crate::comum::{fluencia_lida, gpu};

const FHD: (u32, u32) = (1920, 1080);
/// O quadro máximo a 1 % — o K9. MEDIDO (RTX 5060 Ti): 6 com dois rebotes; 11 com um.
const K9: usize = 10;

/// ⭐⭐ **GATE — o rebote do Baixo assenta a 1 % em ≤ [`K9`] quadros**; o controlo (um rebote por quadro)
/// passa dele.
#[test]
fn o_rebote_assenta_em_dez_quadros() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let a = Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let c3 = cena_de_luz(&w);
    let pts = crate::medida_luz2d_w5c::pontos_da_cena_3();
    let d = degrau_do(NivelDaLuz::Baixa);
    assert!(
        d.rebotes_por_quadro > 1,
        "o Baixo faz dois rebotes por quadro"
    );
    let quadros = |n: u32| {
        let cfg = Config {
            fina: fina_do_quadro(a, d),
            captura: captura_do_quadro(a, d),
            holografica: Some(HOLOGRAFICA_PADRAO),
            rebote: true,
            rebotes_por_quadro: n,
            ..Config::padrao_para(&gpu.device, grade_do_quadro(a, FHD, d).expect("grade"))
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
        s.regiao_fina(i, f);
        let mut historia: Vec<Vec<f64>> = Vec::new();
        for _ in 0..120 {
            s.quadro(&gpu, &c3);
            let fl = fluencia_lida(&gpu, &s);
            historia.push(pts.iter().map(|&p| fl.amostra(p)).collect());
        }
        let fim = historia.last().expect("120 quadros").clone();
        let q = historia
            .iter()
            .position(|h| {
                h.iter()
                    .zip(&fim)
                    .all(|(x, y)| (x - y).abs() <= 0.01 * y.abs())
            })
            .map_or(usize::MAX, |q| q + 1);
        (q, fim)
    };
    let ((produto, fim), (controlo, fim1)) = (quadros(d.rebotes_por_quadro), quadros(1));
    // O mesmo ponto fixo: o 2.º rebote repõe a emissão de BASE (a soma é aditiva) — sem a reposição
    // ele convergia, depressa, para OUTRO valor.
    let dif = fim
        .iter()
        .zip(&fim1)
        .map(|(x, y)| (x - y).abs() / y.abs().max(1e-9))
        .fold(0.0, f64::max);
    eprintln!(
        "K9 · Baixo: 1 % no quadro {produto} (um rebote por quadro: {controlo}) · o convergido difere {:.2} %",
        100.0 * dif
    );
    assert!(
        dif <= 0.005,
        "dois rebotes por quadro convergem para OUTRO valor ({:.2} %)",
        100.0 * dif
    );
    assert!(
        controlo > K9,
        "o controlo de um rebote por quadro assenta em {controlo} — a régua não vê o K9"
    );
    assert!(
        produto <= K9,
        "o rebote do Baixo assenta a 1 % só no quadro {produto} (K9: ≤ {K9})"
    );
}
