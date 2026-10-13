//! ⭐⭐ **AS IMAGENS DE UMA SAÍDA EM GRUPO VÃO À PLACA DE FORMAS** (P4 do Motion, doc 132 §4,
//! `ph2d_gpu_cook::GpuCook::separa_as_imagens_do_grupo`) — a faixa da saída sai do buffer das sprites
//! (fica toda a zero: o passe de sprites não a pinta) e chega ao da placa tal e qual, com a linha de
//! imagem intacta e as de forma mudas como já eram.
//!
//! A fixtura é a do irmão ([`super::as_formas_no_dispositivo::fronteira`]): cinco linhas, uma delas
//! imagem (`geometry_id = 0`). Controlo: a mesma corrente sem grupo deixa as sprites como estavam.
//!
//! `#[ignore]`: precisa de adaptador. Na pista da GPU:
//!   `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-gpu-cook --test it \
//!        as_imagens_do_grupo --release -- --ignored --nocapture`

use super::as_formas_no_dispositivo::{fronteira, grafo, registry};
use super::placa::try_headless_gpu;
use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{
    CookClock, GpuCook, GpuCookError, plan, read_imagens_do_grupo, read_instances,
};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_render::SinkStyle;

/// Coze a fronteira com a saída em grupo ou não, e separa as imagens (o passo do quadro).
fn coze(gpu: &GpuContext, gc: &mut GpuCook, grupo: bool, s: &Stream) -> Result<u32, GpuCookError> {
    let reg = registry();
    let (g, obj, _, out) = grafo();
    let p = plan(&g, &reg, &reg, out);
    gc.set_saidas_em_grupo(vec![grupo]);
    let n = gc.cook(
        gpu,
        &g,
        &reg,
        &reg,
        &p,
        &[(obj, s)],
        CookClock {
            playhead: 0.0,
            tick: None,
        },
        [0.0, 0.0, 1.0, 1.0],
        [0.4, 0.4],
        SinkStyle::PLAIN,
    )?;
    gc.separa_as_imagens_do_grupo(gpu);
    Ok(n)
}

/// ⭐⭐ **A linha de imagem muda de buffer, ao bit** — e o controlo sem grupo não toca em nada.
#[test]
#[ignore = "precisa de adaptador"]
fn a_faixa_da_saida_em_grupo_passa_das_sprites_para_a_placa() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — saltado");
        return;
    };
    let mista = fronteira();
    // O CONTROLO primeiro: sem grupo, a imagem fica nas sprites e a placa não recebe nada.
    let mut gc = GpuCook::new();
    coze(&gpu, &mut gc, false, &mista).expect("coze");
    let antes = read_instances(&gpu, gc.instances().expect("sprites"));
    assert!(gc.imagens_do_grupo().is_none(), "sem grupo, nada à placa");
    let imagem = antes[2];
    assert!(
        imagem.size != [0.0, 0.0] && imagem.opacity > 0.0,
        "a fixtura tem de ter a imagem visível nas sprites ({imagem:?})"
    );
    assert!(
        antes
            .iter()
            .enumerate()
            .all(|(i, r)| i == 2 || r.size == [0.0, 0.0]),
        "as formas já vinham caladas nas sprites"
    );

    // Em grupo: a faixa inteira muda de buffer.
    coze(&gpu, &mut gc, true, &mista).expect("uma saída mista em grupo coze");
    let sprites = read_instances(&gpu, gc.instances().expect("sprites"));
    assert!(
        sprites
            .iter()
            .all(|r| bytemuck::bytes_of(r).iter().all(|&b| b == 0)),
        "a faixa em grupo fica toda a zero nas sprites"
    );
    let placa = gc.imagens_do_grupo().expect("as imagens foram à placa");
    assert_eq!(placa.len(), 5);
    assert_eq!(placa.faixas().len(), 1);
    assert_eq!(placa.faixas()[0].1, 0..5);
    assert_eq!(placa.texturas(), &[(0..5, 0)], "sem partição: o átlas");
    let linhas = read_imagens_do_grupo(&gpu, placa);
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&linhas),
        bytemuck::cast_slice::<_, u8>(&antes),
        "as linhas chegam à placa AO BIT"
    );

    // E o quadro seguinte SEM grupo esquece a separação.
    coze(&gpu, &mut gc, false, &mista).expect("coze");
    assert!(gc.imagens_do_grupo().is_none());
}

/// ⛔ **Uma saída em grupo com a coluna `blend` recusa** — o modo por linha substitui o do grupo e só
/// a cena vectorial o compõe. Controlo: a mesma corrente fora do grupo coze.
#[test]
#[ignore = "precisa de adaptador"]
fn o_modo_por_linha_numa_saida_em_grupo_recusa_o_cozimento() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — saltado");
        return;
    };
    let mut gc = GpuCook::new();
    // A fronteira SEM `geometry_id` (só imagens) e COM a coluna `blend`.
    let f = fronteira();
    let mut imagens = Stream::new(5);
    for c in ["P", "size", "rot", "tint"] {
        imagens.set(c, f.get(c).cloned().expect("a coluna da fixtura"));
    }
    imagens.set("blend", Column::Scalar(vec![2.0; 5]));
    assert!(
        coze(&gpu, &mut gc, false, &imagens).is_ok(),
        "fora do grupo, o modo por linha vai às sprites"
    );
    assert_eq!(
        coze(&gpu, &mut gc, true, &imagens),
        Err(GpuCookError::ModoDaLinhaEmGrupo)
    );
}
