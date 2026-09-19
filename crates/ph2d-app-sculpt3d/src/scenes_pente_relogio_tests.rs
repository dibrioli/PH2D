//! ⭐⭐⭐⭐ **A CAÇA AO RELÓGIO DO PENTE** — as sondas que mediram as cinco
//! tentativas de optimização e a única que sobreviveu.
//!
//! Report do dono (21/09): *«algoritmo mais lento que o modo padrão. tem que
//! otimizar»*. A tabela das cinco, com o mecanismo de cada recusa, vive na
//! constante que é a alavanca viva ([`crate::dyntopo::ALTERNANCIAS`]).
//!
//! ⚠️ Ele saiu do irmão por **TECTO DE LOC** (`773` contra `700`).

use super::*;

/// ⭐⭐⭐⭐ **SONDA — quanto de cada varredura é TRABALHO NOVO.**
///
/// A hipótese: entre dois dabs o pincel anda `0,15` raios, logo **`92,5 %` da
/// pegada já foi penteada** e o campo dela já está no ponto fixo. Se for assim,
/// as varreduras seguintes quase não movem nada e o relax quase não troca nada
/// — e o custo é todo em **re-derivar uma resposta que já se tinha**.
///
/// Ela mede, dab a dab: quanto a retícula pede (em passos) e quantas trocas o
/// relax faz.
#[test]
#[ignore = "sonda: imprime o trabalho novo por dab, nao afirma nada"]
fn diag_quanto_de_cada_dab_e_trabalho_novo() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let mut malha = peca_uma_vez();
    malha.triangulate();
    let brush = ph2d_sculpt3d::Brush {
        verb: ph2d_sculpt3d::Verb::Draw,
        radius: raio,
        strength: 0.25,
        pente: 0.0,
        ..ph2d_sculpt3d::Brush::default()
    };
    let mut stroke = ph2d_sculpt3d::SculptStroke::default();
    stroke.begin(&malha);
    let (mut births, mut region) = (Vec::new(), ph2d_mesh::RegionScratch::default());
    let mut remap = ph2d_mesh::Remap::default();
    let passo_t = raio * 0.15;
    println!("dab  pegada  pedido_p50  trocas_do_relax");
    for k in 0..12 {
        let u = -passo_t * 12.0 + passo_t * k as f32;
        let centro = [u.sin() * RUMOS[0].1[0], u.sin() * RUMOS[0].1[1], u.cos()];
        let direccao = stroke.direccao_do_traco(centro);

        // A medição, ANTES de o passe correr: o que a retícula ainda pede e
        // quantas trocas o relax ainda tem.
        let mut ids = Vec::new();
        {
            let mut faces = Vec::new();
            malha.octree().faces_in_sphere(centro, raio, &mut faces);
            let r2 = raio * raio;
            let pos = malha.positions();
            let todas = malha.faces();
            for &f in &faces {
                for &v in todas[f as usize].verts() {
                    let q = pos[v as usize];
                    let d = [q[0] - centro[0], q[1] - centro[1], q[2] - centro[2]];
                    if d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])) <= r2 {
                        ids.push(v);
                    }
                }
            }
            ids.sort_unstable();
            ids.dedup();
        }
        let (pedido, trocas) = if ids.is_empty() || direccao == [0.0; 3] {
            (0.0, 0)
        } else {
            let passo = ph2d_quadflow::regiao::passo_da_pegada(&malha, &ids) * LADO_DA_CELULA;
            let m = ph2d_quadflow::regiao::mancha(&malha, &ids);
            let p50 = ph2d_quadflow::regiao::retrato_da_pilha(
                &m,
                direccao,
                passo,
                RONDAS_DA_GRELHA,
                m.len() + 1,
            )
            .1;
            let mut copia = malha.clone();
            let mut sc = ph2d_mesh::RegionScratch::default();
            let t = ph2d_mesh::relaxa_valencia_em(&mut copia, centro, raio, &mut sc);
            (p50, t)
        };
        println!("{k:3} {:7} {pedido:11.4} {trocas:16}", ids.len());

        let (cut, done, _) = crate::dyntopo::passe_nos_motores(
            &mut malha,
            brush.verb,
            alvo,
            centro,
            raio,
            crate::dyntopo::Rascunho {
                remap: &mut remap,
                births: &mut births,
                region: &mut region,
            },
            Some(crate::dyntopo::Pente {
                direccao,
                forca: 1.0,
                queda: brush.falloff,
            }),
        );
        if cut {
            stroke.shrink_with(&remap);
        }
        if done {
            stroke.grow_with(&malha, &births);
        }
        stroke.dab(
            &mut malha,
            &brush,
            &ph2d_sculpt3d::Dab::at(centro, raio, [0.0, 0.0, -1.0]),
            ph2d_sculpt3d::Symmetry::default(),
        );
    }
}

