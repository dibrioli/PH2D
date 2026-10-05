//! **O verbo do arrasto e o pincel de peso** — os tipos que a ferramenta de osso guarda e a shell
//! espelha. Vieram da `ph2d-tool-vector` (`params_mode.rs`) com o A14, sem mudar uma lei.

/// ⭐⭐⭐ **O QUE O ARRASTO FAZ NO MODO OSSO** — criar ou transformar (Enio, 2026-09-07:
/// *«do modo como está fica confuso para o usuário»*).
///
/// ⛔ **Antes eram os dois ao mesmo tempo, e a ambiguidade era do PONTEIRO:** um arrasto sobre um
/// osso posava-o, um arrasto no vazio criava. Isso torna **inalcançáveis** dois gestos legítimos —
/// começar um osso *em cima* de outro, e posar um osso *sem medo* de criar um por engano — e
/// obriga o artista a saber o que está por baixo do cursor antes de carregar.
///
/// É a mesma partição que o Moho faz com ferramentas separadas (*Add Bone* × *Translate/Rotate
/// Bone*) e o Spine com modos, e a razão é a mesma: **um gesto, um verbo**.
///
/// ⚠️ Ele é um estado do MODO Osso, e não um par de pills na fileira de modos: a fileira responde
/// *«que ferramenta»*, este responde *«o que ela faz»* — e são perguntas de níveis diferentes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BoneAction {
    /// Arrastar **faz** um osso, a partir da ponta do osso aceso (ou do ponto do press).
    /// Carregar num osso apenas o **selecciona** — é assim que se escolhe onde ramificar.
    #[default]
    Create,
    /// Arrastar **posa** o que está sob o cursor: corpo gira, bolinha desloca, quadradinho da
    /// mancha muda a força, anel duplo da ponta dobra a corrente (IK). ⛔ Nunca cria.
    Transform,
    /// ⭐⭐⭐ **Arrastar PINTA o peso** do osso aceso sobre a arte presa — a correcção à mão de
    /// [`ph2d_skeleton::Correccao`], para quando a conta automática erra num sítio.
    ///
    /// ⚠️ **É um verbo do mesmo nível dos outros dois, e não um modo dentro do *Transformar*:** a
    /// pergunta que a fileira responde é *«o que este arrasto faz»*, e pintar peso não é posar nem
    /// criar. ⛔ Escondê-lo atrás de um modificador de teclado seria a meia-porta que o §5.0 nomeia
    /// (*«um gesto que só existe se o artista adivinhar o modificador é meio gesto»*).
    ///
    /// ⚠️ **PARA QUE LADO ele empurra é uma [`WeightDirection`]**, dois botões na secção dele.
    /// ⛔⛔ Até 2026-09-19 esta linha dizia *«o SINAL do valor é a direcção — não há um segundo
    /// verbo a lembrar»*, e a premissa morreu por **ordem do dono** (*«no lugar de valores
    /// negativos em Brush Strength prefiro botões Add e Subtract»*). Ver [`WeightDirection`].
    Weight,
}

impl BoneAction {
    /// As três, na ordem em que o grupo as mostra. ⛔ Fonte única da iteração.
    pub const ALL: [BoneAction; 3] = [
        BoneAction::Create,
        BoneAction::Transform,
        BoneAction::Weight,
    ];

    /// ⭐⭐ **O ÍNDICE deste verbo em [`Self::ALL`]** — a porta que a fileira do painel acende.
    ///
    /// ⛔⛔ **Ela existe porque o índice era um `bool`** (`usize::from(acao == Transform)`), e isso
    /// está certo com dois verbos e **mente em silêncio** com três: o terceiro leria `0` e acenderia
    /// o primeiro segmento. *Um índice derivado de uma comparação é uma tabela escrita à mão com
    /// outra sintaxe.*
    #[must_use]
    pub fn indice(self) -> usize {
        Self::ALL.iter().position(|a| *a == self).unwrap_or(0)
    }

    /// ⭐ **O verbo que um segmento do painel arma** — porta ÚNICA da ferramenta e da shell (que
    /// põe a ferramenta na mão antes de o clique lhe chegar).
    #[must_use]
    pub fn of_segment(id: ph2d_a11y::NodeId) -> Option<BoneAction> {
        [
            (crate::ids::VECTOR_BONE_ACT_CREATE, BoneAction::Create),
            (crate::ids::VECTOR_BONE_ACT_TRANSFORM, BoneAction::Transform),
            (crate::ids::VECTOR_BONE_ACT_WEIGHT, BoneAction::Weight),
        ]
        .into_iter()
        .find_map(|(x, a)| (x == id).then_some(a))
    }
}

