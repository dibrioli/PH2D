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
    /// `false` = o campo do chão não é assado, e o pintor soma zero — a porta pela qual o PREÇO
    /// dele se mede no MESMO processo, intercalado (ver a nota do módulo: entre duas corridas desta
    /// máquina o mesmo passe já deu `11,36` e `5,50 ms`).
    pub chao_recebe_cor: bool,
    /// `false` = a passagem do RICOCHETE não corre — a mesma porta, para o outro passageiro da
    /// bandeira do quadro assente.
    ///
    /// ⚠️ **Ela existe porque a bandeira levava TRÊS coisas** (a borda re-amostrada, o ricochete e
    /// o campo do chão) e uma medição que as some atribui o preço ao passageiro errado. Foi com as
    /// duas desligadas que a `W7c` mediu a segunda passagem da silhueta **sozinha**.
    pub ricochete: bool,
    /// `false` = a silhueta **não** é re-amostrada — a porta de BISSECÇÃO da `W7c`.
    ///
    /// ⭐⭐⭐ **Ela deixou de ser uma decisão do quadro em 2026-09-19** (`docs/Render3d/12` §12):
    /// medida no caminho do pintor, a `1920×1080`, a segunda passagem custa `1,03×`–`1,09×` do
    /// quadro de movimento — contra os `1,30×`–`1,40×` da tabela de CPU que a tinha posto fora
    /// dele. ⇒ *todo quadro a re-amostra*, e o que sobra aqui é a porta que a desliga para a voltar
    /// a medir ou para bissectar um report.
    pub bordas: bool,
    /// ⭐⭐⭐⭐ **A fita da peça sai do shader do PINTOR quando ninguém a lê** — ver
    /// [`crate::preview::a_fita_sai_do_pintor`] e
    /// [`ph2d_field_gpu::paint::PaintSetup::le_o_campo`].
    ///
    /// ⚠️ **Ela é um campo da sonda e não só uma env var** porque a porta é um `OnceLock`: um gate
    /// que precise dos DOIS lados não pode virá-la a meio do processo, e sem isto a única régua
    /// possível seria um golden de GPU. *A régua desta cura é a CONTA de pipelines, e ela precisa de
    /// comparar os dois caminhos na mesma corrida.*
    pub fita_inerte: bool,
    /// ⭐⭐⭐⭐ **O campo do chão é reaproveitado entre quadros** — ver [`campo_do_chao`] e
    /// [`ChaveDoChao`]. `PH2D_FIELD_CHAO_CACHE=0` bissecta, e um gate precisa dos DOIS lados na
    /// mesma corrida (a porta é um `OnceLock`, que não se vira a meio do processo).
    pub chao_em_cache: bool,
    /// ⭐⭐⭐⭐ **O recorte pela caixa da peça e a GRADE DE LONGE** — `None` é a marcha de sempre,
    /// `Some(0)` só recorta o raio pela caixa, `Some(n)` recorta e salta pela grade de `n` células.
    /// Ver [`crate::preview::a_grade_de_longe`].
    pub longe: Option<u32>,
    /// ⭐⭐⭐⭐ **A oclusão do céu marcha numa GRADE assada de `n` células** e o raio primário
    /// continua exacto — `None` é a oclusão sobre a árvore, a de sempre. Ver
    /// [`ph2d_field_gpu::longe::Longe::so_ceu`] e `docs/Render3d/03` §W9, «a oclusão na grade».
    pub ceu_na_grade: Option<u32>,
    /// ⭐⭐⭐⭐ **O passo da oclusão no quadro de MOVIMENTO** — ver
    /// [`ph2d_field_gpu::trace::MarchSetup::ceu_passo`] e [`crate::preview::o_passo_do_ceu_a_mexer`].
    /// O quadro ASSENTE pede sempre `1`: é ele que a paridade com a CPU mede.
    pub ceu_passo: u32,
    /// ⭐⭐⭐⭐ **A oclusão NO TEMPO** — ver [`ph2d_field_gpu::ceu_tempo`] e
    /// [`crate::preview::o_ceu_vive_no_tempo`]: o assente grava o histórico e o movimento herda-o.
    pub ceu_no_tempo: bool,
    /// ⏱️ **Sem oclusão nenhuma** — o CHÃO do relógio de um quadro: o que ele custa se o céu fosse
    /// de graça. Só as sondas o ligam; é a régua do que a oclusão ainda pesa.
    pub sem_ceu: bool,
    /// ⭐⭐⭐⭐ **O tamanho a ENTREGAR** — o da área, quando o traçado é mais pequeno (a resolução
    /// dinâmica do movimento): a imagem sobe a ele NA PLACA ([`ph2d_field_gpu::amplia`]).
    pub entrega: Option<(u32, u32)>,
}

impl Default for Sonda {
    /// O caminho do produto: escalonada, o chão recebe a cor da peça, o ricochete corre — e a
    /// silhueta é re-amostrada em **todo** quadro.
    fn default() -> Self {
        Self {
            escalonar: true,
            chao_recebe_cor: true,
            ricochete: true,
            bordas: crate::preview::re_amostra_a_silhueta(),
            fita_inerte: crate::preview::a_fita_sai_do_pintor(),
            chao_em_cache: crate::preview::o_campo_do_chao_e_reaproveitado(),
            longe: crate::preview::a_grade_de_longe(),
            ceu_na_grade: None,
            ceu_passo: crate::preview::o_passo_do_ceu_a_mexer(),
            ceu_no_tempo: crate::preview::o_ceu_vive_no_tempo(),
            sem_ceu: false,
            entrega: None,
        }
    }
}
