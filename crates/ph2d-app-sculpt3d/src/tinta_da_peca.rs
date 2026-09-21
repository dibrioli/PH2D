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

/// O nível mais fino que o produto oferece — `lado = 2⁴ = 16` intervalos por
/// aresta, **`256` amostras por vértice** numa malha de quads.
///
/// ⭐⭐⭐ **Ele subiu de `3` para `4` em 2026-09-20, por ORDEM DO DONO**
/// (*«acrescente a opção de 16x»*) — e **depois de medido**, que é o que a
/// redacção anterior desta nota exigia por escrito: *«o nível `4` não é um
/// degrau a mais: é `4×` isso … um degrau novo aqui mede-se antes de se
/// escrever»*.
///
/// # A MEDIÇÃO (2026-09-20, `--release`, peça de fábrica: `98 306` vértices)
///
/// | `k` | lado | amostras | plano | construir | empacotar |
/// |---|---|---|---|---|---|
/// | `2` | `4` | `1,57 M` | `18 MB` | `32,9 ms` | `7,2 ms` |
/// | `3` | `8` | `6,29 M` | `72 MB` | `46,1 ms` | `21,9 ms` |
/// | **`4`** | **`16`** | **`25,2 M`** | **`288 MB`** | **`94,8 ms`** | **`81,7 ms`** |
///
/// ⛔⛔ **A coluna «empacotar» era paga POR QUADRO enquanto o traço durava, e é
/// ela que este degrau tornaria intransponível** — `81,7 ms` contra um quadro
/// de `16,7`. ⭐ Duas curas na mesma wave e as duas MEDIDAS: o empacotamento
/// deixou de existir (`bytemuck::cast_slice` sobre `[f32; 3]`, que já é o bloco
/// de bytes) e o upload passou a subir **só as amostras que o traço escreveu**
/// (`MeshRenderer::upload_tinta_amostras_at`) ⇒ o custo por quadro deixou de
/// ser `O(plano)` e passou a ser `O(pegada do dab)`.
///
/// ⚠️ **O que SOBRA e é do CHAMADOR:** os `288 MB` de VRAM por PEÇA e os
/// `94,8 ms` de construção **uma vez**, no gesto de carregar no chip. Na peça
/// da cena `=52` (`738` vértices) o mesmo degrau custa `2,2 MB` e `519 µs`.
/// ⛔ O tecto de cima é do DEVICE (`max_storage_buffer_binding_size`, que esta
/// casa sobe ao que o adaptador anuncia) e o nível `5` da
/// [`ph2d_mesh_colors::NIVEL_MAX`] fica fora por ele: `1,2 GB` por peça.
///
/// ⚠️ **E ele conta na fila do desfazer** — ver [`SceneObject::footprint_bytes`],
/// que soma o plano porque uma peça apagada entra inteira na fila.
///
/// [`SceneObject::footprint_bytes`]: crate::objects::SceneObject
pub(crate) const NIVEL_MAX: u8 = 4;

/// ⭐ **Quantas amostras o tecto custa POR VÉRTICE, numa malha de quads** — o
/// número que o parágrafo acima usa, e que o gate
/// `o_custo_do_tecto_por_vertice_e_o_que_a_constante_diz` MEDE.
///
/// A aritmética fecha à mão: numa malha de quads fechada há `~1` face e `~2`
/// arestas por vértice, logo `1 + 2(L−1) + (L−1)² = L²` — e a `L = 16` isso são
/// **`256`** amostras por vértice, `3 072` bytes de `f32` contra os `12` do
/// canal por vértice.
///
/// ⚠️ **Ele é `#[cfg(test)]` e isso é a resposta certa, não uma cerca:** o
/// produto não lê este número em sítio nenhum — quem paga a memória é o
/// alocador da [`Tinta`], e o que este valor faz é impedir que a tabela do doc
/// acima envelheça. *Pôr um consumidor artificial no produto para calar o
/// `dead_code` seria escrever código para o linter.*
#[cfg(test)]
pub(crate) const CUSTO_POR_VERTICE_NO_TECTO: usize = 256;

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

