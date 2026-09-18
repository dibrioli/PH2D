//! The attribute stream — columnar per-element data flowing on edges.
//!
//! Houdini's attribute model is the gold standard and is what this implements:
//! a stream is not a scalar, it is a set of **named, typed columns of equal
//! length** (the element count). Modifiers read/write columns by name; a
//! cloner multiplies the element count. Stored SoA (struct-of-arrays) — cache-
//! and GPU-instancing-friendly, and the realized form of a Blender-style field
//! once cooked.
//!
//! Iteration order is deterministic (`BTreeMap`, per ADR-0022) so a cooked
//! stream hashes identically across runs.

use crate::port::Dim;
use rayon::prelude::*;
use std::collections::BTreeMap;

/// Above this element count, [`par_build`] spreads the per-element map across
/// cores; below it stays serial. The threshold exists because rayon's
/// fork/join has a fixed cost that a tiny stream (the boot demo's handful of
/// instances) would pay for nothing — and, more importantly, the small-N path
/// must **not** touch the thread pool or allocate, so the paused/idle no-alloc
/// guarantee (`ph2d-eval-motion/tests/paused_no_alloc.rs`) holds. Correctness is
/// identical on both sides; this is purely a performance floor.
///
/// GPU/M5 Fase 0. Tunable; ~8k is well above any editor-chrome stream and well
/// below where an O(N) node starts to hurt on one core.
pub const PAR_THRESHOLD: usize = 8192;

/// Build a per-element `Vec<T>` by index, in parallel above [`PAR_THRESHOLD`].
///
/// **Determinism contract (the whole reason this is one audited function):**
/// `f(i)` MUST be a pure function of `i` plus shared *immutable* data — no
/// cross-element read whose result depends on iteration order, and **no float
/// reduction** (a sum/average across elements reorders under threads and IEEE
/// addition is not associative). Nodes that reduce or read neighbours
/// (`voronoi`, `boids`, the sims) keep their serial fixed-order pass — see the
/// split in `motion.voronoi`. For a pure map there is nothing to reorder:
/// rayon's indexed `collect` writes element `i` to slot `i`, so the result is
/// **bit-identical** to the serial `(0..n).map(f).collect()`, and the ECS
/// `transform_determinism` / replay-hash gates are unaffected.
pub fn par_build<T, F>(n: usize, f: F) -> Vec<T>
where
    T: Send,
    F: Fn(usize) -> T + Sync + Send,
{
    par_build_if(n >= PAR_THRESHOLD, n, f)
}

/// Como o [`par_build`], mas **quem decide é o chamador**.
///
/// ⚠️ **Porque ela existe** (report do dono, 2026-09-18): o [`PAR_THRESHOLD`] é o ponto de
/// equilíbrio de um nó que corre **UMA** passagem por quadro com um corpo por-elemento pequeno.
/// Um consumidor cujo corpo por-elemento é caro — a separação de contactos varre o vizinhado e faz
/// SAT por par —, e que repete a passagem `varreduras` vezes, tem outro ponto de equilíbrio: a
/// `500` peças ele fica **em série num núcleo** enquanto os outros 31 esperam.
///
/// ⇒ o limiar de cada consumidor é MEDIDO por ele; o que esta porta oferece é a mesma costura
/// auditada de rayon, sem uma segunda no repo. ⭐ A garantia mantém-se: o `collect` indexado
/// preserva a ordem e um `map` puro não tem redução de vírgula flutuante a reordenar, logo os dois
/// lados são **bit-idênticos** (gate: `par_build_is_bit_identical_to_serial_both_sides_of_the_threshold`).
pub fn par_build_if<T, F>(paralelo: bool, n: usize, f: F) -> Vec<T>
where
    T: Send,
    F: Fn(usize) -> T + Sync + Send,
{
    par_build_com_bloco(paralelo, n, || (), |(), i| f(i))
}

