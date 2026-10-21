//! **A Interface nos canais** — as âncoras (`VecAnchors`), o recorte (`VecClipContent`), o auto
//! layout (`VecLayout`, `VecLayoutItem`, `VecLayoutSize`, `VecLayoutAbsolute`) e a pele do widget
//! (`VecWidget`, `VecWidgetIcon`).
//!
//! Os interruptores de PRESENÇA são campos (as âncoras existirem, recortar, a direcção «Off», ser
//! absoluto, ter mínimo/máximo/base): é o que o painel liga e desliga. Eles publicam-se onde o
//! painel os oferece — o recorte numa forma fechada, a direcção numa moldura, as âncoras num filho
//! de moldura, o filho do fluxo num filho de um auto layout. `VecLayoutItem`/`VecLayoutSize`
//! AUSENTES valem o neutro (o `layout_edit` remove-os no neutro). ⛔ O «Show as Panel» da moldura
//! é a visibilidade do painel no editor (`HeroScreen`), não documento: não é canal.

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as c;
use ph2d_ecs::{
    ChildOf, Entity, FieldKey, LayoutAlign, LayoutDir, LayoutJustify, LayoutSize, SimWorld,
    TextosDosCanais, VecAnchors, VecClipContent, VecFrame, VecLayout, VecLayoutAbsolute,
    VecLayoutItem, VecLayoutSize, VecWidget, VecWidgetIcon,
};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::widget::WidgetKind;

fn k(campo: u16, j: usize) -> FieldKey {
    FieldKey::new(VETOR, campo, 0, u16::try_from(j).unwrap_or(u16::MAX))
}

fn b(x: bool) -> f32 {
    f32::from(u8::from(x))
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "o canal é f32; o documento guarda f64"
)]
fn f(x: f64) -> f32 {
    x as f32
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "arredondado e preso antes da conversão"
)]
fn indice(v: f32, max: usize) -> usize {
    (v.round().max(0.0) as usize).min(max)
}

fn pos<T: PartialEq>(todos: &[T], x: &T) -> f32 {
    f32::from(u16::try_from(todos.iter().position(|y| y == x).unwrap_or(0)).unwrap_or(0))
}

const DIRS: [LayoutDir; 4] = [
    LayoutDir::Row,
    LayoutDir::Column,
    LayoutDir::RowWrap,
    LayoutDir::Grid,
];
const ALINHA: [LayoutAlign; 4] = [
    LayoutAlign::Start,
    LayoutAlign::Center,
    LayoutAlign::End,
    LayoutAlign::Stretch,
];
const DISTRIBUI: [LayoutJustify; 5] = [
    LayoutJustify::Start,
    LayoutJustify::Center,
    LayoutJustify::End,
    LayoutJustify::SpaceBetween,
    LayoutJustify::SpaceAround,
];
const ANCORA_ULTIMA: u16 = c::ANCORA_BASE + 3;
const LAYOUT_FIM: u16 = c::LAYOUT_ABSOLUTO + 1;

/// O campo `kk` é da Interface?
#[must_use]
pub fn e_da_interface(kk: FieldKey) -> bool {
    kk.familia == VETOR
        && kk.i == 0
        && (matches!(
            kk.campo,
            c::MOLDURA_RECORTA | c::WIDGET_PELE | c::WIDGET_ICONE
        ) || (c::ANCORAS_EXISTE..=ANCORA_ULTIMA).contains(&kk.campo)
            || (c::LAYOUT_DIR..LAYOUT_FIM).contains(&kk.campo))
}

fn pai(sim: &SimWorld, ent: Entity) -> Option<Entity> {
    sim.world().get::<ChildOf>(ent).map(ChildOf::parent)
}

fn pai_tem<T: ph2d_ecs::Component>(sim: &SimWorld, ent: Entity) -> bool {
    pai(sim, ent).is_some_and(|p| sim.world().get::<T>(p).is_some())
}

