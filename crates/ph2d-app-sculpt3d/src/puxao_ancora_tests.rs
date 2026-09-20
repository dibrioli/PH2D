//! ⭐⭐⭐⭐ **A ÂNCORA DO «PUXAR PELA NORMAL»** — 2.º report do dono
//! (2026-09-19): *«precisa fixar a puxada na normal no momento do clique do
//! mouse, pois no decorrer da puxada a normal muda e não se mantém firme na
//! primeira direcção escolhida no clique»*.
//!
//! # ⚠️ Porque a sonda mora AQUI e não na `ph2d-sculpt3d`
//!
//! A irmã de lá ([`stroke_puxao_normal_tests`](../../ph2d-sculpt3d/src/stroke_puxao_normal_tests.rs))
//! lê o eixo **congelado a `0,00°`** e está certa: sem passe de topologia não
//! nasce barro NOVO debaixo do cursor que anda, logo o vértice mais deslocado
//! é sempre o mesmo e a inclinação não aparece. Esta corre o laço do PRODUTO
//! (refina e depois carimba), que é onde o report vive.
//!
//! ⛔ **Ela é IRMÃ do [`super::gancho_report`] e não parte dele:** a `=14` e a
//! `=50` medem a MÁSCARA, esta mede a ÂNCORA, e juntá-las levou aquele
//! ficheiro a `756` linhas contra o tecto de `700`.

use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

use super::gancho_report::{OLHO, norma};

