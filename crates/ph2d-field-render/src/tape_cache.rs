//! ⭐⭐⭐ **A CACHE DE FITAS ENTRE QUADROS** (W82) — a cura da parede que a W81 mediu.
//!
//! # O número que abriu esta wave
//!
//! Um quadro de movimento a `640×360` custa `~24 ms` e compila `242` fitas de JIT. Medido
//! (`docs/3DModeling/06` §82.9, num traçado que **só compila** e não marcha uma amostra):
//!
//! | threads | 1 | 2 | 4 | 8 | **16** | **32** |
//! |---|---:|---:|---:|---:|---:|---:|
//! | ms | `130,5` | `66,3` | `34,2` | `19,2` | **`13,90`** | **`13,78`** |
//!
//! ⭐⭐⭐ **De 16 para 32 threads a compilação ganha `1 %`: ela satura.** Um JIT mapeia memória
//! **executável**, e `mmap`/`mprotect` são recursos do **kernel** — a montagem em parte
//! **serializa-se**, e núcleos a mais não a atravessam. ⇒ metade do relógio de um quadro é trabalho
//! que nem escala nem muda, refeito inteiro a cada quadro enquanto a mão mexe.
//!
//! # ⭐⭐⭐ O mecanismo: a cache não tem chave, tem um teste de CONTENÇÃO
//!
//! A cerca que a W56 escreveu é *«a árvore especializada só vale DENTRO de `[lo, hi]`»* — e ela lê-se
//! ao contrário: **uma fita construída para `R` serve toda a sub-região de `R`**. ⇒ construindo a
//! fita para `R` **inflada**, ela serve o quadro seguinte sempre que a região nova ainda lá caiba.
//!
//! Medido (§82.12, arrasto = uma órbita de `g` graus por quadro):
//!
//! | arrasto | `f = 1,00` | **`f = 1,25`** | `f = 1,50` | `f = 2,00` |
//! |---|---:|---:|---:|---:|
//! | `1°` | `9,0 %` | **`92,8 %`** | `95,5 %` | `96,7 %` |
//! | `2°` | `9,0 %` | **`84,3 %`** | `92,8 %` | `94,9 %` |
//! | `4°` | `7,5 %` | `48,9 %` | **`82,9 %`** | `91,0 %` |
//! | preço por amostra | `1,00×` | **`1,18×`** | `1,37×` | `1,67×` |
//!
//! ⭐⭐ **A `f = 1` a cache acerta `9 %`** — guardar a região *exacta* não serve de nada, e a
//! **inflação é o mecanismo**, não uma afinação.
//!
//! # ⭐⭐⭐ W148 — A FITA PASSA A SER GUARDADA PARA O CASCO, e a folga passa a ser uma DISTÂNCIA
//!
//! A W82 guardava cada fita para a **caixa** inflada, e o preço estava declarado (*«a cache troca
//! aresta por compilação»*) com o número de quando o ladrilho era `64`: `1,18×`–`1,31×`. ⛔⛔ **A `24`
//! px ele era outro** — medido em 13/09 (`tests/hull_cache_probe.rs`, contagens, vale sob carga): a
//! fita servida guardava **`1,9×`–`2,4×`** as arestas do caminho sem cache. O tubo de um ladrilho fino
//! e oblíquo tem uma caixa quase toda vazia, e o custo de uma amostra da marcha segue as arestas.
//!
//! ⭐ **A cura tem DUAS metades, e nenhuma anda sozinha:**
//!
//! 1. **a fita é compilada para o CASCO e servida por contenção de casco** — a pergunta que a
//!    compilação faz ([`ph2d_field_eval::RegionHulls`]), e não uma forma que se testa mais depressa;
//! 2. **a folga é uma DISTÂNCIA, e não uma escala** ([`PAD_OF_REACH`]). ⛔ O casco escalado por `f`
//!    perde os acertos (`f = 1,25`: `48`–`70 %` e as compilações TRIPLICAM) porque a folga dele é
//!    proporcional à **largura** do tubo — e o que um arrasto move é `braço × ângulo`, que não sabe a
//!    largura de tubo nenhum.
//!
//! ⛔ **E a ORDEM de consulta não era o suspeito:** servir a fita mais velha, a mais nova ou a de
//! menor volume muda as arestas servidas em menos de `5 %`.
//!
//! ⚠️ **A caixa W82 continua viva**, e não por nostalgia: é o caminho de bissecção
//! (`PH2D_FIELD_TAPE_BOX=1`, ver [`Growth`]) e o lado A de todo relógio que decida este módulo — as
//! duas políticas têm de poder correr no **mesmo processo**.

