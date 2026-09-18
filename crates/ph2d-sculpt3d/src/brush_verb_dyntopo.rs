//! ⭐⭐⭐ **QUEM MEXE NA TOPOLOGIA EM DYNAMIC TOPOLOGY** — as duas colunas, verbo
//! a verbo, e a proveniência de cada célula.
//!
//! Irmão (`#[path]`) do [`super::brush_verb`], como o
//! [`super::brush_verb_grip`] e o [`super::brush_verb_campo`]. O corte é de
//! ASSUNTO: lá moram *quais verbos existem*, no do grip *como a mão os conduz*,
//! no do campo *que kernel elástico cada um recebe* — e aqui **se um traço dele
//! muda a CONTAGEM DE VÉRTICES da peça**.
//!
//! ⚠️ **O corte foi FORÇADO pelo tecto de LOC** (o ficheiro dos predicados
//! chegou a `711` de `700` quando o estudo encheu a tabela) e é melhor por
//! isso: esta é a tabela que **cresce uma célula a cada corrida de oráculo**, e
//! ela passa a ter um ficheiro onde se lê inteira — com o censo dela ao lado.
//! ⛔ *Subir o número em vez de cortar é o que o `CLAUDE.md` §2 proíbe por
//! escrito.*
//!
//! # O report que a abriu
//!
//! > *«algumas tools que não deveriam fazer a subdivisão de polígonos no modo
//! > Dynamic Topology estão fazendo (como smooth) enquanto algumas que deveriam
//! > criar subdivisões com Dynamic Topology não estão criando.»*
//! > — o dono, 2026-09-14
//!
//! ⭐⭐ **A causa era UMA LINHA que não menciona verbo nenhum:** o refino tinha
//! **um** chamador de produto — o braço do CARIMBO —, logo a pergunta que o
//! produto respondia era *«este gesto passou pelo caminho do carimbo?»* e não
//! *«este verbo cria superfície nova?»*. O plano inteiro é o
//! [`docs/3D/22`](../../../docs/3D/22_plano_quem_subdivide_no_dyntopo.md).

use super::verb::Verb;

