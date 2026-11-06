//! **A janela que escolhe um objeto da cena:** busca, lista e conta-gotas, pintada dentro de uma aba de família.
//! O fundo escurece só a aba: os painéis ficam vivos, porque o conta-gotas escolhe na Hierarchy.
//! A janela não conhece o mundo: recebe os objetos e responde com o id ([`Saida`]).

mod paint;

use ph2d_editor_core::doc_surface::{Compose, Key, KeyInput, PointerLook};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_kanban_views::edit::{Field, Outcome, Target};

/// O que a janela sabe receber. O objeto vai pelo id que quem a abriu deu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alvo {
    Busca,
    ContaGotas,
    Cancelar,
    Objeto(u64),
}

/// O que quem abriu a janela tem de fazer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Saida {
    Nada,
    Fecha,
    Escolheu(u64),
}

/// O que só quem abriu sabe dizer.
#[derive(Clone, Copy, Debug)]
pub struct Textos<'a> {
    pub titulo: &'a str,
    pub subtitulo: &'a str,
    /// Uma linha de aviso no lugar do subtítulo.
    pub aviso: Option<&'a str>,
}

pub struct ObjectPicker {
    objetos: Vec<(u64, String)>,
    atual: Option<u64>,
    busca: Field,
    /// Os índices em `objetos` que a busca deixa ver, os que começam pela busca primeiro.
    vistos: Vec<usize>,
    /// Índice em `vistos`.
    destaque: usize,
    ini: usize,
    linhas: usize,
    armado: bool,
    prensa: Option<Alvo>,
    hover: Option<Alvo>,
    alvos: Vec<(Alvo, Rect)>,
    caixa: Rect,
    lista: Rect,
}

fn filtra(objetos: &[(u64, String)], busca: &str) -> Vec<usize> {
    let q = busca.trim().to_lowercase();
    let nomes: Vec<String> = objetos.iter().map(|(_, n)| n.to_lowercase()).collect();
    let quem = |comeca: bool| {
        let q = &q;
        nomes.iter().enumerate().filter_map(move |(i, n)| {
            (n.starts_with(q.as_str()) == comeca && n.contains(q.as_str())).then_some(i)
        })
    };
    quem(true).chain(quem(false)).collect()
}

impl ObjectPicker {
    /// `atual` nasce destacado: Enter confirma o objeto de antes.
    #[must_use]
    pub fn new(objetos: Vec<(u64, String)>, atual: Option<u64>) -> Self {
        let nada = Rect::new(0.0, 0.0, 0.0, 0.0);
        let vistos = filtra(&objetos, "");
        let destaque = atual
            .and_then(|id| vistos.iter().position(|&i| objetos[i].0 == id))
            .unwrap_or(0);
        Self {
            objetos,
            atual,
            busca: Field::open(Target::Scratch, ""),
            vistos,
            destaque,
            ini: destaque,
            linhas: 1,
            armado: false,
            prensa: None,
            hover: None,
            alvos: Vec::new(),
            caixa: nada,
            lista: nada,
        }
    }

    /// O conta-gotas espera um clique na Hierarchy.
    #[must_use]
    pub fn armado(&self) -> bool {
        self.armado
    }

    pub fn desarma(&mut self) {
        self.armado = false;
    }

    /// Os nomes que a busca deixa ver, na ordem da lista.
    #[must_use]
    pub fn nomes_vistos(&self) -> Vec<&str> {
        self.vistos
            .iter()
            .map(|&i| self.objetos[i].1.as_str())
            .collect()
    }

    #[must_use]
    pub fn destacado(&self) -> Option<u64> {
        let i = *self.vistos.get(self.destaque)?;
        Some(self.objetos[i].0)
    }