use ph2d_field::FieldDoc;
use ph2d_field_eval::RegionHulls;
use ph2d_field_eval::hybrid::RegionTape;

/// ⭐⭐⭐ **Quanto uma região é inflada antes de a sua fita ser compilada** — na política da CAIXA
/// ([`Growth::Box`]), que é a W82 e hoje o caminho de bissecção.
///
/// ⚠️ **Ele não é um número de conforto: é o mecanismo.** A `1,00` a cache acerta `9 %` e não serve
/// para nada; a `1,25` acerta `84 %`–`93 %` às velocidades de arrasto reais (um quadro de `24 ms` a
/// `90°/s` é `2,2°`) por `1,18×` no custo de uma amostra. Subir mais compra acerto num arrasto
/// **rápido**, que é exactamente onde o artista tolera menos detalhe.
pub const INFLATE: f32 = 1.25;

/// ⭐⭐⭐ **A folga de uma fita, em fracção do ALCANCE a partir do alvo** — na política do CASCO
/// ([`Growth::Hull`]), a que shipa (W148).
///
/// # Porque uma DISTÂNCIA, e porque do ALCANCE
///
/// O que tira uma região de dentro da fita do quadro anterior é o **movimento** da câmera: uma órbita
/// de `θ` move um ponto `braço × θ`, e o braço é a distância ao alvo ([`reach`]). Um pan move-o por
/// uma distância de mundo; um zoom escala-o em torno do alvo. ⇒ a folga mede-se na grandeza do
/// movimento, e o alcance é o que a normaliza.
///
/// ⭐⭐ **Medido que é o alcance, e não o ladrilho**, a `426×240` numa varredura de zoom
/// (`half_extent` `0,4`/`0,8`/`1,6`) × tamanho da peça (`½`/`1`/`2×`): em fracção do alcance o acerto
/// e as arestas ficam estáveis entre as nove células; em ladrilhos, o mesmo número dá `81 %` numa e
/// `92 %` noutra.
///
/// # O número, contra a caixa que shipava (contagens; `÷ sem cache` = arestas da fita servida)
///
/// `426×240`, círculo de 168 arestas, com as leis de câmera do módulo:
///
/// | gesto | caixa `f = 1,25`: compila · ÷ | casco `0,06`: compila · ÷ | **casco `0,08`**: compila · ÷ |
/// |---|---|---|---|
/// | órbita 4 px | `61` · `1,93×` | `42` · `1,65×` | **`20` · `1,84×`** |
/// | órbita 12 px | `108` · `2,04×` | `111` · `1,69×` | **`67` · `1,88×`** |
/// | pan 4 px | `99` · `1,91×` | `45` · `1,67×` | **`20` · `1,83×`** |
/// | pan 12 px | `61` · `1,94×` | `35` · `1,69×` | **`17` · `1,87×`** |
/// | zoom `+1` | `57` · `2,32×` | `27` · `2,01×` | **`9` · `2,14×`** |
/// | zoom `−1` | `78` · `1,82×` | `80` · `1,50×` | **`52` · `1,64×`** |
///
/// ⭐ **`0,08` é melhor que a caixa nas DUAS colunas em todo gesto medido**, e é por isso que é o
/// número que nasce; `0,06` corta mais arestas e compila `+3 %` em duas células. ⏳ **Entre os dois
/// decide o RELÓGIO**, a `load < 5`, com as duas políticas no mesmo processo — a contagem diz que
/// nenhum deles perde para a caixa, não qual dos dois ganha mais.
pub const PAD_OF_REACH: f32 = 0.08;

