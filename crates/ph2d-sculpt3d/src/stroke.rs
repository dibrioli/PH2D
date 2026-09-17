//! **A LEI DO TRAÇO** — a peça que separa este módulo de um port ingênuo.
//!
//! > O efeito de um traço é função do **CAMINHO**, nunca de quão fino o motor
//! > amostrou o caminho. (`docs/3D/04.1`)
//!
//! No ZBrush e no Blender cada dab soma sobre o RESULTADO do anterior, então o
//! que sobrevive a `n` dabs é um **produto sobre a lista de dabs** — e a lista
//! depende da taxa de amostragem do mouse. Passar devagar deposita mais que
//! passar rápido pelo mesmo caminho. A `line/Painter` pagou esse bug **quatro
//! vezes** em 2D (mordida do arado · cápsula do relevo · campo de smear · gate
//! de proteção) até formular a cura, e ela vale igual em 3D:
//!
//! ```text
//! pen-down:  base[v] ← positions[v]          // congela o "pre"
//! por dab:   accum[v] ← max(accum[v], w)     // ENVELOPE, não `+=`
//! por dab:   target[v] ← alvo(verbo)         // do dab que VENCEU
//! aplica:    positions[v] ← lerp(base, target, accum)
//! ```
//!
//! Três propriedades caem disso, e as três são visíveis para o artista:
//!
//! 1. **Independência de espaçamento** — devagar ou rápido dá o mesmo resultado.
//! 2. **Idempotência sob re-stamp** — repetir a mesma lista de dabs não
//!    intensifica nada, o que é o que permitiria editar parâmetros do traço
//!    *depois* dele.
//! 3. **Undo trivial** — `base` **é** o estado anterior e `touched` **é** a
//!    janela; não há um segundo sistema a construir.
//!
//! ⚠️ **O `target` guarda o VENCEDOR, não uma média.** Quando um dab novo eleva
//! o `accum` de um vértice, ele também reescreve o alvo — o mesmo desenho do
//! envelope do impasto 2D, que guarda *os ingredientes do dab mais carregado*.
//! Sem isso, um verbo cujo alvo depende do dab (todos os de plano) teria de
//! recomputar a pegada inteira a cada dab, e o gesto deixaria de ser limitado
//! pela pegada.
//!
//! ⚠️ **Um vértice NÃO capturado tem `pre == posição viva`** — porque só quem foi
//! capturado é escrito. É isso que deixa o Smooth ler a vizinhança sem capturar
//! o anel inteiro, e é por isso que `base_pos_of` cai na malha viva sem mentir.

use crate::brush::{Brush, Symmetry, Verb};
use crate::grip::{Amount, Grip};

/// **O QUE UM TOQUE É** — ver [`dab`].
#[path = "dab.rs"]
mod dab;
pub use dab::Dab;
use ph2d_mesh::{Mesh, QueryScratch, RegionScratch};

/// Um ponto ou vetor refletido pelo trio de sinais de uma cópia da simetria.
fn mirror(v: [f32; 3], s: &[f32; 3]) -> [f32; 3] {
    [v[0] * s[0], v[1] * s[1], v[2] * s[2]]
}

