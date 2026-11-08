//! **Os campos do cartão SLICES** (`P11_oraculos_ferramenta.md` G3, G6; `P11_plano.md` §10): o que se
//! escreve vira um número pela lei do Trial; com várias selecionadas, o valor comum, ou vazio, e o que
//! se escreve vale para todas, num passo só.

use ph2d_pixel_core::slice::{default_center, duplicate};
use ph2d_pixel_core::{Op, Slice, SliceId, SliceShape, Sprite};

/// Um número do cartão.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Campo {
    X,
    Y,
    W,
    H,
    CentroX,
    CentroY,
    CentroW,
    CentroH,
    PivotX,
    PivotY,
}

impl Campo {
    const fn e_tamanho(self) -> bool {
        matches!(self, Self::W | Self::H | Self::CentroW | Self::CentroH)
    }
}

/// O inteiro no começo do texto: `12abc` dá 12, `1.5` dá 1; `abc` e vazio não dão nada (G6).
fn inteiro_no_comeco(t: &str) -> Option<i32> {
    let t = t.trim();
    let (sinal, resto) = match t.strip_prefix('-') {
        Some(r) => (-1_i64, r),
        None => (1, t),
    };
    let fim = resto
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(resto.len());
    let n: i64 = resto[..fim]
        .parse()
        .ok()
        .or_else(|| (fim > 0).then_some(i64::from(i32::MAX)))?;
    Some(
        sinal
            .saturating_mul(n)
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
    )
}

/// **O que o campo fica a valer** (G6): tamanho nunca abaixo de 1; posição e pivot aceitam negativos, e
/// vazio é 0.
#[must_use]
pub fn coage(c: Campo, t: &str) -> i32 {
    let v = inteiro_no_comeco(t);
    if c.e_tamanho() {
        v.unwrap_or(1).max(1)
    } else {
        v.unwrap_or(0)
    }
}

fn com_campo(mut k: SliceShape, c: Campo, v: i32) -> SliceShape {
    let tamanho = v.unsigned_abs();
    match c {
        Campo::X => k.bounds.x = v,
        Campo::Y => k.bounds.y = v,
        Campo::W => k.bounds.w = tamanho,
        Campo::H => k.bounds.h = tamanho,
        Campo::CentroX | Campo::CentroY | Campo::CentroW | Campo::CentroH => {
            if let Some(n) = &mut k.center {
                match c {
                    Campo::CentroX => n.x = v,
                    Campo::CentroY => n.y = v,
                    Campo::CentroW => n.w = tamanho,
                    _ => n.h = tamanho,
                }
            }
        }
        Campo::PivotX | Campo::PivotY => {
            if let Some(p) = &mut k.pivot {
                if c == Campo::PivotX {
                    p.0 = v;
                } else {
                    p.1 = v;
                }
            }
        }
    }
    k
}

fn editar(
    s: &Sprite,
    frame: u32,
    sel: &[SliceId],
    f: impl Fn(SliceShape) -> SliceShape,
) -> Option<Op> {
    let shapes: Vec<(SliceId, SliceShape)> = sel
        .iter()
        .filter_map(|id| Some((*id, f(s.slice(*id)?.shape_at(frame)?))))
        .collect();
    ph2d_pixel_core::slice::set_shapes(s, frame, &shapes)
}

/// Escrever `t` no campo `c`: a key deste quadro de cada selecionada. Quem não tem centro ou pivot fica
/// sem ele (G3).
#[must_use]
pub fn edita(s: &Sprite, frame: u32, sel: &[SliceId], c: Campo, t: &str) -> Option<Op> {
    let v = coage(c, t);
    editar(s, frame, sel, |k| com_campo(k, c, v))
}

/// Ligar ou desligar o `9-Slices`: ao ligar, margem de 1 (T3); quem já o tem fica como está.
#[must_use]
pub fn nove(s: &Sprite, frame: u32, sel: &[SliceId], on: bool) -> Option<Op> {
    editar(s, frame, sel, |k| SliceShape {
        center: on.then(|| k.center.unwrap_or_else(|| default_center(k.bounds))),
        ..k
    })
}

/// Ligar ou desligar o `Pivot`: ao ligar, no canto (T3); quem já o tem fica como está.
#[must_use]
pub fn pivot(s: &Sprite, frame: u32, sel: &[SliceId], on: bool) -> Option<Op> {
    editar(s, frame, sel, |k| SliceShape {
        pivot: on.then(|| k.pivot.unwrap_or((0, 0))),
        ..k
    })
}

fn em_todas(s: &Sprite, sel: &[SliceId], f: impl Fn(&mut Slice)) -> Option<Op> {
    let mut slices = s.slices().to_vec();
    for c in slices.iter_mut().filter(|c| sel.contains(&c.id)) {
        f(c);
    }
    (slices != s.slices()).then_some(Op::Slices(slices))
}

/// O nome vale para a slice inteira (G6) e, com várias, para todas (G3).
#[must_use]
pub fn nome(s: &Sprite, sel: &[SliceId], t: &str) -> Option<Op> {
    em_todas(s, sel, |c| t.clone_into(&mut c.name))
}