/// ⭐⭐⭐ **Quantos QUADROS de regiões a cache guarda** — e a capacidade é **derivada** disto.
///
/// ⛔⛔ **Ela era um número fixo (`CAPACITY = 2048`) e esse número passou a DITAR O PRODUTO** — o
/// defeito que o `CLAUDE.md §0.0` nomeia: *nunca deixe o fallback definir o produto*.
///
/// Medido (`measure_the_tile_size_now_that_the_cache_exists`, com o tecto fixo de `2048`):
///
/// | | ladrilho 24 | 32 | 48 | **64** |
/// |---|---:|---:|---:|---:|
/// | `640×360` | `43,3` | `43,0` | `50,0` | `58,8` |
/// | **`1600×900`** | **`859,2`** | **`677,8`** | `60,8` | `65,1` |
///
/// ⭐ A `1600×900` um ladrilho de `32` pede `50×29 × 4 ≈ 5 800` regiões por quadro contra um tecto
/// de `2 048`: **a cache despejava metade a cada quadro e recompilava tudo**, e o «óptimo» que a
/// varredura devolvia era só o maior ladrilho que ainda cabia no meu tecto. *Um limite que não diz
/// de que recurso é acaba a escolher a constante do lado.*
///
/// ⭐⭐ **O tecto passa a ser o que o quadro PEDE**, e `3` quadros é o que a lei da cache precisa: o
/// quadro corrente, o anterior (de onde vêm os acertos) e o do outro documento (o preview alterna
/// **dois** — ver [`DOCS`]).
///
/// ⚠️ **Medido com a política da CAIXA** (W89). ⏳ A do casco serve **mais** regiões com menos fitas
/// (compila `3×`–`6×` menos), e isto pede ser reconferido contra ela.
const FRAMES_KEPT: usize = 3;

/// ⚠️ **O tecto absoluto, e ele diz de que recurso é: MEMÓRIA EXECUTÁVEL.** Cada fita é um `mmap`
/// do código dela — uma região pequena compila poucos milhares de instruções, e o `mmap` mínimo é
/// uma página. `16 384` fitas são da ordem de dezenas de MiB, e `65 530` é o tecto de mapeamentos
/// que o Linux dá a um processo por omissão.
const CAPACITY_MAX: usize = 16_384;

/// ⭐⭐⭐ **Quantos DOCUMENTOS a cache guarda ao mesmo tempo** — e o `2` não é folga, é a contagem
/// dos degraus do preview.
///
/// ⚠️ **O app alterna dois documentos por construção**: `field3d_preview::coarse_doc` dá o contorno
/// **grosso** enquanto a mão mexe, e o **cheio** corre ao parar. Com um documento só, **cada**
/// transição apagava a cache inteira — medido: `~68` compilações e **zero** acertos no 1.º quadro
/// depois de cada uma, dois quadros frios em cada seis. *Uma bancada que mede um arrasto contínuo
/// não pode ver isto*, e a minha media exactamente isso.
///
/// ⚠️ **Um documento a mais NÃO é mais seguro** — uma fita só é servida ao documento que a
/// construiu (a etiqueta `Entry::gen`), e o que um terceiro slot compraria era memória para um
/// documento que ninguém volta a pedir.
const DOCS: usize = 2;

