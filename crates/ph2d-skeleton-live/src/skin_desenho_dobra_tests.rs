//! Os gates do braço dobrado FORTE (F45–F46) — o gancho que a união deixa e o braço dobrado de
//! VOLTA —, mais a sonda da varredura das duas juntas. Saíram de [`super`] por assunto (tecto de
//! LOC); a fixtura é a dele ([`super::braco_em`]).

use super::super::diagonal;
use super::{braco_em, viragem_maxima};
use crate::skin_desenho::AMOSTRAS_POR_FORMA;
use ph2d_skeleton::MisturaDoAngulo;

/// O braço SEM o contacto na média em CÍRCULO e com metade das amostras do produto — onde os
/// fenómenos que o gancho e a cunha da união curam EXISTEM (medido: no produto, meio-ângulo com
/// `1024` amostras, `0` de `1 121` poses em gancho acima de `150°` e a cunha a `≤ 8°`).
fn controlo(primeira: f32, segunda: f32) -> VecPath {
    dentes::entrada_na_lei(
        primeira,
        segunda,
        AMOSTRAS_POR_FORMA / 2,
        MisturaDoAngulo::Circulo,
    )
    .0
}
use ph2d_vec_scene::VecPath;

/// ⭐⭐ **GATE — o gancho que a UNIÃO deixa sai** (F45). Com o braço em C e as duas juntas a somar
/// `~238°`, a união corta uma cúbica do assado DENTRO da dobra dela e o nó novo sai com a alça
/// além dele: uma meia-volta de `171°`–`179°` no contorno de fora, e o traço desenha a meia-lua.
/// A cura é o desfazer dos ganchos correr TAMBÉM depois da união.
///
/// ⚠️ O CONTROLO: a união com a bola mas SEM essa passagem deixa a meia-volta em alguma pose da
/// faixa — senão o gate não mede o mecanismo. Ele corre no [`controlo`] (a média em círculo): o
/// meio-ângulo do produto espalha a volta e a união já não deixa o gancho na faixa, e o gate
/// continua a provar que a passagem o tira onde ele existe.
#[test]
fn o_gancho_que_a_uniao_deixa_sai() {
    let mut sem_a_passagem = 0;
    for a in (130..=170_u16).step_by(2) {
        for soma in [236_u16, 238, 240] {
            let (primeira, segunda) = (f32::from(a), -f32::from(soma - a));
            let (_, com) = braco_em(primeira, segunda);
            let sem = controlo(primeira, segunda);
            let d = diagonal(&sem);
            if let Some(u) = ph2d_vec_boolean::resolve_overlap(&sem) {
                let so_bola = VecPath {
                    verts: ph2d_vec_boolean::bola::rola_a_bola(
                        u.verts.clone(),
                        &[],
                        ph2d_vec_boolean::overlap::RAIO_DO_VINCO * d,
                        ph2d_vec_boolean::overlap::SOLDA_DA_QUINA * d,
                    ),
                    ..u
                };
                sem_a_passagem += usize::from(viragem_maxima(&so_bola) > 150.0);
            }
            let vira = viragem_maxima(&com);
            assert!(
                vira < ph2d_vec_boolean::overlap::PAREDE_MINIMA,
                "({primeira}°, {segunda}°): um nó da silhueta vira {vira:.1}° — sobrou o gancho"
            );
        }
    }
    assert!(
        sem_a_passagem >= 1,
        "sem a passagem depois da união nenhuma pose fica em gancho — o controlo deixou de medir"
    );
}