/// Como o [`par_build_if`], mas cada trabalhador recebe um **BLOCO DE RASCUNHO** que ele
/// reaproveita entre elementos.
///
/// ⚠️ **Porque ela existe** (report do dono, 2026-09-18): o corpo por-elemento da separação de
/// contactos alocava um `Vec` de vizinhos **por peça e por varredura** — `n × varreduras`
/// alocações, que num quadro com `1024` varreduras e `1000` peças é um milhão.
///
/// ⛔⛔ **E a HIPÓTESE que motivou esta porta foi REFUTADA pela própria medição, que fica escrita
/// aqui:** eu esperava que fosse a contenção no alocador a explicar o paralelo render `1,3×`–`2,5×`
/// em 32 núcleos. Com o bloco reaproveitado o rendimento **não se moveu**. O discriminador — que a
/// carga da máquina não estraga — é o **tempo de CPU contra o de parede**: a corrida paralela ocupa
/// `4`–`5` núcleos e gasta **`5×` o CPU da série para o mesmo trabalho**. ⇒ o que se paga não é
/// alocador, é o **`fork/join` por varredura**: com centenas de bifurcações curtas os
/// trabalhadores passam a vida a GIRAR à espera. *A cura é outra — uma região paralela que
/// atravesse as varreduras — e não é esta porta.*
///
/// ⭐ Ela fica na mesma, porque menos um milhão de alocações por quadro é certo por si.
///
/// ⭐ A garantia de bits é a mesma do irmão: cada elemento é calculado **sozinho** e o `collect`
/// indexado repõe a ordem; o bloco é rascunho, nunca estado que atravesse elementos com
/// significado.
pub fn par_build_com_bloco<T, B, I, F>(paralelo: bool, n: usize, init: I, f: F) -> Vec<T>
where
    T: Send,
    B: Send,
    I: Fn() -> B + Sync + Send,
    F: Fn(&mut B, usize) -> T + Sync + Send,
{
    if paralelo {
        (0..n).into_par_iter().map_init(&init, &f).collect()
    } else {
        let mut bloco = init();
        (0..n).map(|i| f(&mut bloco, i)).collect()
    }
}

/// The identity of the reserved `size` column: **unit scale**.
///
/// A node only writes the columns it changes, so every node that MATERIALIZES
/// `size` on a stream that had none must start from this base (`motion.scale`,
/// the Size channel of `oscillator`/`wiggle`/`noise`/`step`/`stagger`/`drive`,
/// `strobe`, …) — filling the gap with `[0,0]` would collapse every element to
/// nothing.
///
/// **The contract that binds the two ends:** whoever lowers a stream to
/// instances MUST use this same value as its fallback for an absent `size`. If
/// the renderer's fallback and the nodes' base disagree, then a node dropped at
/// its own IDENTITY silently resizes the whole scene — which is exactly what
/// happened (the shell lowered with `0.4` while every node assumed `1.0`, so a
/// `motion.scale` at `amount = 1` blew every quad up by 2.5×; doc 39).
pub const SIZE_IDENTITY: [f32; 2] = [1.0, 1.0];

/// The name of the reserved **value** column — the scalar a `value.*` node emits and a
/// consumer reads (`motion.drive`'s second input, a driven param, doc 58).
///
/// It was a `const VALUE_COL: &str = "v"` re-declared privately in every value node and
/// every consumer of one. That is a convention held together by everyone remembering it;
/// naming it here makes it a fact. (The node crates keep their local aliases — retyping the
/// letter in 30 crates is churn without a gate — but anything NEW reads it from here.)
pub const VALUE_COLUMN: &str = "v";

