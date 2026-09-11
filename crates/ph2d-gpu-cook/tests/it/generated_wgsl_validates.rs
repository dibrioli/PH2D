//! Every WGSL module this crate can generate must parse + validate under naga
//! at `cargo test` time — no device needed, so a kernel typo is caught on any
//! CI lane, not at first dispatch on somebody's GPU (mirrors the render
//! crate's `sprite_wgsl_valid` gate).
//!
//! Coverage is **exhaustive over the presence space**: each registered F1.1
//! kernel × every subset of its readable columns, and the lowering × all 32
//! column subsets — because absence changes the generated text (identity
//! functions, dropped writes), and the absent variants are exactly the ones a
//! smoke test with a full stream never compiles.

use ph2d_render::SinkStyle;

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::KernelResolver;

fn validate(label: &str, src: &str) {
    let module = naga::front::wgsl::parse_str(src).unwrap_or_else(|e| {
        panic!(
            "{label}: generated WGSL failed naga parse:\n{}\n--- module ---\n{src}",
            e.emit_to_string(src)
        )
    });
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator.validate(&module).unwrap_or_else(|e| {
        panic!("{label}: naga validation failed: {e:?}\n--- module ---\n{src}")
    });
}

#[test]
fn every_registered_kernel_validates_across_the_whole_presence_space() {
    // ⚠️ **O registry é DERIVADO, nunca enumerado — e a lista à mão já apodreceu.**
    // Isto foi 49 chamadas `::register` escritas uma a uma, e a auditoria mediu o
    // preço: de todas as crates-nó com `gpu.rs`, EXATAMENTE UMA estava fora — a
    // mais nova (`motion.proximity`), cujo kernel só encontrava um compilador na
    // máquina de quem tem adapter, porque a paridade dele é `#[ignore]`. *Um
    // vermelho que só o device vê é invisível em toda lane sem placa.* Com o
    // `register_all_nodes` o kernel que nascer amanhã entra sozinho, que é
    // precisamente o que uma enumeração não faz.
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();

    let mut validated = 0usize;
    for manifest in reg.manifests() {
        let Some(kernel) = reg.gpu_kernel(manifest.id) else {
            continue;
        };
        if kernel.is_passthrough() {
            continue;
        }
        // The readers of a multi-input kernel are named by port, exactly as the
        // sequencer names them (`encode_kernel_stage`).
        let port_names: Vec<&str> = manifest.inputs.iter().map(|p| p.name).collect();
        let n = kernel.bindings.len().min(16);
        for mask in 0u32..(1 << n) {
            let src = ph2d_gpu_cook::codegen::kernel_module(
                kernel,
                kernel.bindings,
                &port_names,
                ph2d_gpu_cook::codegen::ExtraBuffers {
                    grid: reg.grid(manifest.id),
                    // The node's declared reductions, asked of the REGISTRY — not
                    // `&[]`. A deformer's body calls `reduce_<name>()`, so passing an
                    // empty list here would validate a module the sequencer never
                    // builds and miss a misspelled reduction entirely.
                    reduces: reg.reduces(manifest.id),
                    // The node's LUTs, asked of the REGISTRY for the same reason as the
                    // reductions: the body samples `<name>_sample(t)`, so an empty list
                    // here would validate a module without that accessor and miss a
                    // misdeclared LUT (A1-gpu).
                    luts: reg.luts(manifest.id),
                },
                // O canal PARTILHADO, pedido ao REGISTRY pela mesma razão das duas
                // acima: o `wgsl_lib` de um nó pode chamar o que mora aqui, e um `""`
                // validaria um módulo que o sequenciador nunca constrói.
                reg.wgsl_shared(manifest.id),
                |b| {
                    let idx = kernel
                        .bindings
                        .iter()
                        .position(|x| std::ptr::eq(x, b))
                        .expect("binding belongs to kernel");
                    mask & (1 << idx) != 0
                },
            );
            validate(&format!("{} mask {mask:b}", manifest.name), &src);
            validated += 1;
        }
    }

    // ⛔⛔ **E as VARIANTES, que é onde a lei se copia** (ciclo 3, W1 — doc 106).
    //
    // Este laço lia `reg.gpu_kernel(id)` — o kernel BASE — e um kernel com
    // `variant_by_param` nunca é dispachado: o sequenciador chama
    // `GpuKernel::resolve(param)` e dispacha o que vier de lá. Dez nós declaram
    // variantes, e **nenhuma delas alguma vez encontrou um compilador nesta
    // lane**. Medido: a `DRIVE_HSV` do `motion.drive` chama `drive_resolve` e a
    // `wgsl_lib` dela — uma CÓPIA da do irmão, com dois terços das funções —
    // **não a define**; conduzir matiz, saturação ou valor no dispositivo falha
    // a compilar o módulo, e o único sítio que o dizia era um gate de paridade
    // `#[ignore]` que precisa de adapter.
    //
    // ⚠️ **A varredura é de UM param de cada vez a partir dos defaults**, e não do
    // produto cartesiano: `variant_by_param` é, pelo nome e pela prática, uma
    // função de um param só (o `channel` do `drive`, o `space` do `move`), e o
    // produto de 24 params seria uma explosão para cobrir uma forma que ninguém
    // escreve. Uma variante escolhida por uma COMBINAÇÃO escapa a isto — está
    // nomeado aqui em vez de prometido.
    let mut variants = 0usize;
    for manifest in reg.manifests() {
        let Some(base) = reg.gpu_kernel(manifest.id) else {
            continue;
        };
        if base.variant_by_param.is_none() {
            continue;
        }
        let hints = reg.param_ui(manifest.id).unwrap_or(&[]);
        let port_names: Vec<&str> = manifest.inputs.iter().map(|p| p.name).collect();
        let mut seen: Vec<*const ph2d_nodegraph::gpu::GpuKernel> = vec![std::ptr::from_ref(base)];
        for spec in manifest.params {
            // Os valores que este param pode tomar: os índices de um `Enum` são
            // exactos; um slider dá os extremos e o meio, que é o que separa um
            // ramo por limiar.
            let hint = hints.iter().find(|h| h.param == spec.name);
            let values: Vec<f32> = match hint.map(|h| h.widget) {
                Some(ph2d_node_registry::ParamWidget::Enum { labels }) => (0..labels.len())
                    .map(|i| {
                        #[expect(clippy::cast_precision_loss, reason = "um indice de enum")]
                        let v = i as f32;
                        v
                    })
                    .collect(),
                _ => hint.map_or_else(
                    || vec![spec.default],
                    |h| vec![h.min, (h.min + h.max) * 0.5, h.max],
                ),
            };
            for v in values {
                let resolve = |name: &str| {
                    if name == spec.name {
                        v
                    } else {
                        manifest
                            .params
                            .iter()
                            .find(|p| p.name == name)
                            .map_or(0.0, |p| p.default)
                    }
                };
                let k = base.resolve(&resolve);
                let ptr = std::ptr::from_ref(k);
                if seen.contains(&ptr) || k.is_passthrough() {
                    continue;
                }
                seen.push(ptr);
                let n = k.bindings.len().min(16);
                for mask in 0u32..(1 << n) {
                    let src = ph2d_gpu_cook::codegen::kernel_module(
                        k,
                        k.bindings,
                        &port_names,
                        ph2d_gpu_cook::codegen::ExtraBuffers {
                            grid: reg.grid(manifest.id),
                            reduces: reg.reduces(manifest.id),
                            luts: reg.luts(manifest.id),
                        },
                        reg.wgsl_shared(manifest.id),
                        |b| {
                            let idx = k
                                .bindings
                                .iter()
                                .position(|x| std::ptr::eq(x, b))
                                .expect("binding belongs to variant");
                            mask & (1 << idx) != 0
                        },
                    );
                    validate(
                        &format!(
                            "{} variant [{} = {v}] mask {mask:b}",
                            manifest.name, spec.name
                        ),
                        &src,
                    );
                    validated += 1;
                }
                variants += 1;
            }
        }
    }
    // Controlo positivo: dez nós declaram `variant_by_param`, e o mais rico
    // (`motion.drive`) sozinho tem seis variantes distintas do base. Um piso, não
    // um pino — uma varredura que casasse zero passaria vaziamente.
    assert!(
        variants >= 15,
        "so' {variants} variantes distintas foram varridas — o laco foi as cegas"
    );
    // A compact node's REAL WGSL is its predicate (ADR-0136) — a kernel like
    // any other, dispatched by `encode_kernel_stage`, and therefore due exactly
    // this sweep: the cull predicate shipped with unqualified accessor names
    // (`read_v` on a two-port node) and only THIS class of gate can catch that
    // without an adapter.
    let mut predicates = 0usize;
    for manifest in reg.manifests() {
        let Some(ph2d_nodegraph::gpu::StreamOp::Compact { predicate, .. }) =
            reg.stream_op(manifest.id)
        else {
            continue;
        };
        let port_names: Vec<&str> = manifest.inputs.iter().map(|p| p.name).collect();
        let n = predicate.bindings.len().min(16);
        for mask in 0u32..(1 << n) {
            let src = ph2d_gpu_cook::codegen::kernel_module(
                predicate,
                predicate.bindings,
                &port_names,
                // Um predicado não liga grelha, redução nem LUT (A1-gpu).
                ph2d_gpu_cook::codegen::ExtraBuffers::NONE,
                "", // nem o canal PARTILHADO — o `stream_op` passa o mesmo `""`
                |b| {
                    let idx = predicate
                        .bindings
                        .iter()
                        .position(|x| std::ptr::eq(x, b))
                        .expect("binding belongs to predicate");
                    mask & (1 << idx) != 0
                },
            );
            validate(&format!("{} predicate mask {mask:b}", manifest.name), &src);
            predicates += 1;
        }
    }
    assert!(
        predicates >= 8,
        "the compact predicates must be swept (cull 3 bindings + lifetime 3), got {predicates}"
    );

    // A reduction's `value` is a WGSL EXPRESSION the node author writes by hand,
    // pasted into a module of its own (`fn reduce_value(v) -> f32`) that the
    // kernel sweep above never builds. Until this loop existed those expressions
    // only ever met a compiler on a machine with an adapter — and the first one
    // that is not a bare field access (`motion.collide`'s `rmax`, a `select` with
    // a two-term `&&`) is exactly the shape a typo hides in.
    //
    // BOTH forms, because they are different source: `present = true` reads
    // `src[i]`, `present = false` folds the spec's identity literal, and a
    // mis-declared `dim`/`identity` pair only fails to compile in the second.
    //
    // ⚠️ **E com as ANTERIORES ligadas, na posição em que o nó as declara** — uma redução pode
    // ler as que vêm antes dela (`reduce_<nome>()`), e essa é a forma em que o sequenciador de
    // facto constrói o módulo. Passar `&[]` aqui validaria um módulo que ele nunca gera e
    // deixaria passar um nome de antecessora mal escrito.
    let mut reduces = 0usize;
    for manifest in reg.manifests() {
        let specs = reg.reduces(manifest.id);
        for (si, spec) in specs.iter().enumerate() {
            let earlier: Vec<&_> = specs[..si].iter().collect();
            for present in [true, false] {
                let src = ph2d_gpu_cook::reduce_stage::map_module(
                    spec,
                    present,
                    &earlier,
                    reg.wgsl_shared(manifest.id),
                );
                validate(
                    &format!("{} reduce {} present={present}", manifest.name, spec.name),
                    &src,
                );
                reduces += 1;
            }
        }
    }
    assert!(
        reduces >= 16,
        "the declared reductions must be swept (bend 2 + twist 2 + spherize 2 + \
         four_point_warp 4 + collide 1, × 2 forms), got {reduces}"
    );

    // Grid (3 bindings → 8) + oscillator (2 → 4) + move (2 → 4) + the Fase 2
    // deformers transform/rotate/scale (2 → 4 each). If a kernel is added or
    // gains a binding this grows — the assert is a floor, not a pin.
    assert!(validated >= 16, "validated only {validated} variants");
}