/// O estado vivo de UM traço de escultura.
///
/// Os dois vetores do tamanho da malha (`slot`/`stamp`) vivem aqui e são
/// **reusados entre traços** — carimbados por época, como o `QueryScratch`. O
/// resto é do tamanho da PEGADA do traço, e é isso que mantém a memória de um
/// gesto proporcional ao que o artista tocou e não ao que ele abriu.
#[derive(Clone, Debug, Default)]
pub struct SculptStroke {
    /// **A semente do [`crate::FilterKind::Random`]** — ver
    /// [`Self::set_filter_seed`].
    filter_seed: u32,
    slot: Vec<u32>,
    stamp: Vec<u32>,
    epoch: u32,
    touched: Vec<u32>,
    /// ⭐⭐⭐ **A BASE PERSISTENTE do pincel de tecido** (espec §6.4) — as posições
    /// que o artista congelou com *Set Persistent Base*. **Vazia = sem base.**
    ///
    /// ⚠️ **Ela vive AQUI e não na malha, e a espec autoriza-o:** *«em malhas sem
    /// atributos persistentes a base vive só na sessão de escultura»*. O
    /// `SculptStroke` da cena sobrevive aos traços e morre com ela, que é
    /// exactamente essa duração.
    ///
    /// ⚠️ **E ela é validada pelo COMPRIMENTO, não por uma bandeira:** um remesh
    /// muda a contagem de vértices e a base deixa de descrever a malha ⇒ a lei
    /// cai no repouso do traço sozinha, sem ninguém ter de se lembrar de a
    /// apagar.
    pub persistent_base: Vec<[f32; 3]>,
    /// ⭐⭐⭐ **AS OUTRAS PEÇAS DA CENA, fotografadas no pen-down** — uma **PORTA
    /// com DOIS consumidores**, não um campo do tecido.
    ///
    /// ⚠️ A lista é montada **UMA vez**, quando o gesto nasce, e cada peça fica
    /// na pose desse instante ⇒ *uma peça que se mova durante o traço não se
    /// move para o efeito, e uma que apareça a meio não entra.* Ela nasceu para
    /// a colisão do tecido (espec §5.6) e o segundo consumidor —
    /// [`crate::Verb::SceneProject`] (§6.1) — chegou **sem a contornar**.
    /// ⚠️ **O nome deixou de dizer «colisores» em 2026-09-14**: um segundo
    /// instantâneo da mesma coisa seria a segunda resposta a *«que mais há na
    /// cena?»*. Vazia = não há outras peças; quem a enche é a **shell**, que
    /// pergunta a [`crate::Brush::precisa_das_pecas_da_cena`]. A POSE viaja com
    /// a malha porque levar o RAIO ao espaço local dela é mais barato.
    pub pecas_da_cena: Vec<(Mesh, ph2d_mesh::Pose)>,
    /// **ONDE A PEÇA QUE SE ESCULPE ESTÁ** — a RÉGUA em que os candidatos da
    /// projecção competem (porquê: [`crate::projectar`]). Omissão: identidade.
    pub pose_activa: ph2d_mesh::Pose,
    /// ⭐⭐ **A SUPERFÍCIE DE REFERÊNCIA, fotografada no pen-down** — por vértice
    /// do nível de cima, o ponto da superfície-limite da subdivisão da base
    /// (`SPEC_unblocked_brushes.md` §2). Só o [`Verb::EraseMultires`] a lê.
    ///
    /// ⚠️ **VAZIA é a omissão, e ela quer dizer *«não há referência»*** — o
    /// pincel devolve a posição viva, ou seja **não move nada**. É por isso que
    /// um traço sem pilha de multiresolução é inerte mesmo se alguém contornar
    /// a recusa da shell: *a lei não tem para onde apontar, e não inventa.*
    ///
    /// ⚠️ **Fotografada no PEN-DOWN e não por dab**, como os colisores do pano:
    /// ela é função do nível de BAIXO, que o traço não toca. Recalculá-la por
    /// dab custaria um `subdivide` inteiro por evento de ponteiro.
    ///
    /// ⚠️ **Quem a enche é a shell**, que é quem conhece a pilha — a mesma
    /// divisão dos colisores.
    pub reference: Vec<[f32; 3]>,
    base_pos: Vec<[f32; 3]>,
    base_nrm: Vec<[f32; 3]>,
    /// ⭐⭐⭐ **AS NORMAIS DA MALHA INTEIRA NO PEN-DOWN** — vazia para todo verbo
    /// menos o [`crate::Verb::SceneProject`] em [`crate::ProjectMode::Plane`].
    ///
    /// ⛔⛔ **Ela existe porque o [`Self::base_nrm`] NÃO é o pen-down: ele é o
    /// PRIMEIRO TOQUE.** A captura é preguiçosa (um vértice entra no `base_*`
    /// quando o primeiro dab o alcança), e um vértice que só entra no 3.º dab é
    /// fotografado com a normal que ele tem **nessa altura** — já inclinada
    /// pelos vizinhos que os dois dabs anteriores afundaram. Para vinte e tal
    /// verbos isso é invisível; aqui a normal **É a direcção do raio**, e um
    /// grau de inclinação vira transporte lateral de barro.
    ///
    /// ⚠️ **MEDIDO no corpus do oráculo** (`projectar_normal_plano_area`, seis
    /// dabs): com o `base_nrm` o desvio é `1,036e-2` e é **inteiramente
    /// lateral**; com esta fotografia ele cai para a ordem do `f32`. *A
    /// diferença entre «congelado no pen-down» e «congelado no primeiro toque»
    /// é de `5 000×` a barra desta bancada.*
    ///
    /// ⚠️ **Preenchida no PRIMEIRO dab e não no [`Self::begin`]**, e a razão é a
    /// assinatura: o `begin` não recebe o pincel, logo pagaria um `O(V)` a
    /// TODOS os verbos para servir um. No primeiro dab a malha ainda é a do
    /// pen-down por construção — nada foi escrito.
    nrm0_do_pen_down: Vec<[f32; 3]>,
    base_mask: Vec<f32>,
    /// **A saída por-índice do map do dab** — ver [`super::stroke_map`]. ⚠️
    /// Campo e não local: alocar `count` entradas por dab devolveria ao
    /// alocador o que o paralelismo economiza.
    par_out: Vec<Option<DabOut>>,
    /// **A porta de ABLAÇÃO do piso do pool** — só em teste. ⚠️ Sem ela os
    /// gates ficariam verdes sobre uma rota que NUNCA roda: as fixtures deste
    /// módulo ficam sob o piso (a armadilha do ADR-0120).
    #[cfg(test)]
    pub(crate) par_floor_override: Option<usize>,
    accum: Vec<f32>,
    target: Vec<[f32; 3]>,
    footprint: Vec<u32>,
    /// **A PEGADA DO PEN-DOWN**, para os gestos que a CONGELAM — ver
    /// [`pegada::PegadaCongelada`], onde moram a razão de ela existir, o
    /// porquê de a chave ser o CENTRO e as duas ordens que ela guarda.
    ///
    /// ⭐⭐⭐ **Ela sobrevive à topologia dinâmica desde 2026-09-14** (ordem do
    /// dono: *«Thumb se for possível, deveria subdividir»*), e as duas metades
    /// disso vivem no mesmo sítio.
    pegada_ancorada: Vec<pegada::PegadaCongelada>,
    /// **A TRADUÇÃO DE UM COLAPSO**, do tamanho do COLAPSO e nunca da malha —
    /// ver [`SculptStroke::encolhe_a_pegada_congelada`]. ⚠️ Campo e não local
    /// pelo mesmo motivo do `par_out`: um vetor do tamanho da peça alocado por
    /// dab devolveria ao alocador o que o resto do traço economiza.
    pegada_traducao: Vec<(u32, u32)>,
    /// **OS ÍNDICES SOBRESCRITOS** por esse mesmo colapso — a outra metade da
    /// tradução, e ela existe separada porque responde a outra pergunta: *quem
    /// estava aqui MORREU*.
    pegada_mortos: Vec<u32>,
    moved: Vec<u32>,
    query: QueryScratch,
    /// Os buffers do passeio pela superfície — ver [`crate::dab_alcance`].
    alcance: crate::dab_alcance::Alcance,
    region: RegionScratch,
    /// A união, **sobre as cópias de espelho de UMA chamada a
    /// [`SculptStroke::dab`]**, dos vértices escritos e dos vértices cuja normal
    /// foi recomputada.
    ///
    /// ⚠️ **Eles existem porque `moved` e `region` são o rascunho de UMA cópia.**
    /// O `dab_core` zera a lista antes de a encher, e com o espelho armado ele
    /// roda de duas a oito vezes — então quem lia o rascunho depois do laço
    /// recebia a ÚLTIMA cópia e só ela. A malha ficava certa na memória e a
    /// janela de upload descrevia metade dela: o artista tocava um lado, o outro
    /// deformava na tela (report do Enio, 2026-08-05). A distinção entre
    /// *rascunho de uma cópia* e *o que a chamada fez* não era exprimível, e por
    /// isso não podia ser conferida — agora são campos diferentes com nomes
    /// diferentes, e os acessores públicos leem estes.
    call_moved: Vec<u32>,
    call_refreshed: Vec<u32>,
    /// O último dab pintou MÁSCARA? Decide de qual janela a GPU precisa — ver
    /// [`SculptStroke::last_gpu_dirty`]. Um bool escrito no mesmo `if` que já
    /// separa os dois braços; derivá-lo do `Brush` no chamador seria pedir a ele
    /// que soubesse a regra.
    last_paints_mask: bool,
    /// **O CENTRO DO DAB ANTERIOR**, para derivar [`Dab::path`].
    ///
    /// ⚠️ Guardado ANTES da expansão do espelho: o que interessa é o caminho que
    /// a mão percorreu, e as cópias de simetria são todas do mesmo gesto.
    last_center: Option<[f32; 3]>,
    /// **A INCLINAÇÃO ACUMULADA do [`Verb::ClayThumb`]**, em graus.
    ///
    /// ⚠️ **Irmão do [`Self::last_center`], e pela mesma razão:** é um fato
    /// sobre o GESTO, não sobre o dab. O `Dab` diz onde a mão apertou; quantos
    /// dabs já passaram é do traço, e o *Clay Thumb* o guarda exatamente
    /// aqui — no `StrokeCache`, não no evento.
    ///
    /// ⚠️ **Avançado UMA vez por chamada a [`Self::dab`], antes do espelho** —
    /// a referência só o avança na passada principal, não espelhada, de
    /// simetria, e a nossa fronteira de chamada É essa passada: avançá-lo por
    /// cópia faria a inclinação correr de duas a oito vezes mais rápido com o
    /// espelho armado, e o artista veria a ferramenta mudar de lei ao ligar a
    /// simetria.
    thumb_tilt_deg: f32,
    /// **A ABERTURA DO V do [`Verb::MultiplaneScrape`]**, em graus — o ângulo
    /// do *Multiplane Scrape*, guardado no traço.
    ///
    /// ⚠️ **Ele é do TRAÇO porque o modo dinâmico o SUAVIZA contra o dab
    /// anterior** ([`crate::MULTIPLANE_ANGLE_SMOOTH`]); no modo fixo ele é
    /// simplesmente reescrito, e é isso que impede um valor lido numa passada
    /// dinâmica de ressuscitar depois de o artista desmarcar o modo.
    scrape_angle_deg: f32,
    /// **A LÂMINA EM V deste dab** — hoisted, como o `alpha_frame` e a
    /// silhueta.
    ///
    /// ⚠️ **Estado de DAB num campo de TRAÇO, e a razão é a assinatura:** ela
    /// nasce de `&mut self` (o modo dinâmico tem memória) e é lida do `&self`
    /// que o alvo por-vértice recebe — as duas metades não cabem no mesmo
    /// empréstimo, e passá-la por argumento significaria enfiar um `Option` a
    /// mais em `compute_target`, que dezanove verbos não leem. É a mesma rota
    /// que o `thumb_tilt_deg` já usa.
    ///
    /// ⚠️ **`None` é *este dab não deposita*.** Ela é reescrita no topo de todo
    /// dab, então nunca sobrevive ao seguinte.
    scrape: Option<plane::ScrapePlanes>,
    /// ⭐⭐⭐ **O TRAÇO JÁ TEVE DIRECÇÃO?** — o único bit de memória que o
    /// [`crate::Verb::Plane`] precisa (`SPEC_pincel_de_plano.md` §1 e §8).
    ///
    /// O quadro local daquele pincel nasce da **direcção do traço**, e no
    /// primeiro dab ela ainda não existe ⇒ o quadro é degenerado e **nada se
    /// move** (medido: um traço de UM dab move `0` de `2 401`; de dois, `265`).
    ///
    /// ⚠️⚠️ **E um TRAÇO PARADO continua a trabalhar** (§8), que é a metade que
    /// obriga a este bit em vez de uma pergunta ao dab: se a mão para, a
    /// diferença entre dois centros é nula e o quadro voltaria a degenerar — o
    /// pincel apagava-se no meio do gesto. *A direcção, uma vez existida, não
    /// deixa de existir.*
    ///
    /// ⭐ **Um BIT, e não a direcção:** a medição derrubou a premissa óbvia — *a
    /// orientação do quadro DENTRO do plano não alcança a saída* (a reprodução
    /// usa só a normal, o centro e o raio, e bate a `≤ 8,0e-08` em 18 de 19
    /// configurações). Guardar o vector seria guardar estado que ninguém lê.
    plano_teve_direccao: bool,
    /// ⭐⭐ **O PLANO que este dab do [`crate::Verb::Plane`] ajustou** — escrito
    /// pela construção da pegada, lido pelo alvo por-vértice.
    ///
    /// ⚠️ **Mesma rota que o [`Self::scrape`] e o `thumb_tilt_deg`, e pela mesma
    /// razão de empréstimo:** ele nasce de `&mut self` (a varredura da pegada) e
    /// é lido do `&self` que o alvo recebe — as duas metades não cabem no mesmo
    /// empréstimo, e passá-lo por argumento enfiaria um `Option` a mais numa
    /// função que trinta e três verbos não leem.
    ///
    /// ⭐ **UMA fonte, DOIS leitores:** a silhueta ([`crate::Tectos`]) e o alvo
    /// saem deste mesmo valor, escrito uma vez por dab. ⛔ Derivá-lo outra vez no
    /// alvo seria a segunda resposta à pergunta *«onde está o plano?»*, e as duas
    /// divergiriam no dia em que o deslocamento mudasse de sítio.
    ///
    /// ⚠️ **`None` é *este dab não tem plano*** — sem amostra nenhuma na pegada.
    /// O alvo devolve a posição viva, ou seja não move nada.
    plano: Option<plano_da_pegada::PlanoDaPegada>,
    /// ⭐⭐⭐ **A MEMÓRIA DO PLANO, uma por passe de simetria** — ver [`crate::plano_memoria`]. Nasce
    /// no primeiro dab do traço (o inerte) e esquece-se com ele.
    plano_memorias: Vec<crate::plano_memoria::MemoriaDoPlano>,
    /// Qual passe de simetria está a correr — o índice da [`Self::plano_memorias`].
    passe_simetria: usize,
    /// ⭐⭐ **O CAMPO DE DESLOCAMENTO DO ESFREGÃO, por SLOT** — ver
    /// [`super::stroke_smear`].
    ///
    /// ⚠️ **Vazio para trinta dos trinta e um verbos**, e é o `fill_smear_d`
    /// que o enche — a mesma preguiça do `hc_b` ao lado, e pela mesma razão: um
    /// traço de `Draw` numa malha de 5 M vértices não paga 12 bytes por vértice
    /// tocado por um verbo que ele não escolheu.
    ///
    /// ⚠️ **Indexado por SLOT e não por vértice, e isso É a lei da orla**
    /// (espec §5.4): um vizinho que o traço nunca tocou não tem slot, logo lê
    /// **zero** — que é exactamente o *«campo zerado no início do traço»* da
    /// referência, e o que faz a borda da pincelada comer deslocamento.
    smear_d: Vec<[f32; 3]>,
    /// **O DUPLO BUFFER das passagens do esfregão** — ver
    /// [`super::stroke_smear`]. ⚠️ Campo e não local: alocar a pegada inteira a
    /// cada passagem de cada dab devolveria ao alocador o que o resto deste
    /// ficheiro economiza.
    smear_scratch: Vec<[f32; 3]>,
    /// **O `b` do HC, por SLOT** — ver [`super::stroke_hc`].
    ///
    /// ⚠️ **Vazio para vinte e um dos vinte e dois verbos**, e é o `fill_hc_disp`
    /// que o dimensiona: um plano por-slot a mais no `capture` cobraria 12 bytes
    /// por vértice tocado a quem nunca escolheu este pincel.
    hc_b: Vec<[f32; 3]>,
    /// **A curvatura normalizada de cada vértice**, o factor de afiação por
    /// vértice da referência. Campo e não local porque um arrasto o reconstrói a cada
    /// quadro (e a cada sub-passo dentro dele) — uma alocação por quadro numa
    /// malha grande é o custo que o `hc_b` ao lado já paga para não pagar.
    sharp_f: Vec<f32>,
    /// **AS SESSÕES DE TECIDO, uma por cópia de simetria** — ver
    /// [`stroke_cloth`].
    ///
    /// ⚠️⚠️ **É o primeiro plano deste traço que guarda ESTADO VIVO de solver**
    /// (posição *e* velocidade), e não uma função do gesto. Ele existe porque
    /// uma simulação não é `f(pre, dab)`: o resultado do evento *N* é a entrada
    /// do *N+1*. ⛔ Ele NÃO entra no snapshot de undo, e não precisa: o undo
    /// repõe o `base_pos` dos vértices capturados, e a sessão morre no pen-up.
    ///
    /// ⚠️ **Uma por CÓPIA**, porque duas regiões em lados opostos da peça não
    /// partilham vértice nenhum — juntá-las numa faria o solver resolver um
    /// sistema desconexo.
    cloth: Vec<Option<stroke_cloth::ClothSession>>,
    /// As sessões da LEI DA REFERÊNCIA (`PH2D_CLOTH_LAW=ref`), uma por cópia de
    /// simetria — ver [`stroke_cloth_ref`]. Nascem no 1.º dab, morrem no `begin`.
    cloth_ref: Vec<Option<stroke_cloth_ref::ClothRef>>,
    /// A sessão do FILTRO de tecido — ver [`stroke_cloth_filter`].
    ///
    /// ⚠️ **Uma só, e não uma por cópia de simetria:** o filtro corre sobre a
    /// peça inteira, e uma segunda passagem simularia a mesma malha duas vezes.
    /// ⚠️ Ela nasce no pen-down do filtro e morre no `begin` como as outras — é
    /// isso que a impede de sobreviver a um traço.
    cloth_filter: Option<ph2d_cloth::verlet_gesto::PincelTecido>,
    /// A sessão do pincel de POSE — ver [`stroke_pose`]. Nasce no pen-down,
    /// morre no `begin`.
    ///
    /// ⭐⭐ **Ela existir é a nossa vantagem medida sobre o alvo:** a cadeia é a
    /// mesma do princípio ao fim do traço (espec §10), e guardá-la é ler a
    /// espec, não optimizar.
    pose: Option<stroke_pose::PoseSessao>,
    /// O rascunho das posições que a pose devolve — reutilizado entre eventos
    /// para o traço não alocar a malha inteira a cada movimento do ponteiro.
    pose_saida: Vec<[f32; 3]>,
    /// O **segundo** rascunho da pose, o da auto-suavização.
    ///
    /// ⚠️ **A relaxação é de buffer DUPLO, e isso é lei e não conforto:** com um
    /// só, metade dos vértices lê o valor novo e metade o velho, e a saída passa
    /// a depender da ORDEM por que a região é percorrida. *É a mesma decisão que
    /// o esfregão de deslocamento desta crate já pagou por escrito.*
    pose_alisado: Vec<[f32; 3]>,
    /// Quantas vezes a cadeia da pose foi construída **neste** traço.
    ///
    /// ⭐⭐ **Ele existe para GATEAR a vantagem, não para diagnosticar.** A
    /// construção é `O(V)` na malha inteira e o alvo repete-a a cada movimento
    /// do rato; sem um contador, quem a movesse para dentro do laço de eventos
    /// não partiria teste nenhum — a saída seria a mesma, só mais lenta. Ver
    /// [`super::pose_simetria_tests`].
    pub(crate) pose_construcoes: u32,
    /// A sessão do pincel de CONTORNO — ver [`stroke_boundary`]. Nasce no
    /// pen-down, morre no `begin`.
    ///
    /// ⚠️ **Ela guarda VÁRIAS passagens**, uma por combinação de eixos de
    /// espelho: a espec manda cada passagem **refazer as fases A–E do zero**
    /// para a região dela, com uma âncora nova a partir do ponto reflectido.
    boundary: Option<stroke_boundary::BoundarySessao>,
    /// O rascunho das posições que o contorno escreve — reutilizado entre
    /// eventos para o traço não alocar a malha inteira por movimento.
    boundary_saida: Vec<[f32; 3]>,
    /// Quantas vezes a estrutura do contorno foi construída **neste** traço.
    ///
    /// ⭐ Mesmo papel do irmão da pose: as fases A–E são `O(malha)` e a espec
    /// manda fotografá-las no pen-down. *Sem um contador, quem as movesse para
    /// dentro do laço de eventos não partiria teste nenhum — a saída seria a
    /// mesma, só mais lenta.*
    pub(crate) boundary_construcoes: u32,
    /// ⭐⭐ **O INDICADOR da pose** — o osso que se vê ANTES de premir, com a
    /// adjacência guardada e o orçamento que o impede de arrastar o editor.
    /// Ver [`super::pose_previa`], que é onde a tabela medida está.
    ///
    /// ⚠️ **Mora aqui e não na cena** de propósito: é o que faz o indicador e o
    /// gesto atravessarem a **mesma** porta ([`Self::pose_ossos`]) e torna
    /// inexprimível que um mostre uma cadeia e o outro construa outra.
    pub(crate) pose_previa: super::pose_previa::PosePrevia,
    /// ⭐⭐ **O INDICADOR do contorno** — o troço de borda que vai dobrar e a
    /// linha da profundidade, com o censo de bordas guardado e o MESMO
    /// orçamento do osso da pose. Ver [`super::boundary_previa`].
    ///
    /// ⚠️ Mora aqui pela razão da irmã: é o que faz o indicador e o gesto
    /// atravessarem a mesma porta.
    pub(crate) boundary_previa: super::boundary_previa::ContornoPrevia,
    /// ⭐ **O filtro de tecido deste gesto colide?** — fotografado no pen-down.
    ///
    /// ⚠️ **Uma bandeira e não uma leitura das propriedades a cada passo**: a
    /// simulação nasce e morre com o gesto, e ligar a opção a meio dele mudaria
    /// a lei debaixo da mão. É a mesma razão pela qual a lista de colisores
    /// também é uma fotografia.
    cloth_filter_collisions: bool,
    /// ⭐⭐⭐ **O MATERIAL DA PEÇA — o repouso que ATRAVESSA os gestos** (report do
    /// dono, 2026-09-09: *«se eu fizer mais de uma simulação … o objeto continua
    /// esticando»*).
    ///
    /// ⛔⛔ **Sem isto o pano CRESCE sem limite, e a razão é que o tecto de
    /// esticão é relativo a um repouso que se REBASELINA:** cada
    /// [`Self::cloth_filter_begin`] lia a malha de AGORA como repouso, logo o
    /// segundo gesto media `1,10` sobre um comprimento que já era `1,10`.
    /// Medido em três gestos de gravidade sobre uma esfera: a área ia a
    /// `1,13 → 1,23 → 1,31` do repouso e a altura de `2,0` a `3,46`.
    ///
    /// ⚠️ **Ele vira a BASE PERSISTENTE da lei** ([`ph2d_cloth::verlet::Verlet`]),
    /// e não o repouso — a diferença é exactamente o que se quer: a base entra em
    /// **quatro** leituras da CONSTRUÇÃO (o comprimento de cada restrição entre
    /// as outras) e **não** nos alvos nem nos pesos. *O comprimento do material é
    /// do material; onde ele está é do gesto.*
    cloth_material: Vec<[f32; 3]>,
    /// A ASSINATURA da pose em que o último gesto de tecido deixou a peça.
    ///
    /// ⚠️⚠️ **É ela que invalida o material, e a pergunta é a certa:** se a malha
    /// de agora tem a assinatura que o último gesto deixou, ninguém lhe tocou e o
    /// material continua a valer; se não tem, alguém esculpiu — e um material que
    /// sobrevivesse a isso faria o pano lutar contra a forma nova.
    ///
    /// ⛔ **Perguntar «que ferramenta correu?» seria a segunda resposta**, e ela
    /// envelhece com cada ferramenta nova; perguntar à MALHA não envelhece.
    ///
    /// ⚠️ Uma assinatura de `64` bits sobre os bits dos `f32` — uma colisão
    /// manteria um material velho, e é `2⁻⁶⁴`. ⛔ Guardar as posições seria
    /// `1,2 MB` de clone por gesto para responder a uma pergunta de um bit.
    cloth_left: u64,
    /// ⭐⭐⭐ **QUANTO ARRASTO JÁ FOI CONSUMIDO neste gesto de filtro** — a metade
    /// do produto da lei *«o filtro mede o arrasto, não conta eventos»*.
    ///
    /// ⛔⛔ **Sem ela o efeito do filtro era proporcional a quão DEVAGAR o artista
    /// arrastava.** O shell junta os movimentos do rato **por quadro**, logo o
    /// relógio da simulação era a taxa de quadros: o mesmo arrasto de `1000 px`
    /// feito em `0,2 s` dá `12` passos e em `4 s` dá `240`. Medido sobre uma
    /// esfera, o MESMO arrasto (`s` de `0` a `1`), raio máximo da peça:
    ///
    /// | amostras | 8 | 15 | 30 | 60 | 120 | 240 |
    /// |---|---:|---:|---:|---:|---:|---:|
    /// | Gravity | `1,15` | `1,45` | `2,65` | `7,30` | `25,60` | **`98,20`** |
    ///
    /// ⚠️ **A peça nem se deforma** (volume `1,000` e esticão `1,00` nas seis
    /// colunas): ela **voa**, como uma translação rígida, `85×` mais longe. *Era
    /// esta a metade de «um elástico que estica indefinidamente» que nenhum tecto
    /// podia curar — o tecto mede deformação, e isto não é deformação.*
    ///
    /// ⇒ um passo da lei passa a valer **um [`ph2d_cloth::verlet_gesto::QUANTUM_DE_ARRASTO`]**
    /// e o adaptador corre tantos quantos o arrasto pedir. *É a mesma lei do
    /// espaçamento dos dabs, que o traço já tem: parametrizar pelo caminho, não
    /// pela amostragem.*
    cloth_filter_drag: f64,
    /// **A porta de ABLAÇÃO do orçamento do tecido** — só em teste, e ela existe
    /// para um gate poder afirmar que a lei do gesto **não depende** do número
    /// de sub-passos. Sem ela a propriedade não é observável de fora, e foi
    /// exatamente ela que eu quebrei sem ver: a 3.ª versão do drive dividia pelo
    /// orçamento, e mais sub-passos deixavam o resultado PIOR. Mesmo molde do
    /// [`Self::par_floor_override`].
    #[cfg(test)]
    pub(crate) cloth_substeps_override: Option<u32>,
    /// **O deslocamento laplaciano de cada vértice**, a direção de detalhe da
    /// referência — a MESMA grandeza que a [`crate::FilterKind::EnhanceDetails`]
    /// consome inteira. Aqui ele é medido no pré-passe e relido no gather, e é
    /// isso que o torna um buffer em vez de uma expressão.
    sharp_d: Vec<[f32; 3]>,
}

