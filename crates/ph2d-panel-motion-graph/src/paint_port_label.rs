//! **O NOME DA PORTA, escrito na linha que já existia** (report do Enio, 2026-08-27).
//!
//! ## O defeito
//!
//! *"Vários nós como o próprio boids têm uma série de inputs sem nenhum nome, sem identificação.
//! Como o usuário entender se em nenhum lugar diz o que é?"*
//!
//! Medido no dia: **não havia lugar nenhum.** Nem no cartão, nem em hover, nem no inspector do nó
//! selecionado — e nem na camada de acessibilidade, cujo hit de socket carrega `node` + índice de
//! `port` e nunca o nome. As três pistas que o artista tinha diziam todas o **tipo** e nunca a
//! **identidade**: a posição na lista, o glifo (círculo = um escalar · losango = uma coluna) e a
//! cor (o domínio). Duas portas do mesmo tipo colidem nas três — e é exactamente o caso da foto:
//! o `motion.boids` mostra `target_x` e `target_y` como **dois círculos idênticos**.
//!
//! **58 dos 119 nós têm 2+ entradas** (o pior tem 5), então metade do catálogo herdava a colisão.
//!
//! ## Por que isto não é um redesenho
//!
//! Duas peças já existiam e eram deitadas fora:
//!
//! - o nome **já viaja** até ao painel (`PortView::name`), e o desenhista simplesmente não o
//!   escrevia — o único consumidor era o `declared_param` do construtor do snapshot;
//! - o cartão **já reserva uma linha de `ROW_H` por socket** (`geom::card_rows`), e essas linhas
//!   estavam **vazias**. É por isso que um `motion.boids` desenha alto e em branco por dentro.
//!
//! ⇒ o rótulo entra numa faixa que já estava paga. Nenhuma geometria muda, nenhum nó muda.
//!
//! ## As duas leis
//!
//! 1. **ENTRADAS sempre; SAÍDAS só quando há mais de uma.** Medido **no registry** (2026-08-27):
//!    **127 nós têm exactamente uma saída** e só **três** têm mais — `value.cursor` (`x`/`y`),
//!    `pulse.counter` (`out`/`carry`) e `sim.lifetime` (`out`/`died`/`pulse`). ⚠️ A 1.ª redacção
//!    desta linha dizia `294`, tirado de um **regex sobre o fonte** que contava manifestos
//!    repetidos; a forma estava certa e o número não. *Uma contagem de catálogo pergunta-se ao
//!    registry, nunca ao texto.* Um rótulo
//!    responde *"qual delas?"*, e com uma saída não há pergunta — escrever «Out» em 294 cartões
//!    seria ruído a competir com o nome do nó.
//! 2. **O rótulo é DERIVADO do nome do manifesto**, não uma segunda tabela. `target_x` → `Target
//!    X`, `in0` → `In 0`, `forces` → `Forces`. ⛔ Uma tabela à mão por nó seria a **segunda
//!    resposta** à pergunta *"como se chama esta porta?"* — e a que envelhece é sempre a de fora
//!    (o repo já pagou isso com a lista de extensões ao lado do predicado de import). Quando um
//!    nome derivar mal, a cura é **renomear a porta no manifesto**, que é append-only e barato,
//!    ou acrescentar um `PortUiHint` no registry como side-metadata — nunca um mapa no painel.

