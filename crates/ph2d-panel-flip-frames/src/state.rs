//! O snapshot que o shell publica por frame + o estado (vazio) do painel.
//!
//! O painel é **stateless** (espelho do `ph2d-panel-flip`): o documento (`FlipDoc`),
//! o playhead e os flags de autoria vivem no shell, que publica um
//! [`FlipStripSnapshot`] ANTES do paint. As edições saem por `ToolPanelEvent` e o
//! drain do shell as aplica. Assim o painel não depende do modelo do Flip — e o
//! undo/save continuam vendo UMA fonte de verdade.

use std::cell::RefCell;

/// Uma célula da tira: uma chave real da camada ativa.
///
/// **Não é `Eq`** desde o `weight` (é `f32`) — só `PartialEq`.
#[derive(Clone, Debug, PartialEq)]
pub struct FlipCell {
    /// O quadro em que a chave começa.
    pub key: i32,
    /// Quantos quadros ela ocupa (a EXPOSIÇÃO, à TVPaint). A última chave mostra
    /// `1` enquanto nada a delimita.
    pub exposure: u32,
    /// É um breakdown (inbetween gerado pelo tween) — pinta mais discreto.
    pub breakdown: bool,
    /// O desenho é compartilhado por mais de uma chave (instância/ciclo).
    pub instanced: bool,
    /// **Marcada para o MULTIFRAME** (W7): o mesmo gesto de escultura/balde age em todas
    /// as chaves marcadas. Sem o realce a feature seria invisível — o usuário marcaria
    /// quadros sem saber quais, e um gesto agiria onde ele não vê.
    pub selected: bool,
    /// **O PESO do multiframe** desta chave `[0,1]` — a força com que o pincel a edita
    /// (o falloff temporal, W7). `1.0` fora do multiframe ou com o falloff desligado. A
    /// COR DO FUNDO da célula o desenha: a célula marcada veste o acento e CLAREIA por
    /// `(1 − peso)` — cheio (acento puro) no ativo, mais claro nos vizinhos. Ligar o Falloff
    /// transforma um bloco de acento uniforme num gradiente de claridade, que é como o efeito
    /// — antes invisível até esculpir e dar scrub — se VÊ na hora (Enio 2026-07-14/15). O peso
    /// é o do pincel (`falloff_at`), então a cor não mente sobre a força do gesto.
    pub weight: f32,
}

/// Tudo que a tira pinta, publicado pelo shell a cada frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlipStripSnapshot {
    /// Há um objeto Flip com camada ativa? `false` ⇒ a tira pinta só o chrome.
    pub has_layer: bool,
    /// Nome da camada ativa (a tira mostra UMA camada — a que se está animando).
    pub layer_name: String,
    /// As células, em ordem de quadro.
    pub cells: Vec<FlipCell>,
    /// O quadro do playhead (pode cair no meio de uma exposição).
    pub current_frame: i32,
    /// A chave ATIVA (a que o quadro atual está mostrando), se houver.
    pub current_key: Option<i32>,
    /// O transporte está tocando?
    pub playing: bool,
    /// FPS do objeto.
    pub fps: f32,
    /// Ghost Frames ligados + quantos de cada lado.
    pub ghost: bool,
    pub ghost_before: u32,
    pub ghost_after: u32,
    /// Autokey (desenhar no rabo de um hold cria chave) + Additive (a chave nova
    /// nasce como CÓPIA, não em branco).
    pub autokey: bool,
    pub additive: bool,
    /// Quantos inbetweens o botão Tween vai gerar.
    pub tween_count: u32,
    /// O preset de easing dos inbetweens (índice em `TWEEN_EASE_NAMES`).
    pub tween_ease: u8,
    /// Traços que existem em só UMA das duas chaves entram/saem esmaecendo.
    pub tween_fade: bool,
    /// A sessão de correção de pares está aberta (o toggle **Pairs** está aceso, e o overlay
    /// mostra a correspondência do intervalo).
    pub tween_pairs: bool,
    /// Ciclo (post behavior) da camada ativa — `CycleMode as u8`.
    pub cycle: u8,
    /// **Falloff temporal** do multiframe: os vizinhos recebem menos influência que o
    /// quadro ativo. Só PINCÉIS o respeitam (o balde é discreto).
    pub falloff: bool,
}

