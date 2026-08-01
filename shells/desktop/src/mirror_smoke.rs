//! **A cena do MIRROR** — a simetria VIVA, `PH2D_BUILD_SMOKE=46` (plano 25 §9, a W6.3).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `twist_smoke`/`falloff_smoke`.
//!
//! ⚠️ Ela dá o MATERIAL e **não arma modo nenhum** — a cicatriz que o `impasto_smoke` do Painter
//! prega. O que ela arma é o que um smoke de EFEITO tem de armar: as pilhas, porque é a pilha
//! que está sob teste.
//!
//! Quatro sujeitos, e cada um responde a uma pergunta diferente:
//!
//! - **Esquerda (o HERÓI, já selecionado): o meio-perfil que vira VASO.** Um contorno ABERTO com
//!   as duas pontas na borda esquerda da caixa — que é exactamente onde os defaults põem o eixo.
//!   Espelhado, ele **funde**: um único contorno FECHADO, que preenche. Arraste um nó no modo
//!   **Node** e o outro lado segue, ao vivo — é a razão de existir da wave.
//! - **Meio: a SOBREPOSIÇÃO.** Uma forma preenchida com o eixo a atravessá-la (`Offset 0`), então
//!   as duas cópias montam uma na outra. Tem de sair um bloco SÓLIDO — se o winding não fosse
//!   reposto, sob `NonZero` a sobreposição abriria um **buraco**.
//! - **Direita: `Axes = 2`**, a roseta de 4 dobras. O segundo eixo é a perpendicular pelo mesmo
//!   ponto, então isto é literalmente o mesmo espelho aplicado duas vezes.
//! - **Em baixo (o CONTROLE): `Axes = 0`.** O espelho está lá, na pilha, com os parâmetros todos
//!   — e não faz nada. É o neutro que a lei `every_kind_is_born_neutral` exige, e é o que faz o
//!   clique em **Add** não saltar a forma.

use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_mirror::MirrorSpec;
use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex};

/// Largura do traço, em unidades de MUNDO (a cena vive numa caixa de ~±3,5).
const STROKE_W: f64 = 0.05;

/// O meio-perfil do vaso: contorno ABERTO, as duas pontas no `x` mínimo da caixa dele.
const PROFILE: &[[f64; 2]] = &[
    [0.0, -1.2],
    [0.55, -0.9],
    [0.30, -0.1],
    [0.62, 0.6],
    [0.40, 1.0],
    [0.0, 1.2],
];

fn mirror(axes: f64, offset: f64, fuse: bool) -> PathEffect {
    PathEffect::Mirror(MirrorSpec {
        axes,
        offset,
        fuse: if fuse { 1.0 } else { 0.0 },
        ..MirrorSpec::new()
    })
}

fn poly(pts: &[[f64; 2]], at: [f64; 2], closed: bool, rgb: [u8; 3]) -> VecPath {
    VecPath {
        verts: pts
            .iter()
            .map(|p| VecVertex::corner([p[0] + at[0], p[1] + at[1]]))
            .collect(),
        closed,
        stroke: Some(StrokeSpec::new(
            Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
            STROKE_W,
        )),
        ..VecPath::default()
    }
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => {
            select_hero(app);
            announce(app);
        }
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;

    // ── Esquerda (HERÓI): o meio-perfil que funde no vaso ────────────────────
    let mut vase = poly(PROFILE, [-2.6, 0.6], false, [220, 180, 90]);
    vase.fill = Some(ph2d_vec_scene::Paint::Solid(Rgba8::new(220, 180, 90, 90)));
    vase.effects = vec![FxEntry::new(mirror(1.0, 100.0, true))];
    let hero = scene.push_path(vase);

    // ── Meio: a sobreposição (o eixo atravessa a forma) ──────────────────────
    let mut over = poly(
        &[[-0.5, -0.9], [0.9, -0.9], [0.9, 0.9], [-0.5, 0.9]],
        [0.2, 0.6],
        true,
        [110, 200, 130],
    );
    over.fill = Some(ph2d_vec_scene::Paint::Solid(Rgba8::new(110, 200, 130, 120)));
    over.effects = vec![FxEntry::new(mirror(1.0, 0.0, false))];
    scene.push_path(over);

    // ── Direita: as 4 dobras ─────────────────────────────────────────────────
    let mut rose = poly(
        &[[0.0, 0.0], [1.0, 0.15], [0.7, 0.8], [0.15, 0.55]],
        [2.3, 0.6],
        true,
        [140, 160, 230],
    );
    rose.fill = Some(ph2d_vec_scene::Paint::Solid(Rgba8::new(140, 160, 230, 120)));
    rose.effects = vec![FxEntry::new(mirror(2.0, 100.0, false))];
    scene.push_path(rose);

    // ── Em baixo: o CONTROLE, com o espelho NEUTRO na pilha ──────────────────
    let mut ctrl = poly(
        &[[-0.6, -0.5], [0.6, -0.5], [0.6, 0.5], [-0.6, 0.5]],
        [0.0, -1.9],
        true,
        [150, 150, 150],
    );
    ctrl.effects = vec![FxEntry::new(PathEffect::Mirror(MirrorSpec::new()))];
    scene.push_path(ctrl);
    let _ = hero;
}

