//! ⭐ **O QUE UM VERBO É** — os predicados que o resto do módulo consulta em vez
//! de manter listas paralelas.
//!
//! ⚠️ **Cada um destes é uma PORTA, e é isso que os justifica:** a alternativa é
//! uma lista de verbos escrita à mão em cada sítio que precisa da resposta, e
//! essa lista envelhece calada no dia em que um verbo novo nasce. *Uma lei
//! escrita em dois sítios ainda não é uma lei — só uma porta é.*
//!
//! Irmão do [`super::brush_verb`], e o corte é *o que o verbo É* (aqui) contra
//! *quais verbos existem e como se chamam* (lá).

use super::verb::Verb;
use crate::grip::Grip;

impl Verb {
    /// **Este verbo pode ACUMULAR?** — a porta única do `accumulate`.
    ///
    /// Só a família do CARIMBO. Os outros três grips carregam o gesto TOTAL
    /// desde o pen-down (o puxão, o ângulo varrido, a fração de escala) e
    /// carimbam `accum = 1` ou congelam a pegada: somar um total N vezes seria
    /// multiplicar o gesto pelo número de eventos de ponteiro, que é exatamente
    /// a dependência de taxa de amostragem que a lei do traço existe para não
    /// ter.
    ///
    /// ⚠️ Porta e não um `matches!` no sítio de uso: o painel pergunta para
    /// OFERECER o interruptor e o aplicador pergunta para HONRAR o clique, e
    /// duas cópias divergiriam num controle que aparece e não faz nada.
    ///
    /// ⚠️ **A DEMÃO fica de fora, e é a referência que a tira:** o *Layer*
    /// mede as distâncias contra as posições ORIGINAIS do pen-down
    /// **incondicionalmente** — ele não consulta o *Accumulate*, ao contrário
    /// dos irmãos de
    /// carimbo. E há razão para isso: o que o Accumulate compra num Draw é
    /// *deixar o pincel não se esgotar*, e a demão já tem o próprio motor de
    /// saturação no [`crate::GripLaw::coat`]. Oferecer o interruptor aqui seria
    /// um segundo controle sobre a mesma pergunta.
    #[must_use]
    pub fn accumulates(self) -> bool {
        matches!(self.grip(), Grip::Stamp)
            && self != Self::Layer
            // ⛔ **E a DENSIDADE, que é carimbo e não tem o que acumular:** ela
            // não move um vértice. Oferecer o interruptor seria um controlo que
            // aparece e não faz nada.
            && !self.sem_lei_por_vertice()
    }

    /// Este verbo escreve na MÁSCARA em vez da posição?
    ///
    /// Porta única: o aplicador pergunta para saber onde escrever, e a UI
    /// perguntará para saber que knobs oferecer. Duas listas divergiriam no dia
    /// em que entrar o segundo verbo de canal (Paint, na W7).
    #[must_use]
    pub fn paints_mask(self) -> bool {
        matches!(self, Self::Mask)
    }

