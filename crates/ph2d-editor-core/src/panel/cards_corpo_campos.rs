//! As linhas de corpo de cartão que um pintor de secção pede pela mão: campos num rect já arrumado,
//! barra com chip e amostras com código. Filho de `cards_corpo` pelo tecto de LOC.

use super::{Cell, Widget, ctx, glifo, letra_da_grandeza};
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::card_field::{CardFieldSpec, Glyph};
use crate::widget::card_ink::CardInk;
use crate::widget::key_mark::Eixos;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// O corpo de letra dos rótulos de um cartão, para o rótulo que um corpo pinta à mão.
pub const LABEL_PX: f32 = m::LABEL_PX;
/// O vão entre duas linhas de um corpo, para a grelha que um corpo pinta à mão (lei 3).
pub const ROW_GAP: f32 = m::FIELD_ROW_GAP;

/// Campos Figma dentro de um rect que o chamador já arrumou, sem rótulo nem marca (a linha dele
/// traz os dois). Com dois ids, as letras X e Y.
#[allow(clippy::too_many_arguments)]
pub fn fields_in(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    r: Rect,
    rotulo: &str,
    ids: &[NodeId],
    step: f64,
) {
    let spec = CardFieldSpec {
        step,
        scrub: step,
        default: None,
    };
    let n = ids.len();
    let cells: Vec<Cell<'_>> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            store.mark_card_field_at_paint(id, spec);
            Cell::new(Widget::Field {
                id,
                glyph: glifo(rotulo, Eixos::Xyzw, n, i),
            })
        })
        .collect();
    let mut c = ctx(scene, text_system, store, hit_index, ink, r.x, r.w);
    // A caixa é a do rect dado: sem o vão de linha, que a linha do chamador já pagou.
    c.first = true;
    let _ = c.row(&cells, r.y);
}

/// A caixa de marcar num rect já arrumado: o nome à direita, sem marca de animação e fora do fluxo (a
/// linha do chamador é que divide a fileira). `parada` = não responde: pinta apagada e não regista alvo.
#[allow(clippy::too_many_arguments)]
pub fn check_in(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    r: Rect,
    id: NodeId,
    nome: &str,
    on: bool,
    parada: bool,
) {
    let valor = if on {
        crate::widget::CheckboxValue::Checked
    } else {
        crate::widget::CheckboxValue::Unchecked
    };
    let mut alvos = HitIndex::default();
    super::check_em(
        scene,
        text_system,
        store,
        if parada { &mut alvos } else { hit_index },
        ink,
        r,
        (id, crate::widget::key_mark::Rotulo::vista_de(nome)),
        (valor, parada),
        false,
    );
}

/// Um campo de texto num rect já arrumado, sem rótulo (a linha do chamador traz o dela).
#[allow(clippy::too_many_arguments)]
pub fn text_in(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    r: Rect,
    id: NodeId,
    placeholder: &str,
) {
    let mut c = ctx(scene, text_system, store, hit_index, ink, r.x, r.w);
    c.first = true;
    let _ = c.row(&[Cell::new(Widget::Text { id, placeholder })], r.y);
}

/// Uma amostra com o código num rect que o chamador já arrumou, sem rótulo nem marca.
#[allow(clippy::too_many_arguments)]
pub fn swatch_in(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    r: Rect,
    id: NodeId,
    rgba: [u8; 4],
) {
    let text = format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
    let cell = Cell::new(Widget::Swatch {
        id,
        rgba: Some(rgba),
        text: &text,
        edit: None,
        detach: None,
    });
    let mut c = ctx(scene, text_system, store, hit_index, ink, r.x, r.w);
    c.first = true;
    let _ = c.row(&[cell], r.y);
}

/// Uma barra com chip, no cartão: o campo é o chip, a barra não se pinta e a marca é a dela, que é o
/// canal (receita 11, «Slider + chip ligado»). Devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub fn bar_field(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: &str,
    chip: NodeId,
    bar: NodeId,
    spec: CardFieldSpec,
) -> f32 {
    let letra = letra_da_grandeza(rotulo);
    if let Some(fim) = super::compacta::pede_numero(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        (x, w, y),
        rotulo.to_owned(),
        letra,
        (chip, bar),
        spec,
    ) {
        return fim;
    }
    store.mark_card_field_at_paint(chip, spec);
    let key = [bar];
    let cell = Cell::new(Widget::Field {
        id: chip,
        glyph: Glyph::Text(letra),
    })
    .label(rotulo)
    .key(&key);
    ctx(scene, text_system, store, hit_index, ink, x, w).row(&[cell], y)
}

