//! **O canal da DOAÇÃO** — como um plano de forma chega à tinta do Painter.
//!
//! O Painter aceita uma **segunda fonte de normal** (`docs/3D/05.2`): um plano canvas-shaped de
//! `[nx, ny, nz, cobertura]` que o passe de luz compõe com o gradiente da tinta. Este módulo é o fio
//! entre quem PRODUZ esse plano e quem o INSTALA, e ele existe por duas restrições que se cruzam.
//!
//! ## Por que não é uma chamada direta
//!
//! Só o `painter_bridge::dispatch` pode fazer downcast para `PainterTool` (arch-gate
//! `architecture_no_downcast_to_concrete_tool_in_shell`), e é ele que instala. Mas quem rasteriza a
//! forma é o módulo 3D, que o bridge **não pode** conhecer — a promessa de removibilidade
//! (`docs/3D/02.3`) proíbe qualquer símbolo do módulo fora de um `#[cfg(feature = "sculpt3d")]`.
//!
//! ⚠️ **O que atravessa é `Vec<f32>` e um par de `u32`** — nenhum tipo do módulo 3D. É por isso que
//! este arquivo não tem `cfg` nenhum: o Painter aceita *uma forma*, não *uma malha*, e apagar o
//! módulo 3D deixa este canal existindo e silencioso, que é exatamente o estado de hoje sem a
//! feature. O precedente é o `painter_commit_requested`: um fato transiente que o bridge consome.

use std::sync::Arc;

/// O canal, nos dois sentidos — a notícia a entregar e o tamanho a rasterizar.
#[derive(Default)]
pub(crate) struct DonatedForm {
    /// **A notícia**, e os três estados são distintos:
    ///
    /// - `Some(Some(plano))` — instale este plano;
    /// - `Some(None)` — **APAGUE** a doação (o interruptor desligou, ou a forma sumiu);
    /// - `None` — silêncio, nada mudou.
    ///
    /// ⚠️ Colapsar os dois primeiros num `Option` só perderia o desligamento: *um interruptor que só
    /// sabe ligar não é um interruptor*. E o silêncio é o caso comum — a forma só muda quando a
    /// malha, a câmera ou o canvas mudam, então o produtor fica quieto quase todo frame.
    pub(crate) news: Option<Option<Arc<Vec<f32>>>>,

    /// O tamanho do canvas que o Painter reportou no **último** frame.
    ///
    /// ⚠️ Um **READBACK, nunca uma autoridade.** Quem sabe quão grande é o canvas é o tool, e o
    /// produtor não pode perguntar a ele (o downcast é do bridge). Publicar aqui custa um frame de
    /// atraso num redimensionamento — e o atraso é **inofensivo por construção**, porque
    /// `PainterTool::form_at` recusa um plano do tamanho errado: o sintoma é *um frame sem doação*,
    /// nunca uma forma lida torta.
    pub(crate) canvas: Option<(u32, u32)>,
}
