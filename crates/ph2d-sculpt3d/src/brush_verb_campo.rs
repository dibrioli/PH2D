//! ⭐ **QUAL CAMPO ELÁSTICO cada verbo consome** — a metade *qual* da pergunta
//! cuja metade *se* mora no [`crate::RefMode::field`].
//!
//! Irmão (`#[path]`) do [`super::brush_verb`], e o corte é o mesmo que já
//! separou dele o [`super::brush_verb_grip`] e o
//! [`super::brush_verb_predicados`], com outro sujeito: lá mora *quais verbos
//! existem e como se chamam*, no do grip *como a mão os conduz*, e aqui *que
//! kernel elástico cada um sabe receber*.
//!
//! ⚠️ **O corte foi FORÇADO pelo tecto de LOC** (o ficheiro-mãe chegou a `678`
//! e o pincel de esfregar não cabia) e é melhor por isso: a tabela dos campos
//! carrega a tabela MEDIDA que retirou a família do aperto, e ela passa a ter
//! um ficheiro onde se lê inteira em vez de ser o rabo de uma lista de nomes.
//! ⛔ *Subir o número em vez de cortar é o que o `CLAUDE.md` §2 proíbe por
//! escrito.*

use super::verb::Verb;

impl Verb {
    /// **QUAL campo elástico este verbo consegue consumir** — a metade *qual* da
    /// pergunta cuja metade *se* mora no [`crate::RefMode::field`].
    ///
    /// ⚠️ **Ela é do VERBO porque é um fato sobre ele, não sobre o modo:** um
    /// verbo que gira só sabe consumir uma torção, e o alvo dele nomeia esse
    /// kernel. Enquanto as duas metades viviam na tabela do modo, um par trocado
    /// era **escrivível** e o produto o engolia em silêncio — o alvo caía no
    /// modo que já shipava e a pegada continuava a do campo
    /// ([`Brush::query_radius`] só pergunta `is_some`), ou seja um `l-mode` com
    /// o alcance e sem a lei. A mutação que o instalou passou nos **193** gates.
    ///
    /// ⇒ Com o *qual* aqui, o par deixa de poder discordar: não há segundo sítio
    /// para ele estar escrito.
    #[must_use]
    pub const fn elastic_field(self) -> Option<crate::Field> {
        match self {
            // O agarre (eq. 5). O Snake Hook é o MESMO campo com a âncora a
            // andar — o que os separa é o [`Grip::Hook`], não a lei.
            Self::Move | Self::SnakeHook => Some(crate::Field::Grab),
            Self::Twist => Some(crate::Field::Twist),
            Self::LocalScale | Self::Magnify => Some(crate::Field::Scale),
            // ⛔ **A FAMÍLIA QUE APERTA NÃO TEM CAMPO, e a linha que dizia
            // `Some(Field::Pinch)` foi RETIRADA em 2026-08-15 depois de um
            // report do Enio (*"Blob modo L ruim … em L Pinch ruim"*) e de a
            // medição concordar com ele em três eixos** — sonda
            // `measure_pinch_family_modes`, malha de 64×96, pincel `r = 0,30`,
            // traço de 8 eventos a força 0,75:
            //
            // | verbo | modo | fora do anel | ΔV/V (10⁻⁴) |
            // |---|---|---|---|
            // | Pinch | S | 0,0 % | −0,92 |
            // | Pinch | **L** | **62,4 %** | **−4,43** |
            // | Crease | S | 0,0 % | −9,50 |
            // | Crease | **L** | **43,7 %** | −11,48 |
            // | Blob | B | 0,0 % | +10,52 |
            // | Blob | **L** | **46,5 %** | +11,95 |
            //
            // ⚠️ **Metade a dois terços do gesto caía FORA do anel do cursor** —
            // o `KELVINLET_REACH = 3` é a feature do verbo que AGARRA (o doc
            // dele nomeia o preço: *"o anel do cursor deixa de significar o que
            // eu toco"*) e é o defeito de um verbo que APERTA, que é local por
            // definição.
            //
            // ⚠️ **E o campo PIORAVA justamente o que ele existia para curar.**
            // A nota do [`crate::Verb::Pinch`] afirmava *"com campo ele deixa de
            // REMOVER VOLUME … o que sai de lado sai pela normal: aperta E
            // espirra"*; medido, o Pinch com campo remove **4,8× mais** volume
            // que o sem, e dentro do anel o deslocamento normal é **NEGATIVO**
            // (−0,00078 na banda 0,5-0,75 r contra um lateral de +0,00761): ele
            // AFUNDA, não espirra. O mecanismo é geometria — o traço zero
            // reparte `+s` na normal e `−s/2` no plano, mas numa MALHA os
            // vértices vivem na superfície (`r · n ≈ 0`), então o termo normal é
            // ~zero e não há material fora do plano para receber o que sai de
            // lado. *Uma casca não tem para onde espirrar.*
            //
            // ⛔ **E não há corte honesto que o localize:** o perfil lateral é
            // quase CHATO até o anel (0,00304 · 0,00649 · 0,00761 · 0,00666 nas
            // quatro bandas de dentro) e ainda vale **88 % do pico** em `1,0 r`
            // — cortá-lo ali seria um degrau trinta vezes maior que os 2,90 %
            // que o [`crate::kelvinlet::rim_landing`] foi construído para curar.
            //
            // ⚠️ **A REFERÊNCIA chegou à mesma conclusão, e é isso que fecha:**
            // a deformação elástica da referência porta este paper e declara
            // CINCO famílias — agarrar (em três escalas), escalar e torcer.
            // **Nenhuma é o pinch.** O SculptGL não tem Kelvinlets. O
            // paper tem a família afim de traço zero como MATEMÁTICA e nenhum
            // escultor a shipa como PINCEL.
            //
            // ⇒ Um chip `L` aqui era exatamente o que a §4 do plano proíbe: uma
            // LEI inteira vestida com a autoridade de uma fonte que não a
            // declara. O `L` desaparece destes três por construção — o
            // [`crate::RefMode::declares`] pergunta `field(verb).is_some()` —, e
            // não por uma segunda lista a manter.
            _ => None,
        }
    }
}
