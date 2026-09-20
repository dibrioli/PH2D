//! Os gates do RETÍCULO à vista — ver o cabeçalho do [`super`].

use super::{malha_do_peso, malhas_do_indicador};
use crate::barra_da_cena_tests_support::{PPM, barra_da_cena, forma, raio_de_fabrica};
use ph2d_ecs::Transform;

/// ⭐⭐⭐ **A MALHA CHEGA AO DESENHO, e ela é muito mais densa do que os pontos.**
///
/// ⚠️ **A 2.ª metade é o report:** sem ela um retículo de trinta e quatro vértices passaria neste
/// gate e o artista continuaria a ver o que já via. O número do lado é o que a sonda mediu na barra
/// da cena do dono — `498` vértices de malha contra `34` nós.
#[test]
fn o_reticulo_e_muito_mais_denso_do_que_os_pontos() {
    let (sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    let m = malha_do_peso(&sim, alvo, ossos[1]).expect("a barra tem retículo");

    assert_eq!(m.verts.len(), m.pesos.len(), "um peso por vértice");
    assert!(
        !m.tris.is_empty(),
        "um retículo sem triângulos nao se desenha"
    );
    for t in &m.tris {
        for &k in t {
            assert!(
                (k as usize) < m.verts.len(),
                "triângulo a apontar para fora da malha"
            );
        }
    }
    let nos = crate::peso_a_mao::pontos_de_peso(&sim, alvo, ossos[1], PPM).len();
    assert!(
        m.verts.len() > nos * 5,
        "o retículo tem {} vértices contra {nos} nós — se nao for MUITO mais denso, ele nao \
         responde ao report",
        m.verts.len()
    );
    for &w in &m.pesos {
        assert!((-1e-9..=1.0 + 1e-9).contains(&w), "peso fora da faixa: {w}");
    }
}

/// ⭐⭐⭐ **O RETÍCULO DOBRA COM O OSSO** — ele mostra o lattice a fazer o trabalho, não o repouso.
///
/// ⚠️ **A 1.ª asserção é o CONTROLO e vem primeiro:** em repouso o retículo NÃO se mexe, e sem essa
/// metade a segunda passaria com um retículo que se mexe sozinho.
///
/// (Mutação: devolver `p` em vez de `pele.blend(p, &w)` ⇒ RED na segunda.)
#[test]
fn o_reticulo_dobra_com_o_osso() {
    let (mut sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    let repouso = malha_do_peso(&sim, alvo, ossos[1]).expect("retículo");
    let outra = malha_do_peso(&sim, alvo, ossos[1]).expect("retículo");
    let pior = |a: &super::MalhaDoPeso, b: &super::MalhaDoPeso| {
        a.verts
            .iter()
            .zip(&b.verts)
            .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
            .fold(0.0_f64, f64::max)
    };
    assert_eq!(
        pior(&repouso, &outra),
        0.0,
        "em repouso ele tem de ser estável"
    );

    sim.world_mut()
        .get_mut::<Transform>(ossos[1])
        .expect("Transform")
        .rotation = 45.0_f32.to_radians();
    let dobrado = malha_do_peso(&sim, alvo, ossos[1]).expect("retículo");
    let d = pior(&repouso, &dobrado);
    assert!(
        d > 0.5,
        "dobrar o osso do meio moveu o retículo só {d:.4} — ele está a desenhar o REPOUSO"
    );
}

/// ⭐⭐⭐ **PINTAR UM PESO MOVE O RETÍCULO** — a lei que o artista edita é a que ele vê.
///
/// ⛔⛔ Sem isto o retículo mostraria o padrão-ouro CRU e o pincel escreveria noutro sítio: o
/// artista corrigia um peso e a leitura na tela ficava igual, que é exactamente a cegueira que o
/// indicador de pontos veio curar em 2026-09-19.
///
/// (Mutação: passar `&[]` em vez de `&correcoes` ⇒ RED.)
#[test]
fn pintar_um_peso_move_o_reticulo() {
    let (mut sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    // ⛔⛔⛔ **O OSSO EM FOCO É O DA PONTA e o dedo cai no território do osso do MEIO** — e as duas
    // metades são obrigatórias, porque a lei da mancha **normaliza** depois de somar:
    //
    // | onde | o que a mancha faz |
    // |---|---|
    // | no meio do próprio osso (`w ≈ 1` e mais ninguém reclama) | `+0,5` satura, `−0,6` é desfeito pela renormalização ⇒ **`0,0000`** |
    // | no território de OUTRO osso (`w ≈ 0`) | o peso aparece ⇒ mexe |
    //
    // *A 1.ª redacção deste gate pintou nos dois sentidos no sítio saturado e leu `0,0000` e
    // `0,0019` sobre uma fiação CERTA — um corpus onde a lei é inerte não testa a lei.*
    let antes = malha_do_peso(&sim, alvo, ossos[0]).expect("retículo");

    let r = crate::peso_a_mao::pinta(
        &mut sim,
        alvo,
        ossos[0],
        PPM,
        [-5.0, 2.0],
        raio_de_fabrica(),
        ph2d_skeleton::Especie::Soma(0.5),
    );
    assert!(
        matches!(r, crate::peso_a_mao::Pincelada::Pintada { .. }),
        "a fixtura tem de conseguir pintar, senão o gate afirma sobre um traço recusado: {r:?}"
    );

    let depois = malha_do_peso(&sim, alvo, ossos[0]).expect("retículo");
    let pior = antes
        .pesos
        .iter()
        .zip(&depois.pesos)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    assert!(
        pior > 0.05,
        "a mancha pintada mudou o retículo só {pior:.4} — as correcções nao chegam a ele"
    );
}

/// ⭐⭐ **O RETÍCULO CONCORDA COM O PONTO QUE O PINCEL JÁ PINTA.**
///
/// ⚠️ **Duas leituras da mesma grandeza na mesma tela têm de fechar** — se divergirem, o artista vê
/// um ponto quente sobre um retículo frio e não tem como saber qual é a arte.
#[test]
fn o_reticulo_concorda_com_os_pontos() {
    let (sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    let m = malha_do_peso(&sim, alvo, ossos[1]).expect("retículo");
    let pontos = crate::peso_a_mao::pontos_de_peso(&sim, alvo, ossos[1], PPM);
    assert!(!pontos.is_empty(), "a fixtura tem de ter pontos");

    let (mut pior, mut comparados) = (0.0_f64, 0usize);
    for p in &pontos {
        // O vértice de malha mais perto deste nó, no MESMO espaço (mundo, já posado).
        let mut melhor = (f64::INFINITY, 0.0_f64);
        for (v, &w) in m.verts.iter().zip(&m.pesos) {
            let d = (v[0] - p.mundo[0]).hypot(v[1] - p.mundo[1]);
            if d < melhor.0 {
                melhor = (d, w);
            }
        }
        // ⚠️ Só os nós que a malha de facto cobre — uma quina redonda tem nós fora do triângulo
        // mais perto, e ali a comparação mediria a distância e não o peso.
        if melhor.0 < 0.05 {
            comparados += 1;
            pior = pior.max((melhor.1 - p.peso).abs());
        }
    }
    // ⛔⛔ **O PISO DE POPULAÇÃO vem primeiro** — sem ele um retículo posto no sítio errado não
    // encontra vértice nenhum perto de nó nenhum, `pior` fica em `0` e o gate aprova a avaria que
    // existe para apanhar. *Um zero de «não medido» e um de «concordam» são o mesmo byte.*
    assert!(
        comparados >= pontos.len() / 2,
        "só {comparados} de {} nós tinham vértice de malha por perto — o retículo não está sobre a \
         arte",
        pontos.len()
    );
    assert!(
        pior < 0.05,
        "o retículo e os pontos discordam em {pior:.4} — sao duas leis para a mesma grandeza"
    );
}

/// ⛔⛔ **AS METADES NEGATIVAS** — sem osso em foco, e com um osso que nao governa esta pele.
///
/// *Um retículo desenhado para um osso que o pincel recusa promete um gesto que a porta ao lado
/// nao aceita.*
#[test]
fn sem_osso_ou_com_osso_de_fora_nao_ha_reticulo() {
    let (mut sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    assert!(
        malhas_do_indicador(&sim, None).is_empty(),
        "sem osso em foco nao há de quem mostrar peso"
    );
    assert_eq!(
        malhas_do_indicador(&sim, Some(ossos[0])).len(),
        1,
        "um osso DESTA pele tem de devolver o retículo dela"
    );

    let estranho =
        crate::barra_da_cena_tests_support::osso(&mut sim, "Fora", [9.0, 9.0], 1.0, None);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert!(
        malha_do_peso(&sim, alvo, estranho).is_none(),
        "um osso que nao é tendao desta pele nao tem retículo nela"
    );
}

/// ⚠️ **SONDA — o preço do retículo por quadro.** Ele corre com o verbo `Weight` na mão, logo o
/// recurso é o RELÓGIO DO QUADRO. Corra em `--release`: o debug lê ~20× mais lento.
#[test]
fn diag_o_preco_do_reticulo() {
    let (sim, _scene, map, id, ossos) = barra_da_cena();
    let _ = forma(&map, id);
    let osso = Some(ossos[1]);
    let m = malhas_do_indicador(&sim, osso);
    let (v, t): (usize, usize) = m
        .iter()
        .fold((0, 0), |(a, b), x| (a + x.verts.len(), b + x.tris.len()));
    let mut pior = f64::INFINITY;
    for _ in 0..12 {
        let t0 = std::time::Instant::now();
        let r = malhas_do_indicador(&sim, osso);
        pior = pior.min(t0.elapsed().as_secs_f64() * 1e3);
        std::hint::black_box(r);
    }
    println!(
        "\nretículos={} vértices={v} triângulos={t} · CPU {pior:.3} ms ({:.2} % de um quadro)\nloadavg: {}",
        m.len(),
        pior / 16.67 * 100.0,
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// ⚠️ **SONDA — que osso da barra tem mais a MOSTRAR.** O roteiro de um smoke nomeia UM osso, e a
/// escolha decide se o artista vê a rampa inteira ou uma cor só.
#[test]
fn diag_qual_osso_mostra_mais() {
    let (sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    println!("\n{:-<64}", "");
    println!(
        "{:<10} {:>8} {:>8} {:>10} {:>10}",
        "osso", "min", "max", "x do max", "faixa"
    );
    println!("{:-<64}", "");
    for (k, &o) in ossos.iter().enumerate() {
        let Some(m) = malha_do_peso(&sim, alvo, o) else {
            println!("Bone {}: sem retículo", k + 1);
            continue;
        };
        let (mut lo, mut hi, mut xhi) = (f64::INFINITY, f64::NEG_INFINITY, 0.0);
        for (v, &w) in m.verts.iter().zip(&m.pesos) {
            lo = lo.min(w);
            if w > hi {
                hi = w;
                xhi = v[0];
            }
        }
        println!(
            "Bone {:<4} {lo:>8.3} {hi:>8.3} {xhi:>10.2} {:>10.3}",
            k + 1,
            hi - lo
        );
    }
    println!(
        "{:-<64}\na barra vive em x ∈ [-8,5 ; -1,5]; o canvas corta em ~x = -6,5",
        ""
    );
}

/// ⛔⛔⛔ **A RECUSA MEDIDA: ancorar a mancha NO DEDO deixa o pincel de fábrica INERTE.**
///
/// # O report e o que ele parecia ser
///
/// *«a malha apareceu mas os pesos não estão nos vértices da malha. e não houve melhora na
/// deformação»* (dono, 2026-09-20, com foto). A leitura óbvia — e a que eu segui — é que a mancha
/// devia pousar **onde o dedo aponta**, e não no contorno: medido pela porta do produto, o dedo em
/// cima de um vértice do MIOLO guardava a correcção até **`0,4340`** dali, com um pincel de raio
/// **`0,400`** — *a mancha aterrava mais longe do que ela própria alcança*.
///
/// A cura foi construída (baricêntricas no triângulo POSADO do retículo, que dão a inversa exacta
/// do mapa e põem o desvio em `0,0000` nas cinco células) e **a medição do DESENHO refutou-a**:
///
/// | raio do pincel | mancha no RETÍCULO | mancha no CONTORNO (o produto) |
/// |---|---:|---:|
/// | **`0,40`** (fábrica) | **`0,00000`** | `0,49921` |
/// | `0,80` | `0,30967` | `0,50600` |
/// | `1,20` | `0,55686` | `0,65708` |
///
/// # ⭐⭐⭐ O mecanismo, e é ele que fecha o assunto
///
/// **Numa forma vectorial o desenho é o CONTORNO.** O interior do retículo é andaime: o campo
/// vive lá, mas não há arte para mover. A barra tem meia-altura `0,5` contra um pincel de `0,40`
/// ⇒ uma mancha centrada no meio **não chega a nenhuma das duas bordas**, e o pincel de fábrica
/// ficaria mudo exactamente onde o artista arrasta — que é um report PIOR do que o que a motivou.
///
/// ⚠️ **O `onde_pousa` já escrevia esta lei** (*«no caminho, estar DENTRO da forma também conta …
/// e é pelo miolo que o artista arrasta»*), e a projecção no contorno **é** o que faz um pincel
/// pequeno alcançar a arte a partir do miolo. *Não era um atalho: era a lei.*
///
/// ⚠️ **Fica NOMEADO o que a projecção custa:** ela escolhe a borda MAIS PERTO, logo um arrasto que
/// cruze a linha média da barra salta da borda de baixo para a de cima (medido: `y = 2,343 → 2,000`
/// e `y = 2,566 → 3,000`). Com a fusão de manchas o resultado é as duas bordas ficarem pintadas,
/// que é o que se quer numa barra — mas quem for atrás de *«irregularidades»* deve medir isto
/// primeiro.
///
/// ⛔ O instrumento fica aqui, atrás do `cfg(test)`, porque ele é a prova da recusa — ⚠️ e uma
/// porta pública sem consumidor de produto é a *lei viva e órfã* que o `CLAUDE.md` já nomeia.
fn ancora_no_reticulo(
    m: &super::MalhaDoPeso,
    repousos: &[[f64; 2]],
    mundo: [f64; 2],
) -> Option<[f64; 2]> {
    for t in &m.tris {
        let (i, j, k) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let (a, b, c) = (*m.verts.get(i)?, *m.verts.get(j)?, *m.verts.get(k)?);
        let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if area.abs() <= f64::EPSILON {
            continue;
        }
        let l1 =
            ((b[0] - mundo[0]) * (c[1] - mundo[1]) - (b[1] - mundo[1]) * (c[0] - mundo[0])) / area;
        let l2 =
            ((c[0] - mundo[0]) * (a[1] - mundo[1]) - (c[1] - mundo[1]) * (a[0] - mundo[0])) / area;
        let l3 = 1.0 - l1 - l2;
        if l1 < -1e-9 || l2 < -1e-9 || l3 < -1e-9 {
            continue;
        }
        let (ra, rb, rc) = (*repousos.get(i)?, *repousos.get(j)?, *repousos.get(k)?);
        return Some([
            l3.mul_add(rc[0], l2.mul_add(rb[0], l1 * ra[0])),
            l3.mul_add(rc[1], l2.mul_add(rb[1], l1 * ra[1])),
        ]);
    }
    None
}

/// **Os repousos dos vértices do retículo** — o par de [`super::MalhaDoPeso::verts`], lido do campo.
fn repousos_do_reticulo(sim: &ph2d_ecs::SimWorld, alvo: ph2d_ecs::Entity) -> Vec<[f64; 2]> {
    let skin = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(alvo)
        .expect("pele")
        .clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let campo = g.campo.as_ref().expect("campo");
    (0..campo.malha.rest.len())
        .map(|i| campo.local_do_vertice(i).expect("vértice"))
        .collect()
}

/// ⚠️⚠️ **SONDA — ONDE MORAM OS PESOS, e onde uma pincelada POUSA** (report do dono, 2026-09-20:
/// *«a malha apareceu mas os pesos não estão nos vértices da malha»*).
///
/// Três perguntas que o olho lê como uma:
///
/// 1. **quantos** pontos de peso (o que o artista vê como bolinha) contra vértices de retículo;
/// 2. **onde** — quantos vértices de retículo SÃO uma bolinha;
/// 3. **onde a mancha CAI** com o dedo em cima de um vértice do MIOLO, pela porta do produto,
///    com a coluna do que a [`ancora_no_reticulo`] daria ao lado (ver a recusa medida ali).
#[test]
fn diag_onde_moram_os_pesos() {
    use crate::peso_a_mao::{mundo_e_escala, pinta, pontos_de_peso};
    use ph2d_skeleton_ecs::SkinBind;

    let (mut sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    let osso = ossos[1];
    let m = malha_do_peso(&sim, alvo, osso).expect("retículo");
    let repousos = repousos_do_reticulo(&sim, alvo);
    let pontos = pontos_de_peso(&sim, alvo, osso, PPM);

    println!("\n{:-<84}", "");
    println!("vértices do retículo ..... {}", m.verts.len());
    println!("pontos de peso (bolinhas) . {}", pontos.len());
    let coincidem = m
        .verts
        .iter()
        .filter(|v| {
            pontos
                .iter()
                .any(|p| (v[0] - p.mundo[0]).hypot(v[1] - p.mundo[1]) < 1e-6)
        })
        .count();
    println!(
        "vértices que SÃO uma bolinha: {coincidem} de {}",
        m.verts.len()
    );

    let (x, escala) = mundo_e_escala(&sim, alvo);
    let raio = raio_de_fabrica();
    println!("{:-<84}", "");
    println!(
        "{:>4} {:>18} {:>18} {:>9} {:>11}",
        "i", "dedo (local)", "mancha (local)", "desvio", "retículo"
    );
    for i in [40usize, 120, 200, 260, 330] {
        let Some(local) = repousos.get(i).copied() else {
            continue;
        };
        let mundo = x.apply(local);
        let pelo_reticulo = ancora_no_reticulo(&m, &repousos, m.verts[i])
            .map_or(f64::NAN, |r| (r[0] - local[0]).hypot(r[1] - local[1]));
        let _ = pinta(
            &mut sim,
            alvo,
            osso,
            PPM,
            mundo,
            raio,
            ph2d_skeleton::Especie::Soma(0.2),
        );
        let depois = sim.world().get::<SkinBind>(alvo).expect("pele").clone();
        let Some(c) = depois.correcoes.last() else {
            println!("{i:>4}  nada guardado");
            continue;
        };
        let d = (c.centro[0] - local[0]).hypot(c.centro[1] - local[1]);
        println!(
            "{i:>4} [{:>7.3},{:>7.3}] [{:>7.3},{:>7.3}] {d:>9.4} {pelo_reticulo:>11.4}",
            local[0], local[1], c.centro[0], c.centro[1]
        );
    }
    println!(
        "{:-<84}\nraio do pincel = {raio:.3} mundo ({:.3} local, escala {escala:.3}); \
         a meia-altura da barra e 0,5",
        "",
        raio / escala
    );
}

/// ⛔⛔⛔ **SONDA — a pincelada no MIOLO chega ao DESENHO?** É esta que REFUTA a cura óbvia; a
/// tabela e o mecanismo vivem no doc da [`ancora_no_reticulo`].
#[test]
fn diag_a_pincelada_no_miolo_chega_ao_desenho() {
    use crate::peso_a_mao::{posados, repousos};
    use ph2d_ecs::{SimWorld, Transform};
    use ph2d_skeleton_ecs::{CorreccaoDePeso, SkinBind};

    /// A barra JÁ DOBRADA — uma árvore nova por caso, porque o mundo não se clona.
    fn dobrada() -> (
        SimWorld,
        ph2d_vec_scene::VecScene,
        ph2d_vec_scene::VecPathId,
        ph2d_ecs::Entity,
        Vec<ph2d_ecs::Entity>,
    ) {
        let (mut sim, scene, map, id, ossos) = barra_da_cena();
        let alvo = forma(&map, id);
        for o in &ossos[1..] {
            sim.world_mut()
                .get_mut::<Transform>(*o)
                .expect("Transform")
                .rotation = 45.0f32.to_radians();
        }
        (sim, scene, id, alvo, ossos)
    }
    let desenha = |sim: &SimWorld, scene: &ph2d_vec_scene::VecScene, id| {
        let mut s = scene.clone();
        crate::skin_live::recook_com_mistura(sim, &mut s, true, true, true);
        s.paths().iter().find(|p| p.id == id).expect("path").clone()
    };
    // A mesma pincelada, com o CENTRO dado — é o centro que as duas leis disputam.
    let com_mancha = |centro: [f64; 2], raio: f64| {
        let (mut sim, scene, id, alvo, ossos) = dobrada();
        let bone = *sim
            .world()
            .get::<ph2d_ecs::StableId>(ossos[2])
            .expect("id do osso");
        let mut skin = sim.world().get::<SkinBind>(alvo).expect("pele").clone();
        skin.correcoes.push(CorreccaoDePeso {
            bone,
            centro,
            raio,
            especie: ph2d_skeleton::Especie::Alvo(1.0),
        });
        sim.world_mut().entity_mut(alvo).insert(skin);
        desenha(&sim, &scene, id)
    };

    let (sim, scene, id, alvo, ossos) = dobrada();
    let sem = desenha(&sim, &scene, id);
    let m = malha_do_peso(&sim, alvo, ossos[2]).expect("retículo");
    let reps = repousos_do_reticulo(&sim, alvo);
    // O dedo: o meio da barra (x = -5, y = 2,5), levado à pose.
    let i = reps
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let f = |p: &[f64; 2]| (p[0] + 5.0).hypot(p[1] - 2.5);
            f(a).partial_cmp(&f(b)).expect("finito")
        })
        .map(|(i, _)| i)
        .expect("um vértice");
    let dedo = m.verts[i];
    let no_reticulo = ancora_no_reticulo(&m, &reps, dedo).expect("dentro de um triângulo");
    let no_contorno = crate::ancora_da_mancha::no_contorno(
        &repousos(&sim, alvo, PPM),
        &posados(&sim, alvo, PPM),
        dedo,
    )
    .expect("contorno")
    .repouso;

    println!("\n{:-<76}", "");
    println!(
        "dedo (repouso) .... [{:.3}, {:.3}]",
        no_reticulo[0], no_reticulo[1]
    );
    println!(
        "contorno diz ...... [{:.3}, {:.3}]",
        no_contorno[0], no_contorno[1]
    );
    println!("{:-<76}", "");
    println!(
        "{:>8} {:>18} {:>20} {:>14}",
        "raio", "no retículo", "no contorno (hoje)", "razão"
    );
    for raio in [0.4_f64, 0.8, 1.2] {
        let a = crate::test_support::pior_desvio_do_desenho(&sem, &com_mancha(no_reticulo, raio));
        let b = crate::test_support::pior_desvio_do_desenho(&sem, &com_mancha(no_contorno, raio));
        println!(
            "{raio:>8.2} {a:>18.5} {b:>20.5} {:>13.2}x",
            if b > 1e-12 { a / b } else { f64::INFINITY }
        );
    }
    println!(
        "{:-<76}\nnuma forma vectorial o desenho e o CONTORNO: o miolo do reticulo e andaime",
        ""
    );
}

/// ⚠️⚠️ **SONDA — o retículo pintado por TRIÂNGULO perde quanto?** O desenho enche cada triângulo
/// com a MÉDIA dos três cantos, logo a cor é constante por pedaço: se o peso variar muito DENTRO
/// de um triângulo, o que o artista vê não é o peso do vértice — é uma banda.
#[test]
fn diag_o_reticulo_pinta_por_triangulo_e_nao_por_vertice() {
    let (sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    println!("\n{:-<80}", "");
    println!(
        "{:<10} {:>10} {:>14} {:>14} {:>12}",
        "osso", "triâng.", "salto p50", "salto p99", "salto máx"
    );
    println!("{:-<80}", "");
    for (k, &o) in ossos.iter().enumerate() {
        let Some(m) = malha_do_peso(&sim, alvo, o) else {
            continue;
        };
        let mut saltos: Vec<f64> = m
            .tris
            .iter()
            .filter_map(|t| {
                let w: Vec<f64> = t
                    .iter()
                    .map(|&i| m.pesos.get(i as usize).copied().unwrap_or(0.0))
                    .collect();
                let lo = w.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = w.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                (hi - lo).is_finite().then_some(hi - lo)
            })
            .collect();
        saltos.sort_by(|a, b| a.partial_cmp(b).expect("finito"));
        let q = |f: f64| saltos[((saltos.len() - 1) as f64 * f) as usize];
        println!(
            "Bone {:<5} {:>10} {:>14.4} {:>14.4} {:>12.4}",
            k + 1,
            saltos.len(),
            q(0.5),
            q(0.99),
            saltos.last().copied().unwrap_or(0.0)
        );
    }
    println!(
        "{:-<80}\no peso vive em 0..1; um salto de 0,10 dentro de um triangulo sao ~25 de 255 na cor",
        ""
    );
}
