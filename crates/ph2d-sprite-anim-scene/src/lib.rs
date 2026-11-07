//! **A cena 5 do jogo: o Animated Sprite peça a peça** ([ficha](../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md),
//! E6). O nível 2 com um objecto do menu Add por assunto: o que o menu dá vazio, uma folha cortada por Split by rows com
//! um marcador, quem ouve esse marcador, Starts with e a imagem trocada. Aqui mora o guião; a shell executa cada
//! [`Passo`] pelas portas do produto, e as provas do Play e do Ctrl+Z reais relatam em `[animado-prova]`.

use ph2d_ecs::{Entity, SimWorld, SpriteAnimations, SpriteAnimator, SpriteGrid};
use ph2d_editor_core::{AnimFieldEdit as Ed, ImageUseEdit as Verbo};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

pub const CENA: u32 = 5;
pub const VAZIO: &str = "Empty Sheet";
pub const FOLHA: &str = "Split Sheet";
pub const OUVE: &str = "Hears Step";
pub const INICIO: &str = "Starts With";
pub const TROCADA: &str = "Swapped Image";
pub const PLAYER: &str = "Player";
/// O sinal que o marcador da folha grita e a regra de [`OUVE`] ouve.
pub const SINAL: &str = "step";
/// O índice de «row 1» na lista: a folha entra com o «loop» e o Split junta depois dele.
const ROW_1: u8 = 1;
/// O quadro da animação onde o marcador grita.
const QUADRO_DO_GRITO: u32 = 2;

/// Em fila acima das moedas do chão da esquerda, à direita do Player.
pub const OBJETOS: [(&str, [f32; 2]); 5] = [
    (VAZIO, [-5.5, 1.8]),
    (FOLHA, [-3.5, 1.8]),
    (OUVE, [-1.5, 1.8]),
    (INICIO, [0.5, 1.8]),
    (TROCADA, [-3.5, 4.2]),
];
/// Onde a vista abre: a fila de objectos e o Player.
pub const VISTA: ([f32; 2], f32) = ([-3.5, 2.5], 9.0);

pub const MENSAGEM: &str = "[game-smoke] =5 o Animated Sprite peça a peça: «Empty Sheet» vazio; «Split Sheet» cortada por \
    Split by rows com um marcador «step»; «Hears Step» salta quando o marcador grita; «Starts With» com outra animação \
    em prévia; «Swapped Image» com o aviso; e o Player com regras, a que ganha acesa no Play.";

/// O instante que a foto do item quer (`PH2D_GAME_SMOKE_FOTO`), ou a prova que o guião corre e relata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Foto {
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
pub fn foto() -> Option<Foto> {
    match std::env::var("PH2D_GAME_SMOKE_FOTO").ok()?.trim() {
        "animado_vazio" => Some(Foto::Vazio),
        "animado_uso" => Some(Foto::EmUso),
        "animado_more" => Some(Foto::More),
        "animado_editor" => Some(Foto::Editor),
        "animado_inicio" => Some(Foto::Inicio),
        "animado_aviso" => Some(Foto::Aviso),
        "animado_regra" => Some(Foto::RegraAcesa),
        "animado_estreito" => Some(Foto::Estreito),
        "animado_prova_play" => Some(Foto::ProvaPlay),
        "animado_prova_desfaz" => Some(Foto::ProvaDesfaz),
        _ => None,
    }
}

impl Foto {
    /// O objecto que a foto deixa seleccionado.
    #[must_use]
    pub fn alvo(self) -> &'static str {
        match self {
            Self::Vazio => VAZIO,
            Self::EmUso | Self::More | Self::Editor => FOLHA,
            Self::Inicio | Self::Estreito | Self::ProvaPlay => INICIO,
            Self::Aviso | Self::ProvaDesfaz => TROCADA,
            Self::RegraAcesa => PLAYER,
        }
    }
}

