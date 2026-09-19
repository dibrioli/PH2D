//! As SONDAS da cena `=49` — os instrumentos, não a lei.
//!
//! ⛔ **Saíram do [`super`] por TECTO DE LOC** (`830` contra `700`), e o corte
//! é por RESPONSABILIDADE: ali ficam os dois gates (a cena tem o que mostrar ·
//! a cena está fiada) e aqui ficam as réguas exploratórias e os desenhadores
//! de PPM, que nenhuma tabela cita e que existem para a próxima janela medir.
//!
//! ⚠️ **O nome acaba em `_tests.rs` de propósito:** a classificação da família
//! é DERIVADA, e um ficheiro compilado só sob `cfg(test)` com outro nome
//! passaria a ser lido como PRODUTO pelas réguas de arquitectura — a
//! armadilha que o §41 desta linha já pagou.
//!
//! ⛔⛔ **E uma premissa minha CAIU no corte, desmentida pelo compilador:** eu
//! escrevi que mover o condutor `traco_com` para cá tornava o gate de fiação
//! do irmão mais forte (a agulha montada num ficheiro, varrida noutro). É
//! FALSO — o `traco_com` **é** a agulha, porque o `traco` que os dois gates
//! usam delega nele: ele é o condutor do traço, não um instrumento. Ficou lá.
//! *O que impede aquela agulha de se satisfazer a si própria continua a ser
//! ela ser montada por `format!`, e não o sítio onde mora.*

use super::*;
/// **SONDA — que peça tem GRÃO, e quanto?** (a régua que escolheu a da cena)
#[test]
#[ignore = "sonda"]
fn diag_o_grao_das_pecas_candidatas() {
    type Candidata = (&'static str, fn() -> ph2d_mesh::Mesh);
    let candidatas: Vec<Candidata> = vec![
        (
            "a peca da cena (uv 12k)",
            crate::scenes::pente::peca as fn() -> _,
        ),
        ("uv + ruido", || {
            ph2d_mesh::shapes::uv_sphere_noisy(55, 82, 1.0, 0.01)
        }),
        ("uv + remesh isotropico", || {
            let mut m = ph2d_mesh::shapes::sphere_with_triangles(12_000, 1.0);
            m.triangulate();
            let _ = ph2d_remesh_iso::remesh_isotropic(&mut m, 0.0142);
            m
        }),
        ("sculpt_sphere (a de fabrica)", || {
            ph2d_mesh::shapes::sculpt_sphere(1.0)
        }),
    ];
    for (nome, faz) in candidatas {
        let mut m = faz();
        m.triangulate();
        let v = m.positions().len();
        for (rotulo, eixo) in [("equador x", 0usize), ("meridiano y", 1)] {
            let mut centros = Vec::new();
            for k in 0..24 {
                let u = -0.55 + 0.05 * k as f32;
                let mut c = [0.0, 0.0, u.cos()];
                c[eixo] = u.sin();
                centros.push(c);
            }
            let (q, n) = q_da_faixa(&m, &centros, raio_do_app());
            let (bins, nb) = grade_da_faixa(&m, &centros, raio_do_app());
            eprintln!(
                "{nome:<30} {rotulo:<12} v={v:<7} Q={q:+.4} grade={:.1}% (n={n}/{nb})",
                100.0 * bins[0] as f64 / nb.max(1) as f64
            );
        }
    }
}

/// **SONDA — a escada do pente por RUMO contra a grade**, de onde saem a tabela
/// do cabeçalho da cena e a linha de água dos 45°.
#[test]
#[ignore = "sonda"]
fn diag_a_escada_por_rumo() {
    for (nome, e) in [
        ("ao longo (x)", [1.0f32, 0.0]),
        ("30 graus", [0.866_025_4, 0.5]),
        ("45 graus", RUMOS[2].1),
        ("60 graus", [0.5, 0.866_025_4]),
        ("atravessado (y)", [0.0, 1.0]),
    ] {
        for pente in [0.0f32, 0.25, 0.375, 0.5, 0.625, 0.75, 1.0] {
            let (m, c) = traco(pente, e);
            let (q, _) = q_da_faixa(&m, &c, raio_do_app());
            let (ang, _) = pior_angulo(&m, &c, raio_do_app());
            let (finas, total) = lascas(&m, &c, raio_do_app(), LIMIAR_DA_LASCA);
            eprintln!(
                "{nome:<18} pente {pente:.3}  Q={q:+.4}  pior={ang:6.2}°  \
                 <{LIMIAR_DA_LASCA}°: {finas:3} de {total}"
            );
        }
    }
}

/// ⛔⛔⛔ **SONDA — O REGIME QUE O APP DE FACTO DÁ**, contra o que o gate da cena
/// escolheu. O report do dono foi *«não percebi diferença»*, e o gate estava
/// VERDE: a primeira coisa a medir é se ele corre no regime do artista.
#[test]
#[ignore = "sonda"]
fn diag_o_regime_do_app_contra_o_do_gate() {
    use crate::Camera3d;

    let peca = crate::scenes::pente::peca();
    let bounds = peca.bounds();
    let (vw, vh) = (1920.0f32, 1080.0f32);
    let mut cam = Camera3d {
        yaw: 0.6,
        pitch: 0.35,
        ..Camera3d::default()
    };
    cam.frame(bounds, vw / vh);

    // O raio de fábrica do pincel, em PIXELS, convertido como o `armed_brush_on`
    // converte: através da câmera, no ponto onde o cursor aterra.
    let radius_px = ph2d_panel_sculpt3d::state::Sculpt3dUi::default().radius_px;
    let no_mundo =
        cam.world_radius_for_screen_px([0.0, 0.0, 1.0], radius_px, (vw as u32, vh as u32));

    // O alvo do refino que o passe de topologia usa com o `Detail` de fábrica.
    let detalhe = ph2d_panel_sculpt3d::state::Sculpt3dUi::default().dyn_detail;
    let area = peca.surface_area();
    let tris = ph2d_mesh::tris_for_detail(detalhe);
    let alvo = ph2d_mesh::edge_for_tri_count(area, tris);

    // A aresta média da peça, para se ler quantas cabem num raio.
    let pos = peca.positions();
    let (mut soma, mut n) = (0.0f64, 0usize);
    for f in peca.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (pos[vs[k] as usize], pos[vs[(k + 1) % vs.len()] as usize]);
            soma += f64::from((a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2]));
            n += 1;
        }
    }
    let aresta = soma / n.max(1) as f64;

    eprintln!("--- o que o APP da' ---");
    eprintln!("  raio do pincel: {radius_px} px  ->  {no_mundo:.4} no mundo");
    eprintln!("  refino: Detail {detalhe:.2} -> {tris} triangulos -> aresta alvo {alvo:.4}");
    eprintln!("  aresta media da peca: {aresta:.4}");
    eprintln!(
        "  arestas por raio (antes do refino): {:.1}",
        f64::from(no_mundo) / aresta
    );
    eprintln!(
        "  arestas por raio (depois):          {:.1}",
        f64::from(no_mundo / alvo)
    );
    eprintln!("--- o que o GATE mede ---");
    eprintln!(
        "  raio {:.4} (derivado) · refino {:.4} (derivado do Detail da cena) \
         · arestas por raio {:.1}",
        raio_do_app(),
        alvo_do_refino(),
        f64::from(raio_do_app() / alvo_do_refino())
    );
}

