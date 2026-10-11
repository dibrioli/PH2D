//! ⭐⭐ **A cena 7 — o traço do Flip bloqueia a luz como a imagem** (W5e; plano §2.4: a C0 captura o
//! Flip). Os discos de tinta entram no documento do Flip pela porta do smoke; a entidade do objeto
//! (a que a sincronização do Flip cria) ganha o «bloqueia a luz»; e pelo caminho do produto (a
//! `CamadaDeOclusao` do Flip, o rasterizador do traço numa camada da grade) cada disco tem as FACES e
//! a luz dele bate com a da IMAGEM gêmea no espelho em relação à lâmpada — a régua da cena 6.
//!
//! Controlo na MESMA corrida: a mesma cena sem a família do Flip na luz (a W5d) — o traço não
//! bloqueia, a face de trás recebe tanto quanto a da lâmpada.

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro, smoke};
use ph2d_core::Playhead;
use ph2d_ecs::{FlipObjectRef, GlobalTransform, Name, PresentWorld, SimRef, Transform};
use ph2d_flip::FlipDoc;
use ph2d_flip_entities::entities::FlipEntityMap;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_render::{DISC_TILE_PX, Sprite, disc_tile_pixels};

use crate::comum::Fluencia;

/// Os pontos das faces ficam a esta fração do DIÂMETRO menor, de cada lado do centro (a da cena 6).
const PROFUNDIDADE: f64 = 0.35;
/// A face virada para a lâmpada é pelo menos isto × a de trás (o da cena 6).
pub(crate) const CONTRASTE_DAS_FACES: f64 = 10.0;

/// O que a régua do ESPELHO lê de uma forma desenhada e da imagem gêmea dela (`y → −y`, a lâmpada na
/// origem): as faces (a da lâmpada e a de trás, `[desenho, imagem]`) e a variação ao longo de anéis
/// dentro e logo fora, relativa (um risco ou uma faceta é variação que a imagem não tem).
pub(crate) struct Espelho {
    pub frente: [f64; 2],
    pub tras: [f64; 2],
    pub variacao: f64,
}

/// A régua do espelho (cenas 6 e 7) — `centro` e `semi` do DESENHO; a imagem está em `[x, −y]`.
pub(crate) fn espelho(f: &Fluencia, centro: [f64; 2], semi: [f64; 2]) -> Espelho {
    let k = PROFUNDIDADE * 2.0 * semi[0].min(semi[1]);
    let mut frente = [0.0; 2];
    let mut tras = [0.0; 2];
    for (i, c) in [centro, [centro[0], -centro[1]]].into_iter().enumerate() {
        let n = c[0].hypot(c[1]);
        let u = [-c[0] / n, -c[1] / n];
        frente[i] = f.amostra([c[0] + u[0] * k, c[1] + u[1] * k]);
        tras[i] = f.amostra([c[0] - u[0] * k, c[1] - u[1] * k]);
    }
    let mut tv = [0.0_f64; 2];
    for (f_anel, fora) in [(0.35, 0.0), (0.65, 0.0), (0.9, 0.0), (1.0, 0.03)] {
        let (d, im): (Vec<f64>, Vec<f64>) = (0..720)
            .map(|k| {
                let t = std::f64::consts::TAU * f64::from(k) / 720.0;
                let (sn, cs) = t.sin_cos();
                let n = (semi[1] * cs).hypot(semi[0] * sn);
                let p = [
                    centro[0] + f_anel * semi[0] * cs + fora * semi[1] * cs / n,
                    centro[1] + f_anel * semi[1] * sn + fora * semi[0] * sn / n,
                ];
                (f.amostra(p), f.amostra([p[0], -p[1]]))
            })
            .unzip();
        for k in 0..720 {
            let em = |v: &[f64], j: isize| v[(k as isize + j).rem_euclid(720) as usize];
            tv[0] += (em(&d, 1) - em(&d, 0)).abs();
            tv[1] += (em(&im, 1) - em(&im, 0)).abs();
        }
    }
    Espelho {
        frente,
        tras,
        variacao: (tv[0] - tv[1]) / tv[1],
    }
}

/// A face da lâmpada do TRAÇO contra a da IMAGEM, relativo — MEDIDO (RTX 5060 Ti, `DEGRAU_PADRAO`, 30
/// quadros com o rebote): o disco grande −0,55 %, o pequeno +0,19 %; as faces 45 × e 37 ×. A tolerância
/// é a da cena 6.
const TOL_TRACO_IMAGEM: f64 = 0.01;
/// A variação dos anéis do traço acima da da imagem — MEDIDO: +3,3 % e +2,0 % (a da cena 6).
const TOL_TV_TRACO_IMAGEM: f64 = 0.08;

