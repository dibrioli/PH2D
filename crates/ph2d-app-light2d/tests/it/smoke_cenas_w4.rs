//! As cenas de smoke 2 e 4 (`PH2D_LIGHT2D_SMOKE=2|4`) ensinam o que a tabela delas diz — medido pelo
//! GABARITO antes da mensagem ao dono (plano §6). A curva de custo e a penumbra NA PLACA estão na
//! sonda `medida_luz2d_w4`.

use bevy_ecs::world::World;
use ph2d_app_light2d::smoke;
use ph2d_ecs::Name;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_ecs::coleta::cena_de_luz;

use crate::comum::{largura_10_90, visivel};

fn luz_em(cena: &CenaDeLuz, p: [f64; 2]) -> f64 {
    ph2d_light2d_ref::luz_em(cena, p)[1]
}

/// ⭐⭐ **Cena 2:** os 1 000 vaga-lumes são 1 000 emissores; cada enxame acende o chão em volta, a
/// árvore faz sombra, e longe dos enxames fica a escuridão ambiente.
#[test]
fn a_cena_2_ensina_os_enxames_e_a_sombra_das_arvores() {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 2));
    let cena = cena_de_luz(&w);
    assert_eq!(
        cena.emissores.len(),
        smoke::VAGA_LUMES,
        "um emissor por vaga-lume"
    );
    assert_eq!(
        cena.oclusores.len(),
        smoke::ENXAMES,
        "uma árvore por enxame"
    );
    let ambiente = f64::from(cena.ceu[1]);
    let k = 0; // o enxame de cima à esquerda
    let (cx, cy) = (
        f64::from(smoke::CENTROS_DOS_ENXAMES[0][k]),
        f64::from(smoke::CENTROS_DOS_ENXAMES[1][k]),
    );
    let aberto = luz_em(&cena, [cx - 2.0, cy]);
    let atras = luz_em(&cena, [cx + 2.0, cy]);
    let perto = luz_em(&cena, [cx, cy - 1.4]);
    let longe = luz_em(&cena, [-0.6, 0.0]);
    eprintln!(
        "cena 2: perto {perto:.3} · aberto {aberto:.3} · atrás da árvore {atras:.3} · longe {longe:.3} · ambiente {ambiente:.3}"
    );
    assert!(perto > 0.3, "o chão ao lado do enxame acende ({perto:.3})");
    assert!(perto > 5.0 * ambiente, "e muito acima da ambiente");
    assert!(
        atras < 0.5 * aberto,
        "a árvore faz sombra ({atras:.3} × {aberto:.3})"
    );
    assert!(
        longe < perto / 1.5,
        "longe dos enxames é mais escuro ({longe:.3})"
    );
}

/// As alturas da linha de medida (onde a borda da sombra cruza `x = linha`).
fn alturas() -> Vec<f64> {
    (0..=600).map(|i| -1.7 + 0.004 * f64::from(i)).collect()
}

/// A cena 4 sem a parede da sala `sala` e sem céu: o denominador da fração de luz.
fn sem_parede(sala: usize) -> CenaDeLuz {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 4));
    let nome = ["Parede da pequena", "Parede da grande"][sala];
    let e = w
        .query::<(bevy_ecs::entity::Entity, &Name)>()
        .iter(&w)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .expect("a parede existe");
    w.despawn(e);
    let mut c = cena_de_luz(&w);
    c.ceu = [0.0; 3];
    c
}

