//! **O nome no menu é o pincel na mão** (Bug #83, smoke do dono 10/11: o menu dizia Pastel e o preview mostrava o
//! esbatedor). O pincel é UM só para todas as gavetas (ferramenta × meio); o «aplicado» é por gaveta.

use super::tests::{abre, sem_erros};
use super::*;
use ph2d_tool_painter::PaintMedia;

const PASTEL: &str = "Digital/Pastel/Pastel/Pastel.ph2d-brush.png";
const SMEAR: &str = "Digital/Basic/Smear/Smear Basic.ph2d-brush.png";
const BLUR: &str = "Digital/Basic/Blur/Blur Basic.ph2d-brush.png";
const DEDO: &str = "Digital/Blend/Finger/Finger Tip.ph2d-brush.png";
const BLENDER: &str = "WetPainter/Blend/Soft/Blender Soft.ph2d-brush.png";
const AGUA: &str = "Watercolor/Watercolor/Basic/Watercolor Basic.ph2d-brush.png";
const ROUND: &str = "WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png";
const BORRACHA: &str = "Digital/Eraser/Hard/Hard Eraser.ph2d-brush.png";

/// O preview de `rel` aplicado num pincel limpo, na ferramenta dele.
fn limpo(p: &Presets, rel: &str) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    t.preset_aplicar(&p.bib.procurar(rel).unwrap().preset)
        .unwrap();
    preview(&t)
}

/// O último quadro do preview vivo do pincel de agora.
fn preview(t: &PainterTool) -> Vec<u8> {
    let mut v = Vec::new();
    ph2d_tool_painter::pintar_preview(&t.wet_preview_cena(), &|| false, &mut |px, _, _| {
        v = px.to_vec();
    });
    v
}

fn escolhe(p: &mut Presets, t: &mut PainterTool, toasts: &mut ToastQueue, rel: &str) {
    p.atender(t, PedidoPreset::Escolher(rel.into()), toasts);
    p.quadro(t, toasts);
}

fn trilho(p: &mut Presets, t: &mut PainterTool, toasts: &mut ToastQueue, fio: &str) {
    t.set_paint_tool_mode(fio);
    p.quadro(t, toasts);
}

fn meio(p: &mut Presets, t: &mut PainterTool, toasts: &mut ToastQueue, m: PaintMedia) {
    t.set_paint_media(m);
    p.quadro(t, toasts);
}

/// ⭐ Depois de qualquer volta pelo trilho ou pelo Paint Mode, o que o menu nomeia é o que o pincel pinta.
#[test]
fn o_menu_nomeia_o_pincel_que_esta_na_mao() {
    type Passo = fn(&mut Presets, &mut PainterTool, &mut ToastQueue);
    let voltas: [(&str, Passo); 9] = [
        ("smear e volta", |p, t, q| {
            trilho(p, t, q, "smear");
            escolhe(p, t, q, SMEAR);
            trilho(p, t, q, "brush");
        }),
        ("blur e volta", |p, t, q| {
            trilho(p, t, q, "blur");
            escolhe(p, t, q, BLUR);
            trilho(p, t, q, "brush");
        }),
        ("Finger Tip e Pastel", |p, t, q| {
            escolhe(p, t, q, DEDO);
            escolhe(p, t, q, PASTEL);
        }),
        ("Wet Paint e volta", |p, t, q| {
            meio(p, t, q, PaintMedia::WetPaint);
            escolhe(p, t, q, BLENDER);
            meio(p, t, q, PaintMedia::Digital);
        }),
        ("Watercolor e volta", |p, t, q| {
            meio(p, t, q, PaintMedia::Watercolor);
            escolhe(p, t, q, AGUA);
            meio(p, t, q, PaintMedia::Digital);
        }),
        ("Wet Round e volta", |p, t, q| {
            meio(p, t, q, PaintMedia::WetPaint);
            escolhe(p, t, q, ROUND);
            meio(p, t, q, PaintMedia::Digital);
        }),
        ("borracha e volta", |p, t, q| {
            trilho(p, t, q, "eraser");
            escolhe(p, t, q, BORRACHA);
            trilho(p, t, q, "brush");
        }),
        ("ligados: smear e volta", |p, t, q| {
            t.toggle_link_shared_settings();
            trilho(p, t, q, "smear");
            escolhe(p, t, q, SMEAR);
            trilho(p, t, q, "brush");
        }),
        ("Finger Tip, Watercolor, volta", |p, t, q| {
            escolhe(p, t, q, DEDO);
            meio(p, t, q, PaintMedia::Watercolor);
            escolhe(p, t, q, AGUA);
            meio(p, t, q, PaintMedia::Digital);
        }),
    ];
    let mut falhas = Vec::new();
    for (i, (nome, volta)) in voltas.iter().enumerate() {
        let (mut p, mut t, mut toasts, _) = abre(&format!("espelho{i}"));
        meio(&mut p, &mut t, &mut toasts, PaintMedia::Digital);
        escolhe(&mut p, &mut t, &mut toasts, PASTEL);
        let pastel = limpo(&p, PASTEL);
        assert_eq!(preview(&t), pastel, "{nome}: o Pastel escolhido");
        volta(&mut p, &mut t, &mut toasts);
        sem_erros(&toasts);
        let ativo = p.vista.ativo.clone();
        let certo = match ativo.as_deref() {
            Some(rel) => preview(&t) == limpo(&p, rel),
            None => true,
        };
        if ativo.is_none() && !nome.starts_with("ligados") {
            falhas.push(format!("{nome}: o menu perdeu o preset"));
        }
        if !certo {
            falhas.push(format!("{nome}: o menu diz {ativo:?} e o pincel é outro"));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}
