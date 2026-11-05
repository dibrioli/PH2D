//! ⭐ **A lei do ritmo vertical** ([05 §4](../../../../docs/CartoesUI/05_ESPECIFICACAO_UI_COMPLETA.md)):
//! a distância real entre o que o corpo de um cartão pintou, texto contra controlo. O censo das linhas
//! só vê o que passa pelas portas dos cartões; este vê toda a pintura (`paint_text*` e `HitIndex::register`).

use super::{GRAVACAO, secao_da_linha};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Especie {
    /// Uma linha no corpo de letra do rótulo (`LABEL_PX`).
    Rotulo,
    /// Todo outro texto: frase de ajuda, aviso, título de grupo, valor só de leitura.
    Frase,
    Controlo,
}

/// Uma peça pintada no corpo de um cartão.
#[derive(Clone, Debug, PartialEq)]
pub struct Peca {
    pub secao: NodeId,
    pub especie: Especie,
    pub rect: Rect,
    /// O corpo de letra de um texto; `0` num controlo.
    pub px: f32,
    pub nome: String,
}

fn anota(especie: Especie, rect: Rect, px: f32, nome: impl FnOnce() -> String) {
    let Some(secao) = secao_da_linha() else {
        return;
    };
    GRAVACAO.with(|g| {
        if let Some(c) = g.borrow_mut().as_mut() {
            c.pecas.push(Peca {
                secao,
                especie,
                rect,
                px,
                nome: nome(),
            });
        }
    });
}

/// A entrelinha sobre o corpo de letra (a altura de uma linha do pintor é `1 + isto`): sai do fundo do
/// texto, para a distância ser até às letras e não até à caixa da linha.
const ENTRELINHA: f32 = 0.36; // LITERAL-PX-OK: medida do pintor de texto, não desenho

pub(crate) fn pintou_texto(texto: &str, [x, y, w, alta]: [f32; 4], px: f32) {
    let uma_linha = alta < px * (1.0 + ENTRELINHA) * 1.5; // LITERAL-PX-OK: meia linha de folga
    let especie = if uma_linha && (px - m::LABEL_PX).abs() < 0.5 {
        Especie::Rotulo
    } else {
        Especie::Frase
    };
    let h = (alta - px * ENTRELINHA).max(px);
    anota(especie, Rect::new(x, y, w, h), px, || texto.to_owned());
}

pub(crate) fn pintou_controlo(id: NodeId, rect: Rect) {
    if super::ligado() {
        anota(Especie::Controlo, rect, 0.0, || format!("{id:?}"));
    }
}

/// As fronteiras que a lei mede (05 §4, «Ritmo vertical»).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fronteira {
    RotuloControlo,
    AteAoRotulo,
    ControloFrase,
    FraseControlo,
    FraseFrase,
    ControloControlo,
    /// Dois controlos lado a lado com topo ou altura diferentes.
    Desalinhado,
}

impl Fronteira {
    /// O vão mínimo e máximo da fronteira. A folga de arredondamento é [`FOLGA`].
    #[must_use]
    pub const fn faixa(self) -> (f32, f32) {
        match self {
            // A pílula do segmentado regista os segmentos `SEG_PAD` para dentro.
            Self::RotuloControlo => (m::LABEL_BOTTOM, m::LABEL_BOTTOM + m::SEG_PAD),
            // A porta dá `LABEL_TOP` (8); `FIELD_ROW_GAP` é o chão, porque uma frase curta no corpo do
            // rótulo é medida como rótulo e a porta do aviso dá-lhe 6.
            Self::AteAoRotulo | Self::ControloFrase | Self::FraseControlo | Self::FraseFrase => {
                (m::FIELD_ROW_GAP, f32::INFINITY)
            }
            Self::ControloControlo => (0.0, f32::INFINITY),
            Self::Desalinhado => (0.0, 0.0),
        }
    }
}

/// O arredondamento que a pintura pode introduzir.
pub const FOLGA: f32 = 1.0;

/// Uma distância medida.
#[derive(Clone, Debug, PartialEq)]
pub struct Distancia {
    pub secao: NodeId,
    pub fronteira: Fronteira,
    pub vao: f32,
    pub de: String,
    pub para: String,
}

impl Distancia {
    /// Fora da faixa da fronteira.
    #[must_use]
    pub fn fora_da_lei(&self) -> bool {
        let (min, max) = self.fronteira.faixa();
        self.vao < min - FOLGA || self.vao > max + FOLGA
    }
}

/// O controlo mais baixo que conta: abaixo disto é a marca de animação, ou a dica de um rótulo.
const CONTROLO_MIN_H: f32 = m::GRID_CELL - 2.0;
/// O mais alto: acima disto é o fundo de um bloco, não um controlo.
const CONTROLO_MAX_H: f32 = m::FIELD_H * 2.0;

fn cruza_em_x(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w
}

fn dentro(t: &Rect, c: &Rect) -> bool {
    let (cx, cy) = (t.x + t.w * 0.5, t.y + t.h * 0.5);
    cx >= c.x && cx <= c.x + c.w && cy >= c.y && cy <= c.y + c.h
}

