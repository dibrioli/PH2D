//! **A lista de quadros de uma animação** — a ordem que o artista clicou na folha
//! ([plano 07](../../../docs/GameEngineFuncional/07_animacao_num_cartao_plano.md) §Desenho técnico).
//!
//! Uma [`AnimationTag`] é um INTERVALO `from..=to`; o editor de animações deixa escolher células por
//! qualquer ordem, com repetições. Os dois componentes desta página são IRMÃOS da biblioteca e do
//! tocador, e não campos deles, porque o blob é postcard POSICIONAL: um campo novo na tag subiria o
//! `PROJECT_SCHEMA`. Ausentes = o intervalo de sempre, e um ficheiro antigo abre bit a bit igual.
//!
//! - [`SpriteAnimFrames`] (autorado): por NOME de tag, as células na ordem tocada.
//! - [`SpriteAnimCursor`] (tocador): o PASSO da lista no ecrã. Uma lista pode repetir células, então o
//!   passo não se deriva da célula.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::sprite_anim::{AnimOutcome, AnimationTag, SpriteAnimator, advance_span};
use crate::{SimComponent, SpriteAnimations};

/// Quantos quadros uma lista guarda: uma folha de 64 × 64 células cabe inteira.
pub const ANIM_FRAMES_MAX: usize = 4096;

/// As células de UMA tag, na ordem em que tocam.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagFrames {
    /// O nome da [`AnimationTag`] dona — renomear a tag reescreve-o ([`rename_tag`]).
    pub tag: String,
    pub cells: Vec<u32>,
}

/// **As listas de quadros de uma sprite.** Componente opcional: uma tag sem entrada toca o intervalo.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteAnimFrames(pub SmallVec<[TagFrames; 4]>);

impl SimComponent for SpriteAnimFrames {}

