//! ⭐⭐⭐ **O efeito DEPOIS dos ossos** (`FxStage::AfterBones`, ordem do dono de 2026-10-03) — ver
//! [`crate::skin_desenho`], secção *«ANTES ou DEPOIS dos ossos»*.
//!
//! O padrão-ouro do «depois» é o do Blender com o modificador abaixo do `Armature`: a forma SEM o
//! efeito levada pela lei da imagem ponto a ponto ([`BPalco::ouro`], densa), e o efeito corrido
//! sobre ela com o tamanho do REPOUSO e o centro levado pela pele.

use super::ouro_reguas_tests::*;
use crate::skin_desenho::Leis;
use ph2d_vec_scene::effect::{FxCtx, FxEntry, FxStage, PathEffect, run_stack_with};
use ph2d_vec_scene::{VecPath, VecVertex};

/// As amostras por segmento das réguas.
const POR_SEG: usize = 12;
/// As amostras por segmento da polilinha do padrão-ouro — densa, para a corda não pesar.
const DENSO: usize = 64;

fn depois(e: PathEffect) -> FxEntry {
    FxEntry {
        stage: FxStage::AfterBones,
        ..FxEntry::new(e)
    }
}

fn twist(angle: f64) -> PathEffect {
    PathEffect::Twist(ph2d_vec_scene::fx_twist::TwistSpec { angle })
}

/// Os efeitos que MAPEIAM pontos — os que têm um ideal ponto a ponto sobre uma polilinha densa.
fn mapas() -> Vec<(&'static str, PathEffect)> {
    let mut warp = ph2d_vec_scene::fx_warp_presets::WarpSpec::new(
        ph2d_vec_scene::fx_warp_presets::WarpStyle::Arc,
    );
    warp.bend = 40.0;
    vec![
        ("Twist25", twist(25.0)),
        ("Twist120", twist(120.0)),
        ("Warp", PathEffect::Warp(warp)),
    ]
}

/// Cada tipo da tabela com os parâmetros contínuos a `frac` da faixa — `None` quando sai neutro.
fn do_tipo(kind: usize, frac: f64) -> Option<PathEffect> {
    let mut e = PathEffect::from_kind(kind)?;
    for (i, d) in e.params().to_vec().iter().enumerate() {
        if !d.toggle {
            e.set(i, d.min + frac * (d.max - d.min));
        }
    }
    (!e.is_neutral()).then_some(e)
}

/// As leis do produto sem o contacto — o bake e a fase «depois», que é o que se mede aqui.
fn so_o_bake() -> Leis {
    Leis {
        contacto: false,
        ..Leis::do_ambiente()
    }
}

/// O desenho do produto com esta pilha na forma viva.
fn desenho(p: &BPalco, pilha: &[FxEntry]) -> VecPath {
    let mut sc = p.scene.clone();
    sc.path_mut(p.id).expect("path").effects = pilha.to_vec();
    crate::skin_live::recook_leis(&p.sim, &mut sc, so_o_bake())
        .remove(&p.id)
        .expect("a forma presa tem desenho fiel")
}

/// As amostras de TODOS os contornos (as riscas de um *Hatch*, os laços de um *Knot*).
fn amostras(x: &VecPath) -> Vec<[f64; 2]> {
    let mut v = Vec::new();
    for c in 0..x.contour_count() {
        if let Some((vs, fechado)) = x.contour(c) {
            let mut y = x.clone();
            y.verts = vs.to_vec();
            y.closed = fechado;
            y.subpaths.clear();
            v.extend(b_amostra_com(&y, POR_SEG));
        }
    }
    v
}

/// O afastamento máximo nos DOIS sentidos.
fn afastamento(a: &VecPath, b: &VecPath) -> f64 {
    let (a, b) = (amostras(a), amostras(b));
    b_perfil(&a, &b).2.max(b_perfil(&b, &a).2)
}

/// A fonte sem efeito (quinas vivas cozidas) — o que o artista desenhou.
fn autorado(p: &BPalco) -> VecPath {
    let mut f = p.fonte.clone();
    f.effects.clear();
    f.cooked().into_owned()
}

/// O `FxCtx` do «depois»: o tamanho do repouso, o centro levado pela pele (o padrão-ouro).
fn ctx_depois(p: &BPalco, pele: &ph2d_skeleton::Skin) -> FxCtx {
    let ctx = FxCtx::of(&autorado(p));
    FxCtx {
        center: b_ouro_pt(pele, &p.campo, &p.correcoes, ctx.center).0,
        ..ctx
    }
}

