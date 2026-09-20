//! **How a kernel touches a stream column** — the [`ColumnAccess`] verb and the
//! [`ColumnBinding`] bundle, split from `gpu.rs` at the LOC cap. Pure data; the
//! sequencer reads these to decide what buffer each binding gets and how the
//! generated body reads/writes it.

use crate::port::Dim;

/// How a kernel touches one named stream column.
///
/// Writes always land in a **fresh** buffer (the sequencer never mutates a
/// cooked column in place — the implicit ping-pong), so `ReadWrite` means
/// "read the bound input port's column, write this node's output column".
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColumnAccess {
    /// Read-only. If the input stream lacks the column, the generated module
    /// substitutes the binding's [`ColumnBinding::identity`] as a constant —
    /// the same absent-column fallback the CPU nodes apply (e.g. `falloff` → 1).
    Read,
    /// Write-only (a generator's output). The column is materialized on the
    /// node's output stream.
    Write,
    /// Read + write; an **absent** input column is materialized from
    /// [`ColumnBinding::identity`] — matching CPU behaviours that build their
    /// target channel from its identity when the stream lacks it
    /// (`apply_channel_delta`'s `base_vec2`, doc 39).
    ReadWrite,
    /// Read + write, but **only when the input stream carries the column** —
    /// when absent the write is dropped and the column stays absent, matching
    /// CPU modifiers that pattern-match the column and otherwise pass through
    /// (`motion.move` touches `P` only if `P` exists).
    ReadWriteExisting,
    /// Read + **consume**: the body reads the column, and the node's output
    /// does NOT carry it — the transient-channel convention
    /// (`motion.integrate` eats the `accel` the in-loop forces accumulated, so
    /// every tick starts from zero acceleration). Without this the column would
    /// ride through from the base input (see [`ColumnBinding::port`]) and the
    /// next tick's forces would add onto a stale value, which is exactly what
    /// the CPU's `add_accel` does — divergence, not ε.
    Consume,
    /// **Not an access — a plan-time refusal.** The kernel declares that it
    /// cannot answer correctly for a stream carrying this column on this port,
    /// so [`crate::gpu`]'s sequencer refuses to claim the node and the CPU
    /// `eval` (the canonical path) owns it.
    ///
    /// The case that named it: `motion.integrate` pairs state to elements by
    /// **`id`** when the stream has identity (a gather) and **positionally**
    /// otherwise; the GPU kernel only does positional, so an `id` column on its
    /// `rest` port is a refusal (ADR-0127 D3). The eligibility answer must be
    /// provable, so an input whose columns the plan cannot derive (a CPU
    /// boundary feeds it) is refused too — **the default is to recede, never to
    /// answer wrong**.
    ///
    /// [`ColumnBinding::dim`] / [`ColumnBinding::identity`] are meaningless
    /// here (nothing is read): the binding declares only *which column on which
    /// port* is disqualifying.
    RefuseIfPresent,
    /// Read-only, and a **length-1 port BROADCASTS**: element `i` reads row `i`
    /// when the port is the dispatch length, and row `0` when the port carries
    /// exactly one value — one number held across the whole field.
    ///
    /// This is the third length rule in the engine, and it is the artist-visible
    /// one. `motion.look_at`'s target is two VALUE fields: wire a bare
    /// `value.lfo` and the ENTIRE flock turns toward one moving point; wire a
    /// per-element field and each element aims somewhere else. The CPU expresses
    /// it as `target_at` (`0 => identity, 1 => vals[0], _ => vals[i]`), and this
    /// is that function, declared.
    ///
    /// Plain [`Self::Read`] cannot express it: presence there means *"this port
    /// is the dispatch length"*, so a length-1 target would be judged ABSENT and
    /// silently read its identity — the flock would face the origin instead of
    /// the point the artist animated. Absent still reads the identity (the
    /// `0 =>` arm); only the length-1 case is new.
    ReadBroadcast,
    /// **A leitura mapeada na FONTE que também ESCREVE, e só quando a coluna existe** — a cruza
    /// do [`Self::SourceRead`] com o [`Self::ReadWriteExisting`], e a peça que faltava para um
    /// kernel `SourceRows` poder **RENUMERAR**.
    ///
    /// ⛔⛔ **As duas metades são obrigatórias e nenhuma das outras variantes as tem juntas:**
    ///
    /// - **ler na FONTE** (`i % n`, o índice que o corpo calcula), porque num kernel que muda a
    ///   contagem o elemento `i` da saída não é o elemento `i` de porta nenhuma — o
    ///   `ReadWriteExisting` lê em `i` e leria fora de alcance;
    /// - **escrever SÓ SE PRESENTE**, porque o que o corpo não escreve chega por GATHER (uma
    ///   cópia do template) e escrever uma coluna ausente **CUNHA-A** — uma coluna a mais viaja,
    ///   é serializada e muda o que um nó a jusante vê.
    ///
    /// ⚠️ **Ela existe por uma MEDIÇÃO e não por simetria** (doc 116 §5.3): varridas as cenas
    /// porta a porta de todo multiplicador, **`40`** trazem `Index` e `Count` e **`38`** não — ou
    /// seja, *«escrever sempre»* cunha em metade do produto e *«recuar sempre»* perde a outra
    /// metade. O braço do `match` da CPU que ela exprime é literalmente
    /// `("Index", Column::Scalar(v)) => …`, que só toca a coluna quando ela existe.
    ///
    /// ⭐ O `motion.kaleidoscope` contornou isto **recuando** (`applicable: |p| p(REINDEX) < 0.5`),
    /// o que ali é aceitável porque a renumeração dele é um knob opcional; num cloner ela é
    /// **incondicional**, logo o mesmo contorno recusaria o nó sempre.
    SourceReadWriteExisting,
    /// **The `id`-gather key** (ADR-0130), the conditional successor to
    /// [`Self::RefuseIfPresent`] for `motion.integrate`/`motion.spring`. Names
    /// the column this node pairs its per-element state by — an `id` on the
    /// **base** port.
    ///
    /// It is BOTH a plan-time guard and a real read:
    ///
    /// - **Plan:** like [`Self::RefuseIfPresent`], the node recedes when this
    ///   column is present on its port — **except** when the plan can prove that
    ///   port's stream is a **dense ascending id window**
    ///   ([`crate::gpu::KernelResolver::keeps_dense_window`]). A dense window is
    ///   exactly the case where the gather is arithmetic (`current_id −
    ///   prev_first`, a row offset) rather than a hash/sort — so the node CLAIMS
    ///   it. A present-but-not-dense `id` (a `sort`/`cull` broke the window) or
    ///   an unprovable shape still recedes: the default is to answer right on the
    ///   CPU, never wrong on the GPU.
    /// - **Codegen:** it [`reads`](Self::reads) the current element's id (so the
    ///   generated module can compute the gather row). The column rides through
    ///   the base like any unwritten column; nothing is written or consumed.
    ///
    /// The generated module gives the body `gather_row(i)` (the paired state row
    /// — `i` when the input is not a dense window, `current_id − prev_first`
    /// when it is) and `gather_paired(i)` (whether that row exists in the prior
    /// state, distinct from the global "is there any prior state?" —
    /// [[feedback_layered_defenses_need_per_layer_gates]]). Reading `prev_first`
    /// needs the SAME column bound on the state port too (a plain [`Self::Read`]).
    GatherKey,
    /// **A read from a TEMPLATE port whose length is decoupled from the dispatch**
    /// (ADR-0136, the `StreamOp::SourceRows` family). A count-changing node
    /// dispatches at its OUTPUT length (its `count_law`) while its template port
    /// carries the SOURCE length — so a plain [`Self::Read`], which is present
    /// only at the dispatch length, would judge the column ABSENT and hand back
    /// its identity at every element.
    ///
    /// The read index is **the kernel's own arithmetic**, not `i`: element `i`
    /// reads `read_<col>(row)` for a `row` the body computes (`motion.kaleidoscope`
    /// reads source row `i % window_src_n` — the slice-major fan-out). This is the
    /// same length-decouple the [`Self::GatherKey`] state ports have, without an
    /// id: the mapping is arithmetic, exactly like `sim.spawn`'s `cp_rows`.
    ///
    /// Present iff the port carries the column at ANY non-empty length; the
    /// generated `read_<col>` is a plain buffer read (the body supplies the
    /// index), so a SourceRead adds nothing the kernel could not already write —
    /// it only tells the sequencer the port is length-decoupled. Reads, never
    /// writes: the node's OUTPUT column is a separate [`Self::Write`] binding on
    /// the same port (`kaleidoscope` reads the source `P` and writes a transformed
    /// output `P` — two bindings, because they live in different buffers).
    SourceRead,
    /// **ESCREVE só quando a coluna existe, numa porta TEMPLATE, e NUNCA lê** — o terceiro membro
    /// da família do [`StreamOp::SourceRows`](crate::gpu::StreamOp), e a variante que uma
    /// **medição** obrigou a existir no dia seguinte à [`Self::SourceReadWriteExisting`].
    ///
    /// ⛔⛔ **Ela não é simetria: um buffer LIDO POR NINGUÉM é um crash.** A `Count` do
    /// `motion.clone` vale `total` em toda a linha — *ela não depende do que entrou* —, logo o
    /// corpo escreve-a e nunca a lê. Ligada como a cruza, o módulo declarava `in_Count`, o corpo
    /// não chamava `read_Count`, **a naga apagava esse buffer do layout derivado** e o bind group
    /// do sequenciador ficava com uma entrada a mais — `create_bind_group` estoura, e só no tique
    /// em que a coluna nasce. Quem o apanhou foi o
    /// `every_registered_kernel_validates_across_the_whole_presence_space`, que varre as 2ⁿ
    /// máscaras de presença sem placa nenhuma.
    ///
    /// ⚠️ **As duas metades que ela partilha, e a que ela larga:**
    ///
    /// - **escreve SÓ SE PRESENTE**, como a [`Self::ReadWriteExisting`] e a cruza — escrever uma
    ///   coluna ausente CUNHA-A, e num kernel `SourceRows` o que o corpo não escreve chega por
    ///   GATHER;
    /// - **a porta é length-decoupled** ([`Self::is_source_mapped`]), como a cruza e o
    ///   [`Self::SourceRead`] — sem isso a presença seria julgada pela regra POSICIONAL (tem de
    ///   ter o comprimento do dispatch) e a coluna sairia ABSENT numa cadeia que a traz;
    /// - **não lê**, e é isso que a separa da cruza: nenhum acessor `read_`, nenhum buffer de
    ///   leitura, nenhuma entrada no bind group.
    SourceWriteExisting,
}

