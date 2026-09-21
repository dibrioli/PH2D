//! **O que o artista autora, e o que ele mandou fazer.**
//!
//! ## Por que intents, e não `ToolPanelEvent`
//!
//! Todo painel docado deste app encaminha para uma **tool**
//! (`EditorAction::ToolPanelEvent` → `tool.handle_panel_event`). A cena 3D não é
//! uma `Tool` e não pode ser — a navegação orbital mora no shell justamente para
//! manter o contrato congelado intacto (ADR-0150) —, então este painel segue o
//! precedente do de física: empurra intents numa fila e a ponte do shell as
//! drena. O shell continua a única coisa que toca a `Sculpt3dScene`.
//!
//! ## Por que o estado autorado é UMA struct
//!
//! [`Sculpt3dUi`] junta tudo o que o artista **ajusta** — o pincel, o raio, o
//! espelho, a cavidade, a luz, o detalhe. O painel recebe uma cópia por frame,
//! edita UM campo e devolve a struct INTEIRA ([`Sculpt3dIntent::SetUi`]). Um
//! intent por knob seriam quinze maneiras de dizer a mesma coisa e quinze
//! lugares para o shell esquecer um; é a mesma lei do `PhysicsIntent::SetSettings`.
//!
//! ⚠️ **O que NÃO está nela são os gestos com consequência** — subdividir,
//! remalhar, apagar uma peça. Esses não são um valor que se ajusta, são uma
//! coisa que ACONTECE, e enfiá-los num `SetUi` faria toda mexida de slider ter
//! de decidir se o remesh já rodou.

use ph2d_mesh::Extract;
use ph2d_sculpt3d::{Brush, ClothFilterOrientation, FilterKind, FilterLaw, Symmetry, Verb};

/// ⚠️ **A memória por-verbo mudou-se para o irmão [`crate::slots`], e o caminho
/// NÃO mudou.** O shell endereça `state::VerbSlot` e `state::switch_verb_parts`,
/// e um arch-gate lê o fonte do nascimento atrás de `VerbSlot::for_verb`;
/// re-exportar mantém os dois honestos e deixa o corte ser sobre
/// responsabilidade em vez de sobre quem tem de reescrever um `use`.
pub use crate::slots::{
    VerbSlot, arm_mode_defaults, reconcile_mode, switch_verb, switch_verb_parts, verb_index,
};

/// **As ESCOLHAS nomeadas** — `UiLevel` e `RetopoMode` — ver [`crate::state_modes`].
///
/// ⚠️ **Re-exportadas daqui de propósito:** o corte foi do teto de LOC, e nenhum
/// caminho de chamador muda por causa dele.
pub use crate::state_modes::{DetalheDaTinta, RetopoMode, UiLevel};

/// **COM QUE LUZ** — ver [`luz`]. Irmão (`#[path]`), e o corte foi forçado pelo
/// tecto de LOC deste painel (`645` contra `600`) mais o assunto: *«que modos de
/// luz existem»* é uma pergunta própria, como a dos modos e a do canal.
#[path = "state_luz.rs"]
mod luz;
pub use luz::LightMode;

