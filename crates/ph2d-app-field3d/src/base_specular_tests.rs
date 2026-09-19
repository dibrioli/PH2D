//! ⏱️⭐ **AS CINCO QUE SOBRAM** — `base_weight`, `base_diffuse_roughness`, `specular_weight`,
//! `specular_color` e `specular_ior`: as únicas entradas do OpenPBR que a lei honra e o
//! [`crate::materials::surface_of`] não escreve (`docs/Render3d/05` §21.8).
//!
//! ⚠️ **A pergunta é a mesma das duas waves anteriores, e a resposta pode ser outra:** o verniz
//! passou porque os cinco números dele moviam o quadro. Estas cinco são **sempre vivas** — nenhuma
//! tem um peso que as desligue —, logo cada linha que ganharem é uma linha que o painel mostra
//! **sempre**. *O preço de as autorar não é o mesmo, e por isso a medição também não pode ser.*
//!
//! ⚠️⚠️ **E duas delas mudam de natureza com o METAL:** o `specular_color` tinge o realce de um
//! dieléctrico e **toda a reflexão** de um metal. Medir só no material de omissão (`metalness 0`)
//! responderia por metade da população.

use crate::render_light::{StudioSky, lamps};

/// ⏱️ **SONDA — quanto cada uma das cinco move o quadro, no dieléctrico e no metal.**
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_the_last_five_openpbr_inputs_would_buy() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};

    const BG: [u8; 4] = [12, 34, 56, 200];
    let (w, h) = (640, 360);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = trace(&doc, &reg, &cam, w, h);
    let lamps = lamps(&ph2d_light::LightRig::default());
    let light = Lighting {
        lamps: &lamps,
        points: &[],
        sky: &StudioSky,
        shadows: None,
    };
    let olhar = crate::shading::OPENING_LOOK;
    let pinta = |m: ph2d_material::OpenPbr| {
        let so = [m.prepare()];
        let surface = ph2d_field_render::Surfaces {
            all: &so,
            owners: None,
        };
        shade_render(
            &g,
            &cam,
            &surface,
            &light,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
        )
    };
    let diferenca = |a: &[u8], b: &[u8]| -> (u8, usize) {
        let (mut pior, mut visiveis) = (0u8, 0usize);
        for (i, (pa, pb)) in a
            .as_chunks::<4>()
            .0
            .iter()
            .zip(b.as_chunks::<4>().0.iter())
            .enumerate()
        {
            if !g.hit[i] {
                continue;
            }
            let d = (0..3).map(|c| pa[c].abs_diff(pb[c])).max().unwrap_or(0);
            pior = pior.max(d);
            visiveis += usize::from(d >= 2);
        }
        (pior, visiveis)
    };

    println!(
        "carga: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for (rotulo, metal) in [("DIELÉCTRICO", 0.0_f32), ("METAL", 1.0)] {
        let base = ph2d_material::OpenPbr {
            base_metalness: metal,
            ..ph2d_material::OpenPbr::default()
        };
        let referencia = pinta(base);
        println!("\n--- {rotulo} (metalness {metal}) ---");
        println!("            entrada · valor ·  pior Δ ·  pixels que mudam ≥2");
        let linha = |nome: &str, valor: String, m: ph2d_material::OpenPbr| {
            let (pior, n) = diferenca(&pinta(m), &referencia);
            println!("  {nome:>17} · {valor:>5} · {pior:7} · {n:22}");
        };
        for v in [0.0_f32, 0.5] {
            linha(
                "base_weight",
                format!("{v:.2}"),
                ph2d_material::OpenPbr {
                    base_weight: v,
                    ..base
                },
            );
        }
        for v in [0.5_f32, 1.0] {
            linha(
                "base_diffuse_rough",
                format!("{v:.2}"),
                ph2d_material::OpenPbr {
                    base_diffuse_roughness: v,
                    ..base
                },
            );
        }
        for v in [0.0_f32, 0.5] {
            linha(
                "specular_weight",
                format!("{v:.2}"),
                ph2d_material::OpenPbr {
                    specular_weight: v,
                    ..base
                },
            );
        }
        for c in [[1.0_f32, 0.6, 0.2], [0.2, 0.4, 1.0]] {
            linha(
                "specular_color",
                format!("{:.1}", c[1]),
                ph2d_material::OpenPbr {
                    specular_color: c,
                    ..base
                },
            );
        }
        for v in [1.0_f32, 1.2, 2.0, 2.5] {
            linha(
                "specular_ior",
                format!("{v:.2}"),
                ph2d_material::OpenPbr {
                    specular_ior: v,
                    ..base
                },
            );
        }
    }
}

