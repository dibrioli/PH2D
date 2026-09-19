//! The rig's shared **column contract** and its **forward-kinematics resolve** —
//! the leaf both `rig.skeleton` and `rig.fk` carry (a 60-line copy beats a new
//! foundational crate for two consumers, [[project_brush_along_path_satellite_not_node]]).
//!
//! ## A skeleton is an ordinary instance stream (Motion Nodes M4.N3)
//!
//! The plan floated a `Domain::Rig` for this — which would have meant **unfreezing
//! the node contract**. It is not needed: an element IS a joint, and four ordinary
//! columns describe the chain.
//!
//! | column   | type   | meaning |
//! |----------|--------|---------|
//! | `parent` | Scalar | index of the joint this one hangs from; `< 0` = a **root** |
//! | `len`    | Scalar | length of the bone running from the parent INTO this joint |
//! | `rot`    | Scalar | the joint's **LOCAL** angle (degrees), relative to its parent |
//! | `P`      | Vec2   | the joint's **WORLD** position — *derived*, never authored |
//! | `wrot`   | Scalar | the joint's **WORLD** angle (degrees) — *derived* (skinning reads it) |
//!
//! So a skeleton flows on the SAME wires as everything else: every generic node
//! still works on it (`motion.move` shifts it, `motion.falloff` masks it, and the
//! **`Rotation` channel of `oscillator`/`wiggle`/`noise`/`step` poses it** — they
//! all just read and write columns). Rig is pure fan-out: **zero contract change**.
//!
//! ## Why `rot` is LOCAL and `P` is derived
//!
//! Because that is what makes a chain a chain: rotate one joint and everything
//! downstream of it swings — which only happens if the children's world pose is a
//! *function* of the parent's. Storing world angles per joint (KineFX's choice)
//! would make a generic modifier writing `rot` bend one joint and tear the limb
//! apart. Here a generic modifier writing `rot` poses the joint *locally*, and
//! [`resolve`] rebuilds the world pose — which is exactly what `rig.fk` is for.
//!
//! **The bones can therefore never stretch**: `|P[i] − P[parent]| == len[i]`, by
//! construction, whatever anyone did to `rot` — see the normalisation in [`resolve`],
//! which is what makes that literally true and not merely almost true.

use crate::trig;
use ph2d_nodegraph::attr::{Column, Stream};

pub(crate) const PARENT: &str = "parent";
pub(crate) const LEN: &str = "len";
pub(crate) const ROT: &str = "rot";
pub(crate) const WROT: &str = "wrot";
/// **O ângulo LOCAL** — o que o autor escreve, relativo ao pai.
///
/// ⛔⛔⛔ **Ele existe porque a coluna `rot` mudou de significado em 2026-09-19, por um report do
/// dono** (uma fila de setas em que *«o último objeto tem direção diferente»*). O `rot` que este
/// resolvedor deixava para trás era o ângulo **relativo ao pai**, e `rot` é a coluna que TODO o
/// desenho desta casa lê como *«a rotação deste elemento»*: o lowering faz `stream.get("rot")` nas
/// duas rotas e o dispositivo assa `"rot"` na posição `2` das suas oito colunas.
///
/// Medido num `rig.skeleton` de fábrica: `rot = [90, 0, 0, 0, 0, 0, 0, 0]` contra
/// `wrot = [90, 90, …]` — *a raiz era a única seta certa da imagem, e as outras sete apontavam
/// para onde ninguém tinha pedido.* Censo a decidir de quem é o nome: **72 sítios de produto**
/// escrevem `rot`, e os de fora do rig (`motion.clone`, `motion.look_at`, `motion.orbit`,
/// `motion.distribute_*`, `motion.noise`, `motion.drive`) escrevem todos o MUNDO.
///
/// ⚠️ **A idempotência é a razão de ele ser uma coluna e não um esquecimento:** com o `rot` a
/// carregar o MUNDO, uma segunda resolução somaria o mundo ao mundo e a pose andava.
pub(crate) const LROT: &str = "lrot";