impl ColumnAccess {
    /// Does this access read the bound port's column?
    pub const fn reads(self) -> bool {
        matches!(
            self,
            ColumnAccess::Read
                | ColumnAccess::ReadBroadcast
                | ColumnAccess::ReadWrite
                | ColumnAccess::ReadWriteExisting
                | ColumnAccess::Consume
                // The gather key reads the current element's id.
                | ColumnAccess::GatherKey
                // A source-mapped read reads its template at the body's own index.
                | ColumnAccess::SourceRead
                | ColumnAccess::SourceReadWriteExisting
        )
    }

    /// Does this access write an output column (given whether the input
    /// stream carries it)?
    pub const fn writes(self, present_on_input: bool) -> bool {
        match self {
            ColumnAccess::Read
            | ColumnAccess::ReadBroadcast
            | ColumnAccess::Consume
            | ColumnAccess::RefuseIfPresent
            | ColumnAccess::GatherKey
            | ColumnAccess::SourceRead => false,
            ColumnAccess::Write | ColumnAccess::ReadWrite => true,
            ColumnAccess::ReadWriteExisting
            | ColumnAccess::SourceReadWriteExisting
            | ColumnAccess::SourceWriteExisting => present_on_input,
        }
    }

    /// Does the node's output stream **drop** this column (rather than let the
    /// base input's copy ride through)?
    pub const fn consumes(self) -> bool {
        matches!(self, ColumnAccess::Consume)
    }