/// ⭐ **O CICLO DE VIDA do traço** — `begin`, o que ele esquece, e as portas da
/// base persistente. Ver [`ciclo`]; o corte foi forçado pelo tecto de LOC e o
/// cabeçalho dele diz porquê.
#[path = "stroke_ciclo.rs"]
mod ciclo;

/// **O TECIDO** — a região que simula, o anel pregado, e o primeiro verbo cujo
/// estado sobrevive ao evento. Ver [`stroke_cloth`].
/// ⭐ **Os NÚMEROS calibrados da lei VBD** — irmão do [`stroke_cloth`], e o corte
/// é *os números com a calibração ao lado* contra *a tradução malha ⇄ solver*.
#[path = "stroke_cloth_num.rs"]
pub mod stroke_cloth_num;

/// ⭐ **A ponte do pincel de CONTORNO** — ver [`stroke_boundary`]. Terceiro
/// irmão do [`stroke_cloth`] e do [`stroke_pose`] no papel (os três desviam
/// antes do `dab_core`) e com razão própria: a simetria dele são **passagens
/// que refazem as fases A–E**, e não uma região espelhada.
#[path = "stroke_boundary.rs"]
mod stroke_boundary;
#[path = "stroke_cloth.rs"]
mod stroke_cloth;
/// ⭐ **O FILTRO de tecido** (espec §7) — o mesmo solver na peça inteira, sem
/// pincel. Irmão do [`stroke_cloth_ref`], e o corte é o GESTO: lá um traço com
/// carimbo, aqui um arrasto que não toca a malha.
#[path = "stroke_cloth_filter.rs"]
mod stroke_cloth_filter;
/// A lei da REFERÊNCIA (Verlet + restrições de distância), clean-room, atrás
/// de `PH2D_CLOTH_LAW=ref`. Ver [`stroke_cloth_ref`].
#[path = "stroke_cloth_ref.rs"]
mod stroke_cloth_ref;
/// ⭐ **A ponte do pincel de POSE** — ver [`stroke_pose`]. Irmão do
/// [`stroke_cloth`] no papel (os dois desviam antes do `dab_core`) e não na
/// razão: o tecido porque cada cópia de simetria tem a **sua** região; a pose
/// porque a lei dela **já resolve os oito octantes numa passagem só**.
#[path = "stroke_pose.rs"]
mod stroke_pose;
pub use stroke_cloth_filter::ClothFilterStep;
pub use stroke_cloth_ref::cloth_repica;

