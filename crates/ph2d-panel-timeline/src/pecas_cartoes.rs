//! ⭐ **A PEÇA nos cartões** (`docs/Timeline/16_onda3_containers_arrange.md`, maquete B3/B4 `.c13st`): o
//! corpo de uma strip no Arrange/Containers e a barra de um container na lista, com UM pintor.
//!
//! Só aparência: o rect, os ids e os alvos são os de quem chama. `PH2D_CARDS=0` nunca chega aqui.
//! A cor: o clipe `i` veste `PECAS[i % 5]` pela ordem dos chips, o container veste `PecaContainer`.

use std::f32::consts::SQRT_2;

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, frame_radius, paint_icon, paint_text_elided,
    paint_text_title_elided, push_rounded_clip, stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::banda::{degrade_vertical_em, metrics as m};
use ph2d_editor_core::panel::cards::ferramenta::cards_on;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{StripView, TimelineViewSnapshot};
use ph2d_tokens::{
    PECAS, Spacing, StrokeToken, TIMELINE_MISTURA_ALPHA, TIMELINE_MISTURA_BORDA_ALPHA,
    TIMELINE_PECA_BORDA_ALPHA, TIMELINE_PECA_MUDA_ALPHA, TIMELINE_PECA_PE_ALPHA,
    TIMELINE_PECA_RAIO_PX, TIMELINE_PECA_TOPO_ALPHA, Theme, TimelineTinta,
};
use ph2d_vector::Color;

use crate::folha_tinta::tinta;
use crate::stack_ease_grip::BAND_IN;

/// A margem de dentro, à esquerda e à direita (`padding: 0 7px` da `.c13st`).
const PAD_X: f32 = 7.0; // LITERAL-PX-OK: o padding horizontal da peça, medido na maquete
/// O nome (10,5 px na maquete): o rótulo do cartão.
const FONTE: f32 = m::LABEL_PX;
/// O ícone do tipo: 11 px na maquete, o tamanho do próprio rótulo.
const GLIFO: f32 = m::LABEL_PX;
/// A listra clara da mistura e o seu período (`.c13ov`: 3 px claros, 4 px de fundo).
const LISTRA_W: f32 = 3.0; // LITERAL-PX-OK: a listra clara da mistura, medida na maquete
const LISTRA_PERIODO: f32 = 7.0; // LITERAL-PX-OK: listra + vão da mistura (3 + 4), medidos na maquete
/// O fundo entre as listras (`bg .25`).
const MISTURA_FUNDO_ALPHA: f32 = 0.25; // LITERAL-PX-OK: opacidade do fundo da mistura na maquete
/// O raio da mistura (`border-radius: 4px`).
const MISTURA_RAIO: f32 = 4.0; // LITERAL-PX-OK: o raio da mistura, medido na maquete
/// O anel de fora da peça escolhida (`.c13st.s`: o foco a 60 %).
const ANEL_ALPHA: f32 = 0.6; // LITERAL-PX-OK: opacidade do anel de foco da peça escolhida
/// FNV-1a: a cor de um nome que não está na lista, estável entre corridas.
const FNV_BASE: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIMO: u64 = 0x0100_0000_01b3;

/// O que uma peça mostra.
#[derive(Copy, Clone, Debug)]
pub(crate) struct PecaDesenho<'a> {
    /// A cor da peça: uma de [`PECAS`] ou [`TimelineTinta::PecaContainer`].
    pub tinta: TimelineTinta,
    /// O glifo do tipo: o filme (clipe) ou o cubo (container), os dos chips.
    pub icon: IconId,
    /// O nome, em negrito; corta com reticência.
    pub nome: &'a str,
    /// O que vem depois do nome, mais apagado (o fator de velocidade, `2x`).
    pub extra: Option<&'a str>,
    /// Faixa muda (ou de peso zero): tudo a [`TIMELINE_PECA_MUDA_ALPHA`].
    pub muda: bool,
    /// A peça escolhida: anel claro por dentro, o do foco por fora.
    pub escolhida: bool,
    pub theme: Theme,
}

