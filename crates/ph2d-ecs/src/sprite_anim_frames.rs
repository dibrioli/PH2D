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

use crate::sprite_anim::{AnimOutcome, AnimationTag, SpriteAnimator, advance_span_visiting};
use crate::{SimComponent, SpriteAnimations};

/// Quantos quadros uma lista guarda: uma folha de 64 × 64 células cabe inteira.
pub const ANIM_FRAMES_MAX: usize = 4096;

/// **Um quadro de uma lista**: uma célula de uma folha, ou o buraco que uma folha apagada deixou (plano GEF 26).
///
/// A folha `0` é a do `Sprite` + [`crate::SpriteGrid`]; as outras são as folhas extra do objeto, por ordem.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrameRef {
    Cell {
        sheet: u8,
        cell: u32,
    },
    /// O quadro de uma folha que foi apagada: não pinta nada e dura o seu tempo, até o artista o tirar.
    Empty,
}

impl FrameRef {
    /// Uma célula da folha `0`.
    #[must_use]
    pub const fn cell(cell: u32) -> Self {
        Self::Cell { sheet: 0, cell }
    }

    /// As células `cells` da folha `sheet`, por ordem.
    pub fn run(sheet: u8, cells: impl IntoIterator<Item = u32>) -> Vec<Self> {
        cells
            .into_iter()
            .map(|cell| Self::Cell { sheet, cell })
            .collect()
    }

    /// A célula, se é da folha `sheet`.
    #[must_use]
    pub fn cell_on(self, sheet: u8) -> Option<u32> {
        match self {
            Self::Cell { sheet: s, cell } if s == sheet => Some(cell),
            _ => None,
        }
    }

    /// Cabe nas folhas de hoje? `sheets[k]` = células da folha `k`. O vazio cabe sempre.
    #[must_use]
    pub fn fits(self, sheets: &[u32]) -> bool {
        match self {
            Self::Cell { sheet, cell } => cell < sheet_cells(sheets, sheet),
            Self::Empty => true,
        }
    }
}

/// Células da folha `sheet` (`0` se a folha não existe).
#[must_use]
pub fn sheet_cells(sheets: &[u32], sheet: u8) -> u32 {
    sheets.get(usize::from(sheet)).copied().unwrap_or(0)
}

/// As células de UMA tag, na ordem em que tocam.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagFrames {
    /// O nome da [`AnimationTag`] dona — renomear a tag reescreve-o ([`rename_tag`]).
    pub tag: String,
    pub cells: Vec<FrameRef>,
}

/// **As listas de quadros de uma sprite.** Componente opcional: uma tag sem entrada toca o intervalo; uma entrada
/// VAZIA é uma animação sem quadros (o «+» cria-a assim, e tirar o último quadro deixa-a assim): não toca nada.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteAnimFrames(pub SmallVec<[TagFrames; 4]>);

impl SimComponent for SpriteAnimFrames {}

impl SpriteAnimFrames {
    /// A lista da tag `name`, se ela tem uma.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&[FrameRef]> {
        self.0
            .iter()
            .find(|f| f.tag == name)
            .map(|f| f.cells.as_slice())
    }

    /// Põe a lista da tag `name` — vazia = animação sem quadros; voltar ao intervalo é [`Self::remove`].
    /// Corta em [`ANIM_FRAMES_MAX`].
    pub fn set(&mut self, name: &str, mut cells: Vec<FrameRef>) {
        cells.truncate(ANIM_FRAMES_MAX);
        match self.0.iter_mut().find(|f| f.tag == name) {
            Some(f) => f.cells = cells,
            None => self.0.push(TagFrames {
                tag: name.to_owned(),
                cells,
            }),
        }
    }

    /// Esquece a lista da tag `name`: a tag volta ao intervalo (ou saiu da biblioteca).
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

