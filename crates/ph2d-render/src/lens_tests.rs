use super::*;
use ph2d_lens::{Aberracao, Grade, Grao, Vinheta};

pub(crate) fn lente_neutra() -> Lente {
    Lente {
        aberracao: Aberracao {
            amount: 0.0,
            linear: false,
            angle: 0.0,
        },
        vinheta: Vinheta {
            amount: 0.0,
            midpoint: 1.0,
            roundness: 0.0,
            feather: 0.5,
            center: [0.0; 2],
            cor: [0.0; 3],
        },
        grade: Grade {
            exposure: 0.0,
            temperatura: 0.0,
            matiz: 0.0,
            contraste: 0.0,
            saturacao: 0.0,
            lut_mix: 1.0,
        },
        grao: Grao {
            amount: 0.0,
            size: 1.0,
            response: 0.0,
            colorido: false,
            animado: false,
            semente: 0,
        },
    }
}

fn cubo_identidade() -> Cubo {
    let mut dados = Vec::new();
    for b in 0..2 {
        for g in 0..2 {
            for r in 0..2 {
                dados.push([r as f32, g as f32, b as f32]);
            }
        }
    }
    Cubo {
        n: 2,
        dados,
        dominio_min: [0.0; 3],
        dominio_max: [1.0; 3],
    }
}

/// ⭐ **Lente neutra ⇒ o passe não corre** — o quadro sem `fx.lens` fica byte-idêntico. O
/// `lut_mix` só conta com uma LUT carregada.
/// (Mutação: `de` sem o `neutra` ⇒ devolve `Some` — RED.)
#[test]
fn a_neutral_lens_does_not_run() {
    let l = lente_neutra();
    let rect = [0, 0, 64, 32];
    assert_eq!(LensUniform::de(&l, None, 7, rect), None);
    let mut sem_mix = l;
    sem_mix.grade.lut_mix = 0.0;
    assert_eq!(
        LensUniform::de(&sem_mix, Some(&cubo_identidade()), 7, rect),
        None
    );
    let u = LensUniform::de(&l, Some(&cubo_identidade()), 7, rect).expect("a LUT corre");
    assert_eq!(u.liga, [0, 0, 1, 0]);
    assert_eq!(u.modo[1], 1);
    assert_eq!(
        LensUniform::de(&l, Some(&cubo_identidade()), 7, [0, 0, 0, 32]),
        None
    );
}

/// Cada efeito liga só o seu interruptor.
#[test]
fn each_effect_switches_on_only_its_own_stage() {
    let rect = [3, 5, 64, 32];
    let mut l = lente_neutra();
    l.aberracao.amount = 0.01;
    assert_eq!(
        LensUniform::de(&l, None, 0, rect).unwrap().liga,
        [1, 0, 0, 0]
    );
    let mut l = lente_neutra();
    l.vinheta.amount = 0.4;
    assert_eq!(
        LensUniform::de(&l, None, 0, rect).unwrap().liga,
        [0, 1, 0, 0]
    );
    let mut l = lente_neutra();
    l.grade.exposure = 0.5;
    let u = LensUniform::de(&l, None, 0, rect).unwrap();
    assert_eq!(u.liga, [0, 0, 1, 0]);
    assert_eq!(u.modo[1], 0, "sem LUT carregada a amostragem não corre");
    let mut l = lente_neutra();
    l.grao.amount = 0.3;
    assert_eq!(
        LensUniform::de(&l, None, 0, rect).unwrap().liga,
        [0, 0, 0, 1]
    );
}

/// ⭐ **O grão animado anda com o TIQUE** (o scrub devolve o mesmo grão); parado, não anda.
#[test]
fn the_grain_seed_is_the_ticks() {
    let mut l = lente_neutra();
    l.grao.amount = 0.3;
    l.grao.semente = 100;
    let rect = [0, 0, 8, 8];
    assert_eq!(LensUniform::de(&l, None, 5, rect).unwrap().modo[3], 100);
    l.grao.animado = true;
    assert_eq!(LensUniform::de(&l, None, 5, rect).unwrap().modo[3], 105);
    assert_eq!(LensUniform::de(&l, None, 6, rect).unwrap().modo[3], 106);
}

/// O centro da lente é o do SUB-RECT (a cena), nunca o da janela.
#[test]
fn the_lens_centre_is_the_scene_rects() {
    let mut l = lente_neutra();
    l.aberracao.amount = 0.02;
    l.vinheta.amount = 0.5;
    let u = LensUniform::de(&l, None, 0, [100, 40, 64, 32]).unwrap();
    assert_eq!(u.rect, [100.0, 40.0, 64.0, 32.0]);
    assert_eq!([u.ab[0], u.ab[1]], [32.0, 16.0]);
    assert_eq!([u.vin_geo[0], u.vin_geo[1]], [32.0, 16.0]);
}

#[test]
fn the_rect_is_cut_to_the_target() {
    assert_eq!(corta([0, 0, 64, 32], (64, 32)), [0, 0, 64, 32]);
    assert_eq!(corta([10, 5, 64, 32], (64, 32)), [10, 5, 54, 27]);
    assert_eq!(corta([80, 5, 64, 32], (64, 32)), [64, 5, 0, 27]);
}

fn modulo(src: &str) -> naga::Module {
    naga::front::wgsl::parse_str(src)
        .unwrap_or_else(|e| panic!("lens WGSL não parsa:\n{}", e.emit_to_string(src)))
}

/// **O shader parsa e valida** — um erro de WGSL só apareceria ao construir o pipeline, no
/// arranque do app. E a biblioteca sozinha (a que a sonda do hash compõe) também.
#[test]
fn the_lens_wgsl_parses_and_validates() {
    for src in [LENS_WGSL, LENS_LIB_WGSL] {
        let m = modulo(src);
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&m)
        .unwrap_or_else(|e| panic!("lens WGSL não valida: {e:?}"));
    }
    let nomes: Vec<String> = modulo(LENS_WGSL)
        .entry_points
        .iter()
        .map(|e| e.name.clone())
        .collect();
    for want in ["vs_main", "fs_main"] {
        assert!(nomes.iter().any(|n| n == want), "{want} ∉ {nomes:?}");
    }
}

/// ⚠️ **O uniform tem o MESMO tamanho e a MESMA ordem nos dois lados** — um campo trocado no WGSL
/// lê o vizinho em silêncio.
#[test]
fn the_uniform_layout_matches_the_wgsl() {
    let m = modulo(LENS_WGSL);
    let mut layouter = naga::proc::Layouter::default();
    layouter.update(m.to_ctx()).expect("layout");
    let (h, ty) = m
        .types
        .iter()
        .find(|(_, t)| t.name.as_deref() == Some("LensU"))
        .expect("struct LensU");
    assert_eq!(
        layouter[h].size as usize,
        std::mem::size_of::<LensUniform>()
    );
    let naga::TypeInner::Struct { members, .. } = &ty.inner else {
        panic!("LensU não é struct");
    };
    let nomes: Vec<&str> = members.iter().filter_map(|m| m.name.as_deref()).collect();
    assert_eq!(
        nomes,
        [
            "rect", "liga", "modo", "ab", "ab_d", "vin_geo", "vin_par", "vin_alvo", "grade",
            "bal0", "bal1", "bal2", "lut_min", "lut_max", "grao"
        ]
    );
}