    /// O sinal (o `Ctrl` de todo app de escultura) muda o RESULTADO deste verbo?
    ///
    /// ⚠️ **Era uma blacklist, e ela MENTIA.** Ao excluir só `Smooth`/`Sharpen` e
    /// `Pinch`/`Magnify`, ela afirmava sinal para `Flatten`, `Fill` e `Scrape` —
    /// e o `invert` **nunca chega neles**: o alvo dos três é `project(base,
    /// plane)` (`stroke.rs:410-424`), que não lê o `reach`, o único canal por
    /// onde o sinal viaja até um verbo de posição. Três controles mortos, com uma
    /// função afirmando que estavam vivos.
    ///
    /// A lista verdadeira é a de quem CONSOME o sinal: `Draw`, `Inflate` e
    /// `Clay` somam `reach` (`stroke.rs:397,398,427`), `Crease` soma `-reach`
    /// (`:435`), e `Mask` troca o alvo do canal dele de 1 para 0
    /// (`apply_mask`, `:481`).
    ///
    /// ⚠️ **Whitelist e não blacklist, e a direção é o conserto.** Numa
    /// blacklist um verbo NOVO nasce reivindicando um sinal que talvez não tenha,
    /// em silêncio — que é exatamente como este defeito nasceu. Numa whitelist
    /// ele nasce sem sinal, e quem o tem escreve o nome aqui.
    ///
    /// ⚠️ **Isto NÃO é um `uses_reach()`.** O `Mask` não lê `reach` — o alvo de
    /// posição dele é o próprio lugar — e mesmo assim tem oposto. A pergunta é
    /// sobre *o resultado que o artista vê*, e `reach` e `apply_mask::goal` são
    /// duas implementações dela.
    ///
    /// **As três alternativas, e por que cada uma morre:** *"faça o invert
    /// funcionar no Flatten"* — não há o que negar, o Flatten projeta nos dois
    /// sentidos e o oposto dele é ele mesmo; *"Ctrl troca Fill↔Scrape"* — é o
    /// `_negative` do `Flatten.js`, mas ele tem UM tool com um toggle e nós temos
    /// DOIS verbos com dois chips, então o rail destacaria "Fill" enquanto a
    /// ferramenta raspa; *"Ctrl nega o `plane_offset`"* — o slider já tem sinal
    /// nos dois sentidos, com gate provando
    /// (`the_plane_offset_lifts_the_plane_the_verbs_project_onto`).
    ///
    /// ⚠️ **Nenhuma UI pergunta isto hoje** (o shell arma `invert = ctrl`
    /// incondicionalmente, `sculpt3d/mod.rs`): o consumidor é o [`Brush::reach`], e o
    /// chip que decide oferecer ou não o controle é da wave que trouxer painel.
    ///
    /// ⚠️ **Os dois [`Grip::Turn`] ficam de fora, e a razão é que o gesto já tem
    /// sinal:** varrer para o outro lado torce ao contrário, arrastar para a
    /// esquerda encolhe. Um `Ctrl` ali seria a segunda maneira de dizer a mesma
    /// coisa — e uma que **compõe** com a primeira, então varrer ao contrário
    /// com `Ctrl` apertado voltaria a torcer no sentido original.
    #[must_use]
    pub fn honours_invert(self) -> bool {
        matches!(
            self,
            Self::Draw
                | Self::Inflate
                | Self::Clay
                | Self::Crease
                | Self::Blob
                | Self::Mask
                // ⚠️ **O Ctrl VIRA O V**, e o oposto de cavar um vinco é
                // enchê-lo: com o ângulo negativo as duas normais tombam ao
                // contrário, o telhado vira vale e o culling de lado se desliga
                // (`if (angle >= 0.0f)`). É o `if (flip) angle *= -1` do
                // *Multiplane Scrape*, e não uma força negativa.
                | Self::MultiplaneScrape
                // ⚠️ **A DEMÃO cava, e é a direcção do pincel da referência** —
                // lá o sinal viaja **dentro da força efectiva**, que a
                // referência já entrega negativa. Aqui ele viaja no alvo (o
                // `sign` do `compute_target`), porque o nosso `accum` é a
                // MAGNITUDE da demão e uma magnitude não tem lado.
                | Self::Layer
                // ⚠️ **O TECIDO honra o Ctrl desde 2026-09-06**, quando a lei da
                // referência passou a ser a de omissão: ela carrega um sinal
                // `±1` que multiplica a força do gesto (o *Add/Subtract* do
                // alvo), e o adaptador lê-o do `Brush::invert`. ⛔ A lei VBD
                // anterior não o lia, e é por isso que esta linha não existia.
                | Self::Cloth
                // ⭐⭐ **A POSE honra o Ctrl, e de uma maneira que nenhum outro
                // verbo desta lista usa: ele NÃO nega a força — ele TROCA DE
                // DEFORMAÇÃO.** Girar vira torcer, escalar vira transladar. ⚠️ E
                // num dos três modos (espremer/esticar) ele **não muda nada**,
                // porque ali não há segunda deformação para escolher — medido:
                // a fixtura invertida é idêntica **ao bit** à normal.
                // ⇒ *«honra o Ctrl» aqui quer dizer «o Ctrl é observável», e é
                // exactamente isso que este predicado pergunta.*
                | Self::Pose
                // ⭐ **O contorno honra o Ctrl, e também não é uma negação de
                // força:** ele **ENCAIXA o ângulo** em décimos, nos dois modos
                // que rodam. Medido no corpus: o mesmo traço curto dá um valor
                // **diferente e MAIOR** com a inversão, que é a assinatura de
                // uma truncagem sobre um factor negativo.
                | Self::Boundary
        )
    }

