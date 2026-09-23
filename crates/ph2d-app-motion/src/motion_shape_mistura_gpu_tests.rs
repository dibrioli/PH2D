//! ⭐⭐⭐ **A MISTURA EM GRUPO, medida na PLACA** (doc 118 §1) — as três respostas em forma fechada
//! contra o que o Vello de facto pinta, para FORMAS e para QUADS de imagem.
//!
//! # A fixtura
//!
//! Uma faixa de `64×8` px com, por ordem: um FUNDO opaco a toda a largura (o cenário, `b`) e duas
//! cópias opacas que se sobrepõem — `A` em `x ∈ [4, 28]`, `B` em `x ∈ [20, 44]`. Lêem-se quatro
//! colunas, longe de toda aresta: só `A` (`x = 10`), a SOBREPOSIÇÃO (`x = 24`), só `B` (`x = 38`) e
//! só o fundo (`x = 54`).
//!
//! Para um modo `f(fundo, fonte)` e três cinzentos `b`, `a1`, `a2`:
//!
//! | alcance      | só A        | sobreposição       | só B        |
//! |--------------|-------------|--------------------|-------------|
//! | `Everything` | `f(b, a1)`  | `f(f(b, a1), a2)`  | `f(b, a2)`  |
//! | `Copies`     | `a1`        | `f(a1, a2)`        | `a2`        |
//! | `Scene`      | `f(b, a1)`  | `f(b, a2)`         | `f(b, a2)`  |
//! | (controlo)   | `a1`        | `a2`               | `a2`        |
//!
//! ⚠️ **O espaço é o CODIFICADO** (ordem do dono: *«Igual, como nas formas»*): o Vello mistura os
//! valores que recebe, e é nesse espaço que a forma de uma cena vectorial desta casa já se mistura.
//!
//! ⭐ **O controlo vem primeiro**: antes de afirmar uma célula, o gate afirma que as três linhas
//! da tabela DIFEREM na fixtura — *uma fixtura em que os três alcances dão o mesmo número não
//! distingue um arranjo de camadas de outro, e as asserções afirmariam nada*.

use std::sync::Arc;

use ph2d_eval_motion::{BlendWith, MisturaDoSink, VectorInstance};
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_vector::{Affine, VectorScene};

use super::super::{VecPathStore, encode};

const W: u32 = 64;
const H: u32 = 8;
/// Os três cinzentos da fixtura, no espaço codificado.
const B: f32 = 0.6;
const A1: f32 = 0.5;
const A2: f32 = 0.3;
/// As colunas lidas: só A · sobreposição · só B · só o fundo.
const COLUNAS: [u32; 4] = [10, 24, 38, 54];
/// ⚠️ A barra é de ARREDONDAMENTO: um canal `f32` passa a `u8` uma vez por camada, e há no máximo
/// duas camadas numa coluna. Os alcances distinguem-se por ≥ `18` bytes nesta fixtura (o
/// controlo mede-o), logo a barra não pode esconder a troca de um arranjo por outro.
const BARRA: i32 = 3;

fn try_headless_gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// As três leis de mistura que o sink leva à placa (doc 118), por tag.
fn f(tag: u8, fundo: f32, fonte: f32) -> f32 {
    match tag {
        1 => (fundo + fonte).min(1.0),
        3 => fundo * fonte,
        4 => fundo + fonte - fundo * fonte,
        _ => fonte,
    }
}

/// A linha esperada da tabela: `[só A, sobreposição, só B, só fundo]`.
fn esperado(tag: u8, com: Option<BlendWith>) -> [f32; 4] {
    match com {
        None => [A1, A2, A2, B],
        Some(BlendWith::Everything) => [f(tag, B, A1), f(tag, f(tag, B, A1), A2), f(tag, B, A2), B],
        Some(BlendWith::Copies) => [A1, f(tag, A1, A2), A2, B],
        Some(BlendWith::Scene) => [f(tag, B, A1), f(tag, B, A2), f(tag, B, A2), B],
    }
}

fn byte(x: f32) -> i32 {
    #[allow(clippy::cast_possible_truncation)]
    let v = (x.clamp(0.0, 1.0) * 255.0).round() as i32;
    v
}

/// Uma cópia de `largura × H` centrada em `cx`, em píxeis (a câmera é a identidade).
fn copia(
    cx: f32,
    largura: f32,
    cinza: f32,
    mistura: MisturaDoSink,
    imagem: Option<u32>,
) -> VectorInstance {
    VectorInstance {
        geometry_id: if imagem.is_some() { 0 } else { 1 },
        world_pos: [cx, H as f32 / 2.0],
        size: [largura, H as f32],
        basis: [1.0, 0.0, 0.0, 1.0],
        tint: [cinza, cinza, cinza, 1.0],
        texture_id: imagem.unwrap_or(0),
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        mistura,
    }
}

/// A imagem SÓLIDA de um cinzento, `4×4`, em alfa recta (opaca).
fn solida(cinza: f32) -> crate::motion_leaf_images::Art {
    #[allow(clippy::cast_sign_loss)]
    let v = byte(cinza) as u8;
    (4, 4, Arc::new([v, v, v, 255].repeat(16)))
}

