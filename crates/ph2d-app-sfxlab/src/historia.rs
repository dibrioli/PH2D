//! O desfazer de cada aba: instantâneos de `Recipe`, e um gesto contínuo vale um passo.

use ph2d_sfx_recipe::Recipe;

/// O recurso é a RAM: 1 000 passos do maior som de fábrica (Impact, 4 camadas) cabem em 1 MiB por aba
/// (gate `mil_passos_cabem_num_mega`).
pub const MAX_PASSOS: usize = 1000;

#[derive(Default)]
pub struct Historia {
    antes: Vec<Recipe>,
    depois: Vec<Recipe>,
    gesto: Option<Recipe>,
}

impl Historia {
    /// Uma edição fora de gesto já feita; `antes` é a receita de antes dela.
    pub fn anota(&mut self, antes: Recipe) {
        if self.gesto.is_none() {
            self.empilha(antes);
        }
    }

    pub fn abre_gesto(&mut self, atual: &Recipe) {
        self.gesto = Some(atual.clone());
    }

    #[must_use]
    pub fn em_gesto(&self) -> bool {
        self.gesto.is_some()
    }

    /// `true` = o gesto mudou o documento.
    pub fn fecha_gesto(&mut self, atual: &Recipe) -> bool {
        match self.gesto.take() {
            Some(antes) if antes != *atual => {
                self.empilha(antes);
                true
            }
            _ => false,
        }
    }

    pub fn desfaz(&mut self, atual: &Recipe) -> Option<Recipe> {
        self.fecha_gesto(atual);
        let r = self.antes.pop()?;
        self.depois.push(atual.clone());
        Some(r)
    }

    pub fn refaz(&mut self, atual: &Recipe) -> Option<Recipe> {
        self.fecha_gesto(atual);
        let r = self.depois.pop()?;
        self.antes.push(atual.clone());
        Some(r)
    }

    pub fn zera(&mut self) {
        *self = Self::default();
    }

    fn empilha(&mut self, antes: Recipe) {
        self.depois.clear();
        self.antes.push(antes);
        if self.antes.len() > MAX_PASSOS {
            self.antes.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mil_passos_cabem_num_mega() {
        let impact = ph2d_sfx_recipe::factory::all()
            .into_iter()
            .find(|(n, _)| *n == "Impact")
            .map(|(_, r)| r)
            .expect("Impact");
        let pontos: usize = impact.layers.iter().map(|l| l.envelope.points.len()).sum();
        let um = size_of::<Recipe>()
            + impact.layers.len() * size_of::<ph2d_sfx_recipe::Layer>()
            + pontos * size_of::<ph2d_sfx_recipe::EnvPoint>();
        assert!(um * MAX_PASSOS <= 1 << 20, "{um} bytes por passo");
    }

    #[test]
    fn o_gesto_vale_um_passo_e_o_gesto_que_nao_mudou_nada_nao_vale() {
        let mut h = Historia::default();
        let a = Recipe::new(1, 1.0);
        let mut b = a.clone();
        h.abre_gesto(&a);
        assert!(!h.fecha_gesto(&a));
        assert!(h.desfaz(&a).is_none());
        h.abre_gesto(&a);
        b.seconds = 2.0;
        h.anota(a.clone());
        assert!(h.fecha_gesto(&b));
        assert_eq!(h.desfaz(&b), Some(a.clone()));
        assert_eq!(h.refaz(&a), Some(b));
    }
}