/// **O estado AUTORADO da cena 3D** — tudo o que um controle contínuo ou um
/// rádio deste painel escreve.
#[derive(Clone, Debug, PartialEq)]
pub struct Sculpt3dUi {
    /// **QUAL LEI o filtro roda** — e ela **não é o verbo em mãos**.
    ///
    /// ⚠️ **Um VALOR e não um fato, ao contrário do `filter_armed`:** armar
    /// muda o que o botão esquerdo FAZ (uma consequência, que por isso viaja
    /// fora deste struct), e escolher a lei muda só qual campo o gesto já
    /// armado aplica. Ela viaja aqui, com os outros valores que todo arrasto de
    /// slider reenvia inteiro.
    ///
    /// ⚠️ **E ela é GLOBAL, não por-verbo:** o [`Self::slots`] guarda o pincel
    /// de cada ferramenta porque *afinar a força do Smooth não é afinar a do
    /// Clay*, e uma lei de filtro não pertence a ferramenta nenhuma — três das
    /// sete não têm verbo. O modelo é o do operador *Mesh Filter* da
    /// referência: um Type, escolhido uma vez.
    pub filter_law: FilterLaw,
    /// **O REFERENCIAL do filtro de tecido** (espec §7) — a direcção do «baixo»
    /// da Gravidade e os eixos da Escala. Global como a lei, e pela mesma razão.
    pub cloth_filter_orientation: ClothFilterOrientation,
    /// ⭐⭐⭐ **AS PROPRIEDADES DO FILTRO DE TECIDO** — dele, e não do pincel.
    ///
    /// ⛔⛔ **Elas eram lidas do PINCEL** (`brush.cloth_mass` e as duas irmãs), e
    /// isso estava errado em três sítios: o amortecimento do filtro nasce em `0`
    /// e o do pincel em `0,01` **com a faixa a começar aí** (⇒ o filtro nunca
    /// alcançava o próprio valor de omissão); a plasticidade do alvo é fixa em
    /// `0` no filtro; e as três só apareciam no painel com o **pincel** de
    /// tecido na mão, enquanto o filtro corre com qualquer verbo. Ver
    /// [`ph2d_sculpt3d::ClothFilterProps`].
    ///
    /// ⚠️ **GLOBAIS como a orientação acima, e pela mesma razão**: um filtro não
    /// pertence a ferramenta nenhuma — três dos cinco tipos não têm verbo.
    pub cloth_filter: ph2d_sculpt3d::ClothFilterProps,
    /// ⭐⭐ ***Force Axis*** — quais eixos a Escala do filtro usa (espec §7).
    ///
    /// ⛔ **Era um controlo que NÃO EXISTIA** — e a distinção importa, porque a
    /// cura de um knob morto é ligar o braço e a de um controlo ausente é
    /// criá-lo. O motor já o honra desde 07/09 (gate `escala_eixox` a
    /// `0,009151`); o que faltava eram os três interruptores.
    ///
    /// ⚠️ **Só a Escala os lê**, e quem o diz é o MOTOR
    /// ([`ph2d_sculpt3d::ClothFilterKind::le_os_eixos`]) — nunca uma lista de
    /// nomes aqui.
    pub cloth_filter_axes: [bool; 3],
    /// O verbo, a curva, a força e os dois knobs condicionais.
    ///
    /// ⚠️ O `Brush::radius` é de MUNDO e **derivado por dab** (contra a câmera e
    /// o ponto de acerto), então ele viaja aqui e ninguém o edita: quem o artista
    /// ajusta é o [`Sculpt3dUi::radius_px`] logo abaixo. Guardar os dois num
    /// campo só seria a segunda resposta a *"que tamanho tem o pincel?"*.
    pub brush: Brush,
    /// **O QUE CADA VERBO LEMBRA** — o pincel INTEIRO daquela ferramenta.
    ///
    /// ⚠️ **Um ajuste é da FERRAMENTA, nunca do módulo** (ordem do Enio,
    /// 2026-08-17: *"as configurações dos parâmetros de cada tool não devem se
    /// propagar para outra tool"*). Afinar a força do Smooth e pegar o Clay não
    /// pode mover o Clay: o artista calibrou uma ferramenta, não o painel.
    ///
    /// ⚠️ **A tabela guarda o `Brush` INTEIRO, e não uma lista de campos.** Ela
    /// nasceu como `mode_by_verb` — só a referência por verbo — e a lista do que
    /// *também* devia lembrar (força, curva, dureza, alisamento, altura, raio…)
    /// é exatamente a que apodrece: o campo que a próxima wave acrescentar ao
    /// `Brush` nasce lembrado, sem ninguém o registar aqui. É o preço de um
    /// `clone()` por troca de ferramenta, num gesto que o artista faz com a mão.
    ///
    /// ⚠️ **O `radius_px` viaja JUNTO** porque ele é o tamanho do pincel medido
    /// na régua que o artista vê (pixels de tela) e o `Brush::radius` é o
    /// derivado de mundo: separá-los faria a ferramenta lembrar tudo menos o
    /// próprio tamanho.
    ///
    /// ⚠️ **O slot do verbo VIVO é uma cópia MORTA** — a verdade dele é o
    /// [`Sculpt3dUi::brush`], e o slot só é reescrito quando o artista SAI
    /// daquela ferramenta ([`switch_verb`]). Ler o slot do verbo atual responde
    /// *"como ele estava quando eu o larguei"*, que é a pergunta errada.
    pub slots: [VerbSlot; Verb::ALL.len()],
    /// **COM QUE PROFUNDIDADE O PAINEL SE MOSTRA** — ver [`UiLevel`].
    ///
    /// ⚠️ **Ele mora aqui, e não numa célula do painel, pela razão do `matcap`
    /// logo abaixo:** o painel recebe um retrato NOVO a cada frame e não guarda
    /// nada entre eles, então a escolha tem de viajar no mesmo struct que o resto
    /// — uma célula `thread_local` seria um segundo lugar onde o estado do painel
    /// vive, com um ciclo de vida próprio para divergir.
    ///
    /// ⚠️ **E ele NÃO é salvo**, também como o `matcap`: com que profundidade
    /// olhar não muda a escultura.
    pub ui_level: UiLevel,
    /// ⭐⭐⭐⭐ **A RESOLUÇÃO DA TINTA na peça activa** — ver
    /// [`DetalheDaTinta`].
    ///
    /// ⚠️ **Ela é do PRODUTO e não do pincel**, e a fronteira é a que os dois
    /// sliders `Detail` do `Density` pagaram: *quão fina a MALHA fica debaixo de
    /// um traço* é uma pergunta, *quão fina a TINTA é nesta peça* é outra.
    ///
    /// ⚠️ E ao contrário do [`Self::ui_level`] ela **muda a escultura** — o
    /// plano é onde a tinta fina vive —, logo o dia em que a cena for salva ela
    /// vai junto. *Hoje nenhuma cor viaja no `.ph2dproj`, e essa dívida é a
    /// mesma dela.*
    pub tinta_detalhe: DetalheDaTinta,
    /// O raio autorado, em **pixels de tela**.
    pub radius_px: f32,
    pub symmetry: Symmetry,
    /// Quanto a curvatura escurece a fresta e clareia a crista.
    pub cavity: f32,
    /// **Quanto do AMBIENTE COM DIREÇÃO entra.** `0` = o piso escalar de ontem,
    /// ao byte; `1` = o estúdio (céu em cima, ricochete embaixo).
    pub env: f32,
    /// Quanto do AO ASSADO entra. Nasce em zero, e não por timidez: o canal só
    /// existe depois de um bake, então qualquer default acima de zero faria a
    /// peça escurecer sozinha no instante do primeiro bake.
    pub ao: f32,
    /// Quanto do AO DE TELA entra — o irmão MEDIDO do de cima, e ele nasce
    /// LIGADO porque nunca fica velho.
    pub ssao: f32,
    /// Quanto do espalhamento sub-superficial entra.
    pub sss: f32,
    /// Até onde a luz viaja, como **FRAÇÃO do maior lado da peça**.
    ///
    /// ⚠️ Fração e não comprimento, e é o que impede a segunda verdade: o alcance
    /// continua sendo função do tamanho da escultura (crescer a peça não deixa o
    /// número velho), e o artista ganha o controle de LOOK que faltava.
    pub sss_scatter: f32,
    /// Azimute da lâmpada selecionada, em graus.
    pub light_az_deg: f32,
    /// Elevação da lâmpada selecionada, em graus.
    pub light_elev_deg: f32,
    /// **COM QUE LUZ** — ver [`LightMode`].
    ///
    /// ⚠️ Ele mora no estado AUTORADO e não nos fatos porque o artista o escolhe;
    /// mas ele **não é do documento** (o shell não o salva) — escolher com que
    /// luz olhar não muda a escultura.
    pub lighting: LightMode,
    /// **O padrão do pincel, VISTO NO BARRO** antes de o traço acontecer.
    ///
    /// ⚠️ Nasce **LIGADO**: o preview responde *"esta densidade serve para a
    /// MINHA peça?"*, que é a pergunta que o artista faz no instante em que
    /// escolhe um padrão — e uma resposta que ele tem de procurar num checkbox é
    /// uma resposta que a maioria nunca vê. O interruptor existe porque o tinto
    /// cobre a peça, e há hora de querer o barro limpo.
    pub alpha_preview: bool,
    /// A malha de arestas por cima da forma.
    pub wireframe: bool,
    /// **A vista da GRADE** — o arame esconde a diagonal de cada triângulo.
    ///
    /// ⚠️ Ela só tem sujeito com o [`Self::wireframe`] ligado, e a lei da tabela
    /// de interruptores diz isso: *um controlo de uma vista que não está
    /// desenhada é um controlo morto.*
    pub wire_grade: bool,
    /// **O ALVO DE DENSIDADE do passe de topologia dinâmica** — uma FRACÇÃO em
    /// `0..=1` contra o raio do pincel, nunca um comprimento.
    ///
    /// ⚠️⚠️ **Era um índice `u8` em `DETAIL_STEPS`** (três chips: grosso · médio
    /// · fino) e passou a ser o próprio número em 2026-09-14, por report do
    /// dono: *«porque não temos um slider neste pincel para definir a densidade
    /// da malha»*. ⛔ A tecla `U` continua a ciclar os três degraus com nome —
    /// ela escreve neste mesmo campo, e por isso **não há duas fontes**: o
    /// atalho e a pista dizem sempre a mesma coisa, como o `[`/`]` e a pista do
    /// raio.
    pub dyn_detail: f32,
    /// **Em que resolução o botão RECONSTRUIR voxeliza.**
    ///
    /// ⚠️ Nasce no `ph2d_sdf::DEFAULT_RESOLUTION`, que é o número da referência
    /// SculptGL — e agora é um ponto de partida, não um teto: até esta wave ele
    /// era o único valor alcançável, cravado nos dois chamadores.
    pub remesh_res: f32,
    /// **Quão FINA a retopologia persegue a grade** — fração do curso (`0` grossa,
    /// `1` o mais fino que a entrada resolve). ⚠️ **Não é um tamanho**; o porquê
    /// está na row (`rows_topology::TOPOLOGY`).
    /// ⭐ **QUAL MOTOR de retopologia** — ver [`RetopoMode`].
    pub retopo_mode: RetopoMode,
    pub quad_detail: f32,
    /// Quanto a densidade segue a curvatura — `0` uniforme, `1` a faixa inteira.
    pub quad_adapt: f32,
    /// **O que o botão de extract vai fazer** — a espessura da casca e quantas
    /// passadas a costura recebe.
    ///
    /// ⚠️ **O tipo é o do KERNEL**, e não dois `f32` soltos: o
    /// `Extract::default()` é a única fonte destes dois números, e copiá-los
    /// para cá deixaria o painel mostrando o default antigo no dia em que o
    /// kernel mudasse o dele.
    pub extract: Extract,
}