/// ⭐⭐ **A MÁSCARA QUE SÓ A COR LÊ** — report do Enio (2026-08-30): *"uma opção para livrar as
/// folhas, os frutos do tint que pinta tudo na árvore"*.
///
/// ⛔⛔ **A 1.ª cura usou o `falloff` e PARTIU a planta.** O `falloff` é a máscara de **todos**
/// os modificadores desta casa — o `motion.move` faz `P' = P + (dx, dy) · falloff` —, então pôr
/// `0` numas linhas para as livrar do TINT deixava-as **paradas enquanto o resto se movia**.
/// *O canal escolhido era muito mais largo do que a pergunta feita.*
///
/// ⇒ uma coluna própria: quem a escreve diz *«a cor não alcança esta linha»* e mais nada. Ela
/// é multiplicativa com o `falloff` no `motion.tint`, e **ausente ⇒ `1`** ⇒ toda corrente que
/// não a escreve é byte-idêntica.
///
/// ⚠️ **Declarada AQUI, e não em cada crate**, ao contrário do `falloff_y` (que vive em duas
/// cópias privadas): dois lados que escrevem a mesma string à mão são duas leis à espera de
/// divergir, e esta nasce com dois lados no mesmo dia.
pub const TINT_MASK_COLUMN: &str = "tint_mask";

/// ⭐⭐ **O COLISOR DE UM ELEMENTO** — doc 109, ordem do dono (2026-09-13): *«colidem sozinhas»*.
///
/// O raio de colisão **na unidade da GEOMETRIA** do elemento; o raio de mundo é
/// `collider × max(|sx|, |sy|)` — a lei do `motion.collide` (*o disco que CONTÉM a arte*), para
/// que escalar a peça a jusante escale o colisor com ela, como em todo motor.
///
/// ⚠️ **Ausente ⇒ o elemento NÃO colide.** É a lei do `Collider` da física de corpos rígidos
/// (*«its absence is the off»*), e é o que mantém toda corrente de hoje byte-idêntica.
///
/// ⚠️ **Quem o escreve é quem DESENHA** (o `source.shape`): um consumidor não sabe se recebeu uma
/// sprite (raio inscrito `size/2`) ou uma forma vectorial (raio `size`), e a razão entre as duas
/// é da mídia e da geometria. Declarada aqui porque tem dois lados que nascem na mesma obra.
pub const COLLIDER_COLUMN: &str = "collider";
/// ⭐⭐ **A CAIXA de colisão** (doc 109 §5 — report do dono, 2026-09-13: *«o collider não é gerado
/// conforme a forma da Shape»*): as MEIAS extensões `[hx, hy]` na unidade da geometria do
/// elemento, antes da escala (`size`) e da rotação (`rot`). Presente e válida (finitas, `≥ 0`, uma
/// delas `> 0`), ela é o colisor do elemento e a [`COLLIDER_COLUMN`] não é lida; ausente — ou
/// `[0, 0]`, que é o que a união do `motion.combine` preenche —, o elemento é um disco ou não
/// colide. A porta que a lê é a `ph2d_contact::colisores`.
pub const COLLIDER_BOX_COLUMN: &str = "collider_box";
/// **O CENTRO do colisor** relativo à origem do elemento, na unidade da geometria (antes de `size`
/// e de `rot`). Ausente ⇒ `[0, 0]`: o colisor centrado em `P`, que é o que toda declaração anterior
/// a esta coluna quer dizer.
pub const COLLIDER_OFFSET_COLUMN: &str = "collider_offset";
/// ⭐⭐ **O INVERSO DA INÉRCIA de rotação** (doc 109 §6 — report do dono, 2026-09-13: *«precisa
/// destravar a rot. e colocar outro botão para travar rotação»*): `0` TRAVA a rotação da peça, e a
/// coluna AUSENTE quer dizer *«deriva da forma»* (uma caixa `3·w / (hx² + hy²)`, um disco `2·w / r²`,
/// com `w` a coluna `inv_mass`). É o irmão angular dela, e a razão
/// de ser uma COLUNA é a mesma: quem desenha é quem sabe, e a declaração viaja com a peça.
pub const INV_INERTIA_COLUMN: &str = "inv_inertia";
/// ⭐⭐⭐ **O ATRITO da peça** (doc 109 §7 — report do dono, 2026-09-13: *«os círculos não rotacionam
/// com a colisão, talvez por falta de atrito. Precisamos de parâmetros do material»*): o
/// coeficiente de Coulomb, `0..1`. **Ausente ⇒ `0` ⇒ gelo**, que é a lei de antes desta coluna, ao
/// bit — e é por isso que ela pode nascer sem migração nenhuma.
///
/// ⚠️ **O par combina-se pela média GEOMÉTRICA** (`√(μa·μb)`, a lei do Box2D), na porta
/// `ph2d_contact::atrito::mu` — uma peça de gelo desliza contra tudo, que é o que «gelo» quer
/// dizer. Um `max` faria uma peça de lixa colar tudo o resto ao chão.
pub const FRICTION_COLUMN: &str = "friction";
/// ⭐⭐ **O SALTO da peça** (doc 109 §7): quanto de um embate volta, `0..`[`BOUNCE_MAX`].
/// **Ausente ⇒ `0` ⇒ morto.** O par combina-se pelo MAIOR dos dois (Box2D): uma bola saltitante
/// salta contra uma parede morta.
///
/// ⚠️ **Ela não se chama `restitution`** de propósito: esse nome já é um **param** do
/// `sim.collide` (o do OBSTÁCULO), e duas grandezas com o mesmo nome em sítios diferentes é como
/// um dia alguém lê a do obstáculo julgando ler a da peça. Aqui a coluna é da PEÇA.
pub const BOUNCE_COLUMN: &str = "bounce";
/// ⭐⭐⭐ **O ATRITO DE ROLAMENTO da peça** (doc 109 §7.10 — a ponta que o §7.7 nomeava: *«com
/// `angular_damping = 1` ela rola para sempre num chão infinito»*): o que faz uma bola a rolar
/// **parar sozinha**. **Ausente ⇒ `0` ⇒ rola para sempre**, que é a lei de antes desta coluna.
///
/// ⚠️⚠️ **Ela é a ÚNICA das três que NÃO se combina por par, e a escolha é deliberada.** O atrito
/// e o salto são propriedades de *duas superfícies a esfregar-se*; o rolamento modela a peça a
/// **achatar-se** contra o que toca, logo é dela. ⛔ Combinar por média geométrica (como o atrito)
/// daria `0` em toda cena que existe — nenhum obstáculo declara rolamento — e o artista veria um
/// controlo que **não faz nada**, que é exactamente o report que o §7.6 já custou.
///
/// ✅ **A fronteira que este parágrafo DECLARAVA fechou em 2026-09-16** (doc 111 §10). A redacção
/// era: *«no contacto peça × peça a rotação é posicional e não tem velocidade angular a resistir —
/// ali não há nada que este número possa travar»*. Era verdade, e fazia o botão do cartão ser
/// **morto** numa pilha. O doc 111 §9 deu velocidade angular ao contacto peça × peça, e o §10 ligou
/// este número lá pela MESMA porta (`ph2d_contact::atrito::rolamento`) e na MESMA forma que a taça:
/// **cada peça travada contra o próprio giro, com o próprio rolamento**. ⛔ A forma do PAR (o giro
/// relativo) foi medida e refutada — ela fazia o botão AGITAR o monte em vez de o acalmar.
pub const ROLLING_COLUMN: &str = "rolling";

