//! **O gesto**: premir, arrastar, largar. A seleção é do chamador e entra emprestada.

use ph2d_pixel_core::slice::{DEFAULT_SLICE_COLOR, create, set_shapes};
use ph2d_pixel_core::{Op, Rect, Slice, SliceId, SliceShape, Sprite};

use crate::alcas::{Grab, aplica, hit};

/// O que o premir começou.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Tipo {
    /// Arrasto em vazio: cria ao largar.
    Criar,
    /// `Shift`+arrasto: seleciona as que toca (T2).
    Marquise,
    /// Mexe em `base` com a alça `grab`; `tocada` é a slice sob o ponteiro.
    Editar {
        grab: Grab,
        tocada: SliceId,
        base: Vec<(SliceId, SliceShape)>,
    },
}

/// **Um gesto em curso.** O documento não muda até [`Gesto::release`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gesto {
    tipo: Tipo,
    from: (i32, i32),
    to: (i32, i32),
    /// O ponteiro já mudou de célula: largar não é um clique.
    moveu: bool,
}

/// As slices visíveis no quadro, a de cima primeiro.
fn visiveis(s: &Sprite, frame: u32) -> impl Iterator<Item = (&Slice, SliceShape)> {
    s.slices()
        .iter()
        .rev()
        .filter_map(move |c| c.visible_at(frame).map(|k| (c, k)))
}

/// A slice cujo INTERIOR tem a célula `at`, a de cima primeiro. Uma slice que passa do canvas só se
/// vê dentro dele, mas a caixa é a dela (F3).
#[must_use]
pub fn interior(s: &Sprite, frame: u32, at: (i32, i32)) -> Option<SliceId> {
    visiveis(s, frame)
        .find(|(_, k)| k.bounds.contains(at.0, at.1))
        .map(|(c, _)| c.id)
}

fn celula(p: (f64, f64)) -> (i32, i32) {
    (p.0.floor() as i32, p.1.floor() as i32)
}

/// A caixa entre duas células: a borda final é a da célula do ponteiro, sem a contar (T1: 8 → 30 dá
/// largura 22); a marquise conta-a, porque toca o que o ponteiro toca (T2).
fn caixa(a: (i32, i32), b: (i32, i32), inclui_fim: u32) -> Rect {
    let (x0, x1) = (a.0.min(b.0), a.0.max(b.0));
    let (y0, y1) = (a.1.min(b.1), a.1.max(b.1));
    Rect::new(
        x0,
        y0,
        x1.abs_diff(x0) + inclui_fim,
        y1.abs_diff(y0) + inclui_fim,
    )
}

/// O índice do quadro de um documento, ou 0 (o quadro activo existe sempre).
#[must_use]
pub fn no_quadro(s: &Sprite, frame: ph2d_pixel_core::FrameId) -> u32 {
    s.frame_index(frame).unwrap_or(0) as u32
}

impl Gesto {
    /// **Premir em `p`** (células contínuas), com `tol` células de alcance nas alças. Muda a seleção
    /// como o Trial: premir numa não selecionada substitui (T1); premir em vazio limpa.
    #[must_use]
    pub fn press(
        s: &Sprite,
        frame: u32,
        sel: &mut Vec<SliceId>,
        p: (f64, f64),
        tol: f64,
        shift: bool,
    ) -> Self {
        let at = celula(p);
        let tipo = if shift {
            Tipo::Marquise
        } else if let Some(t) = Self::agarra(s, frame, sel, p, at, tol) {
            t
        } else {
            sel.clear();
            Tipo::Criar
        };
        Self {
            tipo,
            from: at,
            to: at,
            moveu: false,
        }
    }

