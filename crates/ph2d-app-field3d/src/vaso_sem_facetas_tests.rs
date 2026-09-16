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

/// ⭐⭐ **E O BOTÃO `Resolution` NÃO AS DEVOLVE — de perto, no lábio** (2026-09-16, depois do smoke
/// aprovado).
///
/// Duas causas desfaziam as quinas quando o botão subia (registo: `BUGS_3dmodeling.md` #2): a barra
/// de «esta cúbica é um arco?» dividia-se pelo nível, e a quina de FORA do lábio vira `127°` e saía
/// numa cúbica só, que erra acima do que um quarto de círculo erra. Sem a divisão em duas metades,
/// no nível `4` o lábio volta à polilinha: **`10 020` picos, o maior de `6,23°`** (a conta prevê
/// `~5,6°` por segmento). Com ela: `0` em todos os níveis.
///
/// ⚠️ **Este gate prova a DIVISÃO, não a barra**: as metades do lábio são arcos mesmo com a barra
/// antiga, e a barra (o círculo, os níveis altos) é provada no `ph2d-field-profile` pela contagem.
///
/// ⛔ **A 1.ª redacção olhava o vaso INTEIRO e ficou verde com as duas curas desfeitas** — de longe
/// cada faceta tem menos de um pixel, e a régua dos picos precisa de normal PARADA em pixels
/// vizinhos. ⛔ **A 2.ª aproximou-se em PERSPECTIVA e ficou verde outra vez**: aproximar é trazer o
/// olho, e a `half_extent` `0,05` punha-o DENTRO do vaso, a medir a parede interna do outro lado.
#[test]
fn subir_o_resolution_nao_parte_o_labio() {
    const W: u32 = 960;
    const H: u32 = 540;
    let reg = crate::smoke::sampled_registry();
    // ⚠️ **Lente PARALELA**, apontada à quina de fora do lábio (raio `0,33`, topo `0,52`).
    let cam = Orbit {
        target: [0.0, 0.50, 0.3],
        half_extent: 0.03,
        lens: ph2d_field_render::Lens::Ortho,
        ..camara_de_frente()
    };
    #[allow(clippy::cast_precision_loss)]
    let passo = 2.0 * cam.half_extent / H as f32;
    // ⚠️ Pelo caminho da APP: o traçador recebe o que o `coarse_doc` lhe dá, a mexer e parado.
    for (nivel, mexer) in [1, 4, 16, ph2d_field::MAX_PROFILE_RESOLUTION]
        .into_iter()
        .flat_map(|n| [(n, true), (n, false)])
    {
        let doc = crate::smoke::scenes::vaso(nivel);
        let doc = crate::preview::coarse_doc(&doc, mexer).unwrap_or(doc);
        let g = ph2d_field_render::trace(&doc, &reg, &cam, W, H);
        let (picos, med, maior, pares) = facetas(&g, passo);
        assert!(
            pares > 20_000,
            "[nível {nivel} · mexer {mexer}] a régua mediu só {pares} pares — ela não está a ver o \
             lábio"
        );
        assert_eq!(
            picos, 0,
            "⛔ [nível {nivel} · mexer {mexer}] {picos} picos de faceta no lábio (maior {maior:.2}°, \
             passo mediano {med:.3}°) — subir o Resolution desfez os arcos das quinas"
        );
    }
}

/// O perfil da folha de uma peça de uma folha só: `(arcos, primitivas)`.
fn perfil_da_folha(d: &ph2d_field::FieldDoc) -> (usize, usize) {
    match &d.nodes()[0].kind {
        ph2d_field::NodeKind::Leaf(
            ph2d_field::Primitive::Revolve { profile }
            | ph2d_field::Primitive::Extrude { profile, .. },
        ) => (profile.arc_count(), profile.prim_count()),
        _ => panic!("a peça de teste é uma folha com perfil"),
    }
}

