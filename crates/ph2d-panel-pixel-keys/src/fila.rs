//! Uma acção = UMA fileira, como o diálogo do Aseprite e a lei do cartão compacto (Figma):
//! `Nome ……… [Ctrl+E] [↺]`. Cada tecla é um CAMPO (dono 08/11, §9.8): o clique limpa-o e o próximo
//! acorde entra no key-down, sem `+` nem `Confirm`. O campo vazio «—» aparece quando a acção não tem
//! tecla; clicar no NOME abre-o em captura (a 2.ª tecla). Os campos encostam à direita; os que não
//! cabem ao lado do nome descem. O `↺` só aparece fora da fábrica. Uma tecla tomada pergunta NA
//! fileira: «Shift+B is used by Spray. Replace? [Replace] [Cancel]».

use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{folga_do_texto, paint_text_elided, stroke_rounded_rect};
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_pixel_keys::PixelAction;
use ph2d_tokens::{ColorToken, StrokeToken};

use crate::ids;

/// Um campo de tecla: o id (o clique captura), o texto e se está em captura (o anel de foco).
pub(crate) struct Campo {
    pub id: NodeId,
    pub texto: String,
    pub aceso: bool,
}

/// Uma linha da lista: a acção, os campos, se difere da fábrica, a recusa e a pergunta do conflito.
pub(crate) struct Linha {
    pub acao: PixelAction,
    pub campos: Vec<Campo>,
    pub mudada: bool,
    pub recusa: Option<String>,
    pub pergunta: Option<String>,
}

fn pinta_campo(c: &mut CardCtx<'_>, f: &Campo, x: f32, y: f32) {
    let r = Rect::new(x, y, c.botao_w(&f.texto), m::FIELD_H);
    c.botao_em(f.id, &f.texto, r);
    crate::canal::pintou(f.id, &f.texto);
    if f.aceso {
        // FRAME-RAW-OK: o campo à espera da tecla, contornado no acento (o foco)
        stroke_rounded_rect(
            c.scene,
            r,
            m::FIELD_RADIUS,
            StrokeToken::Default.px(),
            c.ink.color(ColorToken::Accent),
        );
    }
}

/// Pinta a fileira de `l` a partir de `y` (o fundo da anterior) e devolve o fundo dela.
pub(crate) fn pinta(c: &mut CardCtx<'_>, l: &Linha, y: f32) -> f32 {
    let topo = y + m::FIELD_ROW_GAP;
    let direita = c.x + c.w;
    let reset = [Seg {
        id: ids::reset(l.acao),
        icon: Some(IconId::Reset),
        text: "",
        on: false,
    }];
    let icones = if l.mudada { m::FIELD_H } else { 0.0 };
    if l.mudada {
        c.bare(
            Rect::new(direita - icones, topo, icones, m::FIELD_H),
            &reset,
        );
    }
    let fim_dos_campos = direita - icones - if l.mudada { m::FIELD_GAP } else { 0.0 };
    let larguras: Vec<f32> = l.campos.iter().map(|f| c.botao_w(&f.texto)).collect();
    let soma: f32 =
        larguras.iter().sum::<f32>() + m::FIELD_GAP * larguras.len().saturating_sub(1) as f32;
    let nome = l.acao.name();
    let nome_w = c.text_system.prefix_width(nome, m::VALUE_PX).ceil();
    let cabe_ao_lado = soma <= fim_dos_campos - c.x - nome_w - m::FIELD_GAP;
    let nome_ate = if cabe_ao_lado {
        fim_dos_campos - soma - m::FIELD_GAP
    } else {
        fim_dos_campos
    };
    // O nome também é alvo: o clique abre o campo vazio em captura (mais uma tecla).
    c.hit_index.register(
        ids::nome(l.acao),
        Rect::new(c.x, topo, (nome_ate - c.x).max(0.0), m::FIELD_H),
    );
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
    let mut fileiras: Vec<Vec<usize>> = Vec::new();
    if cabe_ao_lado {
        fileiras.push((0..l.campos.len()).collect());
    } else {
        fileiras.push(Vec::new());
        let mut usada = 0.0;
        let mut atual: Vec<usize> = Vec::new();
        for (i, w) in larguras.iter().enumerate() {
            let pede = if atual.is_empty() {
                *w
            } else {
                usada + m::FIELD_GAP + w
            };
            if !atual.is_empty() && pede > c.w {
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
        let fim = if n == 0 { fim_dos_campos } else { direita };
        let w: f32 = fila.iter().map(|&i| larguras[i]).sum::<f32>()
            + m::FIELD_GAP * fila.len().saturating_sub(1) as f32;
        let mut x = (fim - w).max(c.x);
        for &i in fila {
            pinta_campo(c, &l.campos[i], x, fy);
            x += larguras[i] + m::FIELD_GAP;
        }
    }
    let mut y = fy + m::FIELD_H;
    if let Some(r) = &l.recusa {
        // A régua lê a recusa pelo nome da fileira.
        crate::canal::pintou(ids::nome(l.acao), r);
        y = c.hint(r, y);
    }
    if let Some(p) = &l.pergunta {
        y = c.hint(p, y);
        let botoes = [
            Cell::new(Widget::Button {
                id: ids::REPLACE,
                text: tr("panel.pixel_keys.replace"),
                primary: true,
            }),
            Cell::new(Widget::Button {
                id: ids::KEYS_CANCEL,
                text: tr("panel.pixel_keys.cancel"),
                primary: false,
            }),
        ];
        y = c.row(&botoes, y);
    }
    y
}

/// ⭐ A largura de fábrica da janela: o nome mais largo ao lado da 1.ª tecla dele, o campo vazio e `↺`
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