/// ⭐⭐⭐ **PARA QUE LADO A PINCELADA DE PESO EMPURRA** — dois botões, e não o sinal de um número.
///
/// ⛔⛔⛔ **Ordem do dono (2026-09-19): *«no lugar de valores negativos em Brush Strength prefiro
/// botões Add e Subtract»*.** A objecção que estava escrita no painel — *«o `Amount` é COM SINAL, e
/// é isso que faz o gesto ser um só; um segundo chip seria a segunda maneira de dizer a mesma
/// coisa»* — fica **registada e não vencida**.
///
/// ⭐ **E a arrumação que ela traz é uma pergunta por controlo:** o número passa a responder
/// *QUANTO* (uma magnitude, que não tem sinal que faça sentido) e os dois botões respondem *PARA QUE
/// LADO*. Enquanto o sinal vivia no número, *«tirar peso»* era um estado invisível — o artista tinha
/// de **ler um menos** para saber o que o próximo arrasto ia fazer.
///
/// ⚠️ **Elas são um SEGMENTO exclusivo e não duas caixas**: as duas ligadas ao mesmo tempo não
/// significam nada, e duas caixas independentes exprimem esse estado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeightDirection {
    /// A pincelada **SOMA** peso ao osso em foco. O valor de fábrica: é o que o artista faz
    /// primeiro, e o que ele espera sem ter escolhido nada.
    #[default]
    Add,
    /// A pincelada **TIRA** peso ao osso em foco.
    Subtract,
}

impl WeightDirection {
    /// As duas, na ordem em que o segmento as mostra. ⛔ Fonte única da iteração.
    pub const ALL: [WeightDirection; 2] = [WeightDirection::Add, WeightDirection::Subtract];

    /// ⭐⭐ **O ÍNDICE desta direcção em [`Self::ALL`]** — a porta que o segmento do painel acende.
    ///
    /// ⚠️ **Derivado da lista e não escrito à mão**, pela mesma razão que o
    /// [`BoneAction::indice`] existe: *um índice derivado de uma comparação é uma tabela escrita à
    /// mão com outra sintaxe*, e ela mente em silêncio no dia em que a lista crescer.
    #[must_use]
    pub fn indice(self) -> usize {
        Self::ALL.iter().position(|d| *d == self).unwrap_or(0)
    }

    /// ⭐⭐⭐ **O `delta` QUE A LEI RECEBE** — a magnitude com o sinal desta direcção.
    ///
    /// ⚠️⚠️ **Ela é uma PORTA com um chamador só, e isso é de propósito.** A composição
    /// *«magnitude × direcção»* é a lei que a ordem do dono criou; escrita como um `if` dentro do
    /// despacho da shell, ela ficaria num sítio onde nenhum teste lhe chega — que é exactamente
    /// como a escolha do alvo do pincel viveu até 19/09. *Uma lei que só existe num laço de input
    /// é uma lei que ninguém pode contradizer.*
    ///
    /// ⚠️ **A magnitude entra em valor ABSOLUTO** — ela é *quanto*, e um *quanto* negativo não quer
    /// dizer nada. Ver [`WEIGHT_AMOUNT_DEFAULT`] para o que a porta do painel faz com um número
    /// negativo escrito à mão.
    #[must_use]
    pub fn delta(self, magnitude: f64) -> f64 {
        let m = magnitude.abs();
        match self {
            Self::Add => m,
            Self::Subtract => -m,
        }
    }
}

/// ⭐⭐⭐ **COMO A PINCELADA ATRIBUI O PESO** — ordem do dono, 2026-09-19: *«precisamos de 2 modos de
/// atribuir peso aos pontos»*.
///
/// ⚠️⚠️ **A diferença NÃO é de UI — é do modelo de dados**, e está na
/// [`ph2d_skeleton::Especie`]: uma mancha cumulativa SOMA (duas sobrepostas acumulam-se por
/// construção) e uma absoluta FIXA (duas sobrepostas não se somam — vence a última pintada). É por
/// isso que este enum não é uma lente do painel: ele escolhe **que espécie de mancha nasce**.
///
/// ⛔ **No modo absoluto a [`WeightDirection`] fica sem sujeito** — *«para que lado»* não tem
/// resposta quando o gesto põe um valor —, e o painel ESCONDE-a. É a lei da casa (*esconde-se o que
/// se pode; diz-se a razão onde uma cerca de produto proíbe esconder*), e aqui nada proíbe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeightMode {
    /// **CUMULATIVO** — cada pincelada soma (ou tira) o *Brush Strength* ao peso do osso em foco.
    ///
    /// É o valor de fábrica: é a lei que já existia, a que o dono aprovou em smoke, e a que um
    /// artista espera de um pincel sem ter escolhido nada.
    #[default]
    Cumulative,
    /// **ABSOLUTO** — o *Brush Strength* é posto **imediatamente** no osso em foco, e o que sobra
    /// (`1 − v`) reparte-se pelos outros ossos daquele ponto mantendo a proporção entre eles.
    Absolute,
}

