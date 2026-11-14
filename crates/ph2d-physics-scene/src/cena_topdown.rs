//! **A cena do Top-Down Player** (`PH2D_GAME_SMOKE=24`, ficha GEF 05): um jogo visto de cima — o chão, três
//! paredes para ver o deslize e o herói de fábrica VISTO DE CIMA (`TOPDOWN_HERO_ART`, olha para +X) com o mover
//! de vista de cima e Facing = Move. Montada SÓ com peças: o «Static Body» e a «Sprite» do menu Add, o `+` do
//! Inspector (as portas vêm da composição) e as regras de animação «Key held → walk» do cartão Animation.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, ZIndexOverride};
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::{NodeId, ids};
use ph2d_physics_ecs::{AnimRule, AnimRules, AnimWhen, Collider, ColliderShape, TopDownPlayer};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::rotation::{self, RotationMode};

/// O número da cena no `PH2D_GAME_SMOKE`.
pub const CENA: u32 = 24;

/// O nome do herói (nome de objecto da cena, não texto da interface).
pub const HEROI: &str = "Hero";

/// Onde o herói nasce (m): no meio da arena — o «Frame Selected» do smoke enquadra-a inteira à volta dele.
pub const INICIO: [f32; 2] = [-3.0, -1.0];

/// As paredes: nome, centro, meia-extensão (m). A diagonal ↗ do início bate de raspão na de cima e desliza
/// até ao canto da da direita (que fica à esquerda do Inspector na foto de 1930 px).
pub const PAREDES: [(&str, [f32; 2], [f32; 2]); 3] = [
    ("Wall Top", [-3.5, 2.2], [4.5, 0.3]),
    ("Wall Right", [0.7, -1.0], [0.3, 3.5]),
    ("Pillar", [-5.5, -3.0], [0.5, 0.5]),
];

/// O chão (m): o centro e o tamanho da arena.
pub const CHAO: ([f32; 2], [f32; 2]) = ([-3.5, -1.0], [9.0, 7.0]);
const CHAO_RGBA: [f32; 4] = [0.22, 0.30, 0.24, 1.0];
const PAREDE_RGBA: [f32; 4] = [0.45, 0.42, 0.38, 1.0];

/// As peças que a cena pede à composição.
#[derive(Clone, Copy)]
pub struct Pecas<'a> {
    /// O «Static Body» do menu Add (nasce com a forma em filha).
    pub estatico: &'a dyn Fn(&mut SimWorld) -> Entity,
    /// A «Sprite» do menu Add.
    pub imagem: &'a dyn Fn(&mut SimWorld) -> Entity,
    /// O `+` do Inspector: anexa o componente pelo nome canónico, com a cascata e as sementes.
    pub anexa: &'a dyn Fn(&mut SimWorld, u64, &str) -> Result<(), String>,
}

fn arruma(sim: &mut SimWorld, e: Entity, nome: &str, at: [f32; 2]) {
    let mut em = sim.world_mut().entity_mut(e);
    em.insert(Name::new(nome));
    if let Some(mut t) = em.get_mut::<Transform>() {
        t.translation = Vec2::new(at[0], at[1]);
    }
}

