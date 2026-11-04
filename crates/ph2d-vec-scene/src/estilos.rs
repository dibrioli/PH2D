//! **Estilos de cor do projeto** (CartoesUI D28/D30, maquete `docs/CartoesUI/maquete/estilos.png`):
//! uma cor com nome, guardada NA CENA, LIGADA como no Figma — editar o estilo muda toda camada que o
//! usa; apagá-lo deixa as camadas com a cor. A ligação é a camada da pilha pelo endereço durável
//! `(path, ItemId)`, guardada no estilo (a `PaintEntry` não muda de forma).
//!
//! ⭐ **A tinta de BASE liga-se a si própria** com os ids reservados [`ItemId::BASE_FILL`] e
//! [`ItemId::BASE_STROKE`] (promovê-la a camada deixaria a barra FILL de cima vazia). A opacidade da
//! base é a alfa da cor dela — o campo ao lado da linha, como o da camada —: o estilo manda só no
//! RGB dela, e mexer na Opacity não a solta.

use std::collections::BTreeMap;

use ph2d_item_id::ItemId;
use serde::{Deserialize, Serialize};

use crate::paint_stack::{MAX_PAINT_LAYERS, PaintKind};
use crate::{Paint, Rgba8, StrokePaint, VecPath, VecPathId, VecScene};

/// O endereço de um estilo (nunca reusado na cena; `0` não existe).
pub type EstiloId = u32;

/// Uma linha de tinta ligada a um estilo: uma camada da pilha, ou uma tinta de base.
pub type Uso = (VecPathId, ItemId);

/// ⭐ **A linha de um cartão de tinta**: `0..MAX_PAINT_LAYERS` são as camadas e estas duas as bases —
/// a casa única desses índices para a cena, a lista de estilos e o Inspector.
pub const LINHA_BASE_FILL: usize = MAX_PAINT_LAYERS;
pub const LINHA_BASE_STROKE: usize = LINHA_BASE_FILL + 1;
/// Quantas linhas há, contadas as duas bases.
pub const LINHAS_DA_PILHA: usize = LINHA_BASE_STROKE + 1;

impl VecPath {
    /// O item que a linha `linha` liga a um estilo (`None`: não há a linha, ou a camada não tem id).
    #[must_use]
    pub fn item_da_linha(&self, linha: usize) -> Option<ItemId> {
        match linha {
            LINHA_BASE_FILL => self.base_fill().map(|_| ItemId::BASE_FILL),
            LINHA_BASE_STROKE => self.base_stroke().map(|_| ItemId::BASE_STROKE),
            _ => self
                .paints
                .get(linha)
                .map(|e| e.id)
                .filter(|id| !id.is_none()),
        }
    }

    /// A cor que a amostra da linha `linha` mostra.
    #[must_use]
    pub fn cor_da_linha(&self, linha: usize) -> Option<Rgba8> {
        match linha {
            LINHA_BASE_FILL => self.base_fill().map(Paint::primary_color),
            LINHA_BASE_STROKE => self.base_stroke().map(crate::StrokeSpec::color),
            _ => self.paints.get(linha).map(crate::PaintEntry::swatch_color),
        }
    }

    /// A linha do item `item` existe neste caminho?
    fn tem_item(&self, item: ItemId) -> bool {
        match item {
            ItemId::BASE_FILL => self.base_fill().is_some(),
            ItemId::BASE_STROKE => self.base_stroke().is_some(),
            _ => self.paints.iter().any(|e| e.id == item),
        }
    }

    /// A tinta de base `item` ainda é a cor sólida com o RGB de `c`?
    fn base_casa(&self, item: ItemId, c: Rgba8) -> bool {
        let rgb = |x: &Rgba8| (x.r, x.g, x.b) == (c.r, c.g, c.b);
        if item == ItemId::BASE_FILL {
            matches!(self.base_fill(), Some(Paint::Solid(x)) if rgb(x))
        } else {
            self.base_stroke()
                .is_some_and(|s| matches!(&s.paint, StrokePaint::Solid(x) if rgb(x)))
        }
    }
}

/// Um estilo de cor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColorStyle {
    pub id: EstiloId,
    pub name: String,
    pub color: Rgba8,
    /// As camadas ligadas (cada uma num estilo só).
    pub usos: Vec<Uso>,
}

impl VecScene {
    /// Os estilos, pela ordem em que nasceram.
    #[must_use]
    pub fn estilos(&self) -> &[ColorStyle] {
        &self.estilos
    }

    #[must_use]
    pub fn estilo(&self, id: EstiloId) -> Option<&ColorStyle> {
        self.estilos.iter().find(|e| e.id == id)
    }

    /// Guarda `color` como estilo novo; devolve o id.
    pub fn add_estilo(&mut self, name: &str, color: Rgba8) -> EstiloId {
        self.next_estilo = self.next_estilo.max(1);
        let id = self.next_estilo;
        self.next_estilo += 1;
        self.estilos.push(ColorStyle {
            id,
            name: name.to_owned(),
            color,
            usos: Vec::new(),
        });
        id
    }

    pub fn rename_estilo(&mut self, id: EstiloId, name: &str) -> bool {
        match self.estilos.iter_mut().find(|e| e.id == id) {
            Some(e) if e.name != name => {
                name.clone_into(&mut e.name);
                true
            }
            _ => false,
        }
    }

    /// Uma cópia sem usos, logo a seguir ao original; devolve o id novo.
    pub fn duplicate_estilo(&mut self, id: EstiloId, name: &str) -> Option<EstiloId> {
        let i = self.estilos.iter().position(|e| e.id == id)?;
        let color = self.estilos[i].color;
        let novo = self.add_estilo(name, color);
        let e = self.estilos.pop()?;
        self.estilos.insert(i + 1, e);
        Some(novo)
    }

