//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA da wave das ÂNCORAS DO HUD** — o item que o
//! [handoff do #20](../../../../docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_HUD_2026-09-17.md)
//! §7 deixou aberto por escrito: *«as quatro âncoras do `VecAnchors` não estão ligadas ao canvas»*.
//!
//! ⚠️⚠️ **A pergunta ZERO vem antes da do §5.0, e esta linha pagou-a caro hoje:** *isto já está
//! CONSTRUÍDO?* Medido com controlo positivo — `UiCanvas` lê `35` ocorrências, **`UiAnchor` lê `0`**
//! e o `git log -S "UiAnchor"` não devolve commit nenhum. ⇒ nunca existiu.
//!
//! ⚠️ **Sonda, não gate.** Corre com `--ignored` e IMPRIME.
//!
//! ```text
//! cargo test -p ph2d-app-components --test it \
//!     mede_o_que_falta_as_ancoras_do_hud -- --ignored --nocapture
//! ```
//!
//! # As perguntas, uma por bloco
//!
//! A) **A BANDA** — o canvas com `Keep` numa janela larga deixa vista por preencher?
//! B) **O CANTO** — um filho no canto da caixa de referência chega à borda REAL do ecrã?
//! C) **A ÂNCORA DE HOJE** — ela move esse filho?
//! D) **O CONTRAFACTUAL** — e se a caixa que ela mede fosse a EFECTIVA?

use ph2d_ecs::VecAnchors;
use ph2d_hud::{Canvas, Fit, View, bands, place};

/// A caixa de referência de fábrica desta casa: `32 × 18`, que é `16:9`.
const REF_W: f32 = 32.0;
const REF_H: f32 = 18.0;

/// Três janelas: a do aspecto exacto, uma ULTRAWIDE e uma alta.
const VISTAS: [(&str, f32, f32); 3] = [
    ("16:9  (o aspecto da caixa)", 16.0, 9.0),
    ("21:9  (ultrawide)", 21.0, 9.0),
    ("4:3   (alta)", 16.0, 12.0),
];