/// ⭐⭐⭐ **Cena 4:** a largura da penumbra (10–90 %) MEDIDA pelo gabarito é a PREVISTA pela geometria
/// (as tangentes do disco pelo canto da parede), nas duas salas — e a grande é tantas vezes mais
/// macia quanto o raio dela.
#[test]
fn a_cena_4_mede_a_penumbra_contra_a_geometria() {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 4));
    let mut com = cena_de_luz(&w);
    com.ceu = [0.0; 3];
    let ys = alturas();
    let mut larguras = [0.0; 2];
    for (sala, raio) in [smoke::LAMPADA_PEQUENA_M, smoke::LAMPADA_GRANDE_M]
        .into_iter()
        .enumerate()
    {
        let (lampada, canto, linha) = smoke::geometria_da_sala(smoke::SALAS_X[sala]);
        let (lampada, canto, linha) = (
            lampada.map(f64::from),
            canto.map(f64::from),
            f64::from(linha),
        );
        let sem = sem_parede(sala);
        let medida: Vec<f64> = ys
            .iter()
            .map(|&y| luz_em(&com, [linha, y]) / luz_em(&sem, [linha, y]))
            .collect();
        let prevista: Vec<f64> = ys
            .iter()
            .map(|&y| visivel([linha, y], lampada, f64::from(raio), canto))
            .collect();
        let (m, p) = (
            largura_10_90(&ys, &medida).expect("medida"),
            largura_10_90(&ys, &prevista).expect("geometria"),
        );
        eprintln!("cena 4, raio {raio} m: penumbra 10–90 % medida {m:.4} m · prevista {p:.4} m");
        assert!(
            (m - p).abs() <= 0.02 * p + 0.004,
            "raio {raio}: a penumbra do gabarito ({m:.4}) não é a da geometria ({p:.4})"
        );
        larguras[sala] = m;
    }
    let razao = larguras[1] / larguras[0];
    let raios = f64::from(smoke::LAMPADA_GRANDE_M / smoke::LAMPADA_PEQUENA_M);
    eprintln!("cena 4: a grande é {razao:.2}× mais macia (raios {raios:.2}×)");
    assert!(razao > 0.8 * raios, "a lâmpada grande faz a sombra macia");
}

/// A penumbra 10–90 % da cena 4 NA PLACA, por sala: `(nome, raio, texel, placa, geometria)`. A
/// fração é a fluência com a parede ÷ sem ela, sem a ambiente nos dois lados (só a lâmpada conta).
pub(crate) fn penumbras_na_placa(
    gpu: &ph2d_gpu::GpuContext,
    degrau: ph2d_app_light2d::Degrau,
) -> Vec<(&'static str, f32, f64, Option<f64>, f64)> {
    let r = crate::comum::renderer(gpu);
    let janela = ph2d_host::WindowSize::new(1920, 1080);
    let afim = ph2d_render::Camera2d::default().world_to_screen_affine(janela);
    let fluencia = |w: &World| {
        let mut l = ph2d_app_light2d::LuzDoQuadro::com_degrau(degrau);
        assert!(l.preparar(w, afim, (1920, 1080)));
        l.gravar(gpu, &mut ph2d_ecs::PresentWorld::new(), &r);
        let f = crate::comum::fluencia_lida(gpu, l.solver().expect("solver"));
        let texel = l.grade().expect("grade").texel_m;
        (f, texel)
    };
    let tira = |w: &mut World, nome: &str| {
        let e = w
            .query::<(bevy_ecs::entity::Entity, &Name)>()
            .iter(w)
            .find(|(_, n)| n.0 == nome)
            .map(|(e, _)| e)
            .expect("existe");
        w.despawn(e);
    };
    let ys = alturas();
    let mut out = Vec::new();
    for (sala, (nome, raio)) in [
        ("pequena", smoke::LAMPADA_PEQUENA_M),
        ("grande", smoke::LAMPADA_GRANDE_M),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut com, mut sem) = (World::new(), World::new());
        assert!(smoke::encenar(&mut com, 4) && smoke::encenar(&mut sem, 4));
        tira(&mut com, "Luz ambiente");
        tira(&mut sem, "Luz ambiente");
        tira(&mut sem, ["Parede da pequena", "Parede da grande"][sala]);
        let ((fc, texel), (fs, _)) = (fluencia(&com), fluencia(&sem));
        let (lampada, canto, linha) = smoke::geometria_da_sala(smoke::SALAS_X[sala]);
        let linha = f64::from(linha);
        let medida: Vec<f64> = ys
            .iter()
            .map(|&y| fc.amostra([linha, y]) / fs.amostra([linha, y]))
            .collect();
        let prevista: Vec<f64> = ys
            .iter()
            .map(|&y| {
                visivel(
                    [linha, y],
                    lampada.map(f64::from),
                    f64::from(raio),
                    canto.map(f64::from),
                )
            })
            .collect();
        out.push((
            nome,
            raio,
            texel,
            largura_10_90(&ys, &medida),
            largura_10_90(&ys, &prevista).expect("geometria"),
        ));
    }
    out
}