/// ⭐ **Amostras com o código, como no Fill**, em `colunas` por linha: o rótulo em cima e uma marca
/// para o grupo. Sem cor conhecida, a seleção diverge e o código diz «Mixed». A caixa inteira abre o
/// seletor. Devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub fn swatch_grid(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: &str,
    amostras: &[(NodeId, Option<[u8; 4]>)],
    colunas: usize,
) -> f32 {
    let ids: Vec<NodeId> = amostras.iter().map(|(id, _)| *id).collect();
    let codigos: Vec<String> = amostras
        .iter()
        .map(|(_, c)| {
            c.map_or_else(
                || ph2d_i18n::tr("panel.vector.mixed").to_owned(),
                |c| format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2]),
            )
        })
        .collect();
    if let ([(id, rgba)], [codigo]) = (amostras, codigos.as_slice())
        && let Some(fim) = super::compacta::pede_amostra(
            scene,
            text_system,
            store,
            hit_index,
            ink,
            (x, w, y),
            rotulo,
            *id,
            *rgba,
            codigo,
        )
    {
        return fim;
    }
    let mut c = ctx(scene, text_system, store, hit_index, ink, x, w);
    let mut y = y;
    for (l, (fila, textos)) in amostras
        .chunks(colunas.max(1))
        .zip(codigos.chunks(colunas.max(1)))
        .enumerate()
    {
        let n = fila.len();
        let cells: Vec<Cell<'_>> = fila
            .iter()
            .zip(textos)
            .enumerate()
            .map(|(i, ((id, rgba), text))| {
                let cell = Cell::new(Widget::Swatch {
                    id: *id,
                    rgba: *rgba,
                    text,
                    edit: None,
                    detach: None,
                });
                let cell = if l == 0 && i == 0 {
                    cell.label(rotulo)
                } else {
                    cell
                };
                if l == 0 && i + 1 == n {
                    cell.key(&ids)
                } else {
                    cell
                }
            })
            .collect();
        y = c.row(&cells, y);
    }
    y
}

/// ⭐ **Factos só de leitura aos pares** (`(rótulo, valor)`, Storage + Source): dois a dois enquanto
/// couberem, sem caixa, porque a caixa promete edição. Devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub fn facts(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    fatos: &[(&str, &str)],
) -> f32 {
    let cells: Vec<Cell<'_>> = fatos
        .iter()
        .map(|&(rotulo, text)| Cell::new(Widget::Readout { text }).label(rotulo))
        .collect();
    ctx(scene, text_system, store, hit_index, ink, x, w).compact(&cells, y)
}

/// ⭐ **Lei 6, uma tira de amostras da mesma grandeza** (os quatro cantos): uma linha só, sem o código;
/// a amostra abre o seletor, que o mostra. Um rótulo e uma marca para o grupo. Devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub fn swatch_strip(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: &str,
    amostras: &[(NodeId, Option<[u8; 4]>)],
) -> f32 {
    let ids: Vec<NodeId> = amostras.iter().map(|(id, _)| *id).collect();
    let n = amostras.len();
    let cells: Vec<Cell<'_>> = amostras
        .iter()
        .enumerate()
        .map(|(i, &(id, rgba))| {
            let cell = Cell::new(Widget::Swatch {
                id,
                rgba,
                text: "",
                edit: None,
                detach: None,
            });
            let cell = if i == 0 { cell.label(rotulo) } else { cell };
            if i + 1 == n { cell.key(&ids) } else { cell }
        })
        .collect();
    ctx(scene, text_system, store, hit_index, ink, x, w).row(&cells, y)
}

/// As duas colunas iguais de uma linha do corpo: `[(x, w); 2]`, com o vão de `metrics`.
#[must_use]
pub fn colunas_iguais(x: f32, w: f32) -> [(f32, f32); 2] {
    let meia = (w - m::FIELD_GAP) * 0.5;
    [(x, meia), (x + meia + m::FIELD_GAP, meia)]
}

/// O `y` a pedir para uma caixa de marcar ficar ao lado da caixa de uma coluna com rótulo pedida em `y`.
#[must_use]
pub fn caixa_ao_lado_do_rotulo(y: f32) -> f32 {
    y + m::LABEL_TOP + m::LABEL_PX + m::LABEL_BOTTOM - m::FIELD_ROW_GAP
}
