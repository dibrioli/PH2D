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
    /// ⭐⭐ **AS OUTRAS PEÇAS DA CENA, para a colisão do tecido** (espec §5.6).
    ///
    /// ⚠️ **A lista é montada UMA vez, quando a simulação nasce**, e cada peça
    /// fica na pose desse instante — é por isso que ela é uma FOTOGRAFIA e não
    /// uma referência: *um colisor animado não se move durante o traço, e um que
    /// apareça a meio não entra.*
    ///
    /// ⚠️ Vazia = sem colisores, que é a omissão. Quem a enche é a shell, que é
    /// quem sabe o que mais há na cena.
    /// ⚠️ **A POSE viaja com a malha** — a lei recebe posições em espaço do
    /// MUNDO e o `Multires::mesh` de cada peça está em espaço LOCAL. Guardar as
    /// duas e levar o RAIO ao espaço local (`Pose::ray_to_local`) é mais barato
    /// e mais exacto que transformar a malha inteira.
    pub cloth_colliders: Vec<(Mesh, ph2d_mesh::Pose)>,
    base_pos: Vec<[f32; 3]>,
    base_nrm: Vec<[f32; 3]>,
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
    /// **A PEGADA DO PEN-DOWN**, para os gestos que a CONGELAM.
    ///
    /// ⚠️⚠️ **Ela existe porque a pegada normal sai das posições VIVAS, e num
    /// gesto que desloca o barro `0,38` num pincel de raio `0,35` isso deixa de
    /// ser inócuo:** os vértices que o próprio gesto levou saem do raio da
    /// consulta, o conjunto amostrado encolhe, e a normal da área — que é uma
    /// média sobre ele — **muda com o comprimento do traço**. O efeito é
    /// invisível num plano (ali toda normal é a mesma) e MEDIDO numa esfera:
    /// `8,5e-3` de desvio contra o oráculo no traço inteiro, contra `1,5e-4`
    /// truncado a 8 eventos.
    ///
    /// ⚠️ **Só o [`crate::Verb::Thumb`] a lê hoje**, e a cerca é deliberada: o
    /// [`crate::Verb::Move`] tem a mesma forma e o mesmo defeito **provável**,
    /// mas é um verbo que já shipa, com corpus próprio por correr — mudá-lo
    /// aqui seria alterar produto a partir de uma inferência. *A fixture que o
    /// decide existe* (o oráculo gravou o agarrar), e a pergunta está nomeada.
    ///
    /// ⛔⛔ **A CHAVE É O CENTRO, e ela nasceu de um gate que reprovou:** a
    /// primeira versão guardava UMA pegada por traço, e a simetria corre o
    /// mesmo traço espelhado — a segunda passagem reusava a pegada da primeira
    /// e só metade da malha se mexia. O censo
    /// `every_verb_inherits_symmetry_from_the_one_place_it_is_expanded` apanhou
    /// no minuto seguinte. ⇒ uma entrada por PASSAGEM, e o centro espelhado é
    /// a identidade natural dela: constante ao longo do gesto, distinto entre
    /// passagens, sem ninguém ter de propagar um índice até aqui.
    pegada_ancorada: Vec<([f32; 3], Vec<u32>)>,
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

impl SculptStroke {
    /// **Quantos vértices o último dab tocou** — sonda, para uma medição de
    /// custo poder dizer se duas colunas fizeram o mesmo trabalho.
    #[doc(hidden)]
    #[must_use]
    pub fn footprint_len(&self) -> usize {
        self.footprint.len()
    }
}

impl SculptStroke {
    /// Congela o `pre`: começa um traço novo sobre `mesh`.
    ///
    /// Não copia a malha — a captura é **preguiçosa, por vértice tocado**. Um
    /// traço numa malha de 5 M vértices que toca 20 mil paga 20 mil, não 5 M.
    /// **Re-sorteia o [`crate::FilterKind::Random`] sem mover nada.**
    ///
    /// ⚠️ **Nasce em `0` e o crate nunca a move**, de propósito: a semente da
    /// referência é `rand()` por invocação, o que faria de todo gate deste
    /// verbo uma medição de sorte. O determinismo é a decisão; variar é gesto
    /// do shell.
    ///
    /// ⚠️ **E ela é MENOS necessária aqui do que na referência**, pelo motivo
    /// que o `stroke_filter.rs` mede: o sorteio hasheia os BITS DA
    /// POSIÇÃO congelada, então um segundo gesto sobre uma malha já perturbada
    /// re-sorteia sozinho. Ela cobre só o caso de re-rolar a MESMA pose.
    pub fn set_filter_seed(&mut self, seed: u32) {
        self.filter_seed = seed;
    }