/// **A ANATOMIA DE UM DAB** — a sequência que amarra os módulos acima. Ver
/// [`dab_core`].
#[path = "stroke_dab_core.rs"]
mod dab_core;

/// **O `pre` QUE O TRAÇO CONGELA** — ver [`freeze`]. Irmão dos três abaixo, e o
/// assunto que os três já pressupunham sem ter casa própria.
/// **QUANTAS VEZES UM DAB ACONTECE** — ver [`symmetry`].
#[path = "stroke_symmetry.rs"]
mod symmetry;

#[path = "stroke_map.rs"]
mod stroke_map;
pub(crate) use stroke_map::DabOut;

#[path = "stroke_freeze.rs"]
mod freeze;

/// **O QUE UMA SONDA PODE PERGUNTAR** — ver [`probe`]. O corte é *o que um dab
/// FAZ* (aqui) contra *o que se consegue MEDIR dele* (lá).
#[path = "stroke_probe.rs"]
mod probe;

/// **O PLANO que quatro verbos ajustam** — ver [`plane`]. O corte é o `e` que o
/// cabeçalho do [`target`] carregava: *para onde um verbo aponta* difere entre
/// os dezasseis, *que forma a superfície tem* é uma pergunta só.
#[path = "stroke_plane.rs"]
mod plane;

