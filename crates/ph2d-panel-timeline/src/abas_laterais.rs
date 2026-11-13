//! ⭐ **As abas da Timeline são abas de PASTA na borda esquerda** (dono 13/11,
//! `docs/Timeline/fotos/18_pedido_abas_laterais_esboco.png`; 2.ª volta: «devem ocupar toda a lateral e
//! crescer com a Timeline»). Keys · Containers · (um por container aberto) · Arrange dividem a altura
//! inteira da lista, de cima para baixo, coladas à moldura por fora, texto a ler de baixo para cima.
//! A ATIVA leva o dobro da altura de uma inativa ([`PESO_ATIVA`]), na tinta da lista (painel e aba
//! leem-se como um objecto), e fica à frente; as INATIVAS, sólidas e apagadas, por trás dela. Cada aba
//! tem as pontas inclinadas (como abas de navegador): as das vizinhas cruzam-se e a da frente cobre.
//!
//! A lista de abas, a escolhida e os ids são os de [`crate::transport_tabs`] — o clique é o de sempre.
//! O cartão reserva a calha delas à esquerda ([`sem_a_calha`]): o painel pode estar encostado à borda
//! da janela e as abas não saem dele.

use ph2d_editor_core::paint;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::banda::{self, metrics as m};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, Theme, TimelineTinta};

use crate::tab::Tab;

/// A grossura de uma aba (a largura da calha): a altura de um campo da barra.
pub(crate) const ESPESSURA: f32 = m::FIELD_H;

/// Quanto a base de uma aba passa da fatia dela, de cada lado (a ponta inclinada que se cruza com a
/// da vizinha).
const PONTA: f32 = m::FIELD_RADIUS;

/// O raio das quinas de fora.
const RAIO: f32 = m::FIELD_RADIUS;

/// O tamanho do nome numa aba: o dos valores dos campos.
const TEXTO_PX: f32 = m::VALUE_PX;

/// A ativa pesa o dobro de uma inativa na divisão da altura (o esboço do dono: a 1.ª alta, as outras
/// menores).
const PESO_ATIVA: f32 = 2.0;

/// Pontos por quina arredondada (a quina é uma quadrática amostrada, sem trigonometria).
const PASSOS_DA_QUINA: usize = 4;

/// O rect do cartão SEM a calha das abas: é dele que a moldura, a barra e a folha se medem.
pub(crate) fn sem_a_calha(rect: Rect) -> Rect {
    Rect::new(
        rect.x + ESPESSURA,
        rect.y,
        (rect.w - ESPESSURA).max(0.0),
        rect.h,
    )
}

/// Uma aba posta: o alvo (a fatia dela na lateral), o desenho (a fatia com as pontas) e o nome que
/// cabe à vista.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Aba {
    pub(crate) id: ph2d_a11y::NodeId,
    pub(crate) nome: String,
    /// O nome inteiro (a dica da aba: a inativa estreita mostra só o começo).
    pub(crate) inteiro: String,
    pub(crate) alvo: Rect,
    pub(crate) desenho: Rect,
    pub(crate) ativa: bool,
    /// A distância (em abas) à ativa: as mais longe pintam-se primeiro, por trás.
    pub(crate) fundo: usize,
}

/// O começo de `nome` que cabe em `w` (por carácter, sem reticências: a aba da frente cobre o resto).
fn comeco(nome: &str, w: f32, largura: &mut dyn FnMut(&str) -> f32) -> String {
    if largura(nome) <= w {
        return nome.to_owned();
    }
    let mut fim = 0;
    for (i, c) in nome.char_indices() {
        let j = i + c.len_utf8();
        if largura(&nome[..j]) > w {
            break;
        }
        fim = j;
    }
    nome[..fim].to_owned()
}

/// A folga do nome em cada ponta da aba: a ponta inclinada e o recuo de um segmento.
const FOLGA: f32 = PONTA + m::SEG_INSET;

/// ⭐ **A pilha**: as abas na ordem de [`crate::transport_tabs::items`] dividem a lateral inteira da
/// lista (entre as quinas dela), a ativa com [`PESO_ATIVA`] e nunca menos que o nome inteiro. Pura (a
/// largura do texto entra por fora), para o gate medir sem pintar.
pub(crate) fn pilha(
    lista: Rect,
    snap: &TimelineViewSnapshot,
    tab: Tab,
    largura: &mut dyn FnMut(&str) -> f32,
) -> Vec<Aba> {
    let celulas = crate::transport_tabs::items(snap);
    let escolhida = crate::transport_tabs::selected(tab, snap);
    let n = celulas.len();
    let x = lista.x - ESPESSURA;
    let topo = lista.y + m::LIST_RADIUS;
    let altura = (lista.h - m::LIST_RADIUS * 2.0).max(0.0);
    let pesos = PESO_ATIVA + n.saturating_sub(1) as f32;
    let nome_da_ativa = celulas
        .get(escolhida)
        .map_or(0.0, |(_, t)| largura(t) + FOLGA * 2.0);
    let ativa_h = (altura * PESO_ATIVA / pesos).max(nome_da_ativa.min(altura));
    let inativa_h = (altura - ativa_h).max(0.0) / n.saturating_sub(1).max(1) as f32;
    let mut y = topo;
    let mut out = Vec::with_capacity(n);
    for (i, (id, nome)) in celulas.into_iter().enumerate() {
        let ativa = i == escolhida;
        let h = if ativa { ativa_h } else { inativa_h };
        // A ativa entra um pouco na moldura (a junta não se vê); a base passa a fatia pelas pontas.
        let dentro = if ativa { m::SEG_INSET } else { 0.0 };
        out.push(Aba {
            id,
            nome: comeco(&nome, h - FOLGA * 2.0, largura),
            inteiro: nome,
            alvo: Rect::new(x, y, ESPESSURA, h),
            desenho: Rect::new(x, y - PONTA, ESPESSURA + dentro, h + PONTA * 2.0),
            ativa,
            fundo: i.abs_diff(escolhida),
        });
        y += h;
    }
    out
}

