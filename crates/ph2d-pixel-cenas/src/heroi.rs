//! A cena 26 «O herói na cena» (`docs/PixelArt/P14_plano.md` §8): a fixture `heroi_p14` como Pixel
//! Sprite com corpo dinâmico e a forma da slice `body` (o Add Collider), sobre um chão de corpo
//! estático, uma Luz pontual à esquerda e uma parede à direita, os dois com Blocks Light. A regra
//! de animação «Key held `move_right` → run» é a do cartão Animation; sem ela toca `idle` (Autoplay).
//!
//! Tudo o que aqui nasce o artista monta pela UI: Add ▸ Pixel Sprite, Add ▸ Light, Rigid Body,
//! Slice + Add Collider, Blocks Light e a regra do cartão.

use ph2d_ecs::{Entity, Name, PixelSprite, Transform, World};
use ph2d_light2d_ecs::{BloqueiaLuz, Luz};
use ph2d_physics_ecs::{AnimRule, AnimRules, AnimWhen, BodyKind, LockRotation, RigidBody};
use ph2d_pixel_core::{NewSprite, Op, Patch, Rect, Sprite, apply, slice};

/// A régua da cena: a do projecto por omissão (`DEFAULT_PIXELS_PER_METER`).
pub const PPM: f64 = 100.0;
/// O herói nasce aqui (m) e cai 14 cm até ao chão.
pub const HEROI: [f32; 2] = [0.0, -0.2];
/// A lâmpada, à esquerda do herói pousado e à altura do meio dele: a sombra cai na face da parede.
/// A cena é compacta para o *Frame All* pôr o herói a ~1/4 da altura (o anel do objecto mede 24 px
/// de arte de raio, a qualquer régua).
pub const LAMPADA: [f32; 2] = [-0.28, -0.25];
/// O disco da lâmpada da cena (m): 5 cm, um quinto do de fábrica, para não dominar a vista.
pub const LAMPADA_RAIO_M: f32 = 0.05;
/// A intensidade da cena: metade da de fábrica, porque a lâmpada está a metade da distância
/// (0,28 m contra 0,55 m) — o herói recebe a mesma luz e não satura.
pub const LAMPADA_INTENSIDADE: f32 = std::f32::consts::PI;
/// O chão: 56 × 10 px, centro em (0, −0,4) m ⇒ o topo em −0,35 m.
pub const CHAO: (u32, u32, [f32; 2]) = (56, 10, [0.0, -0.4]);
/// A parede: 8 × 25 px, centro em (0,19, −0,225) m ⇒ a face esquerda em x = 0,15 m, do chão a −0,10 m.
pub const PAREDE: (u32, u32, [f32; 2]) = (8, 25, [0.19, -0.225]);
/// Cores das peças.
pub const COR_CHAO: [u8; 4] = [120, 100, 80, 255];
pub const COR_PAREDE: [u8; 4] = [200, 200, 200, 255];
/// A ação do mapa de teclas que troca para `run` (→ ou D no mapa de fábrica).
pub const ACAO: &str = "move_right";
/// O nome da tag que a regra toca.
pub const RUN: &str = "run";

/// Onde ficam as peças de uma cena do herói.
#[derive(Clone, Copy, Debug)]
pub struct Arranjo {
    pub heroi: [f32; 2],
    pub lampada: [f32; 2],
    pub chao: (u32, u32, [f32; 2]),
    pub parede: (u32, u32, [f32; 2]),
    /// A cena 27: uma «Wall 2» espelhada e o `.ase` do herói à espera do import (`heroi27`).
    pub segunda_parede: bool,
}

/// A cena 26: a lâmpada à esquerda do herói, a parede à direita.
pub const CENA_26: Arranjo = Arranjo {
    heroi: HEROI,
    lampada: LAMPADA,
    chao: CHAO,
    parede: PAREDE,
    segunda_parede: false,
};

