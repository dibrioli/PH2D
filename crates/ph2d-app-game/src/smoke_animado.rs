//! **A cena 5 do jogo: o Animated Sprite peça a peça** ([ficha](../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md),
//! E6). O nível 2 com um objecto do menu Add por assunto: o que o menu dá vazio, uma folha cortada por Split by rows
//! com um marcador, quem ouve esse marcador, Starts with e a imagem trocada. A shell veste cada um pelos gestos do produto.

use super::*;

pub const VAZIO: &str = "Empty Sheet";
pub const FOLHA: &str = "Split Sheet";
pub const OUVE: &str = "Hears Step";
pub const INICIO: &str = "Starts With";
pub const TROCADA: &str = "Swapped Image";
/// O sinal que o marcador da folha grita e a regra de [`OUVE`] ouve.
pub const SINAL: &str = "step";

/// Em fila acima das moedas do chão da esquerda, à direita do Player.
const OBJETOS: [(&str, [f32; 2]); 5] = [
    (VAZIO, [-5.5, 1.8]),
    (FOLHA, [-3.5, 1.8]),
    (OUVE, [-1.5, 1.8]),
    (INICIO, [0.5, 1.8]),
    (TROCADA, [-3.5, 4.2]),
];

/// O instante que a foto do item quer (`PH2D_GAME_SMOKE_FOTO`), ou a prova que o condutor corre e relata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FotoAnimada {
    Vazio,
    EmUso,
    More,
    Editor,
    Inicio,
    Aviso,
    RegraAcesa,
    Estreito,
    /// O Play real: ▶ noutra animação, Play, Stop; `[animado-prova]` e `[jogo]` dizem o que aconteceu.
    ProvaPlay,
    /// O Ctrl+Z real de Split by rows, do herói de amostra, de Cut again e de Keep the cut.
    ProvaDesfaz,
}

#[must_use]
pub fn foto_animada() -> Option<FotoAnimada> {
    use FotoAnimada as F;
    if armed_level() != Some(CENA) {
        return None;
    }
    match std::env::var("PH2D_GAME_SMOKE_FOTO").ok()?.trim() {
        "animado_vazio" => Some(F::Vazio),
        "animado_uso" => Some(F::EmUso),
        "animado_more" => Some(F::More),
        "animado_editor" => Some(F::Editor),
        "animado_inicio" => Some(F::Inicio),
        "animado_aviso" => Some(F::Aviso),
        "animado_regra" => Some(F::RegraAcesa),
        "animado_estreito" => Some(F::Estreito),
        "animado_prova_play" => Some(F::ProvaPlay),
        "animado_prova_desfaz" => Some(F::ProvaDesfaz),
        _ => None,
    }
}

impl FotoAnimada {
    /// O objecto que a foto deixa seleccionado.
    #[must_use]
    pub fn alvo(self) -> &'static str {
        match self {
            Self::Vazio => VAZIO,
            Self::EmUso | Self::More | Self::Editor => FOLHA,
            Self::Inicio | Self::Estreito | Self::ProvaPlay => INICIO,
            Self::Aviso | Self::ProvaDesfaz => TROCADA,
            Self::RegraAcesa => "Player",
        }
    }
}

pub const CENA: u32 = 5;
/// Onde a vista abre: a fila de objectos e o Player.
pub const CENTRO: [f32; 2] = [-3.5, 2.5];
pub const ALTURA: f32 = 9.0;

/// Monta o nível 2 e os cinco objectos do menu. A imagem, o corte e o resto vêm depois, pelos gestos.
pub(super) fn nivel_5(sim: &mut SimWorld, portas: Portas<'_>) -> Entity {
    let jogador = nivel_2(sim, portas);
    for (nome, at) in OBJETOS {
        let e = do_menu(sim, portas, crate::object_add::ANIMATED_SPRITE);
        arruma(sim, e, nome, at, [1.0, 1.0]);
    }
    println!(
        "[game-smoke] =5 o Animated Sprite peça a peça: «{VAZIO}» vazio; «{FOLHA}» cortada por Split by rows com um \
         marcador «{SINAL}»; «{OUVE}» salta quando o marcador grita; «{INICIO}» com outra animação em prévia; \
         «{TROCADA}» com o aviso; e o Player com regras, a regra que ganha acesa no Play."
    );
    jogador
}

/// A regra de quem ouve: o marcador da folha grita e ela salta. É o que o «+ Rule» do cartão grava.
pub fn ouve_o_marcador(sim: &mut SimWorld, e: Entity) {
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_physics_ecs::AnimRules(vec![
            ph2d_physics_ecs::AnimRule {
                when: ph2d_physics_ecs::AnimWhen::Signal(SINAL.into()),
                play: "jump".into(),
            },
        ]));
}

/// O relato de uma linha do que a prova confere em `nome`: o que toca, quantas animações, o corte e o aviso.
#[must_use]
pub fn relato(sim: &mut SimWorld, nome: &str) -> String {
    let Some(e) = por_nome(sim, nome) else {
        return format!("{nome}: nao existe");
    };
    let w = sim.world();
    format!(
        "{nome}: toca={:?} inicio={:?} anims={} grade={:?} aviso={:?}",
        w.get::<SpriteAnimator>(e).map(|p| p.current.as_str()),
        w.get::<ph2d_ecs::SpriteAnimStart>(e).map(|s| s.0.as_str()),
        w.get::<SpriteAnimations>(e).map_or(0, |l| l.iter().count()),
        w.get::<SpriteGrid>(e).map(|g| (g.hframes, g.vframes)),
        w.get::<ph2d_ecs::SpriteCutNotice>(e).map(|n| n.grid),
    )
}

/// O horário do nível 5, quadro a quadro: o Player anda e pára, para trocar de regra; o Stop vem no fim.
#[must_use]
pub(super) fn gesto_do_nivel_5(quadro: u32) -> Option<Gesto> {
    const PARA: u32 = 1500;
    const VOLTA: u32 = 200;
    if quadro == PARA {
        return Some(Gesto::Para);
    }
    if !(120..PARA).contains(&quadro) || !(quadro - 120).is_multiple_of(VOLTA / 4) {
        return None;
    }
    Some(match (quadro - 120) / (VOLTA / 4) % 4 {
        0 => Gesto::Direita(true),
        1 => Gesto::Direita(false),
        2 => Gesto::Esquerda(true),
        _ => Gesto::Esquerda(false),
    })
}
