//! ⭐⭐⭐⭐ **Os gates da LUZ ENCOSTADA À PEÇA** — o report do dono de 2026-09-24 (foto da cena `=28`
//! com a lâmpada dentro de um nó: *«ao aproximar a luz do objeto resultados muito ruins de render»*).
//! Os dois defeitos eram ANÉIS duros no chão, e nenhum gate os via porque todos mediam valores num
//! ponto e nunca a CONTINUIDADE de um ponto para o vizinho.

use crate::{Orbit, Sharpness, Stencil};
use ph2d_field::{FieldDoc, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;

fn peca(p: Primitive) -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(p, Xform::IDENTITY)],
        ph2d_field::NodeId(0),
    )
    .expect("a peça")
}

/// O maior salto entre dois vizinhos de uma linha de amostras.
fn maior_salto(v: &[f32]) -> f32 {
    v.windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max)
}

/// ⭐⭐⭐⭐ **A SOMBRA DE UMA LUZ NO BURACO DE UM TORO é CONTÍNUA no chão** — ver a
/// `march::march_visibility` (`ate_a_cerca`). Parar no primeiro passo que ultrapassa a luz deixava a
/// última amostra num sítio que depende da FASE dos passos, e perto da luz é ali que o raio passa
/// rente ao tubo: um anel duro por salto de fase.
///
/// ⭐ **O CONTROLO vive dentro:** a MESMA marcha pela porta do cone (`march_cone_to`, que continua a
/// parar no passo) sobre os MESMOS raios — sem ele a barra podia estar a medir uma fixtura sem anéis.
#[test]
fn a_sombra_de_uma_luz_encostada_nao_desenha_aneis_no_chao() {
    let doc = peca(Primitive::Torus {
        major: 0.2,
        minor: 0.085,
    });
    let reg = Registry::new();
    let shape = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
    let cam = Orbit::default();
    let scene = crate::march::Scene {
        shape: &shape,
        cam: &cam,
        basis: cam.basis(),
        sharp: Sharpness::for_frame(cam.half_extent, 1080),
        clip: None,
        step: ph2d_field_eval::safe_march_step(&doc),
        shrink: ph2d_field_eval::field_shrink(&doc, &reg),
        stencil: Stencil::Tetra4,
    };
    // A luz no buraco, a `0,025` do tubo — é a distância que faz o raio passar RENTE ao tubo no fim
    // (medido: com a luz no centro do buraco as duas leis saltam `0,004`, e a fixtura não conteria
    // o fenómeno). O chão por baixo, varrido numa linha.
    let luz = [0.08f32, 0.0, 0.02];
    let n = 3000;
    let mut origens = Vec::with_capacity(n);
    let mut dirs = Vec::with_capacity(n);
    let mut cercas = Vec::with_capacity(n);
    for i in 0..n {
        #[allow(clippy::cast_precision_loss)]
        let x = -1.5 + 3.0 * i as f32 / (n - 1) as f32;
        let q = [x, -0.35, 0.3];
        let d = [luz[0] - q[0], luz[1] - q[1], luz[2] - q[2]];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        origens.push(q);
        dirs.push([d[0] / dist, d[1] / dist, d[2] / dist]);
        cercas.push(dist);
    }
    let nova = crate::march::march_shadow_to(&scene, &origens, &dirs, &cercas, 8.0);
    let antiga = crate::march::march_cone_to(&scene, &origens, &dirs, &cercas, &vec![8.0; n]);
    let (s_nova, s_antiga) = (maior_salto(&nova), maior_salto(&antiga));
    println!("maior salto: lei nova {s_nova:.4} · parar no passo {s_antiga:.4}");
    // Medido: parar no passo salta `0,4335`, a lei nova `0,0052`.
    assert!(
        s_antiga > 0.05,
        "CONTROLO: a paragem no passo tem de desenhar anéis nesta fixtura (salto {s_antiga:.4})"
    );
    assert!(
        s_nova < 0.02,
        "a sombra da luz encostada desenha anéis: salto {s_nova:.4} entre dois vizinhos"
    );
}

/// ⭐⭐⭐⭐ **O CÉU DO CHÃO é CONTÍNUO à volta de uma peça cujo campo NÃO é exacto** — ver a secção
/// da cerca no doc do `ground::ground_sky`. O nó de toro (uma forma por fórmula) SUBESTIMA a
/// distância, e a cerca da bola punha o termo de cada amostra a zero de repente: seis anéis duros.
///
/// ⭐ **O CONTROLO:** a linha tem de passar por chão que a peça escurece, senão o gate mediria um céu
/// constante e seria verde por vácuo.
#[test]
fn o_ceu_do_chao_nao_desenha_aneis_a_volta_de_um_campo_que_nao_e_exacto() {
    let (radius, tube, winds, loops) = (0.20f32, 0.085f32, 2u32, 3u32);
    let doc = peca(Primitive::TorusKnot {
        radius,
        tube,
        cord: ph2d_field::knot_cord_ceiling(radius, tube, winds, loops) * 0.85,
        winds,
        loops,
    });
    let reg = Registry::new();
    let cam = Orbit::default();
    let chao = crate::lowest_point(&doc, &reg).expect("o chão");
    let n = 3000;
    let pontos: Vec<Option<[f32; 3]>> = (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let x = 3.0 * i as f32 / (n - 1) as f32;
            Some([x, chao, 0.07])
        })
        .collect();
    let ceu = crate::ground::ground_sky(&doc, &reg, &cam, &pontos);
    let minimo = ceu.iter().copied().fold(1.0f32, f32::min);
    let salto = maior_salto(&ceu);
    println!("céu mínimo {minimo:.4} · maior salto {salto:.4}");
    assert!(
        minimo < 0.9,
        "CONTROLO: a linha tem de passar por chão escurecido pela peça (mínimo {minimo:.4})"
    );
    assert!(
        salto < 0.01,
        "o céu do chão salta {salto:.4} entre dois vizinhos — a cerca voltou a ser um degrau"
    );
}
