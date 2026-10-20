//! **Os efeitos VIVOS da entidade nos canais** (plano 35 §A.1, *Efeitos*) — o leitor/escritor puro
//! do [`VecContour`] e do [`VecOffset`]. Eles são componentes da ENTIDADE e não do caminho, por
//! isso moram fora do [`crate::canais_campos`]; o [`crate::canais`] encaminha por [`e_de_efeito`].
//! Criar/remover o efeito é estrutura (plano 35 §8), não canal: sem o componente, não há campo.

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as c;
use ph2d_ecs::{FieldKey, MAX_CONTOUR_STEPS, VecContour, VecOffset};

const FIM_CONTORNO: u16 = c::CONTORNO_COR + 4;
const FIM_OFFSET: u16 = c::OFFSET_LADO + 1;

/// O campo `kk` é de um efeito vivo?
#[must_use]
pub fn e_de_efeito(kk: FieldKey) -> bool {
    kk.familia == VETOR
        && ((c::CONTORNO_PASSOS..FIM_CONTORNO).contains(&kk.campo)
            || (c::OFFSET_DISTANCIA..FIM_OFFSET).contains(&kk.campo))
}

fn do_contorno(kk: FieldKey) -> bool {
    (c::CONTORNO_PASSOS..FIM_CONTORNO).contains(&kk.campo)
}

/// Os efeitos vivos de uma entidade, como os canais os vêem.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Efeitos {
    pub contorno: Option<VecContour>,
    pub offset: Option<VecOffset>,
}

impl Efeitos {
    /// Os campos que estes efeitos têm (todos os de cada efeito presente).
    #[must_use]
    pub fn campos(&self) -> Vec<FieldKey> {
        let k = |campo| FieldKey::new(VETOR, campo, 0, 0);
        let mut out = Vec::new();
        if self.contorno.is_some() {
            out.extend((c::CONTORNO_PASSOS..FIM_CONTORNO).map(k));
        }
        if self.offset.is_some() {
            out.extend((c::OFFSET_DISTANCIA..FIM_OFFSET).map(k));
        }
        out
    }

    /// Quanto vale `kk` — `None` se o efeito dele não está na entidade.
    #[must_use]
    pub fn ler(&self, kk: FieldKey) -> Option<f32> {
        if !e_de_efeito(kk) || kk.i != 0 || kk.j != 0 {
            return None;
        }
        if do_contorno(kk) {
            let ct = self.contorno.as_ref()?;
            return Some(match kk.campo {
                c::CONTORNO_PASSOS => f32::from(ct.steps),
                c::CONTORNO_DISTANCIA => f(ct.d),
                c::CONTORNO_ACEL => ct.accel,
                c::CONTORNO_JUNCAO => f32::from(ct.join),
                c::CONTORNO_LADO => f32::from(ct.side),
                cor => f32::from(ct.to[usize::from(cor - c::CONTORNO_COR)]),
            });
        }
        let o = self.offset.as_ref()?;
        Some(match kk.campo {
            c::OFFSET_DISTANCIA => f(o.d),
            c::OFFSET_JUNCAO => f32::from(o.join),
            _ => f32::from(o.side),
        })
    }

    /// Aplica `v` a `kk`, com as leis do painel: passos em `1..=MAX_CONTOUR_STEPS`, códigos de
    /// quina e lado em `0..=2`, cor em `0..=255`. Um efeito ausente fica ausente.
    pub fn escrever(&mut self, kk: FieldKey, v: f32) {
        if !e_de_efeito(kk) || kk.i != 0 || kk.j != 0 {
            return;
        }
        if do_contorno(kk) {
            let Some(ct) = self.contorno.as_mut() else {
                return;
            };
            match kk.campo {
                c::CONTORNO_PASSOS => {
                    ct.steps = inteiro(v, f32::from(MAX_CONTOUR_STEPS)).max(1);
                }
                c::CONTORNO_DISTANCIA => ct.d = f64::from(v),
                c::CONTORNO_ACEL => ct.accel = v,
                c::CONTORNO_JUNCAO => ct.join = byte(v, 2.0),
                c::CONTORNO_LADO => ct.side = byte(v, 2.0),
                cor => ct.to[usize::from(cor - c::CONTORNO_COR)] = byte(v, 255.0),
            }
            return;
        }
        let Some(o) = self.offset.as_mut() else {
            return;
        };
        match kk.campo {
            c::OFFSET_DISTANCIA => o.d = f64::from(v),
            c::OFFSET_JUNCAO => o.join = byte(v, 2.0),
            _ => o.side = byte(v, 2.0),
        }
    }
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
    reason = "arredondado e preso ao domínio antes da conversão"
)]
fn inteiro(v: f32, max: f32) -> u16 {
    v.round().clamp(0.0, max) as u16
}

fn byte(v: f32, max: f32) -> u8 {
    u8::try_from(inteiro(v, max.min(255.0))).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(campo: u16) -> FieldKey {
        FieldKey::new(VETOR, campo, 0, 0)
    }

    /// Cada campo volta a ler o que se escreveu (dentro do domínio), e fora dele prende-se.
    #[test]
    fn o_efeito_le_o_que_escreveu() {
        let mut fx = Efeitos {
            contorno: Some(VecContour::default()),
            offset: Some(VecOffset::new(0.2, 1, 0)),
        };
        for (campo, v) in [
            (c::CONTORNO_PASSOS, 7.0),
            (c::CONTORNO_DISTANCIA, 0.25),
            (c::CONTORNO_ACEL, 1.5),
            (c::CONTORNO_JUNCAO, 2.0),
            (c::CONTORNO_LADO, 1.0),
            (c::CONTORNO_COR + 1, 40.0),
            (c::OFFSET_DISTANCIA, -0.5),
            (c::OFFSET_JUNCAO, 0.0),
            (c::OFFSET_LADO, 2.0),
        ] {
            fx.escrever(k(campo), v);
            assert_eq!(fx.ler(k(campo)), Some(v), "campo {campo}");
        }
        fx.escrever(k(c::CONTORNO_PASSOS), 0.0);
        assert_eq!(fx.contorno.unwrap().steps, 1, "um Contour vivo tem pelo menos um anel");
        fx.escrever(k(c::CONTORNO_PASSOS), 99.0);
        assert_eq!(fx.contorno.unwrap().steps, MAX_CONTOUR_STEPS);
        assert_eq!(fx.campos().len(), 12);
        assert!(fx.campos().into_iter().all(|kk| fx.ler(kk).is_some()));
        assert_eq!(fx.ler(FieldKey::new(VETOR, c::CONTORNO_PASSOS, 1, 0)), None);
        assert!(!e_de_efeito(k(c::TEXTO_EIXO)));
    }

    /// Sem o componente, o campo não existe e escrever não o inventa (a lei do `retune`).
    #[test]
    fn um_efeito_ausente_fica_ausente() {
        let mut fx = Efeitos::default();
        fx.escrever(k(c::OFFSET_DISTANCIA), 3.0);
        assert_eq!(fx, Efeitos::default());
        assert!(fx.campos().is_empty());
        assert_eq!(fx.ler(k(c::CONTORNO_PASSOS)), None);
    }
}