/// Selecciona o vaso — a selecção mora no pen, como nas cenas irmãs.
fn select_hero(app: &mut crate::App) {
    let hero = app
        .gfx
        .as_ref()
        .and_then(|g| g.vec_scene.paths().first().map(|p| p.id));
    if let Some(id) = hero {
        app.vec_pen.select_many(&[id]);
    }
}

/// A mensagem — com os números MEDIDOS da geometria COZIDA, nunca de memória.
fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let scene = &gfx.vec_scene;
    // `(contornos, fechado?)` de cada caminho depois de a pilha correr — é o que o render vê.
    let read = |i: usize| -> (usize, bool, f64) {
        scene.paths().get(i).map_or((0, false, 0.0), |p| {
            let c = p.cooked();
            let (mut lo, mut hi) = (f64::MAX, f64::MIN);
            for v in c.verts_all() {
                lo = lo.min(v.anchor[0]);
                hi = hi.max(v.anchor[0]);
            }
            (c.contour_count(), c.closed, hi - lo)
        })
    };
    let (vc, vclosed, vw) = read(0);
    let (oc, _, _) = read(1);
    let (rc, _, _) = read(2);
    let (cc, _, cw) = read(3);
    // A largura do controle ANTES da pilha: é ela que prova que o neutro não mexeu em nada.
    let raw_ctrl = scene.paths().get(3).map_or(0.0, |p| {
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for v in p.verts_all() {
            lo = lo.min(v.anchor[0]);
            hi = hi.max(v.anchor[0]);
        }
        hi - lo
    });

    eprintln!(
        "[mirror] cena montada: {} formas, todas com um Mirror na pilha.",
        scene.paths().len()
    );
    eprintln!(
        "[mirror]   VASO (herói):  {vc} contorno(s), fechado={vclosed}, largura {vw:.2} \
         — a FUSÃO deu UMA forma fechada, não duas metades."
    );
    eprintln!("[mirror]   SOBREPOSIÇÃO: {oc} contornos — as duas cópias montam uma na outra.");
    eprintln!("[mirror]   ROSETA:       {rc} contornos — `Axes = 2`, as 4 dobras.");
    eprintln!(
        "[mirror]   CONTROLE:     {cc} contorno(s), largura {cw:.2} (crua {raw_ctrl:.2}) \
         — `Axes = 0` é o NEUTRO: a pilha salta-o e nada se move."
    );
    eprintln!("[mirror] o roteiro (a ferramenta VECTOR já está em mãos):");
    eprintln!("  1. O VASO está selecionado. Na seção Effects vê-se o card **Mirror** com");
    eprintln!("     Axes / Angle / Offset / Fuse. É UMA forma fechada, e ela preenche.");
    eprintln!("  2. AO VIVO — entre no modo **Node** e arraste um nó do meio-perfil: o outro");
    eprintln!("     lado segue no mesmo frame. É a razão de existir da wave (o Flip H/V que já");
    eprintln!("     havia vira a forma UMA vez e esquece).");
    eprintln!("  3. FUSE — desmarque o Fuse do vaso: ele parte em duas metades que apenas se");
    eprintln!("     tocam e deixam de preencher. Remarque e volta a fechar.");
    eprintln!("  4. O EIXO — arraste o **Offset** do vaso: a linha desliza, e em `100` ela fica");
    eprintln!("     tangente à caixa (é onde as pontas estão, e é por isso que ela funde).");
    eprintln!("     Arraste o **Angle**: o vaso deita-se, e continua fundido.");
    eprintln!("  5. O WINDING — olhe o do MEIO: as duas cópias sobrepõem-se e o resultado é um");
    eprintln!("     bloco SÓLIDO. Se o sentido do reflexo não fosse reposto, a sobreposição");
    eprintln!("     abriria um buraco sob NonZero.");
    eprintln!("  6. AS 4 DOBRAS — a da direita tem `Axes = 2`. Baixe para 1 e volta a duas");
    eprintln!("     cópias; baixe para 0 e o efeito desaparece sem sair da pilha.");
    eprintln!("  7. O CONTROLE — a de baixo tem um Mirror com `Axes = 0` e não faz nada. É o");
    eprintln!("     que se vê ao clicar **Add > Mirror** numa forma: a pilha ganha o card e o");
    eprintln!("     desenho não salta. Suba o Axes para 1 e ela duplica ao lado.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::effect::FxCtx;
    use ph2d_vec_scene::fx_mirror::mirror_path;

    /// Coze um caminho pela pilha dele e devolve `(contornos, fechado, largura)`.
    fn cook(p: &VecPath) -> (usize, bool, f64) {
        let c = p.cooked();
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for v in c.verts_all() {
            lo = lo.min(v.anchor[0]);
            hi = hi.max(v.anchor[0]);
        }
        (c.contour_count(), c.closed, hi - lo)
    }

    /// **A cena diz a verdade.** A mensagem anuncia quatro fatos, e este gate MEDE os quatro na
    /// mesma geometria — uma cena que afirma um número que a geometria dela não tem é a forma
    /// exata de um smoke que engana quem o corre.
    ///
    /// ⚠️ E é ele que prova o caso de uso INTEIRO com os DEFAULTS: o meio-perfil do `PROFILE` tem
    /// as pontas no `x` mínimo da caixa, que é onde `Offset = 100` põe o eixo.
    #[test]
    fn the_scene_says_what_the_geometry_does() {
        let mut vase = poly(PROFILE, [-2.6, 0.6], false, [220, 180, 90]);
        vase.effects = vec![FxEntry::new(mirror(1.0, 100.0, true))];
        let (c, closed, w) = cook(&vase);
        assert_eq!(c, 1, "o vaso tem de FUNDIR num contorno só");
        assert!(closed, "e ficar fechado, senão não preenche");
        // A largura duplica: o meio-perfil mede 0,62 e o vaso 1,24.
        let half = PROFILE.iter().fold(f64::MIN, |a, p| a.max(p[0]));
        assert!(
            (w - 2.0 * half).abs() < 1e-9,
            "o vaso tem de medir o DOBRO do meio-perfil ({w} vs {})",
            2.0 * half
        );

        let mut over = poly(
            &[[-0.5, -0.9], [0.9, -0.9], [0.9, 0.9], [-0.5, 0.9]],
            [0.2, 0.6],
            true,
            [110, 200, 130],
        );
        over.effects = vec![FxEntry::new(mirror(1.0, 0.0, false))];
        assert_eq!(cook(&over).0, 2, "a sobreposição são duas cópias");

        let mut rose = poly(
            &[[0.0, 0.0], [1.0, 0.15], [0.7, 0.8], [0.15, 0.55]],
            [2.3, 0.6],
            true,
            [140, 160, 230],
        );
        rose.effects = vec![FxEntry::new(mirror(2.0, 100.0, false))];
        assert_eq!(cook(&rose).0, 4, "`Axes = 2` são as 4 dobras");

        let mut ctrl = poly(
            &[[-0.6, -0.5], [0.6, -0.5], [0.6, 0.5], [-0.6, 0.5]],
            [0.0, -1.9],
            true,
            [150, 150, 150],
        );
        ctrl.effects = vec![FxEntry::new(PathEffect::Mirror(MirrorSpec::new()))];
        let (cc, _, cw) = cook(&ctrl);
        assert_eq!(cc, 1, "o controle tem o espelho NEUTRO: nada se multiplica");
        assert!((cw - 1.2).abs() < 1e-9, "e a largura não se mexe: {cw}");
    }

    /// **A SOBREPOSIÇÃO do meio de facto se sobrepõe** — senão o sujeito não contém o fenómeno
    /// que ele existe para mostrar, e o winding reposto seria invisível na tela.
    #[test]
    fn the_overlap_subject_actually_overlaps() {
        let p = poly(
            &[[-0.5, -0.9], [0.9, -0.9], [0.9, 0.9], [-0.5, 0.9]],
            [0.2, 0.6],
            true,
            [110, 200, 130],
        );
        let spec = MirrorSpec {
            axes: 1.0,
            offset: 0.0,
            fuse: 0.0,
            ..MirrorSpec::new()
        };
        let ctx = FxCtx::of(&p);
        let out = mirror_path(&p, &spec, &ctx);
        // O eixo está no centro em x; a forma tem pontos dos dois lados ⇒ as cópias cruzam-se.
        let xs: Vec<f64> = p.verts_all().map(|v| v.anchor[0]).collect();
        let (lo, hi) = xs
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        assert!(
            lo < ctx.center[0] && hi > ctx.center[0],
            "o eixo tem de ATRAVESSAR a forma ({lo}..{hi}, centro {})",
            ctx.center[0]
        );
        assert_eq!(out.contour_count(), 2);
    }
}