/// ⭐⭐⭐ **O TECTO DO SALTO é `1`, o de todo motor — ordem do dono (2026-09-15: *«Limite Bounciness
/// para máximo de 1»*), que REVERTE a dele própria de 2026-09-13.**
///
/// ⚠️⚠️ **A medição que abriu a faixa para `2` continua VÁLIDA e está abaixo, intacta** — ela
/// respondia *«o que acontece acima de `1`?»* e a resposta (*sobe cada vez mais, tudo finito, nada
/// atravessa o chão*) não mudou. ⛔ *O que mudou foi o VEREDITO DE PRODUTO, e ele não precisa de
/// desmentir a medição para valer:* o dono viu a faixa aberta no app e decidiu que a não quer.
/// A tabela fica porque, no dia em que alguém a quiser reabrir, ela é o que poupa a medição toda.
///
/// ---
///
/// **A medição de 2026-09-13**, que shipou o `2` (ordem de então: *«quero mais capacidade de
/// Bounciness — de zero até o dobro do máximo atual»*).
///
/// ⚠️⚠️ **A frase que segurava o `1` nunca tinha sido MEDIDA neste solver.** Ela vive no
/// `sim.collide` desde que ele existe — *«uma batida que devolve mais do que levou é uma máquina
/// de fazer energia, e acaba com a cena em órbita»* — e é verdadeira em Física e **falsa sobre o
/// que este código faz**. Medido numa queda de `0,75` sob gravidade `4`, **sem laço**, 40 s:
/// `1,00` → pico `0,95` · `1,25` → `39,6` · `1,50` → `91,8` · **`2,00` → `192,0`** · `3,00` →
/// `550,8`, **todos finitos, nenhum a atravessar o chão**. O que acontece acima de `1` é a bola
/// subir cada vez mais — que é o que o artista pede ao passar de `1`.
///
/// ⚠️ **O limite REAL é outro e tem recurso com nome:** acima de `2R/dt` a peça atravessa o
/// próprio diâmetro num tique, e aí colidir com algo que não seja um plano infinito deixa de ser
/// fiável. ⛔ Isso é propriedade do INTEGRADOR e não deste número — a cura que já existe é o
/// `Speed Limit` do `sim.step`. Numa cena em LAÇO o problema não nasce (a `=115` a `2,00` fica em
/// pico `3,2`). Tabela inteira: `docs/Motion Nodes/109_o_colisor_na_forma.md` §7.8.
pub const BOUNCE_MAX: f32 = 1.0;

