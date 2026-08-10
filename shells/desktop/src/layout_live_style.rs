//! **A TRADUÇÃO do vocabulário do documento para o do motor** — irmão do [`super`] pelo teto de
//! LOC, e o corte é por ASSUNTO: aqui mora UMA pergunta (*como é que isto se diz em flexbox?*), e
//! o que fica lá responde outra (*onde é que este filho ficou?*).
//!
//! ⚠️ Os `match` são exaustivos de propósito: uma variante nova no documento sem tradução aqui
//! **não compila**, em vez de cair num `_ =>` que a desenharia como outra coisa.

use ph2d_ecs::{VecLayout, VecLayoutItem, VecLayoutSize};
use ph2d_vec_layout::{Align, Dir, FrameStyle, ItemStyle, Justify, Len};

/// **A tradução do vocabulário do DOCUMENTO para o do MOTOR.**
///
/// ⚠️ Porta ÚNICA, e os `match` são exaustivos de propósito: uma direção nova no documento sem
/// tradução aqui **não compila**, em vez de cair num `_ =>` que a desenharia como uma linha.
///
/// ⚠️ **O `gap` chega RESOLVIDO** (W4c.4): o número autorado em `VecLayout::gap`, ou o comprimento
/// que um token de escala dá àquele eixo. Ele entra por aqui, e por mais lugar nenhum — é a mesma
/// razão de esta função existir: o motor tem de receber UM vocabulário, e um segundo sítio a
/// escolher entre o literal e o token faria a moldura espaçar por um número e o painel mostrar
/// outro.
pub(super) fn frame_style(l: &VecLayout, gap: [Option<f64>; 2]) -> FrameStyle {
    use ph2d_ecs::{LayoutAlign as A, LayoutDir as D, LayoutJustify as J};
    FrameStyle {
        dir: match l.dir {
            D::Row => Dir::Row,
            D::Column => Dir::Column,
            D::RowWrap => Dir::RowWrap,
            // ⚠️ **A contagem entra AQUI, e este é o único sítio onde os dois vocabulários se
            // encontram.** Do lado do documento ela é um campo (para sobreviver a uma troca de
            // direção); do lado do motor é o corpo do variante (para o `style_of` não perguntar
            // por um campo que a `FrameStyle` não tem). A assimetria é a mesma — e pelo mesmo
            // motivo — que a do [`size_of`], onde o `Fixed` do documento não carrega número.
            D::Grid => Dir::Grid { columns: l.columns },
        },
        gap: [gap[0].unwrap_or(l.gap[0]), gap[1].unwrap_or(l.gap[1])],
        pad: l.pad,
        align: match l.align {
            A::Start => Align::Start,
            A::Center => Align::Center,
            A::End => Align::End,
            A::Stretch => Align::Stretch,
        },
        justify: match l.justify {
            J::Start => Justify::Start,
            J::Center => Justify::Center,
            J::End => Justify::End,
            J::SpaceBetween => Justify::SpaceBetween,
            J::SpaceAround => Justify::SpaceAround,
        },
    }
}

/// **O tamanho de um nó, traduzido** — porta ÚNICA, irmã exacta do [`frame_style`].
///
/// ⚠️ A `bbox` entra porque o `Fixed` do DOCUMENTO não carrega número: ele diz *"o tamanho que a
/// forma tem"*, e quem sabe qual é esse número é a geometria medida. O motor precisa do número;
/// o documento não o guarda — e é essa assimetria que impede a segunda resposta a *"que tamanho
/// tem esta moldura?"*.
///
/// ⚠️ **O abraço só é honrado num nó que FLUI.** Um nó sem `VecLayout` que traga um `Hug` autorado
/// (de quando ele ainda dispunha, e o artista desligou o fluxo) resolveria para zero e a forma
/// **desapareceria**; aqui ele cai de volta para o tamanho medido, e o motor nunca vê o pedido.
pub(super) fn size_of(
    s: Option<&VecLayoutSize>,
    flows: bool,
    bbox: [f64; 2],
) -> ([Len; 2], [Option<f64>; 2], [Option<f64>; 2]) {
    let Some(s) = s else {
        return (
            [Len::Fixed(bbox[0]), Len::Fixed(bbox[1])],
            [None; 2],
            [None; 2],
        );
    };
    let axis = |i: usize| match s.size[i] {
        ph2d_ecs::LayoutSize::Hug if flows => Len::Hug,
        _ => Len::Fixed(bbox[i]),
    };
    ([axis(0), axis(1)], s.min, s.max)
}

pub(super) fn item_style(it: Option<&VecLayoutItem>) -> ItemStyle {
    it.map_or_else(ItemStyle::default, |i| ItemStyle {
        grow: i.grow,
        shrink: i.shrink,
        basis: i.basis,
    })
}
