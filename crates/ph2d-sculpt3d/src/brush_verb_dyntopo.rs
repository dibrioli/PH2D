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
    /// ⭐⭐⭐ **METADE DESTA TABELA É AGORA O VEREDITO DE UM ORÁCULO LIVRE**
    /// (2026-09-14, `docs/3D/22`). O report do dono — *«algumas tools que não
    /// deveriam subdividir estão fazendo (como smooth) e algumas que deveriam
    /// não estão»* — foi **confirmado nos dois sentidos** pelo SculptGL, que é
    /// **MIT** e portanto se lê e se porta com atribuição (§0.9: *a triagem de
    /// licença pára na primeira porta ABERTA*).
    ///
    /// | verbo | o oráculo livre | nós, antes |
    /// |---|---|---|
    /// | Draw · Clay · Inflate · Flatten · Pinch · Crease | **mexe** | mexia ✓ |
    /// | **Smooth** | ⛔ **NÃO mexe** | mexia — **curado** |
    /// | **Snake Hook · Twist · Local Scale** | ⭐ **mexe** | não mexia — **curado** |
    /// | Move (agarrar) | não mexe | não mexia ✓ |
    /// | **Mask** | ⛔ **NÃO mexe, e o alvo diz-o por escrito** | curado em 14/09 ✓ |
    ///
    /// ⭐⭐ **A célula da MÁSCARA foi curada um dia ANTES do estudo, por
    /// raciocínio, e o oráculo concordou** — ele **sobrescreve** a lei geral só
    /// para ela, com um comentário a dizer que ali não há topologia dinâmica.
    /// *Uma cura que o oráculo depois confirma é a melhor prova de que o
    /// raciocínio que a produziu era do domínio e não do programa.*
    ///
    /// # ⚠️ O que ainda NÃO está respondido, e porquê
    ///
    /// O SculptGL tem **treze** ferramentas; nós temos **trinta e um** verbos.
    /// Os que só existem na outra referência — Sharpen, Fill, Scrape, Magnify,
    /// Blob, Clay Strips, Clay Thumb, Multiplane Scrape, Slide Relax, Surface
    /// Smooth, Layer, Thumb, Nudge, Pose, Cloth, Boundary — **continuam com o
    /// comportamento de hoje**, que é o valor conservador, e a resposta deles é
    /// uma corrida de oráculo **atrás da parede** (GPL ⇒ corre-se, não se lê).
    ///
    /// ⛔ **E três verbos NÃO são pergunta de oráculo nenhum:**
    ///
    /// - **`Density`** ([`Self::sem_lei_por_vertice`]) — ele **é** o passe de
    ///   topologia; perguntar se ele o dispara é perguntar se ele existe.
    /// - **`EraseMultires` e `SmearMultires`** — eles exigem uma **pilha de
    ///   multiresolução**, e a espec §5.6 escreve que combinar isso com
    ///   topologia dinâmica é **incoerente por construção** (uma pilha de níveis
    ///   e uma malha que muda de contagem não coexistem). A porta já os recusa
    ///   com a pilha montada, por outro guarda.
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
            Self::SnakeHook | Self::Twist | Self::LocalScale => true,
            // ⛔ **O AGARRAR não mexe**, e o oráculo livre concorda com o que já
            // fazíamos: ele desloca uma região inteira **sem a esticar** contra
            // o resto — o barro viaja junto, e a densidade dele viaja com ele.
            Self::Move => false,
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
            // ⚠️ Os ancorados que sobram (`Thumb`, `Nudge`, `Pose`, `Cloth`)
            // continuam a **não** alcançar a porta, e esse `false` é o estado
            // actual, não uma resposta.
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
    /// | **Twist** | `true` | ⭐⭐ **oráculo LIVRE** — chama-a |
    /// | **Local Scale** | `true` | ⭐⭐ **oráculo LIVRE** — chama-a |
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
        const DESVIOS: [(&str, bool, &str); 7] = [
            (
                "Smooth",
                false,
                "oraculo livre (MIT): nao chama a topologia",
            ),
            ("Mask", false, "dominio: um gesto que nao move um vertice"),
            ("Snake Hook", true, "oraculo livre (MIT): chama a topologia"),
            ("Twist", true, "oraculo livre (MIT): chama a topologia"),
            (
                "Local Scale",
                true,
                "oraculo livre (MIT): chama a topologia",
            ),
            ("Erase Displacement", false, "espec §5.6: multirresolucao"),
            ("Smear Displacement", false, "espec §5.6: multirresolucao"),
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
        let esperados: Vec<&str> = DESVIOS.iter().map(|(n, _, _)| *n).collect();
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

    /// ⚠️⚠️ **OS VERBOS COM ÂNCORA QUE AINDA NÃO FORAM MEDIDOS CONTINUAM A NÃO
    /// MEXER**, e esse `false` é o valor CONSERVADOR — não uma afirmação.
    ///
    /// ⭐ **Três deles SAÍRAM desta lista em 2026-09-14** (Snake Hook · Twist ·
    /// Local Scale), porque o oráculo livre respondeu por eles. Os que ficam —
    /// **Move · Cloth · Thumb · Nudge · Pose** — não têm resposta ainda: o
    /// `Move` porque o oráculo diz que ele **não** mexe (e isso é uma resposta,
    /// que a tabela regista), e os outros quatro porque só existem na
    /// referência que está atrás da parede.
    ///
    /// ⛔ E eles são precisamente os que mais **esticam** superfície (a pose
    /// roda um membro inteiro), logo são os candidatos mais fortes a mudar de
    /// valor quando a outra metade chegar — este gate existe para essa mudança
    /// ser deliberada.
    #[test]
    fn os_ancorados_por_medir_continuam_a_nao_mexer() {
        /// Os que o oráculo livre JÁ respondeu, e por isso saíram desta lista.
        const RESPONDIDOS: [&str; 3] = ["Snake Hook", "Twist", "Local Scale"];
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
        // ⭐ **O piso dos POR MEDIR**: sem ele, responder a todos de uma vez
        // deixaria este gate verde sobre uma lista vazia.
        let por_medir: Vec<&str> = ancorados
            .iter()
            .copied()
            .filter(|n| !RESPONDIDOS.contains(n))
            .collect();
        assert!(
            por_medir.len() >= 5,
            "só {} ancorados por medir — ou a outra metade chegou (e este gate \
             tem de encolher com a tabela ao lado), ou a varredura partiu-se: \
             {por_medir:?}",
            por_medir.len()
        );
        for v in Verb::ALL
            .into_iter()
            .filter(|v| v.anchors() && !RESPONDIDOS.contains(&v.label()))
        {
            assert!(
                !v.refina_no_dyntopo() && !v.colapsa_no_dyntopo(),
                "`{}` tem âncora, não foi medido, e declara que mexe na \
                 topologia — isto muda comportamento sem oráculo nenhum",
                v.label()
            );
        }
        // ⭐ **E os respondidos MEXEM**, senão a lista de isenções descreveria
        // uma resposta que o produto não dá.
        for nome in RESPONDIDOS {
            let v = Verb::ALL
                .into_iter()
                .find(|v| v.label() == nome)
                .unwrap_or_else(|| panic!("`{nome}` saiu do catálogo"));
            assert!(
                v.refina_no_dyntopo() && v.colapsa_no_dyntopo(),
                "`{nome}` está na lista dos respondidos e não mexe — a lista \
                 deixou de descrever alguma coisa"
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
