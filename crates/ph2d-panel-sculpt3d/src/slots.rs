//! **O QUE CADA FERRAMENTA LEMBRA** — a tabela de slots e as leis que a movem.
//!
//! ⚠️ **O corte contra o [`super::state`] é de RESPONSABILIDADE:** lá mora o que
//! o painel **PUBLICA e DRENA** (o retrato, os intents, as portas do host);
//! aqui mora o que um VERBO lembra e o que acontece quando o artista troca de
//! ferramenta. Os dois falam do mesmo [`Sculpt3dUi`], e o arquivo cruzou o teto
//! de LOC do painel — a linha de corte é essa, nunca o tamanho.
//!
//! ⚠️ **A lei desta wave cabe numa frase:** trocar de verbo é **salvar o pincel
//! vivo no slot que SAI e carregar o slot que ENTRA** — nada mais. A porta
//! antiga (`arm_verb_defaults`) levava o pincel VIVO para o verbo novo e
//! re-armava campo a campo *"se o artista ainda não mexeu"*, uma heurística que
//! só podia acertar enquanto o artista não mexesse em nada. Com memória
//! por-verbo a pergunta deixa de existir: **o slot SABE**.

use ph2d_sculpt3d::{Brush, RefMode, Verb};

use super::state::{BASE_RADIUS_PX, Sculpt3dUi};

/// A posição de um verbo no [`Verb::ALL`] — o índice da tabela
/// [`Sculpt3dUi::slots`].
///
/// ⚠️ **`Verb` não é `usize`**, e uma segunda tabela `verbo -> índice` seria a
/// cópia que diverge no dia em que a lista crescer no meio.
#[must_use]
pub fn verb_index(verb: Verb) -> usize {
    Verb::ALL.iter().position(|&v| v == verb).unwrap_or(0)
}

/// **O QUE UMA FERRAMENTA LEMBRA** — o pincel dela e o tamanho dela.
///
/// ⚠️ **Duas coisas e não uma**, porque são duas RÉGUAS: o `Brush::radius` é de
/// mundo e derivado por dab (contra a câmera e o ponto de acerto), e o
/// `radius_px` é o número que o artista arrasta. Guardar só o pincel faria a
/// ferramenta lembrar a força e esquecer o tamanho.
#[derive(Clone, Debug, PartialEq)]
pub struct VerbSlot {
    pub brush: Brush,
    pub radius_px: f32,
}

impl VerbSlot {
    /// **O ESTADO DE FÁBRICA de um verbo** — o pincel que ele teria se ninguém o
    /// tivesse tocado.
    ///
    /// ⚠️ **As tabelas `Verb::default_*` são a ÚNICA fonte, e é ela que torna
    /// esta função a resposta e não uma segunda cópia dela.** Antes de
    /// 2026-08-17 havia uma porta de *arming* que perguntava às mesmas tabelas
    /// em tempo de troca, sob a heurística *"arma se o artista ainda não
    /// mexeu"*; ela morreu com esta wave, porque um slot que LEMBRA não
    /// precisa adivinhar o que foi tocado — ele sabe.
    #[must_use]
    pub fn for_verb(verb: Verb) -> Self {
        let mode = RefMode::birth_for(verb);
        Self {
            brush: Brush {
                verb,
                mode,
                strength: verb.default_strength(),
                accumulate: verb.default_accumulate(),
                front_faces_only: verb.default_front_faces_only(),
                falloff: verb.default_falloff(mode),
                hardness: verb.default_hardness(),
                auto_smooth: verb.default_auto_smooth(),
                ..Brush::default()
            },
            radius_px: verb.default_radius_px(BASE_RADIUS_PX),
        }
    }
}

impl Sculpt3dUi {
    /// A referência que **aquele** verbo usa.
    ///
    /// ⚠️ **Para o verbo VIVO ela lê o pincel, não o slot** — o slot dele é a
    /// cópia congelada de quando o artista o largou, e responderia *"como estava
    /// antes"* a uma pergunta sobre agora.
    #[must_use]
    pub fn mode_of(&self, verb: Verb) -> RefMode {
        if verb == self.brush.verb {
            self.brush.mode
        } else {
            self.slots[verb_index(verb)].brush.mode
        }
    }

    /// Escolhe a referência de um verbo — no pincel se ele é o vivo, no slot se
    /// não é.
    ///
    /// ⚠️ **Ela RECONCILIA, e o gate `arch_mode_has_reconcile` apanhou o dia em
    /// que ela não o fazia.** A curva de fábrica é função de **verbo E modo**
    /// ([`Verb::default_falloff`]), então escrever o modo e deixar o falloff
    /// onde estava resolve-o contra a referência ERRADA. Enquanto a tabela
    /// guardava só o modo (`mode_by_verb`) não havia nada por-verbo que pudesse
    /// envelhecer; **o slot com o pincel inteiro criou esse estado a jusante no
    /// mesmo commit em que este setter nasceu** — o artista carimbava `B` em
    /// todos, pegava um, e recebia a quártica do SculptGL sob um chip que dizia
    /// Blender.
    pub fn set_mode_of(&mut self, verb: Verb, mode: RefMode) {
        let brush = if verb == self.brush.verb {
            &mut self.brush
        } else {
            &mut self.slots[verb_index(verb)].brush
        };
        reconcile_mode(brush, mode);
    }
}

