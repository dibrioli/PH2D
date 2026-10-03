//! ⭐⭐ **O TRACEJADO ESTICADO PELA ROTA DO PRODUTO** (doc 121 §9.9) — formas de fábrica com traço
//! tracejado (o período AJUSTADO ao contorno pela `dash_fit`, as pontas e juntas do `StrokeSpec`),
//! esticadas, pelo `encode` do produto (a lei da casa: a geometria transformada, caneta e padrão
//! `× √|det|`) e pela placa (o eixo cortado pelo comprimento de arco no ecrã). Até ao §9.9 este quadro
//! ia inteiro ao Vello.

use super::*;
use ph2d_vec_scene::{LineCap, LineJoin, Rgba8, StrokeSpec};

fn tracejada(
    mut forma: VecPath,
    largura: f64,
    dash: (f64, f64),
    cap: LineCap,
    join: LineJoin,
) -> VecPath {
    let mut s = StrokeSpec::new(Rgba8::new(20, 30, 40, 240), largura);
    s.dash = Some(dash);
    s.cap = cap;
    s.join = join;
    forma.stroke = Some(s);
    forma
}

fn store() -> (VecPathStore, Vec<u32>) {
    let formas = [
        // ⚠️ Ponta QUADRADA: com a rente, um traço que acaba a menos de meia largura depois de uma
        // quina sai do traçador da casa com uma MORDIDA no lado de dentro, e a placa desenha a união
        // verdadeira (alfa `203` aqui; divergência DECLARADA, doc 121 §9.9).
        tracejada(
            ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.4),
            0.05,
            (2.0, 1.0),
            LineCap::Square,
            LineJoin::Miter,
        ),
        tracejada(
            ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5),
            0.06,
            (1.5, 1.5),
            LineCap::Round,
            LineJoin::Round,
        ),
        tracejada(
            ph2d_vec_scene::regular_polygon_rounded([0.0, 0.0], 0.5, 0.5, 6, 0.08),
            0.05,
            (6.0, 2.0),
            LineCap::Square,
            LineJoin::Bevel,
        ),
        tracejada(
            ph2d_vec_scene::star_rounded([0.0, 0.0], 0.5, 0.5, 6, 0.5, 0.08, 0.04),
            0.04,
            (3.0, 2.0),
            LineCap::Round,
            LineJoin::Miter,
        ),
    ];
    let mut s = VecPathStore::default();
    let hs = formas.into_iter().map(|f| s.push(f)).collect();
    (s, hs)
}

/// ⭐⭐ **A placa traceja o esticado como a cena Vello** — as barras do traço esticado do produto.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_rota_da_placa_traceja_o_esticado_como_a_casa() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter — o gate não correu");
        return;
    };
    let (store, hs) = store();
    let mut insts = copias(&hs, 160);
    for (i, c) in insts.iter_mut().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "uma fixtura pequena")]
        let k = (i as f32 * 0.618_034).fract();
        c.size[1] *= 0.35 + 2.45 * k;
    }
    assert!(
        insts.iter().filter(|c| !super::super::conforme(c)).count() > 120,
        "controlo: a fixtura tem de ser feita de copias NAO conformes"
    );
    let v = pelo_vello(&gpu, &insts, &store);
    let p = pela_placa(&gpu, &insts, &store);
    let (alfa, cor, nv, np, fora) = compara(&v, &p);
    fotografa("tracejado", &v, &p);
    eprintln!(
        "  tracejado esticado: alfa max {alfa} · cor max {cor} · {fora} px fora · vello {nv} px · placa {np} px"
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
