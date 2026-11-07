//! ⭐ **A grelha triangular de ligar/desligar** (o mapa de colisão da Unity e da Godot): só as linhas
//! pedidas, e as colunas que não cabem rolam dentro do cartão.

use super::super::{CardCtx, Cell, Widget, rotulos};
use crate::interaction::toggle_paint;
use crate::paint::{fill_rounded_rect, paint_text, paint_text_elided};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{ColorToken, StrokeToken};

/// A calha dos rótulos da grelha: até este múltiplo da célula; o nome maior corta-se com reticências.
const CALHA_MAX_CELULAS: f32 = 4.0;

/// A grelha triangular: a linha `i` tem `i + 1` células; `(i, j)` é UM facto sobre o par.
pub struct ToggleTriangle<'a> {
    /// O nome por cima, com a unidade se tiver.
    pub label: &'a str,
    /// As linhas (e colunas) mostradas, cada uma com a sua identidade: `id_of` e `on_of` recebem-na.
    pub linhas: &'a [usize],
    /// O rótulo de uma linha (à esquerda); a coluna leva a identidade em número.
    pub rotulo_de: &'a dyn Fn(usize) -> String,
    /// A barra que rola as colunas quando não cabem (um `Slider` registado) e o painel dela.
    pub rola: Option<(NodeId, NodeId)>,
    /// A linha em destaque, pela identidade.
    pub destaque: Option<usize>,
    /// O grupo do arrasto que pinta (`toggle_paint`): cada célula declara o que mostra.
    pub group: &'static str,
    /// A marca de animação da grelha inteira (um controlo, uma marca).
    pub key: Option<&'a [NodeId]>,
    pub id_of: &'a dyn Fn(usize, usize) -> NodeId,
    pub on_of: &'a dyn Fn(usize, usize) -> bool,
}

thread_local! {
    static LINHA_EM_DESTAQUE: std::cell::Cell<Option<Rect>> = const { std::cell::Cell::new(None) };
}

/// A caixa da linha em destaque no último quadro, para o painel a trazer à vista.
#[must_use]
pub fn linha_em_destaque() -> Option<Rect> {
    LINHA_EM_DESTAQUE.get()
}