/// Os campos da Interface da entidade. `fechado` = ela é uma forma fechada (pode recortar).
pub fn campos(sim: &SimWorld, ent: Entity, fechado: bool, out: &mut Vec<FieldKey>) {
    let w = sim.world();
    let anc = w.get::<VecAnchors>(ent);
    if anc.is_some() || pai_tem::<VecFrame>(sim, ent) {
        out.push(k(c::ANCORAS_EXISTE, 0));
    }
    if anc.is_some() {
        out.extend((c::ANCORA_MIN_X..=ANCORA_ULTIMA).map(|campo| k(campo, 0)));
    }
    if fechado || w.get::<VecClipContent>(ent).is_some() {
        out.push(k(c::MOLDURA_RECORTA, 0));
    }
    let layout = w.get::<VecLayout>(ent);
    if layout.is_some() || w.get::<VecFrame>(ent).is_some() {
        out.push(k(c::LAYOUT_DIR, 0));
        for j in 0..2 {
            out.extend(
                [c::LAYOUT_TAMANHO, c::LAYOUT_TEM_MIN, c::LAYOUT_TEM_MAX].map(|campo| k(campo, j)),
            );
        }
        let s = w.get::<VecLayoutSize>(ent).copied().unwrap_or_default();
        for j in 0..2 {
            if s.min[j].is_some() {
                out.push(k(c::LAYOUT_MIN, j));
            }
            if s.max[j].is_some() {
                out.push(k(c::LAYOUT_MAX, j));
            }
        }
    }
    if layout.is_some() {
        out.extend((c::LAYOUT_VAO_X..=c::LAYOUT_COLUNAS).map(|campo| k(campo, 0)));
    }
    if pai_tem::<VecLayout>(sim, ent) {
        out.extend(
            [
                c::LAYOUT_CRESCE,
                c::LAYOUT_ENCOLHE,
                c::LAYOUT_TEM_BASE,
                c::LAYOUT_ABSOLUTO,
            ]
            .map(|campo| k(campo, 0)),
        );
        if w.get::<VecLayoutItem>(ent)
            .is_some_and(|it| it.basis.is_some())
        {
            out.push(k(c::LAYOUT_BASE, 0));
        }
    }
    if w.get::<VecWidget>(ent).is_some() {
        out.extend([k(c::WIDGET_PELE, 0), k(c::WIDGET_ICONE, 0)]);
    }
}

/// Quanto vale `kk` — `None` se o componente dele não existe (os neutros-por-ausência valem o
/// neutro). O ícone é um texto: o índice em `textos`.
pub fn ler(sim: &SimWorld, ent: Entity, kk: FieldKey, textos: &mut TextosDosCanais) -> Option<f32> {
    let w = sim.world();
    let j = usize::from(kk.j);
    let l = || w.get::<VecLayout>(ent);
    let s = || w.get::<VecLayoutSize>(ent).copied().unwrap_or_default();
    let it = || w.get::<VecLayoutItem>(ent).copied().unwrap_or_default();
    Some(match kk.campo {
        c::ANCORAS_EXISTE => b(w.get::<VecAnchors>(ent).is_some()),
        c::ANCORA_MIN_X..=ANCORA_ULTIMA => {
            let a = w.get::<VecAnchors>(ent)?;
            f(match kk.campo {
                c::ANCORA_MIN_X => a.min[0],
                c::ANCORA_MIN_Y => a.min[1],
                c::ANCORA_MAX_X => a.max[0],
                c::ANCORA_MAX_Y => a.max[1],
                x => a.base[usize::from(x - c::ANCORA_BASE)],
            })
        }
        c::MOLDURA_RECORTA => b(w.get::<VecClipContent>(ent).is_some()),
        c::LAYOUT_DIR => l().map_or(0.0, |l| pos(&DIRS, &l.dir) + 1.0),
        c::LAYOUT_VAO_X | c::LAYOUT_VAO_Y => f(l()?.gap[usize::from(kk.campo - c::LAYOUT_VAO_X)]),
        x @ c::LAYOUT_MARGEM..c::LAYOUT_ALINHA => f(l()?.pad[usize::from(x - c::LAYOUT_MARGEM)]),
        c::LAYOUT_ALINHA => pos(&ALINHA, &l()?.align),
        c::LAYOUT_DISTRIBUI => pos(&DISTRIBUI, &l()?.justify),
        c::LAYOUT_COLUNAS => f32::from(l()?.columns),
        c::LAYOUT_CRESCE => it().grow,
        c::LAYOUT_ENCOLHE => it().shrink,
        c::LAYOUT_TEM_BASE => b(it().basis.is_some()),
        c::LAYOUT_BASE => f(it().basis?),
        c::LAYOUT_TAMANHO => b(*s().size.get(j)? == LayoutSize::Hug),
        c::LAYOUT_TEM_MIN => b(s().min.get(j)?.is_some()),
        c::LAYOUT_MIN => f((*s().min.get(j)?)?),
        c::LAYOUT_TEM_MAX => b(s().max.get(j)?.is_some()),
        c::LAYOUT_MAX => f((*s().max.get(j)?)?),
        c::LAYOUT_ABSOLUTO => b(w.get::<VecLayoutAbsolute>(ent).is_some()),
        c::WIDGET_PELE => f32::from(w.get::<VecWidget>(ent)?.kind),
        c::WIDGET_ICONE => {
            w.get::<VecWidget>(ent)?;
            textos.indice(w.get::<VecWidgetIcon>(ent).map(|i| i.slug.as_str()))
        }
        _ => return None,
    })
}