/// A cena 27: a lâmpada a MEIO, o herói do Pixel à esquerda dela e o Sprite do motor à direita,
/// cada um com a sua parede atrás — o arranjo é simétrico em x, logo cada um recebe a luz do lado
/// de dentro e faz sombra na sua parede.
pub const CENA_27: Arranjo = Arranjo {
    heroi: [-HEROI_27_X, HEROI[1]],
    lampada: [0.0, LAMPADA[1]],
    chao: (72, 10, [0.0, -0.4]),
    parede: (8, 25, [-PAREDE_27_X, PAREDE.2[1]]),
    segunda_parede: true,
};
/// |x| de cada herói da cena 27 (m): a borda de dentro fica fora do disco da lâmpada.
pub const HEROI_27_X: f32 = 0.13;
/// |x| do centro de cada parede da cena 27 (m): a face de dentro a 0,23 m.
pub const PAREDE_27_X: f32 = 0.27;
/// O nome do `.ase` que a cena 27 grava e importa (o import dá ao Sprite o nome do ficheiro).
pub const NOME_DO_SPRITE: &str = "Hero Sprite";

/// O Sprite do motor que o import nasceu (`e`), vestido como o herói do Pixel: no sítio `at`, com
/// corpo dinâmico, `LockRotation`, Blocks Light e a mesma regra do cartão Animation. A forma de
/// colisão já é a filha que o import ligou (a slice `body`, marcada pelo Add Collider).
pub fn veste_o_do_motor(world: &mut World, e: Entity, at: [f32; 2]) {
    let mut t = world
        .get::<Transform>(e)
        .copied()
        .unwrap_or(Transform::IDENTITY);
    t.translation.x = at[0];
    t.translation.y = at[1];
    world.entity_mut(e).insert((
        t,
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        LockRotation,
        BloqueiaLuz { opacidade: 1.0 },
        AnimRules(vec![AnimRule {
            when: AnimWhen::Action(ACAO.into()),
            play: RUN.into(),
        }]),
    ));
}

/// **O herói das cenas 26 e 27** — o que um artista desenharia para o dono VER a regra: a
/// `heroi_p14` (tags, durações, camada Normal Map) com quadros visivelmente diferentes (olhos que
/// piscam no `idle`, pernas que alternam no `run`, olhos em X no `hit`) e a slice `body` ESTÁVEL
/// (uma key só, o pivô nos pés em todos os quadros). A `heroi_p14` dos gates continua com a key que
/// encolhe e o pivô que muda (P14 §7). Ver «Smoke do dono 1 — pulinhos» no plano.
#[must_use]
pub fn heroi_da_cena() -> Sprite {
    use ph2d_pixel_cena::fixture as fx;
    let mut s = fx::heroi_p14();
    let body = fx::body(&s);
    let mut lista = s.slices().to_vec();
    if let Some(b) = lista.iter_mut().find(|c| c.id == body) {
        b.keys.truncate(1);
    }
    apply(&mut s, Op::Slices(lista)).expect("a slice estável");
    let r = s.bounds();
    let cor = s.layers()[0].id;
    let ids: Vec<_> = s.frames().iter().map(|f| f.id).collect();
    for (i, f) in ids.into_iter().enumerate() {
        let pixels = (0..r.h)
            .flat_map(|y| (0..r.w).map(move |x| (x, y)))
            .flat_map(|(x, y)| {
                fx::hemisferio(x, y).map_or([0; 4], |_| {
                    if detalhe(i, x, y) {
                        COR_DETALHE
                    } else {
                        fx::COR
                    }
                })
            })
            .collect();
        apply(
            &mut s,
            Op::Patch(Patch {
                layer: cor,
                frame: f,
                bbox: Some(r),
                rect: r,
                pixels,
            }),
        )
        .expect("o quadro desenhado");
    }
    s
}

/// A cor dos olhos e das pernas.
pub const COR_DETALHE: [u8; 4] = [20, 30, 60, 255];

/// O píxel `(x, y)` do quadro `i` é olho ou perna? Quadros: 0 idle · 1 idle a piscar · 2–4 run
/// (perna esquerda à frente, as duas ao meio, a direita à frente) · 5 hit (olhos em X).
fn detalhe(i: usize, x: u32, y: u32) -> bool {
    let olhos = match i {
        1 => y == 6 && matches!(x, 5 | 6 | 9 | 10),
        5 => matches!((x, y), (5 | 7 | 9 | 11, 4 | 6) | (6 | 10, 5)),
        _ => matches!(y, 5 | 6) && matches!(x, 6 | 10),
    };
    let perna = |x0: u32, y0: u32| (x0..x0 + 2).contains(&x) && (y0..y0 + 3).contains(&y);
    let pernas = match i {
        2 => perna(4, 12) || perna(10, 10),
        3 => perna(6, 12) || perna(8, 12),
        4 => perna(4, 10) || perna(10, 12),
        _ => perna(5, 12) || perna(9, 12),
    };
    olhos || pernas
}

