//! **O vínculo de um MOTIVO a um caminho-guia** — [`VecPatternPath`] (Pattern Along Path, plano 23).
//!
//! Irmão do [`crate::VecTextPath`] (o texto cavalga a guia com glifos) e do [`crate::VecOffset`]
//! (a forma engorda sem que a curva autorada mude). Como o offset, a forma **fonte** continua
//! autorada — é ela que o modo Node edita — e as cópias são **desenho**, uma função pura da relação
//! que a shell re-cozinha por frame numa [`ph2d_vec_render::LiveGeometry`]. Consequência de graça:
//! **undo e save cobrem o pattern sem uma linha a mais** (os dois capturam o mundo ECS, e este
//! componente está no `ComponentRegistry`).
//!
//! # Por que um componente, e não um `PathEffect` da pilha (ADR-0132)
//!
//! Um efeito da pilha é `VecPath -> VecPath` avaliado **dentro** da `ph2d-vec-scene`, e o
//! `cooked()` é auto-contido: ele vê `self`, não a cena. O pattern precisa de um **segundo
//! objeto** — o caminho-guia — cuja geometria de mundo a `ph2d-vec-scene` não tem como resolver.
//! É a mesma razão que fez o `VecOffset` (que precisa do motor booleano) nascer componente, e o
//! `VecTextPath` (que precisa da fonte) também.
//!
//! # Por que um componente OPCIONAL e não campos noutra struct
//!
//! Um componente NOVO cunha a própria blob-key (`stable_type_id`) e **não move nada** — enquanto
//! apender campo a um blob existente é postcard posicional e **bumparia o `PROJECT_SCHEMA`, que
//! RECUSA todo projeto já salvo**. É o raciocínio do `VecTextPath`, do `PhysicsJoint` e do
//! `GravityScale`. E *"o motivo é reto quando não tem o componente"* é uma afirmação que o
//! compilador ajuda a manter, não uma que cada leitor tem de lembrar.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// O vínculo motivo → caminho-guia.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecPatternPath {
    /// O caminho-guia. É o `ph2d_vec_scene::VecPathId`, que é um `u64` — o ECS não depende do
    /// vetor (a mesma razão do `VecTextPath::path` e do `VecShape` guardarem primitivos).
    pub path: u64,
    /// Onde a tilagem começa, em **fração do comprimento total** do caminho — o idioma do
    /// `startOffset` do SVG, e o que sobrevive a editar o caminho (esticar a curva mantém o padrão
    /// onde o artista o pôs). Convertido em distância na fronteira (a mesma porta do texto).
    pub start_offset: f32,
    /// Onde a tilagem TERMINA, em **fração do comprimento total** (`1.0` = a curva inteira). Junto
    /// com `start_offset` define o TRECHO `[start, end]` em que as cópias caem — o resto da curva
    /// fica vazio (o *Start/End* do Trim Paths do After Effects). `end <= start` ⇒ trecho vazio ⇒
    /// nenhuma cópia, que é a resposta honesta.
    pub end_offset: f32,
    /// Multiplica a **largura do motivo** para dar o avanço por cópia: `1.0` encaixa borda-a-borda,
    /// `<1` sobrepõe, `>1` deixa vão. É o `PatternSpec::spacing` do motor.
    pub spacing: f32,
    /// Desvio da guia ao longo da **normal**, positivo para a esquerda do sentido de marcha (a
    /// convenção — e o sinal — do `dy` do texto). Em unidades de mundo.
    pub offset: f32,
    /// Padrão do outro lado da curva, a percorrê-la ao contrário.
    pub flip: bool,
}

impl SimComponent for VecPatternPath {}

impl Default for VecPatternPath {
    fn default() -> Self {
        Self {
            path: 0,
            start_offset: 0.0,
            end_offset: 1.0,
            spacing: 1.0,
            offset: 0.0,
            flip: false,
        }
    }
}
