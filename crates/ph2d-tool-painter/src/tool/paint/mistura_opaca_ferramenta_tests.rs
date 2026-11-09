//! A mistura opaca do Digital pintada pela FERRAMENTA (pen-down → moves → pen-up), sobre vermelho | azul.

use super::diag_auditoria_da_pilha::{SIZE, cp, tela_com};
use super::*;
use ph2d_painter_brush::mistura_opaca::{MisturaOpaca, ModoMisturaOpaca};

const Y: f32 = 512.0;

/// Vermelho à esquerda de `x = 512`, azul à direita; pincel verde, raio 20.
fn vermelho_azul(m: MisturaOpaca) -> PainterTool {
    let mut t = tela_com(20.0, 255);
    t.paint.composite_enabled = false;
    let mut px = vec![255u8; (SIZE * SIZE * 4) as usize];
    for (i, p) in px.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let x = i as u32 % SIZE;
        *p = if x < 512 {
            [255, 0, 0, 255]
        } else {
            [0, 0, 255, 255]
        };
    }
    t.set_source(px, SIZE, SIZE);
    t.paint.brush.color = [0.0, 1.0, 0.0];
    t.paint.brush.mistura_opaca = m;
    t
}

fn traco(t: &mut PainterTool, x0: f32, x1: f32) {
    t.on_canvas_pointer(cp([x0, Y], PointerPhase::Down));
    let mut x = x0;
    while x < x1 {
        x += 4.0;
        t.on_canvas_pointer(cp([x, Y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x1, Y], PointerPhase::Up));
}

fn px(t: &PainterTool, x: u32) -> [u8; 4] {
    let i = ((Y as u32 * SIZE + x) * 4) as usize;
    [
        t.canvas_rgba[i],
        t.canvas_rgba[i + 1],
        t.canvas_rgba[i + 2],
        t.canvas_rgba[i + 3],
    ]
}

fn modo(modo: ModoMisturaOpaca, amount: f32) -> MisturaOpaca {
    MisturaOpaca {
        modo,
        paint_amount: amount,
        color_stretch: 0.95,
        ..MisturaOpaca::default()
    }
}

/// Amount 0 (só arrasta, o Gouache blender do CSP): o pincel leva o vermelho apanhado para dentro do azul, e o
/// verde dele não entra.
#[test]
fn a_mistura_opaca_amount_zero_leva_o_vermelho_para_o_azul() {
    let mut t = vermelho_azul(MisturaOpaca {
        color_stretch: 0.5,
        ..modo(ModoMisturaOpaca::WetBlend, 0.0)
    });
    traco(&mut t, 440.0, 640.0);
    let dentro = px(&t, 520);
    let longe = px(&t, 600);
    assert!(
        dentro[0] > longe[0] + 20,
        "o vermelho não foi levado para o azul: {dentro:?} · {longe:?}"
    );
    assert!(
        dentro[1] < 30 && longe[1] < 30,
        "Amount 0 sobre tinta cheia não deposita verde: {dentro:?} · {longe:?}"
    );
    // Controlo: longe do traço o azul ficou.
    let i = ((300 * SIZE + 540) * 4) as usize;
    assert_eq!(&t.canvas_rgba[i..i + 4], &[0, 0, 255, 255]);
}

/// Amount 0,3: sobre o vermelho a cor depositada mistura o verde do pincel com o vermelho amostrado (não o cobre);
/// Amount 1 é o verde inteiro.
#[test]
fn a_mistura_opaca_pigmento_mistura_a_cor_do_traco() {
    let mut t = vermelho_azul(modo(ModoMisturaOpaca::WetBlend, 0.3));
    t.on_canvas_pointer(cp([470.0, Y], PointerPhase::Down));
    t.on_canvas_pointer(cp([470.0, Y], PointerPhase::Up));
    let p = px(&t, 470);
    assert!(
        (40..=235).contains(&p[0]) && p != [255, 0, 0, 255],
        "Amount 0,3 tem de misturar o verde no vermelho: {p:?}"
    );
    let mut cheio = vermelho_azul(MisturaOpaca {
        pigmento: 0.0,
        ..modo(ModoMisturaOpaca::WetBlend, 1.0)
    });
    cheio.on_canvas_pointer(cp([470.0, Y], PointerPhase::Down));
    cheio.on_canvas_pointer(cp([470.0, Y], PointerPhase::Up));
    let q = px(&cheio, 470);
    assert!(
        q[1] >= 250 && q[0] <= 5,
        "Amount 1 cobre com o verde: {q:?}"
    );
    // Controlo: o Digital de sempre pinta verde puro no mesmo sítio.
    let mut off = vermelho_azul(MisturaOpaca::default());
    traco(&mut off, 380.0, 480.0);
    assert_eq!(px(&off, 470), [0, 255, 0, 255]);
}

/// Desligada, a lei não existe: os outros números mexidos dão o mesmo traço, byte a byte, que o
/// pincel de fábrica.
#[test]
fn a_mistura_opaca_desligada_e_o_traco_de_sempre_ao_bit() {
    let mut a = vermelho_azul(MisturaOpaca::default());
    let mut b = vermelho_azul(MisturaOpaca {
        modo: ModoMisturaOpaca::Off,
        ..modo(ModoMisturaOpaca::Off, 0.1)
    });
    traco(&mut a, 440.0, 640.0);
    traco(&mut b, 440.0, 640.0);
    assert!(a.canvas_rgba.as_slice() == b.canvas_rgba.as_slice());
    // E o desfazer leva o traço da mistura como leva os outros.
    let mut c = vermelho_azul(modo(ModoMisturaOpaca::WetBlend, 0.5));
    let antes = c.canvas_rgba.as_ref().clone();
    c.begin_undo_step();
    traco(&mut c, 440.0, 640.0);
    assert!(c.canvas_rgba.as_slice() != antes.as_slice());
    assert!(c.undo_last(), "desfez");
    assert!(
        c.canvas_rgba.as_slice() == antes.as_slice(),
        "o desfazer não devolveu a tela"
    );
}

/// A foto: Off / Blend 0,3 / Running 0 em três faixas sobre vermelho | azul (`PH2D_MISTURA_FOTO=<rgba>`).
#[test]
#[ignore = "foto: corre à mão"]
fn foto_mistura_opaca_ferramenta() {
    let destino = std::env::var("PH2D_MISTURA_FOTO").expect("PH2D_MISTURA_FOTO");
    let mut t = vermelho_azul(MisturaOpaca::default());
    let faixas = [
        (MisturaOpaca::default(), 412.0),
        (modo(ModoMisturaOpaca::WetBlend, 0.3), 512.0),
        (modo(ModoMisturaOpaca::WetBlend, 0.0), 612.0),
    ];
    for (m, y) in faixas {
        t.paint.brush.mistura_opaca = m;
        t.on_canvas_pointer(cp([400.0, y], PointerPhase::Down));
        let mut x = 400.0;
        while x < 640.0 {
            x += 4.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([640.0, y], PointerPhase::Up));
    }
    // Recorte 320×300 à volta dos traços.
    let mut out = Vec::new();
    for y in 362..662u32 {
        let i = ((y * SIZE + 352) * 4) as usize;
        out.extend_from_slice(&t.canvas_rgba[i..i + 320 * 4]);
    }
    std::fs::write(destino, out).expect("escreve");
}

// ── O arrasto do dono (07/11): pincel de fábrica, azul, sobre um bloco vermelho e depois branco. ──

/// Branco opaco com um bloco vermelho em `x ∈ [BLOCO_X0, BLOCO_X1)`; pincel de FÁBRICA, azul.
const BLOCO_X0: u32 = 400;
const BLOCO_X1: u32 = 520;

fn bloco_vermelho(m: MisturaOpaca, spacing: Option<f32>) -> PainterTool {
    let mut t = PainterTool::default();
    let mut px = vec![255u8; (SIZE * SIZE * 4) as usize];
    for (i, p) in px.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let (x, y) = (i as u32 % SIZE, i as u32 / SIZE);
        if (BLOCO_X0..BLOCO_X1).contains(&x) && (300..700).contains(&y) {
            *p = [255, 0, 0, 255];
        }
    }
    t.set_source(px, SIZE, SIZE);
    t.paint.brush.color = [0.0, 0.0, 1.0];
    if let Some(s) = spacing {
        t.paint.brush.spacing = s;
    }
    t.paint.brush.mistura_opaca = m;
    t
}