use crate::geom::{self, View};
use crate::snapshot::GraphNodeView;
use ph2d_editor_core::paint::{paint_text_title, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::text_elide::paint_text_title_elided;
use ph2d_tokens::{ColorToken, Theme};

use super::socket_center;

/// Tamanho do rótulo — abaixo do título (13) por hierarquia: o nome do nó manda, a porta informa.
const PORT_LABEL_SIZE: f32 = 11.0; // LITERAL-PX-OK: port label font size
/// Distância do rótulo à borda do cartão (o socket é centrado NA borda, raio 5).
const PORT_LABEL_PAD_X: f32 = 11.0; // LITERAL-PX-OK: port label inset from the card edge
/// Recuo do topo da linha — o mesmo raciocínio do título: centra o texto na faixa de `ROW_H`.
const PORT_LABEL_PAD_Y: f32 = 5.0; // LITERAL-PX-OK: port label top inset within its row
/// **Abaixo deste zoom o rótulo não é desenhado.** Não é conforto: a `PORT_LABEL_SIZE * zoom`
/// cai abaixo de ~6 px, que não se lê, e cada glifo continua a custar trabalho de texto num
/// grafo inteiro fora de escala. *Um texto ilegível é custo sem informação.*
const PORT_LABEL_MIN_ZOOM: f32 = 0.55; // LITERAL-PX-OK: zoom threshold, not a design size
/// Avanço médio de um caractere a [`PORT_LABEL_SIZE`]; o precedente é o `CRUMB_CHAR_W` do
/// breadcrumb, que resolve o mesmo problema do mesmo modo. Serve **só** para alinhar à DIREITA
/// o rótulo de uma saída.
///
/// ⛔⛔ **A nota que aqui estava — *"não há API de medição neste caminho"* — deixou de ser
/// verdade em 2026-08-30**, quando nasceu a `TextSystem::prefix_width_weighted` (§0.0: *quem
/// move o número que tornava algo inalcançável tem de reconferir a nota*). O alinhamento à
/// direita passar a usá-la é trabalho em aberto, e o preço é uma medição por rótulo por quadro.
///
/// ⚠️ **E a 2.ª metade da nota antiga (*"a truncagem verdadeira é do `paint_text_title`"*)
/// também morreu:** aquele ramo passa `f32::INFINITY` desde 2026-08-30 — usar esta ESTIMATIVA
/// como limite de corte apagava `Out` inteiro (ver a nota no sítio da chamada).
const LABEL_CHAR_W: f32 = 5.6; // LITERAL-PX-OK: mean advance at PORT_LABEL_SIZE

/// Quantos bytes um rótulo pode ter. Os nomes de porta do catálogo são `[a-z0-9_]` e o mais
/// comprido tem 12 caracteres (`particle_radius` é param, não porta); `24` dá folga e mantém o
/// buffer na pilha — **um rótulo por socket por quadro não pode alocar**.
const PORT_LABEL_CAP: usize = 24;

/// O rótulo humano de uma porta, derivado do nome do manifesto **sem alocar**.
pub struct PortLabel {
    buf: [u8; PORT_LABEL_CAP],
    len: usize,
}

impl PortLabel {
    /// `target_x` → `Target X` · `in0` → `In 0` · `state` → `State`.
    ///
    /// ⚠️ **ASCII de propósito**: os nomes de porta do catálogo são `[a-z0-9_]` por convenção
    /// (há gate de manifesto sobre isso), então a transformação é byte a byte e não precisa de
    /// saber o que é um grafema. Um nome fora dessa classe passa **verbatim** — a régua nunca
    /// corrompe o que não entende.
    #[must_use]
    pub fn of(name: &str) -> Self {
        let mut buf = [0u8; PORT_LABEL_CAP];
        let mut len = 0usize;
        let mut start_of_word = true;
        let mut prev_alpha = false;
        let push = |b: u8, len: &mut usize, buf: &mut [u8; PORT_LABEL_CAP]| {
            if *len < PORT_LABEL_CAP {
                buf[*len] = b;
                *len += 1;
            }
        };
        for b in name.bytes() {
            match b {
                b'_' => {
                    if len > 0 {
                        push(b' ', &mut len, &mut buf);
                    }
                    start_of_word = true;
                    prev_alpha = false;
                }
                b'0'..=b'9' => {
                    // Um dígito colado a uma letra abre palavra: `in0` lê-se `In 0`, não `In0`.
                    if prev_alpha {
                        push(b' ', &mut len, &mut buf);
                    }
                    push(b, &mut len, &mut buf);
                    start_of_word = false;
                    prev_alpha = false;
                }
                b'a'..=b'z' => {
                    push(
                        if start_of_word {
                            b.to_ascii_uppercase()
                        } else {
                            b
                        },
                        &mut len,
                        &mut buf,
                    );
                    start_of_word = false;
                    prev_alpha = true;
                }
                other => {
                    push(other, &mut len, &mut buf);
                    start_of_word = false;
                    prev_alpha = other.is_ascii_alphabetic();
                }
            }
        }
        Self { buf, len }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        // Só bytes ASCII entram no buffer (ou bytes que já vinham de um `&str` válido, copiados
        // sem os partir — a classe `other` copia um byte de cada vez e um nome do catálogo é
        // ASCII por convenção). O recuo é o nome vazio, nunca um panic num caminho de desenho.
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

/// **A largura que sobra para um rótulo de ENTRADA**, dado quantas saídas o nó tem.
///
/// ⚠️ **Exposta porque o censo do catálogo precisa dela**, e a alternativa era o gate recalcular
/// `CARD_W`/`PAD` do seu lado — a segunda cópia da mesma conta, que é o defeito que este ficheiro
/// existe para não repetir. A lei mora aqui; quem mede lê-a.
#[must_use]
pub fn input_label_budget_px(outputs: usize) -> f32 {
    if outputs > 1 {
        geom::CARD_W * 0.5 - PORT_LABEL_PAD_X
    } else {
        geom::CARD_W - 2.0 * PORT_LABEL_PAD_X
    }
}

impl PortLabel {
    /// A largura aproximada deste rótulo — pelo mesmo avanço médio que alinha as saídas.
    /// ⚠️ Aproximada **de propósito**: não há API de medição neste caminho (ver [`LABEL_CHAR_W`]).
    #[must_use]
    pub fn approx_width_px(&self) -> f32 {
        #[expect(clippy::cast_precision_loss, reason = "um rotulo tem < 24 bytes")]
        let n = self.len as f32;
        n * LABEL_CHAR_W
    }
}

/// **Escreve o nome de cada porta na faixa que o cartão já reservou para ela.**
pub(super) fn draw_port_labels(ctx: &mut PaintCtx, n: &GraphNodeView, view: &View, theme: Theme) {
    if view.zoom < PORT_LABEL_MIN_ZOOM {
        return;
    }
    // ⚠️ Só o X do cartão: o Y de cada rótulo sai do `socket_center` da própria porta, que já
    // é de ecrã — derivá-lo aqui outra vez seria a segunda conta da mesma posição.
    let (sx, _) = view.pt(n.x, n.y);
    let w = geom::CARD_W * view.zoom;
    let pad = PORT_LABEL_PAD_X * view.zoom;
    let size = PORT_LABEL_SIZE * view.zoom;
    let colour = resolve(ColorToken::Text2, theme);
    // ⚠️ **`Text2`, o tom apagado — a mesma escolha do readout, e pela mesma razão:** o rótulo
    // não é o nome que o artista deu ao nó, é o que a porta É. Em `Text1` ele competiria com o
    // título dentro do mesmo cartão.

    // Uma SAÍDA rotulada reserva metade da largura; sem ela a entrada fica com o cartão todo.
    let labelled_outputs = n.outputs.len() > 1;
    let in_max_w = input_label_budget_px(n.outputs.len()) * view.zoom;

    for (i, p) in n.inputs.iter().enumerate() {
        let (_, cy) = socket_center(n, view, false, i);
        let label = PortLabel::of(p.name);
        paint_text_title_elided(
            ctx.text_system,
            ctx.scene,
            label.as_str(),
            sx + pad,
            cy - (geom::ROW_H * 0.5 - PORT_LABEL_PAD_Y) * view.zoom,
            size,
            in_max_w,
            colour,
        );
    }

    if !labelled_outputs {
        return;
    }
    for (i, p) in n.outputs.iter().enumerate() {
        let (_, cy) = socket_center(n, view, true, i);
        let label = PortLabel::of(p.name);
        // Alinhada à DIREITA por avanço médio — ver [`LABEL_CHAR_W`].
        let text_w = label.approx_width_px() * view.zoom;
        paint_text_title(
            ctx.text_system,
            ctx.scene,
            label.as_str(),
            sx + w - pad - text_w,
            cy - (geom::ROW_H * 0.5 - PORT_LABEL_PAD_Y) * view.zoom,
            size,
            // ⛔⛔ **A SEGUNDA EXCEPÇÃO da varredura de 2026-08-30 — e esta foi uma REGRESSÃO minha,
            // apanhada pela auditoria.** O `text_w` aqui é `approx_width_px()`, uma ESTIMATIVA
            // pelo avanço médio (`LABEL_CHAR_W`), e é a MESMA quantidade que POSICIONA o
            // rótulo (`x = … − text_w`). Como orçamento de quebra uma estimativa curta é
            // inofensiva (o parley não parte dentro de uma palavra: transborda ~2 px para o
            // `PORT_LABEL_PAD_X`); como limite de CORTE é fatal — medido, `Out` (estimativa
            // `16,80`, real `19,07`) saía como **`…`**, o rótulo inteiro a desaparecer.
            // *Uma grandeza não pode ao mesmo tempo posicionar e limitar: isso é circular.*
            f32::INFINITY,
            colour,
        );
    }
}

#[cfg(test)]
#[path = "paint_port_label_tests.rs"]
mod tests;
