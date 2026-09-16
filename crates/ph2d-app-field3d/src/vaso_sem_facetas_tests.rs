//! ⭐⭐⭐ **O VASO NÃO TEM FACETAS NO MODO MODEL** — o gate do smoke do dono (2026-09-16: *«arestas
//! ainda visíveis»*, com foto).
//!
//! # O defeito
//!
//! A wave do arco pôs as quinas arredondadas do perfil como ARCOS na árvore GLOBAL, e o modo RENDER
//! — que traça na placa — ficou liso. **O modo MODEL traça na CPU** (`smoke_draw_thread::traca`: a
//! placa só entra com `Shading::Render`), e a CPU especializa a árvore por ladrilho: a folha
//! especializada lia o perfil pelo `ProfileIndex`, construído da **polilinha densa**. A normal é o
//! gradiente dessa árvore ⇒ constante em cada segmento ⇒ uma faixa de luz por segmento.
//!
//! | motor | picos de faceta (cena `5`, frente, `1920×1080`) |
//! |---|---:|
//! | CPU, antes da cura | **`12 196`** (maior `11,42°`) |
//! | placa, antes e depois | `0` |
//! | CPU, depois da cura | **`0`** |
//!
//! # A régua
//!
//! A assinatura de uma faceta, e só dela: ao descer uma coluna, a normal fica quase parada, **salta**
//! de uma vez, e volta a ficar parada. Uma superfície lisa muda a normal devagar e por igual, e uma
//! curva de raio pequeno muda-a depressa **mas também por igual** — só a faceta faz um pico isolado
//! entre vizinhos calmos.
//!
//! ⛔ **A 1.ª régua media o salto MÁXIMO e caiu**: os dois motores leram `82–85°` no mesmo pixel, que
//! era o EIXO (o pólo do torno) e as fronteiras de OCLUSÃO (o lábio à frente da parede interna).
//! Esta exige continuidade NO MUNDO entre os dois pixels, salta a coluna do eixo, e conta PICOS.

use ph2d_field_render::{Gbuffer, Orbit};

/// `(picos de faceta, passo mediano em graus, maior pico, pares medidos)`.
pub(super) fn facetas(g: &Gbuffer, passo_mundo: f32) -> (usize, f32, f32, usize) {
    let (w, h) = (g.width as usize, g.height as usize);
    let ang = |a: [f32; 3], b: [f32; 3]| {
        (a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    };
    let (mut picos, mut maior) = (0usize, 0.0f32);
    let mut todos: Vec<f32> = Vec::new();
    let fecha = |corrida: &mut Vec<f32>, picos: &mut usize, maior: &mut f32| {
        for k in 2..corrida.len().saturating_sub(2) {
            let d = corrida[k];
            let viz = corrida[k - 2]
                .max(corrida[k - 1])
                .max(corrida[k + 1])
                .max(corrida[k + 2]);
            if d > 1.5 && d > 4.0 * viz {
                *picos += 1;
                *maior = maior.max(d);
            }
        }
        corrida.clear();
    };
    for x in 0..w {
        // ⚠️ Fora da coluna do eixo: o pólo do torno é uma descontinuidade LEGÍTIMA.
        if x.abs_diff(w / 2) < 6 {
            continue;
        }
        let mut corrida: Vec<f32> = Vec::new();
        for y in 0..h - 1 {
            let (i, j) = (y * w + x, (y + 1) * w + x);
            let continuo =
                g.hit[i] && g.hit[j] && g.normal[i][2] > 0.35 && g.normal[j][2] > 0.35 && {
                    let (p, q) = (g.point[i], g.point[j]);
                    let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2))
                        .sqrt();
                    d < 3.0 * passo_mundo
                };
            if continuo {
                let d = ang(g.normal[i], g.normal[j]);
                corrida.push(d);
                todos.push(d);
            } else {
                fecha(&mut corrida, &mut picos, &mut maior);
            }
        }
        fecha(&mut corrida, &mut picos, &mut maior);
    }
    let pares = todos.len();
    todos.sort_by(f32::total_cmp);
    let med = todos.get(pares / 2).copied().unwrap_or(0.0);
    (picos, med, maior, pares)
}

/// A câmara da foto: a vista de FRENTE.
pub(super) fn camara_de_frente() -> Orbit {
    Orbit {
        rotation: ph2d_viewport3d::views::Standard::Front.rotation(),
        ..Orbit::default()
    }
}

/// ⭐⭐⭐ **O gate.** Os DOIS caminhos do traçado de CPU que o modo MODEL usa — com a cache de fitas
/// entre quadros (o de omissão) e sem ela (o de bissecção) —, na vista da foto.
#[test]
fn o_vaso_nao_tem_facetas_no_traçado_do_modo_model() {
    const W: u32 = 960;
    const H: u32 = 540;
    let doc = crate::smoke::scene(5);
    let reg = crate::smoke::sampled_registry();
    let cam = camara_de_frente();
    #[allow(clippy::cast_precision_loss)]
    let passo = 2.0 * cam.half_extent / H as f32;

    let sem_cache = ph2d_field_render::trace(&doc, &reg, &cam, W, H);
    let cache = ph2d_field_render::TapeCache::new();
    let parado = std::sync::atomic::AtomicBool::new(false);
    let com_cache =
        ph2d_field_render::trace_cancellable(&doc, &reg, &cam, W, H, &parado, true, Some(&cache))
            .expect("o traçado não foi cancelado");

    for (nome, g) in [
        ("sem cache", &sem_cache),
        ("com a cache de fitas", &com_cache),
    ] {
        let (picos, med, maior, pares) = facetas(g, passo);
        // ⚠️ O PISO: uma régua que não mede nada lê zero picos. O vaso ocupa uma boa parte do
        // quadro, e sem ele (ou sem continuidade) os pares caem para perto de zero.
        assert!(
            pares > 40_000,
            "[{nome}] a régua mediu só {pares} pares — ela não está a ver o vaso"
        );
        assert_eq!(
            picos, 0,
            "⛔ [{nome}] {picos} picos de faceta (maior {maior:.2}°, passo mediano {med:.3}°) — o \
             perfil do vaso chega à CPU como SEGMENTOS, e a luz mostra-os como faixas"
        );
    }
}