/// **Os quadros que a tag toca hoje**: a lista, se existe e cabe toda nas folhas, senão o intervalo na folha da tag.
/// `sheets[k]` = células da folha `k`. A lista vazia (animação sem quadros) não toca: `None`.
///
/// ⚠️ Uma lista com UMA célula fora da folha dela não toca (a tag fica pendurada), e não se apara: aparar
/// tocaria outra animação que ninguém autorou. O editor mostra essas células a vermelho. O vazio cabe.
#[must_use]
pub fn tag_cells(
    tag: &AnimationTag,
    list: Option<&[FrameRef]>,
    sheets: &[u32],
) -> Option<Vec<FrameRef>> {
    match list {
        Some(l) => (!l.is_empty() && l.iter().all(|f| f.fits(sheets))).then(|| l.to_vec()),
        None => tag.resolve(sheet_cells(sheets, tag.sheet)).map(|(lo, hi)| {
            (lo..=hi)
                .map(|cell| FrameRef::Cell {
                    sheet: tag.sheet,
                    cell,
                })
                .collect()
        }),
    }
}

/// **[`crate::advance`] para uma tag que pode ter lista.** Sem lista é exatamente o intervalo (o
/// `step` é então `frame - lo`, e fica escrito para quem o mostrar).
pub fn advance_frames(
    animator: &mut SpriteAnimator,
    tag: &AnimationTag,
    list: Option<&[FrameRef]>,
    frame: &mut FrameRef,
    step: &mut u32,
    sheets: &[u32],
    dt_ticks: u64,
) -> AnimOutcome {
    advance_frames_visiting(
        animator,
        tag,
        list,
        frame,
        step,
        sheets,
        dt_ticks,
        &mut |_| {},
    )
}

/// [`advance_frames`] dizendo cada passo em que entra — ver [`crate::SpriteAnimMarkers`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn advance_frames_visiting(
    animator: &mut SpriteAnimator,
    tag: &AnimationTag,
    list: Option<&[FrameRef]>,
    frame: &mut FrameRef,
    step: &mut u32,
    sheets: &[u32],
    dt_ticks: u64,
    visit: &mut dyn FnMut(u32),
) -> AnimOutcome {
    let Some(l) = list else {
        let Some((lo, hi)) = tag.resolve(sheet_cells(sheets, tag.sheet)) else {
            return AnimOutcome::default();
        };
        // Um quadro de outra folha (ou o vazio) está fora do intervalo: entra pela ponta certa.
        let mut cell = match *frame {
            FrameRef::Cell { sheet, cell } if sheet == tag.sheet => cell,
            _ => u32::MAX,
        };
        let out = advance_span_visiting(animator, tag, &mut cell, lo, hi, dt_ticks, visit);
        let now = FrameRef::Cell {
            sheet: tag.sheet,
            cell,
        };
        let changed = now != *frame || out.frame_changed;
        *frame = now;
        *step = cell.saturating_sub(lo);
        return AnimOutcome {
            frame_changed: changed,
            ..out
        };
    };
    // Sem quadros, ou com um fora da folha: não toca, e a imagem fica a que estava.
    if l.is_empty() || !l.iter().all(|f| f.fits(sheets)) {
        return AnimOutcome::default();
    }
    let n = l.len() as u32;
    // O passo guardado só vale se aponta para a célula que está no ecrã; senão procura-a, e uma célula
    // que a lista não tem entra pela ponta certa (o `u32::MAX` cai fora de `0..n`).
    let mut s = if *step < n && l[*step as usize] == *frame {
        *step
    } else {
        l.iter()
            .position(|c| c == frame)
            .map_or(u32::MAX, |p| p as u32)
    };
    let out = advance_span_visiting(animator, tag, &mut s, 0, n - 1, dt_ticks, visit);
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
    list: Option<&[FrameRef]>,
    sheets: &[u32],
) -> Option<(FrameRef, u32)> {
    let seq = tag_cells(tag, list, sheets)?;
    let step = if animator.direction(tag).starts_reversed() {
        seq.len() - 1
    } else {
        0
    };
    match list {
        Some(_) => Some((seq[step], step as u32)),
        None => crate::entry_frame(animator, tag, sheet_cells(sheets, tag.sheet)).map(|cell| {
            (
                FrameRef::Cell {
                    sheet: tag.sheet,
                    cell,
                },
                step as u32,
            )
        }),
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
    if let Some(a) = animator {
        if a.current == old {
            a.current = new.to_owned();
        }
        if a.start.as_deref() == Some(old) {
            a.start = Some(new.to_owned());
        }
    }
    true
}

#[cfg(test)]
#[path = "sprite_anim_frames_tests.rs"]
mod tests;
