//! ⭐⭐⭐ **A FORMA PRESA COM EFEITO contra o padrão-ouro** — ver [`crate::skin_desenho`], secção
//! *«As QUINAS VIVAS e os EFEITOS correm no REPOUSO»*.
//!
//! O padrão-ouro de uma forma com efeito é o que a IMAGEM presa faria com a arte dela: o efeito
//! desenhado em REPOUSO, e cada ponto desse desenho levado pela lei da pele ([`BPalco::ouro`]).

use super::ouro_reguas_tests::*;
use crate::skin_desenho::{Estilo, Leis, estilo_de};
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

/// As amostras por segmento das réguas — o cozido de um *Zig Zag* tem centenas de segmentos, e a
/// régua é `O(n·m)`.
const POR_SEG: usize = 12;

/// Os efeitos da fixtura: os que lêem a CAIXA da forma (`FxCtx`) e um que lê a contagem de nós.
fn efeitos() -> Vec<(&'static str, PathEffect)> {
    let zz = ph2d_vec_scene::fx_zigzag::ZigZagSpec {
        amplitude: 6.0,
        ridges: 24.0,
        ..Default::default()
    };
    let mut warp = ph2d_vec_scene::fx_warp_presets::WarpSpec::new(
        ph2d_vec_scene::fx_warp_presets::WarpStyle::Arc,
    );
    warp.bend = 40.0;
    vec![
        ("ZigZag", PathEffect::ZigZag(zz)),
        (
            "Twist",
            PathEffect::Twist(ph2d_vec_scene::fx_twist::TwistSpec { angle: 25.0 }),
        ),
        ("Warp", PathEffect::Warp(warp)),
    ]
}

fn caminho_mut(scene: &mut VecScene, id: VecPathId) -> &mut VecPath {
    scene.path_mut(id).expect("path")
}

/// O desenho EM REPOUSO da fonte com a pilha — o que o artista viu antes de dobrar.
fn repouso_com(p: &BPalco, pilha: &[FxEntry]) -> VecPath {
    let mut f = p.fonte.clone();
    f.effects = pilha.to_vec();
    f.cooked().into_owned()
}

/// O que se vê pela lei NOVA (o desenhado, sem a silhueta — ela é julgada à parte) e pela
/// ANTIGA (o caminho da cena cozido com a pilha sobre os nós dobrados).
fn as_duas_leis(p: &BPalco) -> (VecPath, VecPath) {
    let mut sc = p.scene.clone();
    let base = Leis {
        c1: false,
        contacto: false,
        ..Leis::do_ambiente()
    };
    let nova = crate::skin_live::recook_leis(
        &p.sim,
        &mut sc,
        Leis {
            efeitos: true,
            ..base
        },
    )
    .remove(&p.id)
    .expect("a forma com efeito tem desenho fiel");
    assert!(
        nova.effects.is_empty(),
        "o desenhado leva a pilha outra vez — um consumidor que o coza aplica-a DUAS vezes"
    );
    let mut sc = p.scene.clone();
    let antigo = crate::skin_live::recook_leis(
        &p.sim,
        &mut sc,
        Leis {
            efeitos: false,
            ..base
        },
    );
    assert!(
        antigo.is_empty(),
        "com `efeitos: false` saiu desenho — a porta de bissecção mente"
    );
    let cru = sc
        .paths()
        .iter()
        .find(|q| q.id == p.id)
        .expect("path")
        .cooked()
        .into_owned();
    (nova, cru)
}

/// Os efeitos FORTES do report do dono (2026-10-03, quatro fotos): pontas de *Bloat* e a cauda de
/// um *Twist* muito fora da barra.
fn efeitos_fortes() -> Vec<(&'static str, PathEffect)> {
    let bloat = |amount| PathEffect::Bloat(ph2d_vec_scene::fx_warp::BloatSpec { amount });
    let twist = |angle| PathEffect::Twist(ph2d_vec_scene::fx_twist::TwistSpec { angle });
    vec![
        ("Bloat-60", bloat(-60.0)),
        ("Bloat-200", bloat(-200.0)),
        ("Twist25", twist(25.0)),
        ("Twist60", twist(60.0)),
        ("Twist120", twist(120.0)),
    ]
}

/// O padrão-ouro de uma forma com efeito: a lei da IMAGEM ponto a ponto sobre o campo cujo DOMÍNIO
/// é o contorno cozido — o que o *Puppet* do After Effects faz com o que a camada desenha.
fn ouro_do_cozido(p: &BPalco, pele: &ph2d_skeleton::Skin, repouso: &VecPath) -> Vec<[f64; 2]> {
    let skin = p
        .sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(p.alvo)
        .expect("pele");
    let eixos =
        crate::skin_live::eixos_do_bind(&p.sim, skin, &crate::skin_live::bone_index(&p.sim));
    let campo = ph2d_vec_skin::pesos::campo_do_caminho(repouso, &eixos)
        .expect("o solver responde sobre o contorno cozido");
    b_amostra_com(repouso, POR_SEG)
        .into_iter()
        .map(|x| b_ouro_pt(pele, &campo, &p.correcoes, x).0)
        .collect()
}

