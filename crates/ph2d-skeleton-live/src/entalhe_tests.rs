//! ⭐⭐⭐ **O ENTALHE DO COTOVELO** — o report do dono de 2026-09-19, medido na barra do produto.
//!
//! ⚠️ **A régua é o PESCOÇO — o sítio mais estreito da forma** — e não a área nem a viragem:
//! - a **área** mal se mexe (`90,6 %` a `150°`) e não distingue um entalhe de um encolhimento;
//! - a **viragem** satura (`33°` a `90°` e a `150°`) e não sabe dizer qual é pior;
//! - os **cruzamentos** do contorno só aparecem quando já é tarde (`0` até `90°`);
//! - o **pescoço** vai de `1,00` (a espessura da barra) a `0,005`, e é o que o olho vê.
//!
//! ⛔⛔ **E ela teve de ser limpa DUAS vezes antes de dizer a verdade:** a `RoundRect` tem dois
//! segmentos de comprimento ZERO, e as amostras repetidas fabricavam `64` cruzamentos **em
//! repouso** e um pescoço de `0,0172` numa barra de espessura `1`.
use crate::barra_da_cena_tests_support::barra_da_cena;
use ph2d_ecs::Transform;

/// O contorno desenhado, achatado denso.
fn contorno(p: &ph2d_vec_scene::VecPath) -> Vec<[f64; 2]> {
    let c = p.cooked();
    let mut out = Vec::new();
    for k in 0..c.contour_count() {
        let Some((v, fechado)) = c.contour(k) else {
            continue;
        };
        let n = v.len();
        let ultimo = if fechado { n } else { n - 1 };
        for i in 0..ultimo {
            let (a, b) = (&v[i], &v[(i + 1) % n]);
            for j in 0..24 {
                let t = f64::from(j) / 24.0;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push([
                    w0 * a.anchor[0]
                        + w1 * a.out_handle[0]
                        + w2 * b.in_handle[0]
                        + w3 * b.anchor[0],
                    w0 * a.anchor[1]
                        + w1 * a.out_handle[1]
                        + w2 * b.in_handle[1]
                        + w3 * b.anchor[1],
                ]);
            }
        }
    }
    // ⚠️ **Pontos REPETIDOS fora** — os dois segmentos de comprimento zero da RoundRect dao 24
    // amostras no mesmo sitio, e elas fabricam cruzamentos e pescocos que nao existem.
    let mut limpo: Vec<[f64; 2]> = Vec::with_capacity(out.len());
    for q in out {
        if limpo
            .last()
            .is_none_or(|p: &[f64; 2]| (p[0] - q[0]).hypot(p[1] - q[1]) > 1e-9)
        {
            limpo.push(q);
        }
    }
    while limpo.len() > 1 {
        let (a, b) = (limpo[0], *limpo.last().expect("nao vazio"));
        if (a[0] - b[0]).hypot(a[1] - b[1]) <= 1e-9 {
            limpo.pop();
        } else {
            break;
        }
    }
    limpo
}

/// A area com sinal (positiva = anti-horario).
fn area(p: &[[f64; 2]]) -> f64 {
    let n = p.len();
    (0..n)
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        * 0.5
}

/// O PESCOCO: dois pontos do contorno perto no ESPACO e longe ao longo da CURVA.
/// Em repouso ele e' a espessura da barra; um entalhe aperta-o.
fn pescoco(p: &[[f64; 2]]) -> f64 {
    let n = p.len();
    // separacao por COMPRIMENTO DE ARCO, nao por indice
    let mut arco = vec![0.0_f64];
    for i in 1..n {
        arco.push(arco[i - 1] + (p[i][0] - p[i - 1][0]).hypot(p[i][1] - p[i - 1][1]));
    }
    let total = arco[n - 1] + (p[0][0] - p[n - 1][0]).hypot(p[0][1] - p[n - 1][1]);
    let minimo = total * 0.12;
    let _ = minimo;
    let sep = 1usize;
    let mut m = f64::INFINITY;
    let _ = sep;
    for i in 0..n {
        for j in (i + 1)..n {
            let ao_longo = (arco[j] - arco[i]).min(total - (arco[j] - arco[i]));
            if ao_longo < minimo {
                continue;
            }
            let d = (p[i][0] - p[j][0]).hypot(p[i][1] - p[j][1]);
            if d < m {
                m = d;
            }
        }
    }
    m
}

