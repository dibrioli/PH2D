//! **A linha ESMAECIDA** — um controlo cujo efeito depende de outro, e que AGORA não age (decisão do
//! dono, 2026-10-04: *«deixá-los esmaecidos»*; a tabela medida é o doc 45 §2.3).
//!
//! ⭐ **O `active = False` do Blender**: a linha inteira desenha-se com menos opacidade e CONTINUA
//! editável — o artista prepara o valor antes de ligar o que o acorda. Quem decide se a linha está
//! inerte é a ferramenta, pela lei que o motor usa ([`ph2d_tool_painter::Inercias`]); aqui só se pinta
//! e se põe a dica (o que liga o controlo) no id. O gate é o censo: dica ⇔ a sonda mede inerte.
//!
//! Vive nos quatro pintores de linha do painel (número, metade de um par, slider com chip, caixa de
//! marcar, menu) e não em cada sítio que os chama: um controlo dependente novo precisa só da LEI.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, Compose, Mix, RoundedRect, VelloBlend};

/// **O alfa da linha inativa — DERIVADO, não escolhido**: a fração que leva o `Text1` sobre a
/// superfície do painel (`Bg2`) à luminância do `TextDisabled`, o tom com que a casa já desenha um
/// texto desabilitado. Um tema que mude um dos três tokens muda o esmaecido com ele.
fn alfa_inativo(theme: Theme) -> f32 {
    let l = |t: ColorToken| t.resolve(theme).relative_luminance();
    let (fundo, texto, apagado) = (
        l(ColorToken::Bg2),
        l(ColorToken::Text1),
        l(ColorToken::TextDisabled),
    );
    if (texto - fundo).abs() < f64::EPSILON {
        return 1.0;
    }
    (((apagado - fundo) / (texto - fundo)) as f32).clamp(0.0, 1.0)
}

/// A dica do primeiro dos `ids` que a ferramenta diz inerte, com o instantâneo publicado AGORA.
fn dica(ids: &[NodeId]) -> Option<&'static str> {
    let inercias = crate::state::current_inercias();
    ids.iter().find_map(|id| inercias.dica_de(*id))
}

/// Uma dica que ESTE módulo escreveu — a única que ele pode apagar (outras linhas têm dicas próprias,
/// registadas no `populate`).
fn e_dica_de_inercia(texto: &str) -> bool {
    ph2d_tool_painter::Dependente::TODOS
        .iter()
        .map(|d| d.dica())
        .chain(std::iter::once(ph2d_tool_painter::DICA_DO_PARAMETRO))
        .any(|chave| tr(chave) == texto)
}

/// **Pinta `pintar` esmaecido se algum dos `ids` está inerte**, e põe (ou tira) a dica de cada id.
/// `area` é onde a linha pinta — a camada recorta-se a ela.
pub(crate) fn talvez_esmaecido<R>(
    ctx: &mut PaintCtx,
    theme: Theme,
    area: Rect,
    ids: &[NodeId],
    pintar: impl FnOnce(&mut PaintCtx) -> R,
) -> R {
    let dica = dica(ids);
    for id in ids {
        let store = ctx.host.store_mut();
        match dica {
            Some(chave) => store.set_tooltip(*id, tr(chave)),
            None if store.tooltip_for(*id).is_some_and(e_dica_de_inercia) => {
                store.set_tooltip(*id, "");
            }
            None => {}
        }
    }
    if dica.is_none() {
        return pintar(ctx);
    }
    let forma = RoundedRect::new(
        f64::from(area.x),
        f64::from(area.y),
        f64::from(area.x + area.w),
        f64::from(area.y + area.h),
        0.0,
    );
    let normal = VelloBlend::new(Mix::Normal, Compose::SrcOver);
    ctx.scene
        .push_layer_shape(normal, alfa_inativo(theme), Affine::IDENTITY, &forma);
    let r = pintar(ctx);
    ctx.scene.pop_layer();
    r
}

/// **Esmaece, POR CIMA, as opções inertes de um grupo já pintado** — para um widget que pinta várias
/// opções numa chamada só (o segmentado): o fundo do painel (`Bg2`) a `1 − α` sobre o rectângulo da
/// opção. Sobre o fundo liso do painel, `fundo·(1 − α) + linha·α` é exactamente a camada a `α` da
/// [`talvez_esmaecido`] — a mesma linha esmaecida por duas portas que concordam ao byte.
pub(crate) fn opcoes_esmaecidas(ctx: &mut PaintCtx, theme: Theme, ids: &[NodeId]) {
    let inercias = crate::state::current_inercias();
    let alfa = alfa_inativo(theme);
    for id in ids {
        let dica = inercias.dica_de(*id);
        let store = ctx.host.store_mut();
        match dica {
            Some(chave) => store.set_tooltip(*id, tr(chave)),
            None if store.tooltip_for(*id).is_some_and(e_dica_de_inercia) => {
                store.set_tooltip(*id, "");
            }
            None => {}
        }
        if dica.is_none() {
            continue;
        }
        let rect = ctx
            .host
            .hit_index_mut()
            .iter_registrations()
            .filter(|(i, _)| i == id)
            .last()
            .map(|(_, r)| r);
        if let Some(r) = rect {
            let veu =
                ph2d_editor_core::paint::resolve(ColorToken::Bg2, theme).multiply_alpha(1.0 - alfa);
            ph2d_editor_core::paint::fill_rounded_rect(ctx.scene, r, 0.0, veu);
        }
    }
}