/// As tintas de uma peça, já com a opacidade — a porta que o pintor e os portões leem.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Cores {
    pub topo: Color,
    pub pe: Color,
    pub borda: Color,
    pub texto: Color,
    pub extra: Color,
    /// `[por dentro, por fora]`, só na escolhida.
    pub anel: Option<[Color; 2]>,
}

/// ⭐ **A porta ÚNICA do desbote da faixa muda** (dono 10/11: «as cores originais, meio apagadas»):
/// a opacidade que o corpo, a mistura, o rótulo e as alças da strip multiplicam — o matiz nunca muda.
pub(crate) const fn desbote(muda: bool) -> f32 {
    if muda { TIMELINE_PECA_MUDA_ALPHA } else { 1.0 }
}

/// As tintas de `p`.
pub(crate) fn cores(p: &PecaDesenho<'_>) -> Cores {
    let k = desbote(p.muda);
    let cor = tinta(p.tinta, p.theme);
    let de = |t: TimelineTinta| tinta(t, p.theme).multiply_alpha(k);
    Cores {
        topo: cor.multiply_alpha(TIMELINE_PECA_TOPO_ALPHA * k),
        pe: cor.multiply_alpha(TIMELINE_PECA_PE_ALPHA * k),
        borda: cor.multiply_alpha(TIMELINE_PECA_BORDA_ALPHA * k),
        texto: de(TimelineTinta::TextoCabecalho),
        extra: de(TimelineTinta::TextoValor),
        anel: p.escolhida.then(|| {
            [
                de(TimelineTinta::TextoCabecalho),
                de(TimelineTinta::ChaveEscolhida).multiply_alpha(ANEL_ALPHA),
            ]
        }),
    }
}

/// A peça inteira: o fundo e o rótulo no rect todo.
pub(crate) fn pinta_peca(ctx: &mut PaintCtx, rect: Rect, p: &PecaDesenho<'_>) {
    pinta_fundo(ctx, rect, p);
    pinta_rotulo(ctx, rect, (rect.x, rect.x + rect.w), p);
}

/// `r` encolhido de `d` em cada lado (`d < 0` cresce).
fn encolhe(r: Rect, d: f32) -> Rect {
    Rect::new(
        r.x + d,
        r.y + d,
        (r.w - d * 2.0).max(0.0),
        (r.h - d * 2.0).max(0.0),
    )
}

/// O fundo: o degradê da cor sobre a fita, o contorno de dentro e, na escolhida, os dois anéis.
pub(crate) fn pinta_fundo(ctx: &mut PaintCtx, rect: Rect, p: &PecaDesenho<'_>) {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    let c = cores(p);
    let raio = frame_radius(p.theme, TIMELINE_PECA_RAIO_PX)
        .min(rect.h * 0.5)
        .min(rect.w * 0.5);
    degrade_vertical_em(ctx.scene, rect, raio, [c.topo, c.pe]);
    let fio = StrokeToken::Thin.px();
    // FRAME-RAW-OK: o contorno da PEÇA, na cor dela (`.c13st`, 1 px a 80 %): duas peças vizinhas
    // da mesma cor só se separam por ele — é a identidade da peça, não a moldura de um controlo.
    let por_dentro = |w: f32| (raio - w * 0.5).max(0.0);
    stroke_rounded_rect(
        ctx.scene,
        encolhe(rect, fio * 0.5),
        por_dentro(fio),
        fio,
        c.borda,
    );
    if let Some([dentro, fora]) = c.anel {
        let w = StrokeToken::Thick.px();
        // FRAME-RAW-OK: o anel da peça escolhida (`.c13st.s`: 2 px claros por dentro).
        stroke_rounded_rect(ctx.scene, encolhe(rect, w * 0.5), por_dentro(w), w, dentro);
        // FRAME-RAW-OK: e o do foco por fora, a 60 % (o `box-shadow` de 2 px da maquete).
        stroke_rounded_rect(ctx.scene, encolhe(rect, -w * 0.5), raio + w * 0.5, w, fora);
    }
}