impl FlipStripSnapshot {
    /// O vão de quadros que a tira cobre: `(first, end)` — do começo da 1ª chave ao fim
    /// da exposição da última. `None` sem células. É a régua inteira (a base, antes de
    /// qualquer ciclo): `end = última.key + exposição`.
    #[must_use]
    pub fn frame_span(&self) -> Option<(i32, i32)> {
        let first = self.cells.first()?.key;
        let last = self.cells.last()?;
        let end = last.key + last.exposure.max(1) as i32;
        Some((first, end))
    }

    /// **O quadro sob a régua de scrub** para um `value` de slider `0..1` (W7.3).
    ///
    /// É o **inverso exato** de onde `paint_cells` desenha o handle do playhead
    /// (`x = inner.x + (frame − first)·ppf` sobre uma régua de largura `total·ppf`): o
    /// `ppf` cancela, então o mapa é `first + round(value·total)` — clicar na régua sob o
    /// handle devolve o quadro do handle, sem depender da escala. Sem isso a régua e o
    /// handle seriam **duas portas para a mesma pergunta** e divergiriam
    /// (`feedback_two_doors_to_the_same_question_diverge`). Cravado no vão exibido.
    #[must_use]
    pub fn scrub_frame(&self, value: f32) -> Option<i32> {
        let (first, end) = self.frame_span()?;
        let total = (end - first).max(1);
        let frame = first + (value.clamp(0.0, 1.0) * total as f32).round() as i32; // CLAMP-OK: bounds literais 0..1
        // Não é `clamp(first, end - 1)`: num vão de 1 quadro (`end == first`) os bounds
        // invertem e o clamp de i32 entra em PÂNICO — o par min/max degrada para "sempre
        // o primeiro quadro", que é o certo.
        Some(frame.min(end - 1).max(first))
    }
}

/// **O que um arrasto na tira pede ao documento.**
///
/// Nenhuma operação NOVA: as duas já existem no `ph2d-flip` e já são o que os botões
/// `◀`/`▶` e a caixa **Hold** chamam. O arrasto é uma segunda forma de PEDIR o que já
/// existe — não um segundo caminho para fazê-lo (duas implementações do mesmo verbo
/// divergem; uma segunda porta de PEDIDO, não).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FlipStripIntent {
    /// Mover a chave `from` para o quadro `to` (`FlipObject::move_frame`).
    MoveKey { from: i32, to: i32 },
    /// A chave `key` passa a expor `frames` quadros (`FlipObject::set_exposure`, que
    /// EMPURRA as seguintes — a semântica da tira de exposição).
    SetHold { key: i32, frames: u32 },
}

thread_local! {
    static CURRENT: RefCell<FlipStripSnapshot> = RefCell::new(FlipStripSnapshot::default());
    static INTENTS: RefCell<Vec<FlipStripIntent>> = const { RefCell::new(Vec::new()) };
}

/// **Enfileira um pedido à mão — só para GATES.**
///
/// No produto quem enfileira é o [`crate::strip_drag`], a partir de um gesto real. Esta
/// porta existe para o lado do SHELL poder testar o seu drain sem montar um painel e um
/// ponteiro; a costura painel↔dispatch é provada no seam desta crate, que dirige o
/// ponteiro de verdade.
#[doc(hidden)]
pub fn push_intent_for_tests(intent: FlipStripIntent) {
    push_intent(intent);
}