/// ⭐⭐⭐⭐ **ESTE GESTO VAI MEXER NA TOPOLOGIA?** — a porta de que a decisão do
/// passe, a triangulação do pen-down e a VOZ são os três consumidores.
///
/// ⛔⛔ **A ordem do dono de 2026-09-19 (*«permita que o dynamic topology
/// funcione para os 3 pincéis»*) trazia a razão escrita ao lado: *«a cor por
/// vértice é uma IMAGEM e a resolução dela É a da malha»*. ⭐ **Com o plano
/// armado essa premissa é FALSA** — a resolução da tinta passou a ser a do
/// plano —, e o que o refino compra ali é ZERO enquanto o que ele custa é o
/// plano inteiro: o endereço de uma amostra é `(face, sítio)`, logo uma face
/// partida deixa o plano a descrever outra malha e a [`garante`] reconstrói-o
/// **SEMEADO da cor por vértice**, que é literalmente a tinta a voltar à
/// resolução da malha.
///
/// ⚠️⚠️ **É o report do dono de 2026-09-20 à letra:** *«traços posteriores estão
/// reduzindo a resolução dos traços em alta resolução anteriores — como se
/// voltasse para o modo mesh»*. Com a topologia dinâmica armada, cada dab de
/// cor refinava, e cada refino deitava fora o trabalho de todos os traços
/// anteriores.
///
/// ⭐ **§0.0 aplicado a uma PREMISSA e não a um número:** *quem move o número
/// que tornava algo inalcançável tem de reconferir a nota* — aqui quem mudou
/// foi a premissa, e a nota é aquela ordem.
///
/// ⚠️ **Os verbos de FORMA continuam a refinar**, com o plano armado ou não:
/// ali a topologia muda porque o artista pediu forma nova, e o plano não tem
/// como sobreviver a uma malha que já não é a dele. É por isso que a voz
/// continua a existir — e a ler esta MESMA porta, senão ela avisaria de um
/// preço que já não se paga.
pub(crate) fn o_gesto_muda_a_topologia(
    verbo: ph2d_sculpt3d::Verb,
    tinta_fina_armada: bool,
) -> bool {
    if verbo.paints_color() && tinta_fina_armada {
        return false;
    }
    verbo.refina_no_dyntopo() || verbo.colapsa_no_dyntopo()
}

impl crate::Sculpt3dScene {
    /// ⭐⭐⭐⭐ **A TINTA FINA ESTÁ ARMADA NA PEÇA ACTIVA?** — contando o plano
    /// que o TRAÇO segura.
    ///
    /// ⛔⛔ **Ela existe por um defeito medido em 2026-09-20, e o defeito vivia
    /// ENTRE as duas metades que já tinham gate:** o pen-down **empresta** o
    /// plano à linha `318` do [`crate::input_down`] e a voz procura-o na peça
    /// na linha `381` — logo `o.tinta.is_some()` lia **`false`** e o aviso da
    /// tinta fina **nunca soava**. *Uma lei verificada nas duas pontas ainda
    /// pode ser contrariada no meio*, e o gate que a defendia era um censo de
    /// TEXTO (`a_voz_da_tinta_fina_e_armada_pela_porta`, que nasceu desta): ele prova
    /// que o campo
    /// é alimentado pela peça e **nunca corre o pen-down**.
    ///
    /// ⚠️ **Os dois sítios são a MESMA pergunta com o plano em mãos
    /// diferentes**, e é por isso que ela é uma porta: quem perguntar só a um
    /// deles acerta metade do tempo, e a metade em que erra é exactamente
    /// **durante o gesto**, que é quando a resposta é usada.
    pub(crate) fn tinta_fina_armada(&self) -> bool {
        self.stroke.tinta_fina.is_some()
            || self
                .objects
                .get(self.active)
                .is_some_and(|o| o.tinta.is_some())
    }

    /// ⭐⭐⭐ **O gesto que está em mãos vai mexer na topologia?** — a
    /// [`o_gesto_muda_a_topologia`] com as duas entradas que a cena tem.
    pub(crate) fn o_gesto_em_maos_muda_a_topologia(&self, verbo: ph2d_sculpt3d::Verb) -> bool {
        o_gesto_muda_a_topologia(verbo, self.tinta_fina_armada())
    }
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

/// ⭐⭐ **O censo da FIAÇÃO** — que os três consumidores desta porta a chamam.
///
/// ⛔ Ele é um irmão separado e não mais um `#[test]` ali dentro porque a
/// pergunta é outra: aquele ficheiro mede a **LEI** (pura, sem cena), e este
/// mede o **ELO** (textual, porque a cena pede um `wgpu::Device`). Quatro
/// mutações sobreviventes pagaram a diferença — ver o cabeçalho dele.
#[cfg(test)]
#[path = "tinta_fiacao_tests.rs"]
mod fiacao_tests;
