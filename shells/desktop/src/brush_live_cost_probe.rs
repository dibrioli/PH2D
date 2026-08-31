//! SONDA (`--ignored`): **quanto custa o [`crate::brush_live::resolve`] por quadro?**
//!
//! # A acusação que esta sonda mede
//!
//! O handoff de `59a80bd6e` deixou o item aberto: *"`resolve` ficou `O(P × G)` por quadro, sem
//! memo — o `cooked()` corre G vezes por pincel por quadro. ⛔ Sem relógio — a acusação é a
//! complexidade com endereço."* Isto é o relógio.
//!
//! # ⚠️ A sonda mede pela PORTA do produto
//!
//! O `object_of` que o quadro passa é
//! `crate::vec_entities::object_selection_for(sim, scene, map, id)` — a expansão REAL, sobre um
//! `SimWorld` com grupos construídos pelo verbo do produto (`group_entities`). Medir `resolve` com
//! um `object_of` sintético mediria metade da conta e chamaria o número de produto, que é o erro
//! que esta casa já pagou três vezes. As duas metades saem SEPARADAS na tabela, para atribuição —
//! mas a coluna que decide é a de ponta a ponta.
//!
//! # ⛔ A fixtura tem de CONTER o fenómeno
//!
//! O `cooked()` devolve `Cow::Borrowed` quando não há quina viva nem pilha de efeitos, e uma
//! fixtura de quadrados leria «não há custo» sobre uma feature cujo custo é precisamente o
//! cozimento. ⇒ há **duas** pontas de arte, e a sonda **afirma** que cada uma é o que diz ser
//! (`has_live_geometry`), imprimindo o preço de um `cooked()` de cada.
//!
//! Rode:
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins brush_live_cost -- --ignored --nocapture
//! ```

use crate::vec_entities::VecEntityMap;
use ph2d_ecs::SimWorld;
use ph2d_vec_scene::{
    BrushStroke, Rgba8, StrokePaint, StrokeSpec, VecPath, VecPathId, VecScene, VecVertex,
};

/// Caminhos na cena. O briefing pediu ~200, e S entra na conta: o `scene.path()` e o
/// `subtree_paths` são os dois LINEARES nela.
const CENA: usize = 200;

/// Corridas por célula. Mediana de 9 (o briefing pede ≥5).
const REPS: usize = 9;

/// O orçamento de um quadro a 60 fps.
const QUADRO_MS: f64 = 16.7;

// ---------------------------------------------------------------------------------------------
// Fixturas
// ---------------------------------------------------------------------------------------------