/// **A NORMAL QUE OS GESTOS TANGENCIAIS LEEM** — ver [`normal_do_gesto`]. Irmã
/// do [`plane`], e o corte é o SUJEITO: lá a superfície sob a pegada inteira,
/// com o peso da máscara; aqui a superfície sob o MIOLO (uma fracção do raio),
/// com a curva suave fixa e os dois lados da silhueta em baldes separados.
#[path = "stroke_normal_do_gesto.rs"]
mod normal_do_gesto;

/// ⭐⭐⭐ **O PLANO DO [`crate::Verb::Plane`]** — ver [`plano_da_pegada`]. Irmão do
/// [`plane`] e do [`normal_do_gesto`], e o corte é a LEI: aquele pesa pela
/// máscara sobre a pegada inteira (referência MIT, paridade a 1 ULP), este pesa
/// pela curva suave sobre DOIS raios próprios e puxa cada posição para o cursor.
/// ⛔ Fundi-los poria as duas leis onde uma edição futura alcança as duas.
#[path = "plano_da_pegada.rs"]
mod plano_da_pegada;

/// **A SUPERFÍCIE LOCAL do `l-mode`** — ver [`surface`]. Irmão do [`plane`], e o
/// corte são dois PAPERS: lá o plano da pegada por média de posições e normais
/// ponderada pela queda (o estimador da referência), aqui a projeção MLS de
/// Alexa et al. 2003.
#[path = "stroke_surface.rs"]
mod surface;

