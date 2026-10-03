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

/// O afastamento MÁXIMO do padrão-ouro à curva `vista`, separado em `(dentro, fora)` do domínio.
///
/// ⚠️ **Fora do domínio o padrão-ouro é uma CONVENÇÃO**: a imagem presa não tem arte fora da
/// malha, e a régua dá a um ponto de fora o peso do vértice mais próximo (constante aos degraus),
/// enquanto a lei da curva mistura as linhas dos nós. Uma crista de *Zig Zag* sai da forma, e é aí
/// que as duas convenções divergem (`~0,08` junto a uma junta a `60°`). ⇒ o gate julga DENTRO e
/// só exige que fora a lei nova não seja pior do que a antiga.
fn afastamento(
    p: &BPalco,
    pele: &ph2d_skeleton::Skin,
    repouso: &VecPath,
    vista: &[[f64; 2]],
) -> (f64, f64) {
    let (mut dentro, mut fora) = (0.0_f64, 0.0_f64);
    for x in b_amostra_com(repouso, POR_SEG) {
        let (y, no_campo) = b_ouro_pt(pele, &p.campo, &p.correcoes, x);
        let d = b_dist(y, vista);
        if no_campo {
            dentro = dentro.max(d);
        } else {
            fora = fora.max(d);
        }
    }
    (dentro, fora)
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
            let (nova, antiga) = as_duas_leis(&p);
            let [(nd, nf), (ad, af)] = [&nova, &antiga]
                .map(|c| afastamento(&p, &pele, &repouso, &b_amostra_com(c, POR_SEG)));
            println!(
                "  {nome:>7} {graus:>4}°: antiga máx {ad:.5} dentro · {af:.5} fora · nova ({} nós) \
                 máx {nd:.5} dentro · {nf:.5} fora",
                nova.verts_all().count()
            );
            assert!(
                nd < 0.01,
                "{nome} a {graus}°: dentro do domínio a lei nova afasta-se {nd} do padrão-ouro \
                 (diagonal ~7,07)"
            );
            if graus >= 60.0 {
                assert!(
                    ad > 0.05,
                    "{nome} a {graus}°: a lei antiga leu máx {ad} — a fixtura deixou de conter o \
                     defeito"
                );
                assert!(
                    nd * 10.0 < ad && nf < af,
                    "{nome} a {graus}°: a lei nova ({nd} · {nf}) não é melhor que a antiga \
                     ({ad} · {af})"
                );
            }
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
    assert_eq!(estilo_de(&fx), Estilo::Efeitos(vec![FxEntry::new(efeito)]));
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
