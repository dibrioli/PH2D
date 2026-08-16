//! **Os intervalos que este painel JÁ ENFORÇA — com um dono só.**
//!
//! ## O que estava errado, e é a mesma doença da wave 4 uma camada mais fora
//!
//! O `apply_value_changed` (o commit) **declara o intervalo** de vários campos deste painel —
//! `.clamp(0, 255)` numa componente de cor, `.clamp(1, 64)` nas subdivisões, `.min(8)` nas
//! iterações de Lloyd — e a **lei do scrub não sabia de nenhum**. O `WidgetStore` não tinha
//! `number_range` para eles, logo o arrasto caía no atalho histórico `DRAG_RATE_X · step`, que
//! não sabe nada sobre esse intervalo.
//!
//! É *um facto com duas metades que não se falam*, e o preço é o mesmo que a sonda
//! `scrub_range_census` mediu em 2026-08-12 para a família do slider ligado — a fração de pixel
//! que atravessa o campo inteiro:
//!
//! | campo | o que o commit enforça | cruzar a 50 unidades/px |
//! |---|---|---|
//! | iterações de Lloyd | `.min(8)` | **0,16 px** |
//! | subdivisões do snap | `.clamp(1, 64)` | **1,26 px** |
//! | componente de cor | `.clamp(0, 255)` | **5,1 px** |
//!
//! Um pixel de arrasto saturava as iterações **seis vezes**. Registada a faixa, a lei
//! proporcional do `number_scrub_law` atravessa cada um em `DRAG_RANGE_PX_H` = 250 px, que é o
//! alvo de desenho que todo campo com faixa deste app já cumpre.
//!
//! ## Por que os números moram AQUI e não em cada sítio
//!
//! ⚠️ **Nenhum número deste arquivo é escolhido — todos já eram enforçados por uma linha que
//! shipa.** O que a extração muda é *quantas cópias existem*: registar a faixa no `populate` e
//! deixar o clamp no `event` seriam **duas** cópias do mesmo intervalo, e a próxima pessoa a
//! afinar uma delas deixaria a caixa a arrastar até um valor que o commit recusa — o *"um facto,
//! duas cópias que discordam"* que este repositório paga todas as semanas. Com um dono, uma
//! afinação move as duas metades ou nenhuma.
//!
//! ⚠️ **Só entram aqui os campos cujo intervalo é COMPLETO e independente de unidade.** Os que
//! têm só piso (`.max(1)` numa contagem, `.max(MIN_CELL_SIZE_M)` num tamanho) precisam da
//! receita documentada em `set_number_drag_rate` — faixa para o stepper **mais** taxa calibrada
//! para o arrasto —, e a taxa é escolha que ninguém mediu ainda; e o raio de magnetismo é
//! clampado em METROS enquanto a caixa mostra a **unidade do projecto**, então a faixa dele só
//! é conhecida onde a unidade é (o `paint`), nunca no `populate`, que corre uma vez no boot.

/// Iterações de relaxação de Lloyd do Voronoi — o commit faz `.min(8)`, e o piso é o `as u32`.
pub(crate) const LLOYD_ITERATIONS: (f64, f64, f64) = (0.0, 8.0, 1.0);

/// Componente sRGB da cor da grade — o commit faz `.clamp(0.0, 255.0)`.
pub(crate) const COLOR_COMPONENT: (f64, f64, f64) = (0.0, 255.0, 1.0);

/// Sub-grade do snap — o commit faz `.clamp(1, 64)`.
pub(crate) const SNAP_SUBDIVISIONS: (f64, f64, f64) = (1.0, 64.0, 1.0);

/// Os campos cuja faixa este módulo possui, na forma que o `populate` regista.
///
/// ⚠️ É uma tabela e não três chamadas soltas porque o `populate` e o gate a percorrem: um campo
/// novo entra aqui e fica coberto pelos dois, que é o que uma lista escrita à mão em dois
/// lugares não dá.
pub(crate) const DECLARED: [(ph2d_editor_core::NodeId, (f64, f64, f64)); 5] = [
    (crate::ids::GS_CFG_VORONOI_LLOYD_ITERS, LLOYD_ITERATIONS),
    (crate::ids::GS_CFG_COLOR_R, COLOR_COMPONENT),
    (crate::ids::GS_CFG_COLOR_G, COLOR_COMPONENT),
    (crate::ids::GS_CFG_COLOR_B, COLOR_COMPONENT),
    (crate::ids::GS_CFG_SNAP_SUBDIVISIONS, SNAP_SUBDIVISIONS),
];