/// **QUE SILHUETA ESTE DAB TEM** — ver [`shape`].
#[path = "stroke_shape.rs"]
mod shape;

/// **A MALHA CRESCEU DEBAIXO DO TRAÇO** — o refino e a lei do `pre`.
///
/// Filho para alcançar os planos congelados; o corte é o mesmo do
/// `stroke_growth_tests.rs`: aqui mora *o que acontece quando a topologia muda
/// no meio de um gesto*, no pai *o que um dab faz*.
#[path = "stroke_growth.rs"]
mod growth;

/// ⭐⭐ **A PEGADA CONGELADA DO PEN-DOWN** — ver [`pegada`], irmão do [`growth`].
#[path = "stroke_pegada.rs"]
mod pegada;

/// **O ALVO de cada verbo**, e o plano que quatro deles ajustam. Filho para
/// alcançar o `pre` congelado; o corte é *a LEI do traço* (aqui) contra *para
/// onde cada verbo aponta* (lá).
#[path = "stroke_target.rs"]
mod target;

/// ⭐⭐ **O CAMPO DE DESLOCAMENTO DO ESFREGÃO** — ver [`stroke_smear`].
#[path = "stroke_smear.rs"]
mod stroke_smear;

/// **O QUE O ANEL DIZ** — a média congelada e a normal que o relax remove.
/// Filho pelo mesmo motivo do [`target`]: os dois leem o `pre`.
#[path = "stroke_ring.rs"]
mod ring;