#[test]
fn the_lowering_validates_for_all_256_column_subsets_and_every_style() {
    // OITO colunas (`uv_cell` juntou-se ao `blend`, doc 89 folha 17), então 256
    // subconjuntos — o bit 6 é a coluna `blend` e o 7 é a `uv_cell`. O ESTILO do sink
    // (folha 17) continua a ser uma CONSTANTE DE CODEGEN, então ele é parte da fonte que o
    // naga tem de aceitar: um estilo que produzisse WGSL malformado só apareceria na
    // primeira vez que um artista escolhesse aquele valor, num device, sem mensagem nenhuma.
    //
    // ⚠️ **O produto cartesiano é de propósito.** A palavra do `flip_uv` é um `if` sobre a
    // coluna E a constante do sink; as duas metades só se encontram em
    // `present[6] && blend > 0`, que é exactamente uma casa deste laço.
    //
    // ⚠️ **E o PIVÔ entra com um valor NEGATIVO e um positivo**: ele é soletrado como
    // literal `f32` na fonte, e `-0.25` sem parênteses num sítio errado é precisamente o
    // tipo de WGSL que compila na cabeça de quem escreve e não no naga.
    let styles = [
        SinkStyle::PLAIN,
        SinkStyle {
            pivot: [0.5, -0.25],
            sampling: ph2d_render::RenderInstance::pack_sampling(1, 0),
            stream_order: true,
            ..SinkStyle::PLAIN
        },
    ];
    for mask in 0u16..256 {
        let present = std::array::from_fn(|i| mask & (1 << i) != 0);
        for blend in 0..ph2d_render::pipeline::BLEND_PIPELINE_COUNT as u8 {
            for base in styles {
                let style = SinkStyle { blend, ..base };
                let src = ph2d_gpu_cook::lower::lower_module(present, style);
                validate(&format!("lowering mask {mask:08b} style {style:?}"), &src);
            }
        }
    }
}