/// ⭐⭐⭐ **O PREVIEW NUNCA TROCA ARCOS POR UMA POLILINHA MAIS CARA** (2026-09-16).
///
/// ⛔⛔ O `coarse_doc` — que decide o que o modo MODEL traça, a mexer **e** parado (W85) — trocava o
/// perfil pelo engrossado sempre que a POLILINHA encolhia, e o engrossado não tem arcos. Medido antes
/// da cura, `(arcos, primitivas)` do que ia para o traçador:
///
/// | peça | nível | documento | a mexer | parado |
/// |---|---:|---|---|---|
/// | vaso da cena 5 | 1 | `(12, 24)` | o mesmo | o mesmo |
/// | vaso da cena 5 | 4 | `(12, 24)` | **`(0, 168)`** | o mesmo |
/// | vaso da cena 5 | 16 | `(12, 24)` | **`(0, 198)`** | **`(0, 329)`** |
/// | vaso da cena 5 | 64 | `(12, 24)` | **`(0, 241)`** | **`(0, 392)`** |
/// | círculo `r = 0,5` | 64 | `(4, 4)` | **`(0, 166)`** | **`(0, 332)`** |
///
/// ⇒ o smoke no nível de omissão estava certo, e subir o `Resolution` na app trazia de volta o custo
/// e as facetas que os gates do `ph2d-field-profile` diziam curados — *eles chamavam o cozedor, e a
/// app traça o que o preview lhe dá*.
#[test]
fn o_preview_nunca_troca_arcos_por_uma_polilinha_mais_cara() {
    for nivel in [1, 4, 16, ph2d_field::MAX_PROFILE_RESOLUTION] {
        let vaso = crate::smoke::scenes::vaso(nivel);
        let circulo = {
            let profile = ph2d_field_profile::cook_path_at(
                &ph2d_vec_scene::ellipse([1.0, 0.0], 0.5, 0.5),
                nivel,
            )
            .expect("o círculo é um perfil");
            ph2d_field::FieldDoc::new(
                vec![ph2d_field::Node {
                    kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }),
                    ..vaso.nodes()[0].clone()
                }],
                ph2d_field::NodeId(0),
            )
            .expect("o toro é um documento")
        };
        for (nome, doc) in [("vaso", &vaso), ("círculo", &circulo)] {
            let antes = perfil_da_folha(doc);
            for mexer in [true, false] {
                let tracado = crate::preview::coarse_doc(doc, mexer).unwrap_or_else(|| doc.clone());
                let depois = perfil_da_folha(&tracado);
                assert!(
                    depois.1 <= antes.1 && depois.0 == antes.0,
                    "⛔ [{nome} · nível {nivel} · {}] o documento tem {antes:?} (arcos, primitivas) e \
                     o traçador recebe {depois:?}",
                    if mexer { "a mexer" } else { "parado" }
                );
            }
        }
    }
}

/// Um contorno MISTO: a base com duas quinas arredondadas (dois ARCOS de `90°`) e o topo em `ondas`
/// cúbicas genéricas (achatadas), à direita do eixo para o torno.
fn misto(ondas: usize) -> ph2d_vec_scene::VecPath {
    use ph2d_vec_scene::{VecVertex, VertexKind};
    let (x0, x1, y0, y1) = (0.5, 1.5, -0.3, 0.3);
    let mut verts = vec![
        VecVertex {
            corner_radius: 0.1,
            ..VecVertex::corner([x0, y0])
        },
        VecVertex {
            corner_radius: 0.1,
            ..VecVertex::corner([x1, y0])
        },
    ];
    let w = (x1 - x0) / ondas as f64;
    for i in 0..=ondas {
        let x = x1 - w * i as f64;
        let d = if i % 2 == 0 { 0.12 } else { -0.12 };
        verts.push(VecVertex {
            anchor: [x, y1],
            in_handle: if i == 0 {
                [x, y1]
            } else {
                [x + w * 0.4, y1 - d]
            },
            out_handle: if i == ondas {
                [x, y1]
            } else {
                [x - w * 0.4, y1 + d]
            },
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        });
    }
    ph2d_vec_scene::VecPath {
        verts,
        closed: true,
        ..ph2d_vec_scene::VecPath::default()
    }
}