/// ⭐⭐ **O ESTICÃO MÁXIMO entre duas amostras vizinhas** — `|Δ dobrado| / |Δ repouso|`. Uma pele de
/// ossos rígidos estica pouco; um RASGO (um pedaço a seguir um osso e o vizinho outro) lê-se aqui
/// como um número enorme, sem convenção nenhuma sobre o que é o «certo».
fn esticao(rest: &[[f64; 2]], def: &[[f64; 2]]) -> f64 {
    rest.windows(2)
        .zip(def.windows(2))
        .filter_map(|(r, d)| {
            let lr = (r[1][0] - r[0][0]).hypot(r[1][1] - r[0][1]);
            (lr > 1e-6).then(|| (d[1][0] - d[0][0]).hypot(d[1][1] - d[0][1]) / lr)
        })
        .fold(0.0, f64::max)
}

/// ⭐⭐⭐ **GATE — O EFEITO DOBRA COM A FORMA: a lei nova segue o padrão-ouro e a antiga não.**
///
/// # As três metades
///
/// 1. **Em repouso** a lei nova É o desenho do artista (a pele não mudou o efeito).
/// 2. ⛔ **O CONTROLO: a lei antiga está LONGE do ouro** na dobra — o efeito medido na caixa da
///    POSE. Sem isto a fixtura não conteria o defeito.
/// 3. **A lei nova está perto**, e por uma ordem de grandeza.
#[test]
fn o_efeito_corre_no_repouso_e_dobra_com_a_forma() {
    for (nome, efeito) in efeitos() {
        let pilha = vec![FxEntry::new(efeito)];
        for graus in [0.0_f32, 60.0, 90.0] {
            let mut p = b_palco(false);
            p.reparte_com(1, false);
            p.lei_do_peso(false);
            caminho_mut(&mut p.scene, p.id).effects = pilha.clone();
            p.dobra_em_s(graus);
            let pele = p.pele();
            let repouso = repouso_com(&p, &pilha);
            let ouro = ouro_do_cozido(&p, &pele, &repouso);
            let (nova, antiga) = as_duas_leis(&p);
            let [(_, _, nmax), (_, _, amax)] =
                [&nova, &antiga].map(|c| b_perfil(&ouro, &b_amostra_com(c, POR_SEG)));
            println!(
                "  {nome:>7} {graus:>4}°: antiga máx {amax:.5} · nova ({} nós) máx {nmax:.5}",
                nova.verts_all().count()
            );
            assert!(
                nmax < 0.01,
                "{nome} a {graus}°: a lei nova afasta-se {nmax} do padrão-ouro (diagonal ~7,07)"
            );
            if graus >= 60.0 {
                assert!(
                    amax > 0.05,
                    "{nome} a {graus}°: a lei antiga leu máx {amax} — a fixtura deixou de conter o \
                     defeito"
                );
                assert!(
                    nmax * 10.0 < amax,
                    "{nome} a {graus}°: a lei nova ({nmax}) não é melhor que a antiga ({amax})"
                );
            }
        }
    }
}