/// A cor, como o nome.
#[must_use]
pub fn cor(s: &Sprite, sel: &[SliceId], rgba: [u8; 4]) -> Option<Op> {
    em_todas(s, sel, |c| c.user.color = rgba)
}

/// O texto do utilizador, como o nome.
#[must_use]
pub fn dados(s: &Sprite, sel: &[SliceId], t: &str) -> Option<Op> {
    em_todas(s, sel, |c| t.clone_into(&mut c.user.text))
}

/// `Duplicate Slice` de todas as selecionadas, num passo; devolve as cópias, a nova seleção (T2).
pub fn duplica(s: &mut Sprite, sel: &[SliceId]) -> Option<(Op, Vec<SliceId>)> {
    let mut lista = s.slices().to_vec();
    let mut novas = Vec::new();
    for id in sel {
        let Some((Op::Slices(mut v), nova)) = duplicate(s, *id) else {
            continue;
        };
        lista.extend(v.pop());
        novas.push(nova);
    }
    (!novas.is_empty()).then_some((Op::Slices(lista), novas))
}

/// Há aqui uma mudança para remover? Só onde o quadro tem key própria e ela não é a única.
#[must_use]
pub fn pode_remover(s: &Sprite, frame: u32, sel: &[SliceId]) -> bool {
    s.slices()
        .iter()
        .filter(|c| sel.contains(&c.id))
        .any(|c| c.keys.len() > 1 && c.has_key_at(frame))
}

/// `Remove Change on This Frame` nas selecionadas que a têm, num passo.
#[must_use]
pub fn remove_mudanca(s: &Sprite, frame: u32, sel: &[SliceId]) -> Option<Op> {
    em_todas(s, sel, |c| {
        if c.keys.len() > 1 {
            c.keys.retain(|k| k.frame != frame);
        }
    })
}

/// Os quadros onde alguma selecionada tem key, em ordem (as marcas da linha do tempo, §9.2).
#[must_use]
pub fn quadros_com_key(s: &Sprite, sel: &[SliceId]) -> Vec<u32> {
    let mut v: Vec<u32> = s
        .slices()
        .iter()
        .filter(|c| sel.contains(&c.id))
        .flat_map(|c| c.keys.iter().map(|k| k.frame))
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// O valor comum, ou `None` se misto.
fn comum<T: PartialEq>(mut it: impl Iterator<Item = T>) -> Option<T> {
    let a = it.next()?;
    it.all(|b| b == a).then_some(a)
}

/// O que o cartão mostra das selecionadas; `None` é misto, e aparece vazio (§10).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Vista {
    pub nome: Option<String>,
    pub cor: Option<[u8; 4]>,
    pub dados: Option<String>,
    /// `x`, `y`, `w`, `h`.
    pub bounds: [Option<i32>; 4],
    /// Todas o têm ligado; se só algumas, os campos aparecem na mesma.
    pub nove: bool,
    pub algum_nove: bool,
    pub centro: [Option<i32>; 4],
    pub pivot: bool,
    pub algum_pivot: bool,
    pub ponto: [Option<i32>; 2],
}

/// As selecionadas no quadro `frame`; `None` sem seleção.
#[must_use]
pub fn vista(s: &Sprite, frame: u32, sel: &[SliceId]) -> Option<Vista> {
    let v: Vec<(&Slice, SliceShape)> = s
        .slices()
        .iter()
        .filter(|c| sel.contains(&c.id))
        .filter_map(|c| Some((c, c.shape_at(frame)?)))
        .collect();
    if v.is_empty() {
        return None;
    }
    // Comum só se TODAS o têm: a que não tem centro (ou pivot) torna o número misto.
    let de = |f: &dyn Fn(&SliceShape) -> Option<i32>| comum(v.iter().map(|(_, k)| f(k))).flatten();
    Some(Vista {
        nome: comum(v.iter().map(|(c, _)| c.name.clone())),
        cor: comum(v.iter().map(|(c, _)| c.draw_color())),
        dados: comum(v.iter().map(|(c, _)| c.user.text.clone())),
        bounds: [
            de(&|k| Some(k.bounds.x)),
            de(&|k| Some(k.bounds.y)),
            de(&|k| Some(k.bounds.w as i32)),
            de(&|k| Some(k.bounds.h as i32)),
        ],
        nove: v.iter().all(|(_, k)| k.center.is_some()),
        algum_nove: v.iter().any(|(_, k)| k.center.is_some()),
        centro: [
            de(&|k| k.center.map(|c| c.x)),
            de(&|k| k.center.map(|c| c.y)),
            de(&|k| k.center.map(|c| c.w as i32)),
            de(&|k| k.center.map(|c| c.h as i32)),
        ],
        pivot: v.iter().all(|(_, k)| k.pivot.is_some()),
        algum_pivot: v.iter().any(|(_, k)| k.pivot.is_some()),
        ponto: [de(&|k| k.pivot.map(|p| p.0)), de(&|k| k.pivot.map(|p| p.1))],
    })
}
