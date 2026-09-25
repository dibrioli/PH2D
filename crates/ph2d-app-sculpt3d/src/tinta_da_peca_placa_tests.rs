//! ⭐⭐⭐ **O TECTO DA PLACA** — os gates da [`super::garante_no_orcamento`] e
//! do [`super::orcamento_da_placa`] (report do dono, 2026-09-24: *«a resolução
//! de 16x não chega para o painter»*).
//!
//! A escada subiu a `256x`, e o que a segura deixou de ser um nível escolhido:
//! é o que a placa aceita para AQUELA peça. Estes gates são puros — o orçamento
//! entra como número —, porque a rota do quadro pede um `wgpu::Device`; o elo
//! do quadro está no censo da fiação.

use super::*;

/// Um toro fechado de quads, pequeno: a contagem é exacta em qualquer tamanho.
fn toro() -> Mesh {
    ph2d_mesh::shapes::torus(8, 4, 1.0, 0.35)
}

fn amostras(m: &Mesh, k: u8) -> u64 {
    let faces = m.faces().iter().map(ph2d_mesh::Face::verts);
    ph2d_mesh_colors::Topologia::nova(m.vert_count(), faces, 0).amostras_ao_nivel(k)
}

/// ⭐⭐⭐ **Um degrau que NÃO CABE na placa DESCE ao maior que cabe** — nunca
/// estoura nem é construído na mesma.
///
/// ⚠️ **O orçamento é a conta EXACTA do `8x`**, logo a fronteira é testada no
/// ponto: o `8x` cabe à justa e o `16x` não. O CONTROLO é o mesmo pedido sem
/// tecto, que tem de ficar no degrau pedido — senão a descida podia ser de
/// toda a gente.
#[test]
fn um_degrau_que_nao_cabe_na_placa_desce_ao_maior_que_cabe() {
    let m = toro();
    let orcamento = amostras(&m, 3);
    let mut t = None;
    assert!(garante_no_orcamento(
        &m,
        &mut t,
        &mut None,
        Some(6),
        orcamento
    ));
    let t = t.expect("um degrau que cabe nasceu");
    assert_eq!(t.nivel(), 3, "o 64x não desceu ao 8x, o maior que cabe");
    assert_eq!(
        t.amostras().len() as u64,
        orcamento,
        "o plano não é o que a conta previu"
    );

    let mut sem_tecto = None;
    garante_no_orcamento(&m, &mut sem_tecto, &mut None, Some(6), u64::MAX);
    assert_eq!(
        sem_tecto.map(|t| t.nivel()),
        Some(6),
        "o CONTROLO: sem tecto a peça fica no degrau pedido"
    );
}

/// ⛔ **Se nem o `2x` cabe, não há plano** — um plano de nível zero é a cor
/// por vértice, que a peça já tem sem gastar placa nenhuma.
#[test]
fn se_nem_o_2x_cabe_nao_ha_plano() {
    let m = toro();
    let mut t = None;
    garante_no_orcamento(&m, &mut t, &mut None, Some(4), amostras(&m, 1) - 1);
    assert!(t.is_none(), "nasceu um plano que a placa não aceita");
}

/// ⚠️ **Descer não é reconstruir a cada quadro:** com o pedido ainda acima do
/// que cabe (o caso de uma peça que não está activa, cujo pedido é o da cena),
/// a segunda chamada tem de reconhecer o plano descido e não tocar em nada.
#[test]
fn o_plano_descido_nao_e_refeito_no_quadro_seguinte() {
    let m = toro();
    let orcamento = amostras(&m, 3);
    let mut t = None;
    assert!(garante_no_orcamento(
        &m,
        &mut t,
        &mut None,
        Some(6),
        orcamento
    ));
    assert!(
        !garante_no_orcamento(&m, &mut t, &mut None, Some(6), orcamento),
        "o plano descido foi refeito com o mesmo pedido"
    );
}

/// ⭐⭐ **O orçamento é o MENOR dos dois tectos da placa, sobre `12` bytes por
/// amostra** — o plano vive num buffer de armazenamento, e os dois limites se
/// aplicam a ele. Cada metade testa o lado que decide.
#[test]
fn o_orcamento_e_o_menor_tecto_da_placa_sobre_doze_bytes() {
    let lim = wgpu::Limits {
        max_storage_buffer_binding_size: 1_200,
        max_buffer_size: 1 << 40,
        ..wgpu::Limits::default()
    };
    assert_eq!(orcamento_da_placa(&lim), 100, "o tecto do binding decide");
    let lim = wgpu::Limits {
        max_storage_buffer_binding_size: u32::MAX.into(),
        max_buffer_size: 2_400,
        ..wgpu::Limits::default()
    };
    assert_eq!(orcamento_da_placa(&lim), 200, "o tecto do buffer decide");
}