/// Aplica `v` a `kk` com as leis do painel (`vec_anchor_edit`, `vec_clip_edit`, `layout_edit`,
/// `widget_edit`). Os interruptores de presença inserem/removem o componente; um campo de um
/// componente ausente não o inventa.
pub fn escrever(sim: &mut SimWorld, ent: Entity, kk: FieldKey, v: f32, textos: &TextosDosCanais) {
    if !e_da_interface(kk) {
        return;
    }
    let liga = v >= 0.5;
    let d = f64::from(v);
    let j = usize::from(kk.j).min(1);
    if (c::LAYOUT_CRESCE..=c::LAYOUT_MAX).contains(&kk.campo) {
        escrever_neutros(sim, ent, kk.campo, j, v);
        return;
    }
    let Ok(mut em) = sim.world_mut().get_entity_mut(ent) else {
        return;
    };
    match kk.campo {
        // A base é a régua que o painel captura ao armar; a curva dela aplica-se a seguir.
        c::ANCORAS_EXISTE => match (liga, em.contains::<VecAnchors>()) {
            (true, false) => {
                em.insert(VecAnchors::armed([0.0; 4]));
            }
            (false, true) => {
                em.remove::<VecAnchors>();
            }
            _ => {}
        },
        c::ANCORA_MIN_X..=ANCORA_ULTIMA => {
            let Some(mut a) = em.get_mut::<VecAnchors>() else {
                return;
            };
            // As fracções não se arredondam (o painel oferece `0,5`): o «discreto» é a chave
            // `Hold`, e o domínio é `0..=1`.
            match kk.campo {
                c::ANCORA_MIN_X => a.min[0] = d.clamp(0.0, 1.0),
                c::ANCORA_MIN_Y => a.min[1] = d.clamp(0.0, 1.0),
                c::ANCORA_MAX_X => a.max[0] = d.clamp(0.0, 1.0),
                c::ANCORA_MAX_Y => a.max[1] = d.clamp(0.0, 1.0),
                x => a.base[usize::from(x - c::ANCORA_BASE)] = d,
            }
        }
        c::MOLDURA_RECORTA => match (liga, em.contains::<VecClipContent>()) {
            (true, false) => {
                em.insert(VecClipContent);
            }
            (false, true) => {
                em.remove::<VecClipContent>();
            }
            _ => {}
        },
        c::LAYOUT_DIR => match indice(v, DIRS.len()).checked_sub(1) {
            None => {
                em.remove::<VecLayout>();
            }
            Some(i) => {
                let mut l = em.get::<VecLayout>().copied().unwrap_or_default();
                l.dir = DIRS[i];
                em.insert(l);
            }
        },
        c::LAYOUT_VAO_X..=c::LAYOUT_COLUNAS => {
            let Some(mut l) = em.get_mut::<VecLayout>() else {
                return;
            };
            match kk.campo {
                c::LAYOUT_VAO_X | c::LAYOUT_VAO_Y => {
                    l.gap[usize::from(kk.campo - c::LAYOUT_VAO_X)] = d.max(0.0);
                }
                x @ c::LAYOUT_MARGEM..c::LAYOUT_ALINHA => {
                    l.pad[usize::from(x - c::LAYOUT_MARGEM)] = d.max(0.0);
                }
                c::LAYOUT_ALINHA => l.align = ALINHA[indice(v, ALINHA.len() - 1)],
                c::LAYOUT_DISTRIBUI => l.justify = DISTRIBUI[indice(v, DISTRIBUI.len() - 1)],
                _ => {
                    let max = usize::from(ph2d_vec_layout::MAX_GRID_TRACKS);
                    l.columns = u16::try_from(indice(v, max).max(1)).unwrap_or(u16::MAX);
                }
            }
        }
        c::LAYOUT_ABSOLUTO => match (liga, em.contains::<VecLayoutAbsolute>()) {
            (true, false) => {
                em.insert(VecLayoutAbsolute);
            }
            (false, true) => {
                em.remove::<VecLayoutAbsolute>();
            }
            _ => {}
        },
        c::WIDGET_PELE => {
            // Só uma pele que o app conhece (a lei do `widget_edit::Kind`).
            let code = u16::try_from(indice(v, usize::from(u16::MAX))).unwrap_or(u16::MAX);
            if WidgetKind::from_code(code).is_some()
                && let Some(mut wd) = em.get_mut::<VecWidget>()
            {
                wd.kind = code;
            }
        }
        _ => {
            if !em.contains::<VecWidget>() {
                return;
            }
            match textos.texto(v) {
                Some(slug) if IconId::from_slug(slug).is_some() => {
                    em.insert(VecWidgetIcon {
                        slug: slug.to_owned(),
                    });
                }
                None if v.round() < 1.0 => {
                    em.remove::<VecWidgetIcon>();
                }
                _ => {}
            }
        }
    }
}

