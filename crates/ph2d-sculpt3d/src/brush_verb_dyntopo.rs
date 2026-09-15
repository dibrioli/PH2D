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

    /// ⭐⭐⭐ **A TABELA, CÉLULA A CÉLULA — e metade dela já é ORÁCULO.**
    ///
    /// ⚠️⚠️ **Este gate mudou em 2026-09-14 porque o ESTUDO CHEGOU**, que é
    /// exactamente o que a redacção anterior mandava fazer (*«se foi o estudo a
    /// chegar, reescreva este gate célula a célula com a tabela medida ao
    /// lado»*). Ele dizia *«só a máscara se afasta do comportamento de hoje»*;
    /// hoje são **sete** células, e cada uma tem a proveniência escrita:
    ///
    /// | verbo | colunas | de onde veio |
    /// |---|---|---|
    /// | **Mask** | `false` | ⭐ **raciocínio de domínio**, curado em 14/09 — e o oráculo livre depois CONCORDOU |
    /// | **Smooth** | `false` | ⭐⭐ **oráculo LIVRE** (MIT) — ele não chama a topologia dinâmica, e o dono nomeou-o à letra |
    /// | **Snake Hook** | `true` | ⭐⭐ **oráculo LIVRE** — chama-a |
    /// | **Move** | `true` | ⭐⭐⭐ **o DONO** — e contra as duas referências |
    /// | **Thumb** | `true` | ⭐⭐⭐ **o DONO** — e custou a pegada congelada sobreviver ao refino |
    /// | **Erase Displacement** | `false` | ⛔ **construção**: pilha de níveis e malha que muda de contagem não coexistem (espec §5.6) |
    /// | **Smear Displacement** | `false` | ⛔ idem |
    ///
    /// ⛔ **E o resto continua a ser o comportamento de hoje**, que é o valor
    /// conservador — a metade atrás da parede ainda não chegou.
    ///
    /// ⚠️ **A régua é a PROVENIÊNCIA e não a contagem:** um verbo novo nesta
    /// lista reprova aqui, e quem o puser tem de escrever de onde veio a
    /// resposta. *Uma tabela sem proveniência por célula é um palpite com cara
    /// de lei.*
    #[test]
    fn cada_desvio_do_comportamento_de_hoje_tem_proveniencia() {
        /// `(verbo, refina?, de onde veio)` — as células que se afastam do que
        /// o produto fazia antes do estudo.
        const DESVIOS: [(&str, bool, &str); 10] = [
            // ⭐⭐ **AS DUAS REFERENCIAS CONCORDAM** — a livre (MIT, lida) e a
            // medida (corrida sem interface pela janela E).
            ("Smooth", false, "as DUAS: nao mexe (medida: 441 -> 441)"),
            ("Snake Hook", true, "as DUAS: mexe (medida: 441 -> 2 723)"),
            // ⭐ **DOMINIO** — curado um dia antes do estudo, e a referencia
            // livre depois SOBRESCREVEU a lei geral dela para dizer o mesmo.
            ("Mask", false, "dominio; e as DUAS referencias confirmaram"),
            // ⭐⭐⭐ **O DONO, com os olhos no smoke `=14`** — e contra as DUAS
            // referencias, que dizem que o agarrar nao mexe. Ver o braco delas.
            (
                "Move / Grab",
                true,
                "o DONO (14/09): «move/drag deve subdividir»",
            ),
            // ⚠️ **`Thumb`, nao `Clay Thumb`** — sao verbos DIFERENTES, e o
            // segundo e' de carimbo e ja' mexia. O que o dono pediu e' o
            // ANCORADO, que e' o unico que congela a pegada.
            (
                "Thumb",
                true,
                "o DONO (14/09): «Thumb se for possivel, deveria subdividir»",
            ),
            // ⭐ **SO' A MEDIDA responde** (nao existem na referencia livre).
            ("Slide Relax", false, "medida: 441 -> 441 nos dois extremos"),
            (
                "Surface Smooth",
                false,
                "medida: 441 -> 441 nos dois extremos",
            ),
            ("Nudge", true, "medida: 441 -> 2 853 com o alvo fino"),
            // ⛔ **CONSTRUCAO** — nao e' pergunta de oraculo nenhum.
            (
                "Erase Displacement",
                false,
                "espec §5.6: multirresolucao exclui dyntopo",
            ),
            (
                "Smear Displacement",
                false,
                "espec §5.6: multirresolucao exclui dyntopo",
            ),
        ];
        let mut corrigidos = Vec::new();
        for v in Verb::ALL {
            // O comportamento de ANTES do estudo: o refino tinha um chamador só
            // — o braço do carimbo —, e quem tem âncora entrava por outro
            // caminho.
            let antes = !v.anchors();
            if v.refina_no_dyntopo() != antes || v.colapsa_no_dyntopo() != antes {
                corrigidos.push(v.label());
            }
        }
        // ⚠️ **A comparação é por CONJUNTO e não por ordem**: a varredura sai na
        // ordem do `Verb::ALL` e a lista está agrupada por PROVENIÊNCIA, que é a
        // informação que ela existe para carregar. *Obrigar as duas ordens a
        // coincidir faria a tabela ser arrumada pela ordem do catálogo, onde a
        // proveniência deixa de se ler.*
        let mut esperados: Vec<&str> = DESVIOS.iter().map(|(n, _, _)| *n).collect();
        esperados.sort_unstable();
        corrigidos.sort_unstable();
        assert_eq!(
            corrigidos, esperados,
            "a tabela mudou uma célula sem passar por aqui. Toda célula que se \
             afasta do comportamento de antes do estudo tem de estar na lista \
             DESVIOS, com a PROVENIÊNCIA ao lado — senão é um palpite com cara \
             de lei"
        );
        // ⭐ **E o valor tem de bater com a proveniência**, senão a tabela
        // documentava uma coisa e o produto fazia outra.
        for (nome, refina, porque) in DESVIOS {
            let v = Verb::ALL
                .into_iter()
                .find(|v| v.label() == nome)
                .unwrap_or_else(|| panic!("`{nome}` saiu do catálogo e a lista ficou para trás"));
            assert_eq!(
                v.refina_no_dyntopo(),
                refina,
                "`{nome}` devia responder {refina} ({porque})"
            );
        }
    }

    /// ⭐⭐⭐ **OS NOVE VERBOS COM ÂNCORA, TODOS MEDIDOS — e quatro deles mexem.**
    ///
    /// ⚠️⚠️ **Este gate mudou de PREMISSA em 2026-09-14.** Ele chamava-se *«os
    /// ancorados por medir continuam a não mexer»* e o `false` deles era o
    /// **valor conservador**; hoje os nove têm resposta, e a lista deixou de ser
    /// uma dívida para ser uma **tabela**. *Uma catraca cuja população foi
    /// inteiramente respondida tem de mudar de forma, senão ela vira licença.*
    ///
    /// | verbo | mexe? | de onde |
    /// |---|---|---|
    /// | **Snake Hook** | ✅ | as DUAS referências |
    /// | **Nudge** | ✅ | a medida (`441 → 2 853`) |
    /// | **Move · Thumb** | ✅ | ⭐⭐⭐ **o DONO**, contra as duas referências |
    /// | **Twist · Local Scale** | ⛔ | ⭐⭐⭐ **o DONO** desempatou, a favor da medida |
    /// | Pose · Boundary · Cloth | ⛔ | as duas, onde as duas existem |
    ///
    /// ⚠️⚠️ **A CONTAGEM não se mexeu e a POPULAÇÃO trocou por baixo dela** — os
    /// que mexem continuam a ser **quatro** e são outros dois. *É exactamente
    /// como um piso segura o número enquanto a lista que ele descreve muda*, e é
    /// por isso que o corpo deste gate compara a LISTA, nunca o tamanho dela.
    #[test]
    fn os_nove_ancorados_tem_resposta_e_quatro_deles_mexem() {
        /// Os ancorados que MEXEM, com a proveniência no gate irmão.
        const MEXEM: [&str; 4] = ["Snake Hook", "Nudge", "Move / Grab", "Thumb"];
        let ancorados: Vec<&str> = Verb::ALL
            .into_iter()
            .filter(|v: &Verb| v.anchors())
            .map(Verb::label)
            .collect();
        assert!(
            ancorados.len() >= 9,
            "o censo varreu só {} verbos com âncora — a varredura partiu-se: {ancorados:?}",
            ancorados.len()
        );
        // ⭐ **O piso dos que NÃO mexem**: sem ele, ligar a porta a todo gesto
        // ancorado deixaria este gate verde sobre uma lista vazia.
        let parados: Vec<&str> = ancorados
            .iter()
            .copied()
            .filter(|n| !MEXEM.contains(n))
            .collect();
        assert!(
            parados.len() >= 5,
            "só {} ancorados parados — ligar a porta a todos passaria aqui: {parados:?}",
            parados.len()
        );
        for v in Verb::ALL.into_iter().filter(|v| v.anchors()) {
            let deve = MEXEM.contains(&v.label());
            assert_eq!(
                v.refina_no_dyntopo(),
                deve,
                "`{}` tem âncora e a tabela medida diz {deve}",
                v.label()
            );
        }
    }

    /// ⭐⭐⭐ **O VEREDITO DO DONO SOBRE CINCO CÉLULAS, PRESO AQUI** — 2026-09-14,
    /// depois de ele correr o smoke `=14`:
    ///
    /// > *«acho que layer, move/drag deve subdividir. Thumb se for possível,
    /// > deveria subdividir. Twist com dynamic topology fica com resultado muito
    /// > ruim.»*
    ///
    /// ⚠️⚠️ **ESTE GATE EXISTE PORQUE TRÊS DELAS COINCIDEM COM O VALOR DE
    /// FÁBRICA, e uma coincidência não é uma decisão.** O `Layer` é de carimbo
    /// e o `_ => !self.anchors()` já lhe responde `true`; a `Twist` e a `Local
    /// Scale` têm âncora e ele já lhes responde `false`. ⇒ escrever-lhes um
    /// braço no `match` seria uma linha que **a mutação não consegue matar** —
    /// o defeito que a densidade já pagou neste mesmo ficheiro. Mas deixá-las
    /// sem nada faria a decisão do dono depender de o `anchors()` nunca mudar,
    /// e **nada liga as duas perguntas**: quem mexer num grip amanhã inverte um
    /// veredito de produto sem que uma linha do diff o diga.
    ///
    /// ⭐ *A decisão vai para onde ela pode ser AFIRMADA — um gate —, e não para
    /// onde ela por acaso já é verdade.*
    ///
    /// ⚠️ **A `Local Scale` é a célula que ele NÃO nomeou**, e está aqui de
    /// propósito: ela seguiu a irmã (mesmo grip, mesma célula nas duas
    /// referências, mesma família na medida). Se ele a quiser de volta a
    /// adensar, é esta linha que muda — *uma herança silenciosa e uma decisão
    /// leem-se igual numa tabela, e é a lista que as separa.*
    #[test]
    fn o_veredito_do_dono_sobre_cinco_celulas() {
        /// `(verbo, mexe?, o que ele disse)`.
        const VEREDITO: [(&str, bool, &str); 5] = [
            ("Layer", true, "«acho que layer … deve subdividir»"),
            ("Move / Grab", true, "«move/drag deve subdividir»"),
            ("Thumb", true, "«Thumb se for possivel, deveria subdividir»"),
            ("Twist", false, "«Twist … fica com resultado muito ruim»"),
            (
                "Local Scale",
                false,
                "nao nomeado: seguiu a Twist, que e' o mesmo grip e a mesma \
                 celula nas duas referencias",
            ),
        ];
        for (nome, mexe, disse) in VEREDITO {
            let v = Verb::ALL
                .into_iter()
                .find(|v| v.label() == nome)
                .unwrap_or_else(|| {
                    panic!("`{nome}` saiu do catálogo e o veredito ficou para trás")
                });
            assert_eq!(
                v.refina_no_dyntopo(),
                mexe,
                "`{nome}` devia responder {mexe} — o dono julgou-o no smoke `=14`: {disse}"
            );
            assert_eq!(
                v.colapsa_no_dyntopo(),
                mexe,
                "`{nome}`: as duas colunas separaram-se debaixo de um veredito \
                 de produto"
            );
        }
    }

    /// ⛔⛔ **OS DOIS VERBOS QUE NENHUMA DAS DUAS REFERÊNCIAS RESPONDE, e eles
    /// ficam NOMEADOS em vez de silenciosos.**
    ///
    /// O **Sharpen** e o **Magnify** não têm equivalente na referência livre nem
    /// tipo próprio na medida — a corrida da janela E cobriu **27** tipos e
    /// nenhum é um deles. ⇒ os dois ficam com o comportamento de antes do
    /// estudo (**mexem**, por serem carimbo), e isso é o **valor conservador**,
    /// não uma resposta.
    ///
    /// ⚠️ **Sem este gate eles leem-se como decididos** — uma célula sem
    /// proveniência e uma com proveniência têm exactamente o mesmo aspecto numa
    /// tabela. *É a mesma doença do `❌ recusado com motivo` contra o `❌ ninguém
    /// fez` que o §5 deste repo já nomeia.*
    #[test]
    fn os_dois_verbos_sem_oraculo_ficam_nomeados() {
        const SEM_ORACULO: [&str; 2] = ["Sharpen", "Magnify"];
        for nome in SEM_ORACULO {
            let v = Verb::ALL
                .into_iter()
                .find(|v| v.label() == nome)
                .unwrap_or_else(|| {
                    panic!("`{nome}` saiu do catálogo e esta lista ficou para trás")
                });
            assert!(
                v.refina_no_dyntopo() && v.colapsa_no_dyntopo(),
                "`{nome}` mudou de valor e continua na lista dos SEM ORÁCULO — \
                 se alguém o mediu, a entrada tem de sair com a medição ao lado"
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
