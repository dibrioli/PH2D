//! **O cartão «Animation» e o editor de animações** — o que os dois leem e escrevem
//! ([plano 07](../../../../../docs/GameEngineFuncional/07_animacao_num_cartao_plano.md)).
//!
//! Irmão de [`super`] (a §11 de sempre): estes tipos são as perguntas NOVAS —
//! quem escolhe a animação, o modo do ciclo, a lista de quadros e a imagem da folha para as miniaturas.

use std::sync::Arc;

/// **Quem escolhe a animação que toca.** Derivado, nunca guardado: `Rules` = o objecto tem regras
/// (`AnimRules`, ou as 4 do `CharacterAnimations`); senão `Me` = o tocador arranca sozinho (autoplay) e
/// `Script` = não arranca.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum AnimChooser {
    #[default]
    Me,
    Script,
    Rules,
}

impl AnimChooser {
    pub const ALL: [Self; 3] = [Self::Me, Self::Script, Self::Rules];

    #[must_use]
    pub fn tag(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(usize::from(tag)).copied().unwrap_or_default()
    }
}

/// **O modo do ciclo** como o artista o diz. Os dados por baixo são `repeat` + `direction` da tag.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum AnimCycleMode {
    #[default]
    Repeat,
    Once,
    PingPong,
}

impl AnimCycleMode {
    pub const ALL: [Self; 3] = [Self::Repeat, Self::Once, Self::PingPong];

    #[must_use]
    pub fn tag(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(usize::from(tag)).copied().unwrap_or_default()
    }

    /// O modo que uma tag mostra: `direction_tag` é o de `AnimDirection::ALL` (2 e 3 = vai-e-volta) e
    /// `repeat` é `0` = para sempre. ⚠️ Uma tag de `Reverse` mostra-se Repete: o editor não oferece o
    /// sentido inverso (a ordem dos quadros já o diz), e o dado fica como estava até alguém mexer.
    #[must_use]
    pub fn of_row(direction_tag: u8, repeat: u32) -> Self {
        if direction_tag >= 2 {
            Self::PingPong
        } else if repeat == 1 {
            Self::Once
        } else {
            Self::Repeat
        }
    }
}

/// Os pixels da folha da sprite (RGBA reto), para miniaturas e para a grelha do editor.
///
/// ⚠️ Leva o [`ph2d_vector::StableImage`] já feito: o Vello guarda a imagem pelo id do blob, e um
/// `draw_image_rgba` por quadro voltaria a subi-la à placa em cada pintura.
#[derive(Clone)]
pub struct SheetImage {
    pub rgba: Arc<Vec<u8>>,
    pub w: u32,
    pub h: u32,
    pub image: ph2d_vector::StableImage,
}

impl SheetImage {
    /// `None` se `rgba` não tem `w × h × 4` bytes.
    #[must_use]
    pub fn new(rgba: Arc<Vec<u8>>, w: u32, h: u32) -> Option<Self> {
        let image = ph2d_vector::StableImage::from_rgba(rgba.clone(), w, h)?;
        Some(Self { rgba, w, h, image })
    }
}

impl PartialEq for SheetImage {
    /// A MESMA folha (o mesmo blob), não pixels iguais: comparar 4,7 MB por quadro seria o custo todo.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.rgba, &other.rgba) && self.w == other.w && self.h == other.h
    }
}

impl std::fmt::Debug for SheetImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SheetImage({}×{})", self.w, self.h)
    }
}

/// O que o personagem acrescenta às regras: o limiar de correr (m/s) e o espelho ao andar para trás.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimCharacterInfo {
    pub run_from: f32,
    pub flip: bool,
}

/// **A condição de uma regra**, como o cartão a lê (espelho de `ph2d_physics_ecs::AnimWhen`; esta crate
/// não depende da física — ADR-0029). `Pose`: Parado · A correr · A subir · A cair · Agachado; `Life`: Levou dano ·
/// Morreu · Curou-se · Esquivou-se — os índices de `CharacterPose::ALL` / `LifeEvent::ALL`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimWhenInfo {
    Pose(u8),
    Action(String),
    Life(u8),
    Signal(String),
}

