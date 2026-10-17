//! **A imagem do fim da gravação** (W8, `docs/Macros/05_plano_W8.md` itens 2b e 2c): a água assentada
//! e a trilha que toca (com as edições das etapas).

use super::RitmoDaAgua;
use super::etapas::imagens_da_gravacao;
use super::roteiro::{SPLASH, grava_como_a_maquina_e_o_tool};
use std::sync::atomic::AtomicBool;

/// O ritmo do worker de uma máquina (o de `tests/it/macro_comum.rs`) e quadros irregulares.
fn maquina() -> (RitmoDaAgua, [f32; 6]) {
    (
        RitmoDaAgua {
            estagios: vec![3, 5, 2, 7, 4, 1, 6],
            nao_volta: vec![false, false, false, true],
        },
        [16.667, 12.5, 20.8, 16.667, 18.9, 14.4],
    )
}

/// Os texels que diferem entre duas telas RGBA do mesmo tamanho.
fn diferem(a: &[u8], b: &[u8]) -> usize {
    assert_eq!(a.len(), b.len(), "tamanhos");
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .filter(|(x, y)| x != y)
        .count()
}

/// **2c — a água que corre no editor depois do Parar assenta na imagem do fim**: o editor continua com
/// a água no worker, em quadros irregulares, até ela parar; a imagem do fim da gravação é essa mesma,
/// ao byte. ⛔ Era o fim no Parar: a água que correu depois escondia o clipe inteiro (a lei do dono,
/// «onde a atual difere do fim, ganha a atual»).
#[test]
fn a_agua_que_corre_depois_do_parar_assenta_na_imagem_do_fim() {
    let (ritmo, dts) = maquina();
    let (mut t, bytes, _) =
        grava_como_a_maquina_e_o_tool(128, &SPLASH, &ritmo, &dts).expect("grava");
    let track = ph2d_macro::codec::read(&bytes)
        .expect("lê")
        .tracks
        .remove(0);
    let no_parar = t.imagem_aplicada().expect("a imagem no Parar").0;
    let mut q = 0;
    while !t.wet_assentada() {
        assert!(q < 40_000, "a água do editor não parou em {q} quadros");
        drop(t.take_preview_arc());
        ph2d_editor_core::tool::Tool::on_tick(&mut t, dts[q % dts.len()]);
        q += 1;
    }
    let editor = t.imagem_aplicada().expect("a imagem assentada").0;
    assert!(
        diferem(&no_parar, &editor) > 0,
        "controlo: a água não correu depois do Parar ({q} quadros)"
    );
    let g = imagens_da_gravacao(&track, &AtomicBool::new(false))
        .expect("reproduz")
        .expect("não parou");
    assert_eq!(
        diferem(&g.fim, &editor),
        0,
        "a imagem do fim não é a água assentada do editor ({q} quadros depois do Parar; no Parar \
         diferiam {})",
        diferem(&no_parar, &editor)
    );
}

