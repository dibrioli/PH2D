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
/// O Animated de DUAS folhas (plano GEF 26 V6): `idle` 4 × 1 e `run` 6 × 2, e uma animação que as mistura.
pub const DUAS: &str = "Two Sheets";
/// As MESMAS duas folhas com a `run` escalada pelo Align (GEF 28 A): o boneco tem o mesmo tamanho em todo quadro.
pub const ALINHADAS: &str = "Aligned Sheets";
/// A `run` foi desenhada 1,5× maior que a `idle` (48 px contra 32 por célula, o boneco 42 contra 28 px de altura):
/// o Align leva-a a 2/3, e os pés ficam no pivô.
pub const ESCALA_DA_RUN: f32 = 2.0 / 3.0;
/// As imagens da cena pela ordem em que a biblioteca as recebe: as duas folhas de `DUAS`.
const IDLE: usize = 2;
const RUN: usize = 3;
/// O sinal que o marcador da folha grita e a regra de [`OUVE`] ouve.
pub const SINAL: &str = "footstep";
/// O índice de «row 1» na lista: a folha entra com o «loop» e o Split junta depois dele.
const ROW_1: u8 = 1;
/// O quadro da animação onde o marcador grita.
const QUADRO_DO_GRITO: u32 = 2;

/// Em fila acima das moedas do chão da esquerda, à direita do Player.
pub const OBJETOS: [(&str, [f32; 2]); 7] = [
    (VAZIO, [-5.5, 1.8]),
    (FOLHA, [-3.5, 1.8]),
    (OUVE, [-1.5, 1.8]),
    (INICIO, [0.5, 1.8]),
    (TROCADA, [-3.5, 4.2]),
    (DUAS, [-1.0, 4.2]),
    (ALINHADAS, [1.5, 4.2]),
];
/// Onde a vista abre: a fila de objectos e o Player.
pub const VISTA: ([f32; 2], f32) = ([-3.5, 2.5], 9.0);

pub const MENSAGEM: &str = "[game-smoke] =5 o Animated Sprite peça a peça: «Empty Sheet» vazio; «Split Sheet» cortada por \
    Split by rows com um marcador «footstep»; «Hears Step» salta quando o marcador grita; «Starts With» com outra animação \
    em prévia; «Swapped Image» com o aviso; «Two Sheets» com as folhas idle e run e a animação «attack» que as mistura; \
    «Aligned Sheets», as mesmas folhas com a run escalada pelo Align: o boneco não cresce ao trocar de folha; \
    e o Player com regras, a que ganha acesa no Play.";

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
    /// «Two Sheets» com a folha `run` visível no cartão Image (plano GEF 26, painel 1 e 3 da maquete).
    Folhas,
    /// …e o editor aberto na «attack», que mistura as duas (painel 4).
    FolhasEditor,
    /// …depois de Delete sheet na `idle`: quadros vazios em alerta (painel 6).
    FolhasApagada,
    /// …depois de Trim com a `run` à vista (dono 08/11): só ela perde a margem; a `idle` fica.
    FolhasTrim,
    /// …depois de Trim na `idle`, a folha 0 (dono 09/11, opção A): cada quadro perde a mesma margem.
    FolhasTrim0,
    /// …no modo Edit (dono 08/11): a `run` inteira, como um Still.
    FolhasEdit,
    /// A ferramenta Pivot com Ctrl sobre o pivô, por cliques no `on_mouse_input` (smoke do dono 09/11): o objeto
    /// fica selecionado, o pivô encaixa no centro do quad e o desenho da `run` não anda.
    FolhasPivo,
    /// «Two Sheets» com a `run` à vista e a imagem `idle` escolhida para ELA: a folha troca e o aviso amarelo diz que
    /// o corte foi refeito (dono 09/11).
    FolhasTroca,
    /// «Two Sheets» e «Aligned Sheets» lado a lado num quadro da `idle` (GEF 28 A)…
    AlinhadasIdle,
    /// …e num quadro da `run`: a de cima cresce, a alinhada não.
    AlinhadasRun,
    /// «Two Sheets» na `idle`, parada no quadro `k` e de perto (`animado_idle_q<k>`): cada quadro cai nos mesmos
    /// pixels do canvas, como no editor (defeito do meio texel do átlas, 09/11).
    IdleQuadro(u32),
    /// A prévia Align (GEF 28 B) no editor de «Two Sheets», num quadro da `run`, ligada pelo botão real.
    Alinhar,
    /// …e a pega do canto arrastada com o rato real até `2/3`: a `run` encolhe para o tamanho da `idle`.
    AlinharEscala,
    /// …e um Ctrl+Z depois: a escala inteira desfaz-se num passo.
    AlinharDesfaz,
}