/// **A pose LOCAL de uma corrente** — a porta ÚNICA da escada, e ela tem TRÊS degraus.
///
/// ⛔⛔ **A primeira tentativa de cura FALHOU por não ter esta porta** (2026-09-19): ela ensinou
/// só o `resolve` a preferir o [`LROT`], e os nós que POSAM — a IK de dois ossos, o FABRIK, a
/// mangueira — continuavam a escrever o solve em [`ROT`]. Resultado: o `resolve` lia o local da
/// resolução ANTERIOR e **deitava fora o solve que acabara de ser calculado**, com a mão a
/// aterrar em `[3.0, 0.0]` em vez de `[1.5, 1.5]`. *Quem lê uma escada e quem escreve nela têm de
/// concordar sobre o degrau, e um `const` partilhado não chega: é preciso uma porta.*
///
/// Os degraus, e cada um foi comprado por um defeito MEDIDO:
///
/// 1. **sem [`LROT`]** ⇒ o [`ROT`] é o local. Mantém a corrente **autorada à mão** — um
///    documento, um MCP, uma fixtura — a posar como sempre posou.
/// 2. ⛔⛔⛔ **com [`LROT`] mas o [`ROT`] REESCRITO** ⇒ ganha o `ROT`. *É assim que o artista
///    dobra um esqueleto no grafo*: uma caneta (`motion.drive` em `Custom…`) ligada ao canal
///    `rot`, que é o nome que ele vê. A primeira redacção desta porta não tinha este degrau e
///    **tornou esse gesto MUDO** — o `motion_rig_probe` apanhou-o, e era a rota do produto.
/// 3. **senão** ⇒ o [`LROT`], que é por onde os nós que POSAM entregam o solve.
pub(crate) fn local(input: &Stream, n: usize) -> Vec<f32> {
    let rot = scalars(input, ROT, 0.0, n);
    if input.get(LROT).is_none() {
        return rot; // autorada à mão: o `rot` é o local, como sempre foi
    }
    // ⭐⭐⭐ **O DISCRIMINADOR É EXACTO E NÃO UM EPSILON:** o [`resolve`] publica o MESMO vector
    // em [`ROT`] e em [`WROT`] (um `w.clone()` e o `w`), logo numa corrente que ele devolveu eles
    // são iguais **ao bit**. Divergirem quer dizer que alguém reescreveu o `rot` DEPOIS — e essa
    // é a pose mais recente.
    let w = scalars(input, WROT, 0.0, n);
    if rot != w {
        return rot;
    }
    scalars(input, LROT, 0.0, n)
}

/// Degrees per turn — the `trig` leaf speaks cycles, the columns speak degrees
/// (the app's one authored-angle unit).
const DEGREES_PER_TURN: f32 = 360.0;

/// A Scalar column read to length `n` (absent / short → `default`).
pub(crate) fn scalars(s: &Stream, name: &str, default: f32, n: usize) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, default);
    v
}

/// Every element's position (absent → the origin).
pub(crate) fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) if v.len() == s.count() => v.clone(),
        _ => vec![[0.0, 0.0]; s.count()],
    }
}