/// Monta a cena e devolve o herói (o objecto que o Inspector abre).
pub fn montar(sim: &mut SimWorld, p: Pecas<'_>) -> Entity {
    let chao = (p.imagem)(sim);
    arruma(sim, chao, "Floor", CHAO.0);
    sim.world_mut().entity_mut(chao).insert((
        Sprite::atlas(WHITE_TILE_KEY, CHAO.1, CHAO_RGBA),
        ZIndexOverride(-1),
    ));
    for (nome, c, h) in PAREDES {
        let e = (p.estatico)(sim);
        arruma(sim, e, nome, c);
        let w = sim.world_mut();
        w.entity_mut(e).insert(Sprite::atlas(
            WHITE_TILE_KEY,
            [2.0 * h[0], 2.0 * h[1]],
            PAREDE_RGBA,
        ));
        let f = ph2d_physics_ecs::forma_para_editar(w, e).unwrap_or(e);
        if let Some(mut col) = w.get_mut::<Collider>(f) {
            col.shape = ColliderShape::Cuboid {
                half_x: h[0],
                half_y: h[1],
            };
        }
    }
    let heroi = (p.imagem)(sim);
    arruma(sim, heroi, HEROI, INICIO);
    if let Err(e) = ph2d_render::veste_o_heroi_de_cima(sim.world_mut(), heroi) {
        eprintln!("[game-smoke] =24 a folha do herói: {e}");
    }
    sim.world_mut().entity_mut(heroi).insert(regras_de_andar());
    if let Err(e) = (p.anexa)(sim, heroi.to_bits(), "ph2d::physics::TopDownPlayer") {
        eprintln!("[game-smoke] =24 o Top-Down Player: {e}");
    }
    if let Some(mut td) = sim.world_mut().get_mut::<TopDownPlayer>(heroi) {
        td.rotation_mode = rotation::to_wire(RotationMode::ToMovement);
        td.default_controls &= foto(foto_pedida().as_deref()) != Some(Foto::Desligado);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    println!(
        "[game-smoke] =24 um jogo visto de cima: o herói (Hero) tem o Top-Down Player com Facing = Move e está \
         selecionado. Play (F5): as setas andam em 8 direções e a imagem vira para onde anda; encoste na diagonal \
         numa parede e ele desliza ao longo dela."
    );
    heroi
}

/// As teclas de andar do Top-Down Player (o mapa do projecto).
pub const ACOES_DE_ANDAR: [&str; 4] = [
    ph2d_input::PLAYER_MOVE_LEFT,
    ph2d_input::PLAYER_MOVE_RIGHT,
    ph2d_input::PLAYER_MOVE_UP,
    ph2d_input::PLAYER_MOVE_DOWN,
];

/// ⭐ **Anda enquanto uma tecla de andar está segura**: uma regra «Key held → walk» por tecla; soltas todas, o
/// tocador volta ao clip de entrada (`idle`) — `ph2d_physics_ecs::segura_regra`, sem código novo.
#[must_use]
pub fn regras_de_andar() -> AnimRules {
    AnimRules(
        ACOES_DE_ANDAR
            .iter()
            .map(|a| AnimRule {
                when: AnimWhen::Action((*a).to_owned()),
                play: ph2d_render::TOPDOWN_HERO_WALK.to_owned(),
            })
            .collect(),
    )
}

/// O instante das fotos do cartão (`PH2D_GAME_SMOKE_FOTO`, ficha GEF 05); sem a variável a cena é a do dono.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Foto {
    /// `topdown_more`: a dobra More aberta.
    MaisAberto,
    /// `topdown_off`: Default Controls desligado — o aviso e o «Turn on».
    Desligado,
}

/// Os nomes que [`foto`] aceita (censo em `ph2d_game_smoke_fotos::nomes`).
pub const NOMES: &str = "topdown_more topdown_off";

/// A foto pedida pelo nome.
#[must_use]
pub fn foto(nome: Option<&str>) -> Option<Foto> {
    match nome?.trim() {
        "topdown_more" => Some(Foto::MaisAberto),
        "topdown_off" => Some(Foto::Desligado),
        _ => None,
    }
}

fn foto_pedida() -> Option<String> {
    std::env::var("PH2D_GAME_SMOKE_FOTO").ok()
}

/// ⭐ **As gavetas do Inspector ao abrir a cena** (`(secção, fechada)`): `cartao` é o que a família abre; na
/// cena 24 o Top-Down Player abre, e a foto `topdown_more` abre também a dobra More.
#[must_use]
pub fn gavetas(nivel: u32, foto: Option<Foto>, cartao: NodeId) -> Vec<(NodeId, bool)> {
    let mut g = vec![(cartao, false)];
    if nivel == CENA {
        g.push((ids::INSP_LIVE_TOPDOWN_SECTION, false));
        if foto == Some(Foto::MaisAberto) {
            g.push((ids::INSP_TD_FOLD_MORE, false));
        }
    }
    g
}

