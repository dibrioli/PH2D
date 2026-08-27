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
//! # ⚠️ O que esta cache DEIXA CAIR de propósito, e o preço disso
//!
//! O caminho sem cache especializa contra o **casco** do tubo do ladrilho (W59), que é mais apertado
//! que a caixa. Uma fita guardada tem de valer numa forma que se possa **testar depressa e sem
//! ambiguidade**, e essa forma é a **caixa**. ⇒ uma fita da cache guarda mais arestas que a fita de
//! hoje, mesmo antes de a inflar. *A cache troca aresta por compilação, e é por isso que ela só se
//! decide com o relógio do quadro inteiro ao lado.*

use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::RegionTape;

/// ⭐⭐⭐ **Quanto uma região é inflada antes de a sua fita ser compilada** — **medido**, ver o doc
/// do módulo.
///
/// ⚠️ **Ele não é um número de conforto: é o mecanismo.** A `1,00` a cache acerta `9 %` e não serve
/// para nada; a `1,25` acerta `84 %`–`93 %` às velocidades de arrasto reais (um quadro de `24 ms` a
/// `90°/s` é `2,2°`) por `1,18×` no custo de uma amostra. Subir mais compra acerto num arrasto
/// **rápido**, que é exactamente onde o artista tolera menos detalhe.
pub const INFLATE: f32 = 1.25;

/// Quantas fitas a cache guarda antes de deitar metade fora.
///
/// ⚠️ **Ele diz de que recurso é: MEMÓRIA EXECUTÁVEL.** Cada fita é um `mmap` do tamanho do código
/// dela; `2048` regiões de um contorno de 168 arestas são da ordem de dezenas de MiB. O despejo é
/// por **idade de uso**, e não por ordem de entrada: numa órbita contínua as regiões antigas nunca
/// mais são pedidas, e as do quadro corrente são pedidas todas.
const CAPACITY: usize = 2048;

/// ⭐⭐ **Quantas fitas vieram da cache** — o par do `FLOAT_TAPES`, que conta as que foram
/// compiladas.
///
/// ⚠️ *Contar o trabalho feito não é contar o trabalho poupado.* Sem este contador, uma cache que
/// nunca acerta é indistinguível de uma que acerta sempre — a imagem é a mesma nas duas.
#[doc(hidden)]
pub static TAPE_HITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Uma caixa de mundo.
type Aabb = ([f32; 3], [f32; 3]);

struct Entry {
    lo: [f32; 3],
    hi: [f32; 3],
    tape: RegionTape,
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
    /// ⚠️ Quanto esta cache infla — ver [`INFLATE`]. É um campo, e não a constante lida
    /// directamente, porque a **varredura** que a escolheu tem de poder correr as duas respostas no
    /// mesmo processo: entre duas corridas desta workstation o mesmo passe já deu `11,36` e
    /// `5,50 ms`.
    inflate: f32,
    /// ⚠️ **O documento a que estas fitas pertencem.** Editar a peça muda toda especialização, e
    /// uma fita da peça de ontem responde um número plausível e errado — o pior modo de falha que
    /// há, porque a imagem sai *quase* certa.
    doc: Option<FieldDoc>,
    frame: u64,
    entries: Vec<Entry>,
}

/// ⭐⭐⭐ A cache. Ver o doc do módulo.
///
/// ⚠️ **Ela vive ENTRE quadros**, então não pode pertencer ao `RegionCompiler`, que nasce e morre
/// com um. O dono dela é quem desenha.
pub struct TapeCache {
    inner: std::sync::RwLock<Inner>,
}

impl Default for TapeCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TapeCache {
    #[must_use]
    pub fn new() -> Self {
        Self::with_inflate(INFLATE)
    }

    /// ⚠️ Só para a sonda: a mesma cache com outra inflação — ver [`INFLATE`].
    #[doc(hidden)]
    #[must_use]
    pub fn with_inflate(f: f32) -> Self {
        Self {
            inner: std::sync::RwLock::new(Inner {
                inflate: f,
                doc: None,
                frame: 0,
                entries: Vec::new(),
            }),
        }
    }

    /// ⚠️ Só para a sonda.
    #[doc(hidden)]
    #[must_use]
    pub fn inflate_of(&self) -> f32 {
        self.inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .inflate
    }

    /// ⭐ **Abre um quadro** — e deita tudo fora se o documento mudou.
    ///
    /// ⚠️ **A comparação é por VALOR, e é barata ao lado do que ela guarda:** um `FieldDoc` de uma
    /// peça de perfil são alguns kiB de `f32` contra os `~14 ms` de compilação que a cache existe
    /// para não repetir. *Uma cache que não sabe quando morrer é uma fonte de imagens erradas.*
    pub fn begin(&self, doc: &FieldDoc) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if inner.doc.as_ref() != Some(doc) {
            inner.doc = Some(doc.clone());
            inner.entries.clear();
        }
        inner.frame = inner.frame.wrapping_add(1);
    }

    /// A fita que **contém** `[lo, hi]`, se alguma houver.
    ///
    /// ⚠️ **Tudo debaixo do cadeado de LEITURA** — ver o campo `seen`.
    #[must_use]
    pub fn get(&self, lo: [f32; 3], hi: [f32; 3]) -> Option<RegionTape> {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let e = inner
            .entries
            .iter()
            .find(|e| (0..3).all(|k| lo[k] >= e.lo[k] && hi[k] <= e.hi[k]))?;
        e.seen
            .store(inner.frame, std::sync::atomic::Ordering::Relaxed);
        TAPE_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Some(e.tape.clone())
    }

    /// Guarda uma fita construída para a caixa `[lo, hi]`.
    pub fn insert(&self, lo: [f32; 3], hi: [f32; 3], tape: RegionTape) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let seen = inner.frame;
        if inner.entries.len() >= CAPACITY {
            // ⚠️ **Despejo por IDADE DE USO, e metade de uma vez.** Deitar uma fora por cada uma
            // que entra faria o quadro em que a cache enche pagar um despejo por região; deitar
            // metade paga-o uma vez em muitos quadros.
            let mut ages: Vec<u64> = inner
                .entries
                .iter()
                .map(|e| e.seen.load(std::sync::atomic::Ordering::Relaxed))
                .collect();
            ages.sort_unstable();
            let corte = ages[ages.len() / 2];
            inner
                .entries
                .retain(|e| e.seen.load(std::sync::atomic::Ordering::Relaxed) > corte);
        }
        inner.entries.push(Entry {
            lo,
            hi,
            tape,
            seen: std::sync::atomic::AtomicU64::new(seen),
        });
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

/// ⭐ **A caixa inflada por `f` em torno do próprio centro** — ver [`INFLATE`].
#[must_use]
pub fn inflate(lo: [f32; 3], hi: [f32; 3], f: f32) -> Aabb {
    let mut out = ([0.0f32; 3], [0.0f32; 3]);
    for k in 0..3 {
        let c = 0.5 * (lo[k] + hi[k]);
        let half = 0.5 * (hi[k] - lo[k]) * f;
        out.0[k] = c - half;
        out.1[k] = c + half;
    }
    out
}