/// Os pontos da quadrática `a → c → b` (o canto `c`), sem o `a`.
fn quina(out: &mut Vec<(f32, f32)>, a: (f32, f32), c: (f32, f32), b: (f32, f32)) {
    for k in 1..=PASSOS_DA_QUINA {
        let t = k as f32 / PASSOS_DA_QUINA as f32;
        let u = 1.0 - t;
        out.push((
            u * u * a.0 + 2.0 * u * t * c.0 + t * t * b.0,
            u * u * a.1 + 2.0 * u * t * c.1 + t * t * b.1,
        ));
    }
}

/// O contorno de uma aba: a base (o lado da moldura) reta e mais comprida, o lado de fora mais
/// curto, as pontas inclinadas e as quinas de fora redondas.
pub(crate) fn contorno(d: Rect) -> Vec<(f32, f32)> {
    let (fora, base) = (d.x, d.x + d.w);
    let (cima, baixo) = (d.y, d.y + d.h);
    let (fc, fb) = (cima + PONTA * 2.0, baixo - PONTA * 2.0);
    // Ao longo de cada ponta inclinada, a fracção que a quina come.
    let r = (RAIO / d.w.max(1.0)).min(0.5);
    let perto = |de: (f32, f32), canto: (f32, f32)| {
        (
            de.0 + (canto.0 - de.0) * (1.0 - r),
            de.1 + (canto.1 - de.1) * (1.0 - r),
        )
    };
    let (c1, c2) = ((fora, fc), (fora, fb));
    let a1 = perto((base, cima), c1);
    let mut p = vec![(base, cima), a1];
    quina(&mut p, a1, c1, (fora, fc + RAIO));
    p.push((fora, fb - RAIO));
    quina(&mut p, (fora, fb - RAIO), c2, perto((base, baixo), c2));
    p.push((base, baixo));
    p
}

fn pilha_pintada(
    ctx: &mut PaintCtx,
    painel: Rect,
    snap: &TimelineViewSnapshot,
    tab: Tab,
) -> Vec<Aba> {
    let lista = banda::geometria(painel).lista;
    let ts = &mut *ctx.text_system;
    pilha(lista, snap, tab, &mut |t| ts.prefix_width(t, TEXTO_PX))
}

fn pinta_uma(ctx: &mut PaintCtx, theme: Theme, a: &Aba) {
    let t = |x| crate::folha_tinta::tinta(x, theme);
    let hover = ctx.host.store().hover_live(a.id) > 0.0;
    let (fundo, texto) = if a.ativa {
        (ColorToken::KanbanList, t(TimelineTinta::TextoCabecalho))
    } else if hover {
        (
            ColorToken::CardFieldHover,
            t(TimelineTinta::TextoPropriedade),
        )
    } else {
        (ColorToken::CardField, t(TimelineTinta::TextoValor))
    };
    ph2d_editor_core::paint_shapes::fill_polygon(
        ctx.scene,
        &contorno(a.desenho),
        paint::resolve(fundo, theme),
    );
    // O nome ao centro da fatia; se não cabe, o começo a partir de baixo (onde se começa a ler).
    let w = ctx.text_system.prefix_width(&a.nome, TEXTO_PX);
    let vao = a.alvo.h - FOLGA * 2.0;
    let pe = a.alvo.y + a.alvo.h - FOLGA - ((vao - w) * 0.5).max(0.0);
    let tx = a.alvo.x + paint::folga_do_texto(a.alvo.w, TEXTO_PX);
    paint::paint_text_rotated_ccw(
        ctx.text_system,
        ctx.scene,
        &a.nome,
        tx.round(),
        pe.round(),
        TEXTO_PX,
        w.ceil() + 1.0,
        texto,
    );
}

/// Pinta as abas DEPOIS da moldura (a sombra dela não lhes cai em cima): a mais longe da ativa
/// primeiro, a ativa por último; e regista o alvo de cada uma (a fatia dela).
pub(crate) fn pinta(
    ctx: &mut PaintCtx,
    theme: Theme,
    painel: Rect,
    (snap, tab): (&TimelineViewSnapshot, Tab),
) {
    let mut abas = pilha_pintada(ctx, painel, snap, tab);
    ph2d_editor_core::widget::composto::grupo(abas.iter().map(|a| a.id));
    for a in &abas {
        ctx.host.hit_index_mut().register(a.id, a.alvo);
        let store = ctx.host.store_mut();
        if store.tooltip_for(a.id) != Some(a.inteiro.as_str()) {
            store.set_tooltip(a.id, a.inteiro.clone());
        }
    }
    abas.sort_by_key(|a| std::cmp::Reverse(a.fundo));
    for a in &abas {
        pinta_uma(ctx, theme, a);
    }
}

#[cfg(test)]
#[path = "abas_laterais_tests.rs"]
mod tests;