/// **O QUE MUDAR DE REFERÊNCIA RE-RESOLVE** — a lei, sobre o pincel que a
/// carrega.
///
/// Trocar a referência muda a LEI do kernel, e a curva de fábrica é função de
/// **verbo e modo** — então a escolha nova pode trazer uma curva nova junto. A
/// regra é a de sempre: *arma se, e só se, o artista ainda não mexeu*, onde
/// **"não mexeu" é o valor estar exatamente no default do modo que SAI**.
/// Nenhuma troca pode APAGAR uma escolha deliberada — o precedente do
/// `arm_inflate_defaults` do Painter.
///
/// ⚠️ **Ela é uma função e não duas porque tem DOIS chamadores:** o chip de
/// referência, que mexe no pincel VIVO, e o botão *aplicar a todos*, que mexe no
/// pincel de cada SLOT. Duas cópias divergiriam no primeiro campo que a próxima
/// wave fizer depender do modo — e o `falloff` já é um.
pub fn reconcile_mode(brush: &mut ph2d_sculpt3d::Brush, mode: RefMode) {
    let from = brush.mode;
    if from == mode {
        return;
    }
    let verb = brush.verb;
    brush.mode = mode;
    if brush.falloff == verb.default_falloff(from) {
        brush.falloff = verb.default_falloff(mode);
    }
}

/// **TROCA DE FERRAMENTA — a porta única, e a única que escreve nos slots.**
///
/// Ela guarda o pincel vivo no slot do verbo que SAI e carrega o do verbo que
/// ENTRA. É isto, e nada mais: nenhum knob é re-armado, nenhum default é
/// re-aplicado, nenhuma escolha atravessa a fronteira.
///
/// ⚠️ **Isto SUBSTITUI o antigo `arm_verb_defaults`, que fazia o oposto** — ele
/// propagava o pincel vivo para o verbo novo e re-armava campo a campo *"se o
/// artista ainda não mexeu"*, uma heurística que só podia acertar enquanto o
/// artista não mexesse em nada. O que ela custava está medido no report do Enio:
/// afinar o Smooth e pegar o Clay levava a força do Smooth junto.
///
/// ⚠️ **Trocar para o verbo que já está em mãos é NO-OP**, e não por higiene:
/// sem a guarda, um clique repetido no mesmo chip salvaria o vivo e o
/// recarregaria — inofensivo hoje, e a linha exata onde um caso especial futuro
/// (um slot que só é escrito sob condição) passaria a apagar o estado vivo.
pub fn switch_verb(ui: &mut Sculpt3dUi, verb: Verb) {
    switch_verb_parts(&mut ui.slots, &mut ui.brush, &mut ui.radius_px, verb);
}

/// **A LEI da troca, sobre as três coisas que ela move** — e nada mais.
///
/// ⚠️ **Ela existe porque a troca tem DOIS chamadores em espaços diferentes:**
/// o painel, que carrega um [`Sculpt3dUi`] inteiro, e o **teclado do shell**,
/// que tem a cena na mão e **não tem** como montar um `Sculpt3dUi` (ele
/// precisaria de fatos da cena 2D que uma escultura não responde). Antes disto
/// a rota do teclado tinha uma lei PRÓPRIA — `Brush::arm_verb_defaults` — e a
/// divergência estava escrita no comentário dela: *"o falloff, a referência e o
/// raio seguem armados só pela rota do painel"*. Trocar de ferramenta pelo
/// atalho deixava três campos como estavam; pelo chip, não.
///
/// ⚠️ **Duas portas para um gesto divergem, e esta já tinha divergido.** Uma lei
/// e dois adaptadores é o que impede a terceira.
pub fn switch_verb_parts(
    slots: &mut [VerbSlot],
    brush: &mut ph2d_sculpt3d::Brush,
    radius_px: &mut f32,
    verb: Verb,
) {
    let from = brush.verb;
    if from == verb {
        return;
    }
    slots[verb_index(from)] = VerbSlot {
        brush: brush.clone(),
        radius_px: *radius_px,
    };
    let slot = slots[verb_index(verb)].clone();
    *brush = slot.brush;
    *radius_px = slot.radius_px;
    // ⚠️ **Redundante hoje e mantido**: todo slot nasce com o próprio verbo e a
    // única escrita passa por aqui, então o campo já está certo. Ele existe
    // porque um `Brush` com o verbo de OUTRA ferramenta é o estado que dirige o
    // kernel errado em silêncio, e é barato tornar isso inalcançável.
    brush.verb = verb;
}

/// **RE-RESOLVE o que depende da REFERÊNCIA, sem trocar de ferramenta.**
///
/// Trocar a referência muda a LEI do kernel, e a curva de fábrica é função de
/// **verbo e modo** — então a escolha nova pode trazer uma curva nova junto. A
/// lei é a de sempre: *arma se, e só se, o artista ainda não mexeu*, onde
/// **"não mexeu" é o valor estar exatamente no default do modo que SAI**.
/// Nenhuma troca pode APAGAR uma escolha deliberada — o precedente do
/// `arm_inflate_defaults` do Painter.
///
/// ⚠️ **O RAIO não entra aqui, e a ausência é medida:** ele é função do VERBO
/// (`default_radius_px`), não do modo, então trocar de referência nunca o move.
/// Ele viaja no [`VerbSlot`], que é onde um tamanho de ferramenta pertence.
pub fn arm_mode_defaults(ui: &mut Sculpt3dUi, mode: RefMode) {
    reconcile_mode(&mut ui.brush, mode);
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