/// As peças da secção que contam: os controlos de altura de controlo, e os textos fora de todos eles
/// (a legenda de um botão ou o valor de um campo não é uma linha do cartão).
fn da_secao(pecas: &[Peca], secao: NodeId) -> (Vec<&Peca>, Vec<&Peca>) {
    let controlos: Vec<&Peca> = pecas
        .iter()
        .filter(|p| p.secao == secao && p.especie == Especie::Controlo)
        .filter(|p| p.rect.h >= CONTROLO_MIN_H && p.rect.h <= CONTROLO_MAX_H)
        .collect();
    let textos: Vec<&Peca> = pecas
        .iter()
        .filter(|p| p.secao == secao && p.especie != Especie::Controlo)
        .filter(|t| !controlos.iter().any(|c| dentro(&t.rect, &c.rect)))
        .collect();
    (controlos, textos)
}

/// A peça imediatamente abaixo de `de` que cruza em `x`.
fn abaixo<'a>(de: &Rect, todas: &[&'a Peca]) -> Option<(&'a Peca, f32)> {
    let fundo = de.y + de.h;
    todas
        .iter()
        .filter(|p| cruza_em_x(de, &p.rect) && p.rect.y >= de.y + de.h * 0.5)
        .map(|p| (*p, p.rect.y - fundo))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

/// Há um controlo à direita de `p`, na mesma fila.
fn tem_controlo_ao_lado(p: &Peca, controlos: &[&Peca]) -> bool {
    let meio = p.rect.y + p.rect.h * 0.5;
    let fim = p.rect.x + p.rect.w;
    controlos
        .iter()
        .any(|c| c.rect.y <= meio && meio <= c.rect.y + c.rect.h && c.rect.x >= fim - FOLGA)
}

impl super::Censo {
    /// Toda fronteira vertical dos corpos dos cartões, de cima para baixo.
    #[must_use]
    pub fn distancias(&self) -> Vec<Distancia> {
        let mut secoes: Vec<NodeId> = self.pecas.iter().map(|p| p.secao).collect();
        secoes.sort_by_key(|s| s.0);
        secoes.dedup();
        let mut out = Vec::new();
        for s in secoes {
            let (controlos, textos) = da_secao(&self.pecas, s);
            let todas: Vec<&Peca> = controlos.iter().chain(&textos).copied().collect();
            desalinhados(&controlos, s, &mut out);
            for p in &todas {
                let Some((q, vao)) = abaixo(&p.rect, &todas) else {
                    continue;
                };
                let fronteira = match (p.especie, q.especie) {
                    // Longe demais para ser o controlo dele: o texto de uma linha de lista.
                    (Especie::Rotulo, Especie::Controlo) if vao > m::FIELD_H => continue,
                    // O rótulo AO LADO (a barra de uma banda, doc 14): o controlo dele não está abaixo.
                    (Especie::Rotulo, Especie::Controlo) if tem_controlo_ao_lado(p, &controlos) => {
                        continue;
                    }
                    (Especie::Rotulo, Especie::Controlo) => Fronteira::RotuloControlo,
                    // O rótulo que quebra em duas linhas (`cards_rotulos`).
                    (Especie::Rotulo, Especie::Rotulo) => continue,
                    (_, Especie::Rotulo) => Fronteira::AteAoRotulo,
                    (Especie::Controlo, Especie::Frase) => Fronteira::ControloFrase,
                    (Especie::Frase, Especie::Controlo) => Fronteira::FraseControlo,
                    (Especie::Frase, Especie::Frase) => Fronteira::FraseFrase,
                    (Especie::Controlo, Especie::Controlo) => Fronteira::ControloControlo,
                    (Especie::Rotulo, Especie::Frase) => continue,
                };
                out.push(Distancia {
                    secao: s,
                    fronteira,
                    vao,
                    de: p.nome.clone(),
                    para: q.nome.clone(),
                });
            }
        }
        out
    }

    /// A lei do ritmo: as fronteiras fora da faixa e os controlos desalinhados, em frases.
    #[must_use]
    pub fn ritmo_fora_da_lei(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .distancias()
            .iter()
            .filter(|d| d.fora_da_lei())
            .map(|d| {
                format!(
                    "{:?} {:.1}: {} > {}",
                    d.fronteira,
                    d.vao,
                    curto(&d.de),
                    curto(&d.para)
                )
            })
            .collect();
        out.sort();
        out.dedup();
        out
    }
}

/// A frase cabe numa linha da tabela.
fn curto(s: &str) -> String {
    s.chars().take(48).collect()
}

/// Dois controlos da mesma fila: um cruza o outro na vertical e nenhum contém o outro.
fn desalinhados(controlos: &[&Peca], s: NodeId, out: &mut Vec<Distancia>) {
    for (i, a) in controlos.iter().enumerate() {
        for b in &controlos[i + 1..] {
            let (ra, rb) = (&a.rect, &b.rect);
            let lado_a_lado = ra.x + ra.w <= rb.x + FOLGA || rb.x + rb.w <= ra.x + FOLGA;
            let cruza = ra.y < rb.y + rb.h - FOLGA && rb.y < ra.y + ra.h - FOLGA;
            // O segmentado regista os segmentos `SEG_PAD` para dentro da pílula, que alinha.
            let dy = (ra.y - rb.y).abs().max((ra.h - rb.h).abs() * 0.5);
            if lado_a_lado && cruza && dy > m::SEG_PAD + FOLGA {
                out.push(Distancia {
                    secao: s,
                    fronteira: Fronteira::Desalinhado,
                    vao: dy,
                    de: a.nome.clone(),
                    para: b.nome.clone(),
                });
            }
        }
    }
}