/// ⭐⭐⭐ **GATE — o braço dobrado de VOLTA não deixa dentes** (F46). Com uma junta a `174°`–`178°` a
/// pele dos dois membros quase coincide e a união deixa, no contorno de fora, dentes e fendas de
/// `~0,02`, um esporão que VOLTA pelo próprio caminho (`180°`, medido a `(0°, 174°)`) e o fundo de um
/// canal que se fecha até ao vinco (`112°` a `(176°, −142°)`). Era a família que a varredura da F45
/// deixou: `574` das `575` poses más dela tinham uma junta a `174°` ou mais.
///
/// ⭐ As quatro metades da cura e o caso de cada uma: a ABERTURA depois do fecho (os dentes), o
/// [`ph2d_vec_boolean::esporao`] (`(0°, 174°)`), o offset de um nó CONVEXO como ARCO (a fenda de
/// boca estreita) e o centro EXACTO quando nenhuma corda passa (`(176°, −142°)`, `(144°, −174°)`).
///
/// ⚠️ O CONTROLO: a união com o fecho de antes — sem abertura, sem esporão — deixa um nó acima da
/// `PAREDE_MINIMA` em alguma pose da faixa.
#[test]
fn o_braco_dobrado_de_volta_nao_deixa_dentes() {
    let mut poses: Vec<(f32, f32)> = vec![
        (0.0, 174.0),
        (144.0, -174.0),
        (176.0, -142.0),
        (39.0, -179.0),
    ];
    for a in [174_u16, 176, 178] {
        poses.extend(
            (0..=356_u16)
                .step_by(12)
                .map(|b| (f32::from(a), f32::from(b) - 178.0)),
        );
    }
    for b in [174.0_f32, 176.0, 178.0, -174.0, -176.0, -178.0] {
        poses.extend((0..=172_u16).step_by(12).map(|a| (f32::from(a), b)));
    }
    let mut antes = 0;
    for (primeira, segunda) in poses {
        let (sem, com) = braco_em(primeira, segunda);
        let d = diagonal(&sem);
        let (raio, solda) = (
            ph2d_vec_boolean::overlap::RAIO_DO_VINCO * d,
            ph2d_vec_boolean::overlap::SOLDA_DA_QUINA * d,
        );
        if let Some(u) = ph2d_vec_boolean::resolve_overlap(&sem) {
            let g = ph2d_vec_boolean::gancho::desfaz_os_ganchos(u.verts.clone(), &[], solda);
            let so_fecho = VecPath {
                verts: ph2d_vec_boolean::bola::rola_a_bola(g, &[], raio, solda),
                ..u
            };
            antes +=
                usize::from(viragem_maxima(&so_fecho) > ph2d_vec_boolean::overlap::PAREDE_MINIMA);
        }
        let vira = viragem_maxima(&com);
        assert!(
            vira < ph2d_vec_boolean::overlap::PAREDE_MINIMA,
            "({primeira}°, {segunda}°): um nó da silhueta vira {vira:.1}° — sobrou um dente"
        );
    }
    assert!(
        antes >= 1,
        "com o fecho de antes nenhuma pose fica em bico — o controlo deixou de medir"
    );
}

/// ⭐⭐ **GATE — a cunha onde os dois membros se TOCAM fecha** (F47, o último aberto da varredura:
/// `(36°, −144°)`, `21°`). Os membros encostam e a união deixa entre eles uma cunha de `~20°`; a
/// cúbica que chega ao fundo dela passa `~1` solda ALÉM do nó e volta pela mesma recta, e a quina
/// verdadeira (`~160°`) fica escondida dentro da cúbica — a bola não a lê como vinco e o nó vira só
/// `21°`. A cura é aparar a cúbica onde ela passa pelo nó ([`ph2d_vec_boolean::esporao`]).
///
/// ⚠️ O CONTROLO: a união com o desfazer dos ganchos e a bola, sem o aparar, deixa a quina na pose
/// medida — no [`controlo`] (a média em círculo), onde a cunha existe.
#[test]
fn a_cunha_onde_os_membros_se_tocam_fecha() {
    let sem = controlo(36.0, -144.0);
    let d = diagonal(&sem);
    let (raio, solda) = (
        ph2d_vec_boolean::overlap::RAIO_DO_VINCO * d,
        ph2d_vec_boolean::overlap::SOLDA_DA_QUINA * d,
    );
    let u = ph2d_vec_boolean::resolve_overlap(&sem).expect("os membros tocam-se");
    let g = ph2d_vec_boolean::gancho::desfaz_os_ganchos(u.verts.clone(), &[], solda);
    let so_bola = VecPath {
        verts: ph2d_vec_boolean::bola::rola_a_bola(g, &[], raio, solda),
        ..u
    };
    assert!(
        viragem_maxima(&so_bola) > ph2d_vec_boolean::overlap::PAREDE_MINIMA,
        "sem o aparar a cunha já fecha — o controlo deixou de medir"
    );
    for a in [35.0_f32, 36.0, 37.0] {
        for b in [-145.0_f32, -144.0, -143.0] {
            let (_, com) = braco_em(a, b);
            let vira = viragem_maxima(&com);
            assert!(
                vira < ph2d_vec_boolean::overlap::PAREDE_MINIMA,
                "({a}°, {b}°): um nó da silhueta vira {vira:.1}° — a cunha ficou aberta"
            );
        }
    }
}

