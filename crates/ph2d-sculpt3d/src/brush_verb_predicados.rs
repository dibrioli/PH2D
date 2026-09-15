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
            // ⛔⛔ **E A PROJECÇÃO, por a lei dela já ter ESCOLHIDO** — e é a
            // razão OPOSTA à do esfregão, que é o que as torna as duas
            // instrutivas. A espec §6.5 diz que este verbo mede *«a partir de
            // onde o vértice está AGORA»* **sempre**, logo o
            // [`crate::Verb::grip_law`] prega o `from_live` a `true` — e o
            // `Accumulate` desta casa é **exactamente** essa coluna. ⇒ ele
            // deixou de mover **nada** neste verbo, e um interruptor que não
            // move nada é o controlo morto que o censo dos knobs caça.
            //
            // ⚠️ **MEDIDO pelo oráculo, e não deduzido:** com a coluna presa ao
            // interruptor, as `16` fixturas partiam-se em *um dab bate, seis
            // dabs desviam*; com ela pregada, **oito** delas saltam de
            // `5,9e-2`–`2,6e-1` para `8,9e-8`–`2,0e-7`. *O corpus escolheu a
            // posição do interruptor, e depois de escolhida não sobra
            // interruptor.*
            && self != Self::SceneProject
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
    /// ⛔⛔⛔ **O [`Verb::SceneProject`] ESTEVE NESTA LISTA E SAIU POR ORDEM DO
    /// DONO (15/09)**, depois do smoke da cena `=45`: *«não vi utilidade na
    /// feature Scene Project + CTRL. Melhor retirá-la e documentá-la como
    /// indesejada.»*
    ///
    /// ⚠️ **Tirar o gesto tirou a CAPACIDADE, e isso foi medido antes de se
    /// cortar:** o `Brush::invert` tem **um** escritor no produto inteiro (o
    /// pen-down) e **nenhum** controlo de painel o oferece ⇒ o `Ctrl` era a
    /// única porta. A lei foi **APAGADA** em vez de ficar viva e inalcançável
    /// (*a cura de um órfão é apagar*), e o parâmetro `sign` do
    /// [`crate::projectar::alvo_do_vertice`] desapareceu com ela.
    ///
    /// ⭐ **O mecanismo, o número (`2,384e-7` nas duas fixturas do oráculo) e o
    /// diagnóstico que pôs a lei certa no dia em que ela saiu ficam registados
    /// naquela função** — *o que foi medido e rejeitado não se reconstrói*.
    ///
    /// ⛔⛔ **E a redacção que aqui estava afirmava a lei REFUTADA** (*«ele vira
    /// o RAIO»*): ela sobreviveu à cura da manhã porque **a mesma lei estava
    /// escrita em DOIS sítios** e só um foi corrigido. *Uma lei escrita em dois
    /// sítios ainda não é uma lei — e a cópia que ninguém relê é a que envelhece.*
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

    /// **O PASSE DE AUTO-SUAVIZAÇÃO CHEGA AO BARRO DESTE VERBO?**
    ///
    /// O *Auto Smooth* corre como um **segundo `dab_core`** sobre a mesma
    /// pegada, logo **todo verbo que resolve a própria região o salta por
    /// construção** ([`Self::resolve_a_propria_regiao`]) — e o painel pintava-o
    /// na mesma. *O artista arrasta e o barro não sente nada*, que é a espécie
    /// que o dono reporta como «não vejo efeito».
    ///
    /// ⚠️⚠️ **Os três que desviam não têm a MESMA resposta, e é por isso que
    /// isto não se deriva daquela porta:**
    /// - [`Self::Pose`] responde **`true`** — ele tem passe PRÓPRIO
    ///   (`stroke_pose::alisa_a_pose`), escrito porque a espec dele o prescreve
    ///   (§15 e item 18: *«ela segue os PESOS, não o raio»*);
    /// - [`Self::Cloth`] e [`Self::Boundary`] respondem **`false`**, e é
    ///   **MEDIDO**: o censo dos knobs lê `0,000e0` entre as duas pontas da
    ///   faixa nos dois.
    ///
    /// ⛔⛔ **E o `false` deles é uma DIVERGÊNCIA DECLARADA, não uma dívida a
    /// pagar às cegas:** nenhuma das duas especs prescreve auto-suavização para
    /// aquele pincel, e *inventar uma lei para um pincel de clean-room sem
    /// referência é exactamente o que a parede existe para impedir*. ⭐ O
    /// contorno tem **`Modo::Suavizar`**, um alisamento **da referência**, entre
    /// os seis dele — oferecer o genérico por cima seria a segunda resposta à
    /// mesma pergunta.
    ///
    /// ⏳ **As duas saídas ficam NOMEADAS:** uma janela **E** pode medir se o
    /// alvo o oferece com aqueles pincéis na mão, ou o dono pode ordenar que ele
    /// seja construído **seguindo os pesos da região de cada um** — o `alisa_a_pose`
    /// já mostra a forma.
    ///
    /// ⚠️ **Os dois `false` da referência ficam onde estavam** — o
    /// [`crate::Brush::auto_smooth_brush`] continua a peneirar [`Self::Smooth`]
    /// e [`Self::Mask`], e por outra razão: *alisar um alisamento é o mesmo
    /// verbo duas vezes*, e um passe que mexesse na posição durante um gesto de
    /// máscara moveria o barro num gesto cuja razão de existir é não movê-lo.
    /// ⭐ **E o [`Self::Density`] entra pela TERCEIRA razão**, medida pelo censo
    /// dos knobs (`0,000e0` entre as duas pontas da faixa): o passe de
    /// auto-suavização corre **depois** do laço por-vértice, e este verbo sai
    /// **antes de tudo** ([`Self::sem_lei_por_vertice`]). *Ele não resolve a
    /// própria região nem tem passe próprio — ele não tem região nenhuma.*
    #[must_use]
    pub fn o_auto_smooth_chega(self) -> bool {
        !matches!(self, Self::Cloth | Self::Boundary | Self::Density)
    }

    /// ⭐⭐ **O `Strength` CHEGA AO BARRO DESTE VERBO?** — a porta que o painel
    /// consulta antes de pintar a fileira da força.
    ///
    /// ⛔⛔ **Ela nasceu de uma medição e de um report do dono.** O censo dos
    /// knobs mede o [`Self::Density`] a arrastar a força de `0,1` a `1,0` com
    /// desvio `0,000e0` no barro: o efeito dele é sobre a TOPOLOGIA, e o
    /// [`crate::SculptStroke::dab`] sai antes de a cadeia de peso existir. *Um
    /// controlo que o artista arrasta e o barro não sente é pior que um
    /// ausente*, e este era o item aberto que o `CLAUDE.md` §5 nomeava
    /// (*«esconder ou pintar em cinzento»*).
    ///
    /// ⭐ **A escolha foi ESCONDER, e ela não é gosto:** o pintor das fileiras
    /// desta crate já declara por escrito que *«uma row condicional é PULADA,
    /// não desenhada apagada — um controlo apagado que ainda despacha mente»*.
    /// A curva é a excepção **porque não pode ser escondida** (cerca de produto
    /// medida e gateada), e por isso é ela que ganha a razão à vista
    /// ([`crate::CurvaInerte`]).
    #[must_use]
    pub fn a_forca_chega_ao_barro(self) -> bool {
        !self.sem_lei_por_vertice()
    }

    /// **ESTE VERBO PRECISA DE UM BORDO ABERTO?**
    ///
    /// ⛔ **Numa peça FECHADA ele não move um único vértice**, e não porque o
    /// resultado seja mau: a região dele **começa na borda** e cresce para
    /// dentro — sem borda não há de onde começar. É a mesma espécie de fronteira
    /// que a [`Self::precisa_de_referencia`] nomeia um degrau acima: *o dado de
    /// entrada NÃO EXISTE.*
    ///
    /// ⚠️ **A cena `=42` abre numa TIGELA por causa disto**, e o doc dela
    /// escreve-o: numa esfera ela mostraria uma ferramenta que parece partida.
    ///
    /// ⏳ **Consumidor NOMEADO e por construir: a recusa em voz alta.** Hoje
    /// quem aponta este pincel a uma peça fechada não recebe queixa nenhuma — o
    /// [`crate::Verb::Density`] já tem a dele (`queixa_do_passe`), e *um pincel
    /// mudo sobre a própria inércia é o que o dono reporta como «não
    /// funciona»*. O consumidor de hoje é o **arnês dos censos**, que sem esta
    /// porta media um verbo inerte e lia os cinco knobs dele como mortos.
    #[must_use]
    pub fn precisa_de_bordo_aberto(self) -> bool {
        matches!(self, Self::Boundary)
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
    ///
    /// ⚠️⚠️ **Este bloco viveu ONZE LINHAS acima até 2026-09-15, colado ao
    /// [`Self::o_auto_smooth_chega`]** — sem linha em branco e sem `fn` entre os
    /// dois, logo o `rustdoc` dava-o ao vizinho e esta porta ficava **sem doc
    /// nenhum**. *Um doc que muda de dono em silêncio é pior que a ausência
    /// dele: ele afirma sobre a função errada, e quem o lê aprende a lei de
    /// outra pergunta.*
    #[must_use]
    pub fn precisa_de_referencia(self) -> bool {
        matches!(self, Self::EraseMultires | Self::SmearMultires)
    }

    /// ⭐⭐⭐ **ESTE VERBO PICA NA SUPERFÍCIE DO PEN-DOWN?** — a porta que
    /// impede o cursor de perseguir o barro que ele próprio acabou de mandar
    /// embora.
    ///
    /// # ⛔⛔ O report do dono (`=45`, 2026-09-14): *«resultado bem bizarro»*
    ///
    /// Todo dab desta casa é re-picado da superfície VIVA: o raio do evento
    /// seguinte é lançado contra a malha que o evento anterior acabou de
    /// mover. Isso é inofensivo enquanto o deslocamento de um dab for uma
    /// **fracção do raio do pincel** — a superfície recua um pouco debaixo do
    /// cursor e o dab seguinte, nove pixels adiante, ainda cai ao lado do
    /// anterior.
    ///
    /// ⚠️ **E é FALSO para este verbo, porque o deslocamento dele não é
    /// limitado pelo raio:** o `d` da [`crate::projectar::distancia`] é uma
    /// distância da **CENA** (espec §6.5 — `direcção · d · peso · força²`), e
    /// nada na lei a compara com o pincel. Medido na `=45` de fábrica, um dab
    /// só move o barro `0,63` com um raio de `0,34` — quase **dois raios** —,
    /// logo a superfície **foge de debaixo do cursor**, o raio seguinte passa
    /// pelo buraco e acerta **no outro lado da peça**:
    ///
    /// ```text
    /// dab 0 -> [ 0,59,  0,38,  0,74]   a frente da bola
    /// dab 1 -> [ 0,00, -0,03, -0,24]   ⛔ o MIOLO — nove pixels depois
    /// dab 3 -> [ 0,41,  0,17,  0,16]   e volta
    /// ```
    ///
    /// Seis dabs = seis crateras em sítios sem relação, e as fronteiras entre
    /// elas **são** os golpes escuros da foto do dono.
    ///
    /// # ⭐ A lei que fica
    ///
    /// *Um traço trabalha a superfície que o artista VIU quando encostou a
    /// caneta.* A consulta da pegada continua nas posições vivas (é lá que o
    /// barro está); o que congela é **onde o dab aterra**.
    ///
    /// ⚠️ **É a MESMA família que o polegar já pagou nesta casa** — lá é a
    /// PEGADA que foge ([`crate::SculptStroke::pegada_ancorada`]), aqui é o
    /// **PICK**, um degrau acima. E a lei-irmã já está escrita na cadeia de
    /// peso: com o `Accumulate` desarmado, a curva de queda deste verbo já mede
    /// as distâncias contra o `pre` do pen-down ([`crate::GripLaw::from_live`])
    /// — *o alvo já vinha da superfície congelada, e só o CENTRO não vinha*.
    ///
    /// ⛔ **Ele é `false` para todos os outros, e não por omissão:** os verbos
    /// de carimbo deslocam `raio × 0,1 × intensidade` (uma fracção do raio, por
    /// construção), os quatro grips de gesto já congelam a pegada ou a âncora, e
    /// o tecido desvia antes do `dab_core`. *Congelar o pick de quem não foge
    /// seria trocar o cursor por um fantasma sem comprar nada.*
    #[must_use]
    pub fn pica_na_superficie_do_pen_down(self) -> bool {
        matches!(self, Self::SceneProject)
    }

    /// **A LEI DESTE VERBO LÊ A DISTÂNCIA AO CURSOR?** — a porta que decide se
    /// a fileira da DUREZA tem sujeito.
    ///
    /// A dureza é um remapeamento da distância **normalizada**
    /// ([`crate::Brush::shaped_distance`]) que corre antes de toda curva de
    /// queda. Onde não há distância a remapear ela é **inerte**, e o painel que
    /// a pinta entrega ao artista um controlo que ele arrasta sem efeito — a
    /// espécie que o dono reporta como *«não vejo efeito»*.
    ///
    /// ⛔ **Os três `false` são por LEI, cada um com a fonte:**
    /// - [`Self::Pose`] — a espec dele di-lo com todas as letras (§1.3:
    ///   *«sem efeito: não há atenuação radial, o vértice é governado pelos
    ///   PESOS dos segmentos e não pela distância ao cursor»*), e a região dele
    ///   cresce pela **ligação** da malha, não pelo raio;
    /// - [`Self::Boundary`] — os pesos dele saem do **anel** e do percurso do
    ///   contorno ([`ph2d_boundary::pesos`]), e nada naquela cadeia consulta o
    ///   `shaped_distance`;
    /// - [`Self::Density`] — ele [`Self::sem_lei_por_vertice`], logo não há peso
    ///   nenhum onde uma distância pudesse entrar.
    ///
    /// ⚠️ **O [`Self::Cloth`] responde `true` apesar de também desviar antes do
    /// `dab_core`** — ele chama o `shaped_distance` na própria cadeia —, e é por
    /// isso que esta pergunta **não** se deriva da
    /// [`Self::resolve_a_propria_regiao`]: *desviar do laço e não ler a
    /// distância são duas propriedades, e três verbos mostram que elas não
    /// coincidem.*
    ///
    /// ⚠️⚠️ **Isto NÃO resolve o `Strength` nem a curva do [`Self::Density`]**,
    /// que são o item aberto com decisão do dono por tomar (esconder ou pintar
    /// em cinzento) — *uma porta que responde a uma pergunta não responde às
    /// vizinhas só por estarem na mesma fileira.*
    #[must_use]
    pub fn a_lei_le_a_distancia_ao_cursor(self) -> bool {
        !matches!(self, Self::Pose | Self::Boundary | Self::Density)
    }
}