impl WeightMode {
    /// Os dois, na ordem em que o segmento os mostra. ⛔ Fonte única da iteração.
    pub const ALL: [WeightMode; 2] = [WeightMode::Cumulative, WeightMode::Absolute];

    /// ⭐⭐ **O ÍNDICE deste modo em [`Self::ALL`]** — a porta que o segmento do painel acende.
    ///
    /// ⚠️ Derivado da lista, pela mesma razão do [`WeightDirection::indice`].
    #[must_use]
    pub fn indice(self) -> usize {
        Self::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }

    /// ⭐⭐⭐ **A MANCHA QUE ESTA PINCELADA ESCREVE** — a porta que junta o modo, a magnitude e a
    /// direcção numa [`ph2d_skeleton::Especie`].
    ///
    /// ⚠️⚠️ **Ela é UMA e vive aqui, ao lado da [`WeightDirection::delta`] que substitui** — pela
    /// mesma razão escrita lá: *uma lei que só existe num laço de input é uma lei que ninguém pode
    /// contradizer*. ⛔ No modo absoluto a `direccao` é **ignorada**, e isso é a lei e não um
    /// esquecimento: o painel esconde os dois botões exactamente porque eles não têm sujeito aqui.
    ///
    /// ⚠️ **A magnitude entra em valor ABSOLUTO nos dois modos** — ela é *quanto*/*quanto vale*, e
    /// o `Alvo` é coagido a `0..1` pela própria lei ([`ph2d_skeleton::Skin::fixa`]).
    #[must_use]
    pub fn especie(self, magnitude: f64, direccao: WeightDirection) -> ph2d_skeleton::Especie {
        match self {
            Self::Cumulative => ph2d_skeleton::Especie::Soma(direccao.delta(magnitude)),
            Self::Absolute => ph2d_skeleton::Especie::Alvo(magnitude.abs()),
        }
    }
}

/// ⭐ **O RAIO de fábrica do pincel de peso, em PÍXEIS DE ECRÃ.**
///
/// ⛔⛔⛔ **Ele era `20.0` em unidades de MUNDO, e o report do dono mediu o que isso vale**
/// (2026-09-19, *«os pontos não ficam coloridos»*): na barra laranja da cena, a ppm `100`,
///
/// | grandeza | mundo | píxeis |
/// |---|---|---|
/// | dois pontos vizinhos da peça | `0,2761` | `27,6` |
/// | a peça INTEIRA, ponta a ponta | `7,0218` | `702,2` |
/// | **o raio de fábrica de então** | `20,0` | **`2 000`** |
///
/// ⇒ o pincel de fábrica era **`2,85 ×` a peça inteira**: um clique agarrava TODOS os pontos dela
/// ao mesmo tempo, e o anel ficava maior que a janela. *Um pincel que não consegue apontar a um
/// sítio não é um pincel.*
///
/// ⛔⛔ **E a premissa do doc anterior era o defeito:** ele dizia que o número tinha de ser *«visível
/// numa forma do tamanho das que o app desenha»* — mas **um default em unidades de MUNDO não pode
/// saber a escala da cena** (um braço de `6` metros e outro de `600` pedem raios `100 ×`
/// diferentes). O que o artista percebe é o tamanho do pincel **no ecrã**, e é essa a única
/// grandeza que um valor de fábrica pode fixar. ⇒ o raio passa a ser de ECRÃ, e é convertido a
/// mundo no sítio onde é usado — o mesmo idioma do `hit_r` do pick desta casa.
///
/// ⚠️ **O número sai da tabela acima:** `4 ×` o raio de pick da casa (`10` px — abaixo disso o gesto
/// lê-se como apontar, não pintar), `1,45 ×` a distância entre dois pontos vizinhos (logo uma
/// pincelada apanha uma VIZINHANÇA e não um ponto só) e `5,7 %` da peça (logo ela aponta).
pub const WEIGHT_RADIUS_DEFAULT: f64 = 40.0; // LITERAL-PX-OK: raio de ecrã, tabela medida acima