    /// Este verbo ajusta um plano à pegada do dab? (Quem responde `true` usa o
    /// knob `plane_offset`.)
    #[must_use]
    pub fn uses_plane(self) -> bool {
        matches!(self, Self::Flatten | Self::Fill | Self::Scrape | Self::Clay)
    }

    /// Este verbo lê o anel de vizinhos? (Quem responde `true` custa a
    /// travessia do CSR por vértice, e é o que decide se o `vert_verts` pode um
    /// dia virar preguiçoso.)
    ///
    /// ⚠️ **O [`Self::SurfaceSmooth`] o percorre DUAS vezes** — uma para a média
    /// das posições, outra para a média dos `b` —, e ele nasceu FORA desta
    /// lista: a pergunta que ela responde é *quem precisa da adjacência*, e uma
    /// resposta falsa aqui é como um `vert_verts` preguiçoso deixaria de
    /// construí-la exatamente para o verbo que mais a usa. O gate irmão
    /// `the_families_that_the_ui_asks_about_agree_with_the_verb_list` enumera
    /// os nomes, e ficou VERDE sobre a omissão até alguém a procurar.
    #[must_use]
    pub fn uses_neighbours(self) -> bool {
        matches!(self, Self::Smooth | Self::Sharpen | Self::SurfaceSmooth)
    }

    /// **ESTE VERBO PASSA PELO APLICADOR POR-VÉRTICE?**
    ///
    /// ⚠️ **A porta existe porque uma feature nova pode ESVAZIAR o censo de
    /// outra pessoa**, e o tecido é a primeira que o faz: os censos do
    /// `stroke_apply` varrem `Verb::ALL` e afirmam coisas sobre o `accum` e o
    /// `target` — dois planos que a simulação **nunca escreve**, porque ela
    /// desvia antes do laço. Excluí-lo por NOME faria cada verbo novo editar o
    /// teste de outra pessoa; excluí-lo por LEI é o que mantém a população dos
    /// censos derivada.
    ///
    /// ⛔ *Nada aqui diz que o tecido é uma excepção tolerada:* ele tem os
    /// próprios gates, no `stroke_cloth_tests`, e eles medem o que ele de facto
    /// promete.
    ///
    /// ⚠️ **E ela nasceu SEPARANDO um `#[must_use]` do item dele** — a primeira
    /// redação foi inserida entre o atributo do `uses_neighbours` e a assinatura
    /// dele, e o atributo mudou de dono em silêncio. É a armadilha que este repo
    /// já tinha registada; quem a apanhou foi o clippy, não uma leitura.
    /// ⚠️⚠️ **E ela passou a ter DUAS metades em 2026-09-14**, quando chegou um
    /// verbo que não desvia para lei nenhuma — ele simplesmente **não tem lei
    /// por-vértice** ([`Self::sem_lei_por_vertice`]). *A pergunta que os censos
    /// fazem é «há aqui um aplicador por-vértice a julgar?», e há duas maneiras
    /// de a resposta ser não.* Os seis censos que reprovaram ao ele nascer
    /// disseram-no à letra — *«dab inerte»*, *«não tocou nada»*, *«o dab não fez
    /// nada em canal nenhum»* —, e cada um deles é um piso de população a fazer
    /// o trabalho dele.
    #[must_use]
    pub fn writes_through_applicator(self) -> bool {
        !self.resolve_a_propria_regiao() && !self.sem_lei_por_vertice()
    }

