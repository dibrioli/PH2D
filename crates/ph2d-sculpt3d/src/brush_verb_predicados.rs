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
    ///
    /// ⚠️⚠️ **E este doc-comment esteve COLADO ao do vizinho de baixo até
    /// 2026-09-14** — a declaração nasceu no meio do bloco do
    /// [`Self::refina_no_dyntopo`], que ficou **sem doc nenhum** enquanto este
    /// carregava os dois. É a armadilha que o cabeçalho deste ficheiro já
    /// regista uma vez, agora com a segunda ocorrência: *inserir um item entre
    /// um doc e o dono dele muda o dono em silêncio, e o clippy só a apanha
    /// quando o atributo fica órfão.*
    #[must_use]
    pub fn sem_lei_por_vertice(self) -> bool {
        matches!(self, Self::Density)
    }

    /// **ESTE VERBO PODE MEXER NA TOPOLOGIA DE TODO?** — a metade das duas
    /// colunas que NÃO depende de ajuste nenhum.
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
    #[must_use]
    pub fn mexe_na_topologia(self) -> bool {
        !self.anchors() && self != Self::Mask
    }

    /// **Este verbo REFINA a malha em Dynamic Topology?**
    ///
    /// ⚠️⚠️ **ELA GANHOU UM ARGUMENTO EM 2026-09-14, e a razão é uma LEITURA
    /// ERRADA MINHA que o dono apanhou pelo produto** — *«por que não pode
    /// aumentar a densidade também?»*.
    ///
    /// A primeira redacção afirmava aqui, num comentário, que *«a densidade
    /// nunca acrescenta superfície, e é lei e não omissão»*. **Não é.** A espec
    /// §3.2 traz a tabela-verdade do passe, e o que o pincel faz é
    /// **ACRESCENTAR a bandeira de colapso** ao modo — ele não **retira** a de
    /// partir, que continua a ser do ajuste de refino. A mesma malha grossa
    /// mede `81 → 81` com o ajuste em «só colapsar» e **`81 → 101`** com
    /// «partir + colapsar»: *duas leituras do mesmo pincel, com o ajuste
    /// diferente.*
    ///
    /// ⇒ o ajuste é o [`crate::DensityModo`], e ele vive **no pincel**
    /// (espec §9.8 — o alvo guarda-o na cena e tem pedido público aberto para o
    /// mudar; ⛔ não copiámos o modelo que ele está a caminho de abandonar).
    ///
    /// ⚠️ **A recusa medida da espec continua de pé, e é OUTRA:** *«fazer o
    /// `Density` também subdividir»* quer dizer o pincel **FORÇAR** o partir,
    /// como ele força o colapso. Ele não força — ele obedece.
    ///
    /// ⚠️ **O argumento é lido por UM verbo só, e é de propósito que ele não é
    /// um campo do `Verb`:** os outros 28 não têm ajuste de refino nenhum (o
    /// passe deles faz as duas metades desde que existe), e dar-lhes um seria
    /// inventar superfície que nenhuma referência declara.
    #[must_use]
    pub fn refina_no_dyntopo(self, densidade: crate::DensityModo) -> bool {
        self.mexe_na_topologia() && (self != Self::Density || densidade.parte_arestas_longas())
    }

    /// **Este verbo COLAPSA arestas curtas em Dynamic Topology?**
    ///
    /// A segunda metade do [`Self::refina_no_dyntopo`], e ela existe separada
    /// pela razão escrita lá: as duas são **leis independentes** que por acaso
    /// vivem na mesma porta — um verbo pode querer relaxar densidade sem criar
    /// detalhe. *Uma tabela com uma coluna só obriga quem a lê a escolher por
    /// ela.*
    ///
    /// ⭐ **Ela NÃO tem argumento, e a assimetria é a lei do pincel de
    /// densidade:** ele acrescenta a bandeira de colapso **aconteça o que
    /// acontecer** com o ajuste (espec §3.2, as três linhas da tabela têm
    /// *colapsar* a `sim`). É por isso que o `Afinar` não é «desligar o
    /// colapso»: não existe esse estado.
    #[must_use]
    pub fn colapsa_no_dyntopo(self) -> bool {
        self.mexe_na_topologia()
    }
}

/// **O CENSO DAS DUAS COLUNAS DO DYNTOPO** — o que esta tabela afirma HOJE.
#[cfg(test)]
mod dyntopo_tests {
    use super::Verb;
    use crate::DensityModo;

