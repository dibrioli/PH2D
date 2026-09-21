//! ⭐⭐⭐⭐ **QUEM É DONO DO PLANO DE TINTA FINA** — a peça, e só ela.
//!
//! A [`ph2d_mesh_colors::Tinta`] é o plano de amostras que faz a tinta deixar
//! de ter a resolução da MALHA (o §5: *«a cor mora nos vértices, logo a
//! resolução da tinta é a da malha»*). Este módulo responde às três perguntas
//! que ninguém mais pode responder:
//!
//! 1. **quando o plano NASCE** — e com que cor (⚠️ nunca branco: ver
//!    [`garante`]);
//! 2. **quem o SEGURA durante um traço** — ele é EMPRESTADO ao
//!    [`ph2d_sculpt3d::SculptStroke`] e volta no fim ([`empresta`] ·
//!    [`devolve`]), que é a forma que o cabeçalho da
//!    [`ph2d_sculpt3d::tinta_fina::TintaDoTraco`] escolheu para não mexer na
//!    assinatura do `dab`;
//! 3. **quando ele MORRE** — quando a topologia debaixo dele muda.
//!
//! ⛔⛔ **A terceira é a que tem modo de falha MUDO.** O plano é *paramétrico
//! nas FACES*: o endereço de uma amostra é `(face, sítio)`, logo ele sobrevive
//! a qualquer pincel que só mova vértices e **não sobrevive** a um que parta ou
//! funda uma face. Um plano da topologia anterior lido sobre a malha de agora
//! não estoura nem desenha lixo óbvio — ele põe **a tinta de uma face na
//! face vizinha**, que é exactamente o defeito que ninguém consegue atribuir.
//! ⇒ [`concorda_com`] é lida em TODO quadro, e a discordância **reconstrói o
//! plano, SEMEADO da cor por vértice** — que é a única resposta que a malha
//! sabe dar, e é honesta porque a [`devolve`] mantém aquele canal em dia: o que
//! sobrevive a um remesh é a cor grossa, e o detalhe fino não tem onde caber
//! numa topologia que já não é a dele.
//!
//! ⚠️⚠️ **E é por isso que o pen-down FALA** (a família do
//! [`crate::recusa`]): um verbo que muda topologia com o plano armado vai
//! custar o detalhe fino, e *apagar trabalho em silêncio é o pior desfecho
//! desta fronteira*. ⛔ Ele fala e **não bloqueia** — a decisão de suprimir o
//! passe de topologia enquanto a tinta fina está armada é de PRODUTO, e está
//! nomeada como aberta.

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;

/// O nível mais fino que o produto oferece — `lado = 2³ = 8` intervalos por
/// aresta, `49` amostras no interior de cada quad, **`64` por vértice** numa
/// malha de quads.
///
/// ⚠️⚠️ **O recurso é a MEMÓRIA do device, e o número é MEDIDO e não estimado**
/// — a 1.ª redacção desta linha dizia *«~3,1 M amostras»* para a peça de
/// fábrica e estava errada por **2×**, porque eu contei os interiores e esqueci
/// que as arestas de um quad também levam `L−1` amostras cada. Ver
/// [`CUSTO_POR_VERTICE_NO_TECTO`], o multiplicador que o gate
/// `o_custo_do_tecto_por_vertice_e_o_que_a_constante_diz` mede num toro de
/// quads.
///
/// **O que ele custa, na peça de fábrica do módulo** (`98 306` vértices, quads):
/// `98 306 × 64 = 6,29 M` amostras, **`75,5 MB`** de `f32` — contra os `1,2 MB`
/// do canal por vértice.
///
/// ⛔ **O nível `4` NÃO é um degrau a mais: é `4×` isso, `302 MB` por PEÇA**, e
/// é aí que o limite deixa de ser conforto de implementação e passa a ser a
/// placa. *Um degrau novo aqui mede-se antes de se escrever.*
pub(crate) const NIVEL_MAX: u8 = 3;

/// ⭐ **Quantas amostras o tecto custa POR VÉRTICE, numa malha de quads** — o
/// número que o parágrafo acima usa, e que o gate
/// `o_custo_do_tecto_por_vertice_e_o_que_a_constante_diz` MEDE.
///
/// A aritmética fecha à mão: numa malha de quads fechada há `~1` face e `~2`
/// arestas por vértice, logo `1 + 2(L−1) + (L−1)² = L²` — e a `L = 8` isso são
/// **`64`** amostras por vértice, `768` bytes de `f32` contra os `12` do canal
/// por vértice.
///
/// ⚠️ **Ele é `#[cfg(test)]` e isso é a resposta certa, não uma cerca:** o
/// produto não lê este número em sítio nenhum — quem paga a memória é o
/// alocador da [`Tinta`], e o que este valor faz é impedir que a tabela do doc
/// acima envelheça. *Pôr um consumidor artificial no produto para calar o
/// `dead_code` seria escrever código para o linter.*
#[cfg(test)]
pub(crate) const CUSTO_POR_VERTICE_NO_TECTO: usize = 64;

/// **O plano ainda descreve esta malha?** — a pergunta da armadilha muda do
/// cabeçalho deste módulo.
///
/// ⚠️ **Ela compara vértices E faces, e as duas metades são precisas:** um
/// colapso seguido de um refino pode devolver a MESMA contagem de vértices com
/// outras faces, e um refino que só parte quads deixa a contagem de faces a
/// subir com a de vértices parada. *Uma régua que conta uma grandeza só aprova
/// a metade das mudanças de topologia.*
pub(crate) fn concorda_com(t: &Tinta, mesh: &Mesh) -> bool {
    t.topologia().verts() == mesh.vert_count() && t.topologia().faces() == mesh.faces().len()
}

