//! **A JANELA É MEDIDA PELO QUE ELA PINTA** — os gates que faltavam, e a razão de faltarem.
//!
//! ⛔⛔ **A auditoria multiagêntica de 2026-08-24 devolveu 25 achados confirmados, e nenhum dos
//! meus doze gates olhava para o que foi DESENHADO** — todos mediam o mapa e o `WidgetStore`. É
//! por isso que doze verdes conviviam com uma janela a desenhar por cima do próprio título, e por
//! isso que os TRÊS reports com foto do Enio (*"estreito e sem scroll"*, *"labels emboladas"*,
//! *"a caixa de texto parece morta"*) tiveram de vir dele.
//!
//! ⇒ estes gates chamam o **pintor real**, sem janela nem GPU, e perguntam-lhe:
//!
//! * **onde** cada coisa ficou (o `HitIndex`, que é a única saída endereçável do pintor), e
//! * **se a tinta muda** quando o estado que ela deve mostrar muda (a codificação da cena).
//!
//! ⚠️ *Um gate que só lê o modelo nunca vê um pintor a mentir sobre ele.*

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::screens::hero::{chrome, ids};
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// Um mapa com os seis verbos de fábrica **mais** uma acção recém-criada e ainda **sem tecla** —
/// que é exactamente o estado das duas fotos do report.
fn map_with_a_fresh_action() -> ph2d_input::InputMap {
    let mut m = ph2d_input::InputMap::with_player_defaults();
    m.create("casa");
    // ⚠️ **A SEGUNDA existe para a fixtura CONTER o fenómeno.** Com uma só, ela era a
    // última da lista e uma linha injectada por baixo dela não empurrava nada — a mutação do
    // `body_lines` **sobreviveu** ao gate na primeira tentativa. *Uma fixtura só prova o que
    // contém.*
    m.create("quintal");
    m
}

/// Abre a janela, regista as linhas, e devolve o par `(store, mapa)` pronto a pintar.
fn opened() -> (WidgetStore, ph2d_input::InputMap) {
    let map = map_with_a_fresh_action();
    let mut store = WidgetStore::with_capacity(64);
    store.open_input_map(VIEWPORT.x + 48.0, VIEWPORT.y + 48.0);
    chrome::sync_input_map_rows(&mut store, &map);
    (store, map)
}

