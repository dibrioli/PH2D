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
                reg.grid(manifest.id),
                // The node's declared reductions, asked of the REGISTRY — not
                // `&[]`. A deformer's body calls `reduce_<name>()`, so passing an
                // empty list here would validate a module the sequencer never
                // builds and miss a misspelled reduction entirely.
                reg.reduces(manifest.id),
                // The node's LUTs, asked of the REGISTRY for the same reason as the
                // reductions: the body samples `<name>_sample(t)`, so an empty list
                // here would validate a module without that accessor and miss a
                // misdeclared LUT (A1-gpu).
                reg.luts(manifest.id),
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
                None,
                &[],
                &[], // a predicate samples no LUT (A1-gpu)
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
    let mut reduces = 0usize;
    for manifest in reg.manifests() {
        for spec in reg.reduces(manifest.id) {
            for present in [true, false] {
                let src = ph2d_gpu_cook::reduce_stage::map_module(spec, present);
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
fn the_lowering_validates_for_all_128_column_subsets_and_every_blend() {
    // SETE colunas (`blend` juntou-se ao `texture_id`, doc 89 folha 07), então 128
    // subconjuntos — o bit 6 é a coluna `blend`. O tag do sink (folha 17) continua a ser uma
    // CONSTANTE DE CODEGEN, então ele é parte da fonte que o naga tem de aceitar: um tag que
    // produzisse WGSL malformado só apareceria na primeira vez que um artista escolhesse
    // aquele modo, num device, sem mensagem nenhuma.
    //
    // ⚠️ **O produto cartesiano é de propósito.** A palavra do `flip_uv` passou a ser um
    // `if` sobre a coluna E a constante do sink; as duas metades só se encontram em
    // `present[6] && blend > 0`, que é exactamente uma casa deste laço.
    for mask in 0u8..128 {
        let present = std::array::from_fn(|i| mask & (1 << i) != 0);
        for blend in 0..ph2d_render::pipeline::BLEND_PIPELINE_COUNT as u8 {
            let src = ph2d_gpu_cook::lower::lower_module(present, blend);
            validate(&format!("lowering mask {mask:06b} blend {blend}"), &src);
        }
    }
}