/// **Sonda — a VARREDURA da dobra** (F45, `#[ignore]`): as duas juntas do braço da cena em todas
/// as poses, de `PASSO` em `PASSO` graus (`2` por omissão; a 2.ª nos DOIS sentidos, C e Z), e imprime
/// cada pose em que um nó do contorno de fora ou de uma ilha vira acima da `PAREDE_MINIMA`. Medido
/// em 2026-10-01 a passo `2`: `575` de `16 110` antes da F46, `574` deles com uma junta a `174°` ou
/// mais (o braço dobrado de volta); depois dela `1`, a `(36°, −144°)`, isolada e anterior; depois
/// da F47 **`0`** (pior viragem `10,9°`). ⚠️ `~10 min` a passo `2`; corra-a com a máquina calma.
#[test]
#[ignore]
fn diag_a_varredura_da_dobra() {
    let vm = |vs: &[ph2d_vec_scene::VecVertex]| {
        (0..vs.len())
            .filter_map(|i| ph2d_vec_boolean::overlap::viragem_do_vertice(vs, i))
            .fold(0.0f64, f64::max)
    };
    let passo: u16 = std::env::var("PASSO")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);
    let (mut poses, mut maus) = (0, 0);
    for a in (0..=178_u16).step_by(passo as usize) {
        for b in (0..=356_u16).step_by(passo as usize) {
            let (primeira, segunda) = (f32::from(a), f32::from(b) - 178.0);
            let (_sem, com) = braco_em(primeira, segunda);
            poses += 1;
            let fora = vm(&com.verts);
            let ilha = com
                .subpaths
                .iter()
                .map(|c| vm(&c.verts))
                .fold(0.0, f64::max);
            if fora.max(ilha) > 15.0 {
                maus += 1;
                eprintln!("  ({a},{segunda}) fora={fora:.1} ilha={ilha:.1}");
            }
        }
    }
    eprintln!("poses={poses} maus={maus}");
}

#[path = "skin_desenho_dentes_sondas_tests.rs"]
mod dentes;

#[path = "skin_desenho_densidade_tests.rs"]
mod densidade;

/// SONDA (A13) — os controlos do gancho e da cunha na média em CÍRCULO e no produto: onde o
/// fenómeno existe, numa varredura fina.
#[test]
#[ignore = "sonda: imprime"]
fn diag_os_controlos_por_lei() {
    let ctl = |sem: &VecPath| -> Option<(f64, f64)> {
        let d = diagonal(sem);
        let (raio, solda) = (
            ph2d_vec_boolean::overlap::RAIO_DO_VINCO * d,
            ph2d_vec_boolean::overlap::SOLDA_DA_QUINA * d,
        );
        let u = ph2d_vec_boolean::resolve_overlap(sem)?;
        let so_bola = VecPath {
            verts: ph2d_vec_boolean::bola::rola_a_bola(u.verts.clone(), &[], raio, solda),
            ..u.clone()
        };
        let g = ph2d_vec_boolean::gancho::desfaz_os_ganchos(u.verts.clone(), &[], solda);
        let so_fecho = VecPath {
            verts: ph2d_vec_boolean::bola::rola_a_bola(g, &[], raio, solda),
            ..u
        };
        Some((viragem_maxima(&so_bola), viragem_maxima(&so_fecho)))
    };
    for (lei, por_forma) in [
        (MisturaDoAngulo::Circulo, AMOSTRAS_POR_FORMA / 2),
        (MisturaDoAngulo::Circulo, AMOSTRAS_POR_FORMA),
        (MisturaDoAngulo::MeioAngulo, AMOSTRAS_POR_FORMA),
    ] {
        let na_lei = |p1: f32, p2: f32| dentes::entrada_na_lei(p1, p2, por_forma, lei).0;
        let (mut g150, mut gp, mut cp, mut pior_g, mut pior_c) =
            (0, 0, 0, (0.0_f64, (0.0, 0.0)), (0.0_f64, (0.0, 0.0)));
        for a in (120..=178_u16).step_by(2) {
            for soma in (220..=256_u16).step_by(4) {
                let (p1, p2) = (f32::from(a), -f32::from(soma - a));
                if let Some((g, c)) = ctl(&na_lei(p1, p2)) {
                    g150 += usize::from(g > 150.0);
                    gp += usize::from(g > ph2d_vec_boolean::overlap::PAREDE_MINIMA);
                    if g > pior_g.0 {
                        pior_g = (g, (p1, p2));
                    }
                    let _ = c;
                }
            }
        }
        for a in (20..=60_u16).step_by(2) {
            for b in (124..=164_u16).step_by(2) {
                let (p1, p2) = (f32::from(a), -f32::from(b));
                if let Some((_, c)) = ctl(&na_lei(p1, p2)) {
                    cp += usize::from(c > ph2d_vec_boolean::overlap::PAREDE_MINIMA);
                    if c > pior_c.0 {
                        pior_c = (c, (p1, p2));
                    }
                }
            }
        }
        println!(
            "{lei:?} {por_forma}: gancho >150° {g150} · >parede {gp} (pior {:.1}° {:?}) · cunha >parede {cp} (pior {:.1}° {:?})",
            pior_g.0, pior_g.1, pior_c.0, pior_c.1
        );
    }
}

