//! **Efeitos de camada** (P13 §4): não destrutivos, aplicados por camada ANTES do blend com a
//! MESMA função do filtro destrutivo (uma lei, dois chamadores). Aqui só o dado.

/// Os parâmetros do contorno são os do filtro Edit ▸ FX ▸ Outline.
pub type OutlineParams = crate::filter::outline::Outline;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EfeitoDeCamada {
    Outline(OutlineParams),
}