/// O cartão que o Inspector traz à vista na cena `nivel`: na 24 o Top-Down Player, que a folha animada do
/// herói (Image + Animation) e o Body empurram para fora da coluna; na foto `fisica_player_pulo` (ficha GEF 04)
/// o Platform Player, cuja Jump (Show Jump Arc, Jump Height) cai abaixo da dobra numa janela de 1040 px.
#[must_use]
pub fn cartao_a_vista(nivel: u32, foto: Option<&str>) -> Option<NodeId> {
    use crate::fotos_dos_cartoes::{CasoFisica, caso};
    if nivel == CENA {
        return Some(ids::INSP_LIVE_TOPDOWN_SECTION);
    }
    (foto.and_then(|n| caso(n.trim())) == Some(CasoFisica::PlayerPulo))
        .then_some(ids::INSP_LIVE_PLAYER_SECTION)
}

/// Aplica as [`gavetas`] da cena `nivel` ao Inspector e pede-lhe que role até ao [`cartao_a_vista`] de `heroi`.
pub fn arruma_o_inspector(store: &mut WidgetStore, nivel: u32, cartao: NodeId, heroi: u64) {
    arruma_o_inspector_com(store, nivel, cartao, heroi, foto_pedida().as_deref());
}

fn arruma_o_inspector_com(
    store: &mut WidgetStore,
    nivel: u32,
    cartao: NodeId,
    heroi: u64,
    pedida: Option<&str>,
) {
    for (id, fechada) in gavetas(nivel, foto(pedida), cartao) {
        store.set_collapsed(id, fechada);
    }
    if let Some(s) = cartao_a_vista(nivel, pedida) {
        store.request_inspector_section(s, heroi);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn na_cena_24_o_cartao_top_down_nasce_a_vista() {
        let c = ids::INSP_LIVE_PLAYER_SECTION;
        let g = gavetas(CENA, None, c);
        assert_eq!(g, vec![(c, false), (ids::INSP_LIVE_TOPDOWN_SECTION, false)]);
        assert_eq!(
            cartao_a_vista(CENA, None),
            Some(ids::INSP_LIVE_TOPDOWN_SECTION)
        );
        let more = ids::INSP_TD_FOLD_MORE;
        assert!(gavetas(CENA, Some(Foto::MaisAberto), c).contains(&(more, false)));
        assert_eq!(
            gavetas(4, Some(Foto::MaisAberto), c),
            vec![(c, false)],
            "as outras cenas não mudam"
        );
        assert_eq!(cartao_a_vista(4, None), None);
    }

    /// ⭐ A foto `fisica_player_pulo` (GEF 04) abre com o cartão Jump à vista: Show Jump Arc e Jump Height ficavam
    /// abaixo da dobra numa janela de 1040 px (a foto comitada só os mostrava a 1972 px de altura). O nome sem o
    /// prefixo (`player_pulo`) não é foto nenhuma — a cena abre sem a caixa marcada.
    #[test]
    fn a_foto_do_pulo_traz_o_cartao_platform_player_a_vista() {
        let p = ids::INSP_LIVE_PLAYER_SECTION;
        assert_eq!(cartao_a_vista(4, Some("fisica_player_pulo")), Some(p));
        assert_eq!(cartao_a_vista(4, Some("fisica_player_pulo_terreno")), None);
        assert_eq!(cartao_a_vista(4, Some("player_pulo")), None);
        assert_eq!(cartao_a_vista(4, Some("fisica_player")), None);
        let mut s = WidgetStore::default();
        arruma_o_inspector_com(&mut s, 4, p, 0xF3, Some("fisica_player_pulo"));
        assert_eq!(s.inspector_section_reveal(), Some((p, 0xF3)));
    }

    #[test]
    fn a_foto_so_existe_pelo_nome() {
        assert_eq!(foto(Some("topdown_more")), Some(Foto::MaisAberto));
        assert_eq!(foto(Some("topdown_off")), Some(Foto::Desligado));
        assert_eq!(foto(Some("fisica_player")), None);
        assert_eq!(foto(None), None);
    }
}