/// ⭐⭐ **Garante que a peça tem o plano que o artista pediu** — e devolve
/// `true` quando alguma coisa mudou (⇒ o device tem de o receber outra vez).
///
/// `nivel = None` desarma: o plano é **largado** e a cor volta a ser a do
/// canal por vértice, que continua a ser escrito pelo mesmo traço.
///
/// ⛔⛔ **O plano NOVO nasce SEMEADO da cor por vértice e nunca branco**, e
/// este é o mesmo defeito que a [`Tinta::semeada`] existe para impedir um nível
/// abaixo: uma peça pintada que ganha detalhe fino veria a tinta **desaparecer
/// no instante em que o artista pede mais resolução para ela**. A semente é
/// exacta nos vértices e interpolada no resto, que é a única resposta que a
/// malha sabe dar.
pub(crate) fn garante(mesh: &Mesh, tinta: &mut Option<Tinta>, nivel: Option<u8>) -> bool {
    let Some(k) = nivel else {
        return tinta.take().is_some();
    };
    let k = k.min(NIVEL_MAX);
    if let Some(t) = tinta.as_ref()
        && t.nivel() == k
        && concorda_com(t, mesh)
    {
        return false;
    }
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    *tinta = Some(match mesh.colors() {
        Some(c) => Tinta::semeada(c, faces(), k),
        None => Tinta::nova(mesh.vert_count(), faces(), k),
    });
    true
}

/// ⭐⭐⭐ **DE ONDE O QUADRO LÊ O PLANO DESTA PEÇA, e se o reconcilia.**
///
/// ⚠️⚠️ **Ela é uma função PURA e isso é a decisão**, não arrumação: a escolha
/// vive no laço de upload, que pede um `wgpu::Device` — um gate ali nasceria
/// `#[ignore]` e **o CI nunca o correria**. *Quando um gate precisa de um
/// device para medir uma decisão que não tem pixel nenhum, a lei está no sítio
/// errado* (a frase que a §33 desta linha pagou).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Rota {
    /// **O TRAÇO segura o plano.** Não se reconcilia nada (o `Option` da peça
    /// está vazio, e reconciliar construiria um plano BRANCO por quadro que o
    /// `close_stroke` depois sobrescreveria), e o upload lê o GESTO — ler a
    /// peça subiria `armado = 0`, e *o artista veria a tinta fina desaparecer
    /// no instante em que começasse a pintar*.
    Emprestado,
    /// **A PEÇA segura o plano.** Reconcilia-se com `pedir` e o upload lê-a.
    DaPeca {
        /// O que pedir à [`garante`] — `None` desarma.
        pedir: Option<u8>,
    },
}

/// A lei da [`Rota`], em quatro perguntas.
///
/// ⚠️ **Só a peça ACTIVA ganha um plano NOVO, e as outras só mantêm o que já
/// têm:** um plano de nível `3` custa `64` amostras por vértice, e armar o knob
/// não pode multiplicar isso por toda a cena. *O plano nasce na peça que o
/// artista tem na mão e vive enquanto ela viver.*
pub(crate) fn rota(
    e_a_activa: bool,
    o_traco_segura: bool,
    a_peca_tem: bool,
    nivel: Option<u8>,
) -> Rota {
    if e_a_activa && o_traco_segura {
        return Rota::Emprestado;
    }
    Rota::DaPeca {
        pedir: if e_a_activa || a_peca_tem {
            nivel
        } else {
            None
        },
    }
}

/// ⭐ **Empresta o plano ao traço** — o `Option` sai da peça e entra no gesto.
///
/// ⚠️ **É um `take` e não um clone, de propósito:** um plano de nível `3` numa
/// peça de cem mil vértices é dezenas de MB, e clonar por pen-down poria o
/// custo de uma pincelada em função do detalhe da tinta. O preço é a peça ficar
/// sem plano enquanto o traço dura — e é por isso que [`devolve`] não é
/// opcional e o `close_stroke` a chama sempre, inclusive no caminho de recusa.
pub(crate) fn empresta(
    tinta: &mut Option<Tinta>,
) -> Option<ph2d_sculpt3d::tinta_fina::TintaDoTraco> {
    tinta
        .take()
        .map(ph2d_sculpt3d::tinta_fina::TintaDoTraco::nova)
}

/// ⭐⭐ **Devolve o plano à peça, e REESCREVE o canal por vértice com ele.**
///
/// ⚠️⚠️ **A segunda metade não é acabamento — é o que mantém UMA cor.** Tudo
/// o que não lê o plano (o assado, a doação de forma, o caminho sem plano
/// armado, o `.ph2dproj`) lê `Mesh::colors`, e sem esta linha o artista pintava
/// com detalhe fino e via a peça voltar à tinta de antes assim que desarmasse.
/// O plano é a resolução ALTA da mesma cor, logo a de baixa deriva-se dele —
/// nunca ao contrário.
pub(crate) fn devolve(
    mesh: &mut Mesh,
    tinta: &mut Option<Tinta>,
    do_traco: Option<ph2d_sculpt3d::tinta_fina::TintaDoTraco>,
) {
    let Some(t) = do_traco.map(ph2d_sculpt3d::tinta_fina::TintaDoTraco::entregar) else {
        return;
    };
    if concorda_com(&t, mesh) {
        let por_vertice = t.plano_por_vertice().to_vec();
        mesh.colors_mut().copy_from_slice(&por_vertice);
    }
    *tinta = Some(t);
}

#[cfg(test)]
#[path = "tinta_da_peca_tests.rs"]
mod tests;
