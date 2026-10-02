//! ⭐⭐ **AS LETRAS DO `source.text` PELA PLACA** — os glifos montados pela MESMA porta do produto
//! (`motion_text_gen::build_stream`, a fonte do sistema), pelo `encode` do produto e pela placa.
//!
//! ⚠️ O censo das rotas (`motion_bridge_gpu_rota_das_formas_probe`) mostra as cenas com texto a ir à
//! placa desde a W3, e nenhum gate de pixel tinha um glifo: as formas de fábrica não têm FUROS (o
//! miolo do `o`, do `B`, do `8` — sub-caminhos de sentido oposto somados pela regra não-nula) nem as
//! quádricas de uma fonte, e uma letra pequena é toda borda.

use super::*;
use ph2d_node_source_text::{Align, Pivot, TextParams};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_vector::PathEl;

/// Os glifos de `texto`, um handle por letra com contorno.
fn letras(texto: &str) -> (VecPathStore, Vec<u32>) {
    let mut s = VecPathStore::default();
    let p = TextParams {
        size: 1.0,
        tracking: 0.0,
        line_height: 1.2,
        align: Align::Left,
        weight: 400.0,
        pivot: Pivot::Center,
    };
    let stream = crate::motion_text_gen::build_stream(&mut s, &p, "", texto);
    let Some(Column::Scalar(g)) = Stream::get(&stream, "geometry_id") else {
        panic!("o stream do texto não publica geometry_id");
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o handle é inteiro pequeno guardado num f32, a convenção do `geometry_id`"
    )]
    let hs = g.iter().map(|&h| h as u32).collect();
    (s, hs)
}

/// Quantos glifos têm mais de um sub-caminho (um furo, ou duas peças).
fn com_furo(store: &VecPathStore, hs: &[u32]) -> usize {
    hs.iter()
        .filter(|&&h| {
            let path = store.get(h).expect("o handle do glifo existe");
            let forma = ph2d_vec_render::forma_para_a_placa(path).expect("o glifo vai à placa");
            let (bp, _) = forma.fill.expect("o glifo é um preenchimento");
            bp.elements()
                .iter()
                .filter(|e| matches!(e, PathEl::MoveTo(_)))
                .count()
                > 1
        })
        .count()
}

/// ⭐⭐ **A placa desenha as letras como a cena Vello** — um terço das cópias no tamanho da paridade
/// das formas, um terço PEQUENAS (`~3–20 px` de corpo) e um terço ESTICADAS. As barras são as do
/// [`super::a_rota_da_placa_desenha_o_que_a_cena_vello_desenha`].
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_rota_da_placa_desenha_as_letras_como_a_cena_vello() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter — o gate não correu");
        return;
    };
    let (store, hs) = letras("Bog8@aeQR%&");
    assert!(
        hs.len() >= 10 && com_furo(&store, &hs) >= 8,
        "controlo: a fixtura tem de ter letras com FURO ({} glifos, {} com furo)",
        hs.len(),
        com_furo(&store, &hs)
    );
    let mut insts = copias(&hs, 240);
    for (i, c) in insts.iter_mut().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "uma fixtura pequena")]
        let k = (i as f32 * 0.618_034).fract();
        match i % 3 {
            0 => c.size = [c.size[0] * 0.25, c.size[1] * 0.25],
            1 => c.size[1] *= 0.35 + 2.45 * k,
            _ => {}
        }
    }
    let v = pelo_vello(&gpu, &insts, &store);
    let p = pela_placa(&gpu, &insts, &store);
    let (alfa, cor, nv, np, fora) = compara(&v, &p);
    fotografa("letras", &v, &p);
    eprintln!(
        "  letras: alfa max {alfa} · cor max {cor} · {fora} px fora · vello {nv} px · placa {np} px"
    );
    assert!(nv > 20_000, "controlo: a cena pinta pouco ({nv} px)");
    let diff = nv.abs_diff(np);
    assert!(diff * 100 <= nv, "área pintada diverge: {nv} contra {np}");
    assert!(alfa <= 100, "alfa {alfa} acima da barra das curvas (100)");
    assert!(cor <= 100, "cor {cor} acima da barra das curvas (100)");
    assert!(
        fora * 20 <= nv,
        "{fora} pixels desviam > 16 — acima de 5 % dos {nv} pintados"
    );
}
