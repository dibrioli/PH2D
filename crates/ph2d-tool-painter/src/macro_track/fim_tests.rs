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