/// Quantas poses e quantos factos de vida existem (os comprimentos de `CharacterPose::ALL` e
/// `LifeEvent::ALL`; `the_rule_mirrors_are_the_physics` prende-os).
pub const ANIM_POSES: u8 = 6;
/// O índice de «Agachado» (`CharacterPose::Crouch`): só se oferece com o agachar armado.
pub const ANIM_POSE_CROUCH: u8 = 4;
/// O índice de «A andar agachado» (`CharacterPose::CrouchWalk`): idem.
pub const ANIM_POSE_CROUCH_WALK: u8 = 5;
pub const ANIM_LIFE_EVENTS: u8 = 4;

/// Uma regra: quando `when` vale, toca `play`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimRuleInfo {
    pub when: AnimWhenInfo,
    pub play: String,
}

/// **Tudo o que o cartão e o editor leem e a §11 de sempre não tinha.**
#[derive(Clone, Debug, PartialEq)]
pub struct AnimCardInfo {
    pub chooser: AnimChooser,
    /// O objecto tem Platform Player? Sem ele «O personagem» não se oferece (não há estados).
    pub has_character_body: bool,
    /// O Platform Player sabe agachar (altura agachada autorada)? Sem isso «Crouching» não se oferece.
    pub can_crouch: bool,
    pub character: Option<AnimCharacterInfo>,
    /// As regras que valem, pela ordem (a de cima ganha) — as gravadas ou as 4 do personagem.
    pub rules: Vec<AnimRuleInfo>,
    /// As ações do mapa de teclas, pelo nome (as opções «Key held»).
    pub rule_actions: Vec<String>,
    /// Os nomes de sinal que o projecto já autorou (as opções «Signal»).
    pub rule_signals: Vec<String>,
    pub hframes: u32,
    pub vframes: u32,
    /// O passo da lista de quadros que está no ecrã (`SpriteAnimCursor`); num intervalo, `frame - from`.
    pub step: u32,
    /// A folha, quando a sprite tem pixels em memória.
    pub sheet: Option<SheetImage>,
}

impl Default for AnimCardInfo {
    /// Uma sprite de UMA célula, escolhida por mim, sem personagem nem pixels.
    fn default() -> Self {
        Self {
            chooser: AnimChooser::Me,
            has_character_body: false,
            can_crouch: false,
            character: None,
            rules: Vec::new(),
            rule_actions: Vec::new(),
            rule_signals: Vec::new(),
            hframes: 1,
            vframes: 1,
            step: 0,
            sheet: None,
        }
    }
}

impl AnimCardInfo {
    /// **As opções do menu «When»**, pela ordem do menu: as poses (só com Platform Player), as teclas,
    /// os factos de vida e os sinais.
    #[must_use]
    pub fn when_options(&self) -> Vec<AnimWhenInfo> {
        let poses = (0..ANIM_POSES)
            .filter(|&i| {
                self.has_character_body
                    && (self.can_crouch || (i != ANIM_POSE_CROUCH && i != ANIM_POSE_CROUCH_WALK))
            })
            .map(AnimWhenInfo::Pose);
        let keys = self.rule_actions.iter().cloned().map(AnimWhenInfo::Action);
        let life = (0..ANIM_LIFE_EVENTS).map(AnimWhenInfo::Life);
        let signals = self.rule_signals.iter().cloned().map(AnimWhenInfo::Signal);
        poses.chain(keys).chain(life).chain(signals).collect()
    }
}

/// Os limites do `frame_ms` do motor (`ph2d_ecs::FRAME_MS_MIN`/`MAX`). ⚠️ Cópia: esta crate não
/// depende da `ph2d-ecs` fora dos testes (ADR-0029); `the_fps_law_is_the_engines` prende-a.
const FRAME_MS_MIN: u32 = 1;
const FRAME_MS_MAX: u32 = 60_000; // LITERAL-PX-OK: ms, o tecto do motor

