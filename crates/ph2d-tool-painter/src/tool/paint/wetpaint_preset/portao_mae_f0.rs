//! **E9 — o portão-mãe** (`docs/Painter/Arquitetura/E9_portao_mae.md`): os limiares finais da F0 §4 (F0b),
//! lidos no render PLANO, sobre uma célula da sonda. A mesma tabela vive em
//! `docs/Painter/Arquitetura/fotos/f0_varredura/resumo_f0.py` (a grelha inteira).
use super::*;
use ph2d_wet_paint::sim::LeiDaTinta;

pub(super) const MANCHAS_AZUL_MAX: f64 = 1.0;
pub(super) const KM_P99_MAX: f64 = 8.0;
pub(super) const KM_MAX_MAX: f64 = 20.0;
pub(super) const FERRUGEM_MAX: f64 = 0.05;
/// `furos = 0` só onde a mesma célula a pressão 1 cobre (furos < 1 %): abaixo disso é tinta fina, não furo.
pub(super) const FUROS_PR1_COBRE: f64 = 1.0;
pub(super) const DEGRAUS_X_MAX: usize = 2;

/// As métricas que reprovam (vazio = passa). `furos_pr1` = os furos da mesma célula a pressão 1. Um `NaN`
/// (nada com alfa ≥ 0,9) não reprova: é a régua que não mede, e o chamador diz se a célula TINHA de medir.
pub(super) fn reprova(c: &Celula, n: usize, furos_pr1: f64) -> Vec<String> {
    let q = c.plano;
    let mut r = vec![];
    if q.manchas_azul > MANCHAS_AZUL_MAX {
        r.push(format!("manchas_azul_plano {:.2}", q.manchas_azul));
    }
    if q.km_p99 > KM_P99_MAX {
        r.push(format!("km_p99_plano {:.1}", q.km_p99));
    }
    if q.km_max > KM_MAX_MAX {
        r.push(format!("km_max_plano {:.1}", q.km_max));
    }
    if q.ferrugem > FERRUGEM_MAX {
        r.push(format!("ferrugem_plano {:.2}", q.ferrugem));
    }
    if furos_pr1 < FUROS_PR1_COBRE && c.furos > 0.0 {
        r.push(format!("furos {:.2}", c.furos));
    }
    if q.degraus_y > n.saturating_sub(1) {
        r.push(format!("degraus_y_plano {}", q.degraus_y));
    }
    if q.degraus_x > DEGRAUS_X_MAX {
        r.push(format!("degraus_x_plano {}", q.degraus_x));
    }
    r
}

#[test]
#[ignore = "E9: liga quando a E6b/E8 fecharem"]
fn portao_mae_f0_lei_pigmento_minimo() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let mut falhas = vec![];
    for pincel in ["Oil/Flat/Thick Impasto", "Watercolor/Fluid/Wet Round"] {
        for lei in [LeiDaTinta::Velha, LeiDaTinta::Pigmento] {
            let cena = Cena {
                lei,
                fundo: Papel::Padrao,
                ..Cena::default()
            };
            let c = celula(&b, pincel, 600.0, 0.5, 1.0, 3, 300, cena, None);
            // pressão 1: a célula É o seu par de cobertura.
            let r = reprova(&c, 3, c.furos);
            let q = c.plano;
            println!(
                "[E9] {pincel} {}: manchas_azul {:.2} km_p99 {:.1} km_max {:.1} ferrugem {:.2} furos {:.2} \
                 degraus {}/{} → {r:?}",
                lei.nome(),
                q.manchas_azul,
                q.km_p99,
                q.km_max,
                q.ferrugem,
                c.furos,
                q.degraus_y,
                q.degraus_x
            );
            if lei == LeiDaTinta::Pigmento {
                assert!(
                    q.km_p99.is_finite(),
                    "{pincel}: P600/pr1 não cobre (alfa < 0,9) — o portão não mediu nada"
                );
                if !r.is_empty() {
                    falhas.push(format!("{pincel}: {r:?}"));
                }
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "lei Pigmento reprova o portão-mãe: {falhas:?}"
    );
}
