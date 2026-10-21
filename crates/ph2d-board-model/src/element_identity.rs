//! A identidade de um [`super::BoardDoc`] nesta sessão — de que documento é ([`Origin`], B15) e em que
//! revisão vai ([`Rev`]). Nenhuma das duas é conteúdo: o `PartialEq` do documento ignora-as.

/// ⭐ **De que documento veio** uma cópia (B15, doc 17 §2.4): os ids são um contador POR quadro, e
/// colar noutro quadro tem de saber que o `ElementId(3)` de lá não é o de cá. Tirada de um contador
/// global do processo; um documento lido de bytes ou clonado é OUTRO documento e nasce com outra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Origin(u64);

/// O lugar da [`Origin`] no documento: o `PartialEq` do documento compara CONTEÚDO (como a [`Rev`]).
#[derive(Debug)]
pub(crate) struct OriginSlot(pub(crate) Origin);

static NEXT_ORIGIN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl Default for OriginSlot {
    fn default() -> Self {
        Self(Origin(
            NEXT_ORIGIN.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        ))
    }
}

impl Clone for OriginSlot {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for OriginSlot {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// ⭐ **Revisão de um documento nesta sessão** — o «mudou alguma coisa?» em O(1) dos caches
/// derivados (as rotas das setas). Tirada de um contador GLOBAL do processo: dois documentos
/// distintos nunca partilham uma (um cache que passa de um quadro a outro não confunde os dois),
/// e um documento lido de bytes nasce com uma nova. Não é conteúdo: duas revisões são sempre
/// «iguais» para o `PartialEq` do documento.
#[derive(Clone, Copy, Debug)]
pub struct Rev(pub(crate) u64);

static NEXT_REV: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl Rev {
    pub(crate) fn fresh() -> Self {
        Self(NEXT_REV.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

impl Default for Rev {
    fn default() -> Self {
        Self::fresh()
    }
}

impl PartialEq for Rev {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
