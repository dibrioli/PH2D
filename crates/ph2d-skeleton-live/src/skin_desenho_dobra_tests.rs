//! Os gates do braço dobrado FORTE (F45–F46) — o gancho que a união deixa e o braço dobrado de
//! VOLTA —, mais a sonda da varredura das duas juntas. Saíram de [`super`] por assunto (tecto de
//! LOC); a fixtura é a dele ([`super::braco_em`]).

use super::super::diagonal;
use super::{braco_em, viragem_maxima};
use ph2d_vec_scene::VecPath;

/// ⭐⭐ **GATE — o gancho que a UNIÃO deixa sai** (F45). Com o braço em C e as duas juntas a somar
/// `~238°`, a união corta uma cúbica do assado DENTRO da dobra dela e o nó novo sai com a alça
/// além dele: uma meia-volta de `171°`–`179°` no contorno de fora, e o traço desenha a meia-lua.
/// A cura é o desfazer dos ganchos correr TAMBÉM depois da união.
///
/// ⚠️ O CONTROLO: a união com a bola mas SEM essa passagem deixa a meia-volta em alguma pose da
/// faixa — senão o gate não mede o mecanismo.
#[test]
fn o_gancho_que_a_uniao_deixa_sai() {
    let mut sem_a_passagem = 0;
    for a in (130..=170_u16).step_by(2) {
        for soma in [236_u16, 238, 240] {
            let (primeira, segunda) = (f32::from(a), -f32::from(soma - a));
            let (sem, com) = braco_em(primeira, segunda);
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
    let mut poses: Vec<(f32, f32)> = vec![(0.0, 174.0), (144.0, -174.0), (176.0, -142.0)];
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

/// **Sonda — a VARREDURA da dobra** (F45, `#[ignore]`): as duas juntas do braço da cena em todas
/// as poses, de `PASSO` em `PASSO` graus (`2` por omissão; a 2.ª nos DOIS sentidos, C e Z), e imprime
/// cada pose em que um nó do contorno de fora ou de uma ilha vira acima da `PAREDE_MINIMA`. Medido
/// em 2026-10-01 a passo `2`: `575` de `16 110` antes da F46, `574` deles com uma junta a `174°` ou
/// mais (o braço dobrado de volta); depois dela `1`, a `(36°, −144°)`, isolada e anterior. ⚠️ `~10 min` a passo `2`; corra-a com a máquina calma.
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