/// ⛔⛔⛔ **SONDA — quantas ARESTAS POR RAIO o pente precisa para se ver.**
///
/// O report do dono (*«não percebi diferença»*) com o gate VERDE: ele corre a
/// `10` arestas por raio e o app dá `2,0`.
#[test]
#[ignore = "sonda"]
fn diag_o_pente_contra_as_arestas_por_raio() {
    for (nome, raio, alvo) in [
        ("o APP de fabrica", 0.1634f32, 0.0805f32),
        ("Detail 0,75", 0.1634, 0.0402),
        ("Detail 1,00", 0.1634, 0.0175),
        ("raio 2x, Detail 1", 0.3268, 0.0175),
        ("o GATE de hoje", 0.35, 0.035),
    ] {
        let mut q = [0.0f64; 2];
        let mut lasca = [0usize; 2];
        let (mut verts, mut n1) = (0usize, 0usize);
        let mut saidas: Vec<Vec<[f32; 3]>> = Vec::new();
        for (i, pente) in [0.0f32, 1.0].into_iter().enumerate() {
            let (m, c) = traco_com(pente, RUMOS[2].1, raio, alvo);
            let (qq, nn) = q_da_faixa(&m, &c, raio);
            q[i] = qq;
            n1 = nn;
            lasca[i] = lascas(&m, &c, raio, LIMIAR_DA_LASCA).0;
            verts = m.positions().len();
            saidas.push(m.positions().to_vec());
        }
        // ⭐ O que o OLHO lê não é a média: é quantos vértices o pente de facto
        // deslocou, e quanto.
        let movidos = saidas[0]
            .iter()
            .zip(&saidas[1])
            .filter(|(a, b)| a != b)
            .count();
        eprintln!(
            "{nome:<20} arestas/raio {:>4.1}  Q {:+.4} -> {:+.4} (D {:+.4})  \
             arestas na faixa {n1:>5}  mexidos {movidos:>5}  lascas {}  v={verts}",
            f64::from(raio / alvo),
            q[0],
            q[1],
            q[1] - q[0],
            lasca[1]
        );
    }
}

