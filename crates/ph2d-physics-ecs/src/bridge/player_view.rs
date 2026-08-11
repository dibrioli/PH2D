//! **O QUE OS SENSORES DO PLAYER OLHARAM** (`W-Probes`) — a leitura que torna
//! visível o que até aqui só existia dentro de um tique.
//!
//! O report do Enio: *"os Rays (ou outros modos de cast) dos Setups dos Players
//! não estão visíveis e nem são plenamente editáveis através da UI"*. Esta é a
//! primeira metade: o overlay de física desenha collider, joint, zona, seta,
//! anel, linha d'água e cruz de contato, e **dos sensores do player não desenha
//! um pixel**. O artista afina `float_height`, `cling_distance`, `wall_reach` e
//! `corner_reach` **às cegas**, inferindo o alcance pelo comportamento.
//!
//! ⚠️ **Isto é a mesma lei que fez o contorno do collider existir** (W2a): *um
//! collider é invisível e um sprite é um quad*. Um **raio** é ainda mais
//! invisível que um collider — ele nem tem forma.
//!
//! # ⚠️ TRÊS estados, e o terceiro é o que o artista não consegue ver hoje
//!
//! Os três sensores condicionais só são lançados nos tiques em que a lei pode
//! agir (`corner_probe_wanted` · `wall_probe_wanted` ·
//! `headroom_probe_wanted`), e essas condições são **estreitas por
//! construção**: o do agachar quer `estava agachado && soltou o botão` — no
//! máximo um tique por gesto. Desenhar só o que foi de facto lançado mostraria
//! **um** raio (o do chão) em quase todo quadro, e a pergunta *"porque é que ele
//! não se agarra à parede?"* continuaria sem resposta na tela.
//!
//! Então o alcance é desenhado **sempre** (é o número que se está a afinar) e a
//! cor diz qual das três coisas aconteceu:
//!
//! | estado | o que significa |
//! |---|---|
//! | [`ProbeState::Idle`] | a lei **não perguntou** neste tique — o sensor está inerte |
//! | [`ProbeState::Clear`] | perguntou e **não achou** nada |
//! | [`ProbeState::Hit`] | perguntou e **achou** |
//!
//! ⚠️ **O `Idle` é a metade nova**, e é a que responde *"a capacidade está
//! desligada"* contra *"o alcance é curto demais"* — dois vereditos opostos que
//! hoje produzem o mesmo nada na tela.
//!
//! # ⚠️ A geometria vem das PORTAS, nunca de uma segunda conta
//!
//! Tudo o que se desenha sai de [`super::player_probes`] — as mesmas funções que
//! **lançam** os raios. Um segundo cálculo do lado do desenho seria uma segunda
//! resposta a *"onde este raio nasce?"*, e divergiria no primeiro dia em que
//! alguém mexesse numa das duas. É a regra que o `scaled_shape` (W6) impôs ao
//! contorno e a `zone_force_world_at` (W-AreaFrame) impôs à seta.
//!
//! # ⚠️ E ele sabe desenhar as DUAS coisas
//!
//! Depois da `W-ShapeCast` o sensor do agachar **varre o corpo** em vez de
//! amostrar linhas. Um overlay que só soubesse linhas o desenharia como um raio
//! que ele não é, e o artista afinaria o `crouch_height` contra um desenho que
//! mente sobre o que o produto mede — daí [`ProbeShape::Sweep`] existir ao lado
//! de [`ProbeShape::Ray`].

use ph2d_ecs::Entity;
use ph2d_platformer::MAX_CORNER_SAMPLES;

/// **De qual sensor esta marca é.** A cor de família sai daqui.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProbeKind {
    /// A perna: um raio para baixo de `float_height + cling_distance`.
    Ground,
    /// A parede: o flanco, três raios na direção em que o jogador empurra.
    Wall,
    /// O perfil do teto que a correção de quina lê.
    Corner,
    /// A folga lateral que a correção de quina mede antes de empurrar.
    Side,
    /// O teto do agachar: o corpo varrido para cima por `rise`.
    Headroom,
}

/// **O que o sensor respondeu** — e *"não foi perguntado"* é uma resposta.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProbeState {
    /// A lei não quis castar neste tique.
    Idle,
    /// Castou e não achou nada no alcance.
    Clear,
    /// Castou e achou.
    Hit,
}