/// ⭐⭐⭐ **QUANTO A CAIXA É DESLOCADA DENTRO DA PRÓPRIA FOLGA** — a cura da travadinha (W89).
///
/// # O mecanismo, medido
///
/// A caixa é inflada **em torno do próprio centro**, então toda região compilada no mesmo quadro
/// tem a mesma folga em todas as direcções — e sai da caixa no **mesmo quadro seguinte**. As
/// falhas chegam em **lote**, e o lote auto-sustenta-se (quem recompila junto volta a expirar
/// junto). Medido num arrasto de `2°/quadro` a `426×240`:
///
/// | quadro | 2 | 3 | 4 | 5 | 6 |
/// |---|---:|---:|---:|---:|---:|
/// | compila | `98` | `323` | `148` | `250` | `122` |
/// | ms | `12,2` | `26,9` | `19,8` | `25,7` | `16,5` |
///
/// ⇒ *o artista não sente a média, sente o `27` no meio dos `12`.*
///
/// # ⭐ A cura desloca a FASE, nunca o tamanho
///
/// Um deslocamento do **centro** dentro da folga que a inflação já pagou mantém o volume da caixa
/// — logo o preço por amostra — e muda **quanto** a região ainda pode derivar antes de sair. Cada
/// região recebe o seu, derivado do índice dela (estável entre quadros), e as coortes dispersam-se
/// e **ficam** dispersas.
///
/// ⚠️ **A contenção é o limite duro**: com `|u| ≤ 1` a caixa deslocada ainda contém a região (a
/// folga por lado é `half·(f−1)`), e é isso que o gate `the_phased_box_still_contains_its_region`
/// afirma.
///
/// # A amplitude é MEDIDA (`measure_what_dispersing_the_cohorts_buys`, regime = quadros 40+ de um
/// arrasto de 90)
///
/// | fase | mediana | média | máximo | compilações |
/// |---|---:|---:|---:|---:|
/// | `0,0` | `16,3` · `12,8` | `16,6` · `13,8` | `31,0` · `24,6` | `5 177` · `5 189` |
/// | **`0,3`** | **`12,8` · `12,5`** | **`13,7` · `12,8`** | **`21,3` · `20,0`** | `5 181` · `5 204` |
/// | `0,5` | `13,2` · `11,9` | `14,4` · `13,0` | `23,0` · `21,9` | `5 266` · `5 305` |
/// | `0,8` | `14,9` · `12,9` | `15,8` · `13,4` | `28,0` · `24,1` | `5 743` · `5 744` |
///
/// (duas corridas, `load` 8–10 · a coluna que decide é o **máximo**, que é o que o artista sente)
///
/// ⭐ **A `0,3` compila o MESMO que a `0,0`** (`+0,3 %`) — a dispersão é de graça. ⛔ A `0,8` compila
/// `+11 %`: uma região com muito pouca folga do lado da deriva expira quase todo o quadro, e a
/// convexidade de `1/vida` cobra-o. *Há uma amplitude ÓPTIMA e ela não é a maior.*
///
/// ⚠️ **O ganho é de `4` a `10 ms` no máximo, não uma ordem de grandeza** — o defeito grande desta
/// wave era outro (ver [`FRAMES_KEPT`] e a nota do módulo).
///
/// ⚠️ **Na política do casco a mesma fase desloca o centro por `δ·u`** ([`pad_phased`]) — a lei é a
/// mesma (dispersar coortes sem mudar o volume), e ⏳ a amplitude foi medida com a caixa.
pub const PHASE: f32 = 0.3;

/// ⭐⭐⭐ **Como uma fita cresce antes de ser compilada, e que pergunta a serve** — ver o doc do
/// módulo (W148).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Growth {
    /// **A W82**: caixa escalada por `inflate`, servida por contenção de caixa. Caminho de bissecção
    /// (`PH2D_FIELD_TAPE_BOX=1`) e lado A dos relógios.
    Box { inflate: f32 },
    /// ⭐ **O que shipa**: a caixa cresce `pad_of_reach × alcance` de cada lado, a fita é compilada
    /// para os cascos dessa caixa, e só é servida a uma região cujos cascos ela contém.
    Hull { pad_of_reach: f32 },
}

/// ⚠️ **A política vem do ambiente?** `PH2D_FIELD_TAPE_BOX=1` volta à caixa da W82.
///
/// *Um interruptor de bissecção é a diferença entre «piorou» e «piorou por causa disto».*
fn growth_from_env() -> Growth {
    static BOX: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *BOX.get_or_init(|| std::env::var("PH2D_FIELD_TAPE_BOX").as_deref() == Ok("1")) {
        Growth::Box { inflate: INFLATE }
    } else {
        Growth::Hull {
            pad_of_reach: PAD_OF_REACH,
        }
    }
}

/// ⭐⭐ **Quantas fitas vieram da cache** — o par do `FLOAT_TAPES`, que conta as que foram
/// compiladas.
///
/// ⚠️ *Contar o trabalho feito não é contar o trabalho poupado.* Sem este contador, uma cache que
/// nunca acerta é indistinguível de uma que acerta sempre — a imagem é a mesma nas duas.
#[doc(hidden)]
pub static TAPE_HITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// ⭐⭐⭐ **Quantas VARREDURAS de despejo correram, e quantas fitas caíram** — o par que faltava.
///
/// ⚠️ **Sem eles, um despejo é invisível a toda régua deste módulo:** ele não muda a imagem, não
/// muda o acerto **médio** de um arrasto contínuo, e o relógio que ele move é o de **um** quadro
/// entre muitos. *Uma cache que despeja no sítio errado mede-se igual a uma que não despeja* — e o
/// despejo corre debaixo do cadeado de ESCRITA, com 32 threads à porta.
#[doc(hidden)]
pub static TAPE_EVICTIONS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Ver [`TAPE_EVICTIONS`] — quantas fitas as varreduras deitaram fora.
#[doc(hidden)]
pub static TAPE_DROPPED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// ⭐⭐ **Quanto tempo se passou DENTRO do [`TapeCache::get`]** — a varredura linear.
///
/// ⚠️ Ela percorre a população inteira por **cada região** de um quadro (~600 a `426×240`), e a
/// população é o tecto derivado (`~2 600` fitas). *O que uma cache guarda a mais não é de graça:
/// alguém a percorre* — e até aqui ninguém tinha medido quanto.
///
/// ⚠️ Desde a W148 ele inclui o teste de contenção dos CASCOS das candidatas que a caixa não rejeitou;
/// o cálculo dos cascos da CONSULTA corre fora (quem chama os passa já feitos).
#[doc(hidden)]
pub static GET_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Ver [`TAPE_EVICTIONS`] — quanto tempo as varreduras passaram **dentro do cadeado de escrita**.
#[doc(hidden)]
pub static EVICT_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Uma caixa de mundo.
type Aabb = ([f32; 3], [f32; 3]);