/// O rótulo: ícone 11 px · nome em negrito · o `extra` apagado, da esquerda, dentro de `livre`
/// (o vão horizontal que as faixas de fade deixam), centrado pelas maiúsculas na altura de `peca`.
pub(crate) fn pinta_rotulo(ctx: &mut PaintCtx, peca: Rect, livre: (f32, f32), p: &PecaDesenho<'_>) {
    let c = cores(p);
    let x1 = livre.1.min(peca.x + peca.w) - PAD_X;
    let mut x = livre.0.max(peca.x) + PAD_X;
    if x1 - x < GLIFO || peca.h < GLIFO {
        return;
    }
    let vao = Spacing::Xs.px();
    let g = Rect::new(x, peca.y + (peca.h - GLIFO) * 0.5, GLIFO, GLIFO);
    paint_icon(ctx.scene, p.icon, g, c.texto, StrokeToken::Default.px());
    x += GLIFO + vao;
    let extra_w = p
        .extra
        .map_or(0.0, |e| ctx.text_system.prefix_width(e, FONTE).ceil() + vao);
    let budget = (x1 - x - extra_w).max(0.0);
    let y = peca.y + folga_do_texto(peca.h, FONTE);
    if budget > 0.0 {
        paint_text_title_elided(
            ctx.text_system,
            ctx.scene,
            p.nome,
            x,
            y,
            FONTE,
            budget,
            c.texto,
        );
    }
    if let Some(e) = p.extra {
        let nome_w =
            ph2d_editor_core::text_elide::largura_do_titulo(ctx.text_system, p.nome, FONTE);
        let ex = x + nome_w.min(budget) + if budget > 0.0 { vao } else { 0.0 };
        paint_text_elided(
            ctx.text_system,
            ctx.scene,
            e,
            ex,
            y,
            FONTE,
            (x1 - ex).max(0.0),
            c.extra,
        );
    }
}

/// ⭐ **A mistura** (`.c13ov`): uma faixa de fade nos cartões — listras a 135° claras a
/// [`TIMELINE_MISTURA_ALPHA`] sobre a fita a 25 %, contorno claro a [`TIMELINE_MISTURA_BORDA_ALPHA`].
pub(crate) fn pinta_mistura(ctx: &mut PaintCtx, theme: Theme, r: Rect, muda: bool) {
    if r.w <= 0.0 || r.h <= 0.0 {
        return;
    }
    let (listra, fundo, borda) = tons_da_mistura(theme, muda);
    let raio = MISTURA_RAIO.min(r.h * 0.5).min(r.w * 0.5);
    fill_rounded_rect(ctx.scene, r, raio, fundo);
    push_rounded_clip(ctx.scene, r, raio);
    let passo = LISTRA_PERIODO * SQRT_2;
    let mut x = r.x - r.h;
    while x < r.x + r.w {
        stroke_polyline(
            ctx.scene,
            &[(x, r.y + r.h), (x + r.h, r.y)],
            LISTRA_W,
            listra,
        );
        x += passo;
    }
    ctx.scene.pop_layer();
    let fio = StrokeToken::Thin.px();
    // FRAME-RAW-OK: o contorno da MISTURA (`.c13ov`, 1 px claro a 55 %): é o desenho do fade.
    stroke_rounded_rect(
        ctx.scene,
        encolhe(r, fio * 0.5),
        (raio - fio * 0.5).max(0.0),
        fio,
        borda,
    );
}

/// `(listra, fundo, contorno)` da mistura — a porta que o pintor e o portão leem.
pub(crate) fn tons_da_mistura(theme: Theme, muda: bool) -> (Color, Color, Color) {
    let k = desbote(muda);
    let claro = tinta(TimelineTinta::TextoCabecalho, theme);
    (
        claro.multiply_alpha(TIMELINE_MISTURA_ALPHA * k),
        tinta(TimelineTinta::FitaPropriedade, theme).multiply_alpha(MISTURA_FUNDO_ALPHA * k),
        claro.multiply_alpha(TIMELINE_MISTURA_BORDA_ALPHA * k),
    )
}

