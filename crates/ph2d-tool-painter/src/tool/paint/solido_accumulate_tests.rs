//! **Com Solid o Accumulate não existe** (decisão do dono, 2026-10-06): sai do painel e o traço corre
//! sem ele. O lado do painel é `seam_accumulate_por_meio::com_solid_o_accumulate_sai_do_painel`.

use super::solido_meios_tests::laco_com;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;

/// ⭐ **COM SOLID O ACCUMULATE NÃO EXISTE** (decisão do dono, 2026-10-06: o Accumulate sai do painel e
/// o traço corre sem ele) — o gesto com Solid é, ao byte, o mesmo com o Accumulate ligado ou
/// desligado, em Strength 0,4 e 1 e com Flow 0,3. Vermelho antes: com o Accumulate ligado o traço
/// chegava a `0,744` e a mancha a `0,400`.
#[test]
fn com_solid_o_accumulate_nao_muda_o_traco() {
    type Ajuste = (&'static str, fn(&mut PainterTool));
    let casos: [Ajuste; 3] = [
        ("Strength 0,4", |t| t.paint.brush.strength = 0.4),
        ("Strength 1", |_| {}),
        ("Strength 0,4 + Flow 0,3", |t| {
            t.paint.brush.strength = 0.4;
            t.paint.brush.flow = 0.3;
        }),
    ];
    for (nome, f) in casos {
        let corre = |acumula: bool| {
            laco_com(PaintMedia::Digital, 1.0, &|t: &mut PainterTool| {
                f(t);
                t.paint.brush.accumulate = acumula;
                t.paint.brush.space_attenuation = acumula;
            })
            .canvas_rgba
            .to_vec()
        };
        let (com, sem) = (corre(true), corre(false));
        assert!(
            sem.iter().any(|&b| b != 255),
            "controlo: {nome}: o gesto pintou"
        );
        let difere = com.iter().zip(&sem).filter(|(a, b)| a != b).count();
        assert_eq!(
            difere, 0,
            "{nome}: com Solid o Accumulate ainda muda {difere} bytes do gesto"
        );
    }
}