/// Um polígono de `n` lados, raio 1, centrado em `(cx, cy)`. **Sem** quina viva e **sem** pilha:
/// o `cooked()` devolve `Cow::Borrowed` e o custo é a clonagem do `into_owned()`.
fn poligono(n: usize, cx: f64, cy: f64) -> VecPath {
    let verts = (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            VecVertex::corner([cx + a.cos(), cy + a.sin()])
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// A MESMA forma com geometria **VIVA**: raio de quina em todo vértice + um ZigZag na pilha.
/// É a ponta CARA do `cooked()` — a que a doc do `art_of` diz que obriga a cozer
/// (*"um motivo com quina viva ou com uma pilha de efeitos"*).
fn poligono_vivo(n: usize, cx: f64, cy: f64) -> VecPath {
    let mut p = poligono(n, cx, cy);
    for v in &mut p.verts {
        v.corner_radius = 0.18;
    }
    p.effects.push(ph2d_vec_scene::effect::FxEntry::new(
        ph2d_vec_scene::effect::PathEffect::ZigZag(ph2d_vec_scene::fx_zigzag::ZigZagSpec {
            amplitude: 0.05,
            ridges: 8.0,
            smooth: false,
            rough_seed: None,
        }),
    ));
    p
}

/// Uma forma-ANFITRIÃ: o traço dela é um pincel que nomeia `art`.
fn hospedeira(cx: f64, art: VecPathId) -> VecPath {
    let mut p = poligono(4, cx, 40.0);
    let mut s = StrokeSpec::new(Rgba8::new(10, 20, 30, 255), 0.5);
    s.paint = StrokePaint::Brush(Box::new(BrushStroke {
        art: Some(art),
        ..BrushStroke::default()
    }));
    p.stroke = Some(s);
    p
}

/// A cena de uma célula da varredura.
///
/// ⚠️ **Os grupos de arte são PARTILHADOS quando `P × G` não cabe em [`CENA`]** — com `P = 50` e
/// `G = 16` seriam 800 membros numa cena de 200. Partilhar é o caso realista (um artista tem
/// poucos motivos e muitos traços), e o número de grupos sai IMPRESSO na tabela para que ninguém
/// leia a célula como se cada pincel tivesse arte própria.
///
/// Devolve `(scene, sim, map, grupos, membros_por_grupo)`.
fn cena(p_pinceis: usize, g: usize, vivo: bool) -> (VecScene, SimWorld, VecEntityMap, usize) {
    cena_s(p_pinceis, g, vivo, CENA)
}

/// A mesma cena, com o tamanho `s` explícito — a [`CENA`] é só o valor por omissão.
fn cena_s(
    p_pinceis: usize,
    g: usize,
    vivo: bool,
    s: usize,
) -> (VecScene, SimWorld, VecEntityMap, usize) {
    let mut scene = VecScene::new();
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();

    let cabem = s.saturating_sub(p_pinceis) / g;
    let grupos = p_pinceis.min(cabem).max(1);

    // Os membros de arte, grupo a grupo.
    let mut membros: Vec<Vec<VecPathId>> = Vec::with_capacity(grupos);
    for gi in 0..grupos {
        #[allow(clippy::cast_precision_loss)]
        let base = gi as f64 * 4.0;
        let mut ids = Vec::with_capacity(g);
        for mi in 0..g {
            #[allow(clippy::cast_precision_loss)]
            let dy = mi as f64 * 2.5;
            let forma = if vivo {
                poligono_vivo(7, base, dy)
            } else {
                poligono(7, base, dy)
            };
            ids.push(scene.push_path(forma));
        }
        membros.push(ids);
    }

    // Os pincéis: cada um nomeia o PRIMEIRO membro do grupo que lhe toca.
    for i in 0..p_pinceis {
        #[allow(clippy::cast_precision_loss)]
        let cx = i as f64 * 3.0;
        let alvo = membros[i % grupos][0];
        scene.push_path(hospedeira(cx, alvo));
    }

    // Enchimento até S ≈ CENA — o `scene.path()` e o `subtree_paths` são lineares nele.
    #[allow(clippy::cast_precision_loss)]
    while scene.paths().len() < s {
        let k = scene.paths().len() as f64;
        scene.push_path(poligono(5, k * 0.5, 90.0));
    }

    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);

    // Os grupos, pelo VERBO do produto — não por `ChildOf` à mão.
    if g > 1 {
        for ids in &membros {
            let bits: Vec<u64> = ids.iter().map(|id| map[id]).collect();
            crate::vec_entities::group_entities(&mut sim, &bits, "ArtGroup".into());
        }
    }
    (scene, sim, map, grupos)
}

// ---------------------------------------------------------------------------------------------
// Relógio
// ---------------------------------------------------------------------------------------------

/// Mediana, mínimo e desvio-padrão de `REPS` corridas, em ms.
struct Stats {
    mediana: f64,
    minimo: f64,
    desvio: f64,
}

/// ⚠️ **Mediana E mínimo.** Não há memo aqui: toda corrida faz *exactamente* o mesmo trabalho, e
/// por isso o MÍNIMO é a estimativa menos contaminada por carga da máquina — ele é a única coluna
/// que uma workstation ocupada não pode inflacionar. A mediana fica ao lado porque é o que o
/// briefing pede, e o desvio diz se as duas devem ser acreditadas.
fn medir(mut f: impl FnMut()) -> Stats {
    // Aquecimento: a primeira corrida paga o cache frio, e não é o que um quadro N faz.
    f();
    let mut ms = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        let t0 = std::time::Instant::now();
        f();
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    ms.sort_by(f64::total_cmp);
    #[allow(clippy::cast_precision_loss)]
    let n = ms.len() as f64;
    let media = ms.iter().sum::<f64>() / n;
    let var = ms.iter().map(|v| (v - media).powi(2)).sum::<f64>() / n;
    Stats {
        mediana: ms[ms.len() / 2],
        minimo: ms[0],
        desvio: var.sqrt(),
    }
}

/// A ROTA ANTIGA (`HEAD~1`), **reimplementada** — ⚠️ não é o binário antigo.
///
/// Era literalmente isto (`git show 59a80bd6e`):
/// ```text
/// fn art_of(scene, host, art) -> Option<VecPath> {
///     if art == host { return None; }
///     scene.path(art).map(|p| p.cooked().into_owned())
/// }
/// ```
fn resolve_antigo(scene: &VecScene) -> usize {
    let mut n = 0usize;
    for path in scene.paths() {
        let Some(b) = path
            .stroke
            .as_ref()
            .and_then(ph2d_vec_scene::StrokeSpec::brush)
        else {
            continue;
        };
        let Some(alvo) = b.art else { continue };
        if alvo == path.id {
            continue;
        }
        if let Some(p) = scene.path(alvo) {
            n += std::hint::black_box(p.cooked().into_owned()).verts.len();
        }
    }
    n
}

fn pct(ms: f64) -> f64 {
    ms / QUADRO_MS * 100.0
}

// ---------------------------------------------------------------------------------------------
// M3 — o `cooked()` sozinho, nas duas pontas
// ---------------------------------------------------------------------------------------------

#[test]
#[ignore = "sonda: rode com --release --bins -- --ignored --nocapture"]
fn brush_live_cost_m3_cooked() {
    println!("\n== M3 — o `cooked()` sozinho (o dominante?) ==");
    let barato = poligono(7, 0.0, 0.0);
    let caro = poligono_vivo(7, 0.0, 0.0);

    // ⛔ CONTROLO DA FIXTURA: sem isto a sonda mede dois `Cow::Borrowed` e conclui que não há custo.
    assert!(
        !barato.has_live_geometry(),
        "a ponta BARATA devia ser um `Cow::Borrowed`"
    );
    assert!(
        caro.has_live_geometry(),
        "a ponta CARA nao tem quina viva nem pilha — a fixtura NAO contem o fenomeno"
    );
    assert!(
        caro.cooked().verts.len() > barato.cooked().verts.len(),
        "o cozimento da ponta cara nao produziu geometria nova"
    );

    const N: usize = 2000;
    for (nome, p) in [
        ("simples (Borrowed)", &barato),
        ("VIVA (quina+ZigZag)", &caro),
    ] {
        let s = medir(|| {
            for _ in 0..N {
                std::hint::black_box(p.cooked().into_owned());
            }
        });
        println!(
            "  {nome:<22} {:>9.1} ns/cooked  (mediana de {N} chamadas: {:.3} ms, min {:.3}, sd {:.3})",
            s.mediana / (N as f64) * 1e6,
            s.mediana,
            s.minimo,
            s.desvio
        );
    }
    println!(
        "  [fixtura] simples: {} verts cozidos · viva: {} verts cozidos",
        barato.cooked().verts.len(),
        caro.cooked().verts.len()
    );
}

// ---------------------------------------------------------------------------------------------
// M1 + M2 — a varredura
// ---------------------------------------------------------------------------------------------

fn varre(vivo: bool) {
    println!(
        "\n== M1/M2 — `resolve` por quadro · arte {} · cena S={CENA} · mediana de {REPS} ==",
        if vivo {
            "VIVA (quina+ZigZag)"
        } else {
            "simples"
        }
    );
    println!(
        "  {:>3} {:>3} {:>4} | {:>9} {:>9} {:>8} {:>7} | {:>9} {:>9} | {:>9}",
        "P", "G", "grp", "TOTAL ms", "min", "sd", "% quad", "expansao", "art_of", "antigo(G=1)"
    );
    for p in [1usize, 10, 50] {
        for g in [1usize, 4, 16] {
            let (scene, sim, map, grupos) = cena(p, g, vivo);

            // (A) ponta a ponta, pela porta do produto.
            let total = medir(|| {
                std::hint::black_box(crate::brush_live::resolve(
                    &scene,
                    &|id| crate::vec_entities::object_selection_for(&sim, &scene, &map, id),
                    &ph2d_vec_scene::VecXforms::new(),
                ));
            });

            // ⛔ CONTROLO: a cena TEM de resolver P pincéis com G membros cada.
            let mapa = crate::brush_live::resolve(
                &scene,
                &|id| crate::vec_entities::object_selection_for(&sim, &scene, &map, id),
                &ph2d_vec_scene::VecXforms::new(),
            );
            assert_eq!(mapa.len(), p, "a cena nao resolveu os {p} pinceis");
            let membros_reais = mapa.values().map(Vec::len).max().unwrap_or(0);
            assert_eq!(
                membros_reais, g,
                "a expansao devolveu {membros_reais} membros, nao os {g} do grupo"
            );

            // (B) a metade da EXPANSÃO, sozinha — `object_selection_for` P vezes.
            let alvos: Vec<VecPathId> = scene
                .paths()
                .iter()
                .filter_map(|pa| {
                    pa.stroke
                        .as_ref()
                        .and_then(ph2d_vec_scene::StrokeSpec::brush)
                        .and_then(|b| b.art)
                })
                .collect();
            let expansao = medir(|| {
                for a in &alvos {
                    std::hint::black_box(crate::vec_entities::object_selection_for(
                        &sim, &scene, &map, *a,
                    ));
                }
            });

            // (C) a metade do `art_of` — `resolve` com um `object_of` SINTÉTICO (sem ECS).
            let sint: std::collections::BTreeMap<VecPathId, Vec<VecPathId>> = alvos
                .iter()
                .map(|a| {
                    (
                        *a,
                        crate::vec_entities::object_selection_for(&sim, &scene, &map, *a),
                    )
                })
                .collect();
            let art_of = medir(|| {
                std::hint::black_box(crate::brush_live::resolve(
                    &scene,
                    &|id| sint.get(&id).cloned().unwrap_or_else(|| vec![id]),
                    &ph2d_vec_scene::VecXforms::new(),
                ));
            });

            // (D) M4 — a rota ANTIGA reimplementada, só faz sentido em G = 1.
            let antigo = if g == 1 {
                let s = medir(|| {
                    std::hint::black_box(resolve_antigo(&scene));
                });
                format!("{:.4}", s.mediana)
            } else {
                "—".to_string()
            };

            println!(
                "  {p:>3} {g:>3} {grupos:>4} | {:>9.4} {:>9.4} {:>8.4} {:>6.2}% | {:>9.4} {:>9.4} | {antigo:>9}",
                total.mediana,
                total.minimo,
                total.desvio,
                pct(total.mediana),
                expansao.mediana,
                art_of.mediana,
            );
        }
    }
}

#[test]
#[ignore = "sonda: rode com --release --bins -- --ignored --nocapture"]
fn brush_live_cost_m1_simples() {
    varre(false);
}

#[test]
#[ignore = "sonda: rode com --release --bins -- --ignored --nocapture"]
fn brush_live_cost_m1_viva() {
    varre(true);
}

// ---------------------------------------------------------------------------------------------
// M4 — a wave, isolada: o MESMO G=1, rota nova contra rota antiga
// ---------------------------------------------------------------------------------------------

#[test]
#[ignore = "sonda: rode com --release --bins -- --ignored --nocapture"]
fn brush_live_cost_m4_baseline() {
    println!("\n== M4 — G=1: a rota NOVA contra a ANTIGA reimplementada (nao o binario antigo) ==");
    println!(
        "  {:>4} {:>7} | {:>10} {:>10} | {:>10} {:>10} | {:>7}",
        "P", "arte", "NOVA ms", "min", "ANTIGA ms", "min", "razao"
    );
    for vivo in [false, true] {
        for p in [1usize, 10, 50] {
            let (scene, sim, map, _) = cena(p, 1, vivo);
            let nova = medir(|| {
                std::hint::black_box(crate::brush_live::resolve(
                    &scene,
                    &|id| crate::vec_entities::object_selection_for(&sim, &scene, &map, id),
                    &ph2d_vec_scene::VecXforms::new(),
                ));
            });
            let velha = medir(|| {
                std::hint::black_box(resolve_antigo(&scene));
            });
            // ⛔ CONTROLO: as duas rotas têm de resolver o MESMO número de pincéis.
            assert_eq!(
                crate::brush_live::resolve(
                    &scene,
                    &|id| { crate::vec_entities::object_selection_for(&sim, &scene, &map, id) },
                    &ph2d_vec_scene::VecXforms::new()
                )
                .len(),
                p
            );
            assert!(
                resolve_antigo(&scene) > 0,
                "a rota antiga nao resolveu nada"
            );
            println!(
                "  {p:>4} {:>7} | {:>10.4} {:>10.4} | {:>10.4} {:>10.4} | {:>6.2}x",
                if vivo { "VIVA" } else { "simples" },
                nova.mediana,
                nova.minimo,
                velha.mediana,
                velha.minimo,
                nova.mediana / velha.mediana
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// M5 — a COMPLEXIDADE que o briefing afirma, contra a que o código tem
// ---------------------------------------------------------------------------------------------

/// ⛔⛔ **O briefing (e o handoff) dizem `O(P × G)`. A leitura do código diz `O(P × S × G)`** — e
/// esta sonda pergunta qual das duas o relógio confirma, variando **S** com `P` e `G` PARADOS.
///
/// As duas rotas lineares na cena, nenhuma delas nomeada na acusação:
/// - `vec_entities_selection::subtree_paths` filtra `scene.paths()` (S) com um `found.contains`
///   (G) ⇒ **O(S × G)** por pincel, só para expandir;
/// - `VecScene::path` é `paths().iter().find(…)` ⇒ **O(S)** por membro, G vezes por pincel.
///
/// Se a acusação estivesse certa, dobrar S não mexeria no relógio. Se a leitura estiver certa,
/// o relógio dobra com S.
#[test]
#[ignore = "sonda: rode com --release --bins -- --ignored --nocapture"]
fn brush_live_cost_m5_escala_com_a_cena() {
    println!("\n== M5 — `resolve` contra o TAMANHO DA CENA (P=10, G=4 parados) ==");
    println!(
        "  {:>5} | {:>10} {:>10} {:>8} {:>8} | {:>10} {:>8}",
        "S", "TOTAL ms", "min", "sd", "vs S=100", "expansao", "vs S=100"
    );
    let mut base = 0.0f64;
    let mut base_exp = 0.0f64;
    for s in [100usize, 400, 1600, 6400] {
        let (scene, sim, map, _) = cena_s(10, 4, false, s);
        assert_eq!(scene.paths().len(), s, "a cena nao tem o tamanho pedido");
        let t = medir(|| {
            std::hint::black_box(crate::brush_live::resolve(
                &scene,
                &|id| crate::vec_entities::object_selection_for(&sim, &scene, &map, id),
                &ph2d_vec_scene::VecXforms::new(),
            ));
        });
        // A metade PURA de expansão — é ela que o `subtree_paths` faz `O(S x G)`.
        let alvos: Vec<VecPathId> = scene
            .paths()
            .iter()
            .filter_map(|pa| {
                pa.stroke
                    .as_ref()
                    .and_then(ph2d_vec_scene::StrokeSpec::brush)
                    .and_then(|b| b.art)
            })
            .collect();
        assert_eq!(alvos.len(), 10, "a cena nao tem os 10 pinceis");
        let exp = medir(|| {
            for a in &alvos {
                std::hint::black_box(crate::vec_entities::object_selection_for(
                    &sim, &scene, &map, *a,
                ));
            }
        });
        if s == 100 {
            base = t.mediana;
            base_exp = exp.mediana;
        }
        println!(
            "  {s:>5} | {:>10.4} {:>10.4} {:>8.4} {:>7.2}x | {:>10.4} {:>7.2}x",
            t.mediana,
            t.minimo,
            t.desvio,
            t.mediana / base,
            exp.mediana,
            exp.mediana / base_exp
        );
    }
    println!("  (se a acusacao `O(P x G)` fosse completa, esta coluna seria PLANA)");
}

// ---------------------------------------------------------------------------------------------
// M6 — o memo PAGA-SE? o preço da CHAVE contra o preço do cozimento
// ---------------------------------------------------------------------------------------------

/// ⭐ **Um memo só vale o que a CHAVE dele custa a comparar.**
///
/// A chave que este produtor precisaria contém o CONTEÚDO dos membros (é o `shape: Vec<VecPath>`
/// que a [`crate::texture_pattern_live::Key`] já carrega, e pela mesma razão: `cooked()` lê
/// `verts`/`corner_radius`/`effects`, então uma chave sem eles congelaria a arte). Esta sonda
/// mede o `PartialEq` dessa chave contra o cozimento que ela evita.
#[test]
#[ignore = "sonda: rode com --release --bins -- --ignored --nocapture"]
fn brush_live_cost_m6_o_memo_paga_se() {
    println!("\n== M6 — o preco da CHAVE contra o preco do cozimento (arte VIVA) ==");
    println!(
        "  {:>3} | {:>12} {:>12} | {:>10} | {:>9}",
        "G", "chave ==", "cozimento", "razao", "poupanca"
    );
    for g in [1usize, 4, 16] {
        let membros: Vec<VecPath> = (0..g)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let dy = i as f64 * 2.5;
                poligono_vivo(7, 0.0, dy)
            })
            .collect();
        let copia = membros.clone();
        // ⛔ CONTROLO: uma chave que compara igual por ser VAZIA nao mediria nada.
        assert_eq!(membros, copia, "a chave nao compara igual consigo mesma");
        assert!(!membros.is_empty() && membros[0].has_live_geometry());

        const N: usize = 5000;
        let chave = medir(|| {
            for _ in 0..N {
                std::hint::black_box(membros == copia);
            }
        });
        let cozer = medir(|| {
            for _ in 0..N {
                for m in &membros {
                    std::hint::black_box(m.cooked().into_owned());
                }
            }
        });
        let ns_chave = chave.mediana / (N as f64) * 1e6;
        let ns_cozer = cozer.mediana / (N as f64) * 1e6;
        println!(
            "  {g:>3} | {ns_chave:>9.1} ns {:>12} | {:>9.0}x | {:>8.2}%",
            format!("{ns_cozer:.1} ns"),
            ns_cozer / ns_chave,
            (1.0 - ns_chave / ns_cozer) * 100.0
        );
    }
    println!("  (a `poupanca` e' o que um acerto de memo devolve de cada quadro)");
}
