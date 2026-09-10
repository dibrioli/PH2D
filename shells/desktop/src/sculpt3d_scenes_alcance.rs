//! **A CENA DAS DUAS PEÇAS VIZINHAS** (`=39`) — o pincel toca só no que se
//! aponta?
//!
//! # ⛔⛔ Ela existe porque a PRIMEIRA tentativa de smoke foi recusada, e com
//! razão
//!
//! Eu mandei o dono julgar a máscara de alcance na cena da orelha (`=36`), e a
//! resposta foi *«do modo como o objecto é não é possível testar. O que estamos
//! a testar aqui?»* — e ele está certo:
//!
//! ⚠️ **Na orelha os dois lados do sulco são a MESMA superfície contínua.** O
//! pincel não salta para outro sítio; ele só arrasta um pouco mais de
//! vizinhança, e o efeito é o sulco ficar **mais ou menos fundo**. Isso é uma
//! diferença de GRAU, e ninguém a julga a olho sem ter as duas lado a lado.
//! *Uma cena em que a resposta certa e a errada se parecem é pior que cena
//! nenhuma.*
//!
//! # ⭐⭐⭐ E a geometria desta cena foi MEDIDA, não escolhida
//!
//! A sonda [`sonda_do_falloff_pela_superficie`](../../../crates/ph2d-sculpt3d/tests/sonda_do_falloff_pela_superficie.rs)
//! §5 varreu as duas famílias possíveis e imprimiu quanto a máscara corta em
//! cada uma:
//!
//! | peça | raio | vértices | cortado |
//! |---|---|---|---|
//! | duas PONTAS ligadas, `alt 1,0` | `0,35` | `91` | `6,9 %` ⛔ invisível |
//! | duas PONTAS ligadas, `alt 1,5` finas | `0,35` | **`39`** | `40,8 %` ⛔ *um alfinete, não um gesto* |
//! | duas PONTAS ligadas, `alt 2,0` muito finas | `0,35` | **`20`** | `64,2 %` ⛔ idem |
//! | **duas PEÇAS soltas, folga `0,05`** | **`0,35`** | **`186`** | **`47,1 %`** ⭐ |
//! | **idem** | **`0,50`** | **`362`** | **`47,9 %`** ⭐ |
//!
//! ⛔⛔ **As pontas LIGADAS não servem, e o motivo não é o alcance — é a
//! RESOLUÇÃO:** para o defeito aparecer nelas, elas têm de ser tão finas que o
//! carimbo toca `9` a `39` vértices. Num raio que um artista de facto usa o
//! corte cai para `6,9 %`, que não se vê. *A superfície liga as duas pontas, e
//! por isso ali o discriminador é o TECTO — e o tecto só morde quando a volta é
//! muito mais longa que o vão, o que exige uma agulha.*
//!
//! ⇒ a cena são **duas peças soltas na mesma malha**, onde a superfície não as
//! liga de forma alguma: o corte é `47 %` sobre centenas de vértices, e a
//! pergunta passa a ser binária — *a outra peça mexeu-se, sim ou não?*
//!
//! ⚠️ **E isto não é um caso inventado:** uma malha com duas partes soltas é o
//! que um `Import` de um modelo em duas peças entrega (a cena `=9` fabrica
//! exactamente esse ficheiro) e o que um `Extract` deixa.

use ph2d_mesh::{Face, Mesh};

/// `=39` — a cena das **DUAS PEÇAS VIZINHAS**.
///
/// ⚠️⚠️ **Ela nasceu `=38` e o gate `no_two_sculpt3d_scenes_claim_the_same_level`
/// apanhou-a:** o `=38` já é dos QUATRO VIEWPORTS. Eu conferi a disponibilidade
/// com um `grep` de UMA forma de declaração e a vizinha usa outra ⇒ *o número da
/// próxima cena CONTA-SE no roteador, e um filtro escrito à mão sobre uma forma
/// só não é o censo* (`CLAUDE.md` §5.0). A segunda cena a reclamar um número
/// fica **INALCANÇÁVEL**, muda, sem erro nenhum.
pub(crate) fn alcance_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("39")
}

/// A folga entre as duas peças, em unidades de mundo.
///
/// ⭐ **MEDIDA, e a vizinha é a prova:** a `0,05` a máscara corta `47,1 %` do
/// carimbo a raio `0,35`; a `0,15` ela corta `6,2 %` no mesmo raio — *três vezes
/// a folga e o defeito deixa de se ver*. A cena tem de conter o fenómeno.
pub(crate) const FOLGA: f32 = 0.05;