    /// **Este verbo DESVIA antes do laço por-vértice e é dono da própria
    /// região?**
    ///
    /// ⭐⭐ **É a porta ÚNICA da pergunta, e ela nasceu porque havia duas
    /// respostas.** O desvio no `stroke_symmetry` nomeava verbos à mão enquanto
    /// os censos do `stroke_apply` perguntavam ao [`Grip`] — e as duas
    /// concordavam **por acaso**, enquanto o único que desviava era a simulação.
    /// A pose quebrou o acaso: ela desvia e o grip dela é o [`Grip::Hold`], que
    /// dois verbos do laço também usam. *Duas respostas à mesma pergunta
    /// divergem no dia do terceiro caso, e este foi esse dia.*
    ///
    /// ⚠️ **Os dois que respondem `true` fazem-no por razões DIFERENTES**, e
    /// isso é o que impede de derivar esta resposta do grip:
    /// - o **tecido** tem um solver, e cada cópia de simetria tem a sua região —
    ///   duas regiões em lados opostos da peça não partilham vértice nenhum;
    /// - a **pose** tem uma cadeia de mapas afins que **já resolve os oito
    ///   octantes de espelho numa passagem só**, e não tem atenuação radial
    ///   nenhuma: a região dela cresce pela **ligação** da malha, não pelo raio.
    ///
    /// ⛔ *Nada aqui diz que eles são excepções toleradas:* cada um tem os
    /// próprios gates — `stroke_cloth_tests` e a bancada da `ph2d-pose` — e eles
    /// medem o que cada um de facto promete.
    #[must_use]
    pub fn resolve_a_propria_regiao(self) -> bool {
        matches!(self, Self::Cloth | Self::Pose | Self::Boundary)
    }

    /// **Este verbo REFINA a malha em Dynamic Topology?**
    ///
    /// ⛔⛔ **ESTA RESPOSTA NÃO É O VEREDITO DO ESTUDO** (`docs/3D/22`). Ela é,
    /// verbo a verbo, o **comportamento de hoje** — mais **uma** correcção que
    /// não depende de oráculo nenhum: a **MÁSCARA**.
    ///
    /// # ⭐ Porque a máscara pode ser curada antes do estudo
    ///
    /// Ela pinta um canal por-vértice e **não move um único vértice**. Refinar
    /// debaixo dela muda a topologia da peça num gesto que não toca na
    /// geometria, e o custo é pago por um artista que só queria proteger uma
    /// zona. *Um gesto que não escreve posição não tem porque mudar a
    /// topologia* — e essa frase não precisa de saber o que outro programa faz.
    ///
    /// # ⚠️ O que o `false` dos verbos com ÂNCORA significa aqui
    ///
    /// Hoje eles **nem chegam à porta**: o refino tem um chamador só, o braço do
    /// carimbo, e quem tem âncora entra por outro caminho (`take_hold`,
    /// `hook_step`, `cloth_step`). O `false` deles é, portanto, **o estado
    /// actual e o valor conservador**: se alguém ligar a porta aos gestos
    /// ancorados antes de o estudo existir, nada muda em silêncio. ⛔ Ele **não**
    /// é a afirmação de que a pose ou o tecido não devem refinar — essa é
    /// exactamente a pergunta aberta, e são eles que mais **esticam** superfície.
    ///
    /// ⚠️ **Existe uma segunda coluna** ([`Self::colapsa_no_dyntopo`]) porque o
    /// refino e o colapso são **leis independentes** que por acaso vivem na
    /// mesma porta: um verbo pode querer relaxar densidade sem criar detalhe.
    /// *Uma tabela com uma coluna só obriga quem a lê a escolher por ela.*
    /// **Este verbo NÃO TEM LEI POR-VÉRTICE?** — a porta que tira a densidade
    /// do `dab_core`.
    ///
    /// ⭐ Irmão do [`Self::resolve_a_propria_regiao`] e o corte é claro: aqueles
    /// têm lei própria **noutro sítio** (um solver, uma cadeia de mapas, uma
    /// borda); este **não tem lei nenhuma sobre posições**. Todo o efeito dele é
    /// sobre o passe de TOPOLOGIA.
    ///
    /// ⚠️ **Porta e não um `matches!` no sítio de uso, pela razão dos irmãos:**
    /// o desvio do traço pergunta para NÃO CORRER a cadeia de peso, e o painel
    /// pergunta para não oferecer os controlos dela. Duas cópias divergiriam num
    /// pincel que mostra uma força que ninguém lê.
    #[must_use]
    pub fn sem_lei_por_vertice(self) -> bool {
        matches!(self, Self::Density)
    }