/// ⭐⭐⭐ **GATE — UM EFEITO QUE SAI DA FORMA NÃO RASGA, e o desenho segue-o em TODA a dobra**
/// (report do dono de 2026-10-03: *«a depender do nível da deformação, tudo se deforma muito
/// ruim»*, pontas de *Bloat* e a cauda de um *Twist*).
///
/// # As três metades
///
/// 1. ⛔ **O CONTROLO: o ideal sobre o campo da FONTE rasga** onde o efeito sai da barra (a cauda
///    do *Twist* — `17`–`191×` de esticão entre amostras vizinhas). Um *Bloat* fica dentro do
///    alcance da barra e não rasga; ele está aqui pela 3.ª metade.
/// 2. **Sobre o campo do contorno COZIDO o ideal não rasga** (`≤ 3,6×`).
/// 3. **O desenho do PRODUTO (com a lei do contacto ligada) fica no ideal até `120°`** — a
///    silhueta reescrevia-o (`0,16`–`2,04`) e era a fonte dos pedaços soltos e da serrilha.
#[test]
fn um_efeito_que_sai_da_forma_nao_rasga() {
    for (nome, efeito) in efeitos_fortes() {
        let pilha = vec![FxEntry::new(efeito)];
        for graus in [60.0_f32, 90.0, 120.0] {
            let mut p = b_palco(false);
            caminho_mut(&mut p.scene, p.id).effects = pilha.clone();
            p.dobra_em_s(graus);
            let pele = p.pele();
            let repouso = repouso_com(&p, &pilha);
            let rest = b_amostra_com(&repouso, POR_SEG);
            let da_fonte: Vec<[f64; 2]> = rest
                .iter()
                .map(|&x| b_ouro_pt(&pele, &p.campo, &p.correcoes, x).0)
                .collect();
            let ouro = ouro_do_cozido(&p, &pele, &repouso);
            let (ef, ec) = (esticao(&rest, &da_fonte), esticao(&rest, &ouro));
            let d =
                crate::skin_live::recook_leis(&p.sim, &mut p.scene.clone(), Leis::do_ambiente())
                    .remove(&p.id)
                    .expect("desenho");
            let (_, _, nmax) = b_perfil(&ouro, &b_amostra_com(&d, POR_SEG));
            println!(
                "  {nome:>9} {graus:>4}°: esticão com o campo da fonte {ef:.2} · do cozido {ec:.2} \
                 · o desenho afasta-se {nmax:.5} do ideal"
            );
            if nome.starts_with("Twist") {
                assert!(
                    ef > 10.0,
                    "{nome} a {graus}°: o CONTROLO não rasga ({ef}) — a fixtura perdeu o defeito"
                );
            }
            assert!(
                ec < 4.0,
                "{nome} a {graus}°: sobre o campo do contorno cozido o ideal estica {ec}"
            );
            assert!(
                nmax < 0.01,
                "{nome} a {graus}°: o desenho afasta-se {nmax} do ideal sem rasgo"
            );
        }
    }
}

/// ⭐⭐ **GATE — O ESTILO: só o efeito ACTIVO conta, e o offset de camada continua de fora.**
#[test]
fn so_o_efeito_activo_conta_e_o_offset_de_camada_fica_de_fora() {
    let p = b_palco(false);
    let base = p.fonte.clone();
    assert_eq!(estilo_de(&base), Estilo::Serve, "o CONTROLO: a barra serve");
    let mut neutro = base.clone();
    neutro.effects.push(FxEntry::new(PathEffect::ZigZag(
        ph2d_vec_scene::fx_zigzag::ZigZagSpec::default(),
    )));
    assert_eq!(
        estilo_de(&neutro),
        Estilo::Serve,
        "um efeito NEUTRO (amplitude 0) tirou a forma do caminho de sempre"
    );
    let (_, efeito) = efeitos().remove(0);
    let mut desligado = base.clone();
    let mut e = FxEntry::new(efeito.clone());
    e.enabled = false;
    desligado.effects.push(e);
    assert_eq!(
        estilo_de(&desligado),
        Estilo::Serve,
        "um efeito DESLIGADO contou"
    );
    let mut fx = base.clone();
    fx.effects.push(FxEntry::new(efeito.clone()));
    assert_eq!(
        estilo_de(&fx),
        Estilo::Efeitos(vec![FxEntry::new(efeito.clone())])
    );
    // ⛔ E o offset de CAD numa camada manda para o caminho de sempre, com efeito ou sem ele.
    let mut camada = ph2d_vec_scene::PaintEntry::stroke(ph2d_vec_scene::StrokeSpec::new(
        ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
        0.05,
    ));
    camada.dilate = 0.2;
    fx.paints.push(camada);
    assert_eq!(
        estilo_de(&fx),
        Estilo::NaoServe,
        "um offset de camada entrou no bake — ele é indexado pelo id da FONTE"
    );
}

/// ⭐⭐ **GATE — CÓPIAS QUE SE SOBREPÕEM DE PROPÓSITO não se fundem.** Um *Repeat* com as cópias a
/// meia altura cruza-se já em REPOUSO: a união do contacto fundi-las-ia num contorno só, e o
/// artista perderia o traço de cada cópia sem dobrar nada.
#[test]
fn copias_sobrepostas_em_repouso_nao_se_fundem() {
    let mut p = b_palco(false);
    let pilha = vec![FxEntry::new(PathEffect::Repeat(
        ph2d_vec_scene::fx_repeat::RepeatSpec {
            copies_y: 2.0,
            move_y: 50.0,
            ..Default::default()
        },
    ))];
    caminho_mut(&mut p.scene, p.id).effects = pilha.clone();
    let repouso = repouso_com(&p, &pilha);
    let contornos = |c: &VecPath| 1 + c.subpaths.len();
    assert!(
        contornos(&repouso) >= 2 && ph2d_vec_boolean::resolve_overlap(&repouso).is_some(),
        "o CONTROLO: as cópias da fixtura têm de se sobrepor em repouso"
    );
    for graus in [0.0_f32, 30.0] {
        p.dobra_em_s(graus);
        let d = crate::skin_live::recook_leis(&p.sim, &mut p.scene.clone(), Leis::do_ambiente())
            .remove(&p.id)
            .expect("desenho");
        assert_eq!(
            contornos(&d),
            contornos(&repouso),
            "a {graus}° as cópias do Repeat fundiram-se — a união correu sobre uma sobreposição \
             que o artista desenhou"
        );
    }
}

