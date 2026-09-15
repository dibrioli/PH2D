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
                // ⭐⭐⭐ **PROJECTAR honra o Ctrl, e ele vira o RAIO — não o
                // sinal do deslocamento.** Medido no corpus, e é o achado que
                // essa distinção produziu: a fixtura `projectar_invertido` tem
                // o alvo ACIMA e os dois sentidos DESLIGADOS, logo escrito como
                // negação no fim ele movia **`0` vértices contra `301`** — não
                // há o que negar quando o raio não acertou em nada.
                // ⭐ *Do lado do artista, `Ctrl` aqui não quer dizer «afasta»:
                // quer dizer «procura do outro lado».*
                | Self::SceneProject
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
}
