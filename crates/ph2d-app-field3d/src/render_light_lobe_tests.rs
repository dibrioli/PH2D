//! ⭐⭐⭐ **A LEI DO LÓBULO** — o céu lido na direcção MÉDIA, e o que ela custa no pixel.
//!
//! # Por que um ficheiro irmão
//!
//! O [`super`] mede a LUZ do modo Render como um todo (o preço, o branco chapado, o matcap). Isto é
//! **um assunto**: a aproximação que o `Environment::radiance` fazia ao avaliar o céu na direcção
//! **espelhada**, e a lei que a substituiu (`docs/Render3d/05` §16).
//!
//! ⚠️ O corte foi forçado pelo tecto de `700` LOC (⛔ *corte, nunca `FILE_OVERAGE_OK`*) e é melhor
//! por isso: o oráculo, os dois gates e as duas sondas leem-se juntos e sem o resto.

use super::*;

/// ⏱️ **SONDA — o erro do céu na direcção ESPELHADA, e o factor que o cura.**
///
/// O §8 do `docs/Render3d/05` leva esta desde 13/09, com a nota de que *«ela torna-se visível no dia
/// em que houver material por objecto»*. ⭐ **Esse dia foi 14/09** — o metal passou a ser autorável
/// (§12), e a `CLAUDE.md` §0.0 diz o resto: *quem move o número que tornava algo inalcançável tem de
/// reconferir a nota.*
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_the_mirrored_direction_error_of_the_sky() {
    use ph2d_material::Environment;
    const RAW: f32 = 1.5;
    let ceu_cru = |up: f32| {
        [0, 1, 2].map(|i| {
            ph2d_light::AMBIENT * (ph2d_light::ENV_BASE[i] + RAW * ph2d_light::ENV_SLOPE[i] * up)
        })
    };
    println!("  α  ·  c(α)  ·   R.y  ·  espelhada  ·  verdade  ·  erro");
    for alpha in [0.05f32, 0.09, 0.25, 0.5, 0.75, 1.0] {
        let c = ph2d_material::lobe_shrink(alpha);
        for up in [1.0f32, 0.0, -1.0] {
            // A verdade: o céu na direcção MÉDIA do lóbulo.
            let verdade = ceu_cru(c * up);
            let espelhada = StudioSky.radiance([0.0, up, (1.0 - up * up).max(0.0).sqrt()], alpha);
            let erro = (0..3)
                .map(|i| ((espelhada[i] - verdade[i]) / verdade[i].max(1.0e-6)).abs())
                .fold(0.0f32, f32::max);
            println!(
                "{alpha:5.2} · {c:6.4} · {up:5.1} · {:10.5} · {:8.5} · {:6.2} %",
                espelhada[1],
                verdade[1],
                erro * 100.0
            );
        }
    }
}

