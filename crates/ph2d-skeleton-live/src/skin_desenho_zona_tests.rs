//! A ZONA DA DOBRA — onde a pele começa a dobrar sobre si mesma (report do dono de 2026-09-30, três
//! fotos com a junta de cima entre `84°` e `89°` e a de baixo a `~125°`).

use super::{Forma, com_e_sem_contacto_em, diagonal};
use ph2d_vec_scene::VecPath;

type P = [f64; 2];

fn cubica(p: &VecPath, i: usize) -> [P; 4] {
    let n = p.verts.len();
    let (c, q) = (&p.verts[i], &p.verts[(i + 1) % n]);
    [c.anchor, c.out_handle, q.in_handle, q.anchor]
}

fn d1(c: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    let f = |k: usize| {
        3.0 * u * u * (c[1][k] - c[0][k])
            + 6.0 * u * t * (c[2][k] - c[1][k])
            + 3.0 * t * t * (c[3][k] - c[2][k])
    };
    [f(0), f(1)]
}

fn d2(c: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    let f = |k: usize| {
        6.0 * u * (c[2][k] - 2.0 * c[1][k] + c[0][k])
            + 6.0 * t * (c[3][k] - 2.0 * c[2][k] + c[1][k])
    };
    [f(0), f(1)]
}

fn ponto(c: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    let f = |k: usize| {
        u * u * u * c[0][k]
            + 3.0 * u * u * t * c[1][k]
            + 3.0 * u * t * t * c[2][k]
            + t * t * t * c[3][k]
    };
    [f(0), f(1)]
}