/// ⭐⭐⭐ **O PREVIEW ENGROSSA SÓ O QUE ESTÁ TESSELADO** (2026-09-16) — a metade que o gate de cima não
/// via: um contorno MISTO (arcos + curvas livres).
///
/// ⛔⛔ O `coarsen` devolvia sempre uma polilinha sem arcos, e o `coarse_doc` só a aceitava se ficasse
/// mais barata. Num contorno só de arcos e rectas (o vaso) isso nunca acontece; num misto acontece
/// assim que o `Resolution` sobe — e quando não acontece, a parte tesselada também não é engrossada.
/// Medido, `(arcos, primitivas)` do que ia para o traçador, antes → depois da cura:
///
/// | ondas | nível | documento | a mexer | parado |
/// |---:|---:|---|---|---|
/// |  1 |  1 | `(2, 34)` | `(2, 34)` → `(2, 19)` | `(2, 34)` → `(2, 30)` |
/// |  1 | 16 | `(2, 121)` | **`(0, 92)`** → `(2, 21)` | `(2, 121)` → `(2, 35)` |
/// |  1 | 64 | `(2, 237)` | **`(0, 93)`** → `(2, 22)` | **`(0, 182)`** → `(2, 37)` |
/// |  4 |  4 | `(2, 221)` | `(2, 221)` → `(2, 154)` | `(2, 221)` → `(2, 205)` |
/// |  4 | 64 | `(2, 857)` | **`(0, 256)`** → `(2, 185)` | **`(0, 471)`** → `(2, 326)` |
/// | 12 |  4 | `(2, 499)` | **`(0, 462)`** → `(2, 392)` | `(2, 499)` → `(2, 464)` |
/// | 12 | 64 | `(2, 1 974)` | **`(0, 785)`** → `(2, 716)` | **`(0, 1 315)`** → `(2, 1 172)` |
///
/// ⇒ três metades: os arcos ficam, o que vai para o traçador é sempre MAIS barato que o documento
/// (a parte tesselada engrossa), e os arcos que ficam são os MESMOS — ponta, *bulge* e ponta seguinte.
#[test]
fn o_preview_engrossa_so_o_que_esta_tesselado() {
    let vaso = crate::smoke::scenes::vaso(1);
    for ondas in [1, 4, 12] {
        for nivel in [1, 4, 16, ph2d_field::MAX_PROFILE_RESOLUTION] {
            let profile = ph2d_field_profile::cook_path_at(&misto(ondas), nivel)
                .expect("o misto é um perfil");
            let doc = ph2d_field::FieldDoc::new(
                vec![ph2d_field::Node {
                    kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }),
                    ..vaso.nodes()[0].clone()
                }],
                ph2d_field::NodeId(0),
            )
            .expect("o misto torneado é um documento");
            let antes = perfil_da_folha(&doc);
            assert_eq!(
                antes.0, 2,
                "[{ondas} ondas · nível {nivel}] a fixtura tem dois arcos"
            );
            for mexer in [true, false] {
                let quando = if mexer { "a mexer" } else { "parado" };
                let tracado = crate::preview::coarse_doc(&doc, mexer)
                    .expect("um contorno misto tem sempre o que engrossar");
                let depois = perfil_da_folha(&tracado);
                assert!(
                    depois.0 == antes.0 && depois.1 < antes.1,
                    "⛔ [{ondas} ondas · nível {nivel} · {quando}] o documento tem {antes:?}                      (arcos, primitivas) e o traçador recebe {depois:?}"
                );
                assert_eq!(
                    arcos_com_pontas(&tracado),
                    arcos_com_pontas(&doc),
                    "[{ondas} ondas · nível {nivel} · {quando}] um arco mudou ao engrossar"
                );
            }
        }
    }
}

/// Cada arco da folha como `(ponta, bulge, ponta seguinte)`.
fn arcos_com_pontas(d: &ph2d_field::FieldDoc) -> Vec<([f32; 2], f32, [f32; 2])> {
    let ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }) = &d.nodes()[0].kind
    else {
        panic!("a peça de teste é um torno");
    };
    profile
        .arcs()
        .iter()
        .flat_map(|a| {
            (0..a.len())
                .filter(|&i| a[i].1 != 0.0)
                .map(|i| (a[i].0, a[i].1, a[(i + 1) % a.len()].0))
        })
        .collect()
}
