//! ⭐⭐⭐ **O CHÃO RECEBE A COR DA PEÇA** — a metade que faltava à `W4` (`docs/Render3d/07` §10).
//!
//! O chão invisível recebe a **sombra** das lâmpadas e o escurecimento de contacto do céu, e nunca
//! recebeu a luz que a PEÇA devolve: um vaso vermelho pousa numa sombra cinzenta, onde a verdade é
//! um avermelhado à volta da base. *Ele desenha o que a peça TIRA e não o que ela PÕE.*
//!
//! # ⚠️ Porque só agora, e porque isto é barato
//!
//! O [`docs/Render3d/08` §10] nomeou-o e disse *«a marcha dela é outra»* — verdade quando o
//! ricochete se recolhia por pixel, marchando do ponto acertado. Com as **sondas** (§14) a resposta
//! num ponto qualquer é uma consulta trilinear à grelha que já está assada, e o ponto do chão é um
//! ponto como outro qualquer. ⇒ §0.0: *quem move o número que tornava algo inalcançável tem de
//! reconferir a nota.*
//!
//! # ⛔⛔ A cerca que esta sonda existe para MEDIR
//!
//! A [`crate::probes::consulta_sondas`] **agarra-se à borda** da grelha (o `clamp` do `u`). Numa
//! superfície fechada isso é inofensivo — todo ponto da peça vive dentro da caixa. **Num plano que
//! vai até ao horizonte é uma mentira**: a irradiância deixaria de cair com a distância e o mundo
//! inteiro ficaria tingido.
//!
//! ⇒ *esta sonda mede a VERDADE convergida num raio de pontos do chão e põe-na ao lado do que a
//! grelha devolve*, e é dessa tabela que sai a lei — nunca de uma janela escolhida.

use crate::{Ground, Orbit, PointLamp, Surfaces};
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;
use ph2d_material::OpenPbr;

/// O raio da peça. A bola pousa no chão, logo o centro está a esta altura.
const RAIO: f32 = 0.5;

/// O chão desta fixtura — o ponto mais baixo da bola.
const CHAO: Ground = Ground { height: 0.0 };

/// ⭐⭐⭐ **O QUE UM BYTE VÊ, em radiância** — a barra DERIVADA da cache da `W9`.
///
/// O campo do chão entra na imagem como luz SOMADA em pré-multiplicado, e o último passo é o sRGB.
/// Junto de zero a curva é o troço linear (`v × 12,92`), logo o degrau de **um byte** vale
/// `1/(255 × 12,92)` de radiância. ⇒ *um desvio abaixo disto não existe para o artista*, e é essa a
/// unidade em que a invariância desta cache se afirma — nunca um epsilon escolhido.
const BYTE_DE_RADIANCIA: f32 = 1.0 / (255.0 * 12.92);

/// A lâmpada, acima e de lado: ela acende o flanco `+x` da bola, que é o lado que sangra.
const LAMPADA: PointLamp = PointLamp {
    world: [1.2, 1.8, 0.8],
    radiance_at_one: [6.0, 6.0, 6.0],
};

/// Um vermelho forte e **sem especular** — o sangramento é transporte difuso, e um lóbulo
/// especular poria um segundo caminho de luz na régua (a lei que a caixa de Cornell já escreve).
fn vermelha() -> OpenPbr {
    OpenPbr {
        base_color: [0.75, 0.06, 0.06],
        specular_weight: 0.0,
        ..OpenPbr::default()
    }
}

/// A bola pousada na origem do chão.
fn bola() -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: RAIO },
            Xform::at(0.0, RAIO, 0.0),
        )],
        NodeId(0),
    )
    .expect("a bola")
}

fn camara() -> Orbit {
    Orbit {
        target: [0.0, RAIO, 0.0],
        half_extent: 1.6,
        ..Orbit::default()
    }
}