/// O tecto do [`FRICTION_COLUMN`]. `1` é a lixa de todo motor, e subi-lo não compra nada: o
/// impulso tangencial já está limitado por Coulomb ao que a normal aguenta.
pub const FRICTION_MAX: f32 = 1.0;

/// **O TECTO DO ROLAMENTO** ([`ROLLING_COLUMN`]) — MEDIDO, e o número é onde a curva PÁRA de
/// responder, não um lugar redondo (§0.0).
///
/// Sonda `probe_the_rolling_ball_stops` (`ph2d-node-sim-collide`), nas condições da cena `=115`:
/// raio `0,2`, gravidade `4`, `dt = 1/60`, a bola já **a rolar** a `1 u/s` num chão plano de
/// `μ = 1` — segundos até parar:
///
/// ```text
///   rolamento │ 0,00  0,02  0,05  0,10  0,25  0,50  1,00 │ 1,50  2,00  3,00  4,00  8,00
///   pára em s │  ∞   18,57  7,43  3,72  1,50  0,78  0,43 │ 0,33  0,33  0,33  0,33  0,33
/// ```
///
/// ⭐⭐ **A `1,50` a coluna SATURA, e o que a segura não é esta lei — é o ATRITO DE COULOMB.** Com o
/// rolamento a matar todo o `ω` em cada tique, a bola passa a ser um bloco a derrapar, e aí quem a
/// trava é `μ·g`: `v/(μ·g) = 0,25 s` em teoria, `0,33` medidos (o contacto não acontece em todos os
/// tiques). ⇒ o recurso que fecha esta faixa tem nome e é de **outra lei**; escrever mais do que
/// `1,5` seria dar ao artista curso de deslizante que devolve sempre o mesmo número.
///
/// ⛔ E não há divergência a temer: o impulso é limitado ao momento angular que a peça **tem**
/// (`ω/invI`), logo o pior caso é parar a rotação neste tique — nunca invertê-la.
pub const ROLLING_MAX: f32 = 1.5;

/// **A PORTA da coerção de um coeficiente de material** — `0..teto`, e um não-finito lê como `0`
/// (o neutro desta grandeza, não a identidade).
///
/// ⚠️ **Ela mora aqui, ao lado das colunas, e não num dos lados:** quem DECLARA (o `source.shape`)
/// e quem CONSOME (o `ph2d-contact`) têm de coagir pelo mesmo tecto, senão o cartão deixa autorar
/// um número que o solver corta — e o artista vê um slider que pára de responder a meio do curso.
#[must_use]
pub fn material_coerce(x: f32, teto: f32) -> f32 {
    if x.is_finite() {
        x.clamp(0.0, teto.max(0.0)) // CLAMP-OK: teto >= 0 forçado
    } else {
        0.0
    }
}