    /// Does a length-1 port broadcast on this access ([`Self::ReadBroadcast`])?
    pub const fn broadcasts(self) -> bool {
        matches!(self, ColumnAccess::ReadBroadcast)
    }

    /// Is this binding a plan-time refusal rather than an access?
    pub const fn refuses(self) -> bool {
        matches!(self, ColumnAccess::RefuseIfPresent)
    }

    /// Is this binding the [`Self::GatherKey`] — a conditional refusal that
    /// claims when its port's stream is a dense id window (ADR-0130)?
    pub const fn is_gather_key(self) -> bool {
        matches!(self, ColumnAccess::GatherKey)
    }

    /// **A porta desta ligação é o TEMPLATE de um [`StreamOp::SourceRows`](crate::gpu::StreamOp)**
    /// — length-decoupled do dispatch, logo presente ao comprimento DELA e não ao da corrida.
    ///
    /// ⚠️⚠️ **O nome diz PORTA e não LEITURA de propósito, e a diferença já mordeu:** a pergunta
    /// que o `column_present` faz é *«com que comprimento julgo esta porta?»*, e ela tem três
    /// respostas afirmativas — a que só lê ([`Self::SourceRead`]), a que lê e escreve
    /// ([`Self::SourceReadWriteExisting`]) e a que **só escreve**
    /// ([`Self::SourceWriteExisting`]). Enquanto o predicado se chamou `is_source_read`, a
    /// terceira não cabia no nome sem o tornar falso.
    ///
    /// ⛔ Sem isto o `column_present` julgaria a porta template pela regra POSICIONAL (tem de ser
    /// o comprimento do dispatch) e a coluna sairia ABSENT numa cadeia que a traz — a lei ficaria
    /// escrita e nunca armaria.
    pub const fn is_source_mapped(self) -> bool {
        matches!(
            self,
            ColumnAccess::SourceRead
                | ColumnAccess::SourceReadWriteExisting
                | ColumnAccess::SourceWriteExisting
        )
    }
}