/// **O `b` DO HC** — o buffer que o [`Verb::SurfaceSmooth`] exige. Filho pelo
/// mesmo motivo dos dois acima, e o corte é *o que a vizinhança DIZ* (ali)
/// contra *o que o passo laplaciano TIROU* (lá) — a segunda pergunta só existe
/// para um verbo, e o buffer que ela obriga é a única parte estrutural dele.
#[path = "stroke_hc.rs"]
mod hc;
pub use hc::{HC_SHAPE_DEFAULT, HC_VERTEX_DEFAULT, HC_VERTEX_MIN};

/// **O QUE O TRAÇO ESCREVE** — os dois aplicadores. Filho pela mesma razão do
/// [`target`]: eles leem os planos congelados. O corte é *a LEI* (aqui) contra
/// *a ESCRITA* (lá).
#[path = "stroke_apply.rs"]
mod apply;

/// **O FILTRO** — o verbo aplicado à malha inteira, sem dab. Filho pela mesma
/// razão do [`apply`], e o corte é o GESTO: lá um carimbo enche o `accum` dab a
/// dab, aqui um arrasto o enche de uma vez. Ver [`mesh_filter`].
#[path = "stroke_filter.rs"]
mod mesh_filter;
pub use mesh_filter::FILTER_DRAG_PER_PX;

