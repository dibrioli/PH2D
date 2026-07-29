//! **Um painel não drena o arrasto de `CurvePoint` de OUTRO painel.**
//!
//! ## O bug que este arquivo existe para impedir (medido 2026-07-29)
//!
//! O `curve_point_drag` do [`WidgetStore`] é um canal **GLOBAL**: o dispatch de 2D calcula
//! `(parent, canal, índice, x, y)` e o deixa lá para *o painel dono do gesto* drenar no mesmo
//! frame. E o `hero.apply_event` pergunta a **TODO painel do registry**, visível ou não, parando no
//! primeiro `Consumed`.
//!
//! O `route_value_changed` deste painel drenava para **QUALQUER** `ValueChanged` — ele tomava o
//! stash *e só então* procurava a camada dona — e devolvia `Consumed` mesmo sem achar nenhuma.
//! Consequência medida: os punhos do **trilho de rampa do painel de vetor** (o Gradient Map do FX
//! raster, plano 24 W11) **não se moviam**. O painel do FX pintava os punhos, o dispatch calculava
//! a posição nova, e o gesto era engolido por um painel de camadas que nem estava na tela — sem
//! erro, sem warning, e com os gates isolados dos dois painéis **verdes**.
//!
//! ⚠️ **O `take` é irreversível, e o próprio código já o dizia**: o `motion-params` carregava o
//! comentário *"Not our editor — put it back is impossible (take)"*. Um canal global que se drena
//! antes de perguntar de quem é o gesto não tem conserto a jusante — é por isso que a pergunta
//! passou a ser **parte da chamada** ([`WidgetStore::take_curve_point_drag_if`]).
//!
//! Este gate é a metade de COMPORTAMENTO. A metade estrutural é o compilador: não existe mais porta
//! que entregue o gesto sem responder *"é meu?"*.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::PainterLayersPanelState;
use ph2d_ui_testkit::MockPanelHost;

/// O pai de um gesto que pertence a outro painel. É um id que **nenhum** editor deste painel pode
/// produzir (os dele são hashes de `painter_layer.*` / `painter_curve.*`), e é o papel que o trilho
/// de rampa do painel de vetor tem na vida real.
const FOREIGN_EDITOR: NodeId = NodeId(0x5EED_0BAD_1234_5678);

/// **O arrasto de outro painel sobrevive à travessia deste.**
///
/// Duas asserções, e as duas importam por motivos diferentes: o stash tem de ficar **intacto**
/// (senão o dono não tem o que drenar) **e** o evento não pode ser consumido (senão o dono nunca é
/// perguntado — o `hero.apply_event` para no primeiro `Consumed`).
#[test]
fn a_curve_drag_that_belongs_to_another_panel_is_left_alone() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut panel_state = PainterLayersPanelState;

    // O dispatch acabou de calcular o arrasto de um punho de OUTRO painel.
    host.store_mut()
        .set_curve_point_drag(FOREIGN_EDITOR, 0, 3, 0.75, 0.5);

    let out = host.apply_panel_event::<PainterLayersPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(FOREIGN_EDITOR),
    );

    assert_ne!(
        out,
        EventOutcome::Consumed,
        "este painel CONSUMIU o `ValueChanged` de um editor que nao e dele — o `hero.apply_event` \
         para no primeiro `Consumed`, entao o painel DONO do gesto nunca sera perguntado"
    );
    let survived = host
        .store_mut()
        .take_curve_point_drag_if(|p| p == FOREIGN_EDITOR);
    assert_eq!(
        survived,
        Some((FOREIGN_EDITOR, 0, 3, 0.75, 0.5)),
        "o arrasto foi ROUBADO: o stash e um canal global e este painel o drenou antes de perguntar \
         de quem era o gesto. O `take` e irreversivel — o dono nao tem como recuperar o que este \
         painel tomou, e o sintoma no produto e um punho que nao se move (medido: o trilho de rampa \
         do Gradient Map do painel de vetor, 2026-07-29)"
    );
}

/// **CONTROLE: o arrasto que É deste painel continua a ser drenado.**
///
/// Sem esta metade, a cura do roubo poderia ser *"nunca drenar"* — e os editores de curva /
/// gradiente das camadas de ajuste do Painter parariam de funcionar, com o gate de cima verde.
///
/// ⚠️ **A fixture TEM de conter o fenômeno:** a pergunta de posse é *"algum id de editor da pilha
/// PUBLICADA casa com este pai?"*, então sem `set_current_layers` a resposta é honestamente NÃO e
/// este gate mediria o vazio (foi como ele nasceu — vermelho sobre produto correto).
#[test]
fn a_curve_drag_of_its_own_editor_is_still_drained() {
    use ph2d_editor_core::ids::painter_curve_editor_id;
    use ph2d_panel_painter_layers::state::set_current_layers;
    use ph2d_tool_painter::LayerStack;

    let mut stack = LayerStack::new();
    let layer = stack.add_raster("L", 8, 8).expect("a camada nasce");
    set_current_layers(Some(stack));

    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut panel_state = PainterLayersPanelState;
    // O editor de curva DAQUELA camada — o id é derivado do id runtime dela.
    let mine = painter_curve_editor_id(layer.0);

    host.store_mut().set_curve_point_drag(mine, 1, 2, 0.25, 0.5);
    let _ = host
        .apply_panel_event::<PainterLayersPanel>(&mut panel_state, WidgetEvent::ValueChanged(mine));

    assert!(
        host.store_mut()
            .take_curve_point_drag_if(|p| p == mine)
            .is_none(),
        "o painel deixou de reconhecer o arrasto do PROPRIO editor de curva — a cura do roubo nao \
         pode ser 'nunca drenar'"
    );
    set_current_layers(None);
}