    /// Apaga o estilo; as camadas ficam com a cor que tinham.
    pub fn remove_estilo(&mut self, id: EstiloId) -> bool {
        let antes = self.estilos.len();
        self.estilos.retain(|e| e.id != id);
        self.estilos.len() != antes
    }

    /// ⭐ Muda a cor do estilo e de TODA camada ligada a ele (a lei do Figma).
    pub fn set_estilo_color(&mut self, id: EstiloId, color: Rgba8) -> bool {
        let Some(e) = self.estilos.iter_mut().find(|e| e.id == id) else {
            return false;
        };
        let mut mudou = e.color != color;
        e.color = color;
        let usos = e.usos.clone();
        for uso in usos {
            mudou |= self.pinta_uso(uso, color, false);
        }
        mudou
    }

    /// Liga a camada `uso` ao estilo (soltando-a de outro) e pinta-a com a cor dele.
    pub fn link_estilo(&mut self, uso: Uso, id: EstiloId) -> bool {
        let Some(color) = self.estilo(id).map(|e| e.color) else {
            return false;
        };
        let mut mudou = self.unlink_estilo(uso);
        if let Some(e) = self.estilos.iter_mut().find(|e| e.id == id) {
            e.usos.push(uso);
            mudou = true;
        }
        self.pinta_uso(uso, color, true) || mudou
    }

    /// ⭐ **Solta as tintas de base que deixaram de ser do estilo** (D30, a cor à mão): a base
    /// desapareceu (o uso morre: ao voltar ela nasce solta) ou deixou de ter o RGB dele — outra cor,
    /// um degradê, um padrão. Vale para TODO escritor do `fill`/`stroke`; corre a cada quadro antes da
    /// captura do undo (`ph2d_app_vec::ids_da_pilha::passe`), no MESMO passo da cor. Repintar por
    /// [`Self::set_estilo_color`] nunca solta: a base e o estilo mudam juntos. Devolve se soltou.
    pub fn solta_bases_mexidas(&mut self) -> bool {
        let de_base = |u: &Uso| u.1.reservado();
        if !self.estilos.iter().any(|e| e.usos.iter().any(de_base)) {
            return false;
        }
        let formas: BTreeMap<VecPathId, &VecPath> = self.paths.iter().map(|p| (p.id, p)).collect();
        let mut mudou = false;
        for e in &mut self.estilos {
            let cor = e.color;
            let antes = e.usos.len();
            e.usos
                .retain(|u| !de_base(u) || formas.get(&u.0).is_some_and(|p| p.base_casa(u.1, cor)));
            mudou |= e.usos.len() != antes;
        }
        mudou
    }

    /// Solta a camada `uso` do estilo dela (a cor fica).
    pub fn unlink_estilo(&mut self, uso: Uso) -> bool {
        let mut mudou = false;
        for e in &mut self.estilos {
            let antes = e.usos.len();
            e.usos.retain(|u| *u != uso);
            mudou |= e.usos.len() != antes;
        }
        mudou
    }

    /// O estilo a que a camada `uso` está ligada.
    #[must_use]
    pub fn estilo_de(&self, uso: Uso) -> Option<&ColorStyle> {
        self.estilos.iter().find(|e| e.usos.contains(&uso))
    }

    /// Quantas camadas VIVAS usam o estilo (as que saíram da cena não contam).
    #[must_use]
    pub fn usos_vivos(&self, id: EstiloId) -> usize {
        self.estilo(id).map_or(0, |e| {
            e.usos
                .iter()
                .filter(|(p, item)| self.path(*p).is_some_and(|p| p.tem_item(*item)))
                .count()
        })
    }

    fn pinta_uso(&mut self, (path, item): Uso, color: Rgba8, ligando: bool) -> bool {
        let Some(p) = self.path_mut(path) else {
            return false;
        };
        if item.reservado() {
            return pinta_base(p, item, color, ligando);
        }
        let Some(e) = p.paints.iter_mut().find(|e| e.id == item) else {
            return false;
        };
        match &mut e.kind {
            PaintKind::Fill(f) if *f != Paint::Solid(color) => {
                *f = Paint::Solid(color);
                true
            }
            PaintKind::Stroke(s) if s.paint != StrokePaint::Solid(color) => {
                s.paint = StrokePaint::Solid(color);
                true
            }
            _ => false,
        }
    }
}

/// O RGB do estilo na tinta de base `item`, à vista ou escondida. A alfa é a Opacity da linha e fica;
/// numa cor sem alfa (a «sem preenchimento») o LIGAR dá-lhe a do estilo, como a escolha no seletor.
fn pinta_base(p: &mut VecPath, item: ItemId, color: Rgba8, ligando: bool) -> bool {
    let junta = |atual: Option<Rgba8>| match atual {
        Some(a) if a.a != 0 || !ligando => Rgba8::new(color.r, color.g, color.b, a.a),
        _ => color,
    };
    if item == ItemId::BASE_FILL {
        let Some(f) = p.base_fill_mut() else {
            return false;
        };
        let atual = match f {
            Paint::Solid(c) => Some(*c),
            _ => None,
        };
        let novo = Paint::Solid(junta(atual));
        let mudou = *f != novo;
        *f = novo;
        mudou
    } else {
        let Some(s) = p.base_stroke_mut() else {
            return false;
        };
        let atual = match &s.paint {
            StrokePaint::Solid(c) => Some(*c),
            _ => None,
        };
        let novo = StrokePaint::Solid(junta(atual));
        let mudou = s.paint != novo;
        s.paint = novo;
        mudou
    }
}

#[cfg(test)]
#[path = "estilos_tests.rs"]
mod tests;
