//! Gates da RESOLUÇÃO da malha do prisma — ver [`Resolucao`].
//!
//! Filho (`#[path]`) do [`super`], e os ajudantes de fixtura são os dele.
//!
//! # Porque este assunto tem ficheiro próprio
//!
//! Report do dono, 2026-09-15, com foto: *«o remesh da face que você cortou fica
//! ruim demais»*. A cura é a lâmina nascer tesselada, e as réguas dela medem uma
//! grandeza que os gates da forma não tocam — *a densidade da malha que o corte
//! DEIXA*, e não o volume que ele tira.

use super::*;
use ph2d_mesh::{Mesh, shapes};

// ─────────────────────────────────────────────────────────────────────────────
// A RESOLUÇÃO (report do dono, 2026-09-15: «o remesh da face que você cortou
// fica ruim demais»). Ver [`Resolucao`].
// ─────────────────────────────────────────────────────────────────────────────

/// Todas as arestas de uma malha de triângulos.
fn arestas_de(m: &Mesh) -> Vec<f32> {
    let p = m.positions();
    let mut tris = Vec::new();
    for f in m.faces() {
        f.triangles(&mut tris);
    }
    let d = |a: u32, b: u32| {
        let (a, b) = (p[a as usize], p[b as usize]);
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    tris.iter()
        .flat_map(|t| [d(t[0], t[1]), d(t[1], t[2]), d(t[2], t[0])])
        .collect()
}

fn com(anel: &[[f32; 2]], r: Resolucao, paredes: Paredes) -> Mesh {
    let bola = shapes::uv_sphere(12, 16, 1.0);
    prisma(
        anel,
        &raios_orto(anel),
        &plano(),
        &bola,
        Profundidade::DaPeca,
        paredes,
        r,
    )
    .expect("o prisma")
}

/// ⭐⭐⭐ **ADENSAR NÃO MOVE A SUPERFÍCIE** — a propriedade que torna esta lei
/// segura de ligar.
///
/// As paredes são **regradas** e as tampas **planas**, então todo ponto novo é
/// uma interpolação *na própria superfície*. ⇒ o volume encerrado é o mesmo, e
/// a caixa também.
///
/// ⚠️ Sem isto, a cura do report viraria uma mudança de FORMA do corte — e um
/// corte que muda de forma conforme a densidade não é uma ferramenta.
#[test]
fn adensar_nao_move_a_superficie() {
    for anel in [caixa(), ce()] {
        for paredes in [Paredes::Fixas, Paredes::Projectadas] {
            let grosso = com(&anel, Resolucao::Minima, paredes);
            let fino = com(&anel, Resolucao::Ate(0.05), paredes);
            assert!(
                fino.face_count() > 20 * grosso.face_count(),
                "a fixtura tem de de facto adensar: {} contra {}",
                fino.face_count(),
                grosso.face_count()
            );
            let (vg, vf) = (volume_com_sinal(&grosso), volume_com_sinal(&fino));
            assert!(
                (vg - vf).abs() <= 1e-4 * vg.abs().max(1e-6),
                "o volume mudou ao adensar: {vg} contra {vf}"
            );
            let (bg, bf) = (grosso.bounds(), fino.bounds());
            for i in 0..3 {
                assert!(
                    (bg.min[i] - bf.min[i]).abs() < 1e-5 && (bg.max[i] - bf.max[i]).abs() < 1e-5,
                    "a caixa mudou no eixo {i}: {:?} contra {:?}",
                    bg.min,
                    bf.min
                );
            }
        }
    }
}

/// **Nenhuma aresta das PAREDES passa do alvo** — é o que a [`Resolucao::Ate`]
/// promete, e é a densidade que a face cortada herda.
///
/// ⚠️ **A metade de baixo é a DÍVIDA, medida e visível:** o interior das tampas
/// fica grosso, e este gate **afirma-o** em vez de o esconder — no dia em que
/// alguém triangular a tampa com pontos interiores, esta metade reprova e a
/// dívida sai do doc da [`Resolucao::Ate`] no mesmo diff.
#[test]
fn nenhuma_aresta_das_paredes_passa_do_alvo() {
    for alvo in [0.4f32, 0.1, 0.05] {
        for anel in [caixa(), ce()] {
            let m = com(&anel, Resolucao::Ate(alvo), Paredes::Fixas);
            let p = m.positions();
            let mut tris = Vec::new();
            for f in m.faces() {
                f.triangles(&mut tris);
            }
            // Uma tampa é plana no eixo do varrimento (aqui `+z`, vista
            // ortográfica): os três vértices à mesma profundidade.
            let e_tampa = |t: &[u32; 3]| {
                let z = |i: u32| p[i as usize][2];
                (z(t[0]) - z(t[1])).abs() < 1e-5 && (z(t[1]) - z(t[2])).abs() < 1e-5
            };
            let d = |a: u32, b: u32| {
                let (a, b) = (p[a as usize], p[b as usize]);
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
            };
            let (mut paredes, mut tampas) = (0.0f32, 0.0f32);
            for t in &tris {
                let maior = d(t[0], t[1]).max(d(t[1], t[2])).max(d(t[2], t[0]));
                if e_tampa(t) {
                    tampas = tampas.max(maior);
                } else {
                    paredes = paredes.max(maior);
                }
            }
            // ⚠️ A diagonal de um quad da grelha é `√2 ×` o lado — é uma aresta
            // legítima de uma tesselação cujo PASSO é o alvo.
            assert!(
                paredes <= alvo * 1.5,
                "alvo {alvo}: a maior aresta de PAREDE é {paredes}"
            );
            assert!(
                tampas > alvo * 1.5,
                "alvo {alvo}: as tampas deixaram de ser grossas ({tampas}) — \
                 se isso foi de propósito, apague esta metade E a dívida escrita \
                 no doc da `Resolucao::Ate`"
            );
        }
    }
}

/// ⛔⛔ **O PRISMA ADENSADO CONTINUA A ENCERRAR VOLUME** — e o caso que isto
/// existe para apanhar é a **junta em T**.
///
/// Um ponto que o adensamento põe numa aresta do anel pertence às paredes; se a
/// tampa continuasse a ir de canto a canto, a malha ficava com uma junta em T e
/// o motor lê-a como superfície **ABERTA** (medido nesta casa: `LaminaAberta`,
/// e uma lâmina aberta não corta nada).
///
/// ⚠️ **O anel IRREGULAR é o sujeito**: com segmentos de comprimentos
/// diferentes cada um parte-se num número diferente de pedaços, que é onde uma
/// costura escrita à mão falha.
#[test]
fn o_prisma_adensado_continua_fechado() {
    let irregular = vec![
        [-0.9, -0.5],
        [0.7, -0.5],
        [0.75, -0.1],
        [0.1, 0.0],
        [0.8, 0.45],
        [-0.9, 0.5],
    ];
    for anel in [caixa(), ce(), irregular] {
        for alvo in [0.37f32, 0.11, 0.043] {
            let m = com(&anel, Resolucao::Ate(alvo), Paredes::Fixas);
            assert_eq!(
                ph2d_mesh::border_edges(&m),
                0,
                "anel de {} pontos, alvo {alvo}: o prisma abriu",
                anel.len()
            );
            assert!(
                volume_com_sinal(&m) > 0.0,
                "o enrolamento saiu invertido com alvo {alvo}"
            );
        }
    }
}

/// **O tecto RENORMALIZA e nunca recusa** — ver [`TECTO_DE_TRIANGULOS`].
///
/// ⚠️ A metade que interessa é a segunda: um alvo absurdo tem de devolver um
/// prisma **grosso**, nunca um erro — *o gesto do artista não se perde porque
/// quem chamou derivou um número mau*.
#[test]
fn um_alvo_absurdo_engrossa_em_vez_de_recusar() {
    for alvo in [1e-6f32, 1e-9, f32::MIN_POSITIVE] {
        let m = com(&caixa(), Resolucao::Ate(alvo), Paredes::Fixas);
        assert!(
            m.face_count() <= TECTO_DE_TRIANGULOS + 4096,
            "alvo {alvo}: {} faces, acima do tecto",
            m.face_count()
        );
        assert_eq!(ph2d_mesh::border_edges(&m), 0, "alvo {alvo}: abriu");
    }
    // Um alvo não-positivo ou não-finito é o pedido de NADA, e a lâmina mínima
    // é exactamente isso — ⛔ não é uma recusa.
    for alvo in [0.0f32, -1.0, f32::NAN] {
        let m = com(&caixa(), Resolucao::Ate(alvo), Paredes::Fixas);
        assert_eq!(
            m.face_count(),
            com(&caixa(), Resolucao::Minima, Paredes::Fixas).face_count(),
            "alvo {alvo} tinha de cair na lâmina mínima"
        );
    }
}

/// ⭐⭐⭐ **A PROVA DE PONTA A PONTA DO REPORT — a face que o corte DEIXA tem a
/// densidade da peça.**
///
/// Report do dono, 2026-09-15, com foto: *«o remesh da face que você cortou fica
/// ruim demais»*. Medido então: a face cortada saía com **DOIS** triângulos de
/// `1,2` de aresta numa peça cuja aresta mediana é `0,035` — `34 ×` mais grossa.
///
/// ⚠️ **As duas metades são obrigatórias.** A de baixo é o CONTROLO: ela corre a
/// mesma cadeia com [`Resolucao::Minima`] e afirma que a face sai grossa ⇒ *se
/// alguém tornar o adensamento inerte, a metade de cima reprova; se alguém
/// tornar a régua cega, a de baixo reprova.* Sem o controlo, uma régua que
/// medisse a peça inteira em vez da face cortada ficaria verde sobre o defeito.
#[test]
fn a_face_que_o_corte_deixa_tem_a_densidade_da_peca() {
    let bola = shapes::sphere_with_triangles(20_000, 1.0);
    let mut da_peca = arestas_de(&bola);
    da_peca.sort_by(f32::total_cmp);
    let alvo = da_peca[da_peca.len() / 2];

    let anel = caixa();
    let corta = |r: Resolucao| {
        let lamina = prisma(
            &anel,
            &raios_orto(&anel),
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            r,
        )
        .expect("o prisma");
        let out =
            ph2d_mesh_bool::corta(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair).expect("o corte");
        // A face cortada são as PAREDES do prisma (`x = ±0,5`, `y = ±0,5`)
        // recortadas pela peça — com `DaPeca` as tampas ficam fora dela.
        let p = out.positions();
        let mut tris = Vec::new();
        for f in out.faces() {
            f.triangles(&mut tris);
        }
        let na_parede = |i: u32| {
            let v = p[i as usize];
            (v[0].abs() - 0.5).abs() < 1e-4 || (v[1].abs() - 0.5).abs() < 1e-4
        };
        let d = |a: u32, b: u32| {
            let (a, b) = (p[a as usize], p[b as usize]);
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        };
        let mut e: Vec<f32> = tris
            .iter()
            .filter(|t| t.iter().all(|&i| na_parede(i)))
            .flat_map(|t| [d(t[0], t[1]), d(t[1], t[2]), d(t[2], t[0])])
            .collect();
        assert!(!e.is_empty(), "a régua não achou a face cortada");
        e.sort_by(f32::total_cmp);
        (e.len() / 3, e[e.len() / 2])
    };

    let (tris_fino, aresta_fina) = corta(Resolucao::Ate(alvo));
    assert!(
        aresta_fina <= alvo * 2.0,
        "a face cortada saiu grossa: aresta {aresta_fina} contra {alvo} da peça"
    );

    let (tris_grosso, aresta_grossa) = corta(Resolucao::Minima);
    assert!(
        aresta_grossa >= alvo * 10.0,
        "CONTROLO: com a lâmina mínima a face tinha de sair grossa, e mede {aresta_grossa}"
    );
    // ⚠️ **A contagem grossa NÃO é `2`, e isso é o motor a trabalhar:** a parede
    // é recortada pela esfera, logo a fronteira dela é uma curva com muitos
    // vértices e o motor tapa-a com um leque. Medido: `436` triângulos de
    // aresta `~0,5` contra `11 380` de aresta `~0,035`. ⇒ *contar triângulos
    // não distingue uma face tesselada de uma face com um leque grande* — é a
    // ARESTA que decide, e esta metade é só a confirmação grosseira.
    assert!(
        tris_fino >= 10 * tris_grosso,
        "CONTROLO: {tris_fino} contra {tris_grosso} triângulos na face cortada"
    );
    println!(
        "face cortada: FINA {tris_fino} tri · aresta {aresta_fina:.4}  |  \
         GROSSA {tris_grosso} tri · aresta {aresta_grossa:.4}  |  peça {alvo:.4}"
    );
}

/// ⛔⛔ **NENHUMA FACE DO PRISMA TEM ÁREA ZERO** — a propriedade que o leque da
/// tampa existe para ter.
///
/// Os pontos que o adensamento põe numa aresta do anel são **colineares** com
/// os cantos dela, logo um leque a partir de um CANTO emite triângulos
/// degenerados. Eles não abrem a malha e não mudam o volume — ⚠️ *o gate de
/// fecho e o de volume ficam os dois verdes sobre eles* —, e é por isso que
/// esta régua existe: uma face sem área é uma face **sem direcção**
/// ([`ph2d_mesh::face_normal`] devolve o vector nulo), e entregá-la a um
/// solucionador exacto é pedir a resposta que ninguém mediu.
///
/// ⚠️ **Nasceu de uma MUTAÇÃO SOBREVIVENTE:** o doc da [`tampa`] já afirmava
/// isto por escrito, e trocar o centro por um canto passava a suíte inteira.
#[test]
fn nenhuma_face_do_prisma_tem_area_zero() {
    for anel in [caixa(), ce()] {
        for r in [Resolucao::Minima, Resolucao::Ate(0.07)] {
            let m = com(&anel, r, Paredes::Fixas);
            let p = m.positions();
            let mut tris = Vec::new();
            for f in m.faces() {
                f.triangles(&mut tris);
            }
            let mut menor = f32::INFINITY;
            for t in &tris {
                let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
                let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let n = [
                    u[1] * v[2] - u[2] * v[1],
                    u[2] * v[0] - u[0] * v[2],
                    u[0] * v[1] - u[1] * v[0],
                ];
                menor = menor.min(0.5 * (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt());
            }
            assert!(
                menor > 1e-9,
                "anel de {} pontos, {r:?}: há face de área {menor}",
                anel.len()
            );
        }
    }
}

#[test]
#[ignore = "sonda: com que peça a cena do Box Trim pode abrir"]
fn diag_o_relogio_do_corte_por_peca() {
    let cubo3 = {
        let c = shapes::cube(1.2);
        ph2d_mesh::subdivide(&ph2d_mesh::subdivide(&ph2d_mesh::subdivide(&c)))
    };
    for (nome, bola) in [
        ("cubo subdividido 3×", cubo3),
        (
            "sphere_with_triangles(20k)",
            shapes::sphere_with_triangles(20_000, 1.0),
        ),
        (
            "sphere_with_triangles(50k)",
            shapes::sphere_with_triangles(50_000, 1.0),
        ),
        ("sculpt_sphere (o default)", shapes::sculpt_sphere(1.0)),
    ] {
        let tris: usize = bola
            .faces()
            .iter()
            .map(|f| f.verts().len().saturating_sub(2))
            .sum();
        let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), tris as f32);
        let anel = caixa();
        let t0 = std::time::Instant::now();
        let lamina = prisma(
            &anel,
            &raios_orto(&anel),
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            Resolucao::Ate(alvo),
        )
        .expect("o prisma");
        let t_lam = t0.elapsed().as_secs_f64() * 1e3;
        let t1 = std::time::Instant::now();
        let out =
            ph2d_mesh_bool::corta(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair).expect("o corte");
        println!(
            "{nome:28} V={:7} T={:7} alvo={alvo:.4} | lâmina {:6} T em {t_lam:6.1} ms | \
             corte {:7.1} ms | saída T={:7}",
            bola.positions().len(),
            tris,
            lamina.face_count(),
            t1.elapsed().as_secs_f64() * 1e3,
            out.face_count(),
        );
    }
}