/// As colunas de **ESCRITURAÇÃO** — aquelas cuja máquina de estado de um nó a
/// jusante lê, e que por isso um escritor genérico não pode sobrescrever.
///
/// ⚠️ **Esta NÃO é a pergunta que o picker do painel faz.** O `INTERNAL` do shell
/// responde *"o que a lista OFERECE ao artista para LER?"*, e as duas respostas
/// divergem de propósito: `falloff` é lido e **escrito** (a família `field.*`
/// inteira existe para o escrever), enquanto `id` é lido e nunca escrito.
/// Colapsá-las tornaria o `falloff` inescrevível.
///
/// ⚠️ **A pergunta não é derivável, e a alternativa foi MEDIDA:** perguntar ao
/// TIPO (*"a coluna que já está lá é escalar?"*) não protege nada aqui — `id` e
/// `sim_t` **são** escalares, tal como `texture_id` e `geometry_id`. O que os
/// separa não é a forma, é o PAPEL, e nenhuma varredura do substrato sabe que
/// um `sim_t` no futuro faz `dt` clampar em zero e **congelar a simulação em
/// silêncio**, ou que trocar um `id` faz o pareamento por identidade entregar a
/// linha de história ao elemento errado. Os dois modos de falha são MUDOS, que é
/// o que justifica um nome escrito à mão em vez de um palpite estrutural.
///
/// ⚠️ **O prefixo `dl_` é do RING do `motion.delay`** e entra aqui pelo mesmo
/// motivo, apesar de aquela crate ter o `is_state` dela: uma crate-nó não pode
/// depender de outra (ADR-0075), então a pergunta *"posso escrever aqui?"* só
/// tem uma casa possível — esta.
#[must_use]
pub fn is_bookkeeping_column(name: &str) -> bool {
    matches!(name, "id" | "sim_t" | "sim_d") || name.starts_with("dl_")
}

/// A typed column of per-element values, stored SoA.
#[derive(Clone, Debug, PartialEq)]
pub enum Column {
    Scalar(Vec<f32>),
    Vec2(Vec<[f32; 2]>),
    Vec3(Vec<[f32; 3]>),
    Vec4(Vec<[f32; 4]>),
}

impl Column {
    /// The heap bytes this column's data occupies — the honest input to a
    /// byte-budgeted cache (ADR-0137: a checkpoint ring capped by COUNT is a
    /// multiplier, the ADR-0117 class). An estimate by design: capacity slack
    /// and allocator overhead are not counted, which errs on the small side —
    /// the direction a budget must NOT err — so callers pad their budgets, not
    /// this number.
    pub fn approx_bytes(&self) -> usize {
        match self {
            Column::Scalar(v) => v.len() * 4,
            Column::Vec2(v) => v.len() * 8,
            Column::Vec3(v) => v.len() * 12,
            Column::Vec4(v) => v.len() * 16,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Column::Scalar(v) => v.len(),
            Column::Vec2(v) => v.len(),
            Column::Vec3(v) => v.len(),
            Column::Vec4(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn dim(&self) -> Dim {
        match self {
            Column::Scalar(_) => Dim::Scalar,
            Column::Vec2(_) => Dim::Vec2,
            Column::Vec3(_) => Dim::Vec3,
            Column::Vec4(_) => Dim::Vec4,
        }
    }
}

/// A stream of `count` elements with named typed columns. An empty stream
/// (`count == 0`, no columns) is the value of an unconnected input.
/// **Cloning a `Stream` is a refcount, not a copy** (ADR-0138): the columns
/// live behind `Arc`, so the sim loop's per-tick snapshots (`Cook::checkpoint`,
/// `advance_tick`'s prev, the pump's boundary hand-off) share data instead of
/// re-copying the whole field. Sound because nothing mutates a `Column` in
/// place — the API has no `get_mut`, and every writer builds a fresh column and
/// [`Stream::set`]s it (the same immutability the GPU side's buffers rely on).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stream {
    count: usize,
    attrs: BTreeMap<String, std::sync::Arc<Column>>,
}

impl Stream {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            attrs: BTreeMap::new(),
        }
    }