/// ⭐ O padrão-ouro do «depois»: a forma sem efeito dobrada ponto a ponto, e a pilha sobre ela.
fn ideal(p: &BPalco, pele: &ph2d_skeleton::Skin, pilha: &[FxEntry]) -> VecPath {
    let forma = autorado(p);
    let dobrada = p.ouro(pele, &b_amostra_com(&forma, DENSO));
    let mut poli = forma.clone();
    poli.verts = dobrada.into_iter().map(VecVertex::corner).collect();
    poli.subpaths.clear();
    poli.closed = true;
    run_stack_with(&poli, pilha, &ctx_depois(p, pele)).expect("o efeito é activo")
}

/// ⭐⭐⭐ **GATE — EM REPOUSO o botão não muda o desenho**, em todo tipo que o pode ter depois. O
/// artista escolhe a ordem com a forma parada e nada pode saltar. Medido: `≤ 0,015` (o ajuste do
/// bake, ampliado pelo efeito); o *Repeater* fica de fora pelo preço (centenas de cópias em debug).
#[test]
fn em_repouso_antes_e_depois_dos_ossos_desenham_o_mesmo() {
    for (kind, nome) in PathEffect::KINDS.iter().enumerate() {
        for frac in [0.3, 0.8] {
            let Some(e) = do_tipo(kind, frac).filter(|e| !e.reads_nodes()) else {
                continue;
            };
            if matches!(e, PathEffect::Repeat(_)) {
                continue;
            }
            let p = b_palco(false);
            let antes = desenho(&p, &[FxEntry::new(e.clone())]);
            let dep = desenho(&p, &[depois(e)]);
            let d = afastamento(&antes, &dep);
            println!("  {nome:>14} {frac}: {d:.5}");
            assert!(
                d < 0.02,
                "{nome} a {frac}: em repouso o «depois» afasta-se {d} do «antes» (diagonal ~7,07)"
            );
        }
    }
}

/// ⭐⭐⭐ **GATE — um efeito que LÊ OS NÓS fica antes dos ossos, diga o dado o que disser.**
///
/// ⛔ **O CONTROLO:** sem a regra (a fase lida do dado), o «depois» de um *Bloat* em repouso
/// afastava-se `0,45`–`0,48` do «antes» — os `15` nós do desenho em repouso contra os `8` do
/// artista. Com ela, os dois desenhos são o MESMO, ao bit.
#[test]
fn um_efeito_que_le_os_nos_fica_antes_dos_ossos() {
    let mut vistos = 0;
    for kind in 0..PathEffect::KINDS.len() {
        let Some(e) = do_tipo(kind, 0.8).filter(PathEffect::reads_nodes) else {
            continue;
        };
        vistos += 1;
        assert!(!depois(e.clone()).runs_after_bones());
        let mut p = b_palco(false);
        p.dobra_em_s(60.0);
        let antes = desenho(&p, &[FxEntry::new(e.clone())]);
        let dep = desenho(&p, &[depois(e)]);
        assert_eq!(antes, dep, "{}: o «depois» de um efeito que lê os nós correu", kind);
    }
    assert_eq!(vistos, 2, "o Zig Zag e o Pucker & Bloat — a tabela mudou");
}

/// ⭐⭐⭐ **GATE — DEPOIS DOS OSSOS o efeito é REFEITO sobre a forma dobrada, com o tamanho do
/// REPOUSO.**
///
/// # As três metades
///
/// 1. **O produto segue o ideal** tão perto quanto a forma SEM efeito o segue (o bake): o efeito
///    não acrescenta erro além de ampliar o do desenho fiel.
/// 2. ⛔ **CONTROLO: o «antes» está LONGE desse ideal** — senão o botão não mudaria nada.
/// 3. ⛔ **CONTROLO: com o tamanho da POSE** (`FxCtx` do desenho dobrado) o efeito afasta-se — é a
///    lei antiga (`PH2D_SKIN_EFEITOS=0`), a que fazia um *Twist* mudar de tamanho ao dobrar. ⚠️ Só
///    `3×`: um *Twist* de `25°` a `90°` em S lê `0,041` contra `0,010` (a caixa da pose em S quase
///    não muda de tamanho).
#[test]
fn depois_dos_ossos_o_efeito_e_refeito_sobre_a_forma_dobrada() {
    for (nome, efeito) in mapas() {
        for graus in [60.0_f32, 90.0, 120.0] {
            let mut p = b_palco(false);
            p.dobra_em_s(graus);
            let pele = p.pele();
            let pilha = vec![depois(efeito.clone())];
            let ouro = b_amostra_com(&ideal(&p, &pele, &pilha), 1);
            let sem = desenho(&p, &[]);
            let ouro_sem = p.ouro(&pele, &b_amostra_com(&autorado(&p), DENSO));
            let (_, _, bake) = b_perfil(&b_amostra_com(&sem, POR_SEG), &ouro_sem);
            let prod = desenho(&p, &pilha);
            let antes = desenho(&p, &[FxEntry::new(efeito.clone())]);
            let da_pose = run_stack_with(&sem, &pilha, &FxCtx::of(&sem)).expect("activo");
            let [(_, _, nmax), (_, _, amax), (_, _, pmax)] =
                [&prod, &antes, &da_pose].map(|c| b_perfil(&b_amostra_com(c, POR_SEG), &ouro));
            println!(
                "  {nome:>8} {graus:>4}°: bake {bake:.5} · depois {nmax:.5} · antes {amax:.5} · \
                 tamanho da pose {pmax:.5}"
            );
            assert!(
                nmax < 2.0 * bake + 0.002,
                "{nome} a {graus}°: o «depois» afasta-se {nmax} do ideal (a forma sem efeito \
                 {bake})"
            );
            assert!(
                amax > 10.0 * nmax,
                "{nome} a {graus}°: o «antes» ({amax}) não se distingue do ideal do «depois»"
            );
            assert!(
                pmax > 3.0 * nmax,
                "{nome} a {graus}°: o tamanho da POSE ({pmax}) não se distingue do do repouso"
            );
        }
    }
}

