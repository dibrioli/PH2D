---
name: project-imageio-avif-pathc-2026-05-28
description: AVIF (W3.T4) re-shipped via Path C libavif-sys; vendored *-sys crates need meson/nasm/cmake host tools
metadata: 
  node_type: memory
  type: project
  originSessionId: 8ce6e619-52d4-4b52-b842-18f37c4540b2
---

W3.T4 AVIF re-shipped 2026-05-28 (commits `6bd4620` code + `13c226a` docs, local) via **Path C**: `libavif-sys` (codec-dav1d decode + codec-rav1e pure-Rust encode). Real SDR + HDR/wide-gamut decode+encode; `nclx`+ICC→ColorProfile, PQ/HLG/linear EOTF. Replaces the deshipped `avif-decode` (§5.17). Verification: 0 RUSTSEC, zero `owning_ref`. This is the **only imageio format crate without `#![forbid(unsafe_code)]`** (FFI) — uses `deny(unsafe_op_in_unsafe_fn)` + RAII guards + catch_unwind. ADR-0054 §5.18.

**Non-obvious gotcha (corrects a handoff claim):** the vendored `*-sys` crates ship C *source*, not prebuilt binaries — `libdav1d-sys` needs **meson+ninja**, rav1e x86 asm needs **nasm**. "Vendored → no CI install" is FALSE. `spike.yml` gained `Install AVIF build tools` on lint/MSRV (apt) + 3-OS matrix (apt/brew/choco). **Windows CI build is the unverified-locally risk** — babysit first push.

**audit-16 (5-lens, 2026-05-28):** remediation commits `b1c44d7` (code) + `91e0a00` (docs). Lens A (FFI safety) verdict SOUND — no UAF/leak/double-free, audit-15 RUSTSEC class genuinely gone. Fixed inline: 3 HIGH (total-pixel bomb via sub-cap axes → `imageSizeLimit`+guard; `is_avif_magic` ignored compatible_brands → scan like libavif; false "is logged" doc claim), 2 MED (HLG on different linear scale than PQ → ×12 diffuse-white unify; encode had no dim cap), several LOW. 25 tests green.

**2 open follow-ups for Coord (ph2d-asset, NOT avif crate's scope — per audit-scope-discipline):**
1. `loader.rs:112` `is_supported_image_extension` omits `avif`/`jxl`/`exr`/`hdr`/`svg` — drag-drop UX filter rejects them.
2. `loader.rs:144` `decode_image_bytes` calls `image::guess_format` FIRST and errors before the registry fallback. `image` only recognizes `ftypavif` (still), so **animation `avis` + `mif1`-major/compatible-brand AVIFs are blocked before reaching AvifImporter**. Still-AVIF works (guess_format→Avif→other→registry). Fix = try registry before erroring, or route by extension.

Related: [[feedback-no-industrial-claims-without-verification]], [[feedback-audit-scope-discipline]].