struct Entry {
    lo: [f32; 3],
    hi: [f32; 3],
    /// ⭐ **Os cascos para que a fita foi compilada** (W148) — `None` na política da caixa, e aí a
    /// contenção de caixa basta.
    hulls: Option<RegionHulls>,
    tape: RegionTape,
    /// ⚠️ **A que DOCUMENTO esta fita pertence** — ver [`DOCS`]. Uma fita só é servida ao documento
    /// que a construiu; a etiqueta é o que permite guardar mais de um sem os misturar.
    doc_id: u32,
    /// O quadro em que ela foi pedida pela última vez — a régua do despejo.
    ///
    /// ⚠️⚠️ **Atómico de propósito, e isto foi MEDIDO.** A 1.ª versão tomava o cadeado de
    /// **escrita** para carimbar a idade a cada acerto — `~200` acertos por quadro, vindos de 32
    /// threads. O acerto subiu para `92 %`, as compilações caíram de `231` para `19`… e o quadro
    /// não mexeu (`1,00×` num caso e **`0,74×`** noutro). *Uma cache que serializa os leitores dela
    /// devolve na trava o que poupou no JIT.* Com o contador atómico o carimbo cabe debaixo do
    /// cadeado de **leitura**, que 32 threads tomam ao mesmo tempo.
    seen: std::sync::atomic::AtomicU64,
}

struct Inner {
    /// Quantos quadros de regiões esta cache guarda — ver [`FRAMES_KEPT`]. É um campo, e não a
    /// constante lida directamente, porque a **varredura** que a escolheu tem de correr as
    /// respostas no mesmo processo.
    frames_kept: usize,
    /// Quanto esta cache desloca a fase de cada caixa — ver [`PHASE`].
    phase: f32,
    /// Quantas fitas esta cache guarda — **derivado** do que um quadro pede, ver [`FRAMES_KEPT`].
    capacity: usize,
    /// ⚠️ **Os documentos que a cache conhece**, cada um com a etiqueta dele. Uma fita só é servida
    /// ao documento que a construiu: a fita da peça de ontem responde um número plausível e errado,
    /// que é o pior modo de falha que há — a imagem sai *quase* certa.
    docs: Vec<(FieldDoc, u32)>,
    /// Qual deles é o do quadro corrente.
    current: u32,
    next_doc_id: u32,
    frame: u64,
    entries: Vec<Entry>,
}

/// ⭐⭐⭐ A cache. Ver o doc do módulo.
///
/// ⚠️ **Ela vive ENTRE quadros**, então não pode pertencer ao `RegionCompiler`, que nasce e morre
/// com um. O dono dela é quem desenha.
pub struct TapeCache {
    inner: std::sync::RwLock<Inner>,
    /// ⚠️ **Fora do cadeado de propósito**: ela é fixa desde a construção, e quem chama pergunta-a
    /// ([`Self::uses_hulls`]) antes de calcular os cascos da consulta — o que tem de acontecer
    /// **fora** do cadeado, onde 32 threads não esperam por ele.
    growth: Growth,
}

impl Default for TapeCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TapeCache {
    /// A cache do produto — a política vem do ambiente ([`growth_from_env`]).
    #[must_use]
    pub fn new() -> Self {
        Self::with_growth(growth_from_env())
    }

