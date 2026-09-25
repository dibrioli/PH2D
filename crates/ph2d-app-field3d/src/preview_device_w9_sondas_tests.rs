//! ⭐⭐⭐⭐ **OS GATES DAS SONDAS GUARDADAS NA PLACA** — a cura do travão ao girar
//! (`ph2d_field_gpu::sondas_na_placa`, report do dono de 2026-09-24: *«travamentos ao rotacionar a
//! tela continuam»*).
//!
//! Medido (`diag_o_assente_com_as_sondas_guardadas`, `1920×1080`): no nó de toro da cena `=28` o
//! quadro ASSENTE custa `252,3 ms` com as sondas frias e `84,4` com elas guardadas — contra `80,7` do
//! quadro de MOVIMENTO. ⇒ com a mão a orbitar, uma hesitação deixa de pôr `~170 ms` de trabalho
//! que a placa não cancela à frente do quadro seguinte.
//!
//! ⛔ A mesma forma de régua da cache do chão: *a saída é a mesma nas duas rotas e a economia é
//! invisível a toda régua de valor* ⇒ cada gate afirma o PIXEL **e** a CONTA.

use super::super::*;

/// A cena de omissão destes gates: o nó de toro da cena `=28`, onde a assadura custa mais.
fn cena() -> (ph2d_field::FieldDoc, ph2d_field_eval::hybrid::Registry) {
    (crate::smoke::scene(28), crate::smoke::sampled_registry())
}

/// ⚠️ **FIXA em MUNDO** — a `tests_lampada` segue a câmera, e orbitar trocaria a chave (o vácuo
/// que a 1.ª redacção do gate da cache do chão já pagou).
const LUZ_A: [ph2d_field_render::PointLamp; 1] = [ph2d_field_render::PointLamp {
    world: [1.6, 2.4, 1.2],
    radiance_at_one: [7.0, 7.0, 7.0],
}];

fn pinta_em(
    t: &crate::gpu_frame::SharedTracer,
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    azimute: f32,
    luz: &[ph2d_field_render::PointLamp],
    ricochete: bool,
) -> Vec<u8> {
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let cam = ph2d_field_render::Orbit {
        rotation: ph2d_field_render::Orbit::from_yaw_pitch(azimute, 0.5).rotation,
        ..ph2d_field_render::Orbit::default()
    };
    crate::gpu_frame::paint_com(
        t,
        doc,
        reg,
        &cam,
        luz,
        &surfaces,
        &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
        [0, 0, 0, 0],
        None,
        LW,
        LH,
        true,
        crate::gpu_frame::Sonda {
            ricochete,
            ..crate::gpu_frame::Sonda::default()
        },
    )
    .expect("o pintor")
    .rgba
}

fn assadas(t: &crate::gpu_frame::SharedTracer) -> usize {
    t.lock().expect("o traçador").sondas_assadas()
}

/// ⭐⭐⭐⭐ **AS SONDAS GUARDADAS NÃO MUDAM A IMAGEM — e orbitar não as reassa.**
///
/// Enche a cache num azimute, pinta noutro (morno), esquece-a e pinta no mesmo (frio). As duas
/// imagens são o mesmo quadro por rotas diferentes.
///
/// ⚠️ **A base da vista CHEGA às sondas, e só como rotação** (o material é avaliado em espaço de
/// vista): em aritmética exacta os produtos internos são os mesmos e em `f32` diferem por ULP.
/// ⇒ a régua é `≤ 1` nível, e não a igualdade ao bit.
///
/// ⭐ **O CONTROLO é que o ricochete APARECE nesta fixtura** — sem ele a igualdade podia estar a
/// comparar dois quadros onde as sondas não pesam nada.
#[test]
#[ignore = "precisa de GPU"]
fn as_sondas_guardadas_nao_mudam_a_imagem_e_orbitar_nao_as_reassa() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, reg) = cena();
    let _ = pinta_em(t, &doc, &reg, 0.0, &LUZ_A, true);
    let antes = assadas(t);
    let morno = pinta_em(t, &doc, &reg, 1.0, &LUZ_A, true);
    let ao_orbitar = assadas(t) - antes;
    t.lock().expect("o traçador").esquece_as_sondas();
    let frio = pinta_em(t, &doc, &reg, 1.0, &LUZ_A, true);
    let sem_ricochete = pinta_em(t, &doc, &reg, 1.0, &LUZ_A, false);

    assert_eq!(
        ao_orbitar, 0,
        "orbitar reassou as sondas — é a hesitação a meio de uma órbita que paga os ~170 ms, e a \
         cache existe para ela"
    );
    let mexidos = frio
        .iter()
        .zip(&sem_ricochete)
        .filter(|(a, b)| a.abs_diff(**b) > 1)
        .count();
    assert!(
        mexidos > 10_000,
        "CONTROLO: o ricochete só move {mexidos} canais nesta fixtura — a igualdade abaixo não \
         afirmaria nada sobre as sondas"
    );
    let (mut acima, mut pior) = (0usize, 0u8);
    for (a, b) in morno.iter().zip(&frio) {
        let d = a.abs_diff(*b);
        pior = pior.max(d);
        if d > 1 {
            acima += 1;
        }
    }
    println!(
        "morno contra frio: {acima} canais acima de 1 nível · pior {pior} · ricochete move {mexidos}"
    );
    assert_eq!(
        acima, 0,
        "as sondas guardadas de outro azimute mudaram a imagem (pior {pior}) — a chave está a \
         excluir um eixo que CHEGA à assadura"
    );
}