/// A radiância total que um material devolve num ponto de frente — directa, céu e emissão.
fn devolve(m: ph2d_field_ecs::FieldMaterial) -> [f32; 3] {
    let s = crate::materials::surface_of(m);
    let n = [0.0_f32, 0.3, 0.953_939_2];
    let v = [0.0_f32, 0.0, 1.0];
    let para_a_luz = [0.4_f32, 0.6, 0.692_820_3];
    let d = s.direct(n, v, para_a_luz, [3.0; 3]);
    let i = s.indirect(n, v, &StudioSky);
    [0, 1, 2].map(|k| d[k] + i[k])
}

/// ⭐⭐⭐ **AS CINCO ÚLTIMAS CHEGAM À LEI — e cada uma MOVE a resposta.**
///
/// ⚠️ **A régua é a RADIÂNCIA, e não a [`ph2d_material::Surface`]:** ela guarda o `OpenPbr` inteiro
/// lá dentro, logo duas superfícies com params diferentes são **sempre** diferentes por `PartialEq`.
///
/// ⚠️ **E o `specular_weight` mede-se no METAL**, não no dieléctrico: ali ele move `4` bytes a meio
/// curso (medido) e num metal move `255`. *Um gate posto no lado fraco de um número que muda de
/// natureza mede o lado que não importa.*
///
/// **Mutações que devem sangrar:** apagar qualquer uma das cinco linhas do
/// [`crate::materials::surface_of`].
#[test]
fn the_last_five_openpbr_inputs_reach_the_law_and_move_the_answer() {
    use ph2d_field_ecs::FieldMaterial;
    let muda = |nome: &str, base: FieldMaterial, outro: FieldMaterial| {
        let (a, b) = (devolve(base), devolve(outro));
        let d = (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0_f32, f32::max);
        assert!(
            d > 1.0e-3,
            "mexer no `{nome}` não mudou a radiância ({a:?} → {b:?}) — ou ele não atravessa o \
             `surface_of`, ou a lei deixou de o ler"
        );
    };
    let d = FieldMaterial::default();
    muda(
        "base_weight",
        d,
        FieldMaterial {
            base_weight: 0.2,
            ..d
        },
    );
    muda(
        "base_diffuse_roughness",
        d,
        FieldMaterial {
            base_diffuse_roughness: 1.0,
            ..d
        },
    );
    muda(
        "specular_color",
        d,
        FieldMaterial {
            specular_color: [1.0, 0.4, 0.1],
            ..d
        },
    );
    muda(
        "specular_ior",
        d,
        FieldMaterial {
            specular_ior: 2.4,
            ..d
        },
    );
    // ⚠️ **No METAL**, pela razão do doc acima.
    let metal = FieldMaterial {
        metalness: 1.0,
        ..d
    };
    muda(
        "specular_weight",
        metal,
        FieldMaterial {
            specular_weight: 0.2,
            ..metal
        },
    );
}

