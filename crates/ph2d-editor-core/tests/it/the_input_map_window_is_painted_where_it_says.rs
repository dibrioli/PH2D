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

/// O mesmo passe, devolvendo o índice de hit INTEIRO — é ele que responde *«o que está sob este
/// ponto?»*, e a resolução dele (o ÚLTIMO rectângulo que cobre o ponto) é a lei que a ordem de
/// registo do pintor explora. Uma lista de registos não a reproduz sem a reimplementar.
fn painted_hit_index(store: &WidgetStore, map: &ph2d_input::InputMap) -> HitIndex {
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
    hit
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

/// ⛔⛔ **O FUNDO DO CARTÃO ABSORVE O CLIQUE — a lei que o pintor PROMETIA num comentário e que
/// gate nenhum media.**
///
/// A nota em `chrome/input_map.rs`, ao lado do `hit_index.register(ids::INPUT_MAP_SURFACE, rect)`,
/// diz que sem ele *"clicar no espaço vazio ENTRE dois controlos da janela caía no canvas por
/// baixo: com o pincel na mão, o artista **pintava** enquanto arrumava os controlos"*. Era uma
/// afirmação sem instrumento: apagar aquela linha não movia um único gate deste ficheiro.
///
/// # A régua é a AUSÊNCIA, e é por isso que ela é uma varredura
///
/// ⚠️ O término deste id **não é** um braço de `match` — é o `None` do `HitIndex`. Todo caminho de
/// canvas do shell pergunta a mesma coisa (`over_canvas_or_gizmo` em `vec_text_reopen.rs`:
/// `hit_index.hit(x, y)` → `None` = canvas cru; e o `on_canvas` de `input_dispatch.rs`), então o
/// que decide se a janela é uma JANELA ou um DESENHO é se existe algum ponto dentro dela que
/// resolve para `None`.
///
/// ⇒ um único ponto de sonda não serve: o buraco vive **entre** controlos, e onde ele fica muda
/// com o número de acções. A varredura é uma grelha densa sobre o cartão inteiro.
///
/// # As três metades
///
/// 1. **nenhum** ponto dentro do cartão cai no canvas;
/// 2. **algum** ponto é respondido pelo próprio fundo — o controlo POSITIVO. Sem ele, uma janela
///    cujos controlos por acaso cobrissem tudo passaria sem que o fundo existisse, e a primeira
///    acção nova reabriria o defeito;
/// 3. e um ponto **fora** do cartão continua a ser canvas — o controlo NEGATIVO. Sem ele, um
///    `HitIndex` que respondesse a tudo passaria a metade (1), e o gate estaria a medir a forma
///    dos dados em vez do produto.
///
/// ⚠️ **O cartão é medido pela porta do próprio pintor** (`input_map_window_size`), nunca pelo
/// rectângulo que o registo publica: derivá-lo do registo faria a metade (1) medir *"o fundo cobre
/// o fundo"*, que é verdade mesmo quando ele não cobre a janela.
///
/// **Mutação que deve sangrar:** apagar o `hit_index.register(ids::INPUT_MAP_SURFACE, rect)`.
#[test]
fn no_point_inside_the_window_falls_through_to_the_canvas() {
    let (store, map) = opened();
    let hit = painted_hit_index(&store, &map);
    // A MESMA conta que o pintor faz — e que a shell usa para a roda e o arrasto.
    let (w, h, _) = chrome::input_map_window_size(&map, VIEWPORT.h);
    let (x0, y0) = (VIEWPORT.x + 48.0, VIEWPORT.y + 48.0);

    // Grelha densa, recuada 1 px da borda: o limite exacto é do desenho, não do gesto.
    const STEPS: usize = 40;
    let mut through: Vec<(f32, f32)> = Vec::new();
    let mut absorbed_by_the_card = 0usize;
    for iy in 0..STEPS {
        for ix in 0..STEPS {
            #[allow(clippy::cast_precision_loss)]
            let fx = (ix as f32 + 0.5) / STEPS as f32;
            #[allow(clippy::cast_precision_loss)]
            let fy = (iy as f32 + 0.5) / STEPS as f32;
            let (px, py) = (x0 + 1.0 + fx * (w - 2.0), y0 + 1.0 + fy * (h - 2.0));
            match hit.hit(px, py) {
                None => through.push((px, py)),
                Some(id) if id == ids::INPUT_MAP_SURFACE => absorbed_by_the_card += 1,
                Some(_) => {}
            }
        }
    }

    assert!(
        through.is_empty(),
        "{} de {} pontos DENTRO da janela do Input Map caem no canvas por baixo (o primeiro em \
         {:?}). Com o pincel na mao, arrumar os controlos PINTA — um cartao flutuante que deixa \
         passar o que nao consome nao e' uma janela, e' um desenho",
        through.len(),
        STEPS * STEPS,
        through[0]
    );
    assert!(
        absorbed_by_the_card > 0,
        "nenhum ponto foi respondido pelo FUNDO: ou ele nao esta' registado, ou os controlos \
         cobrem o cartao inteiro por acaso — e nesse caso a metade acima e' verde sem o fundo \
         existir, e a proxima accao criada reabre o defeito"
    );
    assert_eq!(
        hit.hit(x0 - 4.0, y0 + h * 0.5),
        None,
        "um ponto FORA do cartao deixou de ser canvas — a sonda esta' a medir um indice que \
         responde a tudo, e a metade de cima nao prova nada"
    );
}
