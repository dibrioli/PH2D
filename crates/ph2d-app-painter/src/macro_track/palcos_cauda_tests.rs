//! W14, `docs/Macros/12_W14_a_agua_depois_do_parar.md` — **no fim do clipe a ranhura corre a água até à
//! imagem do artista** (decisão do dono 20/10): pela ordem da shell (segue, a marca liga-se, sobe), com a
//! cabeça parada no fim e o editor a correr a água dele ao lado. Filho de `palcos_fim_tests`.

use super::*;
use ph2d_tool_painter::PaintMedia;

const L: u32 = 96;
/// Um quadro de 60 Hz.
const DT: f32 = 1000.0 / 60.0;

/// O maior desvio, em níveis, entre duas ranhuras.
fn desvio(a: &[u8], b: &[u8]) -> u8 {
    a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).max().unwrap_or(0)
}

/// ⭐⭐ **No fim do clipe a ranhura anda até ao documento do editor**: sem o artista tocar (a água assenta
/// nos dois) e com uma vertical dele meio segundo depois do Parar (a água da gravação pára nesse toque,
/// W12). O 1.º quadro no fim ainda é a tela do Parar — a água anda, não salta. ⛔ VERMELHO antes da W14:
/// a ranhura ficava na tela do Parar (sonda W14: 93 % da área pintada muda ao sair do clipe).
#[test]
fn no_fim_do_clipe_a_ranhura_corre_a_agua_ate_a_imagem_do_artista() {
    for pinta in [false, true] {
        let mut world = World::new();
        let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
        let (mut t, doc, d) =
            ph2d_tool_painter::macro_track::roteiro::grava_duas_horizontais_ligado_em(
                L,
                e.to_bits(),
                PaintMedia::WetPaint,
            )
            .expect("grava");
        let mut c = Clipe::com(world, e, &doc);
        c.palcos.dt_do_quadro(DT);
        let _ = c.quadro_ligado(0.1, Some(&mut t), &mut nunca);
        if pinta {
            (0..30).for_each(|_| tique(&mut t));
            t.set_brush_size_px(8.0);
            #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
            let l = L as f32;
            traco(&mut t, [0.5 * l, 0.1 * l], [0.5 * l, 0.9 * l], 12, true);
        }
        let primeiro = c.quadro_ligado(d, Some(&mut t), &mut nunca);
        let mut r = primeiro.clone();
        let mut parados = 0;
        for _ in 0..40_000 {
            tique(&mut t);
            let agora = c.quadro_ligado(d, Some(&mut t), &mut nunca);
            parados = if agora == r && t.agua_parada() {
                parados + 1
            } else {
                0
            };
            r = agora;
            if parados > 3 {
                break;
            }
        }
        let documento = premul(&t.imagem_aplicada().expect("documento").0);
        assert!(
            desvio(&primeiro, &documento) > 8,
            "controlo (pinta={pinta}): no 1.º quadro do fim a água ainda não correu"
        );
        let dv = desvio(&r, &documento);
        assert!(
            dv <= 1,
            "pinta={pinta}: no fim da cauda a ranhura difere do documento em {dv} níveis"
        );
    }
}

/// ⭐ **Gravado com a água da gravação a correr e reaberto, a cauda pára na imagem gravada**: o ficheiro
/// guarda onde a imagem a tem (mesmo sem tinta do artista — uma marca vazia não se gravava), e no editor a
/// água continua. Reaberto (a sprite, sem o Painter), a ranhura no fim do clipe é a imagem gravada.
#[test]
fn gravado_com_a_agua_a_correr_e_reaberto_a_cauda_para_na_imagem_gravada() {
    let mut world = World::new();
    let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
    let (mut t, doc, d) = ph2d_tool_painter::macro_track::roteiro::grava_duas_horizontais_ligado_em(
        L,
        e.to_bits(),
        PaintMedia::WetPaint,
    )
    .expect("grava");
    let mut c = Clipe::com(world, e, &doc);
    let _ = c.quadro_ligado(0.1, Some(&mut t), &mut nunca);
    (0..20).for_each(|_| tique(&mut t));
    assert!(!t.agua_parada(), "controlo: a água corre ao gravar");
    let marcas = c.palcos.marcas_a_gravar(Some(&t));
    assert!(
        t.marca_do_artista()
            .is_some_and(|m| m.agua_da_gravacao_parou().is_none()),
        "no editor a água da gravação continua depois de gravar"
    );
    let gravada = t.imagem_aplicada().expect("documento").0;
    let mut r = Clipe::novo(&doc);
    assert!(r.palcos.restaura_marcas(&marcas), "o ficheiro abre");
    r.palcos.dt_do_quadro(DT);
    let mut le = |_: u64, _: &mut Placa| Some((gravada.clone(), L, L));
    // A água gravada é a de 20 quadros depois do Parar: 200 quadros chegam e sobram (um quadro sem um passo
    // inteiro da água não muda a ranhura — parar no 1.º igual cortaria a cauda).
    let mut ranhura = Vec::new();
    for _ in 0..200 {
        ranhura = r.quadro_ligado(d, None, &mut le);
    }
    let dv = desvio(&ranhura, &premul(&gravada));
    assert!(dv <= 1, "reaberto, a cauda não parou na imagem gravada ({dv} níveis)");
}