#[must_use]
pub fn foto() -> Option<Foto> {
    let nome = std::env::var("PH2D_GAME_SMOKE_FOTO").ok()?;
    if let Some(k) = nome.trim().strip_prefix("animado_idle_q") {
        return k.parse().ok().map(Foto::IdleQuadro);
    }
    match nome.trim() {
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
        "animado_folhas" => Some(Foto::Folhas),
        "animado_folhas_editor" => Some(Foto::FolhasEditor),
        "animado_folhas_apagada" => Some(Foto::FolhasApagada),
        "animado_folhas_trim" => Some(Foto::FolhasTrim),
        "animado_folhas_trim0" => Some(Foto::FolhasTrim0),
        "animado_folhas_edit" => Some(Foto::FolhasEdit),
        "animado_folhas_pivo" => Some(Foto::FolhasPivo),
        "animado_folhas_troca" => Some(Foto::FolhasTroca),
        "animado_alinhadas_idle" => Some(Foto::AlinhadasIdle),
        "animado_alinhadas_run" => Some(Foto::AlinhadasRun),
        "animado_align" => Some(Foto::Alinhar),
        "animado_align_escala" => Some(Foto::AlinharEscala),
        "animado_align_desfaz" => Some(Foto::AlinharDesfaz),
        _ => None,
    }
}

impl Foto {
    /// Onde a vista da foto abre: perto das duas folhas nas do Align, senão a [`VISTA`] da cena.
    #[must_use]
    pub fn vista(self) -> ([f32; 2], f32) {
        match self {
            Self::AlinhadasIdle | Self::AlinhadasRun => ([0.25, 4.3], 4.0),
            Self::IdleQuadro(_) => (OBJETOS[5].1, 1.4),
            _ => VISTA,
        }
    }

    /// O objecto que a foto deixa seleccionado.
    #[must_use]
    pub fn alvo(self) -> &'static str {
        match self {
            Self::Vazio => VAZIO,
            Self::EmUso | Self::More | Self::Editor => FOLHA,
            Self::Inicio | Self::Estreito | Self::ProvaPlay => INICIO,
            Self::Aviso | Self::ProvaDesfaz => TROCADA,
            Self::RegraAcesa => PLAYER,
            Self::Folhas
            | Self::FolhasEditor
            | Self::FolhasApagada
            | Self::FolhasTrim
            | Self::FolhasTrim0
            | Self::FolhasEdit
            | Self::FolhasPivo
            | Self::FolhasTroca
            | Self::Alinhar
            | Self::AlinharEscala
            | Self::AlinharDesfaz
            | Self::IdleQuadro(_) => DUAS,
            Self::AlinhadasIdle | Self::AlinhadasRun => ALINHADAS,
        }
    }
}