/// ⏱️⭐⭐⭐ **A SONDA: a verdade convergida no chão contra a grelha, ponto a ponto.**
///
/// Cada linha é um ponto do chão a `x` da base da bola. A coluna `VERDADE` é a média da radiância
/// devolvida pesada pelo cosseno sobre o hemisfério do chão (`n = [0,1,0]`), com `1 024` direcções
/// — a MESMA lei que uma sonda assa, avaliada no ponto. A coluna `GRELHA` é o que a consulta
/// trilinear devolve ali.
#[test]
#[ignore = "sonda de diagnóstico: imprime a queda do ricochete no chão"]
fn sonda_ate_onde_o_chao_recebe_a_cor_da_peca() {
    let doc = bola();
    let reg = Registry::new();
    let cam = camara();
    let mats = [vermelha().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let lado = 256usize;

    let grid = crate::probes::bake_probes(
        &doc,
        &reg,
        &cam,
        &surfaces,
        &[LAMPADA],
        crate::probes::PROBE_GRID,
        crate::probes::PROBE_DIRS,
        lado,
    );
    let borda = grid.origin[0] + grid.step * (grid.n - 1) as f32;
    println!(
        "  grelha: {n}³, canto {o:.4?}, passo {s:.4}, borda em x = {borda:.4}",
        n = grid.n,
        o = grid.origin,
        s = grid.step
    );
    println!(
        "  a bola tem raio {RAIO}, logo a borda da grelha fica a {:.3} raios do centro",
        borda / RAIO
    );

    // A verdade: o mesmo integral que uma sonda assa, num ponto do chão.
    let shape = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
    let scene = crate::probes::scene_solta(&shape, &doc, &reg, &cam, lado);
    let base = crate::shade_render::ViewBasis::of(&cam);
    let lift = scene.sharp.hit * crate::march::BIAS;
    let dirs: Vec<[f32; 3]> = (0..8192)
        .map(|k| crate::occlusion::cone_dir(k, 8192))
        .collect();

    // A luz que o chão BRANCO receberia sem a peça — a régua em que o ricochete tem de ser lido.
    // É o `livre` do [`crate::shade_render`], em luminância.
    let branco = crate::catcher_surface();
    let livre_em = |q: [f32; 3]| -> f32 {
        let n = base.world_to_view(crate::GROUND_UP);
        let v = base.world_to_view([0.0, 0.0, 1.0]);
        let mut soma = branco.indirect(n, v, &CeuUniforme(0.3));
        let d = [
            LAMPADA.world[0] - q[0],
            LAMPADA.world[1] - q[1],
            LAMPADA.world[2] - q[2],
        ];
        let cru = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        let inv = cru.sqrt().recip();
        let to_light = base.world_to_view([d[0] * inv, d[1] * inv, d[2] * inv]);
        let rad = LAMPADA.radiance_at_one.map(|c| c / cru);
        let dir = branco.direct(n, v, to_light, rad);
        soma = [soma[0] + dir[0], soma[1] + dir[1], soma[2] + dir[2]];
        crate::ground::LUMA[0] * soma[0]
            + crate::ground::LUMA[1] * soma[1]
            + crate::ground::LUMA[2] * soma[2]
    };

    let campo = crate::ground_bounce::bake_ground_bounce(
        &doc,
        &reg,
        &cam,
        CHAO,
        &surfaces,
        &[LAMPADA],
        crate::ground_bounce::GROUND_BOUNCE_GRID,
        crate::ground_bounce::GROUND_BOUNCE_DIRS,
        lado,
    );
    println!(
        "  campo do chão: {n}², canto {o:.3?}, passo {s:.4}",
        n = campo.n,
        o = campo.origin,
        s = campo.step
    );

    println!(
        "  x/raio ·      r ·  VERDADE(r) · % do livre ·   GRELHA(r) ·    CAMPO(r) · campo/verdade"
    );
    for passo in 0..20 {
        let x = RAIO * (1.0 + 0.25 * passo as f32);
        let q = [x, CHAO.height, 0.0];
        // `r` é medido do CENTRO da bola: é a distância que uma fonte compacta usa.
        let r = (x * x + RAIO * RAIO).sqrt();
        // ── a verdade ────────────────────────────────────────────────────────────────────────
        let origens: Vec<[f32; 3]> = vec![q; dirs.len()];
        let sai = crate::bounce::radiancia_devolvida(
            &scene,
            &base,
            lift,
            12.0,
            &origens,
            &dirs,
            &surfaces,
            &[LAMPADA],
        );
        let (mut soma, mut peso) = ([0.0f32; 3], 0.0f32);
        for (j, d) in dirs.iter().enumerate() {
            let c = d[1]; // `n · d` com `n = [0,1,0]`
            if c <= 0.0 {
                continue;
            }
            soma = [
                soma[0] + c * sai[j][0],
                soma[1] + c * sai[j][1],
                soma[2] + c * sai[j][2],
            ];
            peso += c;
        }
        let verdade = if peso > 0.0 {
            [soma[0] / peso, soma[1] / peso, soma[2] / peso]
        } else {
            [0.0; 3]
        };
        // ── a grelha ─────────────────────────────────────────────────────────────────────────
        let lida = crate::probes::consulta_sondas(&grid, q, crate::GROUND_UP, lift, false);
        // A régua: quanto isto vale ao lado da luz que o chão já recebe.
        let livre = livre_em(q);
        let pct = 100.0 * verdade[0] / livre.max(1e-9);
        let c = campo.sample(q);
        let razao = if verdade[0] > 1e-9 {
            c[0] / verdade[0]
        } else {
            f32::NAN
        };
        println!(
            "  {rr:>6.2} · {r:>6.3} · {v0:>11.6} · {pct:>9.2}% · {l0:>11.6} · {c0:>11.6} · {razao:>10.3}",
            rr = x / RAIO,
            v0 = verdade[0],
            l0 = lida[0],
            c0 = c[0],
        );
    }
}

/// ⏱️⭐⭐ **A SONDA DAS MANCHAS** — o ruído do estimador, que é o defeito que este módulo já pagou
/// duas vezes (os arcos no vaso, o «reflexo mal feito» na face do cubo).
///
/// A régua é a **segunda diferença** ao longo de uma linha do campo, normalizada pelo valor local:
/// ruído independente de célula para célula sobe-a; uma curva lisa mantém-na perto de zero.
#[test]
#[ignore = "sonda de diagnóstico: imprime o ruído do campo do chão por contagem de direcções"]
fn sonda_o_ruido_do_campo_do_chao() {
    let doc = bola();
    let reg = Registry::new();
    let cam = camara();
    let mats = [vermelha().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };

    let referencia = crate::ground_bounce::bake_ground_bounce(
        &doc,
        &reg,
        &cam,
        CHAO,
        &surfaces,
        &[LAMPADA],
        crate::ground_bounce::GROUND_BOUNCE_GRID,
        4096,
        256,
    );
    // ⚠️ A régua é ABSOLUTA e normalizada pelo PICO do campo, não relativa célula a célula: uma
    // célula onde a verdade é quase zero dá erro relativo enorme e contribui zero para a imagem.
    // *O que o olho vê é a mancha ao lado do brilho mais forte.*
    let pico = referencia
        .value
        .iter()
        .fold(0.0f32, |m, v| m.max(v[0]).max(v[1]).max(v[2]));
    // E o que ela vale em BYTES: o canal vermelho sobre o fundo preto do modelador.
    let bytes = |lin: f32| f32::from(ph2d_color::srgb::linear_to_srgb_byte(lin));
    println!(
        "  pico do campo {pico:.6} ⇒ {b:.0}/255 no vermelho, sobre o fundo transparente do modelador",
        b = bytes(pico)
    );
    println!("  direcções · pior desvio / pico · em bytes · desvio médio / pico · em bytes");
    for dirs in [16u32, 32, 64, 128, 256] {
        let campo = crate::ground_bounce::bake_ground_bounce(
            &doc,
            &reg,
            &cam,
            CHAO,
            &surfaces,
            &[LAMPADA],
            crate::ground_bounce::GROUND_BOUNCE_GRID,
            dirs,
            256,
        );
        let (mut pior, mut soma) = (0.0f32, 0.0f32);
        for k in 0..campo.value.len() {
            let e = (campo.value[k][0] - referencia.value[k][0]).abs();
            pior = pior.max(e);
            soma += e;
        }
        #[allow(clippy::cast_precision_loss)]
        let media = soma / campo.value.len() as f32;
        println!(
            "  {dirs:>9} · {pp:>17.4} · {pb:>8.1} · {mp:>19.4} · {mb:>8.1}",
            pp = pior / pico,
            pb = bytes(pior),
            mp = media / pico,
            mb = bytes(media),
        );
    }
}

/// Um céu de luminância constante — o mesmo idioma dos gates do chão.
struct CeuUniforme(f32);

impl ph2d_material::Environment for CeuUniforme {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        [self.0; 3]
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        [self.0; 3]
    }
}

// ── os portões ────────────────────────────────────────────────────────────────────────────────

/// A cena, as superfícies e o g-buffer desta fixtura — a bola vermelha pousada.
fn cena(
    lado: u32,
) -> (
    FieldDoc,
    Registry,
    Orbit,
    crate::Gbuffer,
    Vec<ph2d_material::Surface>,
) {
    let doc = bola();
    let reg = Registry::new();
    let cam = camara();
    let g = crate::trace(&doc, &reg, &cam, lado, lado);
    (doc, reg, cam, g, vec![vermelha().prepare()])
}