/// **A forma do que o sensor olhou.**
///
/// ⚠️ **O `Profile` é MUITO maior que os irmãos, e isso é o preço do zero-alloc:**
/// ele carrega `[bool; MAX_CORNER_SAMPLES]` porque a contagem é autorada
/// (`W-Probes2`) e o caminho é o de um tique. Encaixotá-lo trocaria 257 bytes de
/// pilha por uma alocação **por marca, por tique** — exatamente o que o HR-3
/// recusa. A lista inteira de um player são ~11 marcas.
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ProbeShape {
    /// Uma LINHA, em mundo. `hit` é a distância a que ela achou.
    Ray {
        origin: [f32; 2],
        /// Unitária.
        dir: [f32; 2],
        reach: f32,
        hit: Option<f32>,
        /// Quanto do alcance está DENTRO do corpo — ver
        /// [`super::player_probes::ProbeRay::skin`]. Quem desenha começa em
        /// `origin + dir·skin`; `hit` e `reach` continuam medidos do `origin`,
        /// porque é dali que o cast partiu.
        skin: f32,
    },
    /// O PERFIL do teto — um leque de raios verticais que nascem no topo da
    /// caixa do corpo e sobem `rise`.
    ///
    /// ⚠️ `half_width` e `reach` viajam **separados** porque é assim que a porta
    /// os toma (`corner_offsets(half_width, reach)`): quem desenha chama a porta
    /// com estes dois números, em vez de reconstruir o vão a partir de um só.
    Profile {
        /// O meio do topo da caixa, em mundo.
        top: [f32; 2],
        half_width: f32,
        reach: f32,
        /// Quanto cada raio sobe.
        rise: f32,
        blocked: [bool; MAX_CORNER_SAMPLES],
        /// Quantas das `blocked` este perfil de facto varreu — a contagem
        /// AUTORADA (`corner_samples`). Quem desenha corta aqui.
        samples: usize,
    },
    /// Uma FORMA VARRIDA — o corpo `body` deslocado por `offset`.
    ///
    /// ⚠️ **A forma não viaja aqui**, e é deliberado: o overlay já desenha o
    /// contorno real deste corpo todo quadro, e pedir-lhe que desenhe *o mesmo
    /// contorno, deslocado* mantém uma resposta só para *"que forma tem este
    /// corpo?"*. Uma cópia da geometria aqui seria a segunda.
    Sweep { body: Entity, offset: [f32; 2] },
}

/// **Uma marca de sensor** — o que ele olhou e o que respondeu.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ProbeMark {
    pub kind: ProbeKind,
    pub state: ProbeState,
    pub shape: ProbeShape,
}

impl ProbeMark {
    /// Um raio que foi **lançado**: o estado sai do `hit`, e não de um argumento.
    ///
    /// ⚠️ **É por isso que o estado e a geometria não podem discordar** — não há
    /// como construir uma marca `Hit` sem distância nem uma `Clear` com ela.
    #[must_use]
    pub fn ray(
        kind: ProbeKind,
        origin: [f32; 2],
        dir: [f32; 2],
        reach: f32,
        hit: Option<f32>,
        skin: f32,
    ) -> Self {
        Self {
            kind,
            state: if hit.is_some() {
                ProbeState::Hit
            } else {
                ProbeState::Clear
            },
            shape: ProbeShape::Ray {
                origin,
                dir,
                reach,
                hit,
                skin,
            },
        }
    }

    /// Um raio que **não foi lançado** — o alcance está lá, a resposta não.
    #[must_use]
    pub fn idle_ray(
        kind: ProbeKind,
        origin: [f32; 2],
        dir: [f32; 2],
        reach: f32,
        skin: f32,
    ) -> Self {
        Self {
            kind,
            state: ProbeState::Idle,
            shape: ProbeShape::Ray {
                origin,
                dir,
                reach,
                hit: None,
                skin,
            },
        }
    }

    /// O perfil do teto. `blocked` vazio ⇒ `Clear`; `asked` falso ⇒ `Idle`.
    #[must_use]
    pub fn profile(
        top: [f32; 2],
        half_width: f32,
        reach: f32,
        rise: f32,
        blocked: Option<[bool; MAX_CORNER_SAMPLES]>,
        samples: usize,
    ) -> Self {
        Self {
            kind: ProbeKind::Corner,
            state: match blocked {
                None => ProbeState::Idle,
                Some(b) if b[..samples.min(MAX_CORNER_SAMPLES)].iter().any(|x| *x) => {
                    ProbeState::Hit
                }
                Some(_) => ProbeState::Clear,
            },
            shape: ProbeShape::Profile {
                top,
                half_width,
                reach,
                rise,
                blocked: blocked.unwrap_or([false; MAX_CORNER_SAMPLES]),
                samples,
            },
        }
    }

    /// A varredura do agachar. `blocked` `None` ⇒ a lei não perguntou.
    #[must_use]
    pub fn sweep(body: Entity, offset: [f32; 2], blocked: Option<bool>) -> Self {
        Self {
            kind: ProbeKind::Headroom,
            state: match blocked {
                None => ProbeState::Idle,
                Some(true) => ProbeState::Hit,
                Some(false) => ProbeState::Clear,
            },
            shape: ProbeShape::Sweep { body, offset },
        }
    }
}
