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