    /// O que a última pintura registrou.
    pub fn alvos(&self) -> impl Iterator<Item = (Alvo, Rect)> + '_ {
        self.alvos.iter().copied()
    }

    fn alvo_em(&self, x: f32, y: f32) -> Option<Alvo> {
        self.alvos
            .iter()
            .rev()
            .find(|(_, r)| r.contains(x, y))
            .map(|(a, _)| *a)
    }

    pub fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) -> Saida {
        let alvo = self.alvo_em(x, y);
        match kind {
            PointerKind::Move => {
                self.hover = alvo;
                if let Some(Alvo::Objeto(id)) = alvo
                    && let Some(i) = self.vistos.iter().position(|&i| self.objetos[i].0 == id)
                {
                    self.destaque = i;
                }
                Saida::Nada
            }
            PointerKind::Down => {
                if !self.caixa.contains(x, y) {
                    return Saida::Fecha;
                }
                self.prensa = alvo;
                Saida::Nada
            }
            PointerKind::Up => {
                let prensa = self.prensa.take();
                if prensa.is_some() && prensa == alvo {
                    self.clique(prensa)
                } else {
                    Saida::Nada
                }
            }
        }
    }

    fn clique(&mut self, alvo: Option<Alvo>) -> Saida {
        match alvo {
            Some(Alvo::Objeto(id)) => Saida::Escolheu(id),
            Some(Alvo::ContaGotas) => {
                self.armado = !self.armado;
                Saida::Nada
            }
            Some(Alvo::Cancelar) => Saida::Fecha,
            _ => Saida::Nada,
        }
    }

    pub fn roda(&mut self, x: f32, y: f32, dy: f32) {
        if self.lista.contains(x, y) {
            self.ini = if dy > 0.0 {
                self.ini.saturating_sub(1)
            } else {
                self.ini + 1
            };
            self.prende_a_rolagem();
        }
    }

    fn prende_a_rolagem(&mut self) {
        self.ini = self.ini.min(self.vistos.len().saturating_sub(self.linhas));
    }

    fn anda(&mut self, passo: isize) {
        let fim = self.vistos.len().saturating_sub(1);
        self.destaque = self.destaque.saturating_add_signed(passo).min(fim);
        if self.destaque < self.ini {
            self.ini = self.destaque;
        } else if self.destaque >= self.ini + self.linhas {
            self.ini = self.destaque + 1 - self.linhas;
        }
    }

    fn apos_escrever(&mut self, antes: &str) {
        if self.busca.text != antes {
            self.vistos = filtra(&self.objetos, &self.busca.text);
            self.destaque = 0;
            self.ini = 0;
        }
    }

    /// A tecla: quem tem a janela aberta não deixa nenhuma descer para a tela. `None` = não é da janela (um atalho do app).
    pub fn tecla(&mut self, k: &KeyInput<'_>) -> Option<(Saida, Option<String>)> {
        if !k.pressed {
            return Some((Saida::Nada, None));
        }
        let pagina = self.linhas.cast_signed();
        match k.key {
            Key::Up => self.anda(-1),
            Key::Down => self.anda(1),
            Key::PageUp => self.anda(-pagina),
            Key::PageDown => self.anda(pagina),
            Key::Escape if self.armado => self.armado = false,
            Key::Escape => return Some((Saida::Fecha, None)),
            Key::Enter => {
                let saida = self.destacado().map_or(Saida::Nada, Saida::Escolheu);
                return Some((saida, None));
            }
            _ => {
                let antes = self.busca.text.clone();
                let saiu = self.busca.key(k.key, k.mods, k.text, k.paste.as_deref());
                self.apos_escrever(&antes);
                match saiu {
                    Outcome::Ignored => return None,
                    Outcome::Copy(s) => return Some((Saida::Nada, Some(s))),
                    _ => {}
                }
            }
        }
        Some((Saida::Nada, None))
    }

    /// A tecla morta compõe-se e a letra vem na tecla: só o texto confirmado entra aqui.
    pub fn compose(&mut self, c: &Compose<'_>) -> bool {
        if matches!(c, Compose::DeadKey(_)) {
            return false;
        }
        if let Compose::Commit(ch) = c {
            let antes = self.busca.text.clone();
            self.busca.insert(ch.encode_utf8(&mut [0; 4]));
            self.apos_escrever(&antes);
        }
        true
    }

    #[must_use]
    pub fn aparencia(&self, x: f32, y: f32) -> PointerLook {
        match self.alvo_em(x, y) {
            Some(Alvo::Busca) => PointerLook::Text,
            Some(_) => PointerLook::Hand,
            None => PointerLook::Arrow,
        }
    }
}

#[cfg(test)]
mod tests;
