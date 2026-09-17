//! **A CENA DO PINCEL AFIADO** (`=48`) — o vinco duro, em vez do domo.
//!
//! # ⚠️ Ela abre numa bola LISA, e a escolha é a lição da cena
//!
//! Ao contrário do pincel de plano — que precisa de relevo para ter o que aparar
//! —, este **faz** o relevo: o que ele entrega é uma linha cavada, estreita e de
//! fundo agudo. Sobre um campo de bossas o vinco desapareceria no meio delas.
//!
//! ⚠️⚠️ **E a peça é DENSA de propósito, com o número ao lado:** o vinco mede
//! `≈ 0,55` do raio do pincel na largura a meia profundidade (espec §7.2), e com
//! menos de umas três arestas nessa largura **o fundo não é representável** — o
//! artista veria uma linha de degraus e concluiria que a ferramenta é grossa.
//! *A cena que ensina o contrário do que acontece é pior que uma cena ausente.*

/// `=48` — a cena do **PINCEL AFIADO**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `47`).
pub(crate) fn afiado_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("48")
}

/// **Quantos triângulos a peça desta cena tem.**
///
/// ⚠️ **O recurso que este número nomeia é a LARGURA DO VINCO**, não o relógio:
/// a `40 000` triângulos numa bola de raio `1` a aresta mede `≈ 0,014`, e o
/// vinco de um pincel de raio `0,2` tem `0,11` de largura — cerca de **oito**
/// arestas, bem acima das três em que o fundo deixa de existir.
pub(crate) const TRIANGULOS_DA_PECA: usize = 40_000;

/// A peça com que a `=48` abre: uma bola lisa.
pub(crate) fn peca() -> ph2d_mesh::Mesh {
    ph2d_mesh::shapes::sphere_with_triangles(TRIANGULOS_DA_PECA, 1.0)
}

/// O roteiro da `=48`.
pub(crate) fn announce() {
    if !afiado_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =48 DRAW SHARP -- o vinco duro, no lugar do domo\n\
         [sculpt3d]    Este pincel risca uma linha CAVADA, estreita e de fundo agudo. O\n\
         [sculpt3d]    `Draw` de sempre levanta um monte largo; este cava, e nao alarga\n\
         [sculpt3d]    enquanto aprofunda. A bola abre LISA porque e' ele que faz o relevo.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Abra o painel com a CRASE (`). Na fileira de ferramentas, no FIM,\n\
         [sculpt3d]        ha' um botao `Draw Sharp`. Carregue nele.\n\
         [sculpt3d]        -> Ele ja' nasce a CAVAR: nao ha' nada a ligar.\n\
         [sculpt3d]    (2) Arraste uma linha por cima da bola, de um lado ao outro.\n\
         [sculpt3d]        -> Fica um VINCO fino. Compare com o passo (6).\n\
         [sculpt3d]    (3) Sem largar o botao, va' e volte pela MESMA linha, cinco ou seis\n\
         [sculpt3d]        vezes.\n\
         [sculpt3d]        -> Ele aprofunda cada vez menos e PARA: o vinco nunca passa de um\n\
         [sculpt3d]           raio do pincel de fundura. E repare que ele nao ALARGA.\n\
         [sculpt3d]    (4) Largue o botao e risque a MESMA linha outra vez, do inicio.\n\
         [sculpt3d]        -> Agora ele aprofunda mais, e o vinco fica mais ESTREITO a cada\n\
         [sculpt3d]           passagem nova. Cada vez que a caneta desce, o limite recomeca.\n\
         [sculpt3d]    (5) Segure o CTRL e arraste noutro sitio.\n\
         [sculpt3d]        -> Ele levanta uma CRISTA afiada, que e' o mesmo vinco ao contrario.\n\
         [sculpt3d]    (6) Carregue em `Draw` (o primeiro da fileira) e arraste do mesmo jeito.\n\
         [sculpt3d]        -> Um monte LARGO e mole: mais do dobro da largura, e levanta em vez\n\
         [sculpt3d]           de cavar. E' essa a diferenca entre os dois.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: a linha sair larga e mole como a do `Draw`; se o vinco\n\
         [sculpt3d]    ALARGAR quanto mais voce esfrega; se ele levantar em vez de cavar sem o\n\
         [sculpt3d]    Ctrl; ou se, sem largar o botao, ele cavar sem parar ate' furar a bola.\n"
    );
}