/// ⛔⛔⛔ **SONDA — ONDE O EFEITO SE PERDE** (report do dono, 18/09: *«pouca ou
/// nenhuma diferença»*, com foto do arame).
///
/// ⚠️ **Toda régua desta cena é uma MÉDIA (`Q`) ou uma contagem AO BIT
/// (`mexidos`).** Nenhuma responde à pergunta do olho: *que fracção das arestas
/// da faixa corre com o traço?* Esta conta-as, por faixa de ângulo à grade, e
/// imprime ao lado quanto TRABALHO cada metade fez.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_onde_o_efeito_se_perde -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda"]
fn diag_onde_o_efeito_se_perde() {
    eprintln!(
        "{:<18} {:<6} {:>8}  {:>7} {:>7} {:>7}   {:>6} {:>6} {:>6}",
        "regime", "pente", "Q", "0-15°", "15-30°", "30-45°", "cortes", "fusoes", "trocas"
    );
    for (nome, raio, alvo) in [
        ("regime da cena", raio_do_app(), alvo_do_refino()),
        ("gate de hoje", 0.35f32, 0.035f32),
    ] {
        for pente in [0.0f32, 1.0] {
            let (m, c, (cortes, fusoes, trocas)) = traco_contado(pente, RUMOS[0].1, raio, alvo);
            let (q, _) = q_da_faixa(&m, &c, raio);
            let (bins, n) = grade_da_faixa(&m, &c, raio);
            let pc = |k: usize| 100.0 * bins[k] as f64 / n.max(1) as f64;
            eprintln!(
                "{nome:<18} {pente:<6.2} {q:>+8.4}  {:>6.1}% {:>6.1}% {:>6.1}%   \
                 {cortes:>6} {fusoes:>6} {trocas:>6}   (n={n})",
                pc(0),
                pc(1),
                pc(2)
            );
        }
    }
}