impl Default for Sculpt3dUi {
    fn default() -> Self {
        Self {
            // ⚠️ **O default é o `Smooth`, e ele é DERIVADO** do primeiro do
            // `FilterKind::ALL` em vez de escrito: a lista é a ordem em que os
            // chips aparecem, e o chip aceso ao abrir tem de ser o primeiro
            // dela. Escrever o nome aqui seria a segunda resposta a *"qual lei
            // abre selecionada?"*, que diverge no dia em que a lista for
            // reordenada.
            filter_law: FilterLaw::Mesh(FilterKind::ALL[0]),
            cloth_filter_orientation: ClothFilterOrientation::default(),
            // ⚠️ **O default é a MALHA**, que é o caminho de sempre ao bit —
            // e ele é DERIVADO pelo `#[default]` do enum, e não escrito.
            tinta_detalhe: DetalheDaTinta::default(),
            cloth_filter: ph2d_sculpt3d::ClothFilterProps::default(),
            cloth_filter_axes: [true; 3],
            brush: Brush::default(),
            // ⚠️ **DERIVADO, e não `[RefMode::default(); _]`:** o `S` não
            // declara o [`Verb::ClayStrips`] — o SculptGL não tem essa
            // ferramenta —, então carimbar o default em todo verbo deixava a
            // faixa com um chip que o painel não oferece, ou seja **nenhum
            // aceso**. Cada verbo abre no primeiro modo que o declara, que é o
            // `S` onde ele existe e a referência da própria tool onde não.
            //
            // ⚠️ **E a derivação mudou-se para o motor** ([`RefMode::birth_for`])
            // porque este `Default` **não era o que shipava**: a shell nascia com
            // o `[RefMode::default(); N]` que este comentário recusa, e sete
            // verbos rodavam a lei de força de uma referência que não os tem.
            // Dois lugares respondendo a mesma pergunta, e quem ganhava era o
            // que ninguém tinha escrito de propósito.
            slots: std::array::from_fn(|i| VerbSlot::for_verb(Verb::ALL[i])),
            // ⚠️ **BASIC, e a razão é a mesma do `RefMode::default() == S`:** a
            // tese deste módulo é que a referência do SculptGL é a linha de base
            // sã, e um painel que abre mostrando mais knobs do que a referência
            // que o kernel roda é o painel discordando do motor. O que o Basic
            // esconde é exatamente o conjunto de rows cujo valor o slot do
            // verbo já traz — e o chip que as revela fica no topo da própria
            // seção, a um clique e nomeando-se.
            ui_level: UiLevel::default(),
            radius_px: 50.0, // LITERAL-PX-OK: espelha o DEFAULT_RADIUS_PX do shell (raio de pincel, medido)
            symmetry: Symmetry::default(),
            cavity: 0.0,
            // ⚠️ **ZERO, como a `cavity` acima**: é um canal que muda a leitura de
            // toda escultura já feita, e dois gates de GPU cobram que a luz do
            // barro e a da tinta concordem no estado inicial. Como os vizinhos, o
            // valor que vale é o que o snapshot do shell escreve.
            env: 0.0,
            ao: 0.0,
            // ⚠️ **LIGADO, ao contrário do vizinho.** O assado nasce em zero
            // porque é um canal que não existe até alguém apertar um botão; este
            // é medido a cada frame, então "ligado" quer dizer *mostre o que foi
            // medido*, e antes da primeira medição ele é inerte ao byte.
            ssao: 1.0,
            // ⚠️ **Zero, e a assimetria com o vizinho de cima é deliberada:** os
            // dois AOs são MEDIÇÕES da forma e mostrá-las é honesto; isto é um
            // MATERIAL, e barro não é pele. O `Sculpt3dScene` é a fonte destes
            // dois — este default só existe para o painel nascer coerente antes
            // do primeiro snapshot.
            sss: 0.0,
            // ⚠️ O literal segue o precedente do `ssao: 1.0` acima: o painel é
            // agnóstico ao renderer de propósito, e o valor que vale é o que o
            // snapshot do shell escreve no primeiro frame. Ele espelha o
            // `ph2d_mesh_render::sss::SCATTER_FRACTION`, que é a fonte.
            sss_scatter: 0.25, // LITERAL-PX-OK: fração adimensional, não métrica de layout
            light_az_deg: 0.0,
            light_elev_deg: 45.0, // LITERAL-PX-OK: graus de elevacao, nao metrica de design
            // ⚠️ **O literal segue o precedente do `ssao`/`sss_scatter` acima:**
            // este painel é agnóstico ao renderizador de propósito, e o valor
            // que vale é o que o snapshot da shell escreve no primeiro frame —
            // a fonte é `ph2d_mesh_render::DEFAULT_MATCAP`. Ele espelha para que
            // uma fixture de seam veja o mesmo mundo que o artista vê.
            lighting: LightMode::Matcap(0),
            alpha_preview: true,
            wireframe: false,
            wire_grade: false,
            // O MEIO da faixa, que é o valor com que a cena nasce — a fonte é
            // o `Dyntopo::default` do lado da cena, e este espelho existe só
            // para uma fixtura de costura ver o mesmo mundo que o artista vê.
            dyn_detail: 0.5, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
            // A fonte é a const do motor, não uma cópia dela.
            remesh_res: 150.0, // LITERAL-PX-OK: resolucao de voxel, nao metrica de layout
            // O MEIO do curso. Medido na malha da cena `=35`: 384 vertices e
            // 88,2% de quads -- nem a blocagem nem o teto da entrada.
            // ⚠️ **DERIVADO do catálogo, e não um literal:** o mesmo primeiro
            // elemento que o painel pinta como seleccionado. Um default escrito
            // duas vezes é o que diverge no dia em que a ordem da lista mudar.
            retopo_mode: RetopoMode::ALL[0],
            quad_detail: 0.5, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
            // ⭐⭐⭐ **O `Follow Curvature` NASCE NO MÁXIMO desde 2026-09-04** — report do dono,
            // com foto e seta: *«a ponta problemática ainda não tem a densidade de faces
            // adequada como as outras»*. ⛔ O `0,0` de antes tinha razão escrita (*«abre
            // UNIFORME, que é o modo cujo resultado o artista consegue prever»*) e uma medição
            // de 2026-08-28 por trás (*«pede-se 400 % e a saída move-se 7 %»*) — e **as duas
            // envelheceram**: desde então a fase zero passou a GRADUAR com renormalização
            // (`ADAPT_RATIO 16`), ganhou a **calota** por espinho (§105) e o acabamento ganhou
            // o **remate** (§108). *Uma recusa medida responde UMA pergunta, e a cadeia por
            // baixo desta mudou três vezes.*
            //
            // Medido de ponta a ponta na escultura do dono (`Detail 1`, `PH2D_RECENTER=1`):
            //
            // | `Curv` | pontas amputadas | grade pior no bico | `>60°` | envies. p50/p99 | relógio |
            // |---|---|---|---|---|---|
            // | `0` | ⛔ `2` de `5` | ⛔ `2,37` (`4` acima) | `27` | `3,9` / `29,9` | `150 s` |
            // | `0,5` | `0` de `5` | `0,93` | `10` | `4,0` / `27,1` | `208 s` |
            // | ⭐ `1` | **`0` de `5`** | **`0,95`** | **`4`** | **`3,0` / `20,0`** | **`123 s`** |
            //
            // ⚠️ **Na segunda peça (`sculpt_antes`) o `1` NÃO é grátis** e a troca fica dita: ele
            // leva `>60` de `5` a **`0`** e a grade do bico de `1,16` a `0,99` (nenhuma ponta
            // acima da barra), e paga enviesamento mediano `2,9° → 4,2°` — dentro da banda do
            // oráculo (`4,8`–`7,1°`) — e `+50 %` de relógio. *A ponta é o que o dono fotografa;
            // o enviesamento mediano é a coluna que ele nunca nomeou.*
            //
            // ⚠️ **O que o `0` faz na tela, medido:** dois dos cinco espinhos saem com **`3` a
            // `5` faces dando a volta** (contra `17`–`58` com o `1`) — o espinho fica
            // facetado enquanto o vizinho gordo sai fino, que é a foto dele.
            quad_adapt: 1.0, // LITERAL-PX-OK: fracao, nao metrica de layout

            extract: Extract::default(),
        }
    }
}

