//! ⭐⭐ **O rebote de FORA da grade** (o ABERTO 1 da W5c, o maior; o dono na cena 5: com zoom forte some
//! o brilho à esquerda do «L»). A grade da luz cobre a vista e uma margem; um refletor fora dela não
//! existe para as cascatas — e com zoom forte o chão, o teto e as paredes da sala ficam fora.
//!
//! Medido (W5d): a mesma vista com a grade ESTICADA sobre a sala inteira (o mesmo texel) levava o erro
//! de −63,6 % a −0,5 % — a causa é o «fora da grade». A cura: o ENTORNO (`LuzDoQuadro`, o
//! `Degrau::entorno`), um 2.º solver mais grosso e 4× mais largo, sem céu e com a emissão da grade da
//! vista apagada (`Solver::mascarar_miolo`): a c0 dele — a luz que vem de fora — soma-se à da vista
//! (`Solver::somar_entorno`).
//!
//! A régua (a da W5c): a cena 3 numa vista de 2 m de altura, a câmera deslocada, um ponto da sala 0,3 m
//! para dentro da borda esquerda da vista; o vermelho (a parede vermelha devolve-o) contra a série do
//! gabarito, pelo caminho do produto (os sprites por texel), com o controlo sem o entorno.

use bevy_ecs::world::World;
use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::{Degrau, LuzDoQuadro, smoke};
use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_ref::{OpcoesDoRebote, Rebote};
use ph2d_render::{Camera2d, Sprite};
use vello::kurbo::Point;

use crate::comum::{fluencia_lida, gpu, quad, renderer};

const FHD: (u32, u32) = (1920, 1080);
/// O erro do vermelho com o entorno — MEDIDO (RTX 5060 Ti, nível Médio, 80 quadros): o pior 0,9 %
/// (a câmera em x = 2); sem o entorno −53,7 a −63,6 %.
const TETO: f64 = 0.03;

/// ⭐⭐ **GATE — o rebote chega de fora da grade**: com a câmera em x = 0 e x = 2 (a parede vermelha
/// fora da grade), o vermelho junto à borda esquerda fica a [`TETO`] da série; o controlo sem o
/// entorno erra mais de 40 %.
#[test]
fn o_rebote_chega_de_fora_da_grade() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let c3 = cena_de_luz(&w);
    let serie = Rebote::resolver(
        &c3,
        OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 200,
        },
    );
    let mut r = renderer(&gpu);
    let branca = r.acquire_individual(1, 1, &[255; 4]).expect("textura");
    let mut present = PresentWorld::new();
    let mut q = w.query::<(bevy_ecs::entity::Entity, &Sprite, &GlobalTransform)>();
    for (e, s, g) in q.iter(&w) {
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let mut i = quad(branca, [tx, ty]);
        i.size = s.size;
        i.anchor = s.anchor;
        i.basis = [ax, ay, bx, by];
        present.world_mut().spawn((i, SimRef(e)));
    }
    let mut falhas = Vec::new();
    let mut controlo: f64 = 0.0;
    for centro_x in [0.0_f32, 2.0] {
        let a = Camera2d::new([centro_x, 0.0], 2.0)
            .world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
        let esquerda = (a.inverse() * Point::new(0.0, 540.0)).x;
        let p = [esquerda.max(-2.7) + 0.3, 0.0];
        let gab = 2.0 * std::f64::consts::PI * serie.luz_em(p)[0];
        let mut ler = |d: Degrau| {
            let mut l = LuzDoQuadro::com_degrau(d);
            for _ in 0..80 {
                assert!(l.preparar(&w, a, FHD));
                l.gravar(&gpu, &mut present, &r);
            }
            let f = fluencia_lida(&gpu, l.solver().expect("solver"));
            (f.amostra_canal(p, 0) - gab) / gab
        };
        let d = degrau_do(NivelDaLuz::Media);
        let (com, sem) = (ler(d), ler(Degrau { entorno: 0.0, ..d }));
        eprintln!(
            "rebote de fora · câmera em x = {centro_x} · ponto {:.2}: com o entorno {com:+.3} · sem {sem:+.3}",
            p[0]
        );
        controlo = controlo.max(sem.abs());
        if com.abs() > TETO {
            falhas.push(format!("x = {centro_x}: {com:+.3}"));
        }
    }
    assert!(
        controlo > 0.4,
        "o controlo sem o entorno não mostra o rebote que falta ({controlo:.3}) — a régua não o vê"
    );
    assert!(
        falhas.is_empty(),
        "o rebote de fora da grade não chega (teto {TETO}): {}",
        falhas.join(" · ")
    );
}

