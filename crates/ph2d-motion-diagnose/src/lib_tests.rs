//! Os gates do diagnosticador. Irmão por `#[path]`, então segue sendo módulo
//! FILHO — o `use super::*` alcança os privados (o `param_reader` entre eles).
use super::*;
use ph2d_nodegraph::gpu::KernelResolver;


    /// **O leitor de params cai no DEFAULT DO MANIFESTO, não no zero.**
    ///
    /// ⚠️ Este gate mora aqui, e não ao lado do consumidor real, porque **nenhum
    /// nó do repo hoje consegue distinguir as duas respostas**: o único
    /// `ProducesWhen` que existe é o do `motion.make_point`, cujo `target` tem
    /// default `0` — e um leitor sem fallback também lê `0`. Uma mutação que apaga
    /// o degrau do manifesto sobrevive à suíte inteira por **coincidência de
    /// número**, não por estar certa.
    ///
    /// Aqui os dois divergem por construção (`5.0` contra `0.0`), que é a única
    /// forma de a fixture conter o fenômeno. Sem ele, o dia em que um nó com este
    /// canal nascer com o modo produtor num default não-zero é o dia em que toda
    /// instância dele é marcada — em silêncio, com tudo verde.
    #[test]
    fn an_untouched_param_falls_back_to_the_manifest_default_not_to_zero() {
        static FIVE: ph2d_nodegraph::node::NodeManifest = ph2d_nodegraph::node::NodeManifest {
            id: NodeTypeId::of("diagnose.test.five"),
            name: "diagnose.test.five",
            inputs: &[],
            outputs: &[],
            effect: ph2d_nodegraph::effect::Effect::Pure,
            clock: ph2d_nodegraph::port::Clock::Frame,
            params: &[ph2d_nodegraph::node::ParamSpec {
                name: "mode",
                default: 5.0,
            }],
            lowerings: &[ph2d_nodegraph::node::LoweringKind::Cpu],
        };
        struct Five;
        impl ph2d_nodegraph::node::NodeOp for Five {
            fn manifest(&self) -> &'static ph2d_nodegraph::node::NodeManifest {
                &FIVE
            }
            fn eval(&self, _ctx: &mut ph2d_nodegraph::cook::EvalCtx<'_>) {}
        }
        let mut reg = NodeRegistry::new();
        reg.register(Box::new(Five)).expect("register");

        let mut g = Graph::new();
        let n = g.add_node("diagnose.test.five");
        let ty = NodeTypeId::of("diagnose.test.five");

        // Sem `set_param`: o degrau do manifesto é o único que pode responder.
        let read = param_reader(&g, &reg, n, ty);
        assert!(
            (read("mode") - 5.0).abs() < f32::EPSILON,
            "um param intocado vale o default do manifesto, deu {}",
            read("mode")
        );
        // Um param que o manifesto não declara não tem degrau nenhum: zero.
        assert!(
            read("nao_existe").abs() < f32::EPSILON,
            "e um param inexistente e zero, nao um panico"
        );
        // E o valor AUTORADO vence o default, que é o degrau de cima.
        let mut g2 = Graph::new();
        let n2 = g2.add_node("diagnose.test.five");
        g2.set_param(n2, "mode", 1.0);
        let read2 = param_reader(&g2, &reg, n2, ty);
        assert!(
            (read2("mode") - 1.0).abs() < f32::EPSILON,
            "o valor autorado vence o default"
        );
    }

    /// **The transient set covers every column any node `Consume`-drops.** `accel`
    /// and `inv_mass` are derived facts — some node drops each — so this gate makes
    /// it impossible to introduce a new transient column (a new `Consume` binding)
    /// without landing it in [`TRANSIENT_COLUMNS`], where the diagnoser can reason
    /// about a producer of it. `falloff`, the modulation weight that is never
    /// dropped, is the one named directly, so the gate pins it too. FALSIFIED by
    /// removing "accel"/"inv_mass" (a `Consume` column escapes the analysis) or
    /// "falloff" (the modulation column stops being analysed).
    #[test]
    fn the_transient_set_covers_every_consumed_column() {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("register all nodes");
        for m in reg.manifests() {
            let Some(k) = reg.gpu_kernel(m.id) else {
                continue;
            };
            for b in k.bindings {
                assert!(
                    !b.access.consumes() || TRANSIENT_COLUMNS.contains(&b.column),
                    "a `Consume` binding drops `{}`, which is not in TRANSIENT_COLUMNS — \
                     the diagnoser would never analyse a producer of it",
                    b.column
                );
            }
        }
        assert!(
            TRANSIENT_COLUMNS.contains(&"falloff"),
            "falloff is the modulation weight the diagnoser must reason about"
        );
    }

    /// **The required-upstream set is disjoint from the transient set, and every column
    /// in it is really READ by some registered node.** The two analyses are mirror
    /// images — a producer-inert transient vs a read-required stream — and a column in
    /// both would have them fight over the same node. The second half pins that
    /// `REQUIRED_UPSTREAM` names a real read column (not a typo `"P"` nothing reads),
    /// which is what makes [`missing_upstream`] able to fire at all. FALSIFIED by adding
    /// a transient column to `REQUIRED_UPSTREAM`, or by naming a column no node reads.
    #[test]
    fn the_required_set_is_disjoint_and_really_read() {
        for &c in REQUIRED_UPSTREAM {
            assert!(
                !TRANSIENT_COLUMNS.contains(&c),
                "`{c}` is both required-upstream and transient — the two analyses would fight"
            );
        }
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("register all nodes");
        for &col in REQUIRED_UPSTREAM {
            assert!(
                reg.manifests().any(|m| reads_column(&reg, m.id, col)),
                "no registered node reads `{col}` — REQUIRED_UPSTREAM names a column nothing needs"
            );
        }
    }

    /// **A stateful source that seeds its own state is NOT source-less** — the false
    /// positive the diagnoser shipped on the flock. `motion.boids` READS `P` (a
    /// `ReadWrite` binding on its `state` port), but the `P` it reads is its OWN previous
    /// frame, arriving through the `pre` self-loop (`out --pre--> state`) the editor
    /// auto-plumbs, and it MINTS the initial cloud itself. It has no non-delayed input by
    /// design (the flock has no upstream source), so the naive "reads `P` + no incoming
    /// edge = source-less" rule would flag a graph that WORKS. [`seeds_own_state`] (the
    /// delayed self-loop, a signal a deformer never carries) exempts it. FALSIFIED two
    /// ways: removing the exemption makes the wired boids get a spurious
    /// [`Deficit::MissingSource`]; an over-broad exemption that always fires stops the
    /// positive control (a bare boids, no self-loop) from being flagged.
    #[test]
    fn a_stateful_source_that_seeds_its_own_state_is_not_source_less() {
        use ph2d_nodegraph::graph::Edge;
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("register all nodes");

        // The boids WITH its `pre` self-loop (exactly what the editor builds): the flock
        // seeds and reads its own state, so it is NOT source-less — zero diagnostics.
        let mut g = Graph::new();
        let boids = g.add_node("motion.boids");
        g.connect(Edge {
            from: (boids, 0),
            to: (boids, 2),
            delayed: true,
        })
        .expect("boids pre self-loop");
        let out = g.add_node("motion.output");
        g.connect(Edge {
            from: (boids, 0),
            to: (out, 0),
            delayed: false,
        })
        .expect("boids -> output");
        let diags = diagnose(&g, &reg);
        assert!(
            diags.is_empty(),
            "a self-seeding simulation source must not be flagged source-less: {diags:?}"
        );

        // Positive control: the SAME node WITHOUT its self-loop IS source-less — the
        // exemption is gated on the `pre` self-loop, not on the node type (an over-broad
        // exemption that always fired would silence this too).
        let mut bare = Graph::new();
        let b = bare.add_node("motion.boids");
        let o = bare.add_node("motion.output");
        bare.connect(Edge {
            from: (b, 0),
            to: (o, 0),
            delayed: false,
        })
        .expect("boids -> output");
        let d2 = diagnose(&bare, &reg);
        assert!(
            d2.iter().any(|d| d.deficit == Deficit::MissingSource("P")),
            "a P-reader with no self-loop and no input is genuinely source-less: {d2:?}"
        );
    }

    /// **The appropriate flock-stamp graph is clean** — the exact scene of the fix
    /// (`PH2D_AUTOFIX_SMOKE=7`): `source.shape (Star) -> duplicator.shape`, `boids ->
    /// duplicator.points` (with its `pre` self-loop), `duplicator -> oscillator ->
    /// output`. Headless proof (the smoke needs a window) that NO node warns — the
    /// stateful `boids` is exempt, and `source.shape` / `duplicator` / `oscillator`
    /// carry no spurious deficit either. FALSIFIED by the same `seeds_own_state`
    /// mutation (the boids gets a spurious `MissingSource`), so the scene the artist
    /// runs stays pinned green.
    #[test]
    fn the_appropriate_flock_stamp_graph_is_clean() {
        use ph2d_nodegraph::graph::Edge;
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("register all nodes");
        let mut g = Graph::new();
        let shape = g.add_node("source.shape");
        let boids = g.add_node("motion.boids");
        let dup = g.add_node("motion.duplicator");
        let osc = g.add_node("motion.oscillator");
        let out = g.add_node("motion.output");
        for (from, to, delayed) in [
            ((boids, 0), (boids, 2), true), // the `pre` self-loop
            ((shape, 0), (dup, 0), false),  // shape -> duplicator.shape
            ((boids, 0), (dup, 1), false),  // boids -> duplicator.points
            ((dup, 0), (osc, 0), false),
            ((osc, 0), (out, 0), false),
        ] {
            g.connect(Edge { from, to, delayed }).expect("connect");
        }
        let diags = diagnose(&g, &reg);
        assert!(
            diags.is_empty(),
            "the appropriate flock-stamp graph must warn nowhere: {diags:?}"
        );
    }