#[test]
fn a_cena_7_ensina_o_traco_do_flip() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 7));
    assert!(!smoke::encenar(&mut World::new(), 8), "não há cena 8");
    assert_eq!(smoke::ROUTERS[0].max_level, 7, "o maior nível do `match`");
    let mut doc = FlipDoc::default();
    assert!(
        smoke::semear_flip(&mut doc, 6).is_none(),
        "só a 7 semeia o Flip"
    );
    let id = smoke::semear_flip(&mut doc, 7).expect("o objeto do traço");
    assert_eq!(
        smoke::marcar_flip(&mut w, &doc, id),
        Some(id),
        "antes da sincronização do Flip não há entidade a marcar"
    );
    // A sincronização do Flip (`entities::sync`): a entidade do objeto, com o nome dele.
    let e = w
        .spawn((
            Transform::IDENTITY,
            Name::new(smoke::NOME_DO_TRACO),
            FlipObjectRef(id.0),
        ))
        .id();
    assert_eq!(smoke::marcar_flip(&mut w, &doc, id), None);
    let cena = cena_de_luz(&w);
    assert_eq!(cena.outros.len(), 1, "o traço bloqueia");
    assert!(cena.outros[0].reflete, "e recebe a luz");

    let mut r = crate::comum::renderer(&gpu);
    let disco: Vec<u8> = disc_tile_pixels(DISC_TILE_PX)
        .into_iter()
        .flat_map(|p| p.0)
        .collect();
    let disco = r
        .acquire_individual(DISC_TILE_PX, DISC_TILE_PX, &disco)
        .expect("textura do disco");
    let mut present = PresentWorld::new();
    let mut q = w.query_filtered::<(bevy_ecs::entity::Entity, &Sprite, &GlobalTransform), bevy_ecs::query::With<ph2d_light2d_ecs::BloqueiaLuz>>();
    for (e, s, g) in q.iter(&w) {
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let mut i = crate::comum::quad(disco, [tx, ty]);
        i.size = s.size;
        i.anchor = s.anchor;
        i.basis = [ax, ay, bx, by];
        present.world_mut().spawn((i, SimRef(e)));
    }
    let afim = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(1920, 1080));
    let mut entidades = FlipEntityMap::new();
    entidades.insert(id, e.to_bits());
    let playhead = Playhead::default();
    let mut composite = None;
    let medir = |com_flip: bool,
                 composite: &mut Option<ph2d_app_flip::pass::FlipComposite>,
                 present: &mut PresentWorld| {
        let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
        for _ in 0..30 {
            assert!(l.preparar(&w, afim, (1920, 1080)));
            let mut flip = ph2d_app_flip::pass::luz::OclusaoDoFlip {
                doc: &doc,
                composite,
                models: &[],
                playhead: &playhead,
                entidades: &entidades,
            };
            if com_flip {
                l.gravar_com(&gpu, present, &r, &mut [&mut flip]);
            } else {
                l.gravar(&gpu, present, &r);
            }
        }
        crate::comum::fluencia_lida(&gpu, l.solver().expect("solver"))
    };
    let sem = medir(false, &mut composite, &mut present);
    let f = medir(true, &mut composite, &mut present);
    let mut falhas = Vec::new();
    for d in &smoke::DISCOS_DE_TINTA {
        let semi = [d.raio; 2];
        let controlo = espelho(&sem, d.centro, semi);
        assert!(
            controlo.frente[0] < CONTRASTE_DAS_FACES * controlo.tras[0],
            "controlo: sem o Flip na luz o {} não devia ter faces ({:.3} × {:.3})",
            d.nome,
            controlo.frente[0],
            controlo.tras[0]
        );
        let m = espelho(&f, d.centro, semi);
        let paridade = (m.frente[0] - m.frente[1]) / m.frente[1];
        eprintln!(
            "cena 7 · {}: faces traço {:.3} × {:.3} · imagem {:.3} × {:.3} · traço × imagem {:+.2} % · anéis {:+.1} %",
            d.nome,
            m.frente[0],
            m.tras[0],
            m.frente[1],
            m.tras[1],
            100.0 * paridade,
            100.0 * m.variacao
        );
        if m.frente[0] < CONTRASTE_DAS_FACES * m.tras[0] {
            falhas.push(format!("{} sem faces", d.nome));
        }
        if paridade.abs() > TOL_TRACO_IMAGEM {
            falhas.push(format!("{}: traço × imagem {paridade:+.3}", d.nome));
        }
        if m.variacao > TOL_TV_TRACO_IMAGEM {
            falhas.push(format!("{}: anéis {:+.3}", d.nome, m.variacao));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join(" · "));
}