    fn agarra(
        s: &Sprite,
        frame: u32,
        sel: &mut Vec<SliceId>,
        p: (f64, f64),
        at: (i32, i32),
        tol: f64,
    ) -> Option<Tipo> {
        // As alças das selecionadas primeiro: podem estar fora da caixa.
        let alca = visiveis(s, frame)
            .filter(|(c, _)| sel.contains(&c.id))
            .find_map(|(c, k)| match hit(&k, p, tol) {
                Some(Grab::Move) | None => None,
                Some(g) => Some((c.id, g)),
            });
        let (tocada, grab) = match alca {
            Some(a) => a,
            None => {
                let id = interior(s, frame, at)?;
                if !sel.contains(&id) {
                    sel.clear();
                    sel.push(id);
                }
                (id, Grab::Move)
            }
        };
        // Mover e as bordas valem para toda a seleção (G3, §10); o 9-slice e o pivot são de cada uma.
        let todas = matches!(grab, Grab::Move | Grab::Bounds(_));
        let base = visiveis(s, frame)
            .filter(|(c, _)| {
                if todas {
                    sel.contains(&c.id)
                } else {
                    c.id == tocada
                }
            })
            .map(|(c, k)| (c.id, k))
            .collect();
        Some(Tipo::Editar { grab, tocada, base })
    }

    /// O ponteiro está em `p`.
    pub fn drag(&mut self, p: (f64, f64)) {
        let at = celula(p);
        self.moveu |= at != self.from;
        self.to = at;
    }

    /// As formas a desenhar em vez das do documento, a meio do arrasto.
    #[must_use]
    pub fn previa(&self) -> Vec<(SliceId, SliceShape)> {
        let Tipo::Editar { grab, base, .. } = &self.tipo else {
            return Vec::new();
        };
        let (dx, dy) = (
            self.to.0.saturating_sub(self.from.0),
            self.to.1.saturating_sub(self.from.1),
        );
        base.iter()
            .map(|(id, k)| (*id, aplica(*k, *grab, dx, dy)))
            .collect()
    }

    /// A caixa elástica de criar (`canvas` corta-a, como o largar) ou da marquise, em células.
    #[must_use]
    pub fn elastico(&self, canvas: Rect) -> Option<Rect> {
        match self.tipo {
            Tipo::Criar if self.moveu => Some(caixa(self.from, self.to, 0).intersect(canvas)),
            Tipo::Marquise => Some(caixa(self.from, self.to, 1)),
            _ => None,
        }
    }

    /// **Largar**: o passo do desfazer, se o documento muda. Criar não seleciona a nova (T1). Um
    /// clique numa slice deixa só ela selecionada (T2).
    pub fn release(self, s: &mut Sprite, frame: u32, sel: &mut Vec<SliceId>) -> Option<Op> {
        let previa = self.previa();
        match self.tipo {
            Tipo::Criar => create(s, frame, caixa(self.from, self.to, 0)).map(|(op, _)| op),
            Tipo::Marquise => {
                let area = caixa(self.from, self.to, 1);
                let toca: Vec<SliceId> = s
                    .slices()
                    .iter()
                    .filter(|c| {
                        c.visible_at(frame)
                            .is_some_and(|k| !k.bounds.intersect(area).is_empty())
                    })
                    .map(|c| c.id)
                    .collect();
                *sel = toca;
                None
            }
            Tipo::Editar { tocada, .. } => {
                if !self.moveu {
                    sel.clear();
                    sel.push(tocada);
                }
                set_shapes(s, frame, &previa)
            }
        }
    }
}

/// Uma slice como se desenha neste quadro, com a prévia do gesto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marca {
    pub id: SliceId,
    pub shape: SliceShape,
    pub color: [u8; 4],
    pub selected: bool,
    pub name: String,
}

/// **As slices a desenhar no quadro**, por baixo primeiro; sem cor no ficheiro, a padrão (§4.2).
#[must_use]
pub fn marcas(s: &Sprite, frame: u32, g: Option<&Gesto>, sel: &[SliceId]) -> Vec<Marca> {
    let previa = g.map_or_else(Vec::new, Gesto::previa);
    s.slices()
        .iter()
        .filter_map(|c| {
            let vista = previa.iter().find(|(id, _)| *id == c.id).map(|(_, k)| *k);
            Some(Marca {
                id: c.id,
                shape: vista.or_else(|| c.visible_at(frame))?,
                color: c.color.unwrap_or(DEFAULT_SLICE_COLOR),
                selected: sel.contains(&c.id),
                name: c.name.clone(),
            })
        })
        .collect()
}
