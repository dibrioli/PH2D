//! **COMO o valor conduzido se combina com o canal** — os sete modos, e as
//! guardas de dois deles.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é por RESPONSABILIDADE: o `channel.rs` responde *que canal recebe e como o
//! stream é escrito*, e este responde *que aritmética junta os dois números*.

/// How the driven value combines with the existing channel.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Combine {
    /// `channel + value` — the additive default (matches `motion.step`).
    Add,
    /// `value` — overwrite the channel with the driven value.
    Set,
    /// `channel * value` — scale the existing channel by the value.
    Multiply,
    /// `channel − value` — o simétrico do `Add`.
    ///
    /// ⚠️ **Apendado no índice 3** (folha 06 linha 40; C4D *Blending Mode*). A
    /// composição existia — negar o valor a montante com um `value.math` — e é
    /// exactamente por isso que ele é *ergonomia* e não capacidade: um nó a mais
    /// e um sinal invertido no grafo para dizer «menos».
    Subtract,
    /// `channel / value` — o simétrico do `Multiply`.
    ///
    /// ⚠️ **Um divisor (quase) zero colapsa para `0`, e não para `inf`.** É a mesma
    /// guarda que o `value.math` já tinha, com o mesmo limiar e pelo mesmo motivo:
    /// um `inf` num canal de transform envenena a posição do elemento, e daí em
    /// diante todo `NaN` a jusante vem sem endereço.
    Divide,
    /// `min(channel, value)` — um TECTO sobre o canal.
    Min,
    /// `max(channel, value)` — um PISO sobre o canal.
    Max,
    /// **`value`, medido a partir do ZERO** — o *Remap* do C4D (folha 06 linha 41).
    ///
    /// ⚠️ **Ele NÃO é o `Set`, e a diferença é o que a MÁSCARA faz.** Todo modo mistura o
    /// resultado com o canal por `falloff` (`lerp(canal, combinado, f)`), então:
    ///
    /// ```text
    ///   Set    f = 0  ->  o canal fica como estava   (a mascara PROTEGE o valor de origem)
    ///   Remap  f = 0  ->  o canal vai a ZERO         (a mascara mede a partir do nada)
    /// ```
    ///
    /// É a diferença entre *"pinte por cima onde a máscara deixar"* e *"esta máscara É o
    /// valor"*, e ela não é exprimível por nenhum dos sete acima — há gate a enumerá-los.
    Remap,
}

/// Abaixo desta magnitude um divisor é tratado como zero (o quociente colapsa
/// para `0`), o gêmeo exacto do `MIN_DIVISOR` do `value.math`.
const MIN_DIVISOR: f32 = 1e-9;

