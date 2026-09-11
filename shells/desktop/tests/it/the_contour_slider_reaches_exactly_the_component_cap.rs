//! **O teto de anéis do PAINEL é o teto do COMPONENTE** — a pin da cópia que a fronteira impõe.
//!
//! # Por que a cópia existe
//!
//! A autoridade do número é `ph2d_ecs::MAX_CONTOUR_STEPS`: é lá que o cozimento clampa, e é lá que
//! um projeto salvo com uma contagem absurda é contido. Mas nem `ph2d-panel-vector` nem
//! `ph2d-tool-vector` dependem do ECS — nem devem: um painel que conhece componentes é um painel
//! que conhece a cena. Então o slider carrega a própria faixa.
//!
//! Duas cópias de um número **precisam de um dono que veja as duas**, e a shell é o único sítio
//! onde os dois lados existem. Sem esta pin, a divergência é silenciosa nos dois sentidos:
//!
//! - painel MAIOR que o componente → o artista arrasta até 24 e a cena entrega 16, sem aviso;
//! - painel MENOR → parte da faixa que o documento aceita fica **inalcançável**, e nada na tela
//!   diz que ela existe.
//!
//! # Por que a pin é sobre o TRILHO, e não sobre a const
//!
//! A faixa do painel é `pub(crate)` de propósito (a shell não tem por que ler a régua interna do
//! slider — ela fala em VALORES). O que a shell PODE perguntar é o mapa público: *"onde cai o
//! teto do componente no trilho?"*. Se as duas faixas concordam, o teto cai exatamente no fim do
//! curso — e o penúltimo passo, não. As duas asserções juntas fixam a igualdade sem publicar
//! const nenhuma.

use ph2d_ecs::MAX_CONTOUR_STEPS;
use ph2d_panel_vector::contour_steps_to_track;

#[test]
fn the_contour_slider_reaches_exactly_the_component_cap() {
    let cap = f64::from(MAX_CONTOUR_STEPS);
    let at_cap = contour_steps_to_track(cap);
    assert!(
        (at_cap - 1.0).abs() < 1e-6,
        "o teto do componente ({MAX_CONTOUR_STEPS} anéis) cai em {at_cap} do trilho, não no fim: \
         a faixa do painel é MENOR que a do componente e parte do que o documento aceita ficou \
         inalcançável pelo slider"
    );
    let below = contour_steps_to_track(cap - 1.0);
    assert!(
        below < 1.0 - 1e-6,
        "o penúltimo passo ({}) já satura o trilho ({below}): a faixa do painel é MAIOR que a do \
         componente, e o topo do slider promete anéis que o cozimento vai clampar em silêncio",
        cap - 1.0
    );
}
