//! ⭐⭐⭐ **TODA FORMA DO CATÁLOGO CABE NO INTERPRETADOR DA LEI DO DONO** — o censo que mantém o
//! tecto [`ph2d_field_eval::owners::wgsl::REGISTOS_DO_INTERPRETADOR`] honesto.
//!
//! Uma folha que não cabe não falha: a peça cai na lei COMPILADA, que é certa e volta a custar uma
//! compilação do pintor por forma nova (o report de 2026-10-01, *«5 segundos para aparecer»*).
//! ⇒ uma forma de catálogo acima do tecto seria esse report a voltar **só para ela**, em silêncio.
//! Medido em 2026-10-01: o pior é a engrenagem, com `75` registos de `80`.

use ph2d_field::{FieldDoc, NodeId, Xform};

#[test]
fn toda_forma_do_catalogo_cabe_no_interpretador() {
    let mut medidas = 0usize;
    let mut pior = (0usize, "");
    let mut fora = Vec::new();
    for s in crate::shapes::SHAPES {
        let crate::shapes::Make::Formula(f) = s.make else {
            continue;
        };
        // ⚠️ **Posta e RODADA**: um xform de identidade deixa o compilador dobrar operações, e a
        // contagem de registos de uma folha real sai da folha como ela está na cena.
        let mut x = Xform::at(0.3, -0.2, 0.1);
        x.rotation = [0.2, 0.3, 0.1, 0.927];
        let d = FieldDoc::new(vec![ph2d_field_eval::leaf(f(0.4), x)], NodeId(0)).expect("folha");
        let campo = ph2d_field_eval::Field::new(&d);
        let Some(bc) = campo.tape_bytecode() else {
            fora.push(format!("{} (sem fita)", s.key));
            continue;
        };
        medidas += 1;
        if bc.registos > pior.0 {
            pior = (bc.registos, s.key);
        }
        if bc.registos > ph2d_field_eval::owners::wgsl::REGISTOS_DO_INTERPRETADOR {
            fora.push(format!("{} ({} registos)", s.key, bc.registos));
        }
    }
    // ⭐ **O PISO**: um censo que varre zero é verde a medir nada.
    assert!(
        medidas >= 60,
        "o censo mediu só {medidas} formas do catálogo — a lista mudou de forma ou o filtro deixou \
         de casar"
    );
    assert!(
        fora.is_empty(),
        "formas do catálogo fora do interpretador da lei do dono: {fora:?} (o pior medido: {} com \
         {} registos). Suba o `REGISTOS_DO_INTERPRETADOR` com a medição ao lado — ele é o rascunho \
         POR THREAD do pintor —, nunca deixe uma forma cair na lei compilada em silêncio",
        pior.1,
        pior.0
    );
}

/// ⏱️ **Quanto custa CONSTRUIR a lei do dono na CPU** — ela é reconstruída a cada mudança do
/// documento, incluindo cada quadro de um arrasto (`crate::materials::sync`).
#[test]
#[ignore = "sonda de diagnóstico: o relógio do Owners::new"]
fn diag_o_preco_de_construir_os_donos() {
    let reg = crate::smoke::sampled_registry();
    for n in [2usize, 3, 5, 8, 16] {
        #[allow(clippy::cast_precision_loss)]
        let postas: Vec<FieldDoc> = (0..n)
            .map(|i| {
                FieldDoc::new(
                    vec![ph2d_field_eval::leaf(
                        crate::shapes::a_box(0.2),
                        Xform::at(i as f32 * 0.3, 0.0, 0.0),
                    )],
                    NodeId(0),
                )
                .expect("folha")
            })
            .collect();
        let mut melhor = f64::INFINITY;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let o = ph2d_field_eval::owners::Owners::new(&postas, &reg, 1e-3);
            let w = o.to_wgsl(100);
            melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
            std::hint::black_box(w);
        }
        println!("  {n:>3} folhas · Owners::new + to_wgsl · {melhor:8.3} ms");
    }
}
