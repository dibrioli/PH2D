//! Uma acção = UMA fileira, como o diálogo do Aseprite e a lei do cartão compacto (Figma):
//! `Nome ……… [Ctrl+E] [+] [↺]`. As fichas encostam à direita; as que não cabem ao lado do
//! nome descem para fileiras próprias, também à direita. O `↺` só aparece (e só ocupa lugar) fora
//! da fábrica: na janela estreita cada píxel vai ao nome da acção.

use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{folga_do_texto, paint_text_elided};
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::{CardCtx, Seg};
use ph2d_editor_core::zones::Rect;
use ph2d_pixel_keys::PixelAction;
use ph2d_tokens::ColorToken;

/// Uma ficha: o id (o clique tira-a) e o texto — o contexto, quando não é `Normal`, vai DENTRO
/// dela («Space+H · Transformation»): texto solto ao lado lia-se no censo do ritmo como o rótulo
/// da ficha da fileira de baixo (`lei_do_ritmo`).
pub(crate) type Ficha = (NodeId, String);

/// Uma linha da lista: a acção, as fichas e se difere da fábrica.
pub(crate) struct Linha {
    pub acao: PixelAction,
    pub fichas: Vec<Ficha>,
    pub mudada: bool,
}

fn ficha_w(c: &mut CardCtx<'_>, f: &Ficha) -> f32 {
    c.botao_w(&f.1)
}

fn pinta_ficha(c: &mut CardCtx<'_>, f: &Ficha, x: f32, y: f32) {
    let bw = c.botao_w(&f.1);
    c.botao_em(f.0, &f.1, Rect::new(x, y, bw, m::FIELD_H));
}

/// Pinta a fileira de `l` a partir de `y` (o fundo da anterior) e devolve o fundo dela.
pub(crate) fn pinta(c: &mut CardCtx<'_>, l: &Linha, add: NodeId, reset: NodeId, y: f32) -> f32 {
    let topo = y + m::FIELD_ROW_GAP;
    let direita = c.x + c.w;
    let segs: Vec<Seg<'_>> = std::iter::once(Seg {
        id: add,
        icon: Some(IconId::Plus),
        text: "",
        on: false,
    })
    .chain(l.mudada.then_some(Seg {
        id: reset,
        icon: Some(IconId::Reset),
        text: "",
        on: false,
    }))
    .collect();
    let icones = m::FIELD_H * segs.len() as f32;
    c.bare(Rect::new(direita - icones, topo, icones, m::FIELD_H), &segs);
    let fim_das_fichas = direita - icones - m::FIELD_GAP;
    let larguras: Vec<f32> = l.fichas.iter().map(|f| ficha_w(c, f)).collect();
    let soma: f32 =
        larguras.iter().sum::<f32>() + m::FIELD_GAP * larguras.len().saturating_sub(1) as f32;
    let nome = l.acao.name();
    let nome_w = c.text_system.prefix_width(nome, m::VALUE_PX).ceil();
    let cabe_ao_lado = soma <= fim_das_fichas - c.x - nome_w - m::FIELD_GAP;
    let nome_ate = if cabe_ao_lado {
        fim_das_fichas - soma - m::FIELD_GAP
    } else {
        fim_das_fichas
    };
    let cor = c.ink.color(ColorToken::Text1);
    paint_text_elided(
        c.text_system,
        c.scene,
        nome,
        c.x,
        topo + folga_do_texto(m::FIELD_H, m::VALUE_PX),
        m::VALUE_PX,
        (nome_ate - c.x).max(0.0),
        cor,
    );
    if l.fichas.is_empty() {
        return topo + m::FIELD_H;
    }
    // As fichas por fileiras, da direita: a 1.ª ao lado do nome quando todas cabem; senão por baixo.
    let mut fileiras: Vec<Vec<usize>> = Vec::new();
    if cabe_ao_lado {
        fileiras.push((0..l.fichas.len()).collect());
    } else {
        fileiras.push(Vec::new());
        let mut usada = 0.0;
        let teto = c.w;
        let mut atual: Vec<usize> = Vec::new();
        for (i, w) in larguras.iter().enumerate() {
            let pede = if atual.is_empty() {
                *w
            } else {
                usada + m::FIELD_GAP + w
            };
            if !atual.is_empty() && pede > teto {
                fileiras.push(std::mem::take(&mut atual));
                usada = *w;
            } else {
                usada = pede;
            }
            atual.push(i);
        }
        fileiras.push(atual);
    }
    let mut fy = topo;
    for (n, fila) in fileiras.iter().enumerate() {
        if n > 0 {
            fy += m::FIELD_H + m::FIELD_ROW_GAP;
        }
        let fim = if n == 0 { fim_das_fichas } else { direita };
        let w: f32 = fila.iter().map(|&i| larguras[i]).sum::<f32>()
            + m::FIELD_GAP * fila.len().saturating_sub(1) as f32;
        let mut x = fim - w;
        for &i in fila {
            pinta_ficha(c, &l.fichas[i], x, fy);
            x += larguras[i] + m::FIELD_GAP;
        }
    }
    fy + m::FIELD_H
}

/// ⭐ A largura de fábrica da janela: o nome mais largo ao lado da 1.ª ficha dele, com `+` e `↺`
/// (dono 08/11: nome nenhum com «…»). Mede-se à letra do cartão; a alça continua a mandar depois.
pub(crate) fn largura_de_fabrica(ts: &mut ph2d_text::TextSystem) -> f32 {
    let k = ph2d_pixel_keys::Keymap::factory();
    let corpo = PixelAction::ALL
        .iter()
        .map(|&a| {
            let nome = ts.prefix_width(a.name(), m::VALUE_PX).ceil();
            let ficha = k.keys_of(a).first().map_or(0.0, |(c, _)| {
                ts.prefix_width(&c.to_string(), m::VALUE_PX).ceil()
                    + m::FIELD_PAD_R * 2.0
                    + m::FIELD_GAP
            });
            nome + m::FIELD_GAP + ficha + m::FIELD_H * 2.0
        })
        .fold(0.0, f32::max);
    // As margens da lista e do cartão, dos dois lados, e a calha da barra de rolar.
    corpo + (m::LIST_PAD_X + m::CARD_PAD_X) * 2.0 + m::FIELD_H
}
