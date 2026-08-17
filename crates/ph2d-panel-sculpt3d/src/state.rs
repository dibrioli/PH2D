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
use ph2d_sculpt3d::{Brush, Symmetry, Verb};

/// ⚠️ **A memória por-verbo mudou-se para o irmão [`crate::slots`], e o caminho
/// NÃO mudou.** O shell endereça `state::VerbSlot` e `state::switch_verb_parts`,
/// e um arch-gate lê o fonte do nascimento atrás de `VerbSlot::for_verb`;
/// re-exportar mantém os dois honestos e deixa o corte ser sobre
/// responsabilidade em vez de sobre quem tem de reescrever um `use`.
pub use crate::slots::{
    VerbSlot, arm_mode_defaults, reconcile_mode, switch_verb, switch_verb_parts, verb_index,
};
use std::cell::{Cell, RefCell};

thread_local! {
    /// O retrato vivo que o host publica antes de cada `paint`. `None` até a
    /// cena 3D existir — e é isso que faz o painel se recusar a pintar.
    static CURRENT: RefCell<Option<Sculpt3dSnapshot>> = const { RefCell::new(None) };
    /// O que o artista fez, esperando o shell drenar.
    static INTENTS: RefCell<Vec<Sculpt3dIntent>> = const { RefCell::new(Vec::new()) };
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// **COM QUE PROFUNDIDADE O PAINEL SE MOSTRA** (§2 do plano).
///
/// ⚠️ **Isto não são dois conjuntos de features — é divulgação progressiva do
/// MESMO estado**, e essa escolha é o que impede duas fontes de verdade. Em
/// `Pro` o artista não ganha números novos: ele ganha *acesso* aos números que o
/// verbo e o modo já haviam armado por ele.
///
/// ⚠️ **A regra de quem pode ser `Pro`, e ela é testável:** só uma row cujo
/// valor **o slot do verbo já traz** ([`VerbSlot::for_verb`]). Esconder um
/// número que a ferramenta escolheu bem é divulgação progressiva; esconder um
/// que nasce neutro e tem de ser fornecido é amputação — o artista ficaria com
/// uma ferramenta que não faz o que o nome dela diz e sem nada na tela
/// explicando por quê.
///
/// ⚠️ **Ela é NECESSÁRIA e não suficiente, e é isso que o falloff custou:** a
/// curva nasce no slot do verbo, logo *podia* ser `Pro` — e era, e o
/// smoke reprovou (*"não dá a opção de escolher o falloff e deveria dar"*).
/// Quem decide a segunda metade é a REFERÊNCIA, medida e não lembrada: no
/// Blender a curva é *dobrada* (`DEFAULT_CLOSED` com cabeçalho à vista, mais um
/// popover no cabeçalho de ferramenta), nunca *ausente*. **Dobrar é divulgação
/// progressiva; sumir sem rastro é amputação**, e o nosso `Pro` fazia o segundo.
///
/// ⚠️ **`Ord` é a lei inteira:** uma row aparece quando `nível do painel >=
/// nível da row`. Escrito como dois `if`s (um por lado) o terceiro degrau nasce
/// fora da regra.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiLevel {
    /// O que TODO pincel tem: o verbo, a referência, o raio, a força e a
    /// **CURVA**.
    ///
    /// ⚠️ **Isto dizia *"o vocabulário do SculptGL"* e a frase custou um
    /// smoke.** O SculptGL **não tem** seletor de curva — a dele é fixa —, então
    /// herdar o vocabulário dele apagava do Basic um controle que a nossa malha
    /// tem **doze** vezes e que a OUTRA referência trata como primeiro-classe (o
    /// `FalloffPanel` do Blender não é `brush_settings_advanced`, e no cabeçalho
    /// de ferramenta ele é um popover sempre visível). *Um vocabulário herdado
    /// descreve a ferramenta de onde veio, não a que se está a construir.*
    #[default]
    Basic,
    /// Mais os knobs que o modo tinha armado.
    Pro,
}

impl UiLevel {
    /// A ordem em que os chips são pintados. **É** a ordem do enum.
    pub const ALL: [Self; 2] = [Self::Basic, Self::Pro];

    /// Chave i18n do rótulo.
    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => ph2d_i18n::tr("panel.sculpt3d.ui_level.basic"),
            Self::Pro => ph2d_i18n::tr("panel.sculpt3d.ui_level.pro"),
        }
    }

    /// **Uma coisa que exige `needs` aparece neste nível?** A porta única — o
    /// pintor a consulta para desenhar e o gate de costura para varrer.
    pub fn shows(self, needs: Self) -> bool {
        needs <= self
    }
}