/// ⭐⭐ **O CÉU HONRA A LARGURA DO LÓBULO QUE RECEBE** — a costura, do lado do produto.
///
/// # ⛔⛔ Porque o gate da forma fechada não chega
///
/// Ela pode estar certa e **ninguém a chamar** — era exactamente o estado anterior (`_alpha`, o
/// parâmetro deitado fora). *Um gate sobre a lei é cego a um consumidor que a ignora*, e essa é a
/// espécie de morto que o `CLAUDE.md` §5.0 chama de *«o consumidor que projecta o valor fora»*.
///
/// ⚠️ **A régua é o TERMO DA ALTURA, e o gate mede-o pelo par.**
///
/// ⚠️⚠️ **E ele PARTIU-SE em 2026-09-14, quando o céu ganhou a caixa de luz — de propósito, e a
/// forma como partiu é o achado.** A redacção anterior media a lei do lóbulo **contra a fórmula da
/// rampa** e afirmava, no equador, a **invariância** em `alpha`. Com uma caixa em `+y`, um lóbulo
/// largo avaliado no equador **arrasta a caixa para dentro da média** ⇒ aquela igualdade passou a ser
/// falsa, e passou a ser falsa porque a lei ficou MAIS forte: hoje o `alpha` entra por **dois**
/// caminhos independentes (o encolhimento da rampa e a linha da tabela).
///
/// ⇒ o gate parte-se em duas metades, cada uma sobre o céu de que ela fala:
/// a lei EXACTA da rampa sobre a [`crate::studio::Studio::bare_ramp`] (onde ela é a lei inteira), e
/// a **dependência** sobre o céu do produto, agora nos dois eixos.
///
/// **Mutações que devem sangrar:** voltar a `let up = dir[1];` · passar `1.0` fixo à tabela.
#[test]
fn the_sky_honours_the_lobe_width_it_is_handed() {
    use ph2d_material::Environment;
    let alto = [0.0, 1.0, 0.0];
    let equador = [0.0, 0.0, 1.0];
    let a = ph2d_light::AMBIENT;
    let rampa = crate::studio::Studio::bare_ramp();

    // ── A lei EXACTA, sobre a rampa nua ──────────────────────────────────────────────────────
    let liso = rampa.radiance(alto, 0.0);
    let rugoso = rampa.radiance(alto, 1.0);
    assert!(
        rugoso[1] < liso[1],
        "um lóbulo largo apontado ao topo tem de ler um céu MAIS ESCURO do que um espelho — a média \
         dele desce para o equador ({rugoso:?} contra {liso:?})"
    );
    // ⭐ **E o quanto é o que a lei promete**, não um valor qualquer.
    let esperado = [0, 1, 2].map(|i| {
        a * (ph2d_light::ENV_BASE[i]
            + 1.5 * ph2d_light::ENV_SLOPE[i] * ph2d_material::lobe_shrink(1.0))
    });
    for i in 0..3 {
        assert!(
            (rugoso[i] - esperado[i]).abs() < 1.0e-6,
            "o canal {i} não é o céu na direcção média do lóbulo"
        );
    }
    // ⛔ **No equador o encolhimento da RAMPA é invisível** — e tem de ser: `c·0 = 0`. É este assert
    // que impede a metade de cima de passar sobre um `radiance` que ignorasse a direcção.
    assert_eq!(
        rampa.radiance(equador, 0.0),
        rampa.radiance(equador, 1.0),
        "no equador a largura do lóbulo não pode mudar a rampa"
    );
    // ⭐ E o caminho de omissão: um material liso lê **exactamente** o que lia antes da cura.
    assert_eq!(liso, {
        let up = alto[1];
        [0, 1, 2].map(|i| a * (ph2d_light::ENV_BASE[i] + 1.5 * ph2d_light::ENV_SLOPE[i] * up))
    });

    // ── A DEPENDÊNCIA, sobre o céu do produto ────────────────────────────────────────────────
    // ⭐ No topo a caixa está toda dentro do espelho, e alargar o lóbulo dilui-a: o céu escurece.
    assert!(
        StudioSky.radiance(alto, 1.0)[1] < StudioSky.radiance(alto, 0.0)[1],
        "o céu do produto deixou de escurecer com a largura do lóbulo no topo"
    );
    // ⭐⭐ E no EQUADOR, onde a rampa é cega por construção, o céu do produto **tem** de se mover —
    // um lóbulo largo apanha a caixa que um espelho não vê. *É a metade que a caixa acrescentou.*
    let (eq_liso, eq_rugoso) = (
        StudioSky.radiance(equador, 0.0)[1],
        StudioSky.radiance(equador, 1.0)[1],
    );
    assert!(
        eq_rugoso > eq_liso * 1.05,
        "no equador um lóbulo largo tem de arrastar a caixa para dentro da média ({eq_liso} → \
         {eq_rugoso})"
    );
}