/// ⭐⭐⭐⭐ **E A CHAVE FALTA QUANDO DEVE: trocar a luz ou a peça reassa.**
///
/// ⛔ Sem esta metade a cura lê-se como *«reaproveita sempre»* — e com a luz noutro sítio ela
/// entregaria o ricochete da luz antiga.
#[test]
#[ignore = "precisa de GPU"]
fn trocar_a_luz_ou_a_peca_reassa_as_sondas() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, reg) = cena();
    let luz_b = [ph2d_field_render::PointLamp {
        world: [-1.6, 2.4, -1.2],
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    let _ = pinta_em(t, &doc, &reg, 0.0, &LUZ_A, true);
    let antes = assadas(t);
    let _ = pinta_em(t, &doc, &reg, 0.0, &luz_b, true);
    assert_eq!(assadas(t) - antes, 1, "trocar a LUZ não reassou as sondas");
    // ⚠️⚠️ **As duas peças têm a MESMA bola e a MESMA grade de longe** — e isso é o que a fixtura
    // tem de ter: medido por mutação, trocar a cena `=28` pela `=5` reassava MESMO com a peça fora
    // da chave, porque a bola mudava com ela. *Uma régua que troca duas coisas não diz qual delas a
    // chave vê.* ⇒ o mesmo nó `(p, q)` e `(q, p)`, com o mesmo raio, o mesmo tubo e a mesma corda.
    let no = |winds: u32, loops: u32| {
        let (radius, tube) = (0.5f32, 0.12f32);
        let cord = ph2d_field::knot_cord_ceiling(radius, tube, 2, 3)
            .min(ph2d_field::knot_cord_ceiling(radius, tube, 3, 2))
            * 0.85;
        ph2d_field::FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                ph2d_field::Primitive::TorusKnot {
                    radius,
                    tube,
                    cord,
                    winds,
                    loops,
                },
                ph2d_field::Xform::IDENTITY,
            )],
            ph2d_field::NodeId(0),
        )
        .expect("o nó")
    };
    let (um, outro) = (no(2, 3), no(3, 2));
    let bola = |d: &ph2d_field::FieldDoc| {
        ph2d_field_eval::bounds::bounding_ball(d, &reg).map(|b| (b.center, b.radius))
    };
    assert_eq!(
        bola(&um),
        bola(&outro),
        "FIXTURA: as duas peças têm de partilhar a bola, senão a chave falta por ela"
    );
    let _ = pinta_em(t, &um, &reg, 0.0, &luz_b, true);
    let antes = assadas(t);
    let _ = pinta_em(t, &outro, &reg, 0.0, &luz_b, true);
    assert_eq!(assadas(t) - antes, 1, "trocar a PEÇA não reassou as sondas");
    // E o quadro de MOVIMENTO não assa nada: o ricochete não corre nele.
    let antes = assadas(t);
    let _ = pinta_em(t, &outro, &reg, 1.0, &luz_b, false);
    assert_eq!(
        assadas(t) - antes,
        0,
        "sem ricochete as sondas não se assam"
    );
}