/// O topo do chão (m).
#[must_use]
pub fn topo_do_chao() -> f64 {
    f64::from(CHAO.2[1]) + f64::from(CHAO.1) / PPM / 2.0
}

/// Um sprite `w × h` todo pintado de `cor`, com uma slice `nome` que o cobre (a forma do chão).
#[must_use]
pub fn bloco(w: u32, h: u32, cor: [u8; 4], nome: Option<&str>) -> Option<Sprite> {
    let mut s = Sprite::new(NewSprite {
        width: w,
        height: h,
        ..NewSprite::default()
    })
    .ok()?;
    let r = s.bounds();
    let layer = s.layers()[0].id;
    let frame = s.frames()[0].id;
    let pixels = (0..r.w * r.h).flat_map(|_| cor).collect();
    let op = Op::Patch(Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels,
    });
    apply(&mut s, op).ok()?;
    if let Some(nome) = nome {
        let (op, id) = slice::create(&mut s, 0, Rect::new(0, 0, w, h))?;
        apply(&mut s, op).ok()?;
        let mut lista = s.slices().to_vec();
        lista.iter_mut().find(|c| c.id == id)?.name = nome.into();
        apply(&mut s, Op::Slices(lista)).ok()?;
    }
    Some(s)
}

fn em(x: f32, y: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t
}

/// O objecto da raiz de uma peça (o que o Add ▸ Pixel Sprite faz, sem o Draw).
fn peca(world: &mut World, nome: &str, chave: u32, at: [f32; 2]) -> Entity {
    world
        .spawn((
            em(at[0], at[1]),
            Name::new(nome),
            PixelSprite(chave),
            ph2d_pixel_cena::animador_de_nascenca(),
        ))
        .id()
}

/// Uma parede: a peça com Blocks Light (só luz, sem corpo).
pub fn parede(world: &mut World, nome: &str, chave: u32, at: [f32; 2]) -> Entity {
    let w = peca(world, nome, chave, at);
    world.entity_mut(w).insert(BloqueiaLuz { opacidade: 1.0 });
    w
}

/// As chaves dos documentos das peças: o herói (já nascido), o chão e a parede.
pub struct Chaves {
    pub chao: u32,
    pub parede: u32,
}

/// Monta a cena sobre o herói `heroi` (Pixel Sprite já nascido, com o documento `s` = [`heroi_da_cena`]).
pub fn monta(
    world: &mut World,
    heroi: Entity,
    s: &Sprite,
    k: &Chaves,
    chao: &Sprite,
    a: &Arranjo,
) -> Option<[Entity; 4]> {
    let m = 1.0 / PPM;
    world.entity_mut(heroi).insert((
        em(a.heroi[0], a.heroi[1]),
        Name::new("Hero"),
        LockRotation,
        BloqueiaLuz { opacidade: 1.0 },
        AnimRules(vec![AnimRule {
            when: AnimWhen::Action(ACAO.into()),
            play: RUN.into(),
        }]),
    ));
    let q = s.frames()[0].id;
    ph2d_pixel_cena::adiciona_collider_da_slice(
        world,
        heroi,
        ph2d_pixel_cena::fixture::body(s),
        s,
        q,
        m,
    )?;
    let e = peca(world, "Floor", k.chao, a.chao.2);
    world.entity_mut(e).insert(RigidBody {
        kind: BodyKind::Static,
    });
    let id = chao.slices().first()?.id;
    ph2d_pixel_cena::adiciona_collider_da_slice(world, e, id, chao, chao.frames()[0].id, m)?;
    let w = parede(world, "Wall", k.parede, a.parede.2);
    let l = world
        .spawn((
            em(a.lampada[0], a.lampada[1]),
            Name::new("Light"),
            Luz {
                tamanho_m: LAMPADA_RAIO_M,
                intensidade: LAMPADA_INTENSIDADE,
                ..Luz::default()
            },
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(world);
    ph2d_ecs::assign_missing_root_order(world);
    Some([heroi, e, w, l])
}