/// O filho do fluxo e os limites da moldura: componentes cujo NEUTRO é a ausência.
fn escrever_neutros(sim: &mut SimWorld, ent: Entity, campo: u16, j: usize, v: f32) {
    let liga = v >= 0.5;
    let d = f64::from(v);
    let w = sim.world();
    if campo <= c::LAYOUT_BASE {
        let cur = w.get::<VecLayoutItem>(ent).copied();
        let mut it = cur.unwrap_or_default();
        match campo {
            c::LAYOUT_CRESCE => it.grow = v.max(0.0),
            c::LAYOUT_ENCOLHE => it.shrink = v.max(0.0),
            c::LAYOUT_TEM_BASE => match (liga, it.basis) {
                (true, None) => it.basis = Some(0.0),
                (false, Some(_)) => it.basis = None,
                _ => {}
            },
            _ => match it.basis.as_mut() {
                Some(x) => *x = d.max(0.0),
                None => return,
            },
        }
        neutro_ausente(sim, ent, cur, it);
        return;
    }
    let cur = w.get::<VecLayoutSize>(ent).copied();
    let mut s = cur.unwrap_or_default();
    let opcao = |o: &mut Option<f64>| match (liga, *o) {
        (true, None) => *o = Some(0.0),
        (false, Some(_)) => *o = None,
        _ => {}
    };
    match campo {
        c::LAYOUT_TAMANHO => {
            s.size[j] = if liga {
                LayoutSize::Hug
            } else {
                LayoutSize::Fixed
            };
        }
        c::LAYOUT_TEM_MIN => opcao(&mut s.min[j]),
        c::LAYOUT_TEM_MAX => opcao(&mut s.max[j]),
        c::LAYOUT_MIN => match s.min[j].as_mut() {
            Some(x) => *x = d.max(0.0),
            None => return,
        },
        _ => match s.max[j].as_mut() {
            Some(x) => *x = d.max(0.0),
            None => return,
        },
    }
    neutro_ausente(sim, ent, cur, s);
}

/// O idioma do `layout_edit`: o componente neutro é a AUSÊNCIA dele.
fn neutro_ausente<T: ph2d_ecs::Component + Copy + Default + PartialEq>(
    sim: &mut SimWorld,
    ent: Entity,
    cur: Option<T>,
    next: T,
) {
    if cur == Some(next) {
        return;
    }
    let Ok(mut em) = sim.world_mut().get_entity_mut(ent) else {
        return;
    };
    if next == T::default() {
        em.remove::<T>();
    } else {
        em.insert(next);
    }
}

#[cfg(test)]
#[path = "canais_interface_tests.rs"]
mod tests;
