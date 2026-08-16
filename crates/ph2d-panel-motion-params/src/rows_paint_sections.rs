//! **Os cabeçalhos de SEÇÃO do painel de params** (doc 88 B3, o passe visual).
//!
//! Um nó com treze params cabe na tela e ainda assim não se LÊ. Agrupá-los é o que todo
//! editor profissional faz; aqui o grupo é declarado pelo nó (`ParamGroup`, side-metadata do
//! registry) e a ponte entrega as rows JÁ ordenadas, com as soltas primeiro — então este
//! módulo só responde *"antes da row `i`, desenhe o cabeçalho `t`"*.
//!
//! ⚠️ Usa o cabeçalho CANÔNICO (`docs/UI_Padrao/components/section_header.md`: *"TODA seção
//! usa `paint_section_header`"* · *"TODA seção é colapsável"*), nunca um `paint_text` caseiro
//! — foi improvisar isso que fez o painel do Vector confundir categoria com tipo de forma.
//!
//! ⚠️ O id sai do TÍTULO, então duas seções "Range" de nós diferentes compartilham o estado de
//! dobra. É deliberado e é o comportamento do Blender: o artista dobra *Range*, não *o Range
//! DAQUELE nó*, e a dobra sobrevive à troca de seleção — que é o que a torna útil.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
pub(crate) use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::{SectionHeader, paint_section_header};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Id estável do cabeçalho, derivado do título (hash de string — sem pool, sem teto).
pub(crate) fn section_id(title: &str) -> NodeId {
    // O MESMO mixer do `snapshot_ids::fnv_id` — dois hashes para a mesma família de ids
    // acabariam colidindo entre si em vez de nunca colidir.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in format!("motion_param/section/{title}").bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    NodeId(h)
}

/// **Uma seção começa na row `i`?** — a pergunta que o laço faz ANTES de pintar, porque a dobra
/// da seção anterior tem de FECHAR antes de o cabeçalho novo ser desenhado (senão ele sai dentro
/// do recorte dela).
///
/// ⚠️ Mora aqui, ao lado do `header_at`, e ele a CHAMA: duas varreduras da mesma lista seriam
/// duas respostas a *"há cabeçalho aqui?"*, e elas divergem no dia em que a ordenação mudar.
pub(crate) fn has_header_at(section_at: &[(String, usize)], i: usize) -> bool {
    section_at.iter().any(|(_, at)| *at == i)
}

/// Se uma seção começa na row `i`, desenha o cabeçalho dela e devolve `(altura usada, a dobra)`.
///
/// ⚠️ Devolve as DUAS coisas porque quem pergunta precisa das duas — a altura para avançar, e a
/// dobra para (a) saber se pula as rows seguintes e (b) fechá-la no fim. Separá-las obrigaria o
/// chamador a re-derivar o id a partir do título, que é a segunda cópia de *como se chama esta
/// seção*.
///
/// ⚠️ **`Option<SectionFold>` e não o `bool` de antes** (F4b): o `bool` vinha do `is_collapsed`,
/// que vira no quadro do clique enquanto o `t` ainda desce — as rows gateadas nele sumiriam de
/// repente por baixo de um chevron a rodar.
#[expect(
    clippy::too_many_arguments,
    reason = "espelha a porta de paint das rows deste painel"
)]
pub(crate) fn header_at(
    section_at: &[(String, usize)],
    i: usize,
    inner_x: f32,
    inner_w: f32,
    y: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> Option<(f32, Option<SectionFold>)> {
    if !has_header_at(section_at, i) {
        return None;
    }
    let (title, _) = section_at.iter().find(|(_, at)| *at == i)?;
    let dy = paint_header(
        title,
        inner_x,
        inner_w,
        y,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    let body_top = y + dy;
    let fold = SectionFold::begin(
        store,
        section_id(title),
        inner_x,
        inner_w,
        body_top,
        scene,
        hit_index,
    );
    Some((dy, fold))
}

/// Desenha o cabeçalho e registra o hit-rect. Devolve a altura usada.
#[expect(
    clippy::too_many_arguments,
    reason = "espelha a porta de paint das rows deste painel"
)]
pub(crate) fn paint_header(
    title: &str,
    inner_x: f32,
    inner_w: f32,
    y: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let id = section_id(title);
    let h = TypeToken::Md.px() + Spacing::Md.px();
    let header = SectionHeader::new(id, title)
        .collapsible(!store.is_collapsed(id))
        .open_t(store.section_open_live(id));
    let rect = Rect::new(inner_x, y, inner_w, h);
    paint_section_header(&header, rect, scene, text_system, theme);
    hit_index.register(id, rect);
    h + Spacing::Xs.px()
}