/// **2b — editar uma etapa reescreve a imagem do editor** (W8, `docs/Macros/05_plano_W8.md` §2b): uma
/// gravação em Impasto (cor e relevo), o artista pinta depois do Parar, e a pincelada 2 fica maior. Os
/// texels que ainda eram da gravação passam aos do fim novo em TODOS os planos (cor, relevo, cobertura,
/// material); o traço do artista fica; o passo leva a transação, e o desfazer devolve tudo.
#[test]
fn editar_uma_etapa_reescreve_a_imagem_onde_ela_ainda_e_a_gravacao() {
    use super::etapas::edita_tamanho;
    use super::roteiro::grava_nos_meios_e_o_tool;
    use crate::PaintMedia;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool};
    const LADO: u32 = 96;
    let (mut t, bytes) =
        grava_nos_meios_e_o_tool(LADO, &SPLASH, &[PaintMedia::Impasto]).expect("grava");
    let crua = ph2d_macro::codec::read(&bytes)
        .expect("lê")
        .tracks
        .remove(0);
    let mut editada = crua.clone();
    edita_tamanho(&mut editada, 1, 30.0).expect("edita");
    let nao = AtomicBool::new(false);
    let de = imagens_da_gravacao(&crua, &nao)
        .expect("toca")
        .expect("não parou");
    let para = imagens_da_gravacao(&editada, &nao)
        .expect("toca")
        .expect("não parou");
    assert_eq!(
        t.imagem_aplicada().expect("documento").0,
        de.fim,
        "controlo: o editor no Parar é o fim da gravação"
    );
    // O artista pinta depois do Parar, a cinzento, no canto.
    t.set_brush_color_srgb8([90, 90, 90]);
    t.set_brush_size_px(6.0);
    for (i, phase) in [PointerPhase::Down, PointerPhase::Move, PointerPhase::Up]
        .into_iter()
        .enumerate()
    {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let x = 6.0 + 8.0 * i as f32;
        t.on_canvas_pointer(CanvasPointer {
            pos: [x, 88.0],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        t.on_tick(16.0);
    }
    assert!(
        diferem(&de.fim, &para.fim) > 0,
        "controlo: a edição muda o fim"
    );
    let snap_antes = t.snapshot_model();
    let ativa = snap_antes.layers.active().expect("camada ativa");
    // Os planos de um texel da camada ativa: cor, relevo (bits), cobertura, material.
    let texel = |s: &crate::undo::ModelSnapshot, k: usize| {
        let px = if s.layers.active() == Some(ativa) {
            s.canvas_rgba.as_slice()
        } else {
            s.images[&ativa].rgba8.as_slice()
        };
        (
            px[k * 4..k * 4 + 4].to_vec(),
            s.heights
                .get(&ativa)
                .and_then(|v| v.get(k))
                .map(|h| h.to_bits()),
            s.covers.get(&ativa).and_then(|v| v.get(k)).copied(),
            s.mats.get(&ativa).and_then(|v| v.get(k)).copied(),
        )
    };
    let n_tex = (LADO * LADO) as usize;
    let do_artista: Vec<bool> = (0..n_tex)
        .map(|k| texel(&snap_antes, k) != texel(&de.modelo.0, k))
        .collect();
    assert!(do_artista.iter().any(|&a| a), "controlo: o artista pintou");
    assert!(
        (0..n_tex)
            .any(|k| !do_artista[k] && texel(&de.modelo.0, k).1 != texel(&para.modelo.0, k).1),
        "controlo: a edição muda o relevo da gravação"
    );
    let antes = t.imagem_aplicada().expect("documento").0;

    let n = t.reescreve_a_gravacao(&de.modelo, &para.modelo, 7);
    assert!(n > 0, "a edição não reescreveu nada");
    assert_eq!(
        t.transacao_do_topo(false),
        Some(7),
        "o passo não leva a transação"
    );
    let vivo = t.snapshot_model();
    for (k, &artista) in do_artista.iter().enumerate() {
        let esperado = if artista { &snap_antes } else { &para.modelo.0 };
        assert!(
            texel(&vivo, k) == texel(esperado, k),
            "o texel {k} ({}) não segue a lei: {:?} × {:?}",
            if artista {
                "do artista"
            } else {
                "da gravação"
            },
            texel(&vivo, k),
            texel(esperado, k)
        );
    }

    assert!(t.undo_last(), "o desfazer");
    assert_eq!(
        t.imagem_aplicada().expect("documento").0,
        antes,
        "o desfazer não devolveu a imagem"
    );
    let desfeito = t.snapshot_model();
    assert!(
        (0..n_tex).all(|k| texel(&desfeito, k) == texel(&snap_antes, k)),
        "o desfazer não devolveu os planos"
    );
    assert_eq!(
        t.transacao_do_topo(true),
        Some(7),
        "o refazer não leva a transação"
    );
}
