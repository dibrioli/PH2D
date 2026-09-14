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
            // ⛔ **E o APAGADOR, por uma razão de LEI e não de ausência:** o
            // alvo dele é ABSOLUTO (a superfície de referência), e o tecto
            // `min(f, 1)` diz que ele *nunca ultrapassa*. Acumular um alvo
            // absoluto não nomeia nada — o segundo dab tem o mesmo destino que
            // o primeiro. A espec §4.1 escreve-o com todas as letras: *«sem
            // direcção privilegiada, sem normal e sem acumulador»*.
            && self != Self::EraseMultires
            // ⛔ **E o ESFREGÃO, por uma razão de LEI que é IRMÃ da do apagador
            // e não a mesma:** o alvo dele não é fixo ao longo do traço (o
            // campo `D` é relido a cada dab), mas é **ancorado na superfície de
            // referência** — `R[v] + D′[v]` —, e o `Accumulate` desta casa é o
            // `from_live` do [`Grip::Stamp`], ou seja *de onde a curva de queda
            // mede a distância*. Com um alvo ancorado na referência, mandar a
            // queda medir da posição JÁ esfregada faria a pegada do pincel
            // depender de quanto relevo ele já transportou — uma lei que
            // referência nenhuma declara.
            //
            // ⚠️ **MEDIDO antes de escondido** (`o_acumular_do_esfregao_e_uma_lei_que_ninguem_declara`):
            // ligá-lo muda a saída (não é um controlo morto), e é exactamente
            // por isso que ele não pode ser oferecido sem uma lei atestada por
            // trás. *Esconder um knob vivo e esconder um knob morto leem-se
            // igual numa tabela — o que os separa é a medição escrita ao lado.*
            && self != Self::SmearMultires
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
        matches!(
            self,
            Self::Smooth | Self::Sharpen | Self::SurfaceSmooth | Self::SmearMultires
        )
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

    /// **ESTE VERBO PRECISA DE UMA SUPERFÍCIE DE REFERÊNCIA?**
    ///
    /// ⭐ A porta ÚNICA da pergunta, com **três** consumidores em sítios
    /// diferentes: o **pen-down** da shell (para a fotografar), a **recusa** em
    /// voz alta quando não há pilha de multiresolução (espec §4.3), e o
    /// **arnês** dos censos, que sem ela mediria um verbo inerte.
    ///
    /// ⛔ **Sem pilha, o dado de entrada NÃO EXISTE** — não é que o resultado
    /// seja mau: não há de onde tirar um deslocamento. *O irmão-filtro do alvo
    /// estoirou publicamente por não verificar isto*, e é por isso que o gate
    /// desta fronteira é a **recusa** e não o resultado.
    #[must_use]
    pub fn precisa_de_referencia(self) -> bool {
        matches!(self, Self::EraseMultires | Self::SmearMultires)
    }

    /// **ESTE VERBO CORRE SEM O INTERRUPTOR DA TOPOLOGIA DINÂMICA?**
    ///
    /// ⭐⭐⭐ **ORDEM DO DONO (2026-09-14): *«independente se Dynamic topology
    /// está ligado ou não, Density faz o seu trabalho. Dynamic topology é para
    /// os outros pincéis.»***
    ///
    /// ⚠️ **É uma DIVERGÊNCIA DECLARADA da referência**, e não um esquecimento:
    /// a espec §3.2 tem o desarme na primeira linha da tabela-verdade — o modo
    /// de detalhe em *Manual* desliga o passe inteiro, **este pincel incluído**.
    /// A decisão de produto aqui é outra, e o argumento que a sustenta é do
    /// dono: aquele interruptor governa *«o meu traço também muda a
    /// topologia?»*, e este verbo **não tem traço** — todo o efeito dele já é
    /// sobre a topologia. Exigir-lhe o interruptor é pedir permissão para fazer
    /// a única coisa que ele sabe fazer.
    ///
    /// ⭐ **A resposta é DERIVADA e não uma lista:** quem não tem lei
    /// por-vértice ([`Self::sem_lei_por_vertice`]) é exactamente quem não tem
    /// nada a ganhar com o interruptor. Um verbo novo dessa espécie nasce com a
    /// resposta certa, e um verbo de carimbo nunca a herda por engano.
    ///
    /// ⛔ **O que isto NÃO dispensa:** a recusa com a **pilha de
    /// multiresolução** montada, que é estrutural (refinar a base sob níveis
    /// que são subdivisão dela deixaria cada nível a descrever outra malha), e
    /// a **triangulação** — os dois motores recusam quads por geometria
    /// (`Refine::NotTriangles`), e com o interruptor desligado ninguém a fez.
    /// *Quem corre sem o interruptor herda o trabalho que ele fazia.*
    #[must_use]
    pub fn corre_sem_o_interruptor(self) -> bool {
        self.sem_lei_por_vertice()
    }

    /// **Este verbo REFINA a malha em Dynamic Topology?**
    ///
    /// ⚠️⚠️ **Ela TEVE um argumento durante um dia, e o dono retirou-o**
    /// (2026-09-14: *«não precisamos do modo Thin Only. Deve ser sempre
    /// Equalise.»*). O `DensityModo` existiu entre dois reports do mesmo dia:
    /// o primeiro (*«por que não pode aumentar a densidade também?»*) mostrou
    /// que a minha leitura da tabela-verdade da espec §3.2 estava pela metade —
    /// o pincel **acrescenta** a bandeira de colapso e **não retira** a de
    /// partir —, e o segundo decidiu que a metade que só afina **não é
    /// produto**. ⇒ a densidade leva a malha ao alvo **nos dois sentidos,
    /// sempre**, e a coluna volta a ser função só do verbo.
    ///
    /// ⚠️ **A recusa medida da espec continua de pé:** *«fazer o `Density`
    /// também subdividir»* quer dizer o pincel **FORÇAR** o partir onde o
    /// ajuste da cena o desliga. Nós não temos esse ajuste — nem o queremos,
    /// pela ordem acima —, logo não há nada que ele possa forçar.
    #[must_use]
    pub fn refina_no_dyntopo(self) -> bool {
        self.mexe_na_topologia()
    }

    /// **Este verbo COLAPSA arestas curtas em Dynamic Topology?**
    ///
    /// A segunda metade do [`Self::refina_no_dyntopo`], e ela existe separada
    /// pela razão escrita lá: as duas são **leis independentes** que por acaso
    /// vivem na mesma porta — um verbo pode querer relaxar densidade sem criar
    /// detalhe. *Uma tabela com uma coluna só obriga quem a lê a escolher por
    /// ela.*
    ///
    /// ⚠️ **Hoje ela responde o mesmo que a irmã em TODOS os verbos**, e isso é
    /// um facto sobre o produto de hoje — **não** uma lei. Elas separaram-se
    /// durante um dia, quando o pincel de densidade teve um ajuste próprio, e
    /// voltaram a coincidir quando o dono o retirou. O gate
    /// `as_duas_colunas_coincidem_hoje_e_isso_nao_e_uma_lei` reprova no dia em
    /// que o estudo (`docs/3D/22`) as separar outra vez, para a mudança ficar
    /// visível no diff em vez de acontecer num `match` que ninguém recontou.
    #[must_use]
    pub fn colapsa_no_dyntopo(self) -> bool {
        self.mexe_na_topologia()
    }
}

