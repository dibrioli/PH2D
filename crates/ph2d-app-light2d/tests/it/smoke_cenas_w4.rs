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
    assert!(!smoke::encenar(&mut World::new(), 5), "não há cena 5");
    assert_eq!(smoke::ROUTERS[0].max_level, 4, "o maior nível do `match`");
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
/// rebote: o fundo avermelha junto à parede vermelha e esverdeia junto à verde, cada face da caixa
/// toma a cor da parede que a olha, e sem o rebote a luz do fundo é branca.
#[test]
fn a_cena_3_ensina_o_rebote() {
    use ph2d_light2d_ref::{OpcoesDoRebote, Rebote, luz_em};
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let cena = cena_de_luz(&w);
    assert_eq!(
        cena.oclusores.len(),
        5,
        "chão, teto, duas paredes e a caixa"
    );
    assert!(
        cena.oclusores
            .iter()
            .all(|o| o.albedo.iter().any(|&a| a > 0.0)),
        "as paredes e a caixa refletem: {:?}",
        cena.oclusores
    );
    let serie = Rebote::resolver(
        &cena,
        OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 120,
        },
    );
    let [sw, sh] = smoke::SALA_DO_REBOTE.map(f64::from);
    let (x, y) = (0.5 * sw, 0.5 * sh);
    let perto_vermelha = [-x + 0.3, 0.0];
    let perto_verde = [x - 0.3, 0.0];
    let caixa_esquerda = [-0.42, -y + 0.5];
    let caixa_direita = [0.42, -y + 0.5];
    let r = |p: [f64; 2]| {
        let l = serie.luz_em(p);
        l[0] / l[1]
    };
    let (rv, rg) = (r(perto_vermelha), r(perto_verde));
    let (ce, cd) = (r(caixa_esquerda), r(caixa_direita));
    eprintln!(
        "cena 3: R/G junto à vermelha {rv:.3} · à verde {rg:.3} · caixa esquerda {ce:.3} · direita {cd:.3}"
    );
    assert!(
        rv > 1.2 && rg < 1.0 / 1.2,
        "o fundo tinge-se: {rv:.3} · {rg:.3}"
    );
    assert!(
        ce > 1.1 && cd < 1.0 / 1.1,
        "as faces da caixa tingem-se: {ce:.3} · {cd:.3}"
    );
    for p in [perto_vermelha, perto_verde, caixa_esquerda, caixa_direita] {
        let l = luz_em(&cena, p);
        assert!(
            (l[0] - l[1]).abs() <= 1e-9 * l[0],
            "sem o rebote a luz é branca: {l:?}"
        );
    }
}