    #[must_use]
    pub fn refina_no_dyntopo(self) -> bool {
        // ⛔ **A DENSIDADE NUNCA ACRESCENTA SUPERFÍCIE, e é lei e não omissão:**
        // ela liga o colapso e **não** liga o partir. *Um pincel que também
        // subdividisse é outro produto*, e é por isso que esta linha existe
        // separada da de baixo — as duas colunas deixam de coincidir aqui.
        self != Self::Density && !self.anchors() && self != Self::Mask
    }

    /// **Este verbo COLAPSA arestas curtas em Dynamic Topology?**
    ///
    /// A segunda metade do [`Self::refina_no_dyntopo`], e ela existe separada
    /// pela razão escrita lá. ⚠️ **Hoje as duas colunas coincidem em todos os
    /// verbos**, e isso é um facto sobre o produto de hoje (as duas leis vivem
    /// numa porta só), **não** uma lei — o estudo pode separá-las.
    #[must_use]
    pub fn colapsa_no_dyntopo(self) -> bool {
        !self.anchors() && self != Self::Mask
    }
}

/// **O CENSO DAS DUAS COLUNAS DO DYNTOPO** — o que esta tabela afirma HOJE.
#[cfg(test)]
mod dyntopo_tests {
    use super::Verb;

    /// ⭐⭐⭐ **SÓ DOIS VERBOS SE AFASTAM DO COMPORTAMENTO DE HOJE, E POR RAZÕES
    /// DE ESPÉCIE DIFERENTE.**
    ///
    /// ⚠️⚠️ **É isto que impede a tabela de virar um palpite disfarçado de lei.**
    /// O que cada verbo *deve* fazer é pergunta de **ORÁCULO**
    /// (`docs/3D/22_plano_quem_subdivide_no_dyntopo.md`), e este censo afirma
    /// que os únicos desvios são:
    ///
    /// | verbo | as duas colunas | que espécie de entrada isto é |
    /// |---|---|---|
    /// | **Mask** | `false` · `false` | ⛔ **uma CORRECÇÃO** — ele não move um vértice, e adensava a malha (`830 → 1 331`) |
    /// | **Density** | `false` · **`true`** | ⭐ **a LEI de um verbo novo** — ele existe para colapsar, e nunca acrescenta |
    ///
    /// ⚠️ **As duas espécies leem-se iguais numa lista** — *«dois verbos fora do
    /// padrão»* —, e não são a mesma coisa: apagar a primeira é reabrir um
    /// defeito, apagar a segunda é apagar um pincel. É por isso que este gate as
    /// nomeia **em separado** em vez de contar duas.
    ///
    /// ⇒ o dia em que o estudo existir, é **este gate** que muda, célula a
    /// célula, e a mudança fica visível no diff.
    #[test]
    fn so_dois_verbos_se_afastam_do_comportamento_de_hoje() {
        let mut corrigidos = Vec::new();
        for v in Verb::ALL {
            // O comportamento de HOJE: o refino tem um chamador só — o braço do
            // carimbo —, e quem tem âncora entra por outro caminho.
            let chega_a_porta = !v.anchors();
            if v.refina_no_dyntopo() != chega_a_porta || v.colapsa_no_dyntopo() != chega_a_porta {
                corrigidos.push(v.label());
            }
        }
        assert_eq!(
            corrigidos,
            ["Mask", "Density"],
            "esta tabela mudou o comportamento de um verbo além da máscara e da \
             densidade. Se foi o ESTUDO a chegar, reescreva este gate célula a \
             célula com a tabela medida ao lado; se não foi, é uma regressão"
        );
        // ⚠️ **E as DUAS espécies, em separado** — ver a tabela no doc: a
        // máscara é uma correcção (ela não podia adensar), a densidade é a LEI
        // de um pincel (ele existe para colapsar).
        assert!(
            !Verb::Mask.refina_no_dyntopo() && !Verb::Mask.colapsa_no_dyntopo(),
            "a máscara não pode mexer na topologia por nenhuma das metades"
        );
        assert!(
            !Verb::Density.refina_no_dyntopo() && Verb::Density.colapsa_no_dyntopo(),
            "a densidade LIGA o colapso e NÃO liga o partir — um pincel que \
             também subdividisse é outro produto"
        );
    }