/// ⭐⭐⭐ **O PORTÃO DA WAVE: o chão fica AVERMELHADO à volta da peça, e a cor é a DELA.**
///
/// Ele tem **quatro** metades, e nenhuma sozinha afirma a frase:
///
/// 1. o chão à volta da base **ganha luz** (os bytes sobem contra o mesmo quadro sem campo);
/// 2. a luz é **VERMELHA** — o canal da peça domina os outros dois por uma margem larga. *Um
///    brilho cinzento passaria na metade 1 e seria outra coisa;*
/// 3. **longe da peça o quadro é INTACTO ao byte** — é a metade que impede a cura barata de
///    clarear o mundo inteiro, que é exactamente o que a grelha 3D fazia;
/// 4. e ela **MORRE**: com o campo vazio o quadro volta byte a byte ao de sempre.
#[test]
fn a_peca_tinge_o_chao_a_volta_dela_e_so_a_volta_dela() {
    let lado = 192u32;
    let (doc, reg, cam, g, mats) = cena(lado);
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let mundos = [LAMPADA.world];
    let mut sh = crate::shadow_pass_on(&doc, &reg, &cam, &g, &mundos, Some(CHAO));
    let sem = pinta(&g, &cam, &surfaces, &sh);

    sh.set_ground_bounce(crate::ground_bounce::bake_ground_bounce(
        &doc,
        &reg,
        &cam,
        CHAO,
        &surfaces,
        &[LAMPADA],
        crate::ground_bounce::GROUND_BOUNCE_GRID,
        crate::ground_bounce::GROUND_BOUNCE_DIRS,
        lado as usize,
    ));
    let com = pinta(&g, &cam, &surfaces, &sh);

    // (1) e (2): o pixel de fundo que mais ganhou.
    let (mut melhor, mut ganho) = (usize::MAX, 0i32);
    for i in 0..g.hit.len() {
        if g.hit[i] {
            continue;
        }
        let d = i32::from(com[i * 4]) - i32::from(sem[i * 4]);
        if d > ganho {
            ganho = d;
            melhor = i;
        }
    }
    assert!(
        ganho >= 8,
        "o chão tem de GANHAR luz à volta da peça: o melhor pixel subiu {ganho} bytes no vermelho"
    );
    let px = &com[melhor * 4..melhor * 4 + 3];
    let (r, verde, azul) = (i32::from(px[0]), i32::from(px[1]), i32::from(px[2]));
    assert!(
        r > 2 * verde.max(azul),
        "a luz devolvida tem de ter a COR da peça: o pixel lê ({r}, {verde}, {azul})"
    );

    // (3) **Longe da peça o quadro é INTACTO ao byte.**
    //
    // ⚠️⚠️ A 1.ª redacção desta metade media a coluna `0` do ECRÃ e reprovou sobre produto correcto
    // (`1` byte de vermelho): com `half_extent = 1,6` e um campo de `±3,0` no mundo, **o ecrã
    // inteiro cai dentro do campo**. *Uma régua que diz «longe» em píxeis não sabe onde o campo
    // acaba* — a população certa sai do MUNDO.
    let w = lado as usize;
    let rays = cam.rays();
    let screen = crate::Screen::new(g.width, g.height, cam.half_extent);
    let mut fora = 0u32;
    for i in 0..g.hit.len() {
        if g.hit[i] {
            continue;
        }
        let Some(q) = crate::ground::ground_at(&rays, screen, CHAO, i % w, i / w) else {
            continue;
        };
        if sh.ground_bounce().sample(q) != [0.0; 3] {
            continue;
        }
        fora += 1;
        assert_eq!(
            &com[i * 4..i * 4 + 4],
            &sem[i * 4..i * 4 + 4],
            "fora do campo o fundo tem de ficar INTACTO ao byte (pixel {i}, chão {q:?})"
        );
    }
    assert!(
        fora >= 64,
        "a régua precisa de ver pixels FORA do campo: só {fora} o estavam"
    );

    // ⛔⛔ **A BORDA da silhueta NÃO entra neste gate, e a razão é MEDIDA.** Numa bola sobre o
    // chão as arestas são a silhueta dela, e o ponto de chão que o raio daqueles pixels toca lê
    // campo **`0,000003`** (contra o pico `0,0151`) — e levantar a peça PIORA (`112` arestas com
    // campo a `0` de folga, `29` a `0,5`). *Uma régua que não vê o fenómeno acontecer não prova que
    // ele não aconteceu*, logo a lei da borda é gateada onde ela se observa: por UNIDADE, em
    // `a_borda_recebe_a_media_dos_vizinhos_de_fundo`.

    // (5) a morte.
    sh.set_ground_bounce(crate::ground_bounce::GroundBounce::vazio());
    assert_eq!(
        pinta(&g, &cam, &surfaces, &sh),
        sem,
        "com o campo VAZIO o quadro tem de voltar byte a byte ao de sempre"
    );
}