    /// ⚠️ Só para a sonda e para o relógio: a cache com a política escolhida — ver [`Growth`].
    #[doc(hidden)]
    #[must_use]
    pub fn with_growth(growth: Growth) -> Self {
        Self {
            inner: std::sync::RwLock::new(Inner {
                frames_kept: FRAMES_KEPT,
                phase: PHASE,
                capacity: CAPACITY_MAX,
                docs: Vec::new(),
                current: 0,
                next_doc_id: 1,
                frame: 0,
                entries: Vec::new(),
            }),
            growth,
        }
    }

    /// ⚠️ Só para a sonda: a cache da W82 (a CAIXA) com outra inflação — ver [`INFLATE`].
    #[doc(hidden)]
    #[must_use]
    pub fn with_inflate(f: f32) -> Self {
        Self::with_growth(Growth::Box { inflate: f })
    }

    /// ⚠️ Só para a sonda: a cache do CASCO com outra folga — ver [`PAD_OF_REACH`].
    #[doc(hidden)]
    #[must_use]
    pub fn with_pad_of_reach(r: f32) -> Self {
        Self::with_growth(Growth::Hull { pad_of_reach: r })
    }

    /// ⚠️ Só para a sonda: a mesma cache com outro tecto — ver [`FRAMES_KEPT`].
    #[doc(hidden)]
    #[must_use]
    pub fn with_frames_kept(n: usize) -> Self {
        let c = Self::new();
        c.inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .frames_kept = n;
        c
    }

    /// ⚠️ Só para a sonda: a mesma cache com outra dispersão de fase — ver [`PHASE`].
    #[doc(hidden)]
    #[must_use]
    pub fn with_phase(p: f32) -> Self {
        let c = Self::new();
        c.inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .phase = p;
        c
    }

    /// ⚠️ Só para a sonda.
    #[doc(hidden)]
    #[must_use]
    pub fn phase_of(&self) -> f32 {
        self.inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .phase
    }

    /// A política desta cache — ver [`Growth`].
    #[must_use]
    pub const fn growth_of(&self) -> Growth {
        self.growth
    }

    /// ⭐ **Esta cache pergunta pelos CASCOS?** — quem chama só os calcula se sim.
    #[must_use]
    pub const fn uses_hulls(&self) -> bool {
        matches!(self.growth, Growth::Hull { .. })
    }