/// ⭐⭐⭐ **DOBRAR JÁ NÃO ENCOLHE A BARRA** — a cura do entalhe, medida na cena do dono.
///
/// A mistura linear interpola POSIÇÕES, e isso dá a **CORDA** do arco: a arte colapsa. A lei de
/// hoje roda em torno da JUNTA ([`ph2d_skeleton::centro`]).
///
/// | dobra | área LINEAR | área RÍGIDA | pescoço LINEAR | pescoço RÍGIDO |
/// |---|---|---|---|---|
/// | `30°` | `99,4 %` | **`100,0 %`** | `0,9655` | **`0,9833`** |
/// | `60°` | `97,7 %` | **`100,0 %`** | `0,8646` | **`0,9487`** |
/// | `90°` | `95,4 %` | **`100,0 %`** | `0,7045` | **`0,8927`** |
/// | `120°` | `92,8 %` | **`100,0 %`** | `0,1746` | **`0,3949`** |
///
/// ⛔ **A `150°` o pescoço continua a fechar** (`0,0017`) — ali a face de dentro dobra-se sobre si
/// mesma qualquer que seja a lei, e é uma limitação da geometria, não da mistura. *Fica declarada.*
///
/// ⚠️ **O CONTROLO é a mistura LINEAR, corrida na mesma árvore** (`recook_com_mistura(.., false)`):
/// sem ele este gate ficaria verde sobre uma barra que por acaso não dobra.
#[test]
fn dobrar_ja_nao_encolhe_a_barra() {
    let (sim0, scene0, _m, id0, _o) = barra_da_cena();
    let repouso = area(&contorno(
        scene0.paths().iter().find(|p| p.id == id0).expect("b"),
    ));
    drop(sim0);
    let medir = |graus: f64, rigido: bool| {
        let (mut sim, mut scene, _map, id, ossos) = barra_da_cena();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o angulo do gate, um punhado"
        )]
        let r = graus.to_radians() as f32;
        // dobra as DUAS juntas para o mesmo lado — o cotovelo do report
        sim.world_mut()
            .get_mut::<Transform>(ossos[1])
            .expect("pose")
            .rotation = r * 0.5;
        sim.world_mut()
            .get_mut::<Transform>(ossos[2])
            .expect("pose")
            .rotation = r;
        crate::skin_live::recook_com_mistura(&sim, &mut scene, true, rigido, true);
        let c = contorno(scene.paths().iter().find(|p| p.id == id).expect("b"));
        (area(&c) / repouso * 100.0, pescoco(&c))
    };
    for graus in [30.0_f64, 60.0, 90.0, 120.0] {
        let (a_rig, p_rig) = medir(graus, true);
        let (a_lin, p_lin) = medir(graus, false);
        eprintln!(
            "[entalhe] {graus:>5.0}° · area {a_lin:>6.1} % -> {a_rig:>6.1} % · pescoco {p_lin:.4} \
             -> {p_rig:.4}"
        );
        // ⭐ O CONTROLO vem primeiro: a mistura linear TEM de encolher, senao nao ha' cura a medir.
        assert!(
            a_lin < 99.5,
            "a {graus}° a mistura LINEAR nao encolheu a barra ({a_lin} %) — a fixtura deixou de \
             conter o entalhe"
        );
        assert!(
            a_rig > 99.5,
            "a {graus}° a lei de hoje encolheu a barra para {a_rig} % — ela deixou de rodar em \
             torno da junta"
        );
        assert!(
            p_rig > p_lin,
            "a {graus}° o pescoco nao melhorou ({p_lin} -> {p_rig})"
        );
    }
    // ⛔ E a limitacao DECLARADA: a 150° a face de dentro dobra-se sobre si mesma de qualquer
    // maneira. *Sem esta metade, alguem leria a tabela acima como «curado em todo o percurso».*
    let (a150, p150) = medir(150.0, true);
    eprintln!("[entalhe]   150° · area {a150:>6.1} % · pescoco {p150:.4} (a limitacao declarada)");
    assert!(
        p150 < 0.1,
        "a 150° o pescoco ficou em {p150} — se a lei passou a aguentar esta dobra, esta metade e' a \
         declaracao a apagar"
    );
}