/// ⭐⭐ **A lei barata concorda com a CONVERGIDA onde ela age** — e a barra sai da medição, não de
/// um número escolhido.
///
/// A referência é o mesmo integral com `8 192` direcções **uniformes sobre o hemisfério**, avaliado
/// ponto a ponto: um estimador completamente diferente do que o produto usa (que manda as direcções
/// para dentro do cone). ⚠️ **A faixa medida pára antes da orla**, que é uma aproximação declarada e
/// tem gate próprio.
#[test]
fn o_campo_do_chao_concorda_com_a_convergida() {
    let lado = 128usize;
    let doc = bola();
    let reg = Registry::new();
    let cam = camara();
    let mats = [vermelha().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let campo = crate::ground_bounce::bake_ground_bounce(
        &doc,
        &reg,
        &cam,
        CHAO,
        &surfaces,
        &[LAMPADA],
        crate::ground_bounce::GROUND_BOUNCE_GRID,
        crate::ground_bounce::GROUND_BOUNCE_DIRS,
        lado,
    );

    let shape = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
    let scene = crate::probes::scene_solta(&shape, &doc, &reg, &cam, lado);
    let base = crate::shade_render::ViewBasis::of(&cam);
    let lift = scene.sharp.hit * crate::march::BIAS;
    let dirs: Vec<[f32; 3]> = (0..8192)
        .map(|k| crate::occlusion::cone_dir(k, 8192))
        .collect();

    let (mut pior, mut conta) = (0.0f32, 0u32);
    // De `1` a `4,5` raios: o miolo, que é onde a orla ainda não age.
    for passo in 0..15 {
        let x = RAIO * (1.0 + 0.25 * passo as f32);
        let q = [x, CHAO.height, 0.0];
        let origens: Vec<[f32; 3]> = vec![q; dirs.len()];
        let sai = crate::bounce::radiancia_devolvida(
            &scene,
            &base,
            lift,
            12.0,
            &origens,
            &dirs,
            &surfaces,
            &[LAMPADA],
        );
        let (mut soma, mut peso) = (0.0f32, 0.0f32);
        for (j, d) in dirs.iter().enumerate() {
            if d[1] <= 0.0 {
                continue;
            }
            soma += d[1] * sai[j][0];
            peso += d[1];
        }
        let verdade = soma / peso;
        let lido = campo.sample(q)[0];
        pior = pior.max((lido - verdade).abs() / verdade.max(1e-9));
        conta += 1;
    }
    assert_eq!(conta, 15, "a faixa medida tem de ter os 15 pontos");
    assert!(
        pior <= 0.10,
        "a lei barata tem de bater a convergida no miolo: pior desvio {pior:.4} (medido 0,061)"
    );
}

/// ⭐⭐ **A ORLA esmorece e NUNCA desenha uma aresta** — a aproximação declarada, com a régua que a
/// separa de um corte.
///
/// ⚠️ A régua é a **maior queda entre células vizinhas** na faixa da orla, em unidades do PICO do
/// campo: um corte a pique deixaria ali um degrau do tamanho do valor local, e um esmorecimento
/// linear deixa uma fracção da célula.
#[test]
fn a_orla_do_campo_esmorece_em_vez_de_cortar() {
    let lado = 128usize;
    let doc = bola();
    let reg = Registry::new();
    let cam = camara();
    let mats = [vermelha().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let campo = crate::ground_bounce::bake_ground_bounce(
        &doc,
        &reg,
        &cam,
        CHAO,
        &surfaces,
        &[LAMPADA],
        crate::ground_bounce::GROUND_BOUNCE_GRID,
        crate::ground_bounce::GROUND_BOUNCE_DIRS,
        lado,
    );
    let pico = campo
        .value
        .iter()
        .fold(0.0f32, |m, v| m.max(v[0]))
        .max(1e-9);

    // Uma linha que atravessa a orla, amostrada MAIS FINO que a célula: é ali que um corte aparece.
    let meia = campo.step * (campo.n - 1) as f32 * 0.5;
    let centro = campo.origin[0] + meia;
    let passos = 400usize;
    let mut maior_queda = 0.0f32;
    let mut anterior = campo.sample([centro, CHAO.height, campo.origin[1] + meia]);
    for k in 1..=passos {
        let x = centro + 1.2 * meia * k as f32 / passos as f32;
        let v = campo.sample([x, CHAO.height, campo.origin[1] + meia]);
        maior_queda = maior_queda.max((anterior[0] - v[0]).abs());
        anterior = v;
    }
    assert!(
        maior_queda / pico <= 0.05,
        "a orla tem de ESMORECER: a maior queda entre amostras vizinhas vale {:.4} do pico",
        maior_queda / pico
    );
    // E ela chega mesmo a zero: fora do campo não há luz nenhuma.
    assert_eq!(
        campo.sample([centro + 2.0 * meia, CHAO.height, campo.origin[1] + meia]),
        [0.0; 3],
        "fora do campo a consulta devolve ZERO"
    );
}

/// O quadro desta fixtura — o mesmo `shade_render` do produto.
fn pinta(g: &crate::Gbuffer, cam: &Orbit, surfaces: &Surfaces<'_>, sh: &crate::Shadows) -> Vec<u8> {
    crate::shade_render(
        g,
        cam,
        surfaces,
        &crate::Lighting {
            lamps: &[],
            points: &[LAMPADA],
            sky: &CeuUniforme(0.3),
            shadows: Some(sh),
        },
        &crate::Presentation::of(ph2d_view_transform::Look::default()),
        [0, 0, 0, 0],
    )
}

/// ⭐⭐⭐ **A luz devolvida atravessa o MATERIAL do chão — e o gate mede-o NO PIXEL, não na função.**
///
/// ⛔⛔ **Este gate nasceu de uma mutação que sobreviveu, e a 1.ª redacção dele estava ERRADA.** Eu
/// escrevi que a ponte `branco.indirect(n, v, SoIrradiancia(E))` era a IDENTIDADE (o chão é branco
/// puro, `base_color = [1,1,1]`), e ela reprovou sobre produto correcto: a resposta difusa do
/// material a uma irradiância `E` é **`0,570354 · E`**, constante em `E` e nos três canais. *Uma
/// premissa escrita sem a correr é um palpite com cara de medição.*
///
/// ⭐⭐⭐ **E a MEDIÇÃO fecha o assunto da mutação:** a resposta do chão branco é `~1,00` perto da
/// incidência normal e `0,570` a rasar, e **os pixels com sinal são os de incidência quase normal**
/// (a luz devolvida cai com a distância, e é longe que a vista rasa). Nesta vista a ponte vale, no
/// pixel onde mais vale, **menos de um degrau de saída** — o gate mede-o e afirma-o.
///
/// ⇒ *a ponte é a expressão CERTA e é INOBSERVÁVEL aqui, e isso é uma medição e não uma desculpa*:
/// ela morde numa vista rasante, e o dia em que o chão ganhar albedo ela passa a morder em todo
/// lado. ⛔ Apagá-la seria trocar uma lei do material por uma coincidência de ângulo.
///
/// ⚠️ E o gate entra pelo PIXEL, senão ele chama a mesma função que a mutação troca e fica cego —
/// *um gate que chama a função em vez de percorrer a rota afirma que a lei existe, nunca que o
/// produto a usa*.
#[test]
fn a_luz_devolvida_atravessa_o_material_do_chao() {
    let lado = 192u32;
    let (doc, reg, cam, g, mats) = cena(lado);
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let mundos = [LAMPADA.world];
    let mut sh = crate::shadow_pass_on(&doc, &reg, &cam, &g, &mundos, Some(CHAO));
    sh.set_ground_bounce(crate::ground_bounce::bake_ground_bounce(
        &doc,
        &reg,
        &cam,
        CHAO,
        &surfaces,
        &[LAMPADA],
        crate::ground_bounce::GROUND_BOUNCE_GRID,
        crate::ground_bounce::GROUND_BOUNCE_DIRS,
        lado as usize,
    ));
    let px = pinta(&g, &cam, &surfaces, &sh);

    // ⭐ **O pixel mais claro do fundo** — o fundo é preto transparente, logo a luz devolvida é a
    // ÚNICA coisa que ali está, e o byte é ela.
    let w = lado as usize;
    let rays = cam.rays();
    let screen = crate::Screen::new(g.width, g.height, cam.half_extent);
    let (mut melhor, mut byte) = (usize::MAX, 0u8);
    for i in 0..g.hit.len() {
        if g.hit[i]
            || g.edges
                .binary_search_by_key(&(i as u32), |e| e.pixel)
                .is_ok()
        {
            continue;
        }
        if px[i * 4] > byte {
            byte = px[i * 4];
            melhor = i;
        }
    }
    assert!(
        byte >= 8,
        "a régua precisa de um pixel com sinal: o melhor lê {byte}"
    );
    let (mx, my) = (melhor % w, melhor / w);
    let q = crate::ground::ground_at(&rays, screen, CHAO, mx, my)
        .expect("o pixel mais claro vê o chão");
    let campo = sh.ground_bounce().sample(q)[0];

    // ⭐⭐ **A resposta do chão é a do material NA DIRECÇÃO DE VISTA DESTE PIXEL**, e não um número.
    //
    // ⛔⛔ A 1.ª redacção deste gate cravou `0,570354` como *«a resposta do chão branco»* e reprovou
    // sobre produto correcto (`lido 0,015209`, esperado `0,008513`): aquele número é a resposta numa
    // direcção que eu escolhi à mão (`world_to_view([0,0,1])`, que é o `+z` do MUNDO levado à vista
    // e não a direcção do olho), e a resposta difusa desta casa **depende de `(n, v)`**. *Uma
    // constante medida numa direcção arbitrária lê-se como propriedade do material.*
    let base = crate::shade_render::ViewBasis::of(&cam);
    let branco = crate::catcher_surface();
    let n = base.world_to_view(crate::GROUND_UP);
    let v = crate::shade_render::view_direction(&cam, &screen, mx, my);
    let esperado = branco.indirect(n, v, &crate::shade_render::SoIrradiancia([campo; 3]))[0];
    let lido = ph2d_color::srgb::srgb_to_linear_byte(byte);
    // A folga é a do BYTE: um degrau de saída à volta do valor lido.
    let degrau = ph2d_color::srgb::srgb_to_linear_byte(byte + 1) - lido;
    assert!(
        (lido - esperado).abs() <= degrau,
        "o pixel do chão vale `material(campo)`: lido {lido:.6}, esperado {esperado:.6}, degrau {degrau:.6}"
    );
    // ⭐⭐⭐ **E QUANTO a ponte vale, no pixel onde ela mais vale** — a medição que decide se a
    // mutação que a apaga tem onde sangrar.
    let mut maior = 0.0f32;
    for i in 0..g.hit.len() {
        if g.hit[i] {
            continue;
        }
        let Some(qi) = crate::ground::ground_at(&rays, screen, CHAO, i % w, i / w) else {
            continue;
        };
        let e = sh.ground_bounce().sample(qi)[0];
        if e <= 0.0 {
            continue;
        }
        let vi = crate::shade_render::view_direction(&cam, &screen, i % w, i / w);
        let com_ponte = branco.indirect(n, vi, &crate::shade_render::SoIrradiancia([e; 3]))[0];
        maior = maior.max((com_ponte - e).abs());
    }
    assert!(
        maior <= degrau,
        "a ponte vale no MÁXIMO {maior:.6} nesta vista, contra um degrau de saída de {degrau:.6}"
    );
}

/// ⭐⭐ **A BORDA recebe a média dos vizinhos de FUNDO** — a lei por unidade, onde ela se observa.
///
/// ⛔ Ela vive aqui e não no gate do produto porque a fixtura dele **não contém o fenómeno**: numa
/// bola sobre o chão o ponto de chão das arestas lê campo `0,000003` (ver a nota lá).
#[test]
fn a_borda_recebe_a_media_dos_vizinhos_de_fundo() {
    // Uma cruz `3×3`: o centro é peça, os quatro vizinhos são fundo com valores conhecidos.
    let (w, h) = (3usize, 3usize);
    let mut g = crate::Gbuffer {
        width: w as u32,
        height: h as u32,
        hit: vec![false; w * h],
        normal: vec![[0.0, 0.0, 1.0]; w * h],
        point: vec![[0.0; 3]; w * h],
        curvature: Vec::new(),
        curvature_style: Vec::new(),
        edges: Vec::new(),
    };
    g.hit[4] = true;
    let mut postas = vec![[0.0f32; 3]; w * h];
    postas[3] = [0.1, 0.0, 0.0]; // esquerda
    postas[5] = [0.3, 0.0, 0.0]; // direita
    postas[1] = [0.2, 0.0, 0.0]; // cima
    postas[7] = [0.4, 0.0, 0.0]; // baixo

    let media = crate::ground_shade::edge_ground_bounce(&g, &postas, 4);
    assert!(
        (media[0] - 0.25).abs() <= 1e-6,
        "a borda leva a média dos quatro vizinhos de fundo: {media:?}"
    );

    // ⚠️ **Um pixel de FUNDO leva o dele**, e não a média — é a metade que separa as duas leis.
    assert_eq!(
        crate::ground_shade::edge_ground_bounce(&g, &postas, 3),
        [0.1, 0.0, 0.0]
    );

    // ⚠️⚠️ **Sem vizinho de fundo devolve ZERO e não `1,0`**: ausência de LUZ, nunca luz inventada
    // — a lei oposta à do [`crate::ground_shade::edge_ground_factor`], e as duas estão certas.
    g.hit.iter_mut().for_each(|x| *x = true);
    assert_eq!(
        crate::ground_shade::edge_ground_bounce(&g, &postas, 4),
        [0.0; 3]
    );
}

/// ⏱️ **O RELÓGIO do campo do chão** — ele decide se a lei é assada na CPU e enviada, ou se precisa
/// de kernel próprio no dispositivo.
///
/// ⚠️ Corre em `--release`: em debug esta crate lê `~20×` mais lento e daria um veredito invertido.
#[test]
#[ignore = "sonda de relógio: corra com --release e a máquina calma"]
fn sonda_o_relogio_do_campo_do_chao() {
    let doc = bola();
    let reg = Registry::new();
    let cam = camara();
    let mats = [vermelha().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    println!(
        "  /proc/loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for (n, dirs) in [(48usize, 128u32), (48, 64), (32, 128), (64, 128)] {
        let mut melhor = f64::MAX;
        for _ in 0..5 {
            let t = std::time::Instant::now();
            let campo = crate::ground_bounce::bake_ground_bounce(
                &doc,
                &reg,
                &cam,
                CHAO,
                &surfaces,
                &[LAMPADA],
                n,
                dirs,
                1080,
            );
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1000.0);
            std::hint::black_box(&campo);
        }
        println!(
            "  {n}² × {dirs} = {:>9} raios · {melhor:>8.3} ms (mínimo de 5)",
            n * n * dirs as usize
        );
    }
}

/// ⏱️⭐⭐ **A GRELHA: quanto ela pode encolher antes de a imagem dar por isso.**
///
/// A régua é a mesma do [`crate::ground_bounce::GROUND_BOUNCE_DIRS`] — o desvio contra a VERDADE
/// convergida ao longo do perfil, em **bytes** sobre o fundo preto do modelador —, porque é a saída
/// de 8 bits que decide. ⚠️ O relógio da assadura vai ao lado: ela é `O(n²)`.
#[test]
#[ignore = "sonda de relógio e exactidão: corra com --release"]
fn sonda_a_grelha_do_campo_do_chao() {
    let doc = bola();
    let reg = Registry::new();
    let cam = camara();
    let mats = [vermelha().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let lado = 256usize;
    let shape = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
    let scene = crate::probes::scene_solta(&shape, &doc, &reg, &cam, lado);
    let base = crate::shade_render::ViewBasis::of(&cam);
    let lift = scene.sharp.hit * crate::march::BIAS;
    let dirs: Vec<[f32; 3]> = (0..8192)
        .map(|k| crate::occlusion::cone_dir(k, 8192))
        .collect();
    // A verdade, uma vez, ao longo do perfil que interessa (o miolo, antes da orla).
    let pontos: Vec<[f32; 3]> = (0..15)
        .map(|k| [RAIO * (1.0 + 0.25 * k as f32), CHAO.height, 0.0])
        .collect();
    let verdade: Vec<f32> = pontos
        .iter()
        .map(|q| {
            let origens: Vec<[f32; 3]> = vec![*q; dirs.len()];
            let sai = crate::bounce::radiancia_devolvida(
                &scene,
                &base,
                lift,
                12.0,
                &origens,
                &dirs,
                &surfaces,
                &[LAMPADA],
            );
            let (mut soma, mut peso) = (0.0f32, 0.0f32);
            for (j, d) in dirs.iter().enumerate() {
                if d[1] > 0.0 {
                    soma += d[1] * sai[j][0];
                    peso += d[1];
                }
            }
            soma / peso
        })
        .collect();
    let bytes = |lin: f32| f32::from(ph2d_color::srgb::linear_to_srgb_byte(lin.max(0.0)));
    println!(
        "  /proc/loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("  grelha · pior desvio · em bytes · relógio (mínimo de 5)");
    for n in [16usize, 24, 32, 48, 64] {
        let mut relogio = f64::MAX;
        let mut campo = crate::ground_bounce::GroundBounce::vazio();
        for _ in 0..5 {
            let t = std::time::Instant::now();
            campo = crate::ground_bounce::bake_ground_bounce(
                &doc,
                &reg,
                &cam,
                CHAO,
                &surfaces,
                &[LAMPADA],
                n,
                crate::ground_bounce::GROUND_BOUNCE_DIRS,
                lado,
            );
            relogio = relogio.min(t.elapsed().as_secs_f64() * 1000.0);
        }
        let mut pior = 0.0f32;
        for (q, v) in pontos.iter().zip(&verdade) {
            pior = pior.max((campo.sample(*q)[0] - v).abs());
        }
        println!(
            "  {n:>6}² · {pior:>11.6} · {:>8.1} · {relogio:>8.2} ms",
            bytes(pior)
        );
    }
}

/// ⏱️⭐⭐⭐⭐ **A PREMISSA DA CURA DA `W9`, MEDIDA: o campo do chão depende da CÂMERA?**
///
/// A [`W9`](../../../../docs/Render3d/03_o_plano.md) propõe cachear este campo *«por cena-e-luz»*,
/// e a frase que a justifica é **«o campo não depende da câmera»** — escrita na
/// [`09` §6](../../../../docs/Render3d/09_a_cor_que_a_peca_devolve_ao_chao.md), que lhe põe o preço
/// (`+4,98 ms` por quadro assente) e diz que a dependência é *«só a tolerância de acerto»*.
///
/// ⛔⛔ **Lido o código, a frase não pode estar inteiramente certa:** o
/// [`crate::bounce::radiancia_devolvida`] recebe a `ViewBasis`, que é a ORIENTAÇÃO da câmera, e a
/// resposta de um BSDF ao longo de uma direcção de vista tem um lóbulo especular. ⇒ *antes de
/// construir a cache, meça-se de quanto ela mentiria.*
///
/// ⚠️ **Esta sonda é de VALOR e não de relógio**, e é por isso que ela corre com a máquina ocupada:
/// o que ela lê são bytes do campo, e contenção não os move. *A coluna do relógio da `W9` é outra
/// medição, e essa precisa da máquina.*
///
/// # O que ela imprime
///
/// O campo assado em oito azimutes, contra o do azimute `0`:
///
/// * **`|Δ| máx`** — o pior desvio absoluto de uma célula (o campo é radiância, sem tecto);
/// * **`|Δ|/máx`** — o mesmo em fracção do maior valor do campo, que é a régua que diz se um pixel
///   o veria;
/// * o **CONTROLO**, que é o que dá direito às outras linhas: trocar a **LUZ** tem de mover o campo
///   muito mais do que orbitar. *Uma sonda de invariância sem um eixo que MOVE mede a si própria.*
#[test]
#[ignore = "sonda de diagnóstico: mede a premissa da cache da W9"]
fn sonda_o_campo_do_chao_depende_da_camera() {
    let doc = bola();
    let reg = Registry::new();
    let mats = [vermelha().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let lado = 256usize;
    let assa = |cam: &Orbit, lampada: crate::PointLamp| {
        crate::ground_bounce::bake_ground_bounce(
            &doc,
            &reg,
            cam,
            CHAO,
            &surfaces,
            &[lampada],
            crate::ground_bounce::GROUND_BOUNCE_GRID,
            crate::ground_bounce::GROUND_BOUNCE_DIRS,
            lado,
        )
    };
    // ⚠️ O enquadramento é o da fixtura (o mesmo alvo e a mesma distância), e só o AZIMUTE muda —
    // senão isto mediria o zoom, que é a outra metade da dependência e a que a `09` já nomeia.
    let camara_em = |azimute: f32| Orbit {
        rotation: Orbit::from_yaw_pitch(azimute, 0.5).rotation,
        ..camara()
    };
    let desvio = |a: &crate::ground_bounce::GroundBounce,
                  b: &crate::ground_bounce::GroundBounce| {
        let mut pior = 0.0f32;
        let mut maior = 0.0f32;
        for (x, y) in a.value.iter().zip(b.value.iter()) {
            for k in 0..3 {
                pior = pior.max((x[k] - y[k]).abs());
                maior = maior.max(x[k].abs()).max(y[k].abs());
            }
        }
        (pior, maior)
    };

    let base = assa(&camara_em(0.0), LAMPADA);
    println!("\n  == o campo do chão contra o azimute 0 ==");
    println!("  azimute ·      |Δ| máx ·  |Δ|/máx ·  veredito");
    let mut pior_orbita = 0.0f32;
    for k in 1..8 {
        #[allow(clippy::cast_precision_loss)]
        let a = core::f32::consts::TAU * k as f32 / 8.0;
        let (pior, maior) = desvio(&base, &assa(&camara_em(a), LAMPADA));
        let frac = if maior > 0.0 { pior / maior } else { 0.0 };
        pior_orbita = pior_orbita.max(frac);
        println!(
            "  {:>7.0}° · {pior:>12.6} · {:>7.3} % · {}",
            a.to_degrees(),
            100.0 * frac,
            if frac < 0.01 {
                "invisível"
            } else {
                "⛔ VISÍVEL"
            }
        );
    }

    // ⚠️ **A OUTRA METADE DA CHAVE: o ZOOM.** A `09` §6 diz que a dependência da câmera é *«só a
    // tolerância de acerto»*, e ela sai do `half_extent` (`Sharpness::for_frame`) — logo orbitar e
    // aproximar são perguntas DIFERENTES, e uma cache que só exclua a orientação estaria a
    // adivinhar a segunda.
    println!("\n  half_extent ·      |Δ| máx ·  |Δ|/máx");
    let mut pior_zoom = 0.0f32;
    for he in [0.8f32, 1.6, 3.2] {
        let cam = Orbit {
            half_extent: he,
            ..camara_em(0.0)
        };
        let (pior, maior) = desvio(&base, &assa(&cam, LAMPADA));
        let frac = if maior > 0.0 { pior / maior } else { 0.0 };
        pior_zoom = pior_zoom.max(frac);
        println!("  {he:>11.2} · {pior:>12.6} · {:>7.3} %", 100.0 * frac);
    }

    // ⭐ **O CONTROLO** — trocar a LUZ tem de mover o campo, senão a invariância acima é vácuo.
    let outra = crate::PointLamp {
        world: [-LAMPADA.world[0], LAMPADA.world[1], -LAMPADA.world[2]],
        ..LAMPADA
    };
    let (pior, maior) = desvio(&base, &assa(&camara_em(0.0), outra));
    let frac_luz = if maior > 0.0 { pior / maior } else { 0.0 };
    println!(
        "\n  CONTROLO (a luz do outro lado) · {pior:>12.6} · {:>7.3} %",
        100.0 * frac_luz
    );
    println!(
        "\n  ⇒ orbitar move {:.3} %, aproximar move {:.3} %, e trocar a luz move {:.3} %\n",
        100.0 * pior_orbita,
        100.0 * pior_zoom,
        100.0 * frac_luz,
    );
}

/// ⭐⭐⭐⭐ **A METADE DA CHAVE QUE SAI: ORBITAR não muda o que um byte vê.**
///
/// A [`W9`](../../../../docs/Render3d/03_o_plano.md) propõe cachear o campo do chão *«por
/// cena-e-luz»* — `+4,98 ms` por quadro assente —, e a cura inteira assenta numa frase: **«o campo
/// não depende da câmera»**. Uma cache montada sobre uma frase que ninguém afirma entrega o campo
/// da câmera ANTERIOR, e o defeito é mudo: a imagem sai plausível e errada.
///
/// ⛔⛔ **Até 2026-09-21 isto era uma IMPRESSORA** ([`sonda_o_campo_do_chao_depende_da_camera`]):
/// ela mede o mesmo, **não afirma nada**, e o veredito vivia no nome dela e na minha leitura de uma
/// tabela. *Ninguém lê uma tabela que passa.*
///
/// ⛔⛔⛔ **E a leitura estava ERRADA:** o [`03` §W9](../../../../docs/Render3d/03_o_plano.md)
/// escreve *«byte-idêntico sob 8 azimutes»*, e com dígitos a sério (2026-09-21) orbitar move
/// `6e-9` com **`262`–`506` de `1024`** células a mudar no último bit. *A impressora imprimia `|Δ|`
/// com seis casas, e `6e-9` lê-se `0.000000`.*
///
/// # A barra NOMEIA O RECURSO e é o BYTE de saída
///
/// A orientação CHEGA à assadura — a [`crate::bounce::radiancia_devolvida`] recebe a `ViewBasis` —
/// e o que ela move é **arredondamento**: `6e-9` sobre um campo cujo máximo é `0,0164` são `≈ 3`
/// ULP. A lei é invariante à rotação (os BSDF só leem produtos internos); o que sobra é rodar `n` e
/// `v` em `f32`. ⇒ a barra é o [`BYTE_DE_RADIANCIA`], e a medição está **`50 000×`** abaixo dela.
///
/// ⚠️ **O ZOOM não está aqui, e é de propósito:** ele entra por outra porta (a tolerância de
/// acerto) e a medição diz que ele **É** chave — ver o gate irmão
/// [`a_tolerancia_de_acerto_entra_na_chave_da_cache_do_chao`].
///
/// ⭐ **O CONTROLO é o que dá direito à metade de cima:** trocar a LUZ tem de mover o campo. Sem
/// ele, uma assadura que devolvesse o campo VAZIO passava tudo.
///
/// # Prova de mutação (2026-09-21, **4 de 4 sangram**, com controlo no arnês e na árvore limpa)
///
/// | mutação | veredito |
/// |---|---|
/// | a assadura passa a depender FORTE da câmera (fator `5 %` na orientação) | sangra |
/// | a assadura ignora a LÂMPADA (o controlo vira vácuo) | sangra |
/// | a tolerância deixa de chegar à assadura (`for_frame` cravada) | sangra |
/// | o clamp do `hit` deixa de morder (`HIT_EPS.min` apagado) | sangra |
///
/// ⚠️⚠️ **E a primeira redacção da 1.ª mutação SOBREVIVEU, por culpa da FIXTURA e não do gate:**
/// ela punha o **alcance do raio** a depender da orientação (`±50 %`), e numa bola pequena isso é
/// um **no-op** — todo raio acerta ou falha exactamente na mesma, porque a peça está bem dentro do
/// alcance nas duas configurações. *Uma mutação que a fixtura não consegue observar lê-se
/// exactamente como uma que sobreviveu.* ⇒ a que fica mexe no **valor** que sai do integral, que é
/// a grandeza que o gate mede.
///
/// ⚠️ **O arnês tem controlo sobre o PRÓPRIO FILTRO** — ele conta `passed + failed` e aborta em
/// `0`. A primeira corrida casou **zero** testes (faltava o prefixo `tests::` no `--exact`) e
/// imprimiu `ok` nas quatro, que se lê como *«sobreviveram todas»*.
#[test]
fn o_campo_do_chao_nao_muda_o_que_um_byte_ve_ao_orbitar() {
    let doc = bola();
    let reg = Registry::new();
    let mats = [vermelha().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let assa = |cam: &Orbit, lampada: crate::PointLamp| {
        crate::ground_bounce::bake_ground_bounce(
            &doc,
            &reg,
            cam,
            CHAO,
            &surfaces,
            &[lampada],
            crate::ground_bounce::GROUND_BOUNCE_GRID,
            crate::ground_bounce::GROUND_BOUNCE_DIRS,
            256,
        )
    };
    let camara_em = |azimute: f32| Orbit {
        rotation: Orbit::from_yaw_pitch(azimute, 0.5).rotation,
        ..camara()
    };

    let base = assa(&camara_em(0.0), LAMPADA);
    let maior = base
        .value
        .iter()
        .flatten()
        .fold(0.0f32, |m, c| m.max(c.abs()));
    assert!(
        maior > 0.0,
        "o campo da fixtura é todo zero — as duas metades abaixo não medem nada"
    );
    let pior_contra_base = |o: &crate::ground_bounce::GroundBounce| {
        base.value
            .iter()
            .zip(&o.value)
            .flat_map(|(x, y)| (0..3).map(move |k| (x[k] - y[k]).abs()))
            .fold(0.0f32, f32::max)
    };

    for k in 1..4 {
        #[allow(clippy::cast_precision_loss)]
        let a = core::f32::consts::TAU * k as f32 / 4.0;
        let pior = pior_contra_base(&assa(&camara_em(a), LAMPADA));
        assert!(
            pior <= BYTE_DE_RADIANCIA,
            "orbitar {}° moveu o campo {pior:e}, acima do que um byte vê \
             ({BYTE_DE_RADIANCIA:e}) — a cache da W9 não pode excluir a ORIENTAÇÃO",
            a.to_degrees()
        );
    }

    // ⭐ **O CONTROLO**: a LUZ move, e move quase tudo.
    let outra = crate::PointLamp {
        world: [-LAMPADA.world[0], LAMPADA.world[1], -LAMPADA.world[2]],
        ..LAMPADA
    };
    let frac = pior_contra_base(&assa(&camara_em(0.0), outra)) / maior;
    assert!(
        frac > 0.5,
        "CONTROLO: trocar a luz moveu só {:.3} % do campo — se a LUZ não o move, \
         a metade acima não afirma invariância nenhuma",
        100.0 * frac
    );
}

/// ⭐⭐⭐⭐ **A METADE DA CHAVE QUE FICA: a TOLERÂNCIA DE ACERTO move o campo, logo é chave.**
///
/// ⛔⛔⛔ **Este gate existe porque um CONTROLO derrubou a premissa que eu ia usar.** O
/// [`03` §W9](../../../../docs/Render3d/03_o_plano.md) escreve que o campo é *«byte-idêntico sob
/// `4×` de zoom»* e conclui que *«a chave não leva a câmera de todo»*. Medido: naquela varredura o
/// `hit` esteve **preso em `2e-4`** nas três leituras — a única porta por onde o zoom entra na
/// assadura é a [`crate::Sharpness::for_frame`], que faz `hit = min(HIT_EPS, half_extent/(2·lado_px))`.
/// *A régua media o CLAMP, não o eixo.*
///
/// ⚠️ **E o clamp solta-se DENTRO do produto:** ele pede `lado_px > 2500 × half_extent`, que a
/// `0,2` de enquadramento são **`500` píxeis** — *um zoom apertado numa janela normal já está do
/// outro lado*.
///
/// # A medição (2026-09-21, `lado_px = 1080` fixo, o campo contra o do `hit` de fábrica)
///
/// | `half_extent` | `hit` | `|Δ|` | em bytes | células iguais |
/// |---:|---:|---:|---:|---:|
/// | `1,6` · `0,8` | `2,0e-4` (preso) | `0` | `0,000` | `1024` de `1024` |
/// | `0,4` | `1,85e-4` | `6,5e-6` | `0,021` | `137` |
/// | `0,2` | `9,26e-5` | `5,1e-5` | `0,169` | `130` |
/// | `0,05` | `2,31e-5` | `1,6e-4` | `0,527` | `130` |
/// | `0,005` | `2,31e-6` | `2,8e-4` | **`0,910`** | `130` |
///
/// ⭐ **Ele CONVERGE** — os desvios saturam quando o `hit` desce —, que é a assinatura de a
/// tolerância apertada estar a aproximar-se da verdade e não de ruído. ⇒ *o campo de fábrica está a
/// quase um byte da resposta convergida*, o que é um facto sobre a assadura de hoje e não sobre a
/// cache.
///
/// ⇒ **a chave leva o `hit` e não leva a orientação**, e é `46 000×` que separa os dois eixos
/// (`2,8e-4` contra `6e-9`). *Uma cache que excluísse a câmera inteira — como o plano prescrevia —
/// entregaria, num zoom apertado, um campo quase um byte errado.*
#[test]
fn a_tolerancia_de_acerto_entra_na_chave_da_cache_do_chao() {
    let doc = bola();
    let reg = Registry::new();
    let mats = [vermelha().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let assa = |he: f32| {
        let cam = Orbit {
            half_extent: he,
            ..camara()
        };
        crate::ground_bounce::bake_ground_bounce(
            &doc,
            &reg,
            &cam,
            CHAO,
            &surfaces,
            &[LAMPADA],
            crate::ground_bounce::GROUND_BOUNCE_GRID,
            crate::ground_bounce::GROUND_BOUNCE_DIRS,
            1080,
        )
    };
    let pior = |a: &crate::ground_bounce::GroundBounce, b: &crate::ground_bounce::GroundBounce| {
        a.value
            .iter()
            .zip(&b.value)
            .flat_map(|(x, y)| (0..3).map(move |k| (x[k] - y[k]).abs()))
            .fold(0.0f32, f32::max)
    };
    let de_fabrica = assa(1.6);

    // ── 1. O CONTROLO vem PRIMEIRO: com o clamp a morder, o zoom não move um bit ──────────────
    //
    // ⚠️ Sem ele, a metade de baixo lê-se como *«o zoom move o campo»* e alguém poria o
    // `half_extent` na chave — invalidando a cache em todo arrasto de zoom, incluindo os que o
    // clamp torna inofensivos.
    assert_eq!(
        assa(0.8).value,
        de_fabrica.value,
        "com a tolerância presa no tecto, o zoom não pode mover um bit — \
         se move, a porta do zoom não é a que este gate julga"
    );

    // ── 2. E abaixo do clamp ele MOVE, e move o que um byte vê ────────────────────────────────
    let apertado = assa(0.005);
    let d = pior(&de_fabrica, &apertado);
    assert!(
        d > 0.5 * BYTE_DE_RADIANCIA,
        "com o zoom apertado (hit {:e}) o campo moveu só {d:e} — se a tolerância não o move, \
         ela não precisava de entrar na chave da cache",
        crate::Sharpness::for_frame(0.005, 1080).hit
    );

    // ── 3. E a separação entre os DOIS eixos é o que decide a chave ───────────────────────────
    //
    // ⭐ A orientação move `6e-9` (o gate irmão) e a tolerância move `2,8e-4`: quatro ordens de
    // grandeza. *É essa distância que faz um eixo ser chave e o outro não.*
    assert!(
        d < BYTE_DE_RADIANCIA,
        "a tolerância move {d:e}, um byte inteiro — a `09` §6 dá o campo por convergido a `32²`, \
         e isso deixaria de ser verdade"
    );
}