/// O índice do clipe `nome` na ordem dos chips: a lista agrupada do Arrange da Cena (várias
/// animações), senão a da animação em vista; um nome fora das duas cai num hash estável.
pub(crate) fn indice_do_clipe(snap: &TimelineViewSnapshot, nome: &str) -> usize {
    let agrupada = || {
        snap.animation
            .sources
            .iter()
            .flat_map(|g| g.clips.iter())
            .position(|(_, n, _)| n == nome)
    };
    let da_vista = || snap.clips.iter().position(|n| n == nome);
    let achado = if snap.animation.sources.len() > 1 {
        agrupada().or_else(da_vista)
    } else {
        da_vista().or_else(agrupada)
    };
    achado.unwrap_or_else(|| {
        let h = nome
            .bytes()
            .fold(FNV_BASE, |h, b| (h ^ u64::from(b)).wrapping_mul(FNV_PRIMO));
        (h % PECAS.len() as u64) as usize
    })
}

/// A cor de uma strip: a do container, ou a do clipe pela ordem dele.
pub(crate) fn tinta_do_strip(s: &StripView, snap: &TimelineViewSnapshot) -> TimelineTinta {
    if s.container.is_some() {
        return TimelineTinta::PecaContainer;
    }
    PECAS[indice_do_clipe(snap, &s.clip_name) % PECAS.len()]
}

/// A peça de uma strip nos cartões; `None` no clássico (a strip pinta-se como sempre).
pub(crate) fn peca_do_strip<'a>(
    s: &'a StripView,
    tinta: TimelineTinta,
    muda: bool,
    theme: Theme,
) -> Option<PecaDesenho<'a>> {
    cards_on().then_some(PecaDesenho {
        tinta,
        icon: if s.container.is_some() {
            IconId::Cube
        } else {
            IconId::Film
        },
        nome: &s.clip_name,
        extra: None,
        muda,
        escolhida: false,
        theme,
    })
}

/// O rótulo de uma strip: no vão entre as faixas de fade de DENTRO, com o fator de velocidade.
pub(crate) fn rotulo_do_strip(
    ctx: &mut PaintCtx,
    body: Rect,
    s: &StripView,
    view: crate::graph::TimeView,
    p: PecaDesenho<'_>,
) {
    let bandas = crate::strip_paint::fade_bands(s, view, body);
    let livre = vao_livre(body, bandas.iter().map(|(e, r, _)| (*e, *r)));
    let fator = (s.speed - 1.0).abs() > crate::strip_paint::SPEED_EPS;
    let extra = fator.then(|| fator_str(s.speed));
    let p = PecaDesenho {
        extra: extra.as_deref(),
        ..p
    };
    pinta_rotulo(ctx, body, livre, &p);
}

/// O vão `(x0, x1)` de `body` que as faixas de fade de DENTRO deixam; o corpo inteiro quando elas
/// o cobrem (um rótulo engolido pelo fade não diz de quem é a peça).
pub(crate) fn vao_livre(body: Rect, bandas: impl Iterator<Item = (u8, Rect)>) -> (f32, f32) {
    let (mut x0, mut x1) = (body.x, body.x + body.w);
    for (edge, r) in bandas {
        if r.x < body.x || r.x + r.w > body.x + body.w {
            continue; // de fora, no vão ao lado do corpo
        }
        if edge == BAND_IN {
            x0 = x0.max(r.x + r.w);
        } else {
            x1 = x1.min(r.x);
        }
    }
    if x1 - x0 < GLIFO + PAD_X * 2.0 {
        (body.x, body.x + body.w)
    } else {
        (x0, x1)
    }
}

/// `2x`, `0.5x`, `1.5x` — o fator de esticamento, sem zeros à direita.
pub(crate) fn fator_str(speed: f64) -> String {
    let s = format!("{speed:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    format!("{s}x")
}

#[cfg(test)]
#[path = "pecas_cartoes_tests.rs"]
mod tests;