    /// ⭐ **Abre um quadro** — e deita tudo fora se o documento mudou.
    ///
    /// ⚠️ **A comparação é por VALOR, e é barata ao lado do que ela guarda:** um `FieldDoc` de uma
    /// peça de perfil são alguns kiB de `f32` contra os `~14 ms` de compilação que a cache existe
    /// para não repetir. *Uma cache que não sabe quando morrer é uma fonte de imagens erradas.*
    pub fn begin(&self, doc: &FieldDoc, regions_this_frame: usize) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // ⭐ **O tecto é o que o quadro pede** — ver [`FRAMES_KEPT`]. Ele sobe com a resolução e com
        // ladrilhos mais finos, que é exactamente quando o tecto fixo estrangulava.
        inner.capacity = regions_this_frame
            .saturating_mul(inner.frames_kept)
            .clamp(64, CAPACITY_MAX);
        if let Some((_, g)) = inner.docs.iter().find(|(d, _)| d == doc) {
            inner.current = *g;
        } else {
            let g = inner.next_doc_id;
            inner.next_doc_id = inner.next_doc_id.wrapping_add(1);
            inner.docs.push((doc.clone(), g));
            inner.current = g;
            if inner.docs.len() > DOCS {
                let velho = inner.docs.remove(0).1;
                inner.entries.retain(|e| e.doc_id != velho);
            }
        }
        inner.frame = inner.frame.wrapping_add(1);
    }

    /// ⭐⭐⭐ **A região que uma fita nova guarda** — a caixa crescida pela política desta cache.
    ///
    /// `reach` é o [`reach`] do quadro, e `seed` a identidade estável da região (ver [`PHASE`]).
    #[must_use]
    pub fn grow(&self, lo: [f32; 3], hi: [f32; 3], reach: f32, seed: u64) -> Aabb {
        let amp = self.phase_of();
        match self.growth {
            Growth::Box { inflate } => inflate_phased(lo, hi, inflate, seed, amp),
            Growth::Hull { pad_of_reach } => pad_phased(lo, hi, pad_of_reach * reach, seed, amp),
        }
    }

    /// A fita que **contém** `[lo, hi]` — e, se ela foi compilada para cascos, cujos cascos contêm
    /// `query`.
    ///
    /// ⚠️ **`query` são os cascos da região PEDIDA**, calculados por quem chama e fora do cadeado.
    /// Uma fita com cascos nunca é servida a uma consulta sem eles (`None`): o lado seguro.
    ///
    /// ⚠️ **Tudo debaixo do cadeado de LEITURA** — ver o campo `seen`.
    #[must_use]
    pub fn get(
        &self,
        lo: [f32; 3],
        hi: [f32; 3],
        query: Option<&RegionHulls>,
    ) -> Option<RegionTape> {
        let t0 = std::time::Instant::now();
        let out = self.get_inner(lo, hi, query);
        GET_NS.fetch_add(
            u64::try_from(t0.elapsed().as_nanos()).unwrap_or(u64::MAX),
            std::sync::atomic::Ordering::Relaxed,
        );
        out
    }

    fn get_inner(
        &self,
        lo: [f32; 3],
        hi: [f32; 3],
        query: Option<&RegionHulls>,
    ) -> Option<RegionTape> {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let cur = inner.current;
        let e = inner.entries.iter().find(|e| {
            e.doc_id == cur
                // ⭐ O 1.º nível: seis desigualdades, que rejeitam quase tudo.
                && (0..3).all(|k| lo[k] >= e.lo[k] && hi[k] <= e.hi[k])
                // ⭐ O 2.º: só as sobreviventes pagam o casco — ver o doc do módulo.
                && e
                    .hulls
                    .as_ref()
                    .is_none_or(|h| query.is_some_and(|q| h.contains(q)))
        })?;
        e.seen
            .store(inner.frame, std::sync::atomic::Ordering::Relaxed);
        TAPE_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Some(e.tape.clone())
    }

    /// Guarda uma fita construída para a caixa `[lo, hi]` — com os cascos para que foi compilada,
    /// quando a política os usa.
    pub fn insert(&self, lo: [f32; 3], hi: [f32; 3], hulls: Option<RegionHulls>, tape: RegionTape) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let seen = inner.frame;
        let mut mortas: Vec<Entry> = Vec::new();
        if inner.entries.len() >= inner.capacity {
            // ⭐ **Despejo por IDADE DE USO, METADE de cada vez** — e a metade é MEDIDA (W89).
            //
            // ⛔⛔ **Uma FATIA de `1/8` foi implementada e é PIOR nos três números** (mediana
            // `11,5 → 13,9 ms`, média `12,8 → 16,3`, máximo `21,2 → 61,7`): guardar `7/8` mantém a
            // população colada ao tecto (`3 158` contra `2 234` fitas), e o [`TapeCache::get`] é uma
            // **varredura linear** que paga esse tamanho em cada uma das ~600 regiões do quadro.
            // *O que uma cache guarda a mais não é de graça: alguém a percorre.*
            //
            // ⚠️ A **contagem** é por índice e não por corte de valor: um corte por idade deita fora
            // *«todos os mais velhos que X»*, e num quadro em que metade das fitas foi tocada no
            // mesmo tique isso deita fora **nada** (a cache cresce para sempre) ou **tudo**.
            let t0 = std::time::Instant::now();
            inner
                .entries
                .sort_by_key(|e| e.seen.load(std::sync::atomic::Ordering::Relaxed));
            let k = (inner.entries.len() / 2).max(1);
            mortas = inner.entries.drain(..k).collect();
            EVICT_NS.fetch_add(
                u64::try_from(t0.elapsed().as_nanos()).unwrap_or(u64::MAX),
                std::sync::atomic::Ordering::Relaxed,
            );
            TAPE_EVICTIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            TAPE_DROPPED.fetch_add(k, std::sync::atomic::Ordering::Relaxed);
        }
        let doc_id = inner.current;
        inner.entries.push(Entry {
            lo,
            hi,
            hulls,
            tape,
            doc_id,
            seen: std::sync::atomic::AtomicU64::new(seen),
        });
        // ⚠️ **A libertação sai de DEBAIXO do cadeado** — quem espera por esta cache são as outras
        // 31 threads do quadro, e nenhuma delas precisa que a memória já tenha voltado ao sistema.
        drop(inner);
        drop(mortas);
    }

    /// ⚠️ Só para a sonda: quantas fitas a cache guarda.
    #[doc(hidden)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entries
            .len()
    }

    /// ⚠️ Só para a sonda.
    #[doc(hidden)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// ⭐⭐ **O braço com que a câmera move a peça** — a distância do `target` ao canto mais afastado da