/// **O estado AUTORADO da cena 3D** — tudo o que um controle contínuo ou um
/// rádio deste painel escreve.
#[derive(Clone, Debug, PartialEq)]
pub struct Sculpt3dUi {
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
    /// **COM QUE LUZ** — `None` é o rig do artista, `Some(i)` é o matcap `i`.
    ///
    /// ⚠️ Ele mora no estado AUTORADO e não nos fatos porque o artista o escolhe;
    /// mas ele **não é do documento** (o shell não o salva) — escolher com que
    /// luz olhar não muda a escultura.
    pub matcap: Option<u8>,
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
    /// Qual degrau de detalhe a topologia dinâmica usa (índice em `DETAIL_STEPS`).
    pub detail: u8,
    /// **Em que resolução o botão RECONSTRUIR voxeliza.**
    ///
    /// ⚠️ Nasce no `ph2d_sdf::DEFAULT_RESOLUTION`, que é o número da referência
    /// SculptGL — e agora é um ponto de partida, não um teto: até esta wave ele
    /// era o único valor alcançável, cravado nos dois chamadores.
    pub remesh_res: f32,
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
            matcap: Some(0),
            alpha_preview: true,
            wireframe: false,
            detail: 1,
            // A fonte é a const do motor, não uma cópia dela.
            remesh_res: 150.0, // LITERAL-PX-OK: resolucao de voxel, nao metrica de layout
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
    pub matcaps: &'static [&'static str],
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
}

/// Um gesto do artista, para o shell aplicar.
///
/// ⚠️ **O `SetUi` é MUITO maior que os irmãos, e a caixa não entra — o
/// precedente é o `Step` do `ph2d-ui-state`.** Ele carrega o estado autorado
/// inteiro de propósito (é *"substitua o pincel por este"*, não *"mude este
/// campo"*), e a fila tem **um punhado de elementos por gesto**, drenada no mesmo
/// frame: um `Box` compraria bytes numa fila efémera ao preço de uma indireção e
/// de uma alocação por clique. ⚠️ E o aviso **nasceu de crescer o `Brush`** —
/// a lâmina em V lhe acrescentou dois campos —, o que quer dizer que ele mede a
/// LARGURA do estado autorado e não um defeito desta fila.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum Sculpt3dIntent {
    /// Substitui o estado autorado inteiro — ver [`Sculpt3dUi`].
    SetUi(Sculpt3dUi),
    /// **Arma (ou desarma) o transform.** Ver `ph2d_sculpt3d::TransformKind`.
    ///
    /// ⚠️ Ele carrega o TIPO e não um `Option`: a cena é quem sabe o que já
    /// está armado, e mandar *"desligue"* de dentro do painel exigiria que ele
    /// guardasse uma segunda cópia do arm para decidir. Clicar o aceso desarma —
    /// e quem faz essa conta é `Sculpt3dScene::arm_transform`, uma vez.
    ArmTransform(ph2d_sculpt3d::TransformKind),
    /// **Re-arma a IMAGEM que a cena lembra** — o chip do slot de imagem.
    ///
    /// ⚠️ **Um intent e não um `SetUi`**, porque o painel **não tem** a imagem:
    /// quando o artista escolhe `Grain`, o `Arc<AlphaImage>` sai do
    /// [`Sculpt3dUi`] e só a cena continua a segurá-lo. Sem esta porta o chip da
    /// imagem seria um controle que só sabe deixar de estar aceso.
    ArmStoredImage,
    /// Liga/desliga a topologia dinâmica (e triangula, se ligar).
    ToggleDyntopo,
    /// Desce (`false`) ou sobe (`true`) um nível de multiresolução.
    ChangeLevel(bool),
    Subdivide,
    ReverseLevel,
    /// Achata a pilha de multiresolução numa malha só.
    Flatten,
    Remesh,
    CloseHoles,
    /// Mede quanto do céu cada vértice enxerga e instala o canal.
    ///
    /// ⚠️ Um comando e não um knob: o bake custa ~338 ms na malha da cena `=16`,
    /// então ele é um gesto que o artista PEDE, nunca um passe que roda sozinho.
    BakeAo,
    /// **Assa a FORMA no sprite selecionado** — o objetivo 2 (`docs/3D/02.2`).
    ///
    /// ⚠️ **É o único intent que a cena 3D não sabe executar**, e isso é o
    /// desenho: o bake precisa do mundo, do renderizador e do mapa de atlas, e os
    /// três só existem dentro do laço de frame. Ele ARMA um pedido e sai — o
    /// mesmo caminho que o `Shift+B` já usava, e é por passarem pela MESMA porta
    /// que o botão e o atalho não podem divergir.
    BakeToSprite,
    /// **Usar o sprite selecionado como padrão** — o alpha por IMAGEM.
    ///
    /// ⚠️ **Ele ARMA e sai, pelo mesmo motivo do [`Self::BakeToSprite`] logo
    /// acima:** ler os pixels de um sprite precisa do mundo, do renderizador e
    /// do mapa de atlas, e os três só existem dentro do laço de frame. O painel
    /// não sabe o que é um atlas — e não deve saber.
    AlphaFromSprite,
    /// As quatro primitivas, na ordem em que o painel as lista.
    ///
    /// ⚠️ **Um comando por forma, e não um enum espelho do `Primitive` do
    /// shell.** Um enum duplicado aqui concordaria com o de lá exatamente até
    /// alguém acrescentar a quinta forma num só dos dois; um comando novo não
    /// compila sem que o painel também ganhe o botão dela, que é a ordem certa.
    AddSphere,
    AddCube,
    AddCylinder,
    AddTorus,
    Duplicate,
    Delete,
    ToggleIsolate,
    Merge,
    MaskClear,
    MaskInvert,
    MaskBlur,
    MaskSharpen,
    /// **Recorta a região mascarada numa peça nova** — ver
    /// [`ph2d_mesh::extract_masked`].
    ///
    /// ⚠️ Um comando, e não um `SetUi`: extrair não é ajustar um número, é uma
    /// peça que passa a EXISTIR. Os dois knobs que ele lê viajam no
    /// [`Sculpt3dUi::extract`], que é estado autorado — o comando só diz
    /// *agora*.
    Extract,
}

