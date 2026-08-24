//! ⭐⭐ **GATE 8 — a FORMA POR-FACE da saída**, e ⛔ **por que a barra do oráculo não
//! é medível sobre estas duas peças.**
//!
//! ⛔⛔ **FASE ZERO: a entrada tem de ser uma malha de triângulos BEM FORMADOS.**
//! Medido em 2026-08-24, em duas peças, com tudo o resto igual — mesma superfície,
//! mesmo campo, mesma extracção, mesma densidade:
//!
//! | triangulação de entrada | enviesamento p50 | faces com canto `>60°` | quads |
//! |---|---|---|---|
//! | ⛔ leque sobre uma malha de quadriláteros | `10,4°` · `12,5°` | `7` · `7` | `99,9 %` |
//! | ⭐ remalhada isotropicamente | `5,1°` · `5,5°` | `0` · `3` | `100 %` |
//!
//! ⇒ *o dobro do enviesamento, sem uma linha de algoritmo mudar.*
//!
//! ⚠️⚠️ **E os dois mapas de referência NÃO estão remalhados** — este teste mede-o
//! antes de medir qualquer outra coisa: o aspecto dos triângulos de entrada dá
//! `p50 1,78 / p99 2,17` no toro e `p50 1,65 / p99 11,56 / máx 21,2` no gancho,
//! contra a assinatura do nosso F1 (`1,16 / 1,58 / 3,1`). O gancho é **exactamente**
//! a assinatura do leque (`1,65 / 22,97`).
//!
//! ⇒ **A barra do oráculo (`4,8°`–`7,1°` de enviesamento p50) não é medível aqui**, e
//! baixá-la para o que estas peças dão seria calibrar a régua pelo defeito. O que
//! este teste cobra é a banda **documentada para esta classe de entrada** — e nomeia
//! a medição que falta. *Uma cura medida numa fixtura que não contém o fenómeno
//! lê-se como inútil; uma barra medida nela lê-se como cumprida.*
//!
//! ⏳ **A medição que falta, e o que a destrava:** correr a cadeia inteira da casa —
//! `ph2d-remesh-iso` (a fase zero) → `ph2d-crossfield` → `ph2d-gridmap` (G1–G4) →
//! **o arredondamento inteiro** (`ph2d_gridmap::round`) → esta extracção. Só o último
//! elo faltava, e é por isso que ele faz parte desta linha e não de outra.

mod support;

use ph2d_quadextract::extract;
use ph2d_quadextract::mapa::Mapa;

/// O aspecto dos triângulos de ENTRADA, em percentis — a assinatura da fase zero.
fn input_aspect(m: &Mapa) -> (f64, f64, f64) {
    let d = |a: [f32; 3], b: [f32; 3]| {
        let v = [
            f64::from(a[0] - b[0]),
            f64::from(a[1] - b[1]),
            f64::from(a[2] - b[2]),
        ];
        v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2])).sqrt()
    };
    let mut a: Vec<f64> = m
        .tris
        .iter()
        .map(|t| {
            let p = [
                m.pos[t[0] as usize],
                m.pos[t[1] as usize],
                m.pos[t[2] as usize],
            ];
            let e = [d(p[0], p[1]), d(p[1], p[2]), d(p[2], p[0])];
            let lo = e.iter().copied().fold(f64::MAX, f64::min).max(1.0e-30);
            e.iter().copied().fold(0.0, f64::max) / lo
        })
        .collect();
    a.sort_by(f64::total_cmp);
    let at = |q: f64| {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let i = ((a.len() - 1) as f64 * q).round() as usize;
        a[i]
    };
    (at(0.5), at(0.99), *a.last().unwrap())
}

#[test]
fn a_forma_por_face_bate_a_banda_da_classe_de_entrada_que_os_fixtures_tem() {
    // ⚠️ **As barras são RATCHETS medidos em 2026-08-24 nesta classe de entrada**, e
    // não políticas: elas só descem. O `over_4` do oráculo é **zero** em toda peça
    // orgânica — mas medido sobre entrada remalhada, que não é o que estas peças são.
    for (name, m, skew_band, worst_over_4) in [
        ("toro", support::torus(), (8.0, 12.0), 0usize),
        ("gancho", support::hooked(), (9.0, 15.0), 1usize),
    ] {
        // ── PRIMEIRO: a entrada honra a fase zero? Medir, e dizer.
        let (p50, p99, max) = input_aspect(&m);
        assert!(
            p99 > 1.6,
            "{name}: a entrada tem aspecto p99 {p99:.2} — se ela ja' fosse isotropica \\
             (o nosso F1 da' 1,58), a barra deste teste teria de ser a do ORACULO e \\
             nao a da classe sem fase zero"
        );

        let (mesh, r) = extract(&m.as_map(), None).unwrap();
        let shape = ph2d_quadfill::quad_shape(&mesh);
        println!(
            "{name}: entrada aspecto p50 {p50:.2} p99 {p99:.2} max {max:.1}  ⇒  \
             saida {} quads, aspecto p50 {:.2} p99 {:.2} (>4x: {}), \
             enviesamento p50 {:.1} p99 {:.1} (>60: {}), area spread {:.2}",
            r.quads,
            shape.aspect_p50,
            shape.aspect_p99,
            shape.aspect_over_4,
            shape.skew_p50,
            shape.skew_p99,
            shape.skew_over_60,
            shape.area_spread
        );

        // ── 100 % quads é a barra que NÃO depende da fase zero.
        assert_eq!(mesh.face_count(), r.quads, "{name}: nem tudo e' quad");

        // ── O ASPECTO é bom mesmo sem a fase zero, e a barra é a do oráculo
        // (`p50 1,08`–`1,22`, e **zero** faces acima de `4×` em toda peça orgânica).
        assert!(
            shape.aspect_p50 < 1.35,
            "{name}: aspecto p50 {:.2}",
            shape.aspect_p50
        );
        assert!(
            shape.aspect_over_4 <= worst_over_4,
            "{name}: {} faces com aspecto acima de 4x, contra o ratchet {worst_over_4} \
             (o oraculo entrega ZERO, mas sobre entrada remalhada)",
            shape.aspect_over_4
        );

        // ── O ENVIESAMENTO fica na banda da classe de entrada, e é aqui que a fase
        // zero decide. ⛔ Um valor ABAIXO da banda também reprova: significaria que a
        // premissa deste teste mudou, e a barra tem de voltar a ser a do oráculo.
        assert!(
            shape.skew_p50 >= skew_band.0 && shape.skew_p50 <= skew_band.1,
            "{name}: enviesamento p50 {:.1}° fora da banda {:?} desta classe de \\
             entrada — se ele DESCEU, a entrada mudou e a barra tem de ser a do \\
             oraculo (4,8°-7,1°); se SUBIU, e' regressao",
            shape.skew_p50,
            skew_band
        );
    }
}