/// Pinta a janela inteira e devolve `(onde cada widget ficou, a tinta)`.
fn paint(store: &WidgetStore, map: &ph2d_input::InputMap) -> (Vec<(NodeId, Rect)>, Vec<u32>) {
    let mut hit = HitIndex::default();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    chrome::paint_input_map_window(
        &mut scene,
        &mut text,
        Theme::Forge,
        &mut hit,
        store,
        map,
        VIEWPORT,
    );
    let rects: Vec<_> = hit.iter_registrations().collect();
    let ink = scene.inner().encoding().draw_data.clone();
    (rects, ink)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Rect {
    rects
        .iter()
        .rev()
        .find(|(i, _)| *i == id)
        .unwrap_or_else(|| panic!("o pintor nao registou {id:?} -- widget morto sob o ponteiro"))
        .1
}

fn row_of(map: &ph2d_input::InputMap, name: &str) -> usize {
    map.actions()
        .iter()
        .position(|a| a.name == name)
        .expect("a accao existe")
}

/// ⭐⭐ **O CAMPO DO NOME FICA EM CIMA — acima de TODA linha de acção.**
///
/// Enio, 2026-08-24: *"a caixa de Action name fica em cima e não embaixo do painel"*. E a nota que
/// o punha em baixo justificava-se com a referência — *"como no Godot"* —, que é **falso**: o
/// *Input Map* do Godot põe o campo *Add New Action* no topo do painel.
///
/// **Mutação que deve sangrar:** devolver o campo ao rodapé (`rect.y + rect.h - pad_y - row_h`).
#[test]
fn the_name_field_sits_above_every_action_row() {
    let (store, map) = opened();
    let (rects, _) = paint(&store, &map);
    let field = rect_of(&rects, ids::INPUT_MAP_NEW_NAME);
    let add = rect_of(&rects, ids::INPUT_MAP_ADD);
    assert!(
        (field.y - add.y).abs() < 0.5,
        "o campo e o Add tem de partilhar a linha: {field:?} vs {add:?}"
    );
    for row in 0..map.len() {
        let listen = rect_of(&rects, ids::input_map_listen_id(row));
        assert!(
            field.y + field.h <= listen.y,
            "o campo do nome (y={}) esta' ABAIXO da linha {row} (y={}) -- ele tem de ficar em \
             cima, como no Godot",
            field.y,
            listen.y
        );
    }
}

/// ⭐⭐ **O FOCO VÊ-SE.**
///
/// Enio, 2026-08-24: *"A caixa de texto parece morta, não se vê que o foco está nela ao clicar."*
/// Estava certo, e a costura estava toda feita menos o último elo: o `pointer_down` **já** escrevia
/// `TextInputState::Focused` e o pintor desenhava um rectângulo à mão que **não o lia**.
///
/// ⚠️ **O gate mede TINTA, não estado** — é a única pergunta que apanha um pintor a ignorar o
/// estado que lhe entregam. Um gate que lesse `store.get(..)` ficaria verde com o campo morto.
///
/// **Mutação que deve sangrar:** voltar ao `stroke_rounded_rect` + `paint_text` à mão.
#[test]
fn the_name_field_shows_that_it_has_the_focus() {
    let (mut store, map) = opened();
    let (_, calm) = paint(&store, &map);
    if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(ids::INPUT_MAP_NEW_NAME)
    {
        *state = TextInputState::Focused;
    }
    let (_, focused) = paint(&store, &map);
    assert_ne!(
        calm, focused,
        "a caixa de texto pinta o mesmo com e sem foco: clicar nela nao mostra nada, e o artista \
         conclui que ela esta' morta"
    );
    if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(ids::INPUT_MAP_NEW_NAME)
    {
        *state = TextInputState::Hovered;
    }
    let (_, hovered) = paint(&store, &map);
    assert_ne!(
        calm, hovered,
        "a caixa de texto tambem nasce inerte sob o rato"
    );
}

/// ⭐⭐⭐ **ARMAR A ESCUTA NÃO MOVE UMA LINHA.**
///
/// ⛔ Este é o gate que faltava para o *"labels emboladas"*. O aviso da escuta era pintado **depois**
/// de o cursor vertical já ter avançado, então caía em cima da linha da face vazia — e nenhum gate
/// o via, porque nenhum gate perguntava **onde** as coisas ficavam.
///
/// A lei tem duas metades e as duas importam: a escuta **muda a tinta** (senão não há indicador
/// nenhum) e **não muda a geometria** (senão ela empurra as linhas de baixo, e o indicador passa a
/// competir por um `y` com quem já lá estava).
///
/// **Mutação que deve sangrar:** fazer a face vazia armada ocupar uma linha PRÓPRIA em
/// [`body_lines`] — o `assert_eq` dos rectângulos parte na hora.
#[test]
fn arming_an_action_paints_a_sign_without_moving_a_single_row() {
    let (mut store, map) = opened();
    let (calm_rects, calm_ink) = paint(&store, &map);
    let casa = map.actions()[row_of(&map, "casa")].id;
    store.listen_for_binding(casa);
    let (armed_rects, armed_ink) = paint(&store, &map);
    assert_ne!(
        calm_ink, armed_ink,
        "armar a escuta nao muda uma tinta: nada na tela diz que o app esta' a espera de uma tecla"
    );
    assert_eq!(
        calm_rects, armed_rects,
        "armar a escuta MOVEU os controlos: o aviso ganhou uma linha propria e empurrou o resto"
    );
}

// ⚠️ **A quarta lei desta janela — *a faixa do título nomeia a acção armada* — NÃO tem gate
// aqui, e a razão fica escrita:** a sonda possível daqui é a **tinta da janela inteira**, e armar
// muda a tinta por **duas** razões (a faixa, e o `+` da linha a trocar de estilo). Uma mutação que
// fizesse a faixa dizer sempre `Input Map` **sobreviveu** a esse gate — medido em 2026-08-24.
// *Uma sonda que soma dois sinais não diz qual dos dois falhou.*
//
// ⇒ a lei mora numa função com gate próprio,
// `layout::tests::the_title_strip_names_the_action_it_is_listening_to`, e o pintor tem **um**
// sítio a chamá-la.