/// Enfileira um pedido do arrasto (o painel escreve; o shell drena no mesmo frame).
pub(crate) fn push_intent(intent: FlipStripIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// **Drena os pedidos do arrasto da tira** (o shell chama 1× por frame).
///
/// Canal próprio porque `PanelEvent` está congelado (CLAUDE.md §6) e um arrasto não cabe
/// nos quatro variants dele. Espelha o `drain_intents` que a timeline usa pelo mesmo
/// motivo; o **toque** continua saindo por `PanelEvent::Click`, então nada do que já
/// funcionava mudou de rota.
#[must_use]
pub fn drain_flip_strip_intents() -> Vec<FlipStripIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Publica o snapshot da tira (o shell chama 1× por frame com a tool ativa).
pub fn set_current_flip_strip(snap: FlipStripSnapshot) {
    CURRENT.with(|c| *c.borrow_mut() = snap);
}

/// O snapshot deste frame.
#[must_use]
pub fn current_flip_strip() -> FlipStripSnapshot {
    CURRENT.with(|c| c.borrow().clone())
}

/// Estado retido do painel: **só a sessão de arrasto**.
///
/// O documento continua vivendo no shell (a tira é um espelho). O que mora aqui é estado de
/// VISTA — o gesto em curso e o que ele vai pedir — e ele existe porque um arrasto tem
/// percurso: o painel precisa desenhar *para onde a chave vai* antes de o documento saber
/// disso. O documento só muda no fim ([`crate::strip_drag`]).
#[derive(Clone, Debug, Default)]
pub struct FlipStripState {
    pub(crate) drag: Option<crate::strip_drag::StripDrag>,
}

#[cfg(test)]
mod scrub_tests {
    use super::*;

    fn cell(key: i32, exposure: u32) -> FlipCell {
        FlipCell {
            key,
            exposure,
            breakdown: false,
            instanced: false,
            selected: false,
            weight: 1.0,
        }
    }

    fn snap(cells: Vec<FlipCell>) -> FlipStripSnapshot {
        FlipStripSnapshot {
            has_layer: true,
            cells,
            ..Default::default()
        }
    }

    /// Sem células não há vão nem quadro de scrub — a régua não é oferecida.
    #[test]
    fn no_cells_no_scrub() {
        let s = FlipStripSnapshot::default();
        assert_eq!(s.frame_span(), None);
        assert_eq!(s.scrub_frame(0.5), None);
    }

    /// As BORDAS do value mapeiam nas bordas do vão exibido (`gate the edges`): `0.0` na
    /// 1ª chave, `1.0` no ÚLTIMO quadro exibido (`end − 1`, nunca o um-a-mais).
    #[test]
    fn the_value_edges_map_to_the_span_edges() {
        // Chaves em 0, 4, 8 — a última com exposição 1 ⇒ end = 9, último exibido = 8.
        let s = snap(vec![cell(0, 4), cell(4, 4), cell(8, 1)]);
        assert_eq!(s.frame_span(), Some((0, 9)));
        assert_eq!(s.scrub_frame(0.0), Some(0));
        assert_eq!(
            s.scrub_frame(1.0),
            Some(8),
            "1.0 nao pode cair no quadro um-a-mais (end)"
        );
        // Um valor além de 1 ainda clampa dentro do vão.
        assert_eq!(s.scrub_frame(2.0), Some(8));
    }

    /// 🔴 **Um value ENTRE dois quadros pega o mais PRÓXIMO, não o de baixo** — o scrubber
    /// gruda no quadro sob o cursor, não enviesa pra esquerda. É o caso que os inteiros do
    /// `scrub_is_the_exact_inverse` não pegam (lá `round` e `floor` concordam): `value=0.5`
    /// num vão de 9 é `4.5` → o quadro 5 (`round`), NÃO o 4 (`floor`/`trunc`). Sem este
    /// caso, trocar `round` por `floor` passaria em silêncio (`mute o CÓDIGO, não só o teste`).
    #[test]
    fn a_value_between_frames_snaps_to_the_nearest_frame() {
        let s = snap(vec![cell(0, 4), cell(4, 4), cell(8, 1)]); // vão (0, 9), total 9
        assert_eq!(
            s.scrub_frame(0.5),
            Some(5),
            "0.5·9 = 4.5 tinha de arredondar pro 5 (o mais proximo), nao truncar pro 4"
        );
    }

    /// 🔴 **O mapa value→quadro é o INVERSO EXATO da colocação do handle** — clicar na
    /// régua no ponto onde o handle está devolve o quadro do handle, em qualquer escala
    /// (o `ppf` cancela). Sem isso a régua e o handle divergiriam
    /// (`feedback_two_doors_to_the_same_question_diverge`). Mutação que sangra: medir de
    /// outro `first`, ou uma escala diferente (o `round`/`floor` é pego pelo caso de meio-
    /// quadro acima — aqui os pontos são inteiros e os dois coincidem).
    #[test]
    fn scrub_is_the_exact_inverse_of_the_handle_placement() {
        let s = snap(vec![cell(0, 4), cell(4, 4), cell(8, 1)]);
        let (first, end) = s.frame_span().unwrap();
        let total = (end - first) as f32;
        // Para cada quadro exibido, o value onde o handle é desenhado
        // (`(frame - first) / total`) tem de mapear DE VOLTA nesse mesmo quadro.
        for frame in first..end {
            let value_at_handle = (frame - first) as f32 / total;
            assert_eq!(
                s.scrub_frame(value_at_handle),
                Some(frame),
                "o value do handle no quadro {frame} nao voltou nele"
            );
        }
    }
}
