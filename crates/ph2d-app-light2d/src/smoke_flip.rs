//! ⭐ Cena 7 — **O traço do Flip** (W5e): dois discos de TINTA (um traço de largura = o diâmetro,
//! dureza 1) num objeto Flip e, no espelho de cada um em relação à lâmpada, a IMAGEM gêmea com a mesma
//! cor. O traço bloqueia e recebe a luz como a imagem — o molde da cena 6.
//!
//! O documento do Flip não está no `World`: [`semear_flip`] põe o objeto no documento (a porta da
//! codificação da luz recebe-o) e a entidade dele nasce na sincronização do Flip (`entities::sync`);
//! [`marcar_flip`] dá-lhe o «bloqueia a luz».

use bevy_ecs::world::World;
use ph2d_core::Vec2;
use ph2d_flip::{FlipDoc, FlipObjectId, FlipStroke, Hold, KeyKind, Point, Rgba};
use ph2d_light2d_ecs::BloqueiaLuz;

/// Um disco de tinta da cena 7: o nome, o centro e o raio (metros) e a cor (sRGB).
pub struct DiscoDeTinta {
    pub nome: &'static str,
    pub centro: [f64; 2],
    pub raio: f64,
    pub cor: [u8; 3],
}

/// Os discos de tinta, à altura do espelho da cena 6 (a imagem gêmea fica em `y → −y`).
pub const DISCOS_DE_TINTA: [DiscoDeTinta; 2] = [
    DiscoDeTinta {
        nome: "Disco grande",
        centro: [-2.6, 1.6],
        raio: 0.9,
        cor: [51, 179, 191],
    },
    DiscoDeTinta {
        nome: "Disco pequeno",
        centro: [2.4, 1.6],
        raio: 0.5,
        cor: [217, 140, 64],
    },
];

/// O nome do objeto Flip da cena 7 — o que aparece na Hierarchy.
pub const NOME_DO_TRACO: &str = "Traço do Flip";

/// Cena 7 — o mundo: o chão, a luz ambiente, a lâmpada e as IMAGENS gêmeas; o traço entra pelo
/// documento do Flip ([`semear_flip`]).
pub(crate) fn traco_do_flip(world: &mut World) {
    let gemeas: Vec<_> = DISCOS_DE_TINTA
        .iter()
        .map(|d| (d.nome, [d.centro[0], -d.centro[1]], [d.raio; 2], d.cor))
        .collect();
    super::chao_lampada_e_gemeas(world, &gemeas);
}

/// ⭐ Cena 7 — põe no DOCUMENTO do Flip o objeto com os discos de tinta (só a 7 tem); devolve o id.
pub fn semear_flip(doc: &mut FlipDoc, nivel: u32) -> Option<FlipObjectId> {
    if nivel != 7 {
        return None;
    }
    let id = doc.push_object(NOME_DO_TRACO);
    let obj = doc.object_mut(id)?;
    let camada = obj.add_layer("Tinta");
    let desenho = obj.insert_frame(camada, 0, Hold::Implicit, KeyKind::Keyframe)?;
    let tracos = &mut obj.drawing_mut(desenho)?.strokes;
    for d in &DISCOS_DE_TINTA {
        let [r, g, b] = d.cor.map(super::linear);
        let cor = Rgba([r, g, b, 1.0]);
        let mut s = FlipStroke::new();
        // Um ponto de tinta: dois pontos (quase) no mesmo lugar, largura = o diâmetro.
        for dx in [-5e-4, 5e-4] {
            s.push_point(Point {
                pos: Vec2::new((d.centro[0] + dx) as f32, d.centro[1] as f32),
                width: (2.0 * d.raio) as f32,
                opacity: 1.0,
                color: cor,
            });
        }
        s.hardness = 1.0;
        tracos.push(s);
    }
    Some(id)
}

/// O objeto semeado, se o Flip já lhe criou a entidade, recebe o «bloqueia a luz»; devolve-o de volta
/// se ainda falta (`None` = marcado, ou saiu do documento — um projeto aberto por cima).
pub fn marcar_flip(world: &mut World, doc: &FlipDoc, id: FlipObjectId) -> Option<FlipObjectId> {
    doc.object(id)?;
    let e = world
        .query::<(bevy_ecs::entity::Entity, &ph2d_ecs::FlipObjectRef)>()
        .iter(world)
        .find(|(_, r)| r.0 == id.0)
        .map(|(e, _)| e);
    match e {
        Some(e) => {
            world.entity_mut(e).insert(BloqueiaLuz { opacidade: 1.0 });
            None
        }
        None => Some(id),
    }
}