/// ⭐⭐⭐ **A RÉGUA DO QUE O DONO VÊ: a face cortada é SOMBREADA COMO PLANA.**
///
/// # Porque as outras réguas não bastam
///
/// As de cima medem contagens e comprimentos de aresta — grandezas da MALHA. O
/// que a foto do report mostra é outra coisa: uma face **plana** com o
/// sombreado a escorrer de um canto ao outro, como se fosse curva. ⚠️ *Uma face
/// pode ter a topologia certa e ainda assim ser pintada errada*, e o mecanismo
/// é o sombreamento suave: a normal de um vértice é a média das faces que o
/// tocam, logo um vértice na BORDA do corte mistura o plano com a esfera.
///
/// ⭐ **Com dois triângulos, TODOS os vértices da face são de borda** ⇒ não
/// existe um único ponto dela que seja pintado plano, e a face inteira é um
/// degradé. Com a face tesselada, os vértices do MIOLO têm as faces todas
/// coplanares ⇒ a normal deles é **exactamente** a do plano, e a mistura fica
/// numa banda de um triângulo junto à aresta — que é o corte nítido da
/// referência.
///
/// ⚠️ **A lei de sombreamento aqui é a MESMA do produto** (a média das normais
/// das faces incidentes, `ph2d_mesh::vertex_normals_of`) — escrita à mão porque
/// a do produto pede a adjacência montada, e o que interessa é a grandeza.
#[test]
fn a_face_cortada_e_sombreada_como_plana() {
    let bola = shapes::sphere_with_triangles(20_000, 1.0);
    let tris_da_peca: usize = bola
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), tris_da_peca as f32);
    let anel = caixa();

    let fraccao_plana = |r: Resolucao| {
        let lamina = prisma(
            &anel,
            &raios_orto(&anel),
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            r,
        )
        .expect("o prisma");
        let out =
            ph2d_mesh_bool::corta(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair).expect("o corte");
        let p = out.positions();
        let mut tris = Vec::new();
        for f in out.faces() {
            f.triangles(&mut tris);
        }
        // A normal por vértice: a soma das normais das faces que o tocam.
        let mut n = vec![[0.0f32; 3]; p.len()];
        for t in &tris {
            let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let f = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let len = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
            if len <= 0.0 {
                continue;
            }
            for &i in t {
                let o = &mut n[i as usize];
                for k in 0..3 {
                    o[k] += f[k] / len;
                }
            }
        }
        // A parede `x = −0,5` é uma das quatro faces do corte; a normal dela
        // aponta para `+x` (para dentro do vão que a lâmina abriu).
        let (mut na_parede, mut planos) = (0usize, 0usize);
        for (i, q) in p.iter().enumerate() {
            if (q[0] + 0.5).abs() >= 1e-4 {
                continue;
            }
            na_parede += 1;
            let v = n[i];
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            // ⚠️ `1°` não é um epsilon escolhido: é a folga abaixo da qual um
            // degradé de sombreamento deixa de ser visível numa superfície lisa.
            if len > 0.0 && (v[0] / len).abs() > (1.0f32).to_radians().cos() {
                planos += 1;
            }
        }
        assert!(na_parede > 3, "a régua não achou a parede do corte");
        (planos as f32 / na_parede as f32, na_parede)
    };

    let (grossa, n_grossa) = fraccao_plana(Resolucao::Minima);
    let (fina, n_fina) = fraccao_plana(Resolucao::Ate(alvo));
    println!(
        "face cortada pintada PLANA: grossa {:.1}% de {n_grossa} vértices | \
         fina {:.1}% de {n_fina}",
        grossa * 100.0,
        fina * 100.0
    );
    // ⛔ **O CONTROLO é a metade que torna a de cima uma afirmação:** com a
    // lâmina mínima a face é um degradé de ponta a ponta, e uma régua que o
    // lesse como plano não estaria a medir o defeito do report.
    assert!(
        grossa < 0.10,
        "CONTROLO: com a lâmina mínima a face tinha de ser um degradé, e \
         {:.1}% dela já é plana",
        grossa * 100.0
    );
    assert!(
        fina > 0.80,
        "a face cortada só é pintada plana em {:.1}% — o report de 2026-09-15 \
         voltou",
        fina * 100.0
    );
}