/// ⭐⭐⭐ **DUAS DELAS SÃO EXACTAMENTE INERTES NUM METAL — e por isso não são publicadas ali.**
///
/// # ⚠️ A lei, e porque ela é EXACTA e não «pequena»
///
/// A rugosidade da difusa e o IOR alimentam o lóbulo **dieléctrico**, que o `base_metalness` mistura
/// para fora: `mix3(dieléctrico, metal, 1)` é `dieléctrico × 0 + metal × 1`, e `x × 0` é zero por
/// construção. ⇒ com `metalness == 1` varrer qualquer dos dois devolve **os mesmos bits**.
///
/// ⚠️ **Medido no quadro** antes de ser gateado (`docs/Render3d/05` §22): `0` bytes de diferença em
/// `61 804` pixels de peça, contra `39` e `17` no dieléctrico.
///
/// ⛔ **É a TERCEIRA forma da mesma lei neste painel**, e a primeira cujo predicado não é um peso a
/// zero: a cor do brilho morre com a luminância, os quatro do verniz morrem com o peso dele, e estes
/// dois morrem com o **metal a um**. *O que decide não é a forma do predicado — é o efeito ser
/// sempre nulo.*
///
/// **Mutações que devem sangrar:** apagar o braço `4 | 11` do `visivel` · trocá-lo por `<= 1.0`.
#[test]
fn the_two_dielectric_only_numbers_are_exactly_inert_on_a_metal() {
    use ph2d_field_ecs::FieldMaterial;
    let metal = FieldMaterial {
        metalness: 1.0,
        ..FieldMaterial::default()
    };
    let referencia = devolve(metal);
    for outro in [
        FieldMaterial {
            base_diffuse_roughness: 1.0,
            ..metal
        },
        FieldMaterial {
            specular_ior: 2.5,
            ..metal
        },
    ] {
        assert_eq!(
            devolve(outro),
            referencia,
            "num metal a rugosidade da difusa e o IOR têm de devolver os MESMOS BITS — a lei \
             mistura-os para fora, e a linha deles não é publicada por causa disso"
        );
    }

    // ⭐ **E a outra metade: a linha TRAVA, e só ali.** Sem isto, uma lei certa com um painel que a
    // ignora leria exactamente igual.
    //
    // ⚠️ **TRAVA e não some** — ordem do Enio (14/09): *«não devem desaparecer, mas apenas serem
    // inativados, mas sempre visíveis»*.
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = super::colour_row_tests::a_ball();
    let vivas = |sim: &mut ph2d_ecs::SimWorld| -> Vec<(&'static str, bool)> {
        super::colour_row_tests::rows_of(sim, folha)
            .iter()
            .map(|r| (r.key, r.inert.is_none()))
            .collect()
    };
    let so_dielectricas = ["field.dim.base_diffuse_roughness", "field.dim.specular_ior"];
    let antes = vivas(&mut sim);
    for k in so_dielectricas {
        assert_eq!(
            antes.iter().find(|(c, _)| *c == k).map(|(_, v)| *v),
            Some(true),
            "a linha `{k}` tem de estar VIVA num DIELÉCTRICO (o metal de omissão é `0`): {antes:?}"
        );
    }
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(5), 1.0)
        .expect("o metal");
    let depois = vivas(&mut sim);
    assert_eq!(
        depois.len(),
        antes.len(),
        "o metal fez a lista MUDAR DE TAMANHO — e o que ele muda é o `live`: {antes:?} → {depois:?}"
    );
    for k in so_dielectricas {
        assert_eq!(
            depois.iter().find(|(c, _)| *c == k).map(|(_, v)| *v),
            Some(false),
            "a linha `{k}` continua viva num METAL, onde ela é exactamente inerte: {depois:?}"
        );
    }
    // ⛔ **E trava o que morreu, e mais nada** — as duas, e as que já estavam travadas.
    let travadas: Vec<&str> = depois
        .iter()
        .filter(|(_, v)| !*v)
        .map(|(c, _)| *c)
        .collect();
    assert_eq!(
        travadas,
        vec![
            "field.dim.base_diffuse_roughness",
            "field.dim.specular_ior",
            "field.dim.coat_color",
            "field.dim.coat_roughness",
            "field.dim.coat_ior",
            "field.dim.coat_darkening",
            "field.dim.emission_color",
            // ⚠️ **As seis da SUBSUPERFICIE tambem estao travadas aqui**, e NAO por serem
            // so'-dielectricas: e' o peso dela estar a zero neste material, a mesma lei do verniz.
            // *Duas razoes diferentes para a mesma linha apagada — e este gate mede a lista, nao a
            // razao.*
            "field.dim.subsurface_color",
            "field.dim.subsurface_radius",
            "field.dim.subsurface_scale",
            "field.dim.subsurface_anisotropy",
            "field.dim.thin_walled",
        ],
        "o metal travou mais (ou menos) do que as duas linhas só-dieléctricas"
    );
}

/// ⭐⭐ **O IOR DA SUPERFÍCIE tem a MESMA faixa física do IOR do verniz** — `1` a `2,5`, dura.
///
/// ⚠️ **Os dois são a mesma grandeza e têm de ter a mesma cerca**, senão o artista aprende uma faixa
/// numa linha e encontra outra na linha de baixo. *Duas respostas à mesma pergunta divergem no dia
/// em que uma delas for afinada.*
///
/// **Mutação que deve sangrar:** tirar o `11` do braço `11 | 17` do `material_span`.
#[test]
fn the_surface_ior_has_the_same_physical_range_as_the_coats() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = super::colour_row_tests::a_ball();
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(12), 1.0)
        .expect("o verniz, para a linha dele existir");
    let rows = super::colour_row_tests::rows_of(&mut sim, folha);
    for k in [11u8, 17] {
        let linha = rows
            .iter()
            .find(|r| r.param == ph2d_field::Param::Material(k))
            .unwrap_or_else(|| panic!("a linha do IOR `{k}`"));
        assert!(
            (linha.lo - 1.0).abs() < 1.0e-6,
            "o IOR `{k}` começa em {} e tem de começar em 1 — abaixo do vácuo não há material",
            linha.lo
        );
        assert!(
            matches!(linha.bound, ph2d_field::Bound::Hard(t) if (t - 2.5).abs() < 1.0e-6),
            "o tecto do IOR `{k}` tem de ser DURO e `2,5` (acima do diamante): {:?}",
            linha.bound
        );
    }
}