impl super::InspectorAnimRow {
    /// Quadros por segundo desta animação — a mesma leitura do editor (`ph2d_ecs::fps_of_ms`).
    #[must_use]
    pub fn fps(&self) -> f32 {
        1000.0 / self.frame_ms.max(1) as f32 // LITERAL-PX-OK: ms por segundo
    }

    /// O `frame_ms` de `fps` quadros por segundo (`ph2d_ecs::ms_of_fps`): arredondado e nos limites.
    #[must_use]
    pub fn frame_ms_of_fps(fps: f32) -> u32 {
        if !(fps.is_finite() && fps > 0.0) {
            return FRAME_MS_MAX;
        }
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let ms = (1000.0 / fps).round() as u32; // LITERAL-PX-OK: ms por segundo
        ms.clamp(FRAME_MS_MIN, FRAME_MS_MAX) // CLAMP-OK: a faixa do frame_ms é a do motor
    }
}

/// As células que a linha `row` toca, na ordem: a lista, ou o intervalo `from..=to` cortado na grelha.
#[must_use]
pub fn row_cells(row: &super::InspectorAnimRow, cells: u32) -> Vec<u32> {
    if !row.frames.is_empty() {
        return row.frames.clone();
    }
    let span = row.span(cells);
    let lo = row.from.min(row.to);
    (lo..lo + span).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_tags_dos_dois_seletores_vao_e_voltam() {
        for c in AnimChooser::ALL {
            assert_eq!(AnimChooser::from_tag(c.tag()), c);
        }
        for m in AnimCycleMode::ALL {
            assert_eq!(AnimCycleMode::from_tag(m.tag()), m);
        }
        assert_eq!(AnimChooser::from_tag(9), AnimChooser::Me);
    }

    /// A cópia da lei dos quadros por segundo é a do motor, ponto a ponto (o editor usa a do motor).
    #[test]
    fn the_fps_law_is_the_engines() {
        use super::super::InspectorAnimRow as R;
        assert_eq!(FRAME_MS_MIN, ph2d_ecs::FRAME_MS_MIN);
        assert_eq!(FRAME_MS_MAX, ph2d_ecs::FRAME_MS_MAX);
        for fps in [
            -1.0,
            0.0,
            0.01,
            0.5,
            1.0,
            7.0,
            12.0,
            24.0,
            29.97,
            60.0,
            999.0,
            1e9,
            f32::NAN,
        ] {
            assert_eq!(
                R::frame_ms_of_fps(fps),
                ph2d_ecs::ms_of_fps(fps),
                "{fps} fps"
            );
        }
        for ms in [0, 1, 16, 83, 100, 1000, 60_000] {
            let row = R {
                frame_ms: ms,
                ..anim_row()
            };
            assert_eq!(row.fps(), ph2d_ecs::fps_of_ms(ms), "{ms} ms");
        }
    }

    fn anim_row() -> super::super::InspectorAnimRow {
        super::super::InspectorAnimRow {
            name: String::new(),
            from: 0,
            to: 0,
            frame_ms: 100,
            direction_tag: 0,
            repeat: 0,
            hold_ms: 0,
            repeat_delay_ms: 0,
            signal_on_finish: String::new(),
            signal_on_loop: String::new(),
            per_frame_ms: Vec::new(),
            frames: Vec::new(),
        }
    }

    #[test]
    fn o_modo_le_se_da_direcao_e_do_repeat() {
        assert_eq!(AnimCycleMode::of_row(0, 0), AnimCycleMode::Repeat);
        assert_eq!(AnimCycleMode::of_row(0, 1), AnimCycleMode::Once);
        assert_eq!(AnimCycleMode::of_row(0, 3), AnimCycleMode::Repeat);
        assert_eq!(AnimCycleMode::of_row(2, 1), AnimCycleMode::PingPong);
        assert_eq!(AnimCycleMode::of_row(3, 0), AnimCycleMode::PingPong);
    }
}
