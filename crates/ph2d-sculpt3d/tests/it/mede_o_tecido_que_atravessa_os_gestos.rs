//! ⭐⭐⭐ **O MATERIAL ENTRE GESTOS** — o report de 2026-09-09, e a família que o
//! censo achou à volta dele.
//!
//! > *«quando uso inflate e faço mais de uma simulação o objeto desinfla a cada
//! > início de simulação»*
//!
//! # O que estava a acontecer, medido
//!
//! A wave de 08/09 fez o material atravessar os gestos — a cura do report
//! anterior (*«se eu fizer mais de uma simulação o objeto continua esticando»*).
//! Ela é **incondicional**, e aplicada a um tipo que INFLA faz o gesto seguinte
//! começar por desfazer o anterior: o comprimento de repouso continua a ser o da
//! peça pequena, e a relaxação afunda a peça **abaixo do repouso** antes de a
//! força a voltar a levantar.
//!
//! | três gestos de Inflate, volume normalizado | fim de cada | **fundo** de cada |
//! |---|---|---|
//! | como estava | `1,168 · 1,166 · 1,166` | `1,000 · 0,887 · 0,886` |
//! | com [`ClothFilterKind::muda_o_material`] | `1,168 · 1,357 · 1,571` | `1,000 · 1,168 · 1,357` |
//!
//! ⭐ **O fundo de cada gesto passa a ser o fim do anterior** — a peça nunca
//! desce; é essa a leitura do report.
//!
//! ⚠️ **O censo da família achou um SEGUNDO membro que o report não nomeia** — a
//! `Scale`, pelo mesmo mecanismo (`1,001 · 0,890 · 0,871` de fundo). *O exemplo
//! que o dono aponta pode ser a excepção da família; o censo corre antes do
//! veredito.*
//!
//! # ⛔ A OUTRA METADE não vive aqui, e isso é deliberado
//!
//! Uma mutação que ponha [`ClothFilterKind::muda_o_material`] a devolver sempre
//! `true` reabre o report de 08/09, e quem a mata é o
//! `o_pano_nao_cresce_a_cada_gesto` do ficheiro irmão
//! (`mede_o_tecido_que_estica.rs`) — **medido**: com a mutação ele reprova e este
//! passa. ⛔ A 1.ª redacção deste ficheiro tinha um gate próprio para essa metade
//! e ele era **VÁCUO**: sem máscara, a gravidade sobre uma peça solta é uma
//! translação rígida, e uma translação rígida preserva o volume **tenha o
//! material sido re-semeado ou não** (`1,0000` nos três gestos, com e sem a
//! mutação). *Um gate que mede a grandeza que o gesto não muda passa sempre.*

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{ClothFilterKind, ClothFilterProps, ClothFilterStep, SculptStroke};

fn esfera() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(32, 64, 1.0)
}

fn volume(m: &Mesh) -> f64 {
    let mut t = Vec::new();
    for f in m.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            t.push([v[0], v[k], v[k + 1]]);
        }
    }
    let x: Vec<[f64; 3]> = m
        .positions()
        .iter()
        .map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
        .collect();
    ph2d_cloth::verlet::volume_de(&x, &t)
}

/// Três gestos seguidos do MESMO tipo, sobre o MESMO `SculptStroke` — que é a
/// única forma de a fixtura poder observar o que atravessa gestos.
///
/// Devolve, por gesto, `(fundo, fim)` do volume normalizado ao repouso.
fn tres_gestos(kind: ClothFilterKind) -> Vec<(f64, f64)> {
    let mut m = esfera();
    let v0 = volume(&m);
    let mut st = SculptStroke::default();
    let props = ClothFilterProps::default();
    let mut fora = Vec::new();
    for _ in 0..3 {
        st.cloth_filter_begin(&m, props, kind, [0.0, 0.9, 0.45]);
        let mut fundo = f64::INFINITY;
        for k in 0..120 {
            let p = ClothFilterStep {
                s: (k as f32 + 1.0) / 120.0,
                gravity_axis: [0.0, -1.0, 0.0],
                frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                axes: [true, true, true],
                eye: [0.0, 0.0, 1.0],
            };
            st.cloth_filter_step(&mut m, kind, &p);
            fundo = fundo.min(volume(&m) / v0);
        }
        st.cloth_filter_end();
        fora.push((fundo, volume(&m) / v0));
    }
    fora
}

/// ⭐⭐⭐ **O REPORT** — um gesto novo nunca começa por desfazer o anterior.
#[test]
fn um_gesto_que_muda_o_tamanho_nunca_desinfla_o_anterior() {
    for kind in [ClothFilterKind::Inflate, ClothFilterKind::Scale] {
        let g = tres_gestos(kind);
        println!(
            "{}: fundo {:.3} {:.3} {:.3} | fim {:.3} {:.3} {:.3}",
            kind.label(),
            g[0].0,
            g[1].0,
            g[2].0,
            g[0].1,
            g[1].1,
            g[2].1
        );
        for k in 1..3 {
            // ⚠️ **A barra é o FIM DO GESTO ANTERIOR**, não um número escolhido: o
            // defeito era exactamente a peça descer abaixo dele. A folga de `1 %`
            // é a relaxação a assentar, e o defeito media `−24 %`.
            assert!(
                g[k].0 > g[k - 1].1 * 0.99,
                "{}: o gesto {} afundou para {:.3}, abaixo do fim do anterior ({:.3})",
                kind.label(),
                k + 1,
                g[k].0,
                g[k - 1].1
            );
        }
        // E o gesto seguinte ACRESCENTA — senão o artista não tem como continuar.
        assert!(
            g[2].1 > g[0].1 * 1.10,
            "{}: tres gestos nao acumularam ({:.3} -> {:.3})",
            kind.label(),
            g[0].1,
            g[2].1
        );
    }
}