#[test]
#[ignore = "sonda: a topologia da BORDA do corte, e o anel do circulo"]
fn diag_a_borda_do_corte() {
    let bola = shapes::sphere_with_triangles(50_000, 1.0);
    let tris: usize = bola
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), tris as f32);
    println!("\npeça: T={tris} alvo de aresta={alvo:.4}");

    // Um anel CIRCULAR como o gesto o produz hoje: `n` cordas.
    for n in [50usize, 100, 200, 400] {
        let anel: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let t = i as f32 / n as f32 * std::f32::consts::TAU;
                [0.6 * t.cos(), 0.6 * t.sin()]
            })
            .collect();
        let lamina = prisma(
            &anel,
            &raios_orto(&anel),
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            Resolucao::Ate(alvo),
        )
        .expect("o prisma");
        let out =
            ph2d_mesh_bool::corta(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair).expect("o corte");

        // A FLECHA do polígono contra o círculo verdadeiro, em unidades de cena.
        let flecha = 0.6 * (1.0 - (std::f32::consts::PI / n as f32).cos());

        // A borda do corte: as arestas em que uma face da parede encontra uma
        // face da esfera. A régua é a FORMA dos triângulos que lá vivem.
        let p = out.positions();
        let mut t3 = Vec::new();
        for f in out.faces() {
            f.triangles(&mut t3);
        }
        let na_parede = |i: u32| (p[i as usize][0].hypot(p[i as usize][1]) - 0.6).abs() < 1e-3;
        let mut asp: Vec<f32> = Vec::new();
        let mut menor = f32::INFINITY;
        let mut soltos = 0usize;
        for t in &t3 {
            let toca = t.iter().filter(|&&i| na_parede(i)).count();
            if toca == 0 || toca == 3 {
                continue; // longe da costura, ou no meio da parede
            }
            let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
            let e = |u: [f32; 3], v: [f32; 3]| {
                ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt()
            };
            let (l0, l1, l2) = (e(a, b), e(b, c), e(c, a));
            let s = (l0 + l1 + l2) * 0.5;
            let area = (s * (s - l0) * (s - l1) * (s - l2)).max(0.0).sqrt();
            menor = menor.min(l0.min(l1).min(l2));
            if area <= 1e-12 {
                soltos += 1;
                continue;
            }
            asp.push(l0.max(l1).max(l2) * s / (2.0 * area));
        }
        asp.sort_by(f32::total_cmp);
        let q = |f: f64| asp[((asp.len() - 1) as f64 * f) as usize];
        println!(
            "anel n={n:4} flecha={flecha:.5} ({:.2} do alvo) | COSTURA T={:5}  \
             aspecto p50={:.2} p90={:.2} p99={:.2} MAX={:.0}  aresta min={menor:.2e}  \
             degenerados={soltos}",
            flecha / alvo,
            asp.len(),
            q(0.5),
            q(0.9),
            q(0.99),
            asp.last().copied().unwrap_or(f32::NAN),
        );
    }
}

/// **A COSTURA do corte** — assunto próprio, ficheiro próprio.
///
/// ⚠️ O corte foi por RESPONSABILIDADE e forçado pelo tecto de LOC: aqui mede-se
/// a **densidade** da lâmina e o que ela entrega; ali, o que a curva de
/// interseção deixa na borda.
#[path = "lib_costura_tests.rs"]
mod costura;