impl Combine {
    pub(crate) fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Combine::Set,
            2 => Combine::Multiply,
            // ⚠️ Apendados no FIM: `0..2` ficam onde estavam, então todo documento
            // já autorado lê o mesmo modo.
            3 => Combine::Subtract,
            4 => Combine::Divide,
            5 => Combine::Min,
            6 => Combine::Max,
            // ⚠️ Apendado no FIM, pela mesma razão dos quatro acima.
            7 => Combine::Remap,
            _ => Combine::Add,
        }
    }
    pub(crate) fn apply(self, channel: f32, value: f32) -> f32 {
        match self {
            Combine::Add => channel + value,
            Combine::Set => value,
            Combine::Multiply => channel * value,
            Combine::Subtract => channel - value,
            Combine::Divide => {
                if value.abs() < MIN_DIVISOR {
                    0.0
                } else {
                    channel / value
                }
            }
            Combine::Min => channel.min(value),
            Combine::Max => channel.max(value),
            Combine::Remap => value,
        }
    }

    /// **De onde a máscara mede** — o canal para todo modo, e o ZERO para o [`Combine::Remap`].
    ///
    /// ⚠️ Existe porque um `Remap` não é exprimível como uma `apply` pura: com a mistura a
    /// partir do canal, `lerp(c, x, f) = v·f` exigiria `x = c + v − c/f`, que **depende do
    /// `f`** e portanto não é uma combinação de dois números. Quem muda é a BASE, não a conta.
    fn base(self, channel: f32) -> f32 {
        match self {
            Combine::Remap => 0.0,
            _ => channel,
        }
    }

    /// **A porta ÚNICA: combinar e mascarar.**
    ///
    /// ⚠️ **Ela existe porque a lei estava escrita OITO vezes** — seis pares
    /// `apply`+`blend` espalhados pelos canais do `channel.rs`, e a própria fórmula da
    /// mistura em duas closures idênticas. Um modo cuja base é diferente (o `Remap`) teria
    /// de ser lembrado em seis sítios, e o que uma lei escrita em seis sítios faz é divergir
    /// em dois. *Uma lei escrita N vezes ainda não é uma lei — só uma PORTA é.*
    ///
    /// `falloff = 0` deixa o canal como estava (excepto no `Remap`, ver [`Self::base`]);
    /// `1` toma a conduta inteira.
    pub(crate) fn resolve(self, channel: f32, value: f32, falloff: f32) -> f32 {
        let driven = self.apply(channel, value);
        let base = self.base(channel);
        base + (driven - base) * falloff.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Os sete modos que já shipavam, para uma varredura os enumerar.
    const OLD: [Combine; 7] = [
        Combine::Add,
        Combine::Set,
        Combine::Multiply,
        Combine::Subtract,
        Combine::Divide,
        Combine::Min,
        Combine::Max,
    ];

    /// ⭐⭐ **O `Remap` NÃO era exprimível, e a prova é uma ENUMERAÇÃO.** Sob máscara ele dá
    /// `v·f`, e nenhum dos sete modos antigos consegue esse número — a busca é exaustiva, não
    /// um argumento.
    #[test]
    fn no_older_mode_can_produce_what_remap_produces() {
        let (c, v, f) = (3.0f32, 2.0f32, 0.25f32);
        let remap = Combine::Remap.resolve(c, v, f);
        assert!(
            (remap - v * f).abs() < 1e-6,
            "o Remap mede do zero: {remap}"
        );
        for m in OLD {
            let got = m.resolve(c, v, f);
            assert!(
                (got - remap).abs() > 1e-3,
                "um modo antigo reproduz o Remap ({got}) -- a celula estaria fechada por composicao"
            );
        }
    }

    /// ⚠️ **A diferença é a MÁSCARA, e o gate mede as duas pontas.** Com `falloff = 1` o
    /// `Remap` e o `Set` são o MESMO número — se não fossem, `Remap` seria outra coisa que
    /// não «medir a partir do zero».
    #[test]
    fn remap_and_set_agree_at_full_mask_and_split_at_zero() {
        let (c, v) = (3.0f32, 2.0f32);
        assert!(
            (Combine::Remap.resolve(c, v, 1.0) - Combine::Set.resolve(c, v, 1.0)).abs() < 1e-6,
            "sem mascara os dois entregam o valor"
        );
        assert_eq!(Combine::Set.resolve(c, v, 0.0), c, "Set protege a origem");
        assert_eq!(Combine::Remap.resolve(c, v, 0.0), 0.0, "Remap mede do zero");
    }

    /// **A PORTA reduz ao que shipava.** Para todo modo antigo, `resolve` é exactamente o par
    /// `apply` + a mistura que estava escrita à mão nos oito sítios.
    #[test]
    fn the_door_reproduces_the_hand_written_blend_for_every_old_mode() {
        for m in OLD {
            for (c, v, f) in [
                (3.0f32, 2.0f32, 0.25f32),
                (-1.0, 0.5, 1.0),
                (0.0, -4.0, 0.6),
            ] {
                let hand = {
                    let driven = m.apply(c, v);
                    c + (driven - c) * f.clamp(0.0, 1.0)
                };
                assert!(
                    (m.resolve(c, v, f) - hand).abs() < 1e-6,
                    "a porta mudou o resultado de um modo antigo"
                );
            }
        }
    }

    /// **A máscara é COAGIDA na porta**, como estava nas oito closures.
    #[test]
    fn the_mask_is_clamped_at_the_door() {
        let (c, v) = (1.0f32, 5.0f32);
        assert_eq!(
            Combine::Add.resolve(c, v, 2.0),
            Combine::Add.resolve(c, v, 1.0)
        );
        assert_eq!(
            Combine::Add.resolve(c, v, -1.0),
            Combine::Add.resolve(c, v, 0.0)
        );
    }

    /// ⚠️ **O índice `7` é o que o documento guarda** — um documento autorado guarda o
    /// NÚMERO, não o rótulo, então mover um índice troca o modo de toda cena salva.
    #[test]
    fn the_indices_are_frozen_and_remap_is_the_eighth() {
        for (i, want) in [
            (0, Combine::Add),
            (1, Combine::Set),
            (2, Combine::Multiply),
            (3, Combine::Subtract),
            (4, Combine::Divide),
            (5, Combine::Min),
            (6, Combine::Max),
            (7, Combine::Remap),
        ] {
            assert!(
                Combine::from_param(i as f32) == want,
                "o indice {i} mudou de modo"
            );
        }
        // E um índice fora da lista cai no `Add`, como sempre caiu.
        assert!(Combine::from_param(99.0) == Combine::Add);
    }
}