/// One stream column a kernel touches: its name (the stream convention —
/// `P`, `size`, `rot`, `tint`, `falloff`, …), its element type, how it is
/// accessed, which input port it is read from, and the per-element identity
/// used when a readable column is absent (only the first [`Dim`]-many lanes
/// are meaningful).
#[derive(Copy, Clone, Debug)]
pub struct ColumnBinding {
    pub column: &'static str,
    pub dim: Dim,
    pub access: ColumnAccess,
    /// The value an absent readable column reads as, per element. `P`/`rot`
    /// read `0`, `falloff` reads `1`, `size` reads unit scale (`SIZE_IDENTITY`)
    /// — the same identities the CPU paths use, so absence means the same
    /// thing on both sides.
    pub identity: [f32; 4],
    /// Which **input port** of the node the column is read from (writes always
    /// land on the node's single output, so this only qualifies the read).
    /// `0` for every single-input node — the overwhelming case, and the reason
    /// this is spelled on each binding rather than inferred: a multi-input
    /// kernel that got it wrong would read the right column off the wrong
    /// stream and answer plausibly.
    ///
    /// **Port 0 is the base:** the node's output stream starts as port 0's
    /// (columns the kernel never mentions ride through it, exactly like the CPU
    /// nodes that copy their primary input and rewrite a channel), then written
    /// columns replace and [`ColumnAccess::Consume`]d ones are dropped.
    /// `motion.integrate` is the shape that named this: `rest` (port 0) carries
    /// the live animation forward while `forces` (port 1) carries last tick's
    /// state — and `vel` is read from BOTH (the seed from `rest`, the step from
    /// `forces`), which is why the port cannot be a property of the column name.
    pub port: usize,
}