/// O que o painel precisa saber da cena neste frame: o estado autorado mais os
/// **fatos** que ele só mostra.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sculpt3dSnapshot {
    pub ui: Sculpt3dUi,
    /// **O que o transform ARMOU** — `None` é o estado normal, em que o botão
    /// esquerdo esculpe.
    ///
    /// ⚠️ **Um FATO, como o `dyntopo` ao lado, e não um campo do
    /// [`Sculpt3dUi`]:** armar uma ferramenta muda o que o BOTÃO faz, e uma
    /// consequência dessas não pode viajar dentro do struct de valores que todo
    /// arrasto de slider reenvia inteiro.
    pub transform: Option<ph2d_sculpt3d::TransformKind>,
    /// **O FILTRO está armado?** — armado, o botão esquerdo roda o verbo
    /// corrente na malha INTEIRA em vez de esculpir sob o cursor.
    ///
    /// ⚠️ **Um FATO pelo mesmo motivo do [`Self::transform`] logo acima**, e os
    /// dois são **mutuamente exclusivos** por construção na cena: eles armam o
    /// MESMO botão, e um gesto que significasse as duas coisas não teria como
    /// escolher. Guardá-los como dois `bool` independentes aqui seria dar ao
    /// painel a chance de pintar um estado que a cena não sabe representar.
    pub filter_armed: bool,
    /// A topologia dinâmica está armada? **Lido, nunca escrito por `SetUi`** —
    /// ligá-la TRIANGULA a malha, e uma consequência dessas não pode viajar
    /// dentro de um struct de valores que todo arrasto de slider reenvia.
    pub dyntopo: bool,
    /// O nível de multiresolução vivo, e quantos existem.
    pub level: usize,
    pub level_count: usize,
    /// **O AO assado descreve uma forma que não existe mais.**
    ///
    /// ⚠️ Um FATO, como `dyntopo` e `level`: o painel o MOSTRA e não o possui —
    /// quem sabe se a malha mudou desde o bake é a malha. E ele é mostrado em
    /// vez de escondido porque a obsolescência deste canal é **inerente ao
    /// desenho**: um AO velho não parece velho, parece uma escolha de
    /// iluminação.
    pub ao_stale: bool,
    /// **O NOME do sprite de onde o padrão de imagem veio** — o rótulo do chip.
    ///
    /// ⚠️ **Um FATO, e não um campo do [`Sculpt3dUi`]:** o nome não é um valor
    /// que o artista edita neste painel, é a proveniência do que ele armou; pô-lo
    /// no struct que todo arrasto de slider reenvia inteiro faria uma pista de
    /// escala poder reescrever de onde a imagem veio.
    ///
    /// ⚠️ **E ele sobrevive à troca de padrão de propósito**, porque é isso que
    /// torna o chip um SELETOR em vez de um botão que só sabe desmarcar-se: sem
    /// a lembrança, escolher `Grain` apagaria a imagem e o chip dela voltaria
    /// sendo um rótulo para nada.
    pub alpha_image_name: Option<std::sync::Arc<str>>,
    /// Quantas peças a cena tem, e se uma delas está isolada.
    pub pieces: usize,
    pub isolated: bool,
    /// Quantos vértices a malha viva tem. Zero é digno de ver: é a diferença
    /// entre *"o pincel não funciona"* e *"esta peça está vazia"*.
    pub verts: usize,
    /// **OS NOMES DOS MATERIAIS de matcap**, na ordem em que o renderizador os
    /// numera.
    ///
    /// ⚠️ **Eles chegam no retrato em vez de o painel os importar**, e a razão é
    /// uma aresta de dependência: quem os conhece é a `ph2d-mesh-render`, que
    /// carrega o `wgpu` inteiro. Um painel que a importasse passaria a compilar
    /// um backend gráfico para escrever seis palavras — e o `ph2d-panel-*` deste
    /// repo não fala com device nenhum. É o mesmo caminho de `dyntopo` e
    /// `level`: fatos que o painel MOSTRA e não possui.
    ///
    /// Vazio ⇒ só a opção do rig é pintada.
    pub matcap_keys: &'static [&'static str],
    /// **O tamanho de feature que ESTE modelo comporta** — o seed do
    /// `Alpha Scale`, de `ph2d_sculpt3d::recommended_scale`.
    ///
    /// ⚠️ **Ele chega no retrato porque é um fato do MODELO**, e o painel não tem
    /// malha — o mesmo caminho de `dyntopo`, `level` e `matcaps`: coisas que o
    /// painel MOSTRA e não possui. E ele é um fato e não um estado: o painel o
    /// usa uma vez, no gesto de armar um padrão, e a partir daí quem manda é o
    /// número autorado.
    ///
    /// ⚠️ **Ele existe porque uma escala ABSOLUTA não significa nada sem o
    /// tamanho do modelo.** A primeira versão desta wave shipou um literal, e o
    /// smoke o reprovou em uma frase: *"os poros são gigantescos"*.
    pub alpha_seed: f32,
    /// **O MAIOR LADO do modelo**, em unidades de objeto.
    ///
    /// ⚠️ **Um FATO do modelo, como o [`Self::alpha_seed`]** — e ele existe pela
    /// mesma razão que aquele: uma escala é ABSOLUTA, mas o que ela significa
    /// depende do tamanho da peça. O `alpha_seed` já é essa verdade *resolvida*
    /// (`max(vão ÷ 33, 10 × aresta)`), e é justamente por ser um `max` que ele
    /// **não devolve o vão**: numa malha grossa quem vence é a lei das dez
    /// arestas, e o tamanho do modelo some da conta. O preview precisa do vão
    /// CRU, porque é ele que diz *quanto do modelo cabe no swatch*.
    pub model_span: f32,
    /// **Há um sprite selecionado para a forma acender?**
    ///
    /// ⚠️ Um FATO que o painel MOSTRA e não possui, como `dyntopo` e `ao_stale` —
    /// e este é o único que nem a CENA 3D conhece: quem está selecionado é
    /// pergunta da cena 2D, então ele é injetado pelo bridge do shell.
    ///
    /// ⚠️ **Ele NÃO esconde o botão**, e a diferença é a queixa que o botão veio
    /// resolver: um verbo que só aparece quando já dá para usá-lo é um verbo que
    /// ninguém descobre. Ele acende uma DICA — a mesma lei do `ao_stale`: *a
    /// condição é dita, não deixada para o artista descobrir*, e a linha só
    /// existe quando há o que avisar (um aviso permanente vira moldura).
    pub has_bake_target: bool,
    /// **QUAL LEI o sprite escolhido usa hoje** — `None` quando ele ainda não
    /// está assado (e então a fileira não existe).
    ///
    /// ⚠️ **Um índice e não uma lei**, e a razão é a seta das dependências:
    /// quem define as leis é a `ph2d-form-donation`, que puxa o `wgpu` — um
    /// painel não depende dela. A tradução `índice ⇄ lei` vive **num sítio só**
    /// (`Lei::index`/`from_index`), com ida-e-volta gateada lá.
    ///
    /// ⚠️ Como o `has_bake_target`, é um FATO que o painel MOSTRA e não possui:
    /// quem está selecionado é pergunta da cena 2D, e a lei é do documento.
    pub lei_do_alvo: Option<usize>,
    /// As chaves de i18n dos rótulos da fileira acima, **na ordem dos chips**.
    ///
    /// ⛔ Elas vêm de fora (`lei_da_luz::CHAVES_DOS_ROTULOS`) em vez de serem
    /// escritas aqui: dois nomes escritos no painel seriam a **segunda
    /// ortografia** da mesma lei, e o dia em que uma terceira lei entrasse ela
    /// nasceria com o nome de outra.
    pub lei_rotulos: &'static [&'static str],
}