/// **O ângulo entre dois vectores, em graus** — `NaN` se algum degenerar.
fn angulo(u: [f32; 3], v: [f32; 3]) -> f32 {
    let (a, b) = (norma(u), norma(v));
    if a < 1e-9 || b < 1e-9 {
        return f32::NAN;
    }
    (((u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / (a * b)).clamp(-1.0, 1.0))
        .acos()
        .to_degrees()
}

/// **Um arrasto pelo caminho do produto, e o ESPIGÃO que ele deixa** — devolve
/// `(eixo, arrasto)`: para onde o barro mais afastado foi, e quanto o dedo
/// andou.
///
/// ⚠️ **O pen-down é OBLÍQUO por omissão nos chamadores, e tem de ser:** no
/// polo a normal, o plano e o olho são todos `+z`, logo a fixtura não
/// distingue a lei congelada da lei viva — foi assim que a 1.ª medição desta
/// wave leu `0,00°` sobre o defeito que o dono estava a ver.
///
/// ⚠️ **O centro do gancho anda no plano de profundidade do pen-down** (`z`
/// constante), que é o que o [`crate::pull::Sculpt3dScene::hook_step`] faz; o
/// do agarrar é a âncora, como no `grab_at`.
fn eixo_do_puxao(
    verbo: Verb,
    graus: f32,
    dabs: usize,
    pela_normal: bool,
    verboso: bool,
) -> ([f32; 3], f32) {
    let rad = graus.to_radians();
    let p0 = [rad.sin(), 0.0, rad.cos()];
    let mut malha = ph2d_mesh::shapes::sculpt_sphere(1.0);
    malha.triangulate();
    let brush = Brush {
        verb: verbo,
        radius: 0.30,
        strength: 1.0,
        puxa_pela_normal: pela_normal,
        surface_only: true,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();
    let passo = 0.05f32;
    let mut eixo = [0.0f32; 3];
    for k in 0..dabs {
        let centro = if verbo == Verb::SnakeHook {
            [p0[0] + passo * (k + 1) as f32, 0.0, p0[2]]
        } else {
            p0
        };
        let (cut, done, _) = crate::dyntopo::passe_nos_motores(
            &mut malha,
            brush.verb,
            0.6,
            centro,
            brush.radius,
            crate::dyntopo::Rascunho {
                remap: &mut remap,
                births: &mut births,
                region: &mut region,
            },
            None,
        );
        if cut {
            stroke.shrink_with(&remap);
        }
        if done {
            stroke.grow_with(&malha, &births);
        }
        let dab = if verbo == Verb::SnakeHook {
            Dab::hooking(centro, brush.radius, OLHO, [passo, 0.0, 0.0])
        } else {
            Dab::pulling(
                centro,
                brush.radius,
                OLHO,
                [passo * (k + 1) as f32, 0.0, 0.0],
            )
        };
        stroke.dab(&mut malha, &brush, &dab, Symmetry::default());
        // O vértice que mais se afastou do repouso, e para onde.
        //
        // ⭐ **A régua é `O(1)` por vértice porque o repouso é a esfera
        // UNITÁRIA** — o ponto de repouso mais perto de `p` é `p/‖p‖`. A irmã
        // genérica ([`perto`]) é `O(n)`, e aqui custaria `O(n²)` por dab sobre
        // `~100 k` vértices: a 1.ª redacção desta sonda ficou dez minutos sem
        // imprimir uma linha.
        let mut melhor = 0.0f32;
        eixo = [0.0; 3];
        for p in malha.positions() {
            let r = norma(*p);
            if r < 1e-9 {
                continue;
            }
            if (r - 1.0).abs() > melhor {
                melhor = (r - 1.0).abs();
                let f = 1.0 - 1.0 / r;
                eixo = [p[0] * f, p[1] * f, p[2] * f];
            }
        }
        if verboso {
            println!(
                "     dab {k:>2}: verts {:>6} · eixo ({:+.4},{:+.4},{:+.4}) |{:.4}| · \
                 vs a normal do clique {:>6.2}°",
                malha.positions().len(),
                eixo[0],
                eixo[1],
                eixo[2],
                norma(eixo),
                angulo(eixo, p0),
            );
        }
    }
    (eixo, passo * dabs as f32)
}

/// ⛔⛔ **A SONDA DO REPORT DE 2026-09-19 SOBRE O `Pull Along Normal`** —
/// *«no decorrer da puxada a normal muda e não se mantém firme na primeira
/// direcção escolhida no clique»*.
///
/// ⚠️ A sonda irmã da `ph2d-sculpt3d` lê o eixo **congelado a `0,00°`** sem
/// passe de topologia, e é isso que a torna cega ao report: sem refino não
/// nasce barro novo debaixo do cursor que anda. Esta corre o laço do PRODUTO
/// (refina e depois carimba), que é a diferença que faltava.
#[test]
#[ignore = "sonda: imprime o eixo ao longo do traco, nao afirma nada"]
fn diag_o_eixo_do_puxao_pela_normal() {
    println!("\n== O EIXO DO PUXAO PELA NORMAL, COM O PASSE A CORRER ==\n");
    for verbo in [Verb::SnakeHook, Verb::Move] {
        for graus in [0.0f32, 40.0] {
            for pela_normal in [false, true] {
                println!("  -- {verbo:?} · pen-down a {graus:.0}° · pela normal {pela_normal}");
                eixo_do_puxao(verbo, graus, 16, pela_normal, true);
            }
        }
    }
}

/// ⭐⭐⭐⭐ **O GATE DO 2.º REPORT: o espigão sai a DIREITO da normal do clique, e
/// o comprimento dele é o do arrasto.**
///
/// ⛔⛔ **O CONTROLO vive dentro do gate e é a opção DESLIGADA**, porque sem ele
/// isto não afirma nada: um pincel que não movesse barro nenhum passaria na
/// primeira metade. Desligada, o barro segue a MÃO e o eixo inclina-se — que é
/// o comportamento certo lá, e é a prova de que a fixtura contém o fenómeno.
///
/// ⚠️ **A segunda metade é o que separa esta cura da que foi medida e
/// recusada:** prender o centro no clique endireita o eixo **e** faz o gancho
/// SATURAR a cerca de um raio (a lei dele mede a queda das posições vivas).
/// Sem a barra do comprimento, aquela versão passava aqui — ver
/// [`ph2d_sculpt3d`]`::stroke_normal_do_gesto::AncoraDoPuxao`.
#[test]
fn o_espigao_sai_a_direito_da_normal_do_clique() {
    const DABS: usize = 16;
    const GRAUS: f32 = 40.0;
    let p0 = [GRAUS.to_radians().sin(), 0.0, GRAUS.to_radians().cos()];
    for verbo in [Verb::SnakeHook, Verb::Move] {
        let (ligada, arrasto) = eixo_do_puxao(verbo, GRAUS, DABS, true, false);
        let (desligada, _) = eixo_do_puxao(verbo, GRAUS, DABS, false, false);
        let (a_on, a_off) = (angulo(ligada, p0), angulo(desligada, p0));
        assert!(
            a_on <= 3.0,
            "{verbo:?}: com a opcao ligada o espigao saiu a {a_on:.2}° da normal do \
             clique — a puxada nao esta' presa no pen-down"
        );
        assert!(
            a_off >= 10.0,
            "{verbo:?}: CONTROLO — com a opcao desligada o eixo devia seguir a MAO e \
             leu {a_off:.2}°; a fixtura deixou de conter o fenomeno e o gate acima \
             ja' nao afirma nada"
        );
        let comprimento = norma(ligada);
        assert!(
            comprimento >= 0.9 * arrasto,
            "{verbo:?}: o espigao mede {comprimento:.4} para um arrasto de \
             {arrasto:.4} — ele SATUROU, que e' o que acontece quando o centro do \
             dab fica preso no clique em vez de viajar com o barro"
        );
    }
}