/// ⭐⭐ **GATE — MUDAR O EFEITO DEPOIS DO BIND chega ao desenho** (a pilha que vale é a VIVA, e a
/// gaveta não devolve o cozido da pilha velha).
#[test]
fn mudar_o_efeito_depois_do_bind_chega_ao_desenho() {
    let mut p = b_palco(false);
    p.dobra_em_s(60.0);
    let leis = Leis {
        contacto: false,
        ..Leis::do_ambiente()
    };
    let desenho = |p: &mut BPalco, amplitude: f64| {
        let zz = ph2d_vec_scene::fx_zigzag::ZigZagSpec {
            amplitude,
            ridges: 24.0,
            ..Default::default()
        };
        caminho_mut(&mut p.scene, p.id).effects = vec![FxEntry::new(PathEffect::ZigZag(zz))];
        crate::skin_live::recook_leis(&p.sim, &mut p.scene, leis)
            .remove(&p.id)
            .expect("desenho")
    };
    let a = desenho(&mut p, 4.0);
    let b = desenho(&mut p, 8.0);
    let a2 = desenho(&mut p, 4.0);
    assert_ne!(a.verts, b.verts, "a amplitude nova não chegou ao desenho");
    assert_eq!(a.verts, a2.verts, "a mesma pilha deu outro desenho");
}

/// ⭐⭐ **SONDA — O PREÇO POR QUADRO de uma forma com efeito**, com a pose a MUDAR a cada chamada
/// (um quadro parado vem da gaveta). Lei antiga = os nós dobrados e a pilha cozida sobre eles (o
/// que o desenho pagava ao cozer a forma). Imprime; corra em `--release` e com a máquina calma.
#[test]
fn diag_o_preco_do_efeito_por_quadro() {
    use std::time::Instant;
    for (nome, efeito) in efeitos() {
        let mut p = b_palco(false);
        caminho_mut(&mut p.scene, p.id).effects = vec![FxEntry::new(efeito)];
        let mede = |p: &mut BPalco, efeitos: bool, contacto: bool| {
            let mut sc = p.scene.clone();
            let leis = Leis {
                efeitos,
                contacto,
                ..Leis::do_ambiente()
            };
            let t = Instant::now();
            let mut n = 0_u32;
            while t.elapsed().as_millis() < 300 {
                p.dobra_em_s(if n.is_multiple_of(2) { 90.0 } else { 60.0 });
                let _ = crate::skin_live::recook_leis(&p.sim, &mut sc, leis);
                if !efeitos {
                    let _ = sc
                        .paths()
                        .iter()
                        .find(|q| q.id == p.id)
                        .map(VecPath::cooked);
                }
                n += 1;
            }
            t.elapsed().as_secs_f64() * 1e6 / f64::from(n)
        };
        let antiga = mede(&mut p, false, true);
        let nova = mede(&mut p, true, true);
        let sem_contacto = mede(&mut p, true, false);
        println!(
            "  {nome:>7}: µs/forma/quadro a mexer: antiga {antiga:.1} · nova {nova:.1} (sem a silhueta \
             {sem_contacto:.1}) · loadavg {}",
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
    }
}

/// ⭐ **SONDA — O PREÇO DO CAMPO DO CONTORNO COZIDO**, pago UMA vez por pilha nova (nunca por
/// pose). Imprime; corra em `--release` e com a máquina calma.
#[test]
fn diag_o_preco_do_campo_do_cozido() {
    for (nome, efeito) in efeitos().into_iter().chain(efeitos_fortes()) {
        let p = b_palco(false);
        let repouso = repouso_com(&p, &[FxEntry::new(efeito)]);
        let skin = p
            .sim
            .world()
            .get::<ph2d_skeleton_ecs::SkinBind>(p.alvo)
            .expect("pele");
        let eixos =
            crate::skin_live::eixos_do_bind(&p.sim, skin, &crate::skin_live::bone_index(&p.sim));
        let t = std::time::Instant::now();
        let c = ph2d_vec_skin::pesos::campo_do_caminho(&repouso, &eixos);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        let t = std::time::Instant::now();
        let _ = ph2d_vec_skin::pesos::campo_do_caminho(&p.fonte, &eixos);
        println!(
            "  {nome:>8}: campo do cozido {ms:.1} ms ({} vértices) · da fonte {:.1} ms · loadavg {}",
            c.as_ref().map_or(0, |c| c.malha.rest.len()),
            t.elapsed().as_secs_f64() * 1e3,
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
    }
}