    /// **CONGELA A BASE PERSISTENTE** nas posições de agora (espec §6.4, o
    /// operador *Set Persistent Base*).
    ///
    /// ⚠️⚠️ **A ORDEM é a lei:** para a opção morder, a base tem de ser gravada
    /// **ANTES** do traço que a há-de contradizer. Gravá-la DEPOIS de um traço é
    /// um **no-op exacto** — ali ela É o repouso do traço seguinte, e as quatro
    /// leituras da §6.4 não mudam. *A experiência «deformar → gravar → repetir»
    /// não mede nada, e é a que ocorre primeiro a quem a desenha.*
    /// ⚠️ **Recebe as POSIÇÕES e não a `Mesh`**, e não é arrumação: quem chama
    /// tem a malha e o traço no mesmo `self`, e pedir as duas de uma vez é um
    /// empréstimo duplo. *A porta tem de caber no sítio onde o gesto acontece.*
    pub fn set_persistent_base(&mut self, positions: &[[f32; 3]]) {
        self.persistent_base.clear();
        self.persistent_base.extend_from_slice(positions);
    }

    /// **APAGA a base persistente** — o gesto oposto, e ele existe porque uma
    /// base gravada é invisível: sem ele o artista não tem como voltar ao pano
    /// que acumula sem trocar de cena.
    pub fn clear_persistent_base(&mut self) {
        self.persistent_base.clear();
    }

    pub fn begin(&mut self, mesh: &Mesh) {
        let n = mesh.vert_count();
        if self.slot.len() != n {
            self.slot = vec![u32::MAX; n];
            self.stamp = vec![0; n];
            self.epoch = 0;
        }
        self.epoch = self.epoch.wrapping_add(1);
        // O carimbo 0 é o "nunca visto" do vetor recém-criado, então a época
        // nunca pode valer 0 — a mesma regra do `QueryScratch`, e sem ela um
        // traço a cada 4 bilhões nasceria achando que já capturou tudo.
        if self.epoch == 0 {
            self.epoch = 1;
            self.stamp.fill(0);
        }
        self.touched.clear();
        self.base_pos.clear();
        self.base_nrm.clear();
        self.base_mask.clear();
        self.accum.clear();
        self.target.clear();
        // ⚠️ **Um traço novo não herda a direção do anterior** — sem esta linha o
        // primeiro dab apontaria para onde a mão ia no gesto passado, que é um
        // lugar arbitrário.
        self.last_center = None;
        // ⚠️ **Nem a pegada congelada** — ela é do GESTO, e um traço novo
        // escolhe a dele no próprio pen-down.
        self.pegada_ancorada.clear();
        // ⚠️ **Nem a inclinação**, e a referência faz o mesmo (*Clay Thumb*:
        // a inclinação volta a zero no primeiro passo do traço).
        // Sem ela o segundo traço começaria de onde o primeiro parou, e o
        // artista veria a mesma ferramenta cavar mais fundo por ter sido usada
        // antes.
        self.thumb_tilt_deg = 0.0;
        // ⚠️ **Nem a abertura do V**, pela mesma razão — e no modo dinâmico ela
        // é MEMÓRIA, então herdá-la faria o primeiro dab do traço novo raspar
        // com o ângulo que a superfície tinha noutro lugar.
        self.scrape_angle_deg = 0.0;
        self.scrape = None;
        // ⚠️ **O `b` do HC morre com o traço**, e é o que faz dele o *"array
        // zerado no início do traço"* do *Surface Smooth*: dentro de um gesto
        // ele PERSISTE entre dabs (a lei da referência), entre gestos não.
        self.hc_b.clear();
        // ⚠️ **A região de tecido morre com o traço**, e é isso que a torna
        // barata: ela é medida uma vez por gesto. Herdá-la faria o traço novo
        // simular a região do anterior, num lugar onde o artista já não está.
        self.cloth.clear();
        self.cloth_ref.clear();
        // ⚠️ E a do FILTRO pela mesma razão: ela é do gesto, não do programa.
        self.cloth_filter = None;
    }
}

/// **O TECIDO** — a região que simula, o anel pregado, e o primeiro verbo cujo
/// estado sobrevive ao evento. Ver [`stroke_cloth`].
/// ⭐ **Os NÚMEROS calibrados da lei VBD** — irmão do [`stroke_cloth`], e o corte
/// é *os números com a calibração ao lado* contra *a tradução malha ⇄ solver*.
#[path = "stroke_cloth_num.rs"]
pub mod stroke_cloth_num;

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

/// **O ALVO de cada verbo**, e o plano que quatro deles ajustam. Filho para
/// alcançar o `pre` congelado; o corte é *a LEI do traço* (aqui) contra *para
/// onde cada verbo aponta* (lá).
#[path = "stroke_target.rs"]
mod target;

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
#[path = "stroke_window_tests.rs"]
mod window_tests;

#[cfg(test)]
#[path = "stroke_accum_tests.rs"]
mod accum_tests;

#[cfg(test)]
#[path = "stroke_alpha_tests.rs"]
mod alpha_tests;
