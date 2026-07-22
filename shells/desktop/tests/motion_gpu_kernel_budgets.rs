//! **Every registered GPU kernel fits the budgets the dispatch assumes** —
//! asked of the REAL registry (`register_all_nodes`), so a kernel added
//! tomorrow is covered the day it registers, with no hand list to rot
//! ([[feedback_a_condition_that_enumerates_its_readers_rots]] — the WGSL
//! validation sweep in `ph2d-gpu-cook` is hand-enumerated and was found
//! missing two kernels by the 2026-07-20 audit; this gate lives in the shell
//! precisely because the shell is the one crate that already links everything).
//!
//! Two budgets, both of which fail SILENTLY-or-late in production:
//!
//! 1. **The uniform slot** (`ph2d_gpu_cook::UNIFORM_BYTES`): the packer
//!    (`encode_kernel_stage`) writes `count + playhead + params + conditional
//!    engine fields` by offset arithmetic into a slice of exactly that size. A
//!    kernel with enough params to run off the end is a PANIC at first dispatch
//!    — at the artist's machine, not at `cargo test`. The field count is read
//!    from the GENERATED module text (the same text the pipeline compiles), not
//!    from a parallel formula that could drift from the codegen.
//!
//! 2. **Finite identities**: an absent readable column is generated as a WGSL
//!    literal from `ColumnBinding::identity` via `{v:?}` — `NaN`/`inf` would
//!    emit `NaN`/`inf` tokens, which are not WGSL literals, and the module
//!    would fail to PARSE at first dispatch on the machine of whoever first
//!    unplugs that column.
//!
//! Blind spot, named: a `variant_by_param` kernel's non-default variants are
//! resolved through an opaque `fn` and cannot be enumerated here (the WGSL
//! sweep has the same limit). The default variant is covered; the variants
//! share the node's param list, which is what the uniform budget keys on.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::KernelResolver;

#[test]
fn every_registered_kernel_fits_the_uniform_slot_and_declares_finite_identities() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");

    let mut swept = 0usize;
    for manifest in reg.manifests() {
        let Some(kernel) = reg.gpu_kernel(manifest.id) else {
            continue;
        };
        if kernel.is_passthrough() {
            continue;
        }
        swept += 1;

        // (2) Finite identities — the literal generator cannot spell NaN/inf.
        for b in kernel.bindings {
            assert!(
                b.identity.iter().all(|v| v.is_finite()),
                "{}: binding `{}` declares a non-finite identity {:?} — \
                 `identity_literal` would emit invalid WGSL for it",
                manifest.name,
                b.column,
                b.identity
            );
        }

        // (1) The uniform slot, measured off the generated module itself with
        // every conditional field forced IN (all columns present, the grid
        // attached when the node registered one) — the widest layout this
        // kernel can ever ask the packer to write.
        let port_names: Vec<&str> = manifest.inputs.iter().map(|p| p.name).collect();
        let src = ph2d_gpu_cook::codegen::kernel_module(
            kernel,
            kernel.bindings,
            &port_names,
            reg.grid(manifest.id),
            // The node's declared reductions — a deformer's module carries a
            // `reduce_*` storage binding per spec, so measuring the "widest
            // layout this kernel can ask for" without them would measure a
            // module the sequencer never builds.
            reg.reduces(manifest.id),
            |_| true,
        );
        let struct_body = src
            .split("struct KernelParams {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .unwrap_or_else(|| panic!("{}: no KernelParams struct generated", manifest.name));
        let fields = struct_body
            .lines()
            .filter(|l| l.contains(": f32,") || l.contains(": u32,"))
            .count() as u64;
        // A parse that found no fields would pass the budget vacuously — every
        // kernel's struct carries at least `count` + `playhead`.
        assert!(
            fields >= 2,
            "{}: parsed only {fields} KernelParams fields — the oracle went blind",
            manifest.name
        );
        let bytes = fields * 4;
        assert!(
            bytes <= ph2d_gpu_cook::UNIFORM_BYTES,
            "{}: worst-case uniform layout is {bytes} B ({fields} fields) — over \
             the {} B slot the packer writes into; growing UNIFORM_BYTES is the \
             fix, and it must happen BEFORE this kernel ships",
            manifest.name,
            ph2d_gpu_cook::UNIFORM_BYTES
        );
    }

    // Positive control ([[feedback_a_negative_search_needs_a_positive_control]]):
    // an iteration that silently matched nothing would pass both budgets
    // vacuously. The registry carries 30+ real kernels today; this is a floor,
    // not a pin.
    assert!(
        swept >= 30,
        "swept only {swept} kernels — the loop went blind"
    );
}