/// **Duas peças soltas na MESMA malha**, separadas por [`FOLGA`].
///
/// ⚠️ **Uma malha, dois componentes** — e é isso que faz a pergunta ser binária:
/// a superfície não liga as duas de forma alguma, logo a distância pela
/// superfície é `∞` e nenhum epsilon a explica. ⛔ Duas peças de CENA (como a
/// `=7` põe) não serviriam: o pincel age numa malha de cada vez, então o defeito
/// nem existiria ali.
pub(crate) fn duas_pecas_vizinhas() -> Mesh {
    let uma = ph2d_mesh::shapes::uv_sphere(48, 96, 1.0);
    let n = uma.vert_count() as u32;
    let dx = 2.0 + FOLGA;
    let mut pos: Vec<[f32; 3]> = Vec::with_capacity(uma.vert_count() * 2);
    for p in uma.positions() {
        pos.push([p[0] - dx * 0.5, p[1], p[2]]);
    }
    for p in uma.positions() {
        pos.push([p[0] + dx * 0.5, p[1], p[2]]);
    }
    let mut faces = uma.faces().to_vec();
    for f in uma.faces() {
        let v: Vec<u32> = f.verts().iter().map(|i| i + n).collect();
        faces.push(if v.len() == 3 {
            Face::tri(v[0], v[1], v[2])
        } else {
            Face::quad(v[0], v[1], v[2], v[3])
        });
    }
    Mesh::from_parts(pos, faces).expect("duas esferas soltas continuam uma malha valida")
}

/// O roteiro da `=38`.
pub(crate) fn announce() {
    if !alcance_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =39 AS DUAS PECAS VIZINHAS -- o pincel toca so' no que voce aponta?\n\
         [sculpt3d]    Na tela estao DUAS bolas quase a tocar-se. Elas sao a mesma peca de\n\
         [sculpt3d]    escultura (e' o que um modelo importado em duas partes da'), e por isso\n\
         [sculpt3d]    o pincel ve as duas de uma vez.\n\
         [sculpt3d]\n\
         [sculpt3d]    O QUE ESTAMOS A TESTAR: hoje o pincel mede distancia em linha reta,\n\
         [sculpt3d]    PELO AR. Entao ao pintar a bola da esquerda ele agarra tambem a da\n\
         [sculpt3d]    direita, que esta' perto no espaco e longe pela superficie. Voce aponta\n\
         [sculpt3d]    uma e mexem-se duas.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Gire a peca com o botao da DIREITA ate' ver as duas bolas de lado,\n\
         [sculpt3d]        com o vao entre elas no meio da tela.\n\
         [sculpt3d]    (2) Ponha o pincel GRANDE: abra o painel com a CRASE (`) e leve o\n\
         [sculpt3d]        `Radius` para perto do maximo.\n\
         [sculpt3d]    (3) Pinte UMA VEZ na bola da ESQUERDA, no lado dela que olha para o\n\
         [sculpt3d]        vao. Um clique so', sem arrastar.\n\
         [sculpt3d]    (4) OLHE A BOLA DA DIREITA.\n\
         [sculpt3d]        -> Ela tem de estar INTACTA. Se ela tambem inchou ou afundou, e' o\n\
         [sculpt3d]           defeito: quase METADE da forca do pincel caiu nela (medido: 47%).\n\
         [sculpt3d]    (5) Ctrl+Z e repita algumas vezes, em sitios diferentes do vao. O que\n\
         [sculpt3d]        importa e' se a bola que voce NAO apontou se mexe.\n\
         [sculpt3d]\n\
         [sculpt3d]    ⚠️ NESTA CORRIDA A CURA ESTA' DESLIGADA -- entao a bola da direita VAI\n\
         [sculpt3d]       mexer-se. E' o defeito de hoje, e e' o lado A da comparacao.\n\
         [sculpt3d]       Para ver o lado B, feche o app e rode outra vez com a cura ligada:\n\
         [sculpt3d]\n\
         [sculpt3d]         PH2D_SCULPT3D_SMOKE=39 PH2D_SCULPT_ALCANCE=1\n\
         [sculpt3d]\n\
         [sculpt3d]       Ali a bola da direita tem de ficar QUIETA, e a da esquerda tem de\n\
         [sculpt3d]       receber o mesmo relevo de sempre.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: com a cura ligada a bola da ESQUERDA passar a receber\n\
         [sculpt3d]    menos relevo, ou aparecer um degrau no lado dela virado ao vao. A cura\n\
         [sculpt3d]    e' para tirar o que era do vizinho, nunca para tirar o que e' seu."
    );
}

#[cfg(test)]
#[path = "sculpt3d_scenes_alcance_tests.rs"]
mod tests;