/// **O SHARPEN** — o único filtro com PRÉ-PASSE, e por isso o único que não
/// cabe no `match` por-vértice do [`mesh_filter`]. Filho pela mesma razão dos
/// outros: ele lê os planos congelados. Ver [`sharpen`].
#[path = "stroke_filter_sharpen.rs"]
mod sharpen;
pub use sharpen::sharpen_total_for_measurement;

/// **AS JANELAS QUE UM TRAÇO PUBLICA** — ver [`windows`].
#[path = "stroke_windows.rs"]
mod windows;

#[cfg(test)]
#[path = "stroke_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "stroke_cloth_tests.rs"]
mod cloth_tests;
/// Os gates da FRONTEIRA da pose — o report das reentrâncias, e de onde vem o
/// tecto das suavizações. O corte é o SUJEITO: aqui a BORDA da região.
#[cfg(test)]
#[path = "pose_fronteira_tests.rs"]
mod pose_fronteira_tests;
/// Os gates do INDICADOR da pose — irmão do [`pose_simetria_tests`], e o corte
/// é o SUJEITO: lá o que o GESTO faz à malha, aqui o que se VÊ antes dele.
#[cfg(test)]
#[path = "pose_previa_tests.rs"]
mod pose_previa_tests;
/// Os gates da FIAÇÃO da pose — ver [`super::pose_simetria_tests`].
#[cfg(test)]
#[path = "pose_simetria_tests.rs"]
mod pose_simetria_tests;

#[cfg(test)]
#[path = "stroke_cloth_filter_tests.rs"]
mod cloth_filter_tests;

/// ⭐ **Os SELECTORES do tecido chegam ao motor** — irmão do [`cloth_tests`], e
/// o corte é *o que a LEI faz* contra *o que o PAINEL alcança*.
#[cfg(test)]
#[path = "stroke_cloth_mode_tests.rs"]
mod cloth_mode_tests;

/// ⭐ **Os ARTEFATOS, o orçamento e a densidade** — o outro irmão, e o corte é
/// *o que o gesto faz na região* contra *o que um traço LONGO deixa*.
#[cfg(test)]
#[path = "stroke_cloth_artefatos_tests.rs"]
mod cloth_artefatos_tests;

#[cfg(test)]
#[path = "stroke_growth_tests.rs"]
mod growth_tests;

#[cfg(test)]
#[path = "stroke_pegada_tests.rs"]
mod pegada_tests;

#[cfg(test)]
#[path = "stroke_window_tests.rs"]
mod window_tests;

#[cfg(test)]
#[path = "stroke_accum_tests.rs"]
mod accum_tests;

#[cfg(test)]
#[path = "stroke_alpha_tests.rs"]
mod alpha_tests;
