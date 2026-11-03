//! **Estilos de cor do projeto** (CartoesUI D28/D30, maquete `docs/CartoesUI/maquete/estilos.png`):
//! uma cor com nome, guardada NA CENA, LIGADA como no Figma — editar o estilo muda toda camada que o
//! usa; apagá-lo deixa as camadas com a cor. A ligação é a camada da pilha pelo endereço durável
//! `(path, ItemId)`, guardada no estilo (a `PaintEntry` não muda de forma).

use ph2d_item_id::ItemId;
use serde::{Deserialize, Serialize};

use crate::paint_stack::PaintKind;
use crate::{Paint, Rgba8, StrokePaint, VecPathId, VecScene};

/// O endereço de um estilo (nunca reusado na cena; `0` não existe).
pub type EstiloId = u32;

/// Uma camada da pilha ligada a um estilo.
pub type Uso = (VecPathId, ItemId);

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
            mudou |= self.pinta_uso(uso, color);
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
        self.pinta_uso(uso, color) || mudou
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
                .filter(|(p, item)| {
                    self.path(*p)
                        .is_some_and(|p| p.paints.iter().any(|x| x.id == *item))
                })
                .count()
        })
    }

    fn pinta_uso(&mut self, (path, item): Uso, color: Rgba8) -> bool {
        let Some(p) = self.path_mut(path) else {
            return false;
        };
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

#[cfg(test)]
#[path = "estilos_tests.rs"]
mod tests;
