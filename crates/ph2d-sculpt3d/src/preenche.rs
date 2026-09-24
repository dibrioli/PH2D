//! ⭐⭐⭐ **O PREENCHIMENTO** — a peça inteira com a cor do pincel, respeitando
//! a máscara. O gesto `Fill` que o dono pediu a 2026-09-24 (*«1 e 2 no mesmo
//! ciclo»*, sendo a 1 este botão).
//!
//! # ⭐⭐ Porque é que isto é UMA função e não uma lei nova
//!
//! O pincel de pintura já responde à pergunta *«quanto desta amostra a máscara
//! deixa pintar?»* — é o `keep` que o [`crate::tinta_fina`] calcula por amostra
//! (a soma dos pesos da retícula contra a máscara dos cantos, e o
//! [`crate::mask_ops::free_weight`] por cima). ⛔ Escrever essa conta outra vez
//! aqui seria a **segunda resposta à mesma pergunta**, e as duas divergem no dia
//! em que alguém mexer numa: *o `Fill` pintaria por cima de uma zona que o
//! pincel respeita, ou ao contrário*. ⇒ [`keep_da_amostra`] é a porta, e os
//! dois chamadores são o carimbo do pincel e este preenchimento.
//!
//! # ⚠️ A mistura é EXACTA nas duas pontas
//!
//! `a·(1 − keep) + c·keep`, e não `a + (c − a)·keep`: a segunda forma devolve
//! `a + (c − a)` com `keep = 1`, que em `f32` **não é** `c` em geral. Com a
//! primeira, uma amostra livre sai **exactamente** a cor do pincel e uma
//! mascarada sai **exactamente** a cor que tinha — e é isso que os gates
//! afirmam ao bit.
//!
//! # ⚠️ Uma amostra partilhada é decidida pela PRIMEIRA face que a alcança
//!
//! Uma amostra de aresta pertence a duas faces, e os pesos de uma retícula
//! baricêntrica e os de uma bilinear sobre a MESMA aresta podem diferir no
//! último bit (`i/l` contra `1 − j/l`). ⇒ um carimbo, em ordem de face, e a
//! primeira a chegar decide — a regra que o carimbo do pincel já segue. ⭐ Nos
//! cantos isto é indiferente por construção: o peso de um canto é `(1, 0, 0)`
//! exacto, logo a amostra de um vértice lê `free_weight(m_v)` venha de que face
//! vier, **ao bit o que o preenchimento por vértice escreve**.

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;

/// ⭐ **Quanto desta amostra a máscara deixa pintar** — a lei partilhada.
///
/// `w` são os pesos da amostra sobre os cantos da face (baricêntricos num
/// triângulo, bilineares num quad) e `m` a máscara de cada canto, na mesma
/// ordem. Um vértice isolado é o caso `w = [1]`.
#[must_use]
pub fn keep_da_amostra(w: &[f32], m: &[f32]) -> f32 {
    let k: f32 = w.iter().zip(m).map(|(a, b)| a * b).sum();
    crate::mask_ops::free_weight(k)
}

/// A mistura, exacta nas duas pontas — ver o cabeçalho.
#[must_use]
pub fn mistura(antes: [f32; 3], cor: [f32; 3], keep: f32) -> [f32; 3] {
    let fica = 1.0 - keep;
    [
        antes[0] * fica + cor[0] * keep,
        antes[1] * fica + cor[1] * keep,
        antes[2] * fica + cor[2] * keep,
    ]
}

fn mascara(mesh: &Mesh, v: u32) -> f32 {
    mesh.masks()
        .map_or(ph2d_mesh::DEFAULT_MASK, |k| k[v as usize])
}