/// O menor raio de curvatura CÔNCAVA do contorno de fora, amostrado por segmento (e onde fica).
pub(super) fn raio_concavo_minimo(p: &VecPath, so: impl Fn(P) -> bool) -> (f64, P) {
    let n = p.verts.len();
    let sinal: f64 = (0..n)
        .map(|i| {
            let (a, b) = (p.verts[i].anchor, p.verts[(i + 1) % n].anchor);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        .signum();
    let mut melhor = (f64::INFINITY, [0.0, 0.0]);
    for i in 0..n {
        let c = cubica(p, i);
        for k in 0..=64 {
            let t = f64::from(k) / 64.0;
            let (v, a) = (d1(&c, t), d2(&c, t));
            let cruz = v[0] * a[1] - v[1] * a[0];
            let nv = v[0].hypot(v[1]);
            if nv < 1e-12 {
                continue;
            }
            if cruz * sinal < 0.0 {
                let r = nv * nv * nv / cruz.abs();
                let q = ponto(&c, t);
                if r < melhor.0 && so(q) {
                    melhor = (r, q);
                }
            }
        }
    }
    melhor
}

/// 📏 SONDA — `PH2D_SONDA_ZONA=1`: varre a junta de cima de `70°` a `100°` com a de baixo a
/// `125°`, e imprime por ângulo: se cruza, as ilhas, a maior viragem côncava e o menor raio côncavo
/// (em raios do vinco).
#[test]
fn diag_a_zona_da_dobra() {
    if std::env::var("PH2D_SONDA_ZONA").is_err() {
        return;
    }
    let baixo: f32 = std::env::var("PH2D_SONDA_G")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(125.0);
    let mut g = 70.0_f32;
    while g <= 100.0 {
        let (sem, com) = com_e_sem_contacto_em(baixo, Forma::Dono(g));
        let r = ph2d_vec_boolean::overlap::RAIO_DO_VINCO * diagonal(&com);
        let ilhas: Vec<String> = com
            .subpaths
            .iter()
            .map(|c| {
                let v = VecPath {
                    verts: c.verts.clone(),
                    closed: true,
                    ..VecPath::default()
                };
                format!("{:.2e}", ph2d_vec_boolean::area(&v) / (r * r))
            })
            .collect();
        let vs = &com.verts;
        let (mut vmax, mut imax) = (0.0_f64, 0);
        for i in 0..vs.len() {
            let v = ph2d_vec_boolean::overlap::viragem_do_vertice(vs, i).unwrap_or(0.0);
            if v > vmax {
                (vmax, imax) = (v, i);
            }
        }
        // Só o canto de CIMA (a junta das fotos): a de baixo e o assado têm feições próprias.
        let cima = |q: P| q[1] > 3.5;
        let (rc, onde) = raio_concavo_minimo(&com, cima);
        let (rs, _) = raio_concavo_minimo(&sem, cima);
        println!(
            "g2 {g:5.1} cruza {} | verts {} | viragem max {vmax:6.1} em v{imax} {:?} | \
             raio côncavo min {:.3}r em ({:.3},{:.3}) (sem: {:.3}r) | ilhas {:?}",
            u8::from(com != sem),
            vs.len(),
            vs[imax].anchor.map(|x| (x * 1000.0).round() / 1000.0),
            rc / r,
            onde[0],
            onde[1],
            rs / r,
            ilhas
        );
        g += 1.0;
    }
}

/// A maior VIRAGEM de uma zona côncava mais apertada que `limite` (em graus), e onde: o contorno de
/// fora amostrado a `64` por segmento, as arestas que viram para DENTRO mais depressa que `limite`
/// agrupadas em corridas, e a viragem somada de cada uma. ⚠️ É a régua da bola medida por OUTRO
/// caminho (as tangentes da cúbica, não as da porta), e é por isso que o gate a usa.
pub(super) fn maior_zona_apertada(p: &VecPath, limite: f64) -> (f64, P) {
    let n = p.verts.len();
    let mut am: Vec<(P, P)> = Vec::new();
    for i in 0..n {
        let c = cubica(p, i);
        for k in 0..=64 {
            let t = f64::from(k) / 64.0;
            let v = d1(&c, t);
            let l = v[0].hypot(v[1]);
            if l > 1e-9 {
                am.push((ponto(&c, t), [v[0] / l, v[1] / l]));
            }
        }
    }
    let m = am.len();
    let sinal: f64 = (0..m)
        .map(|i| {
            let (a, b) = (am[i].0, am[(i + 1) % m].0);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        .signum();
    let (mut pior, mut onde, mut soma, mut ini) = (0.0_f64, [0.0; 2], 0.0_f64, [0.0; 2]);
    for i in 0..=m {
        let (a, b) = (am[i % m], am[(i + 1) % m]);
        let dth = (a.1[0] * b.1[1] - a.1[1] * b.1[0]).atan2(a.1[0] * b.1[0] + a.1[1] * b.1[1]);
        let ds = (b.0[0] - a.0[0]).hypot(b.0[1] - a.0[1]);
        if dth * sinal < -1e-9 && ds < limite * dth.abs() {
            if soma == 0.0 {
                ini = a.0;
            }
            soma += dth.abs();
        } else {
            if soma > pior {
                (pior, onde) = (soma, ini);
            }
            soma = 0.0;
        }
    }
    (pior.to_degrees(), onde)
}

/// Os pontos do contorno de fora, `16` por segmento.
pub(super) fn pontos(p: &VecPath) -> Vec<P> {
    (0..p.verts.len())
        .flat_map(|i| {
            let c = cubica(p, i);
            (0..16).map(move |k| ponto(&c, f64::from(k) / 16.0))
        })
        .collect()
}

/// A distância de Hausdorff entre dois contornos amostrados.
pub(super) fn hausdorff(a: &[P], b: &[P]) -> f64 {
    let lado = |x: &[P], y: &[P]| {
        x.iter()
            .map(|p| {
                y.iter()
                    .map(|q| (p[0] - q[0]).hypot(p[1] - q[1]))
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max)
    };
    lado(a, b).max(lado(b, a))
}

/// 📏 SONDA — `PH2D_SONDA_PASSO=1`: de grau em grau, quanto a SILHUETA anda contra quanto o
/// DESENHO anda (Hausdorff), na pose das fotos.
#[test]
fn diag_o_passo_da_silhueta() {
    if std::env::var("PH2D_SONDA_PASSO").is_err() {
        return;
    }
    let mut ant: Option<(Vec<P>, Vec<P>)> = None;
    let mut g = 60.0_f32;
    while g <= 150.0 {
        let (sem, com) = com_e_sem_contacto_em(125.0, Forma::Dono(g));
        let (ps, pc) = (pontos(&sem), pontos(&com));
        if let Some((a_s, a_c)) = &ant {
            let (hs, hc) = (hausdorff(a_s, &ps), hausdorff(a_c, &pc));
            println!(
                "g2 {g:5.1}  desenho {hs:.4}  silhueta {hc:.4}  razão {:.2}",
                hc / hs
            );
        }
        ant = Some((ps, pc));
        g += 1.0;
    }
}

/// ⭐⭐⭐ **GATE — nenhum canto da silhueta é mais apertado que a bola, ANTES e DEPOIS do
/// contacto** (report do dono de 2026-09-30, três fotos com a junta de cima a `84°`–`89°`: *«ainda
/// inconsistente… não é progressivo… artefatos circulares»*).
///
/// Nas quatro formas de dobra, de `60°` a `150°` (a pose das fotos de grau em grau): **(1)** a
/// silhueta é um PONTO FIXO da porta — rolar a bola outra vez não muda nada; **(2)** o menor raio
/// côncavo do contorno é pelo menos `0,9 r`, ou é o do desenho (uma feição que a bola não tocou,
/// porque a viragem dela cabe na solda). ⚠️ A F40 falhava a (2) de `90°` a `92°` (`0,053`/`0,033`/
/// `0,017 r` contra `0,022`/`0,009`/`0,002 r` do desenho): o arredondamento só corria no
/// cruzamento, e ali a pele aperta até ao bico SEM se cruzar. Piso de população: a bola TEM de ter
/// mexido em alguma dobra da faixa sem cruzamento, senão o gate varreria silhuetas intocadas.
#[test]
fn nenhum_canto_da_silhueta_e_mais_apertado_que_a_bola() {
    let mut casos: Vec<(Forma, f32)> = Vec::new();
    for forma in [Forma::C, Forma::Z, Forma::Uma] {
        let mut g = 60.0_f32;
        while g <= 150.0 {
            casos.push((forma, g));
            g += 5.0;
        }
    }
    let mut g = 60.0_f32;
    while g <= 150.0 {
        casos.push((Forma::Dono(g), 125.0));
        g += 1.0;
    }
    let mut antes_do_cruzamento = 0;
    for (forma, g) in casos {
        let (sem, com) = com_e_sem_contacto_em(g, forma);
        let r = ph2d_vec_boolean::overlap::RAIO_DO_VINCO * diagonal(&sem);
        // ⚠️ `&[]` é EXACTAMENTE as quinas do repouso desta barra: as tampas viram `≤ 1,45°`, abaixo
        // da `PAREDE_MINIMA`, logo nenhuma é parede e a lista do produto não protegeria nada.
        assert!(
            ph2d_vec_boolean::silhueta_da_pele(&com, &[]).is_none(),
            "{forma:?} {g}°: rolar a bola outra vez mudou a silhueta — ela não é um ponto fixo"
        );
        let (vira, onde) = maior_zona_apertada(&com, 0.9 * r);
        assert!(
            vira < 12.0,
            "{forma:?} {g}°: uma zona côncava mais apertada que a bola vira {vira:.1}° em {onde:?} \
             — um canto ficou"
        );
        if com != sem
            && ph2d_vec_boolean::resolve_overlap(&sem).is_none()
            && maior_zona_apertada(&sem, 0.9 * r).0 >= 12.0
        {
            antes_do_cruzamento += 1;
        }
    }
    assert!(
        antes_do_cruzamento >= 1,
        "a bola não mexeu em nenhuma dobra antes do cruzamento — o gate não viu o fenómeno"
    );
}

/// A maior viragem de um vértice do contorno de fora, em graus.
fn viragem_maxima(p: &VecPath) -> f64 {
    (0..p.verts.len())
        .filter_map(|i| ph2d_vec_boolean::overlap::viragem_do_vertice(&p.verts, i))
        .fold(0.0, f64::max)
}

/// O braço da dobra forte na pose `(primeira, segunda)`, sem e com a silhueta — o produto inteiro.
fn braco_em(primeira: f32, segunda: f32) -> (VecPath, VecPath) {
    let (mut sim, mut scene, _map, id, ossos) =
        crate::barra_da_cena_tests_support::braco_da_dobra_forte(0.3);
    for (k, g) in [(1, primeira), (2, -segunda)] {
        sim.world_mut()
            .get_mut::<ph2d_ecs::Transform>(ossos[k])
            .expect("Transform")
            .rotation += g.to_radians();
    }
    let sem = crate::skin_live::recook_leis(
        &sim,
        &mut scene,
        super::Leis {
            contacto: false,
            ..super::PRODUTO
        },
    );
    let com = crate::skin_live::recook_leis(&sim, &mut scene, super::PRODUTO);
    (
        sem.get(&id).expect("sem contacto").forma.clone(),
        com.get(&id).expect("com contacto").forma.clone(),
    )
}

/// ⭐⭐⭐ **GATE — o GANCHO da dobra não é QUINA DO ARTISTA** (F42, report do dono de 2026-09-30 com
/// duas fotos: *«melhorou muito o ângulo e suas transições; restam os artefatos de imagem»* — fatias
/// de cinzento e de laranja dentro do castanho, no vinco).
///
/// ⛔ **A causa:** na dobra do mapa um nó do ASSADO vira `180°` (um gancho de raio `~0,005` no vinco,
/// medido na cena `=4` a `(125°, 85°)`), e a bola lia a viragem das quinas no desenho DEFORMADO — o
/// gancho passava por quina desenhada, ficava, e o traço sobre a meia-volta abria as fatias. Hoje as
/// quinas são os nós da FONTE com a viragem do REPOUSO.
///
/// ⭐ **Medido** (varredura de `891` poses, `100°`–`150°` × `70°`–`110°` de meio em meio grau): o
/// gancho aparece no desenho em `16`, a lei de antes deixa-o ficar em `11` — cada uma um buraco no
/// traço —, e o produto em `0`. Aqui corre a faixa da foto (`125°`, `70°`–`110°`), onde ele vive entre
/// `84°` e `85°`: um defeito de UM grau que só uma varredura fina vê.
///
/// ⚠️ **As três metades:** o CONTROLO de que a fixtura contém o gancho (o desenho sem contacto vira
/// `> 150°` em alguma pose da faixa — a [`barra_da_cena_com`] não o contém, e é por isso que este
/// braço tem as proporções da cena); o CONTROLO de mecanismo (com as quinas lidas no deformado, a lei
/// de antes, o gancho FICA); e o produto — nenhum vértice vira mais que a `PAREDE_MINIMA` (este braço
/// não tem quina desenhada) nem sobra zona côncava mais apertada que a bola.
///
/// [`barra_da_cena_com`]: crate::barra_da_cena_tests_support::barra_da_cena_com
#[test]
fn o_gancho_da_dobra_nao_e_quina_do_artista() {
    let (mut ganchos, mut ficavam) = (0, 0);
    for passo in 0..=160_u16 {
        let segunda = 70.0 + 0.5 * f32::from(passo);
        let (sem, com) = braco_em(125.0, segunda);
        let velho = ph2d_vec_boolean::silhueta_da_pele(&sem, &ph2d_vec_boolean::quinas_de(&sem))
            .unwrap_or_else(|| sem.clone());
        println!(
            "  {segunda}°: desenho {:.1}° · lei de antes {:.1}° · produto {:.1}°",
            viragem_maxima(&sem),
            viragem_maxima(&velho),
            viragem_maxima(&com)
        );
        ganchos += usize::from(viragem_maxima(&sem) > 150.0);
        ficavam += usize::from(viragem_maxima(&velho) > 150.0);
        let vira = viragem_maxima(&com);
        assert!(
            vira < ph2d_vec_boolean::overlap::PAREDE_MINIMA,
            "{segunda}°: um vértice da silhueta vira {vira:.1}° — o gancho da dobra ficou como quina"
        );
        let r = ph2d_vec_boolean::overlap::RAIO_DO_VINCO * diagonal(&com);
        let (zona, onde) = maior_zona_apertada(&com, 0.9 * r);
        assert!(
            zona < 12.0,
            "{segunda}°: uma zona côncava mais apertada que a bola vira {zona:.1}° em {onde:?}"
        );
    }
    // ⭐⭐⭐ F50-e (2026-10-03): o CONTROLO INVERTEU-SE. O gancho nascia no AJUSTE — uma cúbica que
    // recua na ponta, menor que a tolerância — e o `fecha` do bake passou a exigir que a cúbica ande
    // no sentido da fonte (`ph2d_vec_skin::curva_segundo_corpo::anda_para_a_frente`). Medido: em
    // nenhuma pose da faixa o desenho SEM contacto vira `> 150°`. O passe dos ganchos fica como
    // rede (a união ainda pode recortar uma cúbica), e esta metade afirma agora a cura na ORIGEM.
    assert_eq!(
        (ganchos, ficavam),
        (0, 0),
        "o gancho voltou a nascer no DESENHO — o ajuste aceitou uma cúbica que recua"
    );
}

/// ⭐⭐ **GATE — a quina que o artista DESENHOU continua em bico** (F42). A metade que impede a cura
/// do gancho de virar «nenhuma quina é protegida»: um «L» com a quina côncava de `90°`, preso a dois
/// ossos, sai com a quina inteira — e o CONTROLO (a mesma silhueta sem lista de quinas) arredonda-a,
/// senão o gate não mediria a protecção.
///
/// ⚠️ **A raiz RODA `30°`:** o movimento é rígido (a quina muda de SÍTIO e não de ângulo), e é isso
/// que separa «a quina está onde o assado a POUSOU» de «a quina está onde a FONTE a tem» — em repouso
/// os dois coincidem e uma lista com as posições do repouso passaria.
#[test]
fn a_quina_desenhada_continua_em_bico() {
    let (a, giro) = ([0.5_f64, 0.5], 30.0_f64.to_radians());
    let em_repouso = [1.0, 1.0];
    let quina = [
        a[0] + (em_repouso[0] - a[0]) * giro.cos() - (em_repouso[1] - a[1]) * giro.sin(),
        a[1] + (em_repouso[0] - a[0]) * giro.sin() + (em_repouso[1] - a[1]) * giro.cos(),
    ];
    let l: Vec<ph2d_vec_scene::VecVertex> = [
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 1.0],
        em_repouso,
        [1.0, 3.0],
        [0.0, 3.0],
    ]
    .into_iter()
    .map(ph2d_vec_scene::VecVertex::corner)
    .collect();
    let peca = VecPath {
        verts: l,
        closed: true,
        ..VecPath::default()
    };
    let (mut sim, mut scene, _map, id, ossos) =
        crate::barra_da_cena_tests_support::peca_presa(peca, &[a, [2.0, 0.5], [3.5, 0.5]]);
    #[expect(clippy::cast_possible_truncation, reason = "um ângulo de teste")]
    let giro_f32 = giro as f32;
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ossos[0])
        .expect("Transform")
        .rotation += giro_f32;
    let com = crate::skin_live::recook_leis(&sim, &mut scene, super::PRODUTO);
    let com = com.get(&id).expect("desenho");
    let perto = |p: &VecPath| {
        (0..p.verts.len())
            .filter(|&i| {
                let a = p.verts[i].anchor;
                (a[0] - quina[0]).hypot(a[1] - quina[1]) < 1e-4
            })
            .filter_map(|i| ph2d_vec_boolean::overlap::viragem_do_vertice(&p.verts, i))
            .fold(0.0, f64::max)
    };
    let vira = perto(com);
    assert!(
        (vira - 90.0).abs() < 1.0,
        "a quina desenhada do «L» saiu a {vira:.1}° — a bola comeu-a"
    );
    let sem_quinas = ph2d_vec_boolean::silhueta_da_pele(com, &[]).expect("controlo: a bola actua");
    assert!(
        perto(&sem_quinas) < 1.0,
        "sem lista de quinas a bola não arredonda o «L» — o gate não mede a protecção"
    );
}

/// ⭐⭐⭐ **GATE — com a junta QUASE RECTA não sobra meia-lua** (F43, report do dono de 2026-09-30, com
/// foto: *«quase perfeito, artefatos curados na quina dobrada; resquício quando quase reto»*).
///
/// ⛔ Na dobra do MAPA a velocidade do contorno chega a zero mesmo com a junta a `~35°`, e o assado
/// devolve uma cúbica que DOBRA — a tangente invertida num nó, ou um zigue-zague de `0,0016` de largura
/// por dentro dela. Invisível como forma; o traço desenha-o como uma meia-lua. Não é vinco côncavo,
/// logo a bola não lhe toca, e a união ainda o piorava (lia o zigue-zague como cruzamento e
/// reescrevia-o num dardo real).
///
/// ⭐ **Medido** (`110°`/`125°`/`140°` × `0°`–`175°` de meio em meio grau, `1 053` poses): o produto
/// tinha nós a virar `~180°` a `110°` com a de cima a `10°`–`17,5°` e `34°`–`36°`, e a `125°`/`140°` a
/// `34°`–`36°`; agora em nenhuma pose até `175°`. Aqui correm `110°` e `125°` × `0°`–`70°`. ⚠️ **As metades:** o CONTROLO de que a fixtura contém o fenómeno (o desenho vira
/// `> 150°` em alguma pose) e o de MECANISMO (a união e a bola sem o passe deixam-no em alguma pose);
/// e o produto — nenhum vértice vira mais que a `PAREDE_MINIMA`.
#[test]
fn com_a_junta_quase_recta_nao_sobra_meia_lua() {
    let (mut no_desenho, mut sem_passe) = (0, 0);
    for (primeira, passo) in [110.0_f32, 125.0]
        .into_iter()
        .flat_map(|a| (0..=140_u16).map(move |b| (a, b)))
    {
        let segunda = 0.5 * f32::from(passo);
        let (sem, com) = braco_em(primeira, segunda);
        no_desenho += usize::from(viragem_maxima(&sem) > 150.0);
        let d = diagonal(&sem);
        let unido = ph2d_vec_boolean::resolve_overlap(&sem).unwrap_or_else(|| sem.clone());
        let so_bola = VecPath {
            verts: ph2d_vec_boolean::bola::rola_a_bola(
                unido.verts.clone(),
                &[],
                ph2d_vec_boolean::overlap::RAIO_DO_VINCO * d,
                ph2d_vec_boolean::overlap::SOLDA_DA_QUINA * d,
            ),
            ..unido
        };
        sem_passe += usize::from(viragem_maxima(&so_bola) > 150.0);
        let vira = viragem_maxima(&com);
        assert!(
            vira < ph2d_vec_boolean::overlap::PAREDE_MINIMA,
            "({primeira}°, {segunda}°): um vértice da silhueta vira {vira:.1}° — sobrou a meia-lua"
        );
    }
    // ⭐⭐⭐ F50-e (2026-10-03): o CONTROLO INVERTEU-SE. O gancho nascia no AJUSTE — uma cúbica que
    // recua na ponta, menor que a tolerância — e o `fecha` do bake passou a exigir que a cúbica ande
    // no sentido da fonte (`ph2d_vec_skin::curva_segundo_corpo::anda_para_a_frente`). Medido: em
    // nenhuma pose da faixa o desenho SEM contacto vira `> 150°`. O passe dos ganchos fica como
    // rede (a união ainda pode recortar uma cúbica), e esta metade afirma agora a cura na ORIGEM.
    assert_eq!(
        (no_desenho, sem_passe),
        (0, 0),
        "a meia-volta voltou a nascer no DESENHO — o ajuste aceitou uma cúbica que recua"
    );
}

/// ⭐⭐ **GATE — com as duas juntas a dobrar NO MESMO SENTIDO não sobra a meia-lua** (F43-bis, report do
/// dono de 2026-09-30 com foto: *«quase perfeito»*, uma meia-lua no lado de dentro, por cima da junta
/// de cima). A varredura da F43 só dobrava a junta de cima ao CONTRÁRIO da de baixo; medida a outra
/// metade (`60°`–`150°` × `−40°`–`10°`), o defeito vivia em `−10,5°` com a de baixo de `60°` a `84°` —
/// a pose da foto. Ali a cúbica recua só no último `1,4 %` do parâmetro, abaixo da amostragem do
/// passe, e quem o vê são as tangentes EXACTAS das pontas ([`ph2d_vec_boolean::gancho`]).
///
/// ⚠️ O CONTROLO: o desenho sem contacto vira `> 150°` em alguma pose da faixa.
#[test]
fn com_as_duas_juntas_no_mesmo_sentido_nao_sobra_meia_lua() {
    let mut no_desenho = 0;
    for primeira in [72.0_f32, 84.0] {
        for passo in 14..=28_u16 {
            let segunda = -0.5 * f32::from(passo);
            let (sem, com) = braco_em(primeira, segunda);
            no_desenho += usize::from(viragem_maxima(&sem) > 150.0);
            let vira = viragem_maxima(&com);
            assert!(
                vira < ph2d_vec_boolean::overlap::PAREDE_MINIMA,
                "({primeira}°, {segunda}°): um vértice da silhueta vira {vira:.1}° — sobrou a meia-lua"
            );
        }
    }
    // ⭐⭐⭐ F50-e (2026-10-03): o CONTROLO INVERTEU-SE. O gancho nascia no AJUSTE — uma cúbica que
    // recua na ponta, menor que a tolerância — e o `fecha` do bake passou a exigir que a cúbica ande
    // no sentido da fonte (`ph2d_vec_skin::curva_segundo_corpo::anda_para_a_frente`). Medido: em
    // nenhuma pose da faixa o desenho SEM contacto vira `> 150°`. O passe dos ganchos fica como
    // rede (a união ainda pode recortar uma cúbica), e esta metade afirma agora a cura na ORIGEM.
    assert_eq!(
        no_desenho, 0,
        "a meia-volta voltou a nascer no DESENHO — o ajuste aceitou uma cúbica que recua"
    );
}

/// ⭐⭐⭐ **GATE — nas ILHAS a bola rola por dentro** (F44, report do dono de 2026-10-01, três fotos:
/// *«quando uma parte do membro se sobrepõe a outra formando uma ilha, nessa ilha as quinas ainda
/// não estão corretas»*). O braço dobrado em C fecha-se sobre si e a união deixa um BURACO; a bola
/// rolava só no contorno de fora, e as ilhas ficavam com cantos até `153°` (a junta do traço abria o
/// espinho para dentro do preenchimento).
///
/// **Medido** (`3 540` poses em C, `60°`–`176°` × `60°`–`178°` de dois em dois graus): ilhas com canto
/// acima da `PAREDE_MINIMA` de `595` poses para `0`, e o contorno de fora IGUAL ao bit (os mesmos
/// `113` nós — o braço dobrado de volta sobre si, aberto e anterior a esta wave).
///
/// ⚠️ As três metades: o CONTROLO de que a fixtura contém o fenómeno (a união crua deixa uma ilha
/// em bico); que alguma ilha SOBREVIVE arredondada (senão o gate passaria por apagar todas); e que
/// alguma é FECHADA (a bola não cabe — ver `ph2d_vec_boolean::ilha`).
#[test]
fn nas_ilhas_a_bola_rola_por_dentro() {
    let (mut em_bico, mut ficou, mut fechou) = (0, 0, 0);
    for a in (90..=130_u16).step_by(4) {
        for b in (90..=130_u16).step_by(4) {
            let (primeira, segunda) = (f32::from(a), -f32::from(b));
            let (sem, com) = braco_em(primeira, segunda);
            let crua = ph2d_vec_boolean::resolve_overlap(&sem).map_or(0, |u| u.subpaths.len());
            if let Some(u) = ph2d_vec_boolean::resolve_overlap(&sem) {
                em_bico += usize::from(u.subpaths.iter().any(|c| {
                    let p = VecPath {
                        verts: c.verts.clone(),
                        ..VecPath::default()
                    };
                    viragem_maxima(&p) > 100.0
                }));
            }
            ficou += usize::from(!com.subpaths.is_empty());
            fechou += usize::from(crua > com.subpaths.len());
            for c in &com.subpaths {
                let p = VecPath {
                    verts: c.verts.clone(),
                    ..VecPath::default()
                };
                let vira = viragem_maxima(&p);
                assert!(
                    vira < ph2d_vec_boolean::overlap::PAREDE_MINIMA,
                    "({primeira}°, {segunda}°): um canto da ilha vira {vira:.1}°"
                );
            }
        }
    }
    assert!(
        em_bico >= 1,
        "nenhuma ilha em bico — a fixtura deixou de conter o fenómeno"
    );
    assert!(
        ficou >= 1,
        "nenhuma ilha sobreviveu — o gate não mede o arredondamento"
    );
    assert!(
        fechou >= 1,
        "nenhuma ilha foi fechada — o gate não mede a ilha onde a bola não cabe"
    );
}

#[path = "skin_desenho_dobra_tests.rs"]
mod dobra;

#[cfg(test)]
#[path = "skin_desenho_esporao_sondas_tests.rs"]
mod esporao_sondas;