/// ⭐⭐⭐ **Cena 4 NA PLACA (o degrau BAIXO, o mais grosso): as duas penumbras SÃO a geometria.** Pela
/// luz direta (C3) a visibilidade do disco vem de raios até ele, não das sondas das cascatas: medido
/// 0,3289 × 0,3283 m (pequena) e 1,4365 × 1,4064 m (grande). Antes da C3: 0,945 e 1,708 (a
/// resolução angular das cascatas) — o «tamanho afeta a qualidade» do smoke do dono, 09–10/10.
#[test]
fn a_cena_4_na_placa_as_penumbras_sao_a_geometria() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let p = penumbras_na_placa(&gpu, ph2d_app_light2d::DEGRAU_PADRAO);
    let (pequena, grande) = (p[0].3.expect("pequena"), p[1].3.expect("grande"));
    eprintln!(
        "placa: pequena {pequena:.4} m · grande {grande:.4} m (geometria {:.4} · {:.4})",
        p[0].4, p[1].4
    );
    for (m, g) in [(pequena, p[0].4), (grande, p[1].4)] {
        assert!(
            (m - g).abs() < 0.05 * g,
            "a penumbra na placa ({m:.4}) não é a da geometria ({g:.4})"
        );
    }
}

/// ⭐⭐⭐ **Cena 3 (W5c):** o que a cena ensina é o que acontece — pelo GABARITO com a série do
/// rebote: o fundo avermelha junto à parede vermelha e esverdeia junto à verde, a sala clareia sem
/// estourar, e sem o rebote a luz do fundo é branca.
#[test]
fn a_cena_3_ensina_o_rebote() {
    use ph2d_light2d_ref::{OpcoesDoRebote, Rebote, luz_em};
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let cena = cena_de_luz(&w);
    assert_eq!(cena.oclusores.len(), 4, "chão, teto e as duas paredes");
    assert!(
        cena.oclusores
            .iter()
            .all(|o| o.albedo.iter().any(|&a| a > 0.0)),
        "as paredes refletem: {:?}",
        cena.oclusores
    );
    let serie = Rebote::resolver(
        &cena,
        OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 120,
        },
    );
    let [sw, _] = smoke::SALA_DO_REBOTE.map(f64::from);
    let x = 0.5 * sw;
    let (perto_vermelha, perto_verde, meio) = ([-x + 0.3, 0.0], [x - 0.3, 0.0], [0.0, -0.8]);
    let r = |p: [f64; 2]| {
        let l = serie.luz_em(p);
        l[0] / l[1]
    };
    let (rv, rg) = (r(perto_vermelha), r(perto_verde));
    let (com, sem) = (serie.luz_em(meio)[1], luz_em(&cena, meio)[1]);
    eprintln!(
        "cena 3: R/G junto à vermelha {rv:.3} · à verde {rg:.3} · o meio {sem:.3} → {com:.3}"
    );
    assert!(
        rv > 1.2 && rg < 1.0 / 1.2,
        "o fundo tinge-se: {rv:.3} · {rg:.3}"
    );
    assert!(
        com > 1.5 * sem,
        "o rebote clareia a sala: {sem:.3} → {com:.3}"
    );
    assert!(com < 1.0, "a sala não estoura: {com:.3}");
    for p in [perto_vermelha, perto_verde, meio] {
        let l = luz_em(&cena, p);
        assert!(
            (l[0] - l[1]).abs() <= 1e-9 * l[0],
            "sem o rebote a luz é branca: {l:?}"
        );
    }
}