/// **Forward kinematics**: rebuild `P` and `wrot` from (`parent`, `len`, `rot`).
///
/// A **root** (no parent) stays exactly where it is — it keeps the `P` it arrived
/// with, so a `motion.move` upstream still places the rig, and re-resolving never
/// drags the limb back to the origin. Every other joint hangs off its parent:
///
/// ```text
/// wrot[i] = wrot[parent] + rot[i]
/// P[i]    = P[parent] + len[i] · (cos wrot[i], sin wrot[i])
/// ```
///
/// **A stream with no `parent` column is all roots → every `P` survives untouched.**
/// That is the identity rule (doc 39): dropping `rig.fk` on a plain point cloud must
/// not move a single element.
///
/// Joints are assumed **topologically ordered** (a parent before its children) —
/// which every rig source emits. A forward reference (`parent >= i`) is treated as a
/// root rather than read as garbage: it cannot deadlock or read uninitialised state.
pub(crate) fn resolve(input: &Stream) -> Stream {
    let n = input.count();
    let parent = scalars(input, PARENT, -1.0, n);
    let len = scalars(input, LEN, 0.0, n);
    let rot = local(input, n);
    let base = positions(input);

    let mut p = vec![[0.0f32; 2]; n];
    let mut w = vec![0.0f32; n];
    for i in 0..n {
        let pi = parent[i];
        // A finite, backward-pointing index is a parent; anything else is a root.
        let par = (pi >= 0.0 && pi.is_finite() && (pi as usize) < i).then_some(pi as usize);
        match par {
            None => {
                p[i] = base[i];
                w[i] = rot[i];
            }
            Some(j) => {
                w[i] = w[j] + rot[i];
                let (cos, sin) = trig::cos_sin_cycles(w[i] / DEGREES_PER_TURN);
                // NORMALISE the direction before stepping along it. The parabolic
                // `cos/sin` pair (HR-5) is ~0.1 % off the unit circle, and an
                // un-normalised step would make the bone ~0.1 % long or short — so
                // "the bones never stretch" would be a claim that is only ALMOST
                // true, and a limb's total reach would drift with its pose. One
                // `sqrt` (which HR-5 allows) makes it exactly true; the residual
                // ~0.05° of angle error is invisible, a stretching bone is not.
                let inv = 1.0 / (cos * cos + sin * sin).sqrt();
                p[i] = [p[j][0] + len[i] * cos * inv, p[j][1] + len[i] * sin * inv];
            }
        }
    }

    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != "P" && name != WROT {
            out.set(name.clone(), col.clone());
        }
    }
    out.set("P", Column::Vec2(p));
    // ⚠️ **As TRÊS colunas, e nenhuma é redundante:** o [`LROT`] guarda o que o autor escreveu (a
    // porta por onde a re-resolução entra), o [`ROT`] leva o MUNDO porque é o que o desenho lê, e
    // o [`WROT`] fica porque tem um leitor de PRODUTO — o `rig.skin_deformer` compara a pose de
    // repouso com a posada por ele, e os três `pose.rs` lêem-no para saber onde a ponta aponta.
    // *Apagá-lo por parecer um alias do `ROT` partia esses quatro nós em silêncio.*
    out.set(LROT, Column::Scalar(rot));
    out.set(ROT, Column::Scalar(w.clone()));
    out.set(WROT, Column::Scalar(w));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A chain of `n` joints, each `len` long, each turned `rot` degrees from its
    /// parent. Joint 0 is the root: anchored at `root`, and pointing along `+x` (its
    /// `rot` is a WORLD angle, exactly as `rig.skeleton` publishes it).
    pub(crate) fn chain(n: usize, len: f32, rot: f32, root: [f32; 2]) -> Stream {
        let mut p = vec![[0.0, 0.0]; n];
        p[0] = root;
        Stream::new(n)
            .with(
                PARENT,
                Column::Scalar((0..n).map(|i| i as f32 - 1.0).collect()),
            )
            .with(
                LEN,
                Column::Scalar((0..n).map(|i| if i == 0 { 0.0 } else { len }).collect()),
            )
            .with(
                ROT,
                Column::Scalar((0..n).map(|i| if i == 0 { 0.0 } else { rot }).collect()),
            )
            .with("P", Column::Vec2(p))
    }

    fn ps(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    /// A straight chain lies along `+x`, spaced by the bone length, anchored at the
    /// root's own position — the root is NOT dragged to the origin.
    #[test]
    fn a_straight_chain_hangs_off_its_root_where_the_root_already_is() {
        let out = resolve(&chain(4, 1.0, 0.0, [5.0, 2.0]));
        assert_eq!(
            ps(&out),
            vec![[5.0, 2.0], [6.0, 2.0], [7.0, 2.0], [8.0, 2.0]]
        );
    }

    /// The local angles COMPOUND down the chain (that is what makes it a chain): a
    /// quarter turn per joint walks a square. FALSIFIED by treating `rot` as a world
    /// angle, which would fan the bones out from the root instead of curling them.
    #[test]
    fn local_angles_compound_so_the_chain_curls() {
        let out = resolve(&chain(4, 1.0, 90.0, [0.0, 0.0]));
        let p = ps(&out);
        // wrot: 90, 180, 270 → right-angle turns: up, left, down.
        let near =
            |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3;
        assert!(near(p[1], [0.0, 1.0]), "{:?}", p[1]);
        assert!(near(p[2], [-1.0, 1.0]), "{:?}", p[2]);
        assert!(near(p[3], [-1.0, 0.0]), "{:?}", p[3]);
    }

    /// **Bones never stretch**, no matter the pose — the invariant the whole
    /// representation rests on, held to a tenth of a per-mille.
    ///
    /// FALSIFIED by stepping along the raw parabolic `(cos, sin)` without normalising it:
    /// the pair is ~0.1 % off the unit circle, so every bone would come out ~0.1 % long or
    /// short DEPENDING ON ITS ANGLE — a limb whose reach quietly changes as it moves.
    #[test]
    fn every_bone_keeps_its_length_whatever_the_pose() {
        for rot in [0.0, 17.0, 43.0, 90.0, -140.0, 400.0] {
            let p = ps(&resolve(&chain(6, 0.7, rot, [1.0, 1.0])));
            for i in 1..6 {
                let (dx, dy) = (p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]);
                let d = (dx * dx + dy * dy).sqrt();
                assert!((d - 0.7).abs() < 1e-4, "bone {i} at rot {rot} measured {d}");
            }
        }
    }

    /// ⭐⭐⭐ **O `rot` QUE SAI É O ÂNGULO DE MUNDO — e é essa a coluna que o desenho lê.**
    ///
    /// Report do dono, 2026-09-19 (com foto): *«por que em skeleton o último objeto tem direção
    /// diferente?»* — uma fila vertical de setas em que sete apontavam para a direita e a de
    /// baixo para cima. ⛔ **A diferente era a RAIZ, e era a única certa:** o `rot` que este
    /// resolvedor deixava para trás era o ângulo relativo ao PAI (`[90, 0, 0, …]` numa cadeia
    /// recta), e `rot` é a coluna que o `lower` lê nas duas rotas e que o dispositivo assa na
    /// posição `2` das suas oito. Só a raiz coincidia, porque uma raiz não tem pai.
    ///
    /// ⚠️ **As três metades, e cada uma mata uma cura barata:**
    /// 1. numa cadeia RECTA o `rot` é constante — é a imagem do report, invertida;
    /// 2. numa cadeia CURVA ele ACUMULA (`30 · 60 · 90`), senão «constante» seria satisfeito por
    ///    um `rot` cravado a zero;
    /// 3. e ele concorda com o [`WROT`] **em todas**, que é o que prova ser o mesmo ângulo e não
    ///    um terceiro número parecido.
    #[test]
    fn o_rot_que_sai_e_o_angulo_de_mundo() {
        let w = |s: &Stream| scalars(s, WROT, 0.0, s.count());
        let r = |s: &Stream| scalars(s, ROT, 0.0, s.count());

        // (1) Recta: a raiz aponta a 90° e todos os ossos vão atrás dela.
        let recta = resolve(&chain(4, 1.0, 0.0, [0.0, 0.0]));
        assert_eq!(
            r(&recta),
            vec![0.0; 4],
            "uma cadeia recta aponta toda para o mesmo lado"
        );

        // (2) Curva: o ângulo ACUMULA — sem isto, um `rot` cravado a zero passava em (1).
        let curva = resolve(&chain(4, 1.0, 30.0, [0.0, 0.0]));
        assert_eq!(
            r(&curva),
            vec![0.0, 30.0, 60.0, 90.0],
            "cada junta soma o seu angulo ao do pai"
        );

        // (3) E é o MESMO ângulo que o `wrot` carrega, nas duas.
        for s in [&recta, &curva] {
            assert_eq!(r(s), w(s), "o `rot` e o `wrot` sao o mesmo angulo de mundo");
        }
    }

    /// ⭐⭐⭐ **OS TRÊS DEGRAUS DA ESCADA, cada um medido — e o do meio matou a premissa deste
    /// gate no dia em que ele nasceu.**
    ///
    /// ⚠️⚠️ **A primeira redacção afirmava *«escrever no `rot` não posa nada»*, e estava ERRADA.**
    /// Ela reprovou horas depois, contra o `motion_rig_probe`: *é ligando uma caneta ao canal
    /// `rot` que o artista dobra um esqueleto no grafo* — o nome que ele vê na UI —, e a lei que
    /// eu tinha escrito tornava esse gesto **MUDO**. A premissa morreu, e ela está aqui com a
    /// morte à vista em vez de o gate ter sido apagado.
    ///
    /// ⛔⛔ **As duas falhas anteriores desta mesma porta foram silenciosas**, e é por isso que os
    /// três degraus existem: os nós que POSAM escreviam o solve em [`ROT`] e ele era deitado fora
    /// (a mão da IK aterrava em `[3.0, 0.0]`); e as fixturas do `rig.skin_deformer` rodavam uma
    /// corrente **já resolvida** escrevendo num campo que ninguém voltava a ler.
    ///
    /// ⭐ **O discriminador do degrau 2 é EXACTO:** o `resolve` publica o mesmo vector em [`ROT`]
    /// e em [`WROT`], logo eles são iguais ao bit até alguém reescrever um deles.
    ///
    /// ⚠️⚠️ **Este gate nasceu de DUAS falhas medidas no mesmo dia**, e as duas foram silenciosas:
    /// a primeira tentativa de cura ensinou só o `resolve` a preferir o [`LROT`] e deixou os três
    /// nós que POSAM a escrever o solve em [`ROT`] — a mão da IK aterrava em `[3.0, 0.0]` em vez
    /// de `[1.5, 1.5]`; e a segunda apanhou as FIXTURAS do `rig.skin_deformer`, cujo `chain()`
    /// devolve uma corrente **já resolvida** e que rodava a cadeia escrevendo num campo que
    /// ninguém voltava a ler.
    ///
    /// ⭐ **O gate é de COMPORTAMENTO e não um censo de texto:** um `grep` por `with(ROT` passa a
    /// ficar verde no dia em que alguém escrever a coluna por uma variável — *aqui mede-se o
    /// barro*.
    #[test]
    fn as_duas_portas_posam_e_a_intocada_fica() {
        // ⚠️⚠️ **A cadeia é CURVA de propósito, e a recta tornava o controlo VÁCUO** — foi uma
        // mutação sobrevivente que o mostrou: com `angle = 0` o local e o mundo são ambos
        // `[0,0,0,0]`, logo apagar a preservação do [`LROT`] fazia a segunda resolução acumular
        // ZEROS e a pose não se mexia. *Uma fixtura onde as duas leituras coincidem não pode
        // distinguir qual delas o código usou.*
        let base = resolve(&chain(4, 1.0, 30.0, [0.0, 0.0]));
        let dobra = vec![0.0, 90.0, 0.0, 0.0];

        // (2) O `rot` REESCRITO ganha — é o gesto do artista (uma caneta no canal `rot`).
        let pelo_rot = resolve(&base.clone().with(ROT, Column::Scalar(dobra.clone())));
        assert_ne!(
            ps(&pelo_rot),
            ps(&base),
            "reescrever o `rot` e' como o artista dobra um esqueleto: TEM de posar"
        );

        // (3) E o `lrot` também — é por onde os nós que posam entregam o solve.
        let pelo_lrot = resolve(&base.clone().with(LROT, Column::Scalar(dobra.clone())));
        assert_ne!(
            ps(&pelo_lrot),
            ps(&base),
            "o `lrot` e' a porta da pose: ela TEM de mover a cadeia"
        );

        // ⛔ O CONTROLO, e é ele que impede a cura barata «ler sempre o `rot`»: uma corrente
        //    resolvida e NÃO tocada tem de resolver-se no mesmo sítio. Sem isto, a escada
        //    perderia a idempotência — o `rot` que sai é o MUNDO, e relê-lo como local
        //    acumularia o mundo sobre o mundo.
        assert_eq!(
            ps(&resolve(&base)),
            ps(&base),
            "uma corrente intocada resolve-se no mesmo sitio"
        );
        // E as duas portas dão a MESMA pose, porque são a mesma dobra por caminhos diferentes.
        assert_eq!(ps(&pelo_rot), ps(&pelo_lrot));
    }

    /// Resolving twice changes nothing (the pose is a pure function of the columns),
    /// and a stream with NO `parent` column is all roots → every position survives.
    /// The second half is the identity rule: `rig.fk` on a point cloud is a no-op.
    #[test]
    fn resolve_is_idempotent_and_a_point_cloud_is_untouched() {
        let once = resolve(&chain(5, 1.0, 30.0, [0.0, 0.0]));
        assert_eq!(ps(&resolve(&once)), ps(&once));

        let cloud =
            Stream::new(3).with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]));
        assert_eq!(
            ps(&resolve(&cloud)),
            vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]],
            "a stream with no bones passes through untouched"
        );
    }

    /// A forward-referencing parent (a hand-authored / MCP-edited document) degrades
    /// to a root — no panic, no uninitialised read.
    #[test]
    fn a_forward_parent_reference_degrades_to_a_root() {
        let s = Stream::new(2)
            .with(PARENT, Column::Scalar(vec![1.0, -1.0])) // joint 0 points AHEAD
            .with(LEN, Column::Scalar(vec![9.0, 0.0]))
            .with(LROT, Column::Scalar(vec![0.0, 0.0]))
            .with("P", Column::Vec2(vec![[4.0, 4.0], [0.0, 0.0]]));
        assert_eq!(
            ps(&resolve(&s))[0],
            [4.0, 4.0],
            "treated as a root, kept put"
        );
    }
}
