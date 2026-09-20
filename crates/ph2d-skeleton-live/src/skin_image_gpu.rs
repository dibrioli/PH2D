//! ⭐⭐⭐ **A MALHA DE UMA IMAGEM PRESA, NO FORMATO QUE A PLACA POSA** (F9 W2, ordem do dono de
//! 2026-09-20: *«implemente se esse é o padrão ouro»*).
//!
//! ⚠️ **Irmão do [`crate::skin_image`] por RESPONSABILIDADE e pelo tecto de LOC:** ali mora *o que
//! é um ponto desta mídia e onde ele vai parar*; aqui, *o que a placa precisa de receber para o
//! pôr lá*.
//!
//! # ⛔⛔ Porque isto existe: a medição
//!
//! Na arte do dono, `--release`, `load 4,05` (sonda `de_que_e_feito_o_quadro_da_pele`):
//! `attach_skin_meshes` custava **`35,9 %` de um quadro a 8 imagens**, e **`96 %` disso era a lei a
//! correr POR VÉRTICE**. O custo é **exactamente linear** no número de imagens ⇒ `16` imagens são
//! `72 %` e `23` são o quadro inteiro. O dispositivo desenha `100 352` triângulos em `0,73 ms`.
//! *É o §0.0 à letra: o caminho mais lento a definir o tecto do mais rápido.*
//!
//! # ⭐⭐⭐ O que muda, e o que NÃO muda
//!
//! O [`ph2d_render::SpriteMesh`] passa a trazer o **REPOUSO** e uma [`SpriteMeshSkin`]; quem posa
//! para DESENHAR é o `vs_main`. ⛔⛔ **A lei não muda, e ela NÃO é uma mistura linear de afins** —
//! o [`Skin::blend`] roda em torno da JUNTA desde 2026-09-19, e é essa que sobe (com a tabela de
//! juntas e os ângulos das poses ao lado dos afins). ⚠️ A primeira redacção desta wave implementou
//! a linear, que é a lei ANTIGA, e o gate de paridade leu `2,315e-3 m` — *a dívida estava escrita
//! no `crate::skin_gpu_tests` e eu não a li antes de desenhar*.
//!
//! ⚠️ **E quem pergunta *«onde está a arte?»* continua a ser servido**: o
//! [`ph2d_render::SpriteMesh::posado`] posa na CPU, com a MESMA tabela, **só quando alguém
//! pergunta**. *O custo não desapareceu: deixou de ser pago por quadro para ser pago por pergunta.*
//!
//! # ⭐⭐⭐ As CORRECÇÕES À MÃO entram na TABELA, e isso fecha uma dívida nomeada
//!
//! O cabeçalho do [`crate::skin_gpu`] avisa por escrito que a mancha do artista
//! ([`ph2d_skeleton_ecs::CorreccaoDePeso`]) *«vira um DEFEITO MUDO no dia em que a outra metade
//! shipar»* — a arte desenharia pela placa **sem** as correcções e pela CPU **com** elas.
//!
//! ⇒ a tabela que sobe é a do [`ph2d_skeleton::Skin::weights_corrected`], **com as manchas já
//! dentro**. Ela é derivada uma vez por construção de malha (que é uma vez por quadro hoje, e uma
//! vez por bind quando o memo do payload chegar), e não por vértice por quadro.
//!
//! # ⚠️ `K = 4` e a truncagem DECLARADA
//!
//! Um vértice leva os `K` ossos de maior peso, renormalizados. ⚠️ **A CPU das costuras lê a MESMA
//! tabela truncada** ([`ph2d_render::SpriteMeshSkin::posa`]) ⇒ não há dois mapas. ⛔ *Desenhar por
//! um e apontar por outro é o defeito que esta fila já pagou.*

use ph2d_poly2d::Mesh2d;
use ph2d_render::{OSSOS_POR_VERTICE, SpriteMesh, SpriteMeshSkin};
use ph2d_skeleton::{Correccao, Skin, Xform};

/// ⭐⭐ **A PORTA QUE DECIDE QUEM POSA** — a placa por omissão, `PH2D_SKIN_GPU=0` para bissecar.
///
/// ⚠️ **Ela nasce ABERTA e a razão é medida**, não uma preferência: o caminho da CPU é o TECTO do
/// produto (ver o cabeçalho), e a lei da casa — *tudo o que é novo shipa desligado* — vale enquanto
/// o que existe funciona. ⛔ Aqui não funciona: a `16` imagens presas o quadro acaba.
///
/// ⚠️ **O caminho da CPU FICA VIVO**, e não por cortesia: ele é a **referência** contra a qual a
/// paridade se mede, e é o que responde a um report do dono sem uma recompilação.
#[must_use]
pub fn a_placa_posa() -> bool {
    !matches!(
        std::env::var("PH2D_SKIN_GPU").as_deref(),
        Ok("0") | Ok("false")
    )
}