/// **O CENSO DAS DUAS COLUNAS DO DYNTOPO** — o que esta tabela afirma HOJE.
#[cfg(test)]
mod dyntopo_tests {
    use super::Verb;

    /// ⭐⭐⭐ **A DENSIDADE LEVA A MALHA AO ALVO NOS DOIS SENTIDOS, SEMPRE.**
    ///
    /// ⚠️⚠️ **Este gate mudou DUAS vezes em 2026-09-14, e os dois passos ficam
    /// registados porque o segundo só se entende com o primeiro.**
    ///
    /// 1. Ele nasceu a afirmar o CONTRÁRIO — *«a densidade LIGA o colapso e NÃO
    ///    liga o partir; um pincel que também subdividisse é outro produto»* —,
    ///    e quem o desmentiu foi o dono, pelo produto: *«por que não pode
    ///    aumentar a densidade também?»*. A espec §3.2 traz a tabela-verdade do
    ///    passe e diz que o pincel **ACRESCENTA a bandeira de colapso** e **não
    ///    RETIRA** a de partir, que segue o ajuste de refino da cena — medido
    ///    na mesma malha grossa: `81 → 81` contra **`81 → 101`**.
    /// 2. Nasceu então um ajuste com os dois sentidos, e o dono **retirou-o no
    ///    mesmo dia**: *«não precisamos do modo Thin Only. Deve ser sempre
    ///    Equalise.»* ⇒ a coluna volta a ser função só do verbo.
    ///
    /// ⭐ *Um gate pode pinar a leitura errada de uma espec tão bem como pina um
    /// defeito*, e o que o separou de uma medição foi ninguém ter corrido a
    /// outra célula. ⛔ **A recusa medida da espec continua de pé e é OUTRA
    /// pergunta:** ela é sobre o pincel **FORÇAR** o partir onde o ajuste da
    /// cena o desliga — e nós não temos esse ajuste.
    #[test]
    fn a_densidade_leva_a_malha_ao_alvo_nos_dois_sentidos() {
        assert!(
            Verb::Density.colapsa_no_dyntopo(),
            "o colapso é a LEI deste pincel — ele acrescenta a bandeira \
             aconteça o que acontecer"
        );
        assert!(
            Verb::Density.refina_no_dyntopo(),
            "a densidade tem de PARTIR também: é a célula `81 -> 101` da espec \
             e o report do dono (*«por que não pode aumentar a densidade \
             também?»*)"
        );
    }