/// SONDA (A13) — as passagens que os gates do gancho, da cunha e do braço provam MEXEM no produto?
/// O braço da dobra forte em todas as poses de `6°` em `6°`, a silhueta refeita com cada passagem
/// desligada: em quantas poses o desenho final muda.
#[test]
#[ignore = "sonda: imprime (minutos)"]
fn diag_as_passagens_no_produto() {
    use ph2d_vec_boolean::overlap::{RAIO_DO_VINCO, SOLDA_DA_QUINA};
    let silhueta = |d: &VecPath,
                    quinas: &[([f64; 2], f64)],
                    sem: usize|
     -> Vec<ph2d_vec_scene::VecVertex> {
        let dg = dentes::caixa_diag(d);
        let (solda, raio) = (SOLDA_DA_QUINA * dg, RAIO_DO_VINCO * dg);
        let mut desenho = d.clone();
        desenho.verts = ph2d_vec_boolean::gancho::desfaz_os_ganchos(d.verts.clone(), quinas, solda);
        let unido = ph2d_vec_boolean::resolve_overlap(&desenho).map(|mut u| {
            if sem != 1 {
                u.verts = ph2d_vec_boolean::gancho::desfaz_os_ganchos(
                    std::mem::take(&mut u.verts),
                    quinas,
                    solda,
                );
            }
            if sem != 2 {
                u.verts = ph2d_vec_boolean::esporao::tira_os_esporoes(
                    std::mem::take(&mut u.verts),
                    quinas,
                    solda,
                );
            }
            u
        });
        let base = unido.as_ref().unwrap_or(&desenho);
        let rolado = ph2d_vec_boolean::bola::rola_a_bola(base.verts.clone(), quinas, raio, solda);
        if sem == 3 {
            return rolado;
        }
        let paredes: Vec<([f64; 2], f64)> = quinas.iter().map(|&(p, _)| (p, 180.0)).collect();
        let aberto =
            ph2d_vec_boolean::bola::rola_a_bola_por_dentro(rolado.clone(), &paredes, raio, solda);
        if aberto == rolado {
            rolado
        } else {
            ph2d_vec_boolean::bola::rola_a_bola(aberto, quinas, raio, solda)
        }
    };
    let nomes = [
        "todas",
        "sem o gancho depois da união",
        "sem o esporão (e o aparar)",
        "sem a abertura",
    ];
    let (mut muda, mut pior, mut poses, mut confere) = ([0usize; 4], [0.0_f64; 4], 0, 0);
    for a in (0..=178_u16).step_by(6) {
        for b in (0..=356_u16).step_by(6) {
            let (p1, p2) = (f32::from(a), f32::from(b) - 178.0);
            let (d, quinas) = dentes::entrada(p1, p2, AMOSTRAS_POR_FORMA);
            if ph2d_vec_boolean::resolve_overlap(&d).is_none() {
                continue;
            }
            poses += 1;
            let todas = silhueta(&d, &quinas, 0);
            if let Some(s) = ph2d_vec_boolean::silhueta_da_pele(&d, &quinas) {
                let ancoras = |v: &[ph2d_vec_scene::VecVertex]| {
                    v.iter().map(|x| x.anchor).collect::<Vec<_>>()
                };
                let igual = s.verts.len() == todas.len()
                    && ancoras(&s.verts)
                        .iter()
                        .zip(ancoras(&todas))
                        .all(|(x, y)| (x[0] - y[0]).hypot(x[1] - y[1]) < 1e-12);
                confere += usize::from(igual);
            }
            for k in 1..4 {
                let v = silhueta(&d, &quinas, k);
                if v != todas {
                    muda[k] += 1;
                    let vm = viragem_maxima(&VecPath {
                        verts: v,
                        ..d.clone()
                    });
                    pior[k] = pior[k].max(vm);
                }
            }
        }
    }
    println!("{poses} poses com contacto · a sonda refaz o produto em {confere}");
    for k in 1..4 {
        println!(
            "  {}: muda o desenho em {} poses · pior viragem sem ela {:.1}°",
            nomes[k], muda[k], pior[k]
        );
    }
}