/// Um gesto do guião, sobre o objecto de nome dado.
#[derive(Clone, Debug, PartialEq)]
pub enum Passo {
    Anim(&'static str, Ed),
    Verbo(&'static str, Verbo),
    /// Escolhe a imagem da biblioteca da cena (pela ordem em que a cena as traz).
    Imagem(&'static str, usize),
    /// O botão Split by rows sobre a folha que `nome` mostra.
    Corta(&'static str),
    /// A regra que ouve o marcador.
    Ouve(&'static str),
    /// A selecção e a vista da foto.
    Foto(Option<Foto>),
    /// A entrada do artista, que fecha o passo de desfazer.
    Entrada,
    Desfaz,
    Relata(String, &'static str),
    RelataRegra,
}

static IMAGENS: OnceLock<Vec<[u8; 32]>> = OnceLock::new();
static QUADRO: AtomicU32 = AtomicU32::new(0);

/// A cena trouxe as imagens: o guião pode começar.
pub fn guarda_imagens(ids: Vec<[u8; 32]>) {
    let _ = IMAGENS.set(ids);
}

#[must_use]
pub fn imagem(i: usize) -> Option<[u8; 32]> {
    IMAGENS.get()?.get(i).copied()
}

/// O quadro do guião; `None` até a cena montar.
#[must_use]
pub fn proximo_quadro() -> Option<u32> {
    IMAGENS
        .get()
        .map(|_| QUADRO.fetch_add(1, Ordering::Relaxed))
}

/// O que o guião faz no quadro `q`.
#[must_use]
pub fn passos(q: u32, foto: Option<Foto>) -> Vec<Passo> {
    use Passo as P;
    let desfaz = foto == Some(Foto::ProvaDesfaz);
    let mut v = match q {
        0 => vec![
            P::Verbo(OUVE, Verbo::SampleHero),
            P::Verbo(INICIO, Verbo::SampleHero),
            P::Verbo(TROCADA, Verbo::SampleHero),
            P::Imagem(FOLHA, 0),
            P::Imagem(TROCADA, 1),
            P::Ouve(OUVE),
        ],
        2 if !desfaz => vec![
            P::Corta(FOLHA),
            P::Anim(INICIO, Ed::StartsWith("run".into())),
        ],
        5 if !desfaz => vec![
            P::Anim(FOLHA, Ed::SetCurrent("row 1".into())),
            P::Anim(FOLHA, Ed::StartsWith("row 1".into())),
            P::Anim(FOLHA, Ed::MarkerAdd(ROW_1)),
            P::Anim(INICIO, Ed::SetCurrent("jump".into())),
        ],
        8 if !desfaz => vec![
            P::Anim(FOLHA, Ed::MarkerStep(ROW_1, 0, QUADRO_DO_GRITO)),
            P::Anim(FOLHA, Ed::MarkerSignal(ROW_1, 0, SINAL.into())),
        ],
        12 => vec![P::Foto(foto)],
        _ => Vec::new(),
    };
    match foto {
        Some(Foto::ProvaPlay) => v.extend(prova_do_play(q)),
        Some(Foto::ProvaDesfaz) => v.extend(prova_do_desfazer(q)),
        Some(Foto::RegraAcesa) if q.is_multiple_of(30) => v.push(P::RelataRegra),
        _ => {}
    }
    v
}

/// ▶ noutra animação; o condutor joga (quadro 60) e pára (1500): o `[jogo]` compara os documentos.
fn prova_do_play(q: u32) -> Vec<Passo> {
    let relata = |t: &str| vec![Passo::Relata(t.to_owned(), INICIO)];
    match q {
        40 => vec![
            Passo::Anim(INICIO, Ed::SetCurrent("crouch".into())),
            Passo::Anim(INICIO, Ed::Playing(true)),
        ],
        50 => relata("depois do play da linha"),
        90 | 300 | 1490 => relata("no jogo"),
        1520 => relata("depois do Stop"),
        _ => Vec::new(),
    }
}

/// Cada gesto, a entrada do artista que fecha o passo, um Ctrl+Z, e o estado a comparar com o de antes. Split by rows
/// corre com a aba à frente, e Keep the cut também. Cut again sem proposta só abre o editor: não é um passo.
fn prova_do_desfazer(q: u32) -> Vec<Passo> {
    const INTERVALO: u32 = 40;
    const COMECO: u32 = 40;
    let Some(d) = q.checked_sub(COMECO) else {
        return Vec::new();
    };
    let (titulo, nome, gesto): (&str, &'static str, Vec<Passo>) = match d / INTERVALO {
        0 => (
            "Split by rows, com a aba aberta",
            FOLHA,
            vec![Passo::Anim(FOLHA, Ed::OpenEditor), Passo::Corta(FOLHA)],
        ),
        1 => (
            "Try with the sample hero",
            VAZIO,
            vec![Passo::Verbo(VAZIO, Verbo::SampleHero)],
        ),
        2 => (
            "Keep the cut",
            TROCADA,
            vec![Passo::Verbo(TROCADA, Verbo::KeepCut)],
        ),
        _ => return Vec::new(),
    };
    let relata = |f: &str| Passo::Relata(format!("{titulo}, {f}"), nome);
    match d % INTERVALO {
        0 => {
            let mut v = vec![relata("antes"), Passo::Entrada];
            v.extend(gesto);
            v
        }
        1..=3 => vec![Passo::Entrada],
        10 => vec![relata("feito"), Passo::Desfaz],
        20 => vec![relata("1 Ctrl+Z")],
        _ => Vec::new(),
    }
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

/// O relato de uma linha do que a prova confere em `e`: o que toca, quantas animações, o corte e o aviso.
#[must_use]
pub fn relato(sim: &SimWorld, nome: &str, e: Entity) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// O guião da prova do desfazer não corta nem marca antes do gesto dela: um passo por gesto.
    #[test]
    fn a_prova_do_desfazer_nao_veste_a_folha() {
        let todos = || (0..300).flat_map(|q| passos(q, Some(Foto::ProvaDesfaz)));
        assert_eq!(todos().filter(|p| matches!(p, Passo::Corta(_))).count(), 1);
        assert_eq!(todos().filter(|p| *p == Passo::Desfaz).count(), 3);
    }
}