    /// ⭐⭐⭐ **A TABELA-VERDADE DO PINCEL DE DENSIDADE, célula a célula** — é a
    /// da espec §3.2, e ela reprovou a primeira redacção desta casa.
    ///
    /// ⚠️⚠️ **O gate que aqui estava afirmava o CONTRÁRIO de uma das células**
    /// (*«a densidade LIGA o colapso e NÃO liga o partir — um pincel que também
    /// subdividisse é outro produto»*), e quem o desmentiu foi o dono, pelo
    /// produto: *«por que não pode aumentar a densidade também?»*. A espec diz
    /// que o pincel **ACRESCENTA** a bandeira de colapso e não **RETIRA** a de
    /// partir — e mede as duas células na mesma malha grossa, `81 → 81` contra
    /// **`81 → 101`**.
    ///
    /// ⇒ *um gate pode pinar a leitura errada de uma espec tão bem como pina um
    /// defeito*, e o que o separa de uma medição é ninguém ter corrido a outra
    /// célula.
    #[test]
    fn a_densidade_colapsa_sempre_e_parte_conforme_o_ajuste() {
        // As três linhas da tabela da espec que têm o passe armado. A quarta
        // (*Detailing* Manual) é o dyntopo desarmado, e vive no `refine_for_dab`.
        assert!(
            Verb::Density.colapsa_no_dyntopo(),
            "o colapso é a LEI deste pincel, não um ajuste: ele acrescenta a \
             bandeira aconteça o que acontecer com o modo"
        );
        assert!(
            Verb::Density.refina_no_dyntopo(DensityModo::Igualar),
            "com o ajuste a pedir «partir + colapsar» o passe PARTE — é a \
             célula que a espec mede em `81 -> 101`"
        );
        assert!(
            !Verb::Density.refina_no_dyntopo(DensityModo::Afinar),
            "com o ajuste em «só colapsar» o passe não parte — `81 -> 81`"
        );
        // ⛔ **A recusa medida da espec, e ela é OUTRA pergunta:** o pincel não
        // pode FORÇAR o partir como força o colapso. Com o ajuste a pedir só o
        // colapso, ele obedece — e é isso que este par afirma.
        assert!(
            Verb::Density.refina_no_dyntopo(DensityModo::Afinar)
                != Verb::Density.colapsa_no_dyntopo(),
            "no modo `Afinar` as duas colunas TÊM de se separar, senão o pincel \
             está a forçar o partir — a recusa medida da espec"
        );
    }

    /// ⭐⭐⭐ **SÓ UM VERBO SE AFASTA DO COMPORTAMENTO DE HOJE, E COM O AJUSTE DE
    /// OMISSÃO ELE É A MÁSCARA.**
    ///
    /// ⚠️⚠️ **É isto que impede a tabela de virar um palpite disfarçado de lei.**
    /// O que cada verbo *deve* fazer é pergunta de **ORÁCULO**
    /// (`docs/3D/22_plano_quem_subdivide_no_dyntopo.md`), e o único desvio que
    /// não depende dele é:
    ///
    /// | verbo | as duas colunas | que espécie de entrada isto é |
    /// |---|---|---|
    /// | **Mask** | `false` · `false` | ⛔ **uma CORRECÇÃO** — ele não move um vértice, e adensava a malha (`830 → 1 331`) |
    ///
    /// ⚠️ **A densidade SAIU desta lista em 2026-09-14**, e a saída é o registo:
    /// no ajuste de omissão ([`DensityModo::Igualar`]) ela faz exactamente o que
    /// os outros 27 fazem — as duas metades. *O que a distingue não é a
    /// topologia, é ela não ter lei por-vértice.*
    ///
    /// ⇒ o dia em que o estudo existir, é **este gate** que muda, célula a
    /// célula, e a mudança fica visível no diff.
    #[test]
    fn so_a_mascara_se_afasta_do_comportamento_de_hoje() {
        let mut corrigidos = Vec::new();
        for v in Verb::ALL {
            // O comportamento de HOJE: o refino tem um chamador só — o braço do
            // carimbo —, e quem tem âncora entra por outro caminho.
            let chega_a_porta = !v.anchors();
            if v.refina_no_dyntopo(DensityModo::default()) != chega_a_porta
                || v.colapsa_no_dyntopo() != chega_a_porta
            {
                corrigidos.push(v.label());
            }
        }
        assert_eq!(
            corrigidos,
            ["Mask"],
            "esta tabela mudou o comportamento de um verbo além da máscara. Se \
             foi o ESTUDO a chegar, reescreva este gate célula a célula com a \
             tabela medida ao lado; se não foi, é uma regressão"
        );
        assert!(
            !Verb::Mask.refina_no_dyntopo(DensityModo::default())
                && !Verb::Mask.colapsa_no_dyntopo(),
            "a máscara não pode mexer na topologia por nenhuma das metades"
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
    ///
    /// ⚠️ **Ele varre os DOIS ajustes de densidade**, e não porque algum verbo
    /// com âncora seja a densidade: é a metade justa da pergunta — um ajuste que
    /// passasse a armar a topologia de um verbo ancorado entraria por aqui.
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
            for modo in DensityModo::ALL {
                assert!(
                    !v.refina_no_dyntopo(modo) && !v.colapsa_no_dyntopo(),
                    "`{}` tem âncora e declara que mexe na topologia — ele nem \
                     chega à porta hoje, então isto muda comportamento sem o estudo",
                    v.label()
                );
            }
        }
    }

    /// ⭐⭐⭐ **O AJUSTE SÓ ALCANÇA UM VERBO, E É A DENSIDADE.**
    ///
    /// ⚠️ **Ele existe porque o argumento novo é uma superfície nova**, e uma
    /// superfície que alcance verbo a mais é a forma deste módulo de partir: os
    /// outros 27 não têm ajuste de refino nenhum (o passe deles faz as duas
    /// metades desde que existe), e dar-lhes um seria inventar controlo que
    /// nenhuma referência declara.
    ///
    /// ⭐ *A prova é por VARREDURA e não por leitura:* ele troca o ajuste e
    /// recolhe **quem muda de resposta**.
    #[test]
    fn o_ajuste_de_densidade_so_alcanca_a_densidade() {
        let sensiveis: Vec<&str> = Verb::ALL
            .into_iter()
            .filter(|v| {
                v.refina_no_dyntopo(DensityModo::Igualar)
                    != v.refina_no_dyntopo(DensityModo::Afinar)
            })
            .map(Verb::label)
            .collect();
        assert_eq!(
            sensiveis,
            ["Density"],
            "o ajuste de densidade passou a mexer noutro verbo — ou ele deixou \
             de mexer na densidade"
        );
    }
}
