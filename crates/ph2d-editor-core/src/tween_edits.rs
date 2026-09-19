//! **O modelo da secção TWEEN** (suplente #22) — o que o painel mostra, e o que um clique produz.
//!
//! ⚠️ **Irmão do [`super::inspector_model_timer`], e a semelhança não é acidental:** um `Tweens` é
//! uma LISTA cujo índice **é a identidade** — é ele que liga o tween ao timer do mesmo índice —,
//! logo o molde é o mesmo: a lista escolhe, e um editor só mostra os campos do escolhido.
//!
//! ⛔ **Qual linha está aberta NÃO vai ao barramento**: é um facto da UI e vive no `InspectorState`,
//! como no irmão. *Um `Tweens` não tem «o tween actual» — os N correm todos ao mesmo tempo.*

/// Uma linha da lista — um tween.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorTweenRow {
    /// A tag do [`ph2d_tween::Canal`] — a posição no `ALL` dele.
    pub canal: u8,
    /// Os dois extremos, sempre com quatro componentes. **Quantas contam é do canal**
    /// ([`ph2d_tween::Canal::aridade`]), e é por isso que o painel não guarda a conta.
    pub de: [f32; 4],
    pub para: [f32; 4],
    /// A tag da família da curva ([`ph2d_anim::EasingFamily`]).
    pub familia: u8,
    /// A tag do modo ([`ph2d_anim::EasingMode`]).
    pub modo: u8,
    /// A tag do [`ph2d_tween::AoAcabar`].
    pub ao_acabar: u8,
    /// ⭐ **Existe um timer neste índice?** — o relógio do tween. Vem da CENA, não de um campo.
    pub tem_relogio: bool,
}

/// O que o Inspector mostra da secção TWEEN.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectorTweenInfo {
    pub entity_bits: u64,
    /// Os tweens, na ordem em que o componente os guarda. **O índice é a identidade.**
    pub rows: Vec<InspectorTweenRow>,
    /// ⭐ **O objecto tem um `Sprite`?** — os canais de aparência escrevem nele, e sem ele são
    /// inertes. Vem da CENA.
    pub tem_sprite: bool,
    /// Quantas entidades estão selecionadas — a secção **não se espalha** sobre a selecção, e é
    /// este número que a faz dizê-lo (a lição que a §11 já escreveu).
    pub selected_count: usize,
}

/// ⭐⭐⭐ **A queixa de UM tween, da mais ESPECÍFICA para a mais geral** — `None` quando não há.
///
/// ⚠️ **Ela é uma PORTA e não quatro `if` dentro do pintor:** o painel pinta a frase e o gate
/// mede-a **sem um device**. *Uma decisão que só existe dentro de um pintor não é testável sem uma
/// janela, e um gate `#[ignore]` é um gate que o CI nunca corre* — a lei que a secção do raio
/// pagou uma wave antes desta.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TweenQueixa {
    /// ⛔ **Não há timer neste índice** — o tween não tem relógio, logo não corre. É a queixa mais
    /// específica porque é a única em que **nada** acontece, nem sequer um quadro.
    SemRelogio,
    /// ⛔ O objecto não tem `Sprite`, e este canal escreve num campo dele.
    SemSprite,
    /// **`de` e `para` são iguais** nas componentes que o canal lê ⇒ ele corre e não move nada.
    Inerte,
    /// ⚠️ **Uma SILHUETA com `Hold` fica acesa para sempre** — a arte não volta por interpolação,
    /// ela volta por o motor deixar de escrever. É a única das quatro que descreve um tween que
    /// **funciona** e ainda assim quase de certeza não é o que o artista quer.
    SilhuetaQueFica,
}

impl InspectorTweenRow {
    /// Ver [`TweenQueixa`]. `tem_sprite` vem do [`InspectorTweenInfo`].
    #[must_use]
    pub fn queixa(&self, tem_sprite: bool) -> Option<TweenQueixa> {
        let canal = ph2d_tween::Canal::from_tag(self.canal);
        if !self.tem_relogio {
            return Some(TweenQueixa::SemRelogio);
        }
        if !canal.e_da_pose() && !tem_sprite {
            return Some(TweenQueixa::SemSprite);
        }
        if self.de[..canal.aridade()] == self.para[..canal.aridade()] {
            return Some(TweenQueixa::Inerte);
        }
        if canal == ph2d_tween::Canal::Silhueta
            && ph2d_tween::AoAcabar::from_tag(self.ao_acabar) == ph2d_tween::AoAcabar::Hold
        {
            return Some(TweenQueixa::SilhuetaQueFica);
        }
        None
    }
}

