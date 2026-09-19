//! **A CENA DA PAREDE FINA** (`=50`) — esculpir uma barbatana sem que o outro
//! lado dela venha junto.
//!
//! # ⚠️⚠️ A peça é FINA de propósito, e a espessura é a única coisa que decide
//!
//! A lei que esta cena mostra é a RAZÃO `superfície / ar`
//! ([`ph2d_sculpt3d::RAZAO_MAXIMA`]), e numa chapa ela escreve-se
//! `(2d + t)/t > 3,5` ⇒ **`d > 1,25 × t`**. ⭐ Ela **não depende do raio do
//! pincel**: quem decide é a espessura. ⇒ uma peça grossa não tem o fenómeno, e
//! uma cena de esfera mostraria a cura a não fazer nada.
//!
//! ⚠️ **A barbatana tem `0,06` de espessura contra `2,0` de lado** — a mesma
//! proporção da fixtura que mediu o defeito (`docs/3D/geodesica/`), e a mesma em
//! que o carimbo atravessava a parede com `98,8 %` da força.
//!
//! # ⭐ E o roteiro começa pelo lado ERRADO da peça
//!
//! O defeito é invisível de onde o artista carimba: a face da frente ganha a
//! bossa que ele pediu, e é a de TRÁS que se mexe. ⇒ o passo (2) manda-o **rodar
//! a peça e olhar as costas ANTES de tocar nela** — sem isso ele não tem com que
//! comparar, e a cena ensinaria que não há nada para ver.

use ph2d_mesh::{Face, Mesh};

/// Meio lado da chapa.
pub(crate) const MEIO: f32 = 1.0;
/// A espessura — `3 %` do lado, que é o regime em que o defeito vive.
pub(crate) const ESPESSURA: f32 = 0.06;
/// Vértices por lado, em cada face.
const N: usize = 81;

/// `=50` — a cena da **PAREDE FINA**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `49`).
pub(crate) fn parede_fina_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("50")
}

fn f(i: usize, j: usize) -> u32 {
    (i * N + j) as u32
}
fn t(i: usize, j: usize) -> u32 {
    (N * N + i * N + j) as u32
}

/// **A BARBATANA** — duas grelhas `N × N` a `±t/2`, costuradas por uma cinta de
/// um quad de altura na beira.
///
/// ⚠️ **A cinta reutiliza os vértices das duas faces** (não há anel extra): é
/// isso que faz o caminho pela superfície medir exactamente *ir à beira, subir a
/// espessura, e voltar* — que é a grandeza de que a lei vive. Um anel a mais ali
/// alongaria o caminho e a cena passaria a medir a malha em vez da peça.
pub(crate) fn barbatana() -> Mesh {
    let passo = 2.0 * MEIO / (N - 1) as f32;
    let meia = ESPESSURA * 0.5;
    let mut pos = Vec::with_capacity(2 * N * N);
    for z in [meia, -meia] {
        for i in 0..N {
            for j in 0..N {
                pos.push([-MEIO + passo * i as f32, -MEIO + passo * j as f32, z]);
            }
        }
    }
    let mut faces = Vec::with_capacity(2 * (N - 1) * (N - 1) + 4 * (N - 1));
    for i in 0..N - 1 {
        for j in 0..N - 1 {
            faces.push(Face::quad(
                f(i, j),
                f(i + 1, j),
                f(i + 1, j + 1),
                f(i, j + 1),
            ));
            faces.push(Face::quad(
                t(i, j),
                t(i, j + 1),
                t(i + 1, j + 1),
                t(i + 1, j),
            ));
        }
    }
    for k in 0..N - 1 {
        faces.push(Face::quad(f(k + 1, 0), f(k, 0), t(k, 0), t(k + 1, 0)));
        faces.push(Face::quad(
            f(k, N - 1),
            f(k + 1, N - 1),
            t(k + 1, N - 1),
            t(k, N - 1),
        ));
        faces.push(Face::quad(f(0, k), f(0, k + 1), t(0, k + 1), t(0, k)));
        faces.push(Face::quad(
            f(N - 1, k + 1),
            f(N - 1, k),
            t(N - 1, k),
            t(N - 1, k + 1),
        ));
    }
    Mesh::from_parts(pos, faces).expect("a barbatana é construída aqui e é válida")
}

/// O roteiro da `=50`.
pub(crate) fn announce() {
    if !parede_fina_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =50 A PAREDE FINA -- esculpir um lado sem levar o outro junto\n\
         [sculpt3d]    Esta peca e' uma BARBATANA: 2,0 de lado e so' 0,06 de espessura.\n\
         [sculpt3d]    E' uma orelha, uma folha, uma nadadeira -- o tipo de peca em que\n\
         [sculpt3d]    o pincel atravessava a parede.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Escolha `Draw` (tecla 1) e ponha o pincel GRANDE: `]` umas vezes,\n\
         [sculpt3d]        ate' ele ficar claramente mais largo do que a peca e' grossa.\n\
         [sculpt3d]    (2) ANTES DE TOCAR: rode a peca (arrastar com o botao do meio) e\n\
         [sculpt3d]        olhe as COSTAS dela. Decore como ela esta' -- lisa.\n\
         [sculpt3d]        -> Sem este passo nao ha' com que comparar.\n\
         [sculpt3d]    (3) Volte a' frente e faca DUAS OU TRES BOSSAS fortes ONDE QUISER --\n\
         [sculpt3d]        no meio, perto da borda, ou mesmo EM CIMA dela.\n\
         [sculpt3d]        -> Antes de 19/09 o pincel grande atravessava em TODA a peca.\n\
         [sculpt3d]    (4) Rode outra vez e olhe as COSTAS.\n\
         [sculpt3d]        -> Elas tem de estar LISAS onde voce carimbou. Antes desta wave\n\
         [sculpt3d]           aparecia ali a mesma bossa, com 99% da forca -- a peca\n\
         [sculpt3d]           DESLIZAVA em vez de ganhar relevo.\n\
         [sculpt3d]\n\
         [sculpt3d]    (5) O CONTROLO, e e' ele que prova que a cura esta' ligada: no painel\n\
         [sculpt3d]        (tecla CRASE `), DESMARQUE `Connected Only`. Repita o passo (3)\n\
         [sculpt3d]        noutro sitio e olhe as costas.\n\
         [sculpt3d]        -> AGORA a bossa aparece dos dois lados. Volte a marcar a caixa.\n\
         [sculpt3d]\n\
         [sculpt3d]    (6) E AGORA O TAMANHO: aperte `]` mais umas vezes ate' o circulo do\n\
         [sculpt3d]        pincel ficar do tamanho de meia peca, e repita (3) e (4).\n\
         [sculpt3d]        -> As costas continuam LISAS. E' este o passo que mudou em 19/09:\n\
         [sculpt3d]           ate' entao a correcao so' valia com o pincel pequeno.\n\
         [sculpt3d]\n\
         [sculpt3d]    COMO SABER QUE DEU ERRADO: se no passo (4) ou no (6) as costas\n\
         [sculpt3d]    tiverem bossa, a mascara nao esta' a correr -- confirme que\n\
         [sculpt3d]    `Connected Only` esta' MARCADO no painel."
    );
}

#[cfg(test)]
#[path = "scenes_parede_fina_tests.rs"]
mod tests;