/// ⭐ **QUANTO cada pincelada empurra o peso, de fábrica** — uma MAGNITUDE, em `0..1`.
///
/// ⚠️ **Pequeno de propósito:** o peso vive em `0..1` e a pincelada acumula, logo o artista chega ao
/// extremo insistindo. *Um valor de fábrica que salta para o extremo numa pincelada faz o gesto ser
/// um interruptor.*
///
/// ⛔⛔ **Ele deixou de ter SINAL em 2026-09-19** (ordem do dono — ver [`WeightDirection`]): para
/// que lado a pincelada empurra é agora dois botões. ⚠️ **E um número negativo escrito à mão entra
/// em valor ABSOLUTO, nunca cortado a zero:** cortá-lo deixaria o pincel **inerte e calado**, que é
/// a espécie de defeito que esta casa lê como *«a ferramenta não funciona»*. O campo é re-semeado
/// do estado da ferramenta a cada quadro, logo a tela mostra de volta a magnitude que ela usa — *o
/// ecrã corrige-se à vista em vez de guardar um número que ninguém honra*.
pub const WEIGHT_AMOUNT_DEFAULT: f64 = 0.15;

/// ⭐ **O PISO do raio do pincel de peso, em PÍXEIS DE ECRÃ** — ver [`WEIGHT_RADIUS_DEFAULT`].
///
/// ⚠️ **O recurso é o ECRÃ:** abaixo de um píxel o anel deixa de ser desenhável e o pen-down nunca
/// acha arte — e a recusa (`ForaDaArte`) lê-se exactamente como um pincel partido.
pub const WEIGHT_RADIUS_MIN: f64 = 1.0; // LITERAL-PX-OK: piso de ecrã, um píxel

#[cfg(test)]
mod direccao_do_peso_tests {
    use super::WeightDirection;

    /// ⭐⭐⭐ **A DIRECÇÃO COMPÕE A MAGNITUDE COM O SINAL DELA** — a lei que a ordem do dono criou
    /// (2026-09-19), medida na porta e não num `if` de um laço de input.
    ///
    /// ⚠️ **A 3.ª metade é a que impede o pincel de ficar INERTE:** uma magnitude negativa escrita
    /// à mão entra em valor absoluto, logo ela empurra com a mesma força **para o lado que os
    /// botões dizem** — nunca `0`. *Cortar a zero devolveria um pincel que não faz nada e não diz
    /// porquê, que é a família de reports que esta casa já pagou três vezes.*
    #[test]
    fn a_direccao_compoe_a_magnitude_com_o_sinal_dela() {
        assert!(
            (WeightDirection::Add.delta(0.15) - 0.15).abs() < 1e-12,
            "Add deixou de SOMAR"
        );
        assert!(
            (WeightDirection::Subtract.delta(0.15) + 0.15).abs() < 1e-12,
            "Subtract deixou de TIRAR"
        );
        for lado in WeightDirection::ALL {
            let d = lado.delta(-0.4);
            assert!(
                (d.abs() - 0.4).abs() < 1e-12,
                "{lado:?}: uma magnitude negativa deixou de entrar em ABSOLUTO ({d}) — o pincel \
                 fica inerte e calado"
            );
            assert_eq!(
                d.is_sign_negative(),
                lado == WeightDirection::Subtract,
                "{lado:?}: quem manda no sinal deixou de ser o BOTAO"
            );
        }
    }

    /// ⭐⭐ **O ÍNDICE SAI DA LISTA** — a porta que o segmento do painel acende.
    ///
    /// ⚠️ **As duas metades:** cada lado acende o seu, e a lista tem exactamente a população do
    /// enum. *Sem a segunda, uma variante nova fora do `ALL` leria `0` e acenderia o primeiro
    /// segmento — é o defeito que o [`super::BoneAction::indice`] já pagou com três verbos.*
    #[test]
    fn o_indice_da_direccao_sai_da_lista() {
        for (i, lado) in WeightDirection::ALL.iter().enumerate() {
            assert_eq!(lado.indice(), i, "{lado:?} acende o segmento errado");
        }
        let mut vistos = 0usize;
        for lado in WeightDirection::ALL {
            vistos += match lado {
                WeightDirection::Add | WeightDirection::Subtract => 1,
            };
        }
        assert_eq!(
            vistos,
            WeightDirection::ALL.len(),
            "a lista e o enum deixaram de contar a mesma populacao"
        );
    }