fn arrasto(t: &mut PainterTool, x0: f32, x1: f32, y: f32) {
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    let passo = 4.0f32.copysign(x1 - x0);
    let mut x = x0;
    while (x1 - x) * passo > 0.0 {
        x += passo;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
}

fn de_fabrica(modo: ModoMisturaOpaca) -> MisturaOpaca {
    MisturaOpaca {
        modo,
        ..MisturaOpaca::default()
    }
}

/// O defeito do smoke: com o espaçamento de fábrica cada dab amostrava a tinta FRESCA do anterior, e o
/// Blend/Running pintavam quase azul puro dentro do vermelho (`[34,0,221]` / `[60,0,195]`).
#[test]
fn a_mistura_de_fabrica_arrastada_mostra_o_vermelho_dentro_do_bloco() {
    let off = {
        let mut t = bloco_vermelho(MisturaOpaca::default(), None);
        arrasto(&mut t, 360.0, 640.0, Y);
        px(&t, 500)
    };
    assert_eq!(&off[..3], &[0, 0, 255], "controlo: o Off pinta azul puro");
    {
        let modo = ModoMisturaOpaca::WetBlend;
        let mut t = bloco_vermelho(de_fabrica(modo), None);
        arrasto(&mut t, 360.0, 640.0, Y);
        let p = px(&t, 500);
        // Wet blend (CSP, amostra da tela viva, Amount 0,5): a tinta é sobretudo a do pincel, com o vermelho
        // apanhado (medido 08/11: `54,1,202`; fração do fundo ≈ 0,2), nem o azul puro do Off nem o vermelho.
        assert!(
            (30..=200).contains(&p[0]),
            "{modo:?}: o vermelho de baixo não entrou na mistura: {p:?}"
        );
        assert!(
            p[2] >= 120 && p[2] < 250 && p[1] <= 40,
            "{modo:?}: não é a tinta do pincel suja de vermelho: {p:?}"
        );
    }
}

/// O pincel leva a tinta apanhada no vermelho para além do bloco, por vários píxeis, e esvazia-se no branco.
#[test]
fn a_mistura_de_fabrica_leva_o_vermelho_para_la_do_bloco() {
    let mut t = bloco_vermelho(de_fabrica(ModoMisturaOpaca::WetBlend), None);
    arrasto(&mut t, 360.0, 700.0, Y);
    // «Rubor» = R − G: o azul do pincel e o branco do papel têm 0 (R = G); o vermelho levado tem > 0.
    let rubor = |x: u32| {
        let p = px(&t, x);
        i32::from(p[0]) - i32::from(p[1])
    };
    // Wet blend (amostra da tela viva): dentro do bloco R − G 52; o rubor cai a 17 em +20 px e a 7 em +40 px
    // (medido 08/11) — a memória do reservatório leva-o para lá da borda.
    let alem = (BLOCO_X1..BLOCO_X1 + 200)
        .take_while(|&x| rubor(x) >= 15)
        .count();
    assert!(
        alem >= 12,
        "o vermelho só foi levado {alem} px além do bloco"
    );
    assert!(
        rubor(BLOCO_X1 + 170) < rubor(BLOCO_X1 + 4),
        "o pincel não se esvaziou no branco"
    );
}

/// A mesma mistura com o espaçamento 0,05 e 0,2: as taxas são por distância, não por dab.
#[test]
fn a_mistura_opaca_nao_depende_do_espacamento() {
    {
        let modo = ModoMisturaOpaca::WetBlend;
        let corre = |s: f32| {
            let mut t = bloco_vermelho(de_fabrica(modo), Some(s));
            arrasto(&mut t, 360.0, 700.0, Y);
            [470u32, 510, 530, 560, 620].map(|x| px(&t, x))
        };
        let (fino, largo) = (corre(0.05), corre(0.2));
        for (a, b) in fino.iter().zip(&largo) {
            let d = (0..3).map(|c| a[c].abs_diff(b[c])).max().unwrap();
            assert!(d <= 24, "{modo:?}: 0,05 {fino:?} × 0,2 {largo:?}");
        }
    }
}

/// A foto do arrasto do dono: bloco vermelho sobre branco, três traços azuis de fábrica (Off e Wet blend da esquerda
/// para a direita, Wet blend da direita para a esquerda) que entram pelo branco, cruzam o vermelho e acabam no branco
/// (`PH2D_MISTURA_FOTO=<rgba>`, recorte 400×300 a partir de `(340, 362)`); imprime os píxeis medidos.
#[test]
#[ignore = "foto: corre à mão"]
fn foto_mistura_opaca_arrasto_de_fabrica() {
    let destino = std::env::var("PH2D_MISTURA_FOTO").expect("PH2D_MISTURA_FOTO");
    let mut t = bloco_vermelho(MisturaOpaca::default(), None);
    eprintln!(
        "raio {} spacing {}",
        t.paint.brush.radius_px, t.paint.brush.spacing
    );
    for (modo, y, de, ate) in [
        (ModoMisturaOpaca::Off, 412.0f32, 360.0, 720.0),
        (ModoMisturaOpaca::WetBlend, 512.0, 360.0, 720.0),
        (ModoMisturaOpaca::WetBlend, 612.0, 720.0, 360.0),
    ] {
        t.paint.brush.mistura_opaca = de_fabrica(modo);
        arrasto(&mut t, de, ate, y);
        let p = |x: u32| {
            let i = ((y as u32 * SIZE + x) * 4) as usize;
            [t.canvas_rgba[i], t.canvas_rgba[i + 1], t.canvas_rgba[i + 2]]
        };
        eprintln!(
            "{modo:?}: x430 {:?} x500 {:?} x530 {:?} x560 {:?} x620 {:?} x700 {:?}",
            p(430),
            p(500),
            p(530),
            p(560),
            p(620),
            p(700)
        );
    }
    let mut out = Vec::new();
    for y in 362..662u32 {
        let i = ((y * SIZE + 340) * 4) as usize;
        out.extend_from_slice(&t.canvas_rgba[i..i + 400 * 4]);
    }
    std::fs::write(destino, out).expect("escreve");
}

/// ⭐ **A tela de antes é a do pen-down DESTE traço, não a do grupo de undo** (smoke do dono 07/11): com dois traços
/// no mesmo passo (as pinceladas sobrepostas da W16) o arrasto azul sobre o vermelho do traço anterior leva o
/// vermelho — amostrando o `before` do undo via o branco de antes do vermelho e saía azul pálido.
#[test]
fn a_mistura_opaca_amostra_o_pen_down_do_traco_e_nao_o_grupo_do_undo() {
    let mut t = tela_com(20.0, 255);
    t.paint.composite_enabled = false;
    t.paint.brush.color = [1.0, 0.0, 0.0];
    for dy in [-16.0, 0.0, 16.0] {
        t.on_canvas_pointer(cp([300.0, Y + dy], PointerPhase::Down));
        // O 1.º traço abre o passo; os seguintes entram nele (como a macro com traços sobrepostos).
        t.poe_tracos_fora(2);
        let mut x = 300.0;
        while x < 700.0 {
            x += 4.0;
            t.on_canvas_pointer(cp([x, Y + dy], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([700.0, Y + dy], PointerPhase::Up));
    }
    assert!(
        t.paint.stroke_undo.is_some(),
        "o grupo de undo ficou aberto"
    );
    assert_eq!(px(&t, 560), [255, 0, 0, 255]);
    t.paint.brush.color = [0.0, 0.0, 1.0];
    t.paint.brush.mistura_opaca = MisturaOpaca {
        modo: ModoMisturaOpaca::WetBlend,
        ..MisturaOpaca::default()
    };
    // Controlo: o MESMO arrasto sobre a MESMA tela, posta sem grupo de undo aberto. A mistura amostra a tela VIVA
    // deste traço: com o grupo aberto dá o mesmo píxel; amostrar o `before` do grupo (o branco) clarearia (G sobe).
    let antes_do_azul = t.canvas_rgba.as_ref().clone();
    let mistura = t.paint.brush.mistura_opaca;
    traco(&mut t, 200.0, 640.0);
    let p = px(&t, 560);
    let mut c = tela_com(20.0, 255);
    c.paint.composite_enabled = false;
    c.set_source(antes_do_azul, SIZE, SIZE);
    c.paint.brush.color = [0.0, 0.0, 1.0];
    c.paint.brush.mistura_opaca = mistura;
    traco(&mut c, 200.0, 640.0);
    let q = px(&c, 560);
    let d = (0..3).map(|k| p[k].abs_diff(q[k])).max().unwrap_or(0);
    assert!(d <= 2, "com o grupo de undo {p:?} × sem ele {q:?}");
    // Dureza 1: cada carimbo cobre e o arrasto é sobretudo azul, com o vermelho apanhado (medido 08/11: `10,1,246`).
    assert!(
        p[0] >= 5 && p[2] > 120 && p[1] < 40,
        "dentro do vermelho a mistura leva vermelho E azul: {p:?}"
    );
    // A memória: só os ladrilhos tocados, nunca a tela inteira.
    let bytes = t.paint.mistura_opaca.antes.bytes();
    let tela = (SIZE * SIZE * 4) as usize;
    assert!(
        bytes > 0 && bytes * 8 < tela,
        "ladrilhos {bytes} B de {tela} B"
    );
}