/// ⭐⭐⭐ **A MALHA DE REPOUSO MAIS A TABELA QUE A PLACA LÊ** — o irmão do
/// [`crate::skin_image::posed_sprite_mesh_corrigida`], com a MESMA assinatura.
///
/// ⚠️ **As posições saem em REPOUSO** (`p2l.apply(q)`, sem a pele) e a UV é a MESMA de sempre — ela
/// já era a do ponto de repouso, porque a tinta está pintada na forma de repouso.
///
/// `None` quando a UV não fecha (um `size` com lado nulo), exactamente como o irmão.
#[must_use]
pub fn sprite_mesh_para_a_placa(
    mesh: Mesh2d,
    p2l: Xform,
    pele: &Skin,
    pesos: &[f64],
    anchor: [f32; 2],
    size: [f32; 2],
    correcoes: &[Correccao],
) -> Option<SpriteMesh> {
    let ossos_da_tabela = if mesh.rest.is_empty() {
        0
    } else {
        pesos.len() / mesh.rest.len()
    };
    let mut scratch = pele.scratch();
    let mut w_saem = Vec::with_capacity(mesh.rest.len());
    let mut b_saem = Vec::with_capacity(mesh.rest.len());
    let repouso: Vec<[f32; 2]> = mesh
        .rest
        .iter()
        .enumerate()
        .map(|(v, &q)| {
            let p = p2l.apply(q);
            let fatia = pesos.get(v * ossos_da_tabela..(v + 1) * ossos_da_tabela);
            // ⭐⭐⭐ **A MESMA porta do produto** ([`Skin::weights_corrected`]) — a lei euclidiana ou
            // o padrão-ouro, **com as manchas do artista já dentro**. ⛔ Derivar o peso aqui com
            // outra conta seria a segunda resposta a *«quanto este osso manda neste ponto»*.
            pele.weights_corrected(
                p,
                fatia.filter(|_| ossos_da_tabela != 0),
                &mut scratch,
                correcoes,
            );
            let (w, b) = maiores_k(&scratch);
            w_saem.push(w);
            b_saem.push(b);
            f32_de(p)
        })
        .collect();
    let uv = mesh
        .rest
        .iter()
        .map(|&q| SpriteMesh::uv_at(f32_de(p2l.apply(q)), anchor, size))
        .collect::<Option<Vec<[f32; 2]>>>()?;
    // ⚠️ **Os afins são os das POSES dos ossos, em LOCAL→LOCAL** — a conjugação para o espaço do
    // quad é do DESENHO (`ph2d_render::sprite_mesh::conjuga_para_o_quad`), porque o mesmo bind
    // serve nove instâncias num 9-slice e cada uma tem o seu quad.
    let afins: Vec<[f32; 6]> = pele
        .bones()
        .iter()
        .map(|b| std::array::from_fn(|k| b.pose.0[k] as f32))
        .collect();
    // ⭐⭐⭐ **As duas tabelas da lei NÃO-linear saem da crate que a implementa** — a das juntas dos
    // eixos de REPOUSO e a dos ângulos da pose. ⛔ Derivá-las aqui seria a segunda resposta a
    // *«por onde estes dois ossos se encontram»*, e o `junta` é `pub(crate)` lá de propósito.
    let juntas: Vec<[f32; 2]> = pele.tabela_de_juntas().into_iter().map(f32_de).collect();
    let angulos: Vec<[f32; 2]> = pele.angulos_das_poses().into_iter().map(f32_de).collect();
    Some(SpriteMesh {
        local: repouso,
        uv,
        tris: mesh.tris,
        skin: Some(SpriteMeshSkin {
            pesos: w_saem,
            ossos: b_saem,
            afins,
            juntas,
            angulos,
        }),
    })
}

/// ⭐⭐ **OS `K` OSSOS DE MAIOR PESO, RENORMALIZADOS** — a truncagem do formato.
///
/// ⚠️ **Renormalizar é obrigatório e não cosmético:** o [`ph2d_skeleton::Skin::blend`] supõe
/// `Σŵ = 1`, e uma soma menor **encolhe o ponto para a origem** em vez de o deixar onde está.
///
/// ⚠️ **Soma zero devolve `SEM_PELE`**, que é o que diz ao shader *«não poses este vértice»* — a
/// mesma leitura honesta de *«nenhum osso manda aqui»* que o `blend` já tinha.
fn maiores_k(w: &[f64]) -> ([f32; OSSOS_POR_VERTICE], [u32; OSSOS_POR_VERTICE]) {
    let mut melhores: [(f64, usize); OSSOS_POR_VERTICE] = [(0.0, 0); OSSOS_POR_VERTICE];
    for (i, &peso) in w.iter().enumerate() {
        if peso <= 0.0 {
            continue;
        }
        // Insere por ordem decrescente, empurrando o mais fraco para fora.
        if let Some(pos) = melhores.iter().position(|(p, _)| peso > *p) {
            melhores[pos..].rotate_right(1);
            melhores[pos] = (peso, i);
        }
    }
    let soma: f64 = melhores.iter().map(|(p, _)| p).sum();
    if soma <= 0.0 {
        return (
            [0.0; OSSOS_POR_VERTICE],
            [ph2d_render::SEM_PELE; OSSOS_POR_VERTICE],
        );
    }
    (
        std::array::from_fn(|k| (melhores[k].0 / soma) as f32),
        std::array::from_fn(|k| u32::try_from(melhores[k].1).unwrap_or(0)),
    )
}

fn f32_de(q: [f64; 2]) -> [f32; 2] {
    [q[0] as f32, q[1] as f32]
}

#[cfg(test)]
#[path = "skin_image_gpu_tests.rs"]
mod skin_image_gpu_tests;