    /// ⭐ **DE FÁBRICA ELA SOMA** — é o que o artista faz primeiro, e o que ele espera sem ter
    /// escolhido nada.
    #[test]
    fn de_fabrica_a_pincelada_soma() {
        assert_eq!(WeightDirection::default(), WeightDirection::Add);
    }
}

#[cfg(test)]
mod modo_do_peso_tests {
    use super::{WeightDirection, WeightMode};
    use ph2d_skeleton::Especie;

    /// ⭐⭐⭐ **O MODO ESCOLHE A ESPÉCIE DA MANCHA** — a porta que junta as três respostas do painel
    /// (modo · magnitude · direcção) numa só coisa (F29, ordem do dono de 2026-09-19).
    ///
    /// ⚠️ **As duas metades dizem coisas diferentes:** no cumulativo a direcção **manda no sinal**
    /// (é a lei que a wave anterior trouxe); no absoluto ela é **ignorada**, e isso é a lei e não um
    /// esquecimento — *«para que lado»* não tem resposta quando o gesto põe um valor. É por isso que
    /// o painel esconde os dois botões ali.
    #[test]
    fn o_modo_escolhe_a_especie_da_mancha() {
        assert_eq!(
            WeightMode::Cumulative.especie(0.15, WeightDirection::Add),
            Especie::Soma(0.15),
            "o modo cumulativo deixou de produzir uma mancha que SOMA"
        );
        assert_eq!(
            WeightMode::Cumulative.especie(0.15, WeightDirection::Subtract),
            Especie::Soma(-0.15),
            "no modo cumulativo a direccao deixou de mandar no sinal"
        );
        for lado in WeightDirection::ALL {
            assert_eq!(
                WeightMode::Absolute.especie(0.6, lado),
                Especie::Alvo(0.6),
                "{lado:?}: a direccao chegou ao modo ABSOLUTO, onde ela nao tem sujeito"
            );
        }
    }

    /// ⚠️ **A magnitude entra em ABSOLUTO nos dois modos** — ela é *quanto*/*quanto vale*, e um
    /// número negativo escrito à mão na caixa não pode deixar o pincel inerte nem pedir um peso
    /// negativo. *É a mesma cerca que a [`WeightDirection::delta`] já declara, e ela vale para a
    /// espécie nova pela mesma razão.*
    #[test]
    fn uma_magnitude_negativa_entra_em_absoluto_nos_dois_modos() {
        assert_eq!(
            WeightMode::Absolute.especie(-0.6, WeightDirection::Add),
            Especie::Alvo(0.6),
            "um alvo negativo deixou de ser lido em valor absoluto"
        );
        assert_eq!(
            WeightMode::Cumulative.especie(-0.15, WeightDirection::Add),
            Especie::Soma(0.15),
            "o modo cumulativo deixou de ler a magnitude em absoluto"
        );
    }

    /// ⭐⭐ **O ÍNDICE SAI DA LISTA, e a lista tem a população do enum** — a mesma lei (e o mesmo
    /// defeito medido) do [`WeightDirection::indice`].
    #[test]
    fn o_indice_do_modo_sai_da_lista() {
        for (i, modo) in WeightMode::ALL.iter().enumerate() {
            assert_eq!(modo.indice(), i, "{modo:?} acende o segmento errado");
        }
        let mut vistos = 0usize;
        for modo in WeightMode::ALL {
            vistos += match modo {
                WeightMode::Cumulative | WeightMode::Absolute => 1,
            };
        }
        assert_eq!(
            vistos,
            WeightMode::ALL.len(),
            "a lista e o enum deixaram de contar a mesma populacao"
        );
    }

    /// ⭐ **DE FÁBRICA ELE É CUMULATIVO** — é a lei que já existia e a que o dono aprovou em smoke.
    ///
    /// ⚠️ **Isto é o que mantém um rig já autorado com a mesma aparência:** toda mancha nova nasce
    /// `Soma`, logo a lei corre pelo caminho de sempre até o artista escolher o outro modo.
    #[test]
    fn de_fabrica_o_modo_e_cumulativo() {
        assert_eq!(WeightMode::default(), WeightMode::Cumulative);
    }
}