    /// ⚠️ **O `false` dos verbos com ÂNCORA é o valor CONSERVADOR, não uma
    /// afirmação.** Hoje eles nem chegam à porta; se alguém a ligar aos gestos
    /// ancorados antes de o estudo existir, **nada muda em silêncio**.
    ///
    /// ⛔ E eles são precisamente os que mais **esticam** superfície (a pose roda
    /// um membro inteiro), logo são os candidatos mais fortes a mudar de valor
    /// quando a tabela for medida — este gate existe para essa mudança ser
    /// deliberada.
    #[test]
    fn nenhum_verbo_com_ancora_refina_ou_colapsa_hoje() {
        let ancorados: Vec<&str> = Verb::ALL
            .into_iter()
            .filter(|v: &Verb| v.anchors())
            .map(Verb::label)
            .collect();
        assert!(
            ancorados.len() >= 8,
            "o censo varreu só {} verbos com âncora — a varredura partiu-se: {ancorados:?}",
            ancorados.len()
        );
        for v in Verb::ALL.into_iter().filter(|v| v.anchors()) {
            assert!(
                !v.refina_no_dyntopo() && !v.colapsa_no_dyntopo(),
                "`{}` tem âncora e declara que mexe na topologia — ele nem chega \
                 à porta hoje, então isto muda comportamento sem o estudo",
                v.label()
            );
        }
    }

    /// ⭐⭐⭐ **AS DUAS COLUNAS SEPARAM-SE EM EXACTAMENTE UM VERBO: a DENSIDADE.**
    ///
    /// ⚠️ **Este gate nasceu a dizer o contrário**, e a mudança é o registo: até
    /// 14/09 elas coincidiam em todos os verbos, e ele dizia *«isso é um facto
    /// sobre o produto de hoje, não uma lei — este gate reprova no dia em que
    /// alguém as separar»*. O dia chegou, e não foi o estudo: foi o pincel de
    /// **densidade**, que **liga o colapso e não liga o partir**.
    ///
    /// ⭐ *Um gate escrito para reprovar no dia em que a premissa dele morre é o
    /// que torna a morte dela visível no diff* — em vez de a coincidência se
    /// desfazer num `match` que ninguém recontou.
    #[test]
    fn as_duas_colunas_separam_se_em_exactamente_um_verbo() {
        let separados: Vec<&str> = Verb::ALL
            .into_iter()
            .filter(|v| v.refina_no_dyntopo() != v.colapsa_no_dyntopo())
            .map(Verb::label)
            .collect();
        assert_eq!(
            separados,
            ["Density"],
            "as duas colunas separaram-se noutro verbo. Se foi o ESTUDO a \
             chegar, reescreva este gate com a tabela medida ao lado; se não \
             foi, é uma regressão"
        );
    }
}