#[cfg(test)]
mod tests {
    use super::ColumnAccess;

    /// **O ÍNDICE de cada verbo** — e este `match` é EXAUSTIVO de propósito: uma variante nova
    /// é **erro de compilação aqui**, e não uma linha que cai num `_ =>` em silêncio.
    ///
    /// ⚠️ Ele é o que torna a [`TODAS`] derivada em vez de escrita à mão. *Uma lista de variantes
    /// mantida à mão é a lista que envelhece* — e as que já existem nos gates deste enum
    /// envelheceram exactamente assim (o `Consume`/`RefuseIfPresent` varrem seis nomes de nove).
    fn indice(a: ColumnAccess) -> usize {
        match a {
            ColumnAccess::Read => 0,
            ColumnAccess::Write => 1,
            ColumnAccess::ReadWrite => 2,
            ColumnAccess::ReadWriteExisting => 3,
            ColumnAccess::Consume => 4,
            ColumnAccess::RefuseIfPresent => 5,
            ColumnAccess::ReadBroadcast => 6,
            ColumnAccess::SourceReadWriteExisting => 7,
            ColumnAccess::GatherKey => 8,
            ColumnAccess::SourceRead => 9,
            ColumnAccess::SourceWriteExisting => 10,
        }
    }

    /// Todos os verbos, na ordem do [`indice`] — ver o gate que prende as duas coisas.
    const TODAS: [ColumnAccess; 11] = [
        ColumnAccess::Read,
        ColumnAccess::Write,
        ColumnAccess::ReadWrite,
        ColumnAccess::ReadWriteExisting,
        ColumnAccess::Consume,
        ColumnAccess::RefuseIfPresent,
        ColumnAccess::ReadBroadcast,
        ColumnAccess::SourceReadWriteExisting,
        ColumnAccess::GatherKey,
        ColumnAccess::SourceRead,
        ColumnAccess::SourceWriteExisting,
    ];

    /// ⭐⭐⭐ **A LISTA É DERIVADA, e uma variante nova não pode ficar de fora dela.**
    ///
    /// O [`indice`] é exaustivo ⇒ quem acrescentar um verbo é obrigado a dar-lhe um índice; este
    /// gate exige que esse índice caiba na [`TODAS`] **e** que ela esteja na ordem dele. ⇒ o par
    /// não pode divergir sem alguém reparar.
    #[test]
    fn toda_variante_esta_na_lista_e_na_ordem_do_indice() {
        for (i, a) in TODAS.into_iter().enumerate() {
            assert_eq!(indice(a), i, "{a:?} esta' fora de ordem na TODAS");
        }
    }