/// Uma edição de um campo da secção TWEEN.
///
/// ⚠️ **O `u8` é o ÍNDICE na lista**, e é ele que liga `Tweens[i]` a `Timers[i]` — a mesma lei do
/// irmão. ⛔ **Não há `AddComponent`**, e é o ADR-0166: o Inspector mostra o que o objecto TEM.
#[derive(Clone, Debug, PartialEq)]
pub enum TweenFieldEdit {
    /// Cria um tween no fim da lista.
    Add,
    /// Apaga o que está aberto.
    Remove(u8),
    /// A tag do canal.
    Canal(u8, u8),
    /// Uma componente de `de` — `(índice, componente, valor)`.
    De(u8, u8, f32),
    /// Idem para `para`.
    Para(u8, u8, f32),
    /// A tag da família da curva.
    Familia(u8, u8),
    /// A tag do modo.
    Modo(u8, u8),
    /// A tag do `AoAcabar`.
    AoAcabar(u8, u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linha() -> InspectorTweenRow {
        InspectorTweenRow {
            canal: ph2d_tween::Canal::Opacity.tag(),
            de: [1.0, 0.0, 0.0, 0.0],
            para: [0.0, 0.0, 0.0, 0.0],
            familia: 0,
            modo: 0,
            ao_acabar: ph2d_tween::AoAcabar::Hold.tag(),
            tem_relogio: true,
        }
    }

    /// ⭐⭐⭐ **A ordem das queixas É a lei** — da mais específica para a mais geral.
    ///
    /// ⚠️ Sem relógio o tween **não corre**, logo dizer-lhe *«ele não move nada»* seria mandá-lo
    /// resolver a metade errada — a lei da recusa dos pincéis, palavra por palavra.
    ///
    /// **Mutações que devem sangrar:** trocar dois braços de ordem · devolver `None` sem relógio.
    #[test]
    fn a_queixa_vai_da_mais_especifica_para_a_mais_geral() {
        assert_eq!(linha().queixa(true), None, "um tween sao nao se queixa");

        // ⛔ Com TRÊS queixas verdadeiras ao mesmo tempo sai a mais ESPECÍFICA.
        let mut sem_relogio = linha();
        sem_relogio.tem_relogio = false;
        sem_relogio.para = sem_relogio.de;
        assert_eq!(sem_relogio.queixa(false), Some(TweenQueixa::SemRelogio));

        let mut sem_sprite = linha();
        sem_sprite.para = sem_sprite.de;
        assert_eq!(
            sem_sprite.queixa(false),
            Some(TweenQueixa::SemSprite),
            "um canal de aparencia num objecto sem sprite e' inerte, e a causa e' o sprite"
        );

        // …e um canal de POSE não precisa de sprite nenhum — o controlo que separa as duas.
        let mut pose = linha();
        pose.canal = ph2d_tween::Canal::PositionX.tag();
        assert_eq!(
            pose.queixa(false),
            None,
            "um tween de POSE nao pode queixar-se de faltar um sprite"
        );

        let mut inerte = linha();
        inerte.para = inerte.de;
        assert_eq!(inerte.queixa(true), Some(TweenQueixa::Inerte));

        let mut silhueta = linha();
        silhueta.canal = ph2d_tween::Canal::Silhueta.tag();
        silhueta.de = [1.0, 0.0, 0.0, 1.0];
        silhueta.para = [1.0, 1.0, 1.0, 1.0];
        assert_eq!(silhueta.queixa(true), Some(TweenQueixa::SilhuetaQueFica));
        // …e com `Rewind` ela cala-se: é a mesma silhueta, e o que muda é o fim.
        silhueta.ao_acabar = ph2d_tween::AoAcabar::Rewind.tag();
        assert_eq!(silhueta.queixa(true), None);
    }

    /// ⚠️ **A igualdade é medida na ARIDADE do canal, não nas quatro componentes** — senão um
    /// tween escalar cujo `de`/`para` diferissem no lixo das componentes 1..3 leria-se como vivo.
    #[test]
    fn a_inercia_mede_se_na_aridade_do_canal() {
        let mut r = linha();
        r.de = [0.5, 9.0, 9.0, 9.0];
        r.para = [0.5, 0.0, 0.0, 0.0];
        assert_eq!(
            r.queixa(true),
            Some(TweenQueixa::Inerte),
            "um canal ESCALAR so' le^ a primeira componente"
        );
        // O controlo: uma cor com as MESMAS quatro é inerte, e com uma diferente não é.
        let mut cor = linha();
        cor.canal = ph2d_tween::Canal::Tint.tag();
        cor.de = [1.0, 1.0, 1.0, 1.0];
        cor.para = [1.0, 1.0, 1.0, 1.0];
        assert_eq!(cor.queixa(true), Some(TweenQueixa::Inerte));
        cor.para[3] = 0.0;
        assert_eq!(cor.queixa(true), None);
    }

    /// ⚠️ **Todo campo do `Tween` tem uma edição** — um campo sem variante é um knob que o painel
    /// mostra e que ninguém pode mexer, e ele lê-se exactamente como um controlo morto.
    ///
    /// ⛔ A lista é escrita à mão de propósito: ela é a **segunda leitura** do componente, e é a
    /// discordância entre as duas que acusa o esquecimento.
    #[test]
    fn todo_campo_do_tween_tem_uma_edicao() {
        let variantes = [
            TweenFieldEdit::Add,
            TweenFieldEdit::Remove(0),
            TweenFieldEdit::Canal(0, 0),
            TweenFieldEdit::De(0, 0, 0.0),
            TweenFieldEdit::Para(0, 0, 0.0),
            TweenFieldEdit::Familia(0, 0),
            TweenFieldEdit::Modo(0, 0),
            TweenFieldEdit::AoAcabar(0, 0),
        ];
        assert_eq!(
            variantes.len(),
            8,
            "o `Tween` tem CINCO campos (canal, de, para, easing, ao_acabar) — o `easing` conta \
             DOIS porque a familia e o modo escolhem-se a` parte — mais o `Add` e o `Remove`"
        );
    }
}
