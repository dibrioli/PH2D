//! ⭐ **O jogo corre sobre uma CÓPIA** (R0a): quando a corrida entra, o documento é fotografado;
//! quando sai, ele volta ao bit. É o *Play Mode* do Unity — o que muda a jogar não fica. Pausar não
//! repõe: só a saída da corrida.
//!
//! A sessão é genérica na fotografia (`S`): quem a tira e quem a repõe é a shell, pela mesma porta do
//! undo e do save. Aqui mora só o QUANDO.

/// O que a shell faz neste quadro.
#[derive(Debug, PartialEq, Eq)]
pub enum Passo<S> {
    Nada,
    /// A corrida entrou: fotografe o documento ANTES de qualquer motor andar e entregue-o a
    /// [`Sessao::guarda`].
    Fotografa,
    /// A corrida saiu: reponha esta fotografia.
    Repoe(S),
}

#[derive(Debug)]
pub struct Sessao<S> {
    corria: bool,
    foto: Option<S>,
}

impl<S> Default for Sessao<S> {
    fn default() -> Self {
        Self {
            corria: false,
            foto: None,
        }
    }
}

impl<S> Sessao<S> {
    /// Uma vez por quadro, com a resposta a *«a cena está a correr?»*, antes dos motores.
    pub fn quadro(&mut self, corre: bool) -> Passo<S> {
        let antes = std::mem::replace(&mut self.corria, corre);
        match (antes, corre) {
            (false, true) => Passo::Fotografa,
            (true, false) => self.foto.take().map_or(Passo::Nada, Passo::Repoe),
            _ => Passo::Nada,
        }
    }

    /// A fotografia pedida por [`Passo::Fotografa`]. Fora de uma corrida é ignorada.
    pub fn guarda(&mut self, foto: S) {
        if self.corria {
            self.foto = Some(foto);
        }
    }

    /// Com o jogo a correr o undo não regista: o que o jogo muda não é do artista.
    #[must_use]
    pub fn a_correr(&self) -> bool {
        self.corria
    }

    /// Outro documento entrou (abrir, novo): a fotografia já não é dele e a saída não repõe nada.
    pub fn esquece(&mut self) {
        self.foto = None;
    }
}

#[cfg(test)]
#[path = "sessao_tests.rs"]
mod tests;