impl CardCtx<'_> {
    /// A grelha triangular de ligar/desligar: legendas em cima e à esquerda; aceso = o acento, a
    /// diagonal (o par consigo próprio) leva contorno para o olho achar o eixo.
    ///
    /// A célula é sempre [`m::GRID_CELL`] (a medida do toque). As colunas que não cabem rolam dentro
    /// do cartão, de coluna em coluna, pela barra acima da grelha: os rótulos à esquerda ficam, e a
    /// linha que já não tem célula à vista sai. A barra fica ACIMA para não fugir à mão que a arrasta.
    pub fn toggle_triangle(&mut self, t: &ToggleTriangle<'_>, y: f32) -> f32 {
        self.first = false;
        let mut y = y + m::LABEL_TOP;
        let mut titulo = Cell::new(Widget::Readout { text: "" }).label(t.label);
        titulo.key = t.key;
        y += rotulos::faixa(self, &[titulo], &[(self.x, self.w)], y);
        toggle_paint::limpa_grupo(t.group);
        LINHA_EM_DESTAQUE.set(None);
        let topo = y;
        let n = t.linhas.len();
        let lado = m::GRID_CELL;
        let passo = lado + m::GRID_GAP;
        let rotulos: Vec<String> = t.linhas.iter().map(|&l| (t.rotulo_de)(l)).collect();
        let medida = rotulos
            .iter()
            .map(|r| self.text_system.prefix_width(r, m::LABEL_PX).ceil())
            .fold(lado, f32::max);
        let calha = medida.min(lado * CALHA_MAX_CELULAS);
        let cabem = (((self.w - calha) / passo).floor() as usize).max(1);
        let mut primeira = 0;
        if n > cabem {
            let v = match t.rola.and_then(|(id, _)| self.store.get(id)) {
                Some(crate::interaction::InteractiveState::Slider { value, .. }) => *value,
                _ => 0.0,
            };
            let escondidas = n - cabem;
            primeira = ((v * escondidas as f32).round() as usize).min(escondidas);
            if let Some((id, _)) = t.rola {
                let trilha = Rect::new(
                    self.x + calha + m::GRID_GAP,
                    y,
                    self.w - calha - m::GRID_GAP,
                    m::LABEL_PX,
                );
                self.hit_index.register(id, trilha);
                let fio = crate::paint::folga_do_texto(m::LABEL_PX, m::GRID_GAP);
                let barra = Rect::new(trilha.x, y + fio, trilha.w, m::GRID_GAP);
                fill_rounded_rect(
                    self.scene,
                    barra,
                    m::GRID_GAP * 0.5,
                    self.ink.color(ColorToken::CardField),
                );
                let parte = cabem as f32 / n as f32;
                let polegar = Rect::new(
                    trilha.x + trilha.w * (1.0 - parte) * primeira as f32 / escondidas as f32,
                    y + fio,
                    trilha.w * parte,
                    m::GRID_GAP,
                );
                fill_rounded_rect(
                    self.scene,
                    polegar,
                    m::GRID_GAP * 0.5,
                    self.ink.color(ColorToken::CardLabel),
                );
                y += m::LABEL_PX + m::LABEL_BOTTOM;
            }
        }
        let fim_das_colunas = (primeira + cabem).min(n);
        let coluna_x = |x: f32, j: usize| x + calha + m::GRID_GAP + passo * (j - primeira) as f32;
        let legenda = self.ink.color(ColorToken::CardLabel);
        for j in primeira..fim_das_colunas {
            self.legenda(&t.linhas[j].to_string(), coluna_x(self.x, j), y, lado);
        }
        y += m::LABEL_PX + m::LABEL_BOTTOM;
        let mut pares = Vec::new();
        for (i, rotulo) in rotulos.iter().enumerate().skip(primeira) {
            let destacada = t.destaque == Some(t.linhas[i]);
            if destacada {
                let faixa = Rect::new(self.x, y, self.w, lado);
                fill_rounded_rect(
                    self.scene,
                    faixa,
                    m::SEG_RADIUS,
                    self.ink.color(ColorToken::AccentSoft),
                );
                LINHA_EM_DESTAQUE.set(Some(faixa));
            }
            paint_text_elided(
                self.text_system,
                self.scene,
                rotulo,
                self.x,
                y + crate::paint::folga_do_texto(lado, m::LABEL_PX),
                m::LABEL_PX,
                calha,
                if destacada {
                    self.ink.color(ColorToken::KanbanCardFg)
                } else {
                    legenda
                },
            );
            for j in primeira..=i.min(fim_das_colunas - 1) {
                let (a, b) = (t.linhas[i], t.linhas[j]);
                let id = (t.id_of)(a, b);
                let on = (t.on_of)(a, b);
                let cell = Rect::new(coluna_x(self.x, j), y, lado, lado);
                let fundo = if on {
                    ColorToken::Accent
                } else if self.store.hover_live(id) > 0.0 {
                    ColorToken::CardFieldHover
                } else {
                    ColorToken::CardField
                };
                fill_rounded_rect(self.scene, cell, m::SEG_RADIUS, self.ink.color(fundo));
                if a == b {
                    // FRAME-RAW-OK: a diagonal da grelha, contornada para o olho achar o eixo de simetria
                    crate::paint::stroke_rounded_rect(
                        self.scene,
                        cell,
                        m::SEG_RADIUS,
                        StrokeToken::Default.px(),
                        legenda,
                    );
                }
                self.hit_index.register(id, cell);
                toggle_paint::celula(id, t.group, on, cell);
                pares.push(id);
            }
            y += passo;
        }
        crate::widget::composto::grupo(pares);
        if let Some((barra, painel)) = t.rola {
            if n > cabem {
                crate::interaction::roda_lateral::publica(
                    crate::interaction::roda_lateral::Regiao {
                        barra,
                        painel,
                        rect: Rect::new(self.x, topo, self.w, y - topo),
                        curso_px: passo * (n - cabem) as f32,
                    },
                );
            } else {
                crate::interaction::roda_lateral::limpa(barra);
            }
        }
        y - m::GRID_GAP
    }

    fn legenda(&mut self, texto: &str, x: f32, y: f32, lado: f32) {
        let tw = self.text_system.prefix_width(texto, m::LABEL_PX);
        paint_text(
            self.text_system,
            self.scene,
            texto,
            x + (lado - tw) * 0.5,
            y,
            m::LABEL_PX,
            lado,
            self.ink.color(ColorToken::CardLabel),
        );
    }
}