    /// The empty stream (zero elements, no columns) as a `const` — the value of
    /// an unconnected input. Equal to [`Stream::default`], but `const` so a
    /// `static EMPTY_STREAM` can be borrowed for `&Stream` accessors (see
    /// [`crate::value::CookValue::as_stream`]).
    pub const fn empty() -> Self {
        Self {
            count: 0,
            attrs: BTreeMap::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Attach/replace a column. The column length must equal the stream's
    /// element count (a per-element attribute). Returns the stream for
    /// chaining. Panics on length mismatch — a programming error in a node.
    pub fn with(mut self, name: impl Into<String>, col: Column) -> Self {
        self.set(name, col);
        self
    }

    pub fn set(&mut self, name: impl Into<String>, col: Column) {
        assert_eq!(
            col.len(),
            self.count,
            "column length must equal stream element count"
        );
        self.attrs.insert(name.into(), std::sync::Arc::new(col));
    }

    pub fn get(&self, name: &str) -> Option<&Column> {
        self.attrs.get(name).map(std::sync::Arc::as_ref)
    }

    /// Deterministic iteration over `(name, column)` pairs.
    /// The stream's column data in bytes — see [`Column::approx_bytes`].
    pub fn approx_bytes(&self) -> usize {
        self.columns().map(|(_, c)| c.approx_bytes()).sum()
    }

    pub fn columns(&self) -> impl Iterator<Item = (&String, &Column)> {
        self.attrs.iter().map(|(n, c)| (n, c.as_ref()))
    }

    /// ⭐⭐⭐ **ISTO É O MESMO STREAM QUE AQUELE?** — as duas contagens iguais e **as mesmas
    /// ALOCAÇÕES** em todas as colunas, na mesma ordem e com os mesmos nomes.
    ///
    /// ⚠️ **É identidade, não igualdade**, e é de propósito: uma coisa que não mudou partilha as
    /// colunas com a versão de ontem (*clonar partilha, escrever substitui* — a lei que o teste
    /// desta secção pina), então a pergunta responde-se com um ponteiro por coluna em vez de
    /// percorrer um milhão de números. Dois streams com o mesmo CONTEÚDO e alocações diferentes
    /// respondem `false` — o pior que acontece é fazer-se o trabalho que já se fazia.
    ///
    /// ⚠️ **A imutabilidade é o que a torna segura:** uma coluna nunca é escrita no sítio (não há
    /// `get_mut`), então o mesmo ponteiro é sempre o mesmo conteúdo. Quem guardar o stream ao lado
    /// da resposta (um cache) segura o `Arc` e impede que a alocação seja libertada e reutilizada
    /// noutro sítio — que é a única forma de este `ptr::eq` mentir.
    ///
    /// Os dois consumidores: o [`crate::cook::Cook::set_external`] (não voltar a HASHAR o que não
    /// mudou) e o envio da costura para o dispositivo (não voltar a ENVIAR) — ciclo 8, doc 113 §6.
    #[must_use]
    pub fn shares_storage_with(&self, other: &Self) -> bool {
        self.count == other.count
            && self.attrs.len() == other.attrs.len()
            && self
                .attrs
                .iter()
                .zip(other.attrs.iter())
                .all(|((na, ca), (nb, cb))| na == nb && std::sync::Arc::ptr_eq(ca, cb))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_holds_typed_columns() {
        let s = Stream::new(3)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]))
            .with("size", Column::Scalar(vec![1.0, 1.0, 1.0]));
        assert_eq!(s.count(), 3);
        assert_eq!(s.get("P").unwrap().dim(), Dim::Vec2);
        assert_eq!(s.get("size").unwrap().len(), 3);
        assert!(s.get("missing").is_none());
    }

    #[test]
    #[should_panic(expected = "column length must equal stream element count")]
    fn mismatched_column_length_panics() {
        let mut s = Stream::new(3);
        s.set("bad", Column::Scalar(vec![1.0])); // len 1 != count 3
    }

