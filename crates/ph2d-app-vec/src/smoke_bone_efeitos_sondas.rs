//! Sondas da cena `=5` — a FOTO sem a placa: o desenho de cada barra presa em SVG, às juntas
//! `SONDA_G1`/`SONDA_G2` (omissão `110`/`110`, A5-b: os dentes do *Zig Zag*). Saída em
//! `SONDA_SAIDA` (omissão `target/prova/efeitos`).

use super::*;

/// ⭐ **SONDA — a FOTO da `=5`**, uma barra por ficheiro.
#[test]
#[ignore = "sonda: escreve SVG"]
fn diag_a_foto_dos_efeitos() {
    let g = |nome: &str, omissao: f32| -> f32 {
        std::env::var(nome)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(omissao)
    };
    let (g1, g2) = (g("SONDA_G1", 110.0), g("SONDA_G2", 110.0));
    let saida = std::env::var("SONDA_SAIDA").unwrap_or_else(|_| "target/prova/efeitos".into());
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut st = crate::state::VecState::default();
    build(&mut scene, &mut sim, &mut st);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut st.entities);
    let pend = st.bone_smoke_pend.take().expect("pendentes");
    for (id, raiz) in &pend {
        assert_eq!(
            ph2d_skeleton_live::skin_live::bind(&mut sim, &mut scene, &st.entities, &[*id], *raiz),
            1
        );
        crate::smoke_bone_par::dobra_duas(&mut sim, raiz.expect("raiz"), g1, g2);
    }
    let d = ph2d_skeleton_live::skin_live::recook_desenhando(&sim, &mut scene.clone());
    std::fs::create_dir_all(&saida).expect("pasta");
    for ((id, _), (nome, _)) in pend.iter().zip(EFEITOS) {
        let Some(x) = d.get(id) else { continue };
        let mut c = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
        for v in x.forma.verts_all() {
            c = [
                c[0].min(v.anchor[0]),
                c[1].min(v.anchor[1]),
                c[2].max(v.anchor[0]),
                c[3].max(v.anchor[1]),
            ];
        }
        let so: ph2d_skeleton_live::skin_desenho::SkinDesenhado = [(*id, x.clone())].into();
        let f = format!("{saida}/efeitos_{g1}_{g2}_{}.svg", nome.replace(' ', "_"));
        let caixa = [c[0] - 0.2, c[1] - 0.2, c[2] + 0.2, c[3] + 0.2];
        std::fs::write(
            &f,
            crate::smoke_bone_copias::sondas::svg_do_desenho(&so, caixa),
        )
        .expect("svg");
        // Cada contorno: a área e a largura média (`2·área/perímetro`), em larguras do traço.
        let w = x.forma.stroke.as_ref().map_or(1.0, |s| s.width);
        for l in crate::smoke_bone_copias::sondas::polilinhas(&x.forma, 16) {
            let (mut a, mut per) = (0.0, 0.0);
            for s in l.windows(2) {
                a += s[0][0] * s[1][1] - s[1][0] * s[0][1];
                per += (s[1][0] - s[0][0]).hypot(s[1][1] - s[0][1]);
            }
            println!(
                "    contorno: área {:.5} · largura média {:.2} traços",
                a / 2.0,
                (a.abs() / per) / w
            );
        }
        println!(
            "  {f} · contornos {} · camada do traço {}",
            x.forma.contour_count(),
            x.traco.is_some()
        );
    }
}
