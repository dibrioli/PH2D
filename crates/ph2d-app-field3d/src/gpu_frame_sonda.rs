//! ⭐⭐ **A [`Sonda`]** — o que as sondas de calibração podem desligar no quadro do dispositivo.
//! Irmão do [`super`] por tecto de LOC: é assunto próprio (nenhum caminho de produto o lê fora da
//! fábrica).

/// ⭐⭐ **O que a SONDA de calibração pode desligar — e nada disto é um caminho de produto.**
///
/// ⚠️ *Um número que ninguém consegue voltar a medir é um palpite com data* — esta porta existe para
/// que a comparação entre as duas ordens da fita se possa refazer noutra máquina, na mesma corrida.
///
/// ⛔⛔ **Ela tinha um segundo campo, `tecto`, e ele saiu com o tecto** (2026-09-15): a cerca que
/// mandava uma fita larga para a CPU foi removida quando a medição mostrou que a grandeza dela
/// **não ordena os resultados** — ver a nota no lugar do `MAX_GUARDADOS`, na [`ph2d_field_gpu`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sonda {
    /// `false` = a fita sai na ordem **CRUA** da travessia — ver
    /// [`ph2d_field_eval::tape_schedule`].
    pub escalonar: bool,
    /// `false` = a silhueta **não** é re-amostrada — a porta de BISSECÇÃO da `W7c`.
    ///
    /// ⭐⭐⭐ **Ela deixou de ser uma decisão do quadro em 2026-09-19** (`docs/Render3d/12` §12):
    /// *todo quadro a re-amostra*, e o que sobra aqui é a porta que a desliga para a voltar a medir
    /// ou para bissectar um report.
    pub bordas: bool,
    /// ⭐⭐⭐⭐ **O recorte pela caixa da peça e a GRADE DE LONGE** — `None` é a marcha de sempre,
    /// `Some(0)` só recorta o raio pela caixa, `Some(n)` recorta e salta pela grade de `n` células.
    /// Ver [`crate::preview::a_grade_de_longe`].
    pub longe: Option<u32>,
    /// ⭐⭐⭐⭐ **O tamanho a ENTREGAR** — o da área, quando o traçado é mais pequeno (a resolução
    /// dinâmica do movimento): a imagem sobe a ele NA PLACA ([`ph2d_field_gpu::amplia`]).
    pub entrega: Option<(u32, u32)>,
    /// ⏱️⭐⭐⭐⭐ **A marcha com a fita INTERPRETADA** — o *ubershader*, ver
    /// [`ph2d_field_eval::device::DeviceField::tape_interpretada`]. Só a sonda o liga por ora.
    pub fita_interpretada: bool,
}

impl Default for Sonda {
    /// O caminho do produto: escalonada, e a silhueta re-amostrada em **todo** quadro.
    fn default() -> Self {
        Self {
            escalonar: true,
            bordas: crate::preview::re_amostra_a_silhueta(),
            longe: crate::preview::a_grade_de_longe(),
            entrega: None,
            fita_interpretada: false,
        }
    }
}