    /// **Cloning shares, writing replaces** (ADR-0138): the clone's columns are
    /// the SAME allocations (a refcount, not a copy — what makes the sim loop's
    /// per-tick snapshots cheap), and a `set` on the clone swaps in a fresh
    /// column without touching the original. A `Stream` whose clone deep-copied
    /// would fail the pointer identity here — the exact regression this pins.
    /// ⭐⭐ **A régua da identidade** ([`Stream::shares_storage_with`]): o clone partilha, a escrita
    /// desfaz, e um stream com o MESMO conteúdo noutra alocação responde `false` — o lado
    /// conservador, que só custa o trabalho que já se fazia.
    #[test]
    fn shared_storage_is_identity_and_never_equality() {
        let a = Stream::new(2)
            .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
            .with("size", Column::Scalar(vec![1.0, 1.0]));
        let b = a.clone();
        assert!(a.shares_storage_with(&b), "clonar partilha");
        let mut c = a.clone();
        c.set("size", Column::Scalar(vec![1.0, 1.0]));
        assert!(
            !a.shares_storage_with(&c),
            "escrever desfaz a partilha, mesmo escrevendo o MESMO conteudo"
        );
        let d = Stream::new(2)
            .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
            .with("size", Column::Scalar(vec![1.0, 1.0]));
        assert_eq!(a.get("P"), d.get("P"), "o conteudo e' o mesmo...");
        assert!(!a.shares_storage_with(&d), "...e a alocacao nao");
        // E as duas metades da contagem de colunas.
        let e = a.clone().with("extra", Column::Scalar(vec![0.0, 0.0]));
        assert!(
            !a.shares_storage_with(&e),
            "uma coluna a mais nao e' o mesmo stream"
        );
    }

    #[test]
    fn cloning_a_stream_shares_columns_and_writing_replaces_them() {
        let a = Stream::new(2).with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]));
        let mut b = a.clone();
        assert!(
            std::ptr::eq(a.get("P").unwrap(), b.get("P").unwrap()),
            "a cloned column must be the same allocation"
        );
        b.set("P", Column::Vec2(vec![[9.0, 9.0], [8.0, 8.0]]));
        assert!(
            !std::ptr::eq(a.get("P").unwrap(), b.get("P").unwrap()),
            "writing un-shares"
        );
        assert_eq!(
            a.get("P").unwrap(),
            &Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]),
            "the original is untouched"
        );
    }

    #[test]
    fn empty_stream_is_default() {
        assert_eq!(Stream::default(), Stream::new(0));
        assert!(Stream::default().is_empty());
    }

    #[test]
    fn par_build_is_bit_identical_to_serial_both_sides_of_the_threshold() {
        // A closure with the shape a node uses: a per-element f32 map that would
        // ULP-drift if the arithmetic were reordered (mul + add of a fractional
        // index). Above the threshold `par_build` runs on the thread pool; the
        // result MUST equal the explicit serial map byte-for-byte, or a
        // parallelized node would silently diverge from the CPU canon.
        let f = |i: usize| (i as f32) * 0.1 + (i as f32).sqrt() * 1.3;
        for n in [
            0usize,
            1,
            100,
            PAR_THRESHOLD - 1,
            PAR_THRESHOLD,
            PAR_THRESHOLD * 4 + 7,
        ] {
            let serial: Vec<f32> = (0..n).map(f).collect();
            let built = par_build(n, f);
            assert_eq!(built, serial, "par_build diverged from serial at n = {n}");
        }
    }

    #[test]
    fn par_build_preserves_index_order() {
        // The identity map: element i must land at slot i (rayon's indexed
        // collect), not merely contain the same multiset — a set-preserving but
        // order-shuffling collect would pass a hash-of-sorted check yet corrupt
        // the P/tint/size correspondence across columns.
        let n = PAR_THRESHOLD * 2;
        let built = par_build(n, |i| i as u32);
        assert!(built.iter().copied().eq(0..n as u32));
    }
}