/// Pinta a fixtura pela porta do PRODUTO ([`encode`]) e devolve as quatro colunas (canal R).
fn mede(
    pass: &mut VelloPass,
    gpu: &GpuContext,
    tag: u8,
    com: BlendWith,
    imagens: bool,
    sinks: (u32, u32),
) -> [i32; 4] {
    let mut store = VecPathStore::default();
    let quadrado = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
    assert_eq!(
        quadrado, 1,
        "a fixtura assume que o 1.º handle do armazém é `1`"
    );
    let fundo = copia(W as f32 / 2.0, W as f32, B, MisturaDoSink::default(), None);
    let m = |sink| MisturaDoSink {
        blend: tag,
        com,
        sink,
    };
    let (ia, ib) = if imagens {
        (Some(1), Some(2))
    } else {
        (None, None)
    };
    let insts = [
        fundo,
        copia(16.0, 24.0, A1, m(sinks.0), ia),
        copia(32.0, 24.0, A2, m(sinks.1), ib),
    ];
    let mut art = |id: u32, _uv: [f32; 4]| match id {
        1 => Some(solida(A1)),
        2 => Some(solida(A2)),
        _ => None,
    };
    let mut cena = VectorScene::new();
    encode(&insts, &store, &mut art, Affine::IDENTITY, None, &mut cena);
    let px = pass
        .render_and_readback(gpu, cena.inner(), (W, H))
        .expect("o readback");
    let linha = (H / 2) as usize * W as usize * 4;
    COLUNAS.map(|x| i32::from(px[linha + x as usize * 4]))
}

/// ⭐ O CONTROLO da fixtura: os três alcances DIFEREM em pelo menos uma coluna, para cada modo.
#[test]
fn a_fixtura_distingue_os_tres_alcances() {
    for tag in [1u8, 3, 4] {
        let linhas = BlendWith::ALL.map(|c| esperado(tag, Some(c)).map(byte));
        for i in 0..3 {
            for j in (i + 1)..3 {
                let separa = (0..4)
                    .map(|k| (linhas[i][k] - linhas[j][k]).abs())
                    .max()
                    .unwrap_or(0);
                assert!(
                    separa > 4 * BARRA,
                    "tag {tag}: os alcances {:?} e {:?} só se separam por {separa} bytes nesta \
                     fixtura — a barra de {BARRA} não os distinguiria",
                    BlendWith::ALL[i],
                    BlendWith::ALL[j]
                );
            }
        }
    }
}

/// ⭐⭐⭐ **A MEDIÇÃO** — cada modo × cada alcance × as duas médias, contra a tabela.
#[test]
#[ignore = "precisa de GPU"]
fn a_mistura_em_grupo_pinta_a_tabela_do_doc_118() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let Ok(mut pass) = VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (W, H)) else {
        println!("sem VelloPass — saltado");
        return;
    };
    let mut falhas = Vec::new();
    for imagens in [false, true] {
        let media = if imagens { "imagem" } else { "forma" };
        // ⚠️ O CONTROLO de cada média primeiro: sem mistura (tag 0) a leitura tem de ser o desenho
        // normal — senão as colunas não estão a ser comparadas contra nada.
        let normal = mede(&mut pass, &gpu, 0, BlendWith::Everything, imagens, (1, 1));
        let alvo = esperado(0, None).map(byte);
        assert!(
            normal.iter().zip(alvo).all(|(m, e)| (m - e).abs() <= BARRA),
            "{media}: sem mistura a faixa leu {normal:?} contra {alvo:?} — a fixtura não pinta o \
             que diz pintar, e nenhuma célula abaixo afirma nada"
        );
        for tag in [1u8, 3, 4] {
            for com in BlendWith::ALL {
                let lido = mede(&mut pass, &gpu, tag, com, imagens, (1, 1));
                let alvo = esperado(tag, Some(com)).map(byte);
                println!("  {media:6} tag {tag} {com:?}: lido {lido:?} · tabela {alvo:?}");
                if lido.iter().zip(alvo).any(|(m, e)| (m - e).abs() > BARRA) {
                    falhas.push(format!(
                        "{media} tag {tag} {com:?}: {lido:?} contra {alvo:?}"
                    ));
                }
            }
        }
        // ⛔ DECLARADO: `Subtract` (tag 2) não tem tradução na placa e desenha como o normal —
        // o gate PRENDE a fronteira, para que o dia em que ela mudar seja um acto deliberado.
        let sub = mede(&mut pass, &gpu, 2, BlendWith::Scene, imagens, (1, 1));
        assert_eq!(
            sub, normal,
            "{media}: o `Subtract` deixou de desenhar como o normal"
        );
        // ⭐ **Um grupo é um NÓ, não um modo** — dois sinks VIZINHOS com a mesma mistura em `Copies`
        // são DOIS grupos isolados: cada um pousa em `Normal`, logo na sobreposição ganha o de
        // cima (`a2`) e não `f(a1, a2)`. ⚠️ Nasceu de uma mutação SOBREVIVENTE: com um sink só na
        // fixtura, tirar o `sink` da chave da corrida não mudava um pixel.
        let dois = mede(&mut pass, &gpu, 3, BlendWith::Copies, imagens, (1, 2));
        let alvo = [A1, A2, A2, B].map(byte);
        if dois.iter().zip(alvo).any(|(m, e)| (m - e).abs() > BARRA) {
            falhas.push(format!(
                "{media} dois sinks em Copies: {dois:?} contra {alvo:?}"
            ));
        }
    }
    assert!(
        falhas.is_empty(),
        "células fora da barra de {BARRA}:\n{}",
        falhas.join("\n")
    );
}
