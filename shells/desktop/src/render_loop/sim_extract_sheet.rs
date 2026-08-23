//! **A FOLHA ABERTA no canvas** — as células FANTASMA de um sprite com grelha.
//!
//! Irmão do [`super::sim_extract_slice`], e o molde é o dele: uma função **pura** que transforma a
//! instância base em N, e um chamador que só as coloca. O 9-slice abre um sprite em nove quads de
//! *conteúdo*; isto abre-o em `hframes × vframes` quads de **pré-visualização**.
//!
//! # O problema que ele resolve
//!
//! Enio, 2026-08-23: *«você digita 8 quadros e não vê onde eles começam ou terminam»*.
//!
//! Um sprite com grelha desenha **UMA célula** — o `atlas_uv` da instância é a sub-UV daquele
//! frame. Então o artista escreve `hframes = 8`, vê um desenho, e não tem como saber se os cortes
//! caem onde a arte espera: um `7` em vez de `8` mostra cada célula com uma lasca da vizinha e a
//! deriva só aparece ao chegar ao fim da tira. ⚠️ **Arrastar a barra de frames mostra QUE algo
//! está errado; não mostra ONDE.**
//!
//! # ⚠️ Fantasma, e não documento
//!
//! As células extra são **presente puro** — não existem no `SimWorld`, não entram no undo, não
//! são salvas e desaparecem com o interruptor. É a mesma natureza do `override_for_entity` (a
//! pré-visualização de uma ferramenta), e o oposto do 9-slice, cujos nove quads **são** o que o
//! sprite é.
//!
//! ⚠️ Elas partilham o `SimRef` do sprite e levam `SlicePatchMirror` pela razão que o irmão
//! documenta: é isso que faz o passe pós-caminhada (z-order + clip) servi-las sem aprender nada, e
//! o HUD não as contar como entidades.
//!
//! # ⛔ Sem alocação, por construção
//!
//! Não há `Vec` de células: o chamador itera `0..cell_count` e chama [`cell`] por índice. Uma
//! grelha não tem teto declarado (`hframes × vframes` é do artista), e devolver uma coleção
//! obrigaria ou a um cap inventado ou a alocar por quadro — HR-3.

use ph2d_render::{RenderInstance, Sprite};

/// **A entidade cuja folha está aberta** — a pergunta, respondida num sítio só.
///
/// ⚠️ **Uma função, e não o predicado escrito duas vezes.** Ele é lido pelo EXTRACT (que emite as
/// células) e pelo OVERLAY (que desenha as linhas por cima delas); com uma cópia em cada, um
/// quadro em que discordassem daria linhas sobre células que não existem — e a divergência só
/// apareceria no ecrã. É a mesma lei que a caixa «Playing» pagou neste módulo em 2026-08-23.
pub(crate) fn previewed(hero: &ph2d_editor::screens::hero::HeroScreen) -> Option<ph2d_ecs::Entity> {
    matches!(
        hero.store.checkbox(ph2d_editor::ids::INSP_SHEET_PREVIEW),
        Some((_, ph2d_editor::widget::CheckboxValue::Checked))
    )
    .then(|| hero.gizmo.selection)
    .flatten()
    .map(ph2d_ecs::Entity::from_bits)
}

/// Quanto uma célula fantasma pesa contra a viva.
///
/// ⚠️ **Baixo o suficiente para a célula VIVA continuar a ser a resposta à pergunta «qual está no
/// ecrã?»**, e alto o suficiente para a arte se ler. A célula viva desenha-se a 100% pelo caminho
/// normal; se as vizinhas competissem com ela, a pré-visualização trocaria um problema (não vejo a
/// grelha) por outro (não vejo o frame).
pub(super) const GHOST_OPACITY: f32 = 0.28;

/// **A DECISÃO de abrir a folha deste sprite**, e quantas células ela tem.
///
/// ⚠️ **Extraída para ter gate, e a razão é uma limitação medida:** o `sim_extract::run` pede um
/// `SpriteRenderer` vivo, então o laço que de facto emite as células **não é alcançável de um
/// teste** sem um arnês de GPU — o mesmo buraco que deixa o fan-out do 9-slice sem gate de emissão
/// e os quatro goldens da spec por escrever. O que se pode fazer é **encolher o resíduo**: com a
/// decisão aqui, o que fica no extract é um `for` sobre duas funções já gateadas, curto o
/// suficiente para se conferir a olho.
///
/// `None` = não abrir. As três razões:
/// - a caixa não está marcada, ou está marcada sobre **outra** entidade;
/// - há uma **pré-visualização de ferramenta** neste sprite (a textura dela é um bake de quadro
///   inteiro, não uma folha — é a mesma razão pela qual o extract salta a sub-UV da célula);
/// - o sprite não tem grelha.
pub(super) fn should_open(
    previewed: Option<ph2d_ecs::Entity>,
    entity: ph2d_ecs::Entity,
    has_tool_override: bool,
    spr: &Sprite,
) -> Option<u32> {
    if previewed != Some(entity) || has_tool_override {
        return None;
    }
    cell_count(spr)
}