    /// ⭐⭐⭐ **E ELA CORRE SEM O INTERRUPTOR — ordem do dono.**
    ///
    /// *«Independente se Dynamic topology está ligado ou não, Density faz o seu
    /// trabalho. Dynamic topology é para os outros pincéis.»* (2026-09-14)
    ///
    /// ⚠️ **É uma DIVERGÊNCIA DECLARADA da referência** (espec §3.2, 1.ª linha:
    /// o modo de detalhe em *Manual* desarma o passe inteiro, este pincel
    /// incluído), e o gate afirma-a **pelos dois lados** — senão ele mediria
    /// metade e ficaria verde sobre um `true` cravado.
    #[test]
    fn so_quem_nao_tem_lei_por_vertice_corre_sem_o_interruptor() {
        let livres: Vec<&str> = Verb::ALL
            .into_iter()
            .filter(|v| v.corre_sem_o_interruptor())
            .map(Verb::label)
            .collect();
        assert_eq!(
            livres,
            ["Density"],
            "a porta que dispensa o interruptor alcançou outro verbo — um de \
             carimbo que a herde passa a mudar a topologia com o modo desligado"
        );
        // ⚠️ **O outro lado:** o piso de população impede que um `false` cravado
        // deixe este censo verde a medir uma lista vazia.
        let presos = Verb::ALL
            .into_iter()
            .filter(|v| !v.corre_sem_o_interruptor())
            .count();
        assert!(
            presos >= 25,
            "só {presos} verbos precisam do interruptor — a varredura partiu-se"
        );
    }

    /// ⭐⭐⭐ **SÓ UM VERBO SE AFASTA DO COMPORTAMENTO DE HOJE, E É A MÁSCARA.**
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
    /// ⚠️ **A densidade ENTROU e SAIU desta lista no mesmo dia** — ela esteve
    /// aqui enquanto teve um ajuste próprio, e no ajuste único que shipa faz
    /// exactamente o que os outros 27 fazem. *O que a distingue não é a
    /// topologia: é ela não ter lei por-vértice e não precisar do interruptor.*
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
            if v.refina_no_dyntopo() != chega_a_porta || v.colapsa_no_dyntopo() != chega_a_porta {
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
            !Verb::Mask.refina_no_dyntopo() && !Verb::Mask.colapsa_no_dyntopo(),
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
                "`{}` tem âncora e declara que mexe na topologia — ele nem \
                 chega à porta hoje, então isto muda comportamento sem o estudo",
                v.label()
            );
        }
    }

    /// ⚠️⚠️ **AS DUAS COLUNAS COINCIDEM HOJE, E ISSO NÃO É UMA LEI.**
    ///
    /// ⭐ **Este gate já morreu e ressuscitou**, e o ciclo é o registo: ele
    /// existia a dizer exactamente isto, **reprovou** em 14/09 quando o pincel
    /// de densidade ganhou um ajuste que as separava, foi reescrito com a morte
    /// da premissa no diff — e voltou no mesmo dia, quando o dono retirou o
    /// ajuste. *Uma premissa que morre e renasce em doze horas é a melhor prova
    /// de que ela tinha de estar num gate e não num comentário.*
    ///
    /// ⇒ o estudo (`docs/3D/22`) pode separá-las outra vez, e quando o fizer é
    /// aqui que a mudança aparece.
    #[test]
    fn as_duas_colunas_coincidem_hoje_e_isso_nao_e_uma_lei() {
        let separados: Vec<&str> = Verb::ALL
            .into_iter()
            .filter(|v| v.refina_no_dyntopo() != v.colapsa_no_dyntopo())
            .map(Verb::label)
            .collect();
        assert!(
            separados.is_empty(),
            "as duas colunas separaram-se em {separados:?}. Se foi o ESTUDO a \
             chegar, reescreva este gate com a tabela medida ao lado; se não \
             foi, é uma regressão"
        );
    }
}