/// O mesmo traço do gate, com as TRÊS contagens de trabalho por dentro.
fn traco_contado(
    pente: f32,
    e: [f32; 2],
    raio: f32,
    alvo: f32,
) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>, (usize, usize, usize)) {
    let mut malha = peca_uma_vez();
    malha.triangulate();
    let brush = Brush {
        verb: Verb::Draw,
        radius: raio,
        strength: 0.25,
        pente,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();
    let mut centros = Vec::new();
    let (mut cortes, mut fusoes, mut trocas) = (0usize, 0usize, 0usize);
    let passo = raio * 0.15;
    for k in 0..24 {
        let u = -passo * 12.0 + passo * k as f32;
        let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
        centros.push(centro);
        let direccao = stroke.direccao_do_traco(centro);
        let alvo_do_colapso = ph2d_mesh::collapse_target(alvo);
        let campo_colapso = ph2d_sculpt3d::campo_do_pente(
            alvo_do_colapso,
            direccao,
            pente,
            ph2d_sculpt3d::Porta::Colapso,
        );
        let antes = malha.positions().len();
        if matches!(
            ph2d_mesh::collapse_in_sphere_sized(
                &mut malha,
                centro,
                brush.radius,
                alvo_do_colapso,
                Some(&campo_colapso),
                &mut remap,
                &mut region,
            ),
            ph2d_mesh::Collapse::Done { .. }
        ) {
            stroke.shrink_with(&remap);
        }
        fusoes += antes.saturating_sub(malha.positions().len());
        let antes = malha.positions().len();
        let campo =
            ph2d_sculpt3d::campo_do_pente(alvo, direccao, pente, ph2d_sculpt3d::Porta::Refino);
        let _ = ph2d_mesh::refine_in_sphere_sized(
            &mut malha,
            centro,
            brush.radius,
            alvo,
            Some(&campo),
            &mut births,
            &mut region,
        );
        stroke.grow_with(&malha, &births);
        cortes += malha.positions().len().saturating_sub(antes);
        if pente > 0.0 {
            let preferencia = ph2d_sculpt3d::preferencia_do_pente(direccao, pente);
            trocas +=
                ph2d_mesh::alinha_arestas(&mut malha, centro, brush.radius, &preferencia, &mut region);
        }
        stroke.dab(
            &mut malha,
            &brush,
            &Dab::at(centro, brush.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    (malha, centros, (cortes, fusoes, trocas))
}

/// ⛔⛔⛔ **SONDA — o CHÃO do flip contra a grade, nos QUATRO rumos** (report do
/// dono, 18/09).
///
/// A varredura das três constantes do flip (`docs/3D/ferramentas/varre_as_constantes_do_flip.py`) diz que a
/// alavanca é o [`chão de qualidade`] e não o ganho nem as rondas. Esta sonda
/// corre os quatro rumos que o gate da cena mede e imprime, por rumo, o que o
/// olho lê e o que a cerca defende — para o chão se escolher no MEIO de uma
/// janela medida, e não no primeiro número que funciona.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_o_chao_por_rumo -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda"]
fn diag_o_chao_por_rumo() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    for (nome, e) in RUMOS {
        let mut linha = String::new();
        for pente in [0.0f32, 1.0] {
            let agora = std::time::Instant::now();
            let (m, c, (_, _, trocas)) = traco_contado(pente, e, raio, alvo);
            let ms = agora.elapsed().as_secs_f64() * 1000.0 / 24.0;
            let (q, _) = q_da_faixa(&m, &c, raio);
            let (bins, n) = grade_da_faixa(&m, &c, raio);
            let (ang, _) = pior_angulo(&m, &c, raio);
            let (finas, tri) = lascas(&m, &c, raio, LIMIAR_DA_LASCA);
            linha.push_str(&format!(
                "  |{pente:>4.1}| Q {q:>+7.4} grade {:>5.1}% pior {ang:>6.2} lascas {finas:>3}/{tri:<5} \
                 trocas {trocas:>5} {ms:>6.2}ms/dab (n={n})",
                100.0 * bins[0] as f64 / n.max(1) as f64
            ));
        }
        eprintln!("{nome:<18}{linha}");
    }
}

/// ⛔⛔⛔ **SONDA — o flip muda a malha FORA do pincel?** (a pergunta que o gate
/// `o_alinhamento_para_na_borda_da_esfera` levantou ao ficar vermelho com o chão
/// mais baixo).
///
/// A região do passe **cresce** por rodada: cada uma semeia a seguinte com as
/// faces que mudou. Esta sonda mede o alcance em RAIOS DE PINCEL, no regime que
/// a `=49` dá — que é a única pergunta de produto.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_o_alcance_do_flip -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda"]
fn diag_o_alcance_do_flip() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let mut malha = peca_uma_vez();
    malha.triangulate();
    let mut births = Vec::new();
    let mut region = ph2d_mesh::RegionScratch::default();
    let centro = [0.0, 0.0, 1.0];
    let direccao = [1.0, 0.0, 0.0];
    // Um refino primeiro, como no produto — é ele que semeia o que o flip vê.
    let campo = ph2d_sculpt3d::campo_do_pente(alvo, direccao, 1.0, ph2d_sculpt3d::Porta::Refino);
    let _ = ph2d_mesh::refine_in_sphere_sized(
        &mut malha,
        centro,
        raio,
        alvo,
        Some(&campo),
        &mut births,
        &mut region,
    );
    let antes = malha.faces().to_vec();
    let pos = malha.positions().to_vec();
    let preferencia = ph2d_sculpt3d::preferencia_do_pente(direccao, 1.0);
    let trocas = ph2d_mesh::alinha_arestas(&mut malha, centro, raio, &preferencia, &mut region);
    let mut alcance = 0.0f32;
    for (i, f) in antes.iter().enumerate() {
        if *f == malha.faces()[i] {
            continue;
        }
        for v in f.verts() {
            let p = pos[*v as usize];
            let d = [p[0] - centro[0], p[1] - centro[1], p[2] - centro[2]];
            alcance = alcance.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
        }
    }
    let aresta = {
        let (mut soma, mut n) = (0.0f64, 0usize);
        for f in &antes {
            let vs = f.verts();
            for k in 0..vs.len() {
                let (a, b) = (pos[vs[k] as usize], pos[vs[(k + 1) % vs.len()] as usize]);
                soma += f64::from((a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2]));
                n += 1;
            }
        }
        soma / n.max(1) as f64
    };
    eprintln!(
        "raio {raio:.4} · aresta {aresta:.4} · trocas {trocas} · \
         alcance {alcance:.4} = {:.2} raios ({:.1} arestas)",
        alcance / raio,
        f64::from(alcance) / aresta
    );
}