/// Um gesto do guião, sobre o objecto de nome dado.
#[derive(Clone, Debug, PartialEq)]
pub enum Passo {
    Anim(&'static str, Ed),
    Verbo(&'static str, Verbo),
    /// Veste o objecto com o herói de fábrica: é a cena que o faz, o artista não tem este botão.
    Heroi(&'static str),
    /// Escolhe a imagem da biblioteca da cena (pela ordem em que a cena as traz).
    Imagem(&'static str, usize),
    /// Load sheet respondido com a imagem da biblioteca da cena (pela ordem em que a cena as traz).
    Folha(&'static str, usize),
    /// Uma ferramenta de imagem de um clique (`OneShotImageOp`, «Trim Transparency»…) sobre `nome`.
    Ferramenta(&'static str, &'static str),
    /// O botão Split by rows sobre a folha que `nome` mostra.
    Corta(&'static str),
    /// A regra que ouve o marcador.
    Ouve(&'static str),
    /// A selecção e a vista da foto.
    Foto(Option<Foto>),
    /// Entra no modo Edit do objecto selecionado.
    Edita(&'static str),
    /// A fase `0..=4` do gesto do Pivot com Ctrl ([`Foto::FolhasPivo`]): o botão Pivot (abaixo, acima), Ctrl e o
    /// botão sobre o pivô, o arrasto até perto do centro do quad, soltar e relatar.
    Pivo(&'static str, u8),
    /// A fase da prévia Align pelas portas do rato ([`previa_align`]): `0`/`1` o botão Align, `2..=7` a pega do canto.
    Alinha(&'static str, u8),
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
            P::Heroi(OUVE),
            P::Heroi(INICIO),
            P::Heroi(TROCADA),
            P::Imagem(FOLHA, 0),
            P::Imagem(TROCADA, 1),
            P::Imagem(DUAS, IDLE),
            P::Imagem(ALINHADAS, IDLE),
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
        _ => {
            let mut v = duas_folhas(DUAS, q, foto);
            v.extend(duas_folhas(ALINHADAS, q, None));
            v.extend(alinha(q, foto));
            v.extend(previa_align(q, foto));
            v
        }
    };
    match foto {
        Some(Foto::ProvaPlay) => v.extend(prova_do_play(q)),
        Some(Foto::ProvaDesfaz) => v.extend(prova_do_desfazer(q)),
        Some(Foto::RegraAcesa) if q.is_multiple_of(30) => v.push(P::RelataRegra),
        _ => {}
    }
    v
}

/// ⭐ «Two Sheets» (plano GEF 26 V6), pelas portas do artista: o corte da `idle`, Load sheet com a `run`, as três
/// animações (`idle` na folha 1, `run` na 2 e `attack` misturada) e a `run` visível. Na foto do alerta, Delete sheet
/// apaga a `idle`.
fn duas_folhas(nome: &'static str, q: u32, foto: Option<Foto>) -> Vec<Passo> {
    use Passo as P;
    use ph2d_editor_core::AnimFrameRef as F;
    let (i, r) = (|cell| F { sheet: 0, cell }, |cell| F { sheet: 1, cell });
    match q {
        3 => vec![P::Anim(nome, Ed::Grid(4, 1)), P::Folha(nome, RUN)],
        4 => vec![
            P::Anim(nome, Ed::Rename(0, "idle".into())),
            P::Anim(nome, Ed::To(0, 3)),
            P::Anim(nome, Ed::Add),
        ],
        6 => vec![
            P::Anim(nome, Ed::Rename(1, "run".into())),
            P::Anim(nome, Ed::Frames(1, (0..12).map(r).collect())),
            P::Anim(nome, Ed::Add),
        ],
        9 => vec![
            P::Anim(nome, Ed::Rename(2, "attack".into())),
            P::Anim(nome, Ed::Frames(2, vec![r(0), r(1), i(0), i(1), r(2)])),
            P::Anim(nome, Ed::SetCurrent("attack".into())),
            P::Verbo(nome, Verbo::ShowSheet(1)),
        ],
        10 if foto == Some(Foto::FolhasApagada) => vec![P::Verbo(nome, Verbo::ShowSheet(0))],
        11 if foto == Some(Foto::FolhasApagada) => vec![P::Verbo(nome, Verbo::DeleteSheet)],
        13 if foto == Some(Foto::FolhasTroca) => vec![P::Imagem(nome, IDLE)],
        10 if foto == Some(Foto::FolhasTrim) => vec![P::Ferramenta(nome, "trim_transparency")],
        10 if foto == Some(Foto::FolhasTrim0) => vec![P::Verbo(nome, Verbo::ShowSheet(0))],
        11 if foto == Some(Foto::FolhasTrim0) => vec![P::Ferramenta(nome, "trim_transparency")],
        // Como o dono no smoke 09/11: a ferramenta Pivot ativa antes do Edit.
        11 | 12 if foto == Some(Foto::FolhasEdit) => vec![P::Pivo(nome, (q - 11) as u8)],
        13 if foto == Some(Foto::FolhasEdit) => vec![P::Edita(nome)],
        15..=19 if foto == Some(Foto::FolhasPivo) => vec![P::Pivo(nome, (q - 15) as u8)],
        14 => vec![P::Relata("as folhas".into(), nome)],
        _ => Vec::new(),
    }
}

/// ⭐ «Aligned Sheets» (GEF 28 A): com a `run` à vista, o verbo do Align escala-a; as fotos põem as duas no mesmo
/// quadro de uma folha, paradas.
fn alinha(q: u32, foto: Option<Foto>) -> Vec<Passo> {
    use Passo as P;
    let quadro = |anim: &str| {
        [DUAS, ALINHADAS]
            .into_iter()
            .flat_map(|n| {
                [
                    P::Anim(n, Ed::SetCurrent(anim.into())),
                    P::Anim(n, Ed::Playing(false)),
                ]
            })
            .collect()
    };
    match (q, foto) {
        (10, _) => vec![P::Verbo(ALINHADAS, Verbo::ScaleSheet(ESCALA_DA_RUN))],
        (11, Some(Foto::AlinhadasIdle)) => quadro("idle"),
        (11, Some(Foto::AlinhadasRun)) => quadro("run"),
        (11, Some(Foto::IdleQuadro(k))) => vec![
            P::Anim(DUAS, Ed::SetCurrent("idle".into())),
            P::Anim(DUAS, Ed::Playing(false)),
            P::Anim(DUAS, Ed::SetStep(k)),
        ],
        (15, _) => vec![P::Relata("alinhadas".into(), ALINHADAS)],
        _ => Vec::new(),
    }
}

/// ⭐ A prévia Align (GEF 28 B) com o rato real: a `run` parada no 4.º quadro, o botão Align, e (nas fotos do gesto)
/// a pega do canto puxada para o Pivot até `2/3`; depois um Ctrl+Z. Cada etapa relata as folhas.
fn previa_align(q: u32, foto: Option<Foto>) -> Vec<Passo> {
    use Passo as P;
    let Some(f) =
        foto.filter(|f| matches!(f, Foto::Alinhar | Foto::AlinharEscala | Foto::AlinharDesfaz))
    else {
        return Vec::new();
    };
    let gesto = f != Foto::Alinhar;
    match q {
        20 => vec![
            P::Anim(DUAS, Ed::SetCurrent("run".into())),
            P::Anim(DUAS, Ed::Playing(false)),
            P::Anim(DUAS, Ed::SetStep(3)),
        ],
        24 | 25 => vec![P::Alinha(DUAS, (q - 24) as u8)],
        30 => vec![P::Relata("align antes".into(), DUAS)],
        32..=37 if gesto => vec![P::Alinha(DUAS, (q - 30) as u8)],
        40 if gesto => vec![P::Relata("align escalada".into(), DUAS)],
        50 if f == Foto::AlinharDesfaz => vec![P::Desfaz],
        60 if f == Foto::AlinharDesfaz => vec![P::Relata("align 1 Ctrl+Z".into(), DUAS)],
        _ => Vec::new(),
    }
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
        "{nome}: toca={:?} inicio={:?} anims={} grade={:?} aviso={:?} folhas={} visivel={} quadros={:?} extra={:?}",
        w.get::<SpriteAnimator>(e).map(|p| p.current.as_str()),
        w.get::<SpriteAnimator>(e).and_then(|p| p.start.as_deref()),
        w.get::<SpriteAnimations>(e).map_or(0, |l| l.iter().count()),
        w.get::<SpriteGrid>(e).map(|g| (g.hframes, g.vframes)),
        w.get::<SpriteAnimations>(e).and_then(|l| l.1),
        1 + w.get::<ph2d_ecs::SpriteSheets>(e).map_or(0, |s| s.0.len()),
        ph2d_ecs::shown_sheet(w, e),
        w.get::<ph2d_ecs::SpriteAnimFrames>(e).map(|f| f
            .0
            .iter()
            .map(|t| (t.tag.clone(), t.cells.len()))
            .collect::<Vec<_>>()),
        w.get::<ph2d_ecs::SpriteSheets>(e).map(|s| s
            .0
            .iter()
            .map(|x| (x.size, x.anchor))
            .collect::<Vec<_>>()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ «Two Sheets»: Load sheet com a `run` e a «attack» misturada como na maquete (run, run, idle, idle, run); Delete
    /// sheet só na foto do alerta. *Mutação: a tira toda numa folha ⇒ reprova.*
    #[test]
    fn two_sheets_carrega_a_run_e_mistura_a_attack() {
        let todos = |f| (0..20).flat_map(move |q| passos(q, f)).collect::<Vec<_>>();
        let v = todos(None);
        assert!(v.contains(&Passo::Folha(DUAS, RUN)));
        let attack = v.iter().find_map(|p| match p {
            Passo::Anim(DUAS, Ed::Frames(2, f)) => {
                Some(f.iter().map(|x| x.sheet).collect::<Vec<_>>())
            }
            _ => None,
        });
        assert_eq!(attack, Some(vec![1, 1, 0, 0, 1]));
        let apaga = |v: &[Passo]| v.contains(&Passo::Verbo(DUAS, Verbo::DeleteSheet));
        assert!(!apaga(&v));
        assert!(apaga(&todos(Some(Foto::FolhasApagada))));
    }

    /// O guião da prova do desfazer não corta nem marca antes do gesto dela: um passo por gesto.
    #[test]
    fn a_prova_do_desfazer_nao_veste_a_folha() {
        let todos = || (0..300).flat_map(|q| passos(q, Some(Foto::ProvaDesfaz)));
        assert_eq!(todos().filter(|p| matches!(p, Passo::Corta(_))).count(), 1);
        assert_eq!(todos().filter(|p| *p == Passo::Desfaz).count(), 2);
    }
}