/// **Preenche a cor POR VÉRTICE** — o caminho sem tinta fina.
///
/// Devolve se alguma cor mudou: uma malha toda mascarada não muda nada, e um
/// passo de desfazer sobre isso nomearia uma edição que não aconteceu.
pub fn preenche_vertices(mesh: &mut Mesh, cor: [f32; 3]) -> bool {
    let keeps: Vec<f32> = (0..mesh.vert_count())
        .map(|v| keep_da_amostra(&[1.0], &[mascara(mesh, v as u32)]))
        .collect();
    let mut mudou = false;
    for (c, keep) in mesh.colors_mut().iter_mut().zip(keeps) {
        let nova = mistura(*c, cor, keep);
        mudou |= nova != *c;
        *c = nova;
    }
    mudou
}

/// Porque o plano não foi preenchido.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Recusa {
    /// O plano não descreve esta malha — contagens ou a forma de uma face.
    ///
    /// ⛔⛔ **A lei do §14 da tinta fina:** indexar um plano que não descreve a
    /// malha estoura (faces a mais) ou, pior, escreve tinta válida no sítio
    /// errado (faces a menos). ⇒ recusa ANTES de qualquer escrita.
    NaoDescreve,
}

/// ⭐⭐ **Preenche o PLANO DE TINTA FINA**, amostra a amostra, com a máscara
/// interpolada da retícula.
///
/// Devolve se alguma amostra mudou.
///
/// # Errors
///
/// [`Recusa::NaoDescreve`] quando o plano não descreve a malha — e nesse caso
/// nenhuma amostra foi escrita.
pub fn preenche_plano(tinta: &mut Tinta, mesh: &Mesh, cor: [f32; 3]) -> Result<bool, Recusa> {
    let topo = tinta.topologia();
    if !topo.descreve(mesh.vert_count(), mesh.faces().len())
        || mesh
            .faces()
            .iter()
            .enumerate()
            .any(|(f, face)| face.verts().len() != topo.cantos_de(f))
    {
        return Err(Recusa::NaoDescreve);
    }
    let mut feita = vec![false; tinta.amostras().len()];
    let mut keeps: Vec<(u32, f32)> = Vec::new();
    // ⭐ **Os VÉRTICES primeiro, e pela MESMA chamada do preenchimento por
    // vértice:** a amostra de um vértice é o próprio índice dele (a lei do
    // `de_vertice`), logo decidi-las aqui torna o prefixo por-vértice do plano
    // **igual ao bit** ao que o [`preenche_vertices`] escreve — e cobre o
    // vértice que nenhuma face alcança, que o laço das faces deixaria de fora.
    for (v, f) in feita.iter_mut().enumerate().take(topo.verts()) {
        *f = true;
        keeps.push((
            v as u32,
            keep_da_amostra(&[1.0], &[mascara(mesh, v as u32)]),
        ));
    }
    for (fi, face) in mesh.faces().iter().enumerate() {
        let cantos = face.verts();
        let lado = tinta.lado_da_face(fi) as f32;
        let m: Vec<f32> = cantos.iter().map(|&v| mascara(mesh, v)).collect();
        let mut pousa = |idx: u32, w: &[f32]| {
            if !std::mem::replace(&mut feita[idx as usize], true) {
                keeps.push((idx, keep_da_amostra(w, &m)));
            }
        };
        if cantos.len() == 3 {
            tinta.para_cada_amostra_tri(fi, cantos, |idx, (i, j, k)| {
                pousa(idx, &[i as f32 / lado, j as f32 / lado, k as f32 / lado]);
            });
        } else {
            tinta.para_cada_amostra_quad(fi, cantos, |idx, (i, j)| {
                pousa(
                    idx,
                    &crate::tinta_fina::bilinear(i as f32 / lado, j as f32 / lado),
                );
            });
        }
    }
    let amostras = tinta.amostras_mut();
    let mut mudou = false;
    for (idx, keep) in keeps {
        let a = &mut amostras[idx as usize];
        let nova = mistura(*a, cor, keep);
        mudou |= nova != *a;
        *a = nova;
    }
    Ok(mudou)
}