/// ⭐⭐⭐ **GATE — quando o esqueleto ANDA, o efeito «depois» vai com a forma.** O centro do efeito é
/// levado pela pele: o esqueleto inteiro deslocado (nenhuma dobra) desenha o repouso deslocado.
///
/// ⛔ **O CONTROLO:** com o centro do REPOUSO parado no mundo, um *Twist* roda a forma deslocada à
/// volta de um ponto que ficou para trás.
#[test]
fn quando_o_esqueleto_anda_o_efeito_depois_vai_com_a_forma() {
    const PASSO: [f64; 2] = [3.0, -1.5];
    let pilha = vec![depois(twist(120.0))];
    let p = b_palco(false);
    let mut repouso = desenho(&p, &pilha);
    ph2d_vec_scene::bake_xform(
        &mut repouso,
        &ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, PASSO[0], PASSO[1]]),
    );
    let mut p = b_palco(false);
    {
        use ph2d_ecs::Transform;
        #[expect(clippy::cast_possible_truncation, reason = "um passo de teste")]
        let passo = ph2d_core::Vec2::new(PASSO[0] as f32, PASSO[1] as f32);
        p.sim
            .world_mut()
            .get_mut::<Transform>(p.ossos[0])
            .expect("Transform")
            .translation += passo;
    }
    let andou = desenho(&p, &pilha);
    let pele = p.pele();
    let sem = desenho(&p, &[]);
    let parado = run_stack_with(&sem, &pilha, &FxCtx::of(&autorado(&p))).expect("activo");
    let (d, controlo) = (afastamento(&andou, &repouso), afastamento(&parado, &repouso));
    println!(
        "  andou {PASSO:?}: depois {d:.5} · com o centro parado {controlo:.5} · centro levado {:?}",
        ctx_depois(&p, &pele).center
    );
    assert!(d < 0.01, "o «depois» não foi com a forma ({d})");
    assert!(controlo > 0.5, "o CONTROLO não se afasta ({controlo}) — a fixtura não anda");
}

/// ⭐⭐ **GATE — a pilha MISTA parte-se pela ordem de cada entrada:** um efeito «antes» e outro
/// «depois» = o «antes» cozido e dobrado, e o «depois» por cima.
#[test]
fn a_pilha_mista_corre_o_antes_dobrado_e_o_depois_por_cima() {
    let zz = PathEffect::ZigZag(ph2d_vec_scene::fx_zigzag::ZigZagSpec {
        amplitude: 6.0,
        ridges: 24.0,
        ..Default::default()
    });
    let mut p = b_palco(false);
    p.dobra_em_s(90.0);
    let ctx = ctx_depois(&p, &p.pele());
    // ⚠️ O «depois» vem PRIMEIRO na pilha: a fase ganha à posição.
    let mista = desenho(&p, &[depois(twist(60.0)), FxEntry::new(zz.clone())]);
    let so_antes = desenho(&p, &[FxEntry::new(zz)]);
    let esperado = run_stack_with(&so_antes, &[depois(twist(60.0))], &ctx).expect("activo");
    let d = afastamento(&mista, &esperado);
    println!("  mista vs (antes dobrado + depois): {d:.6}");
    assert!(d < 1e-6, "a pilha mista não é as duas fases em sequência ({d})");
}