/// ⭐⭐ **Cena 5:** o que a cena ensina é o que acontece — pelo caminho do PRODUTO (os sprites com a
/// imagem deles: o disco e a elipse pela máscara do disco do atlas), cada corpo recebe a luz com
/// FACES: o ponto de dentro do lado da lâmpada mais claro que o do lado de trás. A régua: em cada
/// corpo, dois pontos de dentro sobre a reta centro → lâmpada, a `PROFUNDIDADE` do menor lado.
#[test]
fn a_cena_5_ensina_o_volume() {
    use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro};
    use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef};
    use ph2d_render::{DISC_TILE_KEY, DISC_TILE_PX, RenderInstance, Sprite, disc_tile_pixels};
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let cena = cena_de_luz(&w);
    assert_eq!(
        cena.oclusores.len(),
        smoke::CORPOS_DO_VOLUME.len(),
        "os corpos bloqueiam"
    );
    assert!(
        cena.oclusores
            .iter()
            .all(|o| o.albedo.iter().any(|&a| a > 0.0)),
        "os corpos recebem a luz (refletem): nenhum tem «não recebe luz»"
    );
    let mut r = crate::comum::renderer(&gpu);
    let branca = r.acquire_individual(1, 1, &[255; 4]).expect("textura");
    let disco: Vec<u8> = disc_tile_pixels(DISC_TILE_PX)
        .into_iter()
        .flat_map(|p| p.0)
        .collect();
    let disco = r
        .acquire_individual(DISC_TILE_PX, DISC_TILE_PX, &disco)
        .expect("textura do disco");
    let mut present = PresentWorld::new();
    let mut q = w.query_filtered::<(bevy_ecs::entity::Entity, &Sprite, &GlobalTransform), bevy_ecs::query::With<ph2d_light2d_ecs::BloqueiaLuz>>();
    for (e, s, g) in q.iter(&w) {
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let e_disco =
            matches!(s.source, ph2d_render::SpriteSource::Atlas { key } if key == DISC_TILE_KEY);
        let tex = if e_disco { disco } else { branca };
        let mut i = crate::comum::quad(tex, [tx, ty]);
        i.size = s.size;
        i.anchor = s.anchor;
        i.basis = [ax, ay, bx, by];
        let _: &RenderInstance = &i;
        present.world_mut().spawn((i, SimRef(e)));
    }
    let afim = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(1920, 1080));
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    for _ in 0..30 {
        assert!(l.preparar(&w, afim, (1920, 1080)));
        l.gravar(&gpu, &mut present, &r);
    }
    let f = crate::comum::fluencia_lida(&gpu, l.solver().expect("solver"));
    let lampada = smoke::LAMPADA_DO_VOLUME.map(f64::from);
    let mut falhas = Vec::new();
    for corpo in smoke::CORPOS_DO_VOLUME {
        let (nome, lado) = (corpo.nome, corpo.lado);
        let c = corpo.centro.map(f64::from);
        let d = [lampada[0] - c[0], lampada[1] - c[1]];
        let n = d[0].hypot(d[1]);
        let u = [d[0] / n, d[1] / n];
        let k = PROFUNDIDADE * f64::from(lado[0].min(lado[1]));
        let (frente, tras) = (
            f.amostra([c[0] + u[0] * k, c[1] + u[1] * k]),
            f.amostra([c[0] - u[0] * k, c[1] - u[1] * k]),
        );
        eprintln!("cena 5 · {nome}: a face da lâmpada {frente:.3} · a de trás {tras:.3}");
        if frente < CONTRASTE_DAS_FACES * tras {
            falhas.push(format!("{nome}: {frente:.3} × {tras:.3}"));
        }
    }
    assert!(
        falhas.is_empty(),
        "um corpo sem faces: {}",
        falhas.join(" · ")
    );
}