/// Estado retido por-instância. Vazio de propósito: a autoridade é a
/// `Sculpt3dScene` do shell, e o painel renderiza o retrato do frame.
#[derive(Clone, Debug, Default)]
pub struct Sculpt3dPanelState;

pub(crate) use crate::state_channel::{
    current, push_intent, set_last_content_h, set_last_visible_h,
};
/// **O CANAL host ⟷ painel** — cortado para o irmão [`crate::state_channel`] na integração de
/// 2026-09-10 (teto de LOC, acumulado por duas linhas). ⚠️ **Re-exportado daqui de propósito:** o
/// corte foi de tamanho, e nenhum caminho de chamador muda por causa dele.
pub use crate::state_channel::{
    drain_intents, last_content_h, last_visible_h, set_current_sculpt3d,
};

/// **AS LEITURAS DERIVADAS** — ver [`leituras`]. Irmão (`#[path]`), cortado
/// pelo tecto de LOC (`615` contra `600`) mais o assunto: *o `state.rs` é o
/// MODELO, e «qual chip está aceso?» é uma LEITURA dele.*
#[path = "state_leituras.rs"]
mod leituras;
pub use leituras::alpha_chip_index;

/// **O RAIO DE FÁBRICA, em pixels de tela** — a base contra a qual as frações
/// de raio da referência são resolvidas.
///
/// ⚠️ Ele espelha o `Sculpt3dUi::default().radius_px` e o `DEFAULT_RADIUS_PX`
/// do shell **de propósito**: é o mesmo número, e ele existe aqui como *nome*
/// porque o arming precisa perguntar *"o artista ainda está no raio de fábrica
/// deste verbo?"*, e um literal `50.0` no meio dessa comparação seria a segunda
/// resposta que a próxima wave esquece de mover.
pub const BASE_RADIUS_PX: f32 = 50.0; // LITERAL-PX-OK: raio de pincel, espelha o default do shell