/// ⭐⭐ **GATE — sem nada fora da vista, o entorno não acrescenta nada**: um corpo que brilha no meio da
/// vista (nas cascatas), sem céu; a luz em volta dele com o entorno e sem ele é a mesma. O entorno não
/// recolhe a emissão do MIOLO (a grade da vista: `Solver::mascarar_miolo`) — sem a máscara, ele contava
/// outra vez a luz que a vista já tem.
#[test]
fn o_entorno_nao_conta_duas_vezes_o_que_esta_na_vista() {
    use ph2d_app_light2d::{
        ENTORNO_PADRAO, MARGEM_DO_ENTORNO, captura_do_quadro, fina_do_quadro, grade_do_quadro,
        regiao_da_janela,
    };
    use ph2d_light2d_ecs::{CenaDeLuz, Emissor};
    use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Solver};
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let cena = CenaDeLuz {
        emissores: vec![Emissor::Poligono {
            vertices: (0..24)
                .map(|k| {
                    let a = std::f32::consts::TAU * k as f32 / 24.0;
                    [0.3 * a.cos(), 0.3 * a.sin()]
                })
                .collect(),
            radiancia: [1.0; 3],
            alcance_m: f32::INFINITY,
        }],
        luzes: 1,
        ..Default::default()
    };
    let a = Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
    let d = degrau_do(NivelDaLuz::Baixa);
    let g = grade_do_quadro(a, FHD, d).expect("grade");
    let ler = |com: bool| {
        let mut s = Solver::novo(
            &gpu,
            Config {
                fina: fina_do_quadro(a, d),
                captura: captura_do_quadro(a, d),
                holografica: Some(HOLOGRAFICA_PADRAO),
                ..Config::padrao_para(&gpu.device, g)
            },
        )
        .expect("solver");
        let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
        s.regiao_fina(i, f);
        let mut e = com.then(|| {
            let de = Degrau {
                c0_px: d.c0_px * ENTORNO_PADRAO,
                margem: MARGEM_DO_ENTORNO,
                ..d
            };
            let ge = grade_do_quadro(a, FHD, de).expect("grade do entorno");
            let mut e = Solver::novo(
                &gpu,
                Config {
                    holografica: Some(HOLOGRAFICA_PADRAO),
                    ..Config::padrao_para(&gpu.device, ge)
                },
            )
            .expect("o entorno");
            e.regiao_fina([0; 2], [0; 2]);
            let gv = s.config().grade;
            assert!(e.mascarar_miolo(
                &gpu,
                Some([
                    gv.origem_m[0],
                    gv.origem_m[1],
                    gv.origem_m[0] + f64::from(gv.largura) * gv.texel_m,
                    gv.origem_m[1] + f64::from(gv.altura) * gv.texel_m,
                ])
            ));
            e
        });
        if let Some(e) = e.as_mut() {
            e.quadro(&gpu, &cena);
        }
        assert!(s.somar_entorno(&gpu, e.as_ref()));
        s.quadro(&gpu, &cena);
        fluencia_lida(&gpu, &s)
    };
    let (com, sem) = (ler(true), ler(false));
    let pior = [0.5_f64, 1.0, 2.0, 3.5]
        .iter()
        .flat_map(|&r| (0..8).map(move |k| (r, f64::from(k) * std::f64::consts::FRAC_PI_4)))
        .map(|(r, t)| {
            let p = [r * t.cos(), r * t.sin()];
            (com.amostra(p) - sem.amostra(p)).abs() / sem.amostra(p).max(1e-6)
        })
        .fold(0.0, f64::max);
    eprintln!("o entorno sem nada fora da vista: a luz difere no máximo {:.3} %", 100.0 * pior);
    assert!(
        pior <= 0.01,
        "o entorno conta outra vez a luz da vista ({:.2} %)",
        100.0 * pior
    );
}