/// Os pontos lidos ficam a esta fração do menor lado do corpo, de cada lado do centro.
const PROFUNDIDADE: f64 = 0.35;
/// A face virada para a lâmpada é pelo menos isto × a de trás — MEDIDO (RTX 5060 Ti, `DEGRAU_PADRAO`):
/// o menor 18,5 × (a cruz), o maior 63 × (a elipse).
const CONTRASTE_DAS_FACES: f64 = 10.0;

/// ⭐⭐ **GATE — a cena 6 ensina as formas DESENHADAS** (W5d; o dono, 11/10: «não temos desenhos
/// esféricos para testar»). O círculo e a elipse vetoriais entram no documento pela porta do smoke; a
/// entidade de cada um (a que o editor cria) ganha o nome e o «bloqueia a luz»; e pelo caminho do
/// produto (a camada vetorial da `LuzDoQuadro`) cada forma tem as FACES — a da lâmpada ≥
/// [`CONTRASTE_DAS_FACES`] × a de trás — e a luz do interior do DESENHO bate com a da IMAGEM gêmea, no
/// espelho em relação à lâmpada (a paridade desenho × imagem da lei do interior).
#[test]
fn a_cena_6_ensina_as_formas_desenhadas() {
    use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro};
    use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef, Transform, VecPathRef};
    use ph2d_render::{DISC_TILE_PX, Sprite, disc_tile_pixels};
    use ph2d_vec_scene::{VecScene, VecXforms, Xform};
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 6));
    let mut doc = VecScene::new();
    assert!(
        smoke::semear_documento(&mut doc, 5).is_empty(),
        "só a cena 6 semeia o documento"
    );
    let semeadas = smoke::semear_documento(&mut doc, 6);
    assert_eq!(semeadas.len(), smoke::REDONDAS.len());
    let semeadas = smoke::marcar_semeadas(&mut w, &doc, semeadas);
    assert_eq!(
        semeadas.len(),
        smoke::REDONDAS.len(),
        "antes da sincronização do editor não há entidade a marcar"
    );
    // A sincronização do editor (`entities::sync`): a entidade de cada caminho, no lugar dele.
    let mut xforms = VecXforms::new();
    for (id, _) in &semeadas {
        w.spawn((
            Transform::IDENTITY,
            Name::new(format!("Path {id}")),
            VecPathRef(*id),
        ));
        xforms.insert(*id, Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]));
    }
    assert!(smoke::marcar_semeadas(&mut w, &doc, semeadas).is_empty());
    let cena = cena_de_luz(&w);
    assert_eq!(
        cena.vetores.len(),
        smoke::REDONDAS.len(),
        "os desenhos bloqueiam"
    );
    assert!(cena.vetores.iter().all(|v| v.reflete), "e recebem a luz");
    let mut nomes: Vec<String> = w
        .query::<(&Name, &VecPathRef)>()
        .iter(&w)
        .map(|(n, _)| n.as_str().to_owned())
        .collect();
    nomes.sort();
    assert_eq!(
        nomes,
        ["Círculo", "Elipse"],
        "o nome que aparece na Hierarchy"
    );
    // O chão no FUNDO: sem `RootOrder` uma raiz desenha na frente e cobria os desenhos (a foto).
    let chao = w
        .query::<(&Name, &ph2d_ecs::RootOrder)>()
        .iter(&w)
        .find(|(n, _)| n.as_str() == "Chão")
        .map(|(_, o)| o.0);
    assert_eq!(
        chao,
        Some(0),
        "o chão da cena 6 desenha por baixo dos desenhos"
    );

    let mut r = crate::comum::renderer(&gpu);
    let disco: Vec<u8> = disc_tile_pixels(DISC_TILE_PX)
        .into_iter()
        .flat_map(|p| p.0)
        .collect();
    let disco = r
        .acquire_individual(DISC_TILE_PX, DISC_TILE_PX, &disco)
        .expect("textura do disco");
    let mut present = PresentWorld::new();
    let mut q = w.query_filtered::<(bevy_ecs::entity::Entity, &Sprite, &GlobalTransform), bevy_ecs::query::With<ph2d_light2d_ecs::BloqueiaLuz>>();
    for (e, s, g) in q.iter(&w) {
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let mut i = crate::comum::quad(disco, [tx, ty]);
        i.size = s.size;
        i.anchor = s.anchor;
        i.basis = [ax, ay, bx, by];
        present.world_mut().spawn((i, SimRef(e)));
    }
    let afim = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(1920, 1080));
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    for _ in 0..30 {
        assert!(l.preparar(&w, afim, (1920, 1080)));
        l.desenhar_vetores(&gpu, &doc, &xforms);
        l.gravar(&gpu, &mut present, &r);
    }
    let f = crate::comum::fluencia_lida(&gpu, l.solver().expect("solver"));
    let mut falhas = Vec::new();
    for forma in &smoke::REDONDAS {
        // A régua do ESPELHO (partilhada com a cena 7): as faces do desenho e da imagem gêmea (no
        // espelho em relação à lâmpada, `y → −y`) e a variação ao longo de anéis dentro e logo fora.
        // Ponto a ponto não: as cunhas da penumbra da lâmpada diferem entre a curva de Bézier e a
        // textura do disco.
        let m = crate::o_traco_do_flip::espelho(&f, forma.centro, forma.semi);
        for (i, qual) in ["desenho", "imagem"].into_iter().enumerate() {
            eprintln!(
                "cena 6 · {} ({qual}): a face da lâmpada {:.3} · a de trás {:.3}",
                forma.nome, m.frente[i], m.tras[i]
            );
            if m.frente[i] < CONTRASTE_DAS_FACES * m.tras[i] {
                falhas.push(format!(
                    "{} ({qual}) sem faces: {:.3} × {:.3}",
                    forma.nome, m.frente[i], m.tras[i]
                ));
            }
        }
        eprintln!(
            "cena 6 · {}: a variação nos anéis, desenho × imagem {:+.1} %",
            forma.nome,
            100.0 * m.variacao
        );
        if m.variacao > TOL_TV_DESENHO_IMAGEM {
            falhas.push(format!(
                "{}: a variação nos anéis {:+.1} % acima da imagem",
                forma.nome,
                100.0 * m.variacao
            ));
        }
        let e = (m.frente[0] - m.frente[1]).abs() / m.frente[1];
        eprintln!(
            "cena 6 · {}: desenho × imagem {:+.2} %",
            forma.nome,
            100.0 * (m.frente[0] - m.frente[1]) / m.frente[1]
        );
        if e > TOL_DESENHO_IMAGEM {
            falhas.push(format!("{}: desenho × imagem {:.3}", forma.nome, e));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join(" · "));
}

/// A face da lâmpada do DESENHO contra a da IMAGEM gêmea, relativo — MEDIDO (RTX 5060 Ti,
/// `DEGRAU_PADRAO`, 30 quadros com o rebote): o círculo −0,44 %, a elipse −0,05 %; as faces 46 × e 47 ×.
const TOL_DESENHO_IMAGEM: f64 = 0.01;
/// A variação total dos anéis do desenho acima da da imagem (relativo) — MEDIDO (RTX 5060 Ti): o círculo
/// −0,3 %, a elipse +3,9 %. A camada do vetor com a cobertura pelo limiar (a dos sprites) saía em
/// riscos: ver a prova de mutação no handoff W5d.
const TOL_TV_DESENHO_IMAGEM: f64 = 0.08;