    /// ⭐⭐⭐ **O PERFIL DA CRUZA** (`SourceReadWriteExisting`, doc 116 §5.3) — e as duas metades
    /// que a definem, mais os quatro predicados que ela **não** é.
    ///
    /// ⚠️ **A 2.ª metade é a que o `ReadWriteExisting` também tem**, e a 1.ª é a que o
    /// `SourceRead` também tem; o que não existia era o PAR. *Sem a `is_source_read` a porta
    /// template seria julgada pela regra POSICIONAL e a coluna sairia absent numa cadeia que a
    /// traz — a lei ficaria escrita e nunca armaria.*
    #[test]
    fn a_cruza_le_na_fonte_e_escreve_so_o_que_existe() {
        let a = ColumnAccess::SourceReadWriteExisting;
        assert!(a.reads(), "ela LE");
        assert!(a.is_source_mapped(), "e le na FONTE (length-decoupled)");
        assert!(a.writes(true), "escreve quando a coluna existe");
        assert!(!a.writes(false), "e NAO a cunha quando ela falta");
        // ⛔ E não é nenhuma das outras coisas — sem isto, um predicado novo que a apanhasse por
        // acidente mudaria o que ela faz sem tocar nesta linha.
        assert!(!a.consumes() && !a.broadcasts() && !a.refuses() && !a.is_gather_key());
        // ⭐ O CONTROLO: o irmão que lê na fonte NUNCA escreve, e é por isso que ele não chegava.
        assert!(ColumnAccess::SourceRead.is_source_mapped());
        assert!(!ColumnAccess::SourceRead.writes(true));
        // …e o irmão que escreve condicionalmente NÃO é de porta template.
        assert!(ColumnAccess::ReadWriteExisting.writes(true));
        assert!(!ColumnAccess::ReadWriteExisting.is_source_mapped());
    }

    /// ⭐⭐⭐ **O PERFIL DA ESCRITA SEM LEITURA** (`SourceWriteExisting`) — a variante que um
    /// **buffer lido por ninguém** obrigou a existir.
    ///
    /// ⚠️ **A metade que a define é uma AUSÊNCIA** (`!reads()`), e uma ausência não se prova com
    /// o predicado sozinho: o CONTROLO é a cruza ao lado, que tem tudo isto **mais** a leitura.
    /// *Sem ele este gate passaria com um alias da cruza.*
    #[test]
    fn a_escrita_sem_leitura_e_de_porta_template_e_nao_cunha() {
        let a = ColumnAccess::SourceWriteExisting;
        assert!(
            !a.reads(),
            "ela NAO le -- e' isso que lhe tira o buffer de leitura"
        );
        assert!(a.is_source_mapped(), "mas a porta dela e' o TEMPLATE");
        assert!(a.writes(true), "escreve quando a coluna existe");
        assert!(!a.writes(false), "e NAO a cunha quando ela falta");
        assert!(!a.consumes() && !a.broadcasts() && !a.refuses() && !a.is_gather_key());
        // ⭐ O CONTROLO: a cruza é ela MAIS a leitura, e é a leitura que as separa.
        let cruza = ColumnAccess::SourceReadWriteExisting;
        assert!(cruza.reads(), "a cruza LE");
        assert_eq!(cruza.is_source_mapped(), a.is_source_mapped());
        assert_eq!(cruza.writes(true), a.writes(true));
        assert_eq!(cruza.writes(false), a.writes(false));
    }

    /// ⭐ **Exactamente DOIS verbos escrevem condicionalmente, e exactamente DOIS leem na fonte** —
    /// as populações derivadas da [`TODAS`], para que um verbo novo que caia numa delas por
    /// acidente reprove aqui em vez de mudar o produto em silêncio.
    #[test]
    fn as_duas_populacoes_condicionais_sao_as_que_a_casa_declara() {
        let condicionais: Vec<_> = TODAS
            .into_iter()
            .filter(|a| a.writes(true) && !a.writes(false))
            .collect();
        assert_eq!(
            condicionais,
            vec![
                ColumnAccess::ReadWriteExisting,
                ColumnAccess::SourceReadWriteExisting,
                ColumnAccess::SourceWriteExisting
            ],
            "quem escreve SO SE PRESENTE"
        );
        let na_fonte: Vec<_> = TODAS.into_iter().filter(|a| a.is_source_mapped()).collect();
        assert_eq!(
            na_fonte,
            vec![
                ColumnAccess::SourceReadWriteExisting,
                ColumnAccess::SourceRead,
                ColumnAccess::SourceWriteExisting
            ],
            "quem vive numa porta TEMPLATE (length-decoupled)"
        );
    }
}