/// Quantas células a grelha deste sprite tem — `None` quando ele **não é uma folha**.
///
/// ⚠️ `1×1` devolve `None`, e não `Some(1)`: abrir uma folha de uma célula desenharia zero
/// fantasmas e um interruptor que não faz nada. *A ausência de grelha é uma resposta, não um caso
/// degenerado a tratar mais à frente.*
pub(super) fn cell_count(spr: &Sprite) -> Option<u32> {
    let n = spr.hframes.max(1).saturating_mul(spr.vframes.max(1));
    (n > 1).then_some(n)
}

/// A célula `index` posta no lugar dela: `(sub-UV, deslocamento do CENTRO em metros LOCAIS)`.
///
/// O deslocamento é relativo ao centro da célula **viva** — que é onde o quad do sprite já está.
///
/// `None` para a célula viva (o caminho normal já a desenha, e desenhá-la duas vezes somaria alfa
/// e daria a ela um realce que ninguém pediu) e para um índice fora da grelha.
///
/// ⚠️ **O `Y` do mundo cresce para CIMA e o `V` da textura cresce para BAIXO**, por isso a linha
/// seguinte da grelha fica **abaixo**: o sinal do `dy` é negativo. É a mesma conta do
/// `cy = y_top - …` que o [`ph2d_render::nine_slice`] faz.
pub(super) fn cell(spr: &Sprite, base_uv: [f32; 4], index: u32) -> Option<([f32; 4], [f32; 2])> {
    let cells = cell_count(spr)?;
    if index >= cells {
        return None;
    }
    let hf = spr.hframes.max(1);
    let live = spr.frame.min(cells - 1);
    if index == live {
        return None;
    }
    // ⚠️ **A sub-UV sai da MESMA função que o extract usa** para a célula viva
    // ([`super::sim_extract::sprite_sheet_subrect`]) — reimplementá-la aqui daria uma segunda
    // resposta a *«onde está a célula N»*, e a divergência só apareceria na pré-visualização.
    let uv = super::sim_extract::sprite_sheet_subrect(base_uv, spr.hframes, spr.vframes, index);
    let dcol = index % hf;
    let drow = index / hf;
    let lcol = live % hf;
    let lrow = live / hf;
    let dx = (f64::from(dcol) - f64::from(lcol)) as f32 * spr.size[0];
    let dy = (f64::from(lrow) - f64::from(drow)) as f32 * spr.size[1];
    Some((uv, [dx, dy]))
}

/// A instância fantasma: a base, com a sub-UV da célula, deslocada e esmaecida.
///
/// ⚠️ **O flip do sprite espelha a GRELHA, não cada célula no seu lugar** — a lição que o 9-slice
/// pagou (`apply_patch`, auditoria de 2026-08-22). O bit de flip já inverte o conteúdo de cada
/// quad no shader; o que falta é geométrico, e é **negar o deslocamento**. Sem isto, uma folha
/// espelhada abre-se com o conteúdo virado e as células na ordem original.
pub(super) fn ghost(base: &RenderInstance, uv: [f32; 4], offset: [f32; 2]) -> RenderInstance {
    let mut ri = *base;
    ri.atlas_uv = uv;
    let mirror = [
        base.flip_uv & RenderInstance::FLIP_X_BIT != 0,
        base.flip_uv & RenderInstance::FLIP_Y_BIT != 0,
    ];
    // O `anchor` É o centro do quad em metros locais (`local = anchor + quad_pos * size` no
    // shader), então deslocar é somar — e somar ao anchor do sprite preserva o pivô autorado.
    ri.anchor = [
        base.anchor[0] + if mirror[0] { -offset[0] } else { offset[0] },
        base.anchor[1] + if mirror[1] { -offset[1] } else { offset[1] },
    ];
    ri.opacity = base.opacity * GHOST_OPACITY;
    ri
}

#[cfg(test)]
#[path = "sim_extract_sheet_tests.rs"]
mod tests;