/// Estado retido por-instância. Vazio de propósito: a autoridade é a
/// `Sculpt3dScene` do shell, e o painel renderiza o retrato do frame.
#[derive(Clone, Debug, Default)]
pub struct Sculpt3dPanelState;

/// Host → painel, uma vez por frame antes do `paint`.
pub fn set_current_sculpt3d(snapshot: Option<Sculpt3dSnapshot>) {
    CURRENT.with(|c| *c.borrow_mut() = snapshot);
}

/// O que `paint` e `event` leem. `None` quando não há cena 3D — e aí o painel
/// **não pinta**: um painel de escultura sem escultura seria seis seções de
/// controles que não alcançam nada.
pub(crate) fn current() -> Option<Sculpt3dSnapshot> {
    CURRENT.with(|c| c.borrow().clone())
}

/// Painel → host. Enfileirado pelo `event`, drenado pela ponte do shell.
pub(crate) fn push_intent(intent: Sculpt3dIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Leva tudo o que o artista fez desde o último frame.
pub fn drain_intents() -> Vec<Sculpt3dIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Contabilidade de rolagem (o dock do shell precisa das alturas medidas).
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(Cell::get)
}

/// Ver [`last_content_h`].
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(Cell::get)
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}

/// **Qual chip da fileira de padrão está aceso**, dado o retrato.
///
/// `0` é o pincel liso, `1..=9` são os `Alpha::ALL` deslocados de um, e o último
/// é o slot de IMAGEM.
///
/// ⚠️ **Ela é `pub` para o GATE poder perguntar ao produto.** O `event` não a
/// chama — ele resolve a pergunta INVERSA (*este índice arma o quê?*) —, então
/// isto não é uma porta compartilhada, é o retrato respondendo *"qual está
/// aceso?"* em vez de um teste re-derivando a aritmética por conta própria. Um
/// gate com a sua terceira cópia concordaria consigo mesmo enquanto o painel
/// pintasse outra coisa, que é o oráculo-espelho que esta casa recusa.
#[must_use]
pub fn alpha_chip_index(snap: &Sculpt3dSnapshot) -> usize {
    match snap.ui.brush.alpha.as_ref() {
        None => 0,
        Some(a) if a.is_image() => ph2d_sculpt3d::Alpha::ALL.len() + 1,
        Some(a) => ph2d_sculpt3d::Alpha::ALL
            .iter()
            .position(|x| x == a)
            .map_or(0, |i| i + 1),
    }
}

/// **O RAIO DE FÁBRICA, em pixels de tela** — a base contra a qual as frações
/// de raio da referência são resolvidas.
///
/// ⚠️ Ele espelha o `Sculpt3dUi::default().radius_px` e o `DEFAULT_RADIUS_PX`
/// do shell **de propósito**: é o mesmo número, e ele existe aqui como *nome*
/// porque o arming precisa perguntar *"o artista ainda está no raio de fábrica
/// deste verbo?"*, e um literal `50.0` no meio dessa comparação seria a segunda
/// resposta que a próxima wave esquece de mover.
pub const BASE_RADIUS_PX: f32 = 50.0; // LITERAL-PX-OK: raio de pincel, espelha o default do shell