impl SpriteAnimFrames {
    /// A lista da tag `name`, se ela tem uma.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&[u32]> {
        self.0
            .iter()
            .find(|f| f.tag == name)
            .map(|f| f.cells.as_slice())
    }

    /// Põe (ou tira, com `cells` vazio) a lista da tag `name`. Corta em [`ANIM_FRAMES_MAX`].
    pub fn set(&mut self, name: &str, mut cells: Vec<u32>) {
        cells.truncate(ANIM_FRAMES_MAX);
        if cells.is_empty() {
            self.0.retain(|f| f.tag != name);
            return;
        }
        match self.0.iter_mut().find(|f| f.tag == name) {
            Some(f) => f.cells = cells,
            None => self.0.push(TagFrames {
                tag: name.to_owned(),
                cells,
            }),
        }
    }

    /// Esquece a lista da tag `name` (a tag saiu da biblioteca).
    pub fn remove(&mut self, name: &str) {
        self.0.retain(|f| f.tag != name);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// **O passo da lista que está no ecrã.** Só existe ao lado de um [`SpriteAnimFrames`].
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteAnimCursor {
    pub step: u32,
}

impl SimComponent for SpriteAnimCursor {}

/// **As células que a tag toca hoje**: a lista, se existe e cabe toda na grelha, senão o intervalo.
///
/// ⚠️ Uma lista com UMA célula fora da grelha não toca (a tag fica pendurada), e não se apara: aparar
/// tocaria outra animação que ninguém autorou. O editor mostra essas células a vermelho.
#[must_use]
pub fn tag_cells(tag: &AnimationTag, list: Option<&[u32]>, cells: u32) -> Option<Vec<u32>> {
    match list {
        Some(l) if !l.is_empty() => (cells > 0 && l.iter().all(|&c| c < cells)).then(|| l.to_vec()),
        _ => tag.resolve(cells).map(|(lo, hi)| (lo..=hi).collect()),
    }
}

/// **[`crate::advance`] para uma tag que pode ter lista.** Sem lista é exatamente o intervalo (o
/// `step` é então `frame - lo`, e fica escrito para quem o mostrar).
pub fn advance_frames(
    animator: &mut SpriteAnimator,
    tag: &AnimationTag,
    list: Option<&[u32]>,
    frame: &mut u32,
    step: &mut u32,
    cells: u32,
    dt_ticks: u64,
) -> AnimOutcome {
    let Some(l) = list.filter(|l| !l.is_empty()) else {
        let out = crate::advance(animator, tag, frame, cells, dt_ticks);
        if let Some((lo, _)) = tag.resolve(cells) {
            *step = frame.saturating_sub(lo);
        }
        return out;
    };
    if cells == 0 || l.iter().any(|&c| c >= cells) {
        return AnimOutcome::default();
    }
    let n = l.len() as u32;
    // O passo guardado só vale se aponta para a célula que está no ecrã; senão procura-a, e uma célula
    // que a lista não tem entra pela ponta certa (o `u32::MAX` cai fora de `0..n`).
    let mut s = if *step < n && l[*step as usize] == *frame {
        *step
    } else {
        l.iter()
            .position(|&c| c == *frame)
            .map_or(u32::MAX, |p| p as u32)
    };
    let out = advance_span(animator, tag, &mut s, 0, n - 1, dt_ticks);
    let cell = l[s as usize];
    let changed = cell != *frame || out.frame_changed;
    *frame = cell;
    *step = s;
    AnimOutcome {
        frame_changed: changed,
        ..out
    }
}

/// **A célula (e o passo) por onde um ciclo começa** — o [`crate::entry_frame`] de uma tag que pode ter
/// lista: o primeiro quadro, ou o último se a direção começa ao contrário.
#[must_use]
pub fn entry_cell(
    animator: &SpriteAnimator,
    tag: &AnimationTag,
    list: Option<&[u32]>,
    cells: u32,
) -> Option<(u32, u32)> {
    let seq = tag_cells(tag, list, cells)?;
    let step = if animator.direction(tag).starts_reversed() {
        seq.len() - 1
    } else {
        0
    };
    match list.filter(|l| !l.is_empty()) {
        Some(_) => Some((seq[step], step as u32)),
        None => crate::entry_frame(animator, tag, cells).map(|f| (f, step as u32)),
    }
}

/// Quadros por segundo de um `frame_ms` (apresentação; o dado é o ms).
#[must_use]
pub fn fps_of_ms(ms: u32) -> f32 {
    1000.0 / ms.max(1) as f32
}

/// O `frame_ms` de uns quadros por segundo, arredondado e dentro de [`crate::FRAME_MS_MIN`]..=MAX.
#[must_use]
pub fn ms_of_fps(fps: f32) -> u32 {
    if !(fps.is_finite() && fps > 0.0) {
        return crate::FRAME_MS_MAX;
    }
    ((1000.0 / fps).round() as u32).clamp(crate::FRAME_MS_MIN, crate::FRAME_MS_MAX)
}

/// **Renomear uma tag e tudo o que a aponta pelo nome**: a biblioteca, a lista de quadros e o tocador.
/// (Os estados do personagem vivem na `ph2d-physics-ecs`; quem lá chega é a porta dela.)
///
/// Devolve `false` sem mexer em nada se o nome novo é inválido, já existe, ou a tag não existe.
pub fn rename_tag(
    library: &mut SpriteAnimations,
    frames: Option<&mut SpriteAnimFrames>,
    animator: Option<&mut SpriteAnimator>,
    old: &str,
    new: &str,
) -> bool {
    if old == new
        || crate::validate_tag_name(new).is_err()
        || library.get(new).is_some()
        || library.get(old).is_none()
    {
        return false;
    }
    for t in &mut library.0 {
        if t.name == old {
            t.name = new.to_owned();
        }
    }
    if let Some(f) = frames {
        for e in &mut f.0 {
            if e.tag == old {
                e.tag = new.to_owned();
            }
        }
    }
    if let Some(a) = animator
        && a.current == old
    {
        a.current = new.to_owned();
    }
    true
}

#[cfg(test)]
#[path = "sprite_anim_frames_tests.rs"]
mod tests;