#[test]
#[ignore = "sonda do §5.0 — imprime, nao afirma"]
fn mede_o_que_falta_as_ancoras_do_hud() {
    println!("\n=== §5.0 · o que falta as ANCORAS do HUD ===\n");
    let canvas = Canvas::new(REF_W, REF_H, Fit::Keep).expect("canvas");

    // ─── A) A BANDA ──────────────────────────────────────────────────────────
    println!("A) A BANDA — o que sobra da vista com `Keep`");
    for (nome, hw, hh) in VISTAS {
        let v = View {
            center: [0.0, 0.0],
            half: [hw, hh],
        };
        let p = place(&canvas, v);
        let b = bands(&canvas, v);
        println!(
            "   {nome}  ->  escala {:.4}   banda x = {:.4}   banda y = {:.4}",
            p.scale[0], b[0], b[1]
        );
    }
    println!(
        "   ⭐ A banda EXISTE e ja' e' calculada. O doc do `bands` diz, por escrito, que ela\n\
               «nao entra na pose — soma'-la seria descentrar o que ja' esta' centrado».\n\
               ⇒ ela e' vista VAZIA entre a caixa de referencia e a borda real do ecra.\n"
    );

    // ─── B) O CANTO ──────────────────────────────────────────────────────────
    println!("B) O CANTO — um filho no canto da caixa chega a' borda real?");
    let v = View {
        center: [0.0, 0.0],
        half: [21.0, 9.0],
    };
    let p = place(&canvas, v);
    let b = bands(&canvas, v);
    // O canto direito da caixa de referencia, levado ao mundo pela pose da raiz.
    let canto_mundo = REF_W / 2.0 * p.scale[0] + p.translate[0];
    let borda_mundo = v.half[0];
    println!("   canto da caixa, em mundo  = {canto_mundo:.4}");
    println!("   borda real da vista       = {borda_mundo:.4}");
    println!(
        "   ⛔ NAO: faltam {:.4} de mundo — que e' exactamente a banda ({:.4}).\n\
               *A pontuacao «colada no canto» fica a uma banda do canto.*\n",
        borda_mundo - canto_mundo,
        b[0]
    );

    // ─── C) A ÂNCORA DE HOJE ─────────────────────────────────────────────────
    println!("C) A ANCORA DE HOJE — ela move esse filho?");
    let base = [
        f64::from(-REF_W / 2.0),
        f64::from(-REF_H / 2.0),
        f64::from(REF_W / 2.0),
        f64::from(REF_H / 2.0),
    ];
    // A regra que o artista arma: preso a' aresta MAXIMA em x (o canto direito).
    let a = VecAnchors {
        min: [1.0, 1.0],
        max: [1.0, 1.0],
        base,
    };
    let [dmin, dmax] = a.delta_local(base);
    println!("   base  = {base:?}");
    println!("   agora = {base:?}   (a caixa de referencia NAO muda — o canvas ESCALA)");
    println!("   delta = min {dmin:?}  max {dmax:?}");
    println!(
        "   ⛔ ZERO, e por CONSTRUCAO. A ancora responde «a moldura mudou de tamanho?» e a\n\
               caixa do canvas e' a MESMA em toda janela — o que muda e' a ESCALA da raiz.\n\
               ⇒ nao e' um fio por ligar: e' a regua a medir uma grandeza que nao se mexe.\n"
    );

    // ─── D) O CONTRAFACTUAL ──────────────────────────────────────────────────
    println!("D) O CONTRAFACTUAL — e se a caixa medida fosse a EFECTIVA?");
    for (nome, hw, hh) in VISTAS {
        let v = View {
            center: [0.0, 0.0],
            half: [hw, hh],
        };
        let p = place(&canvas, v);
        let b = bands(&canvas, v);
        // A banda, trazida a unidades LOCAIS do canvas (dividir pela escala da raiz).
        let bx = f64::from(b[0] / p.scale[0]);
        let by = f64::from(b[1] / p.scale[1]);
        let efectiva = [base[0] - bx, base[1] - by, base[2] + bx, base[3] + by];
        let [_, dmax] = a.delta_local(efectiva);
        let alcanca = (f64::from(canto_local() + dmax[0] as f32) * f64::from(p.scale[0])
            - f64::from(v.half[0]))
        .abs();
        println!(
            "   {nome}  ->  efectiva x = [{:.3}, {:.3}]   dmax = [{:.3}, {:.3}]   erro a' borda = {alcanca:.6}",
            efectiva[0], efectiva[2], dmax[0], dmax[1]
        );
    }
    println!(
        "   ⭐⭐ CHEGA, e o neutro e' EXACTO: na janela de 16:9 a banda e' zero, a caixa efectiva\n\
               E' a de referencia, e o delta e' `0.0` por subtraccao de iguais ⇒ **byte-identico**\n\
               ao que se desenha hoje.\n"
    );

    println!("=== o que a wave TEM de trazer ===");
    println!("   1. a caixa EFECTIVA do canvas, por quadro, em unidades locais dele");
    println!("   2. o passe das ancoras a aceita'-la como «a moldura», ao lado do `VecFrame`");
    println!("   ⛔ e NADA de lei de ancora nova: o `delta_local` ja' responde certo — ele so'");
    println!("      precisa de receber uma caixa que de facto mude.");
    println!(
        "   ⭐⭐ E isto DISSOLVE a recusa declarada do `Fit::Expand`: ela dizia que chegar a'\n\
         borda «exigiria redimensionar a moldura por quadro, que e' escrever no DOCUMENTO» —\n\
         e a caixa efectiva e' DERIVADA por quadro, como a pose, sem tocar no documento.\n"
    );
}

/// O canto direito da caixa de referência, em unidades locais do canvas.
fn canto_local() -> f32 {
    REF_W / 2.0
}