impl Verb {
    /// **ESTE VERBO PODE MEXER NA TOPOLOGIA DE TODO?** — a metade das duas
    /// colunas que NÃO depende de ajuste nenhum.
    ///
    /// ⭐⭐⭐ **ESTA TABELA É O VEREDITO DE DOIS ORÁCULOS** (2026-09-14,
    /// `docs/3D/22`), e o report do dono — *«algumas tools que não deveriam
    /// subdividir estão fazendo (como smooth) e algumas que deveriam não
    /// estão»* — foi **confirmado nos dois sentidos**:
    ///
    /// | fonte | licença | como | o que respondeu |
    /// |---|---|---|---|
    /// | a **livre** | **MIT** | ⭐ lê-se e porta-se, com atribuição (§0.9: *a triagem pára na primeira porta ABERTA*) | 13 ferramentas |
    /// | a **medida** | GPL | ⛔ corre-se **sem interface**, por uma janela **E**; a saída é DADO (GPLv2 §0) | **27** tipos, `343` células |
    ///
    /// ⭐⭐⭐ **E EXISTE UMA TERCEIRA FONTE, QUE GANHA DAS DUAS: O DONO.** Em
    /// 2026-09-14 ele correu o smoke `=14` e devolveu cinco células —
    /// *«acho que layer, move/drag deve subdividir. Thumb se for possível,
    /// deveria subdividir. Twist com dynamic topology fica com resultado muito
    /// ruim»* —, e ela ganha pelo mesmo princípio que dá o lugar ao oráculo:
    /// *uma referência responde o que outro programa FAZ; o dono responde o que
    /// este produto TEM DE fazer.* ⚠️ Cada uma dessas células está **nomeada**
    /// no gate `o_veredito_do_dono_sobre_cinco_celulas`, e não espalhada por
    /// braços de `match` que se leem como se fossem medição.
    ///
    /// # A tabela, e a proveniência de cada célula
    ///
    /// | verbo | mexe? | de onde |
    /// |---|---|---|
    /// | Draw · Clay · Inflate · Flatten · Fill · Scrape · Pinch · Crease · Blob · Clay Strips · Clay Thumb · Multiplane Scrape | ✅ | as duas (onde as duas existem) |
    /// | **Snake Hook** | ✅ | **as duas** — e foi preciso LIGAR a porta ao gesto ancorado |
    /// | **Nudge** | ✅ | a medida (`441 → 2 853`) |
    /// | **Move · Thumb · Layer** | ✅ | ⭐⭐⭐ **o DONO**, com os olhos no smoke `=14` — contra as duas referências |
    /// | **Twist · Local Scale** | ⛔ | ⭐⭐⭐ **o DONO** (*«fica com resultado muito ruim»*), e isso **desempata** as duas |
    /// | **Smooth** | ⛔ | **as duas**, e o dono nomeou-o à letra |
    /// | **Slide Relax · Surface Smooth** | ⛔ | a medida (`441 → 441` nos dois extremos) |
    /// | Pose · Boundary · Cloth | ⛔ | as duas (onde as duas existem) |
    /// | **Mask** | ⛔ | ⭐ **domínio**, curado um dia ANTES do estudo — e as duas confirmaram |
    /// | **Draw Sharp** | ⛔ | o oráculo do PRÓPRIO pincel (espec §2.8) — e o nome dele diz o mesmo: um vinco fino não sobrevive a um remalhe |
    /// | Density | ✅ | ⛔ construção: ele **É** o passe de topologia |
    /// | Erase · Smear Displacement | ⛔ | ⛔ construção: multirresolução exclui dyntopo (espec §5.6) |
    /// | **Sharpen · Magnify** | ✅ | ⚠️ **NENHUM oráculo os responde** — valor conservador, com gate a nomeá-los |
    ///
    /// ⭐⭐ **A célula da MÁSCARA foi curada um dia ANTES do estudo, por
    /// raciocínio de domínio, e a referência livre depois SOBRESCREVEU a lei
    /// geral dela para dizer o mesmo.** *Uma cura que o oráculo depois confirma
    /// é a melhor prova de que o raciocínio que a produziu era do domínio e não
    /// do programa.*
    ///
    /// ⛔ **E NENHUMA das duas referências separa as duas colunas por verbo** —
    /// quem refina também colapsa, nas duas. É facto sobre elas e **não** uma
    /// lei: o irmão [`Self::colapsa_no_dyntopo`] existe separado porque a
    /// separação **é exprimível** (a referência medida tem o modo de refino da
    /// cena com os três estados), e nós podemos querê-la antes delas.
    #[must_use]
    pub fn mexe_na_topologia(self) -> bool {
        match self {
            // ⛔⛔ **O SMOOTH NÃO MEXE, e são DUAS fontes independentes a
            // dizê-lo:** o report do dono nomeia-o à letra, e o oráculo livre
            // **não chama** a topologia dinâmica de lado nenhum do alisador.
            //
            // ⚠️ O argumento legítimo do outro lado (*manter a densidade ao
            // relaxar*) existe e perdeu: relaxar não cria detalhe, e crescer a
            // malha sob um gesto que só a acalma é pagar topologia por nada.
            Self::Smooth => false,
            // ⛔ **A MÁSCARA não mexe**, e ela é a única célula desta tabela que
            // foi curada ANTES do estudo — por raciocínio de domínio (*um gesto
            // que não escreve posição não tem porque mudar a topologia*) e não
            // por leitura de alvo nenhum. Medido antes da cura: `830 → 1 331`
            // vértices num dab que não move um vértice.
            Self::Mask => false,
            // ⛔⛔ **O PINCEL AFIADO não mexe, e a fonte é o ORÁCULO deste
            // pincel:** com a topologia dinâmica ligada o alvo **não refina nem
            // colapsa** com ele (espec do afiado §2.8 e §9.4, medido pela obra
            // do dyntopo).
            //
            // ⭐ **E o mecanismo é o que o NOME promete:** um vinco de meio raio
            // de largura não sobrevive a um remalhe — a documentação pública do
            // alvo aponta o pincel de VINCO para quem quer detalhe com
            // topologia dinâmica. *Refinar aqui apagaria a ferramenta com o
            // próprio gesto dela.*
            Self::DrawSharp => false,
            // ⭐⭐⭐ **OS TRÊS QUE PASSARAM A MEXER**, e são a segunda metade do
            // report (*«algumas que deveriam criar subdivisões não estão
            // criando»*). Eles **esticam** superfície — o gancho transporta
            // matéria, a torção e a escala local varrem uma região inteira —,
            // logo são os que mais produzem aresta longa, e eram exactamente os
            // que nunca ganhavam um vértice.
            //
            // ⚠️ **Eles têm ÂNCORA, logo entram por outra porta** (`hook_step`,
            // `turn_at`) — e foi preciso ligá-la lá, o que é a metade de
            // FIAÇÃO desta wave.
            Self::SnakeHook => true,
            // ⭐⭐⭐ **O AGARRAR E O POLEGAR PASSARAM A MEXER — ORDEM DO DONO**
            // (2026-09-14, depois de ver o `=14` com os olhos):
            //
            // > *«acho que layer, move/drag deve subdividir. Thumb se for
            // > possível, deveria subdividir.»*
            //
            // ⚠️⚠️ **Isto é uma DIVERGÊNCIA DECLARADA das DUAS referências** — as
            // duas dizem que o agarrar não mexe —, e o argumento dele ganha
            // delas pela mesma razão que o §0.9 deste repo dá ao oráculo: a
            // referência responde *o que outro programa faz*, e o dono responde
            // *o que este produto tem de fazer*. O argumento que as duas
            // carregam (*a região viaja como um corpo, e a densidade viaja com o
            // material*) é verdadeiro no MIOLO do agarrar e falso no **anel**,
            // onde o barro que anda encontra o barro que ficou — e é ali que a
            // aresta estica.
            //
            // ⚠️ **O polegar custou uma peça**, e é o *«se for possível»* dele: é
            // o único verbo que CONGELA a pegada no pen-down, e um índice
            // guardado não sobrevive a uma renumeração sozinho. As duas metades
            // vivem em [`crate::SculptStroke::grow_with`] e na irmã do colapso.
            Self::Move | Self::Thumb => true,
            // ⭐ **O EMPURRÃO passou a mexer, e a referência medida é clara:**
            // `441 → 2 853` com o alvo fino. Ele é irmão do gancho — transporta
            // matéria ao longo da superfície —, e era dos que *«deveriam criar
            // subdivisões e não estavam»*.
            Self::Nudge => true,
            // ⛔⛔ **OS TRÊS QUE DEIXARAM DE MEXER**, e a referência medida
            // devolve `441 → 441` nos dois extremos do slider para os três:
            //
            // - **Slide Relax** — ele desliza vértices sobre a superfície sem
            //   criar detalhe; adensar debaixo dele é pagar topologia por um
            //   gesto que só arruma a que já existe.
            // - **Surface Smooth** — é o alisador que preserva forma, e cai com
            //   o `Smooth` pela mesma razão.
            // ⚠️ **A DEMÃO SAIU DESTA LISTA em 14/09, por ordem do dono**
            //   (*«acho que layer … deve subdividir»*) — a referência medida
            //   devolvia `441 → 441` para ela, e o veredito do dono ganha da
            //   referência pelo mesmo motivo escrito no braço do agarrar. Ela
            //   não tem braço próprio **de propósito**: é de carimbo, logo o
            //   `_` abaixo já lhe responde `true`, e uma linha que a mutação não
            //   consegue matar não é lei — é comentário com sintaxe de código.
            //   Quem a prende é o gate `o_veredito_do_dono_sobre_cinco_celulas`.
            Self::SlideRelax | Self::SurfaceSmooth => false,
            // ⛔ **OS DOIS DE MULTIRRESOLUÇÃO**, e não é pergunta de oráculo: uma
            // pilha de níveis e uma malha que muda de contagem **não coexistem**
            // (espec §5.6). A porta já os recusa por outro guarda, com a pilha
            // montada; esta linha di-lo na tabela, que é onde alguém procura.
            Self::EraseMultires | Self::SmearMultires => false,
            // **O RESTO CONTINUA COMO ESTAVA** — o valor conservador enquanto a
            // metade atrás da parede não chega: quem tem âncora não alcança a
            // porta, quem carimba alcança-a.
            //
            // ⚠️ **A DENSIDADE cai aqui, e é onde ela pertence** — ele **É** o
            // passe de topologia, e perguntar se ele o dispara é perguntar se
            // ele existe. ⚠️⚠️ **Eu escrevi-lhe um braço EXPLÍCITO e ele era
            // REDUNDANTE**, o que uma prova de mutação mostrou: apagá-lo não
            // mudava um bit, porque este `_` já lhe responde `true`. *Uma linha
            // que a mutação não consegue matar não é lei — é comentário com
            // sintaxe de código*, e o comentário é mais honesto.
            //
            // ⚠️⚠️ **A TORÇÃO E A ESCALA LOCAL CAEM AQUI desde 14/09, e o `false`
            // delas é agora uma RESPOSTA e não o valor conservador.** Elas
            // tiveram um braço `true` durante um dia, escrito sobre a referência
            // livre contra a medida — e o dono desempatou com os olhos:
            //
            // > *«Twist com dynamic topology fica com resultado muito ruim.»*
            //
            // ⭐ **Isto confirma a referência MEDIDA** (`441 → 441` nos dois
            // extremos do slider) e derruba o argumento geométrico que eu tinha
            // escrito ao lado do `true` (*«uma torção cisalha, logo produz
            // aresta longa»*): ela cisalha, sim, e o que o refino faz com esse
            // cisalhamento é **estragar a forma**, não acompanhá-la.
            //
            // ⚠️ **A LOCAL SCALE seguiu a irmã e o dono NÃO a nomeou.** Elas são
            // o mesmo `Grip::Turn`, a mesma célula nas duas referências e a
            // mesma família na medida (*mover uma região como um corpo*) — e
            // deixá-la sozinha do outro lado seria fabricar uma divergência que
            // nenhuma das três fontes pede. *A decisão fica NOMEADA em vez de
            // silenciosa*, no gate `o_veredito_do_dono_sobre_cinco_celulas`.
            //
            // ⚠️ Os ancorados que sobram (`Pose`, `Cloth`) continuam a **não**
            // alcançar a porta, e esse `false` é o estado actual, não uma
            // resposta.
            // ⛔⛔ **O BOX TRIM não mexe na topologia POR DAB — ele não tem
            // dab.** Ele muda a malha inteira de uma vez, na booleana do
            // pen-up, e pôr os motores de refino a correr sobre um gesto que
            // não carimba seria trabalho sobre a peça que o artista não pediu.
            // ⚠️ E ele cai no default `!anchors()`, que responderia **`true`** —
            // é por isso que o braço é explícito.
            Self::BoxTrim => false,
            _ => !self.anchors(),
        }
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
        // ⛔⛔ **ELA DERIVAVA DA `sem_lei_por_vertice`, E A COINCIDÊNCIA
        // QUEBROU** (2026-09-15, com o Box Trim): enquanto a densidade era o
        // único verbo sem lei por-vértice, as duas perguntas tinham a mesma
        // resposta **por acaso**. São perguntas diferentes — *«este verbo tem
        // lei por-vértice?»* e *«o passe de topologia corre para ele mesmo com
        // o interruptor desligado?»* —, e o Box Trim responde `true` à primeira
        // e **`false`** à segunda: ele muda a malha por uma BOOLEANA, não pelos
        // motores de refino, e a triangulação que este caminho herdaria (ver o
        // doc acima) é trabalho que a porta do corte já faz sozinha.
        //
        // ⚠️ É a mesma armadilha que este módulo já pagou três vezes: *duas
        // respostas à mesma pergunta divergem no dia do terceiro membro*, e
        // aqui foi ao contrário — uma resposta a servir duas perguntas, que
        // diverge no dia do SEGUNDO membro.
        matches!(self, Self::Density)
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

    /// ⭐⭐⭐ **Este verbo HONRA o pente de topologia?**
    ///
    /// Espec §6, medida pelo oráculo com um traço igual, o mesmo enquadramento,
    /// o controlo a `0` e a `1`, **e o controlo de que o próprio verbo agiu**
    /// dentro de cada célula.
    ///
    /// ⛔ **CINCO ignoram-no, com a saída BYTE-IDÊNTICA**, e a família tem forma:
    /// são os que lêem as posições de **repouso** ou que trabalham
    /// **ancorados**, mais o que **não escreve posição nenhuma**.
    ///
    /// | verbo | o próprio verbo agiu? | porquê ignora |
    /// |---|---|---|
    /// | [`Verb::DrawSharp`] | moveu `187` vértices | lê as posições do pen-down |
    /// | [`Verb::Thumb`] | moveu `56` | pegada congelada |
    /// | [`Verb::Move`] | moveu `56` | ancorado |
    /// | [`Verb::Twist`] | moveu `49` (varredura **angular**) | ancorado |
    /// | [`Verb::Mask`] | escreveu o canal (soma `0` → `34,777`) | não escreve posição |
    ///
    /// ⚠️⚠️ **DUAS destas células tiveram de ser REFEITAS no oráculo, e é a
    /// lição mais transferível do censo:** na 1.ª redacção a torção corria sobre
    /// um percurso RECTO — e um verbo de torção ancorado varre ângulo **zero**
    /// numa recta, logo moveu `0` vértices. *A célula lia-se exactamente como
    /// «ignora o pente» sendo que era **inerte**.* A máscara move `0` por lei, e
    /// só o controlo de CANAL a separa de uma célula morta.
    ///
    /// ⛔ **A lista é EXPLÍCITA e não derivada**, de propósito: ela é um facto
    /// MEDIDO sobre o alvo, e derivá-la de [`Self::anchors`] faria a resposta
    /// mudar no dia em que alguém mexesse naquela outra pergunta, sem ninguém
    /// recontar. O gate `os_cinco_que_ignoram_o_pente_tem_a_forma_que_a_espec_da`
    /// afirma que a **forma** ainda descreve a lista — e reprova no dia em que
    /// ela deixar de descrever, que é quando alguém tem de voltar ao oráculo.
    #[must_use]
    pub fn honra_o_pente(self) -> bool {
        !matches!(
            self,
            Self::DrawSharp | Self::Thumb | Self::Move | Self::Twist | Self::Mask
        )
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

/// **O CENSO DAS DUAS COLUNAS DO DYNTOPO** — o que esta tabela afirma HOJE, e
/// os gates dela; ver o módulo irmão.
///
/// ⚠️ **O corte é de RESPONSABILIDADE e foi forçado pelo tecto de LOC**
/// (`788` contra `700`): aqui moram as **respostas** — que verbo mexe na
/// topologia, quem refina, quem colapsa, quem honra o pente —, e lá as
/// **provas** delas. ⛔ Uma entrada no `FILE_OVERAGE_OK` não era saída: o
/// `CLAUDE.md` §5.0 declara que a cura de um tecto é o corte.
#[cfg(test)]
#[path = "brush_verb_dyntopo_tests.rs"]
mod dyntopo_tests;