/// ⭐⭐⭐⭐ **SONDA — a GRELHA das rondas × alternâncias, com o relógio.**
///
/// Report do dono (21/09): *«algoritmo mais lento que o modo padrão»*. As duas
/// escadas que escolheram `4` e `2` correram **antes** de a troca de ligação
/// entrar no passe — e ela faz parte do trabalho. *Uma escada corrida antes de
/// a composição mudar responde sobre outro produto* (§0.0).
#[test]
#[ignore = "sonda: imprime a grelha, nao afirma nada"]
fn diag_a_grelha_das_rondas_e_alternancias() {
    use std::time::Instant;
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    println!("rondas alt raio   grade%   vinco_p90   fil50 fil90   4bracos%   ms/traco");
    for (rondas, alt, raio_k) in [
        (4usize, 2usize, 1.00f32),
        (4, 1, 1.00),
        (4, 1, 0.90),
        (4, 1, 0.80),
        (4, 1, 0.70),
        (4, 2, 0.80),
    ] {
        crate::dyntopo::RONDAS_DO_TESTE.with(|c| c.set(rondas));
        crate::dyntopo::ALTERNANCIAS_DO_TESTE.with(|c| c.set(alt));
        crate::dyntopo::RAIO_DO_TESTE.with(|c| c.set(raio_k));
        let (mut g, mut v90, mut f50, mut f90) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        let (mut quatro, mut n) = (0usize, 0usize);
        let mut relogio = f64::INFINITY;
        for (_, e) in RUMOS.iter() {
            let t = Instant::now();
            let (m, c) = super::super::traco_com(1.0, *e, raio, alvo);
            relogio = relogio.min(t.elapsed().as_secs_f64() * 1000.0);
            let (bal, nb) = grade_da_faixa(&m, &c, raio);
            g += 100.0 * bal[0] as f64 / nb.max(1) as f64;
            let (_, b, _, _) = vinco_da_faixa(&m, &c, raio);
            v90 += b;
            let (x, y, _, _) = ph2d_sculpt3d::medida_da_fileira::fileira_da_faixa(&m, &c, raio);
            f50 += x;
            f90 += y;
            let grau = ph2d_mesh_render::bracos_na_vista_da_grade(&m);
            let pos = m.positions();
            for (v, q) in pos.iter().enumerate() {
                let mut d2 = f32::INFINITY;
                for t in &c {
                    let w = [q[0] - t[0], q[1] - t[1], q[2] - t[2]];
                    d2 = d2.min(w[0].mul_add(w[0], w[1].mul_add(w[1], w[2] * w[2])));
                }
                if d2.sqrt() > raio * 0.5 {
                    continue;
                }
                n += 1;
                if grau[v] == 4 {
                    quatro += 1;
                }
            }
        }
        crate::dyntopo::RONDAS_DO_TESTE.with(|c| c.set(0));
        crate::dyntopo::ALTERNANCIAS_DO_TESTE.with(|c| c.set(0));
        crate::dyntopo::RAIO_DO_TESTE.with(|c| c.set(0.0));
        println!(
            "{rondas:6} {alt:3} {raio_k:5.2}  {:7.2}   {:9.3}   {:5.1} {:5.1}   {:8.2}   {relogio:8.1}",
            g / 4.0,
            v90 / 4.0,
            f50 / 4.0,
            f90 / 4.0,
            100.0 * quatro as f64 / n.max(1) as f64
        );
    }
}