/// O céu como ele era **antes** de honrar a largura do lóbulo — o controlo desta medição.
///
/// ⚠️ Ele existe só aqui: um segundo `Environment` no produto seria a segunda resposta à mesma
/// pergunta. Aqui ele é o **lado A** de um A/B, e sem ele a sonda não teria com que comparar.
struct MirroredSky;
impl ph2d_material::Environment for MirroredSky {
    fn radiance(&self, dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        const RAW: f32 = 1.5;
        [0, 1, 2].map(|i| {
            ph2d_light::AMBIENT
                * (ph2d_light::ENV_BASE[i] + RAW * ph2d_light::ENV_SLOPE[i] * dir[1])
        })
    }
    fn irradiance(&self, n: [f32; 3]) -> [f32; 3] {
        ph2d_light::env_ambient([n[0], -n[1], n[2]])
    }
}

/// ⏱️ **SONDA — o que a cura do lóbulo muda NO PIXEL**, por material.
///
/// O §8 previa `p95 = 23` e `max = 33` bytes num metal rugoso, e `ruído` (`p50 = 0`, `p95 = 1`) no
/// material de omissão. ⭐ **Esta sonda mede isso no caminho do produto** — a mesma esfera, a mesma
/// luz, os mesmos dois sombreamentos, com o céu velho e o novo.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_the_lobe_cure_changes_in_the_pixel() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};

    const BG: [u8; 4] = [0, 0, 0, 0];
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
    let olhar = crate::shading::OPENING_LOOK;

    println!("material                      · muda · |Δ| p50 · p95 · max");
    for (nome, m) in [
        (
            "omissão (dieléctrico, r 0,30)",
            ph2d_material::OpenPbr::default(),
        ),
        (
            "metal polido  (metal 1, r 0,10)",
            ph2d_material::OpenPbr {
                base_metalness: 1.0,
                specular_roughness: 0.10,
                ..ph2d_material::OpenPbr::default()
            },
        ),
        (
            "metal escovado (metal 1, r 0,50)",
            ph2d_material::OpenPbr {
                base_metalness: 1.0,
                specular_roughness: 0.50,
                ..ph2d_material::OpenPbr::default()
            },
        ),
        (
            "metal fosco   (metal 1, r 1,00)",
            ph2d_material::OpenPbr {
                base_metalness: 1.0,
                specular_roughness: 1.0,
                ..ph2d_material::OpenPbr::default()
            },
        ),
    ] {
        let so = [m.prepare()];
        let surface = ph2d_field_render::Surfaces {
            all: &so,
            owners: None,
        };
        let pinta = |sky: &(dyn ph2d_material::Environment + Sync)| {
            shade_render(
                &g,
                &cam,
                &surface,
                &Lighting {
                    lamps: &lamps,
                    points: &[],
                    sky,
                    shadows: None,
                },
                &ph2d_field_render::Presentation::of(olhar),
                BG,
            )
        };
        let antes = pinta(&MirroredSky);
        let depois = pinta(&StudioSky);
        let mut d: Vec<i32> = Vec::new();
        for (i, (a, b)) in antes
            .as_chunks::<4>()
            .0
            .iter()
            .zip(depois.as_chunks::<4>().0.iter())
            .enumerate()
        {
            if !g.hit[i] {
                continue;
            }
            d.push(
                (0..3)
                    .map(|c| i32::from(b[c]) - i32::from(a[c]))
                    .max_by_key(|v| v.abs())
                    .unwrap_or(0)
                    .abs(),
            );
        }
        let mudam = d.iter().filter(|v| **v != 0).count();
        d.sort_unstable();
        let q = |f: f64| d[((d.len() - 1) as f64 * f) as usize];
        println!(
            "{nome:30} · {:4.0}% · {:7} · {:3} · {:3}",
            mudam as f64 / d.len() as f64 * 100.0,
            q(0.5),
            q(0.95),
            d[d.len() - 1]
        );
    }
}
