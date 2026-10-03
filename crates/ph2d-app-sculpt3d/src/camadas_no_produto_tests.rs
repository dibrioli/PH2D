//! ⭐⭐⭐⭐ **As CAMADAS no produto** (`docs/3D/30` §5 e §11 — a W2): o traço
//! real (pen-down → movimentos COM QUADROS → pen-up), o desfazer e o
//! ficheiro, sobre a peça da cena `52`. Pedem a placa (`#[ignore]`).

use super::{HostDeTeste, Sculpt3dScene, cena_52};
use crate::pilha_da_peca::PilhaDaPeca;
use ph2d_tool_painter::LayerId;

/// Um traço com um QUADRO (`sync_mesh`) entre cada movimento — é o quadro que
/// desce as amostras sujas à camada e recompõe a peça.
fn traco_com_quadros(s: &mut Sculpt3dScene, gpu: &ph2d_gpu::GpuContext, x0: f32) {
    let mut host = HostDeTeste {
        ponteiro: (x0, 350.0),
    };
    assert!(
        crate::input_down::pointer_down(&mut host, s, winit::event::MouseButton::Left),
        "o pen-down não foi da cena"
    );
    for k in 1..=8u8 {
        crate::input::pointer_move(s, x0 + 6.0 * f32::from(k), 350.0);
        s.sync_mesh(gpu);
    }
    crate::input::pointer_up(s);
    s.sync_mesh(gpu);
}

fn pilha(s: &Sculpt3dScene) -> &PilhaDaPeca {
    s.objects[s.active]
        .pilha
        .as_ref()
        .expect("a peça com plano tem pilha")
}

fn bytes(s: &Sculpt3dScene, id: LayerId) -> Vec<u8> {
    let p = pilha(s);
    p.plano(id).expect("plano").rgba8(p.amostras()).to_vec()
}

fn bits(a: &[[f32; 3]]) -> Vec<[u32; 3]> {
    a.iter().map(|c| c.map(f32::to_bits)).collect()
}

/// ⛔ O INVARIANTE: o plano da peça É a composição da pilha, ao bit.
fn a_peca_e_a_composicao(s: &Sculpt3dScene, quando: &str) {
    let o = &s.objects[s.active];
    let peca = o.tinta.as_ref().expect("plano");
    let mut esperada = peca.clone();
    pilha(s).pinta_tinta(&mut esperada, || panic!("{quando}: a base é opaca"));
    assert_eq!(
        bits(peca.amostras()),
        bits(esperada.amostras()),
        "{quando}: a peça não é a composição da pilha"
    );
}

/// ⭐⭐⭐⭐ **GATE — Pintar escreve SÓ a camada activa, e o `Ctrl+Z` tira-o**
/// (`docs/3D/30` §5): uma camada nova por cima, um traço vermelho com quadros
/// a meio, e (1) a base fica intacta ao byte, (2) a de cima ganha a tinta com
/// opacidade, (3) a peça é a composição da pilha em todo o momento, (4) o
/// desfazer devolve a camada de cima ao transparente e a peça ao que era, (5)
/// gravar e abrir devolve a MESMA pilha.
#[test]
#[ignore = "precisa de adaptador"]
fn pintar_escreve_so_a_camada_activa_e_o_desfazer_a_tira() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    a_peca_e_a_composicao(&s, "ao armar");
    let base = pilha(&s).base().expect("base");
    let cima = s.objects[s.active]
        .pilha
        .as_mut()
        .expect("pilha")
        .nova_camada("cima")
        .expect("camada");
    let base_antes = bytes(&s, base);
    let peca_antes = s.objects[s.active]
        .tinta
        .as_ref()
        .expect("plano")
        .amostras()
        .to_vec();

    traco_com_quadros(&mut s, &gpu, 400.0);

    assert_eq!(
        bytes(&s, base),
        base_antes,
        "a BASE mudou — o traço caiu fora da camada activa"
    );
    let cima_px = bytes(&s, cima);
    let pintadas = cima_px
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[3] > 0)
        .count();
    assert!(
        pintadas > 50,
        "o CONTROLO: o traço pintou a camada de cima ({pintadas})"
    );
    a_peca_e_a_composicao(&s, "depois do traço");
    let peca = s.objects[s.active]
        .tinta
        .as_ref()
        .expect("plano")
        .amostras()
        .to_vec();
    assert_ne!(
        bits(&peca),
        bits(&peca_antes),
        "o CONTROLO: a peça mostra o traço"
    );

    assert!(s.undo_stroke(), "o desfazer tinha uma entrada");
    s.sync_mesh(&gpu);
    assert!(
        bytes(&s, cima).iter().all(|&b| b == 0),
        "o desfazer não devolveu a camada ao transparente"
    );
    assert_eq!(bytes(&s, base), base_antes);
    let peca = s.objects[s.active]
        .tinta
        .as_ref()
        .expect("plano")
        .amostras()
        .to_vec();
    assert_eq!(
        bits(&peca),
        bits(&peca_antes),
        "o desfazer não devolveu a peça ao que era"
    );

    assert!(s.redo_stroke(), "o refazer");
    s.sync_mesh(&gpu);
    assert_eq!(
        bytes(&s, cima),
        cima_px,
        "o refazer devolve a tinta ao byte"
    );
    a_peca_e_a_composicao(&s, "depois do refazer");

    let (lidas, _) = crate::doc::decode(&s.to_doc_bytes()).expect("o documento abre");
    assert_eq!(
        lidas[s.active].pilha.as_ref(),
        Some(pilha(&s)),
        "o ficheiro levou a pilha"
    );
}

/// ⭐⭐⭐ **GATE — Na peça de UMA camada o traço é o de sempre**: pinta a base,
/// a peça é a composição, e o desfazer devolve-a.
#[test]
#[ignore = "precisa de adaptador"]
fn na_peca_de_uma_camada_o_traco_pinta_a_base() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let base = pilha(&s).base().expect("base");
    let base_antes = bytes(&s, base);
    traco_com_quadros(&mut s, &gpu, 400.0);
    assert_ne!(bytes(&s, base), base_antes, "o traço pintou a base");
    a_peca_e_a_composicao(&s, "depois do traço");
    assert!(s.undo_stroke());
    s.sync_mesh(&gpu);
    assert_eq!(
        bytes(&s, base),
        base_antes,
        "o desfazer devolveu a base ao byte"
    );
    a_peca_e_a_composicao(&s, "depois do desfazer");
}