/// caixa (W148).
///
/// ⚠️ **É o ALVO, e não o centro da peça:** um pan leva o alvo para fora do centro, e uma órbita
/// passa a varrer a peça com um braço maior. *O movimento de um arrasto é `braço × ângulo`, e o braço
/// mede-se de onde a câmera gira.* A sonda que escolheu o [`PAD_OF_REACH`] chama esta mesma função.
#[must_use]
pub fn reach(bbox: Aabb, target: [f32; 3]) -> f32 {
    let mut far = 0.0f32;
    for c in 0..8u8 {
        let p = [
            if c & 1 == 0 { bbox.0[0] } else { bbox.1[0] },
            if c & 2 == 0 { bbox.0[1] } else { bbox.1[1] },
            if c & 4 == 0 { bbox.0[2] } else { bbox.1[2] },
        ];
        let d = (0..3).map(|k| (p[k] - target[k]).powi(2)).sum::<f32>();
        far = far.max(d);
    }
    far.sqrt()
}

/// O deslocamento de fase de uma região, por eixo, com `|u| ≤ amp` — ver [`PHASE`].
///
/// ⚠️ **Uma função, dois leitores** ([`inflate_phased`] e [`pad_phased`]): as duas políticas dispersam
/// as coortes com a MESMA semente, e duas cópias do misturador divergiriam em silêncio.
fn phase_u(seed: u64, amp: f32) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    let mut z = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for o in &mut out {
        // splitmix64: um misturador barato e determinístico — a fase de uma região tem de ser a
        // MESMA em todos os quadros, senão a caixa muda de sítio a cada compilação e a dispersão
        // vira ruído.
        z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut x = z;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^= x >> 31;
        // u ∈ [-amp, amp]
        *o = (((x >> 40) as f32) / 8_388_608.0 - 1.0) * amp;
    }
    out
}

/// ⭐⭐⭐ **A caixa inflada por `f` e DESLOCADA dentro da folga** — ver [`PHASE`]. A política da
/// CAIXA ([`Growth::Box`]).
///
/// ⚠️ **A contenção é uma invariante, não uma esperança:** o deslocamento por eixo é
/// `half·(f−1)·u` com `|u| ≤ amp ≤ 1`, e a folga por lado é exactamente `half·(f−1)` — então a
/// região continua dentro da caixa para toda amplitude admissível. O gate
/// `the_phased_box_still_contains_its_region` afirma-o sobre a amplitude que ship.
#[must_use]
pub fn inflate_phased(lo: [f32; 3], hi: [f32; 3], f: f32, seed: u64, amp: f32) -> Aabb {
    let u = phase_u(seed, amp);
    let mut out = ([0.0f32; 3], [0.0f32; 3]);
    for k in 0..3 {
        let c = 0.5 * (lo[k] + hi[k]);
        let half = 0.5 * (hi[k] - lo[k]);
        let folga = half * (f - 1.0);
        out.0[k] = c - half * f + folga * u[k];
        out.1[k] = c + half * f + folga * u[k];
    }
    out
}

/// ⭐⭐⭐ **A caixa crescida `pad` de cada lado e DESLOCADA dentro da folga** — a política do CASCO
/// ([`Growth::Hull`]).
///
/// ⚠️ **A mesma invariante da [`inflate_phased`]**: o deslocamento é `pad·u` com `|u| ≤ amp ≤ 1`, e a
/// folga por lado é `pad` — a região continua dentro para toda amplitude admissível. O gate
/// `the_padded_region_still_contains_its_query` afirma-o.
///
/// ⚠️ **Só a CAIXA cresce; os cantos do tubo não se mexem.** O casco herda a folga pela caixa: o
/// `hull_uv` lê a folga como o que a caixa tem além dos pontos, e infla o polígono por ela.
#[must_use]
pub fn pad_phased(lo: [f32; 3], hi: [f32; 3], pad: f32, seed: u64, amp: f32) -> Aabb {
    let u = phase_u(seed, amp);
    let mut out = ([0.0f32; 3], [0.0f32; 3]);
    for k in 0..3 {
        out.0[k] = lo[k] - pad + pad * u[k];
        out.1[k] = hi[k] + pad + pad * u[k];
    }
    out
}
