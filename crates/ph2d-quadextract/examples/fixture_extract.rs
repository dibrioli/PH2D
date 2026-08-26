//! ⭐⭐⭐ **A EXTRACÇÃO DE UM MAPA DE REFERÊNCIA** — o experimento que parte a cadeia ao meio.
//!
//! ```text
//! cargo run --release -p ph2d-quadextract --example fixture_extract -- <fixture.mapa.gz> <saida.obj>
//! ```
//!
//! ⛔⛔ **Por que ele existe.** Em 2026-08-26 mediu-se que as nossas linhas de grade
//! **espiralam** (`2,8×`–`4,2×` voltas por anel contra `0,8×`–`1,0×` do oráculo) e que a
//! **densidade não cura** — logo o defeito é estrutural. Faltava saber **de que lado** dele:
//!
//! | se a extracção de um mapa de REFERÊNCIA… | então o defeito está… |
//! |---|---|
//! | …fecha os anéis | ⇒ **no nosso MAPA** (G3/G5) |
//! | …espirala na mesma | ⇒ **na EXTRACÇÃO** |
//!
//! ⭐ Os fixtures de `docs/3D/cleanroom/fixtures/` são mapas de grade inteira **verificados a
//! `3,55e-15`** sobre a **nossa** malha e o **nosso** campo. ⇒ *mesma entrada, mesmo campo,
//! mapa diferente* — o controlo que isola a fase.

#[path = "../tests/support/mod.rs"]
mod support;

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args
        .next()
        .unwrap_or_else(|| panic!("uso: fixture_extract <fixture.mapa.gz> <saida.obj>"));
    let out_path = args
        .next()
        .unwrap_or_else(|| String::from("/tmp/fixture.obj"));

    let m = support::load(&name);
    let (out, e) = ph2d_quadextract::extract(&m.as_map(), None)
        .unwrap_or_else(|err| panic!("{name}: a extraccao recusou: {err}"));
    println!(
        "{name}: {} quads, {} nao-quads · orfas {} · χ = {}",
        e.quads,
        out.face_count() - e.quads,
        e.orphan,
        ph2d_quadextract::euler_characteristic(&out)
    );
    let text = ph2d_mesh::write_obj(&[ph2d_mesh::ExportPiece {
        mesh: &out,
        name: Some("Piece"),
        pose: ph2d_mesh::Pose::default(),
    }]);
    std::fs::write(&out_path, text).unwrap_or_else(|err| panic!("{out_path}: {err}"));
    println!("  (gravado em {out_path})");
}
