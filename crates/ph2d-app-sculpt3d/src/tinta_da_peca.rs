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

/// O nível mais fino que o produto oferece — `lado = 2⁸ = 256` intervalos por
/// aresta. ⛔⛔ **Mas o TECTO de uma peça NÃO é ele: é a PLACA**
/// ([`orcamento_da_placa`]), e o degrau pedido DESCE ao maior que cabe
/// ([`garante_no_orcamento`]).
///
/// ⭐⭐⭐ **Subiu de `4` para `8` em 2026-09-24, por report do dono** (*«a
/// resolução de 16x não chega para o painter»*) — e o report estava certo, com
/// número. O Painter pinta à resolução do ECRÃ, logo a pergunta é *quantos
/// píxeis cabem entre duas amostras* (sonda
/// `diag_pixeis_por_amostra_da_tinta_fina`, peça da cena `=52`, arestas de
/// frente, na vista `1900×1000`):
///
/// | `k` | lado | px por amostra p50 · p90 · máx | plano aqui | na peça de fábrica |
/// |---|---|---|---|---|
/// | `4` | `16` | `1,89` · `2,52` · `3,58` | `2,3 MB` | `302 MB` |
/// | `5` | `32` | `0,94` · `1,26` · `1,79` | `9,1 MB` | `1,2 GB` |
/// | `6` | `64` | `0,47` · `0,63` · `0,90` | `36 MB` | ⛔ `4,8 GB` |
/// | `7` | `128` | `0,24` · `0,31` · `0,45` | `145 MB` | ⛔ `19 GB` |
/// | `8` | `256` | `0,12` · `0,16` · `0,22` | `580 MB` | ⛔ `77 GB` |
///
/// ⛔⛔ **O tecto de ontem (`16x`) tinha sido medido na peça ERRADA:** os
/// `288 MB` eram da peça de fábrica, e na peça em que o artista pinta com
/// tinta fina (grossa, porque a resolução é da tinta) o mesmo degrau custa
/// `2,3 MB`. *Um limite medido no caso mais caro e aplicado a todos é o caminho
/// lento a definir o rápido* (§0.0). O recurso é por PEÇA, e quem o diz é a
/// placa: `4 GiB` por buffer nesta máquina.
///
/// ⚠️ **O que SOBRA e é do CHAMADOR:** a construção (`~3,8 ns` por amostra ⇒
/// `~180 ms` a `256x` na peça da lição) corre **uma vez**, no gesto de
/// carregar no chip; o custo POR QUADRO é `O(pegada do dab)` desde 2026-09-20
/// (`MeshRenderer::upload_tinta_amostras_at`).
///
/// ⚠️ **E ele conta na fila do desfazer** — ver [`SceneObject::footprint_bytes`],
/// que soma o plano porque uma peça apagada entra inteira na fila.
///
/// [`SceneObject::footprint_bytes`]: crate::objects::SceneObject
pub(crate) const NIVEL_MAX: u8 = 8;

/// ⭐ **Quantas amostras o tecto custa POR VÉRTICE, numa malha de quads** — o
/// número que o parágrafo acima usa, e que o gate
/// `o_custo_do_tecto_por_vertice_e_o_que_a_constante_diz` MEDE.
///
/// A aritmética fecha à mão: numa malha de quads fechada há `~1` face e `~2`
/// arestas por vértice, logo `1 + 2(L−1) + (L−1)² = L²` — e a `L = 256` isso
/// são **`65 536`** amostras por vértice, `768 KB` de `f32` contra os `12` bytes
/// do canal por vértice.
///
/// ⚠️ **Ele é `#[cfg(test)]` e isso é a resposta certa, não uma cerca:** o
/// produto não lê este número em sítio nenhum — quem paga a memória é o
/// alocador da [`Tinta`], e o que este valor faz é impedir que a tabela do doc
/// acima envelheça. *Pôr um consumidor artificial no produto para calar o
/// `dead_code` seria escrever código para o linter.*
#[cfg(test)]
pub(crate) const CUSTO_POR_VERTICE_NO_TECTO: usize = 65_536;

/// **O plano ainda descreve esta malha?** — a pergunta da armadilha muda do
/// cabeçalho deste módulo.
///
/// ⚠️ **Ela compara vértices E faces, e as duas metades são precisas:** um
/// colapso seguido de um refino pode devolver a MESMA contagem de vértices com
/// outras faces, e um refino que só parte quads deixa a contagem de faces a
/// subir com a de vértices parada. *Uma régua que conta uma grandeza só aprova
/// a metade das mudanças de topologia.*
///
/// ⭐⭐⭐⭐ **E A CONTA NÃO MORA AQUI DESDE 2026-09-21: ela é a
/// [`ph2d_mesh_colors::Topologia::descreve`].** O pânico do dono provou que a
/// pergunta tem um SEGUNDO leitor que esta crate não alcança — a porta do
/// device, em `ph2d-mesh-render` —, e *uma lei escrita em dois sítios ainda
/// não é uma lei; só uma PORTA é*. O que fica aqui é a tradução de `Tinta` e
/// `Mesh` para as duas contagens.
pub(crate) fn concorda_com(t: &Tinta, mesh: &Mesh) -> bool {
    t.topologia()
        .descreve(mesh.vert_count(), mesh.faces().len())
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

/// ⭐⭐⭐⭐ **QUE DEGRAU O DOCUMENTO PEDE DE VOLTA?** — e sem ele o plano que
/// acabou de ser lido é deitado fora no PRIMEIRO QUADRO.
///
/// ⛔⛔⛔ **Report do dono, 2026-09-21: *«sobreviveu mas sem os detalhes 8x»*.**
/// O plano ATRAVESSA o ficheiro (gate ao bit) e o `install_doc` instala-o — e
/// depois o quadro corre a [`rota`] com o `tinta_nivel` da CENA, que num app
/// acabado de abrir é `None`, logo `DaPeca { pedir: None }` e a [`garante`]
/// faz `tinta.take()`. *A cor por vértice sobrevive (ela viaja na malha), e é
/// por isso que o artista vê a tinta lá com a grossura errada em vez de a ver
/// desaparecer.*
///
/// ⚠️⚠️ **Os gates da wave mediam `encode`/`decode` e o produto tem MAIS UM
/// ELO** — a reconciliação do quadro. *Uma ida-e-volta medida a montante do
/// consumidor não afirma nada sobre o consumidor*, que é a forma que esta linha
/// já pagou na ponte da curva do pincel de pose.
///
/// ⚠️ **O knob é UM e os planos são N, e isso não é uma perda:** a [`rota`] dá
/// `pedir: nivel` a toda peça que JÁ tem plano, logo numa sessão viva todos
/// convergem para o degrau da fileira — *um documento só pode conter um
/// degrau, por construção*. Se um dia contiver dois, o da peça ACTIVA ganha e
/// os outros reconciliam-se, que é o que a fileira já faz hoje.
pub(crate) fn degrau_do_documento(objects: &[crate::SceneObject], activa: usize) -> Option<u8> {
    let da_activa = objects
        .get(activa)
        .and_then(|o| o.tinta.as_ref())
        .map(Tinta::nivel);
    // ⛔ E o recurso ao PRIMEIRO que tenha plano não é conforto: sem ele, uma
    //    peça activa sem detalhe fino deixaria a fileira desarmada e o quadro
    //    deitaria fora o plano de TODAS as outras.
    da_activa.or_else(|| {
        objects
            .iter()
            .find_map(|o| o.tinta.as_ref().map(Tinta::nivel))
    })
}

/// ⭐⭐⭐⭐ **ALGUMA peça carrega tinta fina AGORA?** — a lei que o aviso da
/// exportação faz, sobre as partes.
///
/// ⛔⛔ **Ela percorre a [`plano_da_peca`] e nunca o `Option` de cada peça, e
/// isso é o defeito que ela existe para não ter:** durante um traço o plano
/// está **emprestado ao gesto** e o `Option` da peça dona está VAZIO. Lido daí,
/// um `Ctrl+Shift+E` a meio de uma pincelada dizia *«nada de fino se perde»*
/// sobre uma peça cujo plano está na mão do traço. *Cada consumidor que lê o
/// `Option` directo inventa o seu próprio defeito* — a lei que a porta do plano
/// nasceu a escrever, e este é o **quarto** consumidor dela.
#[must_use]
pub(crate) fn alguma_peca_tem_plano(
    objects: &[crate::SceneObject],
    do_traco: Option<&ph2d_sculpt3d::tinta_fina::TintaDoTraco>,
) -> bool {
    (0..objects.len()).any(|i| plano_da_peca(objects, do_traco, i).is_some())
}

/// A lei da [`crate::Sculpt3dScene::plano_de`], com as partes que o laço de
/// upload consegue emprestar em separado.
pub(crate) fn plano_da_peca<'a>(
    objects: &'a [crate::SceneObject],
    do_traco: Option<&'a ph2d_sculpt3d::tinta_fina::TintaDoTraco>,
    i: usize,
) -> Option<&'a Tinta> {
    let obj = objects.get(i)?;
    if let Some(t) = do_traco
        && t.dono() == obj.id.0
    {
        return Some(t.tinta());
    }
    obj.tinta.as_ref()
}

/// ⭐⭐⭐⭐ **O PASSE DE TOPOLOGIA VAI CORRER NESTE PEN-DOWN?** — a pergunta
/// inteira, com as TRÊS metades que o consumidor de facto aplica.
///
/// ⛔⛔ **Ela nasceu de uma lente ESTREITA medida em 2026-09-21**, ao lado do
/// pânico do §14: a VOZ que avisa *«a tinta fina perde detalhe com a
/// topologia»* perguntava só pelo INTERRUPTOR, e o
/// [`crate::Sculpt3dScene::open_dyntopo_stroke`] corre com
/// `interruptor || verbo.corre_sem_o_interruptor()` **e** com a pilha por
/// montar. ⇒ o mesmo `if` produzia os dois erros de uma vez:
///
/// | configuração | o consumidor | a voz de antes |
/// |---|---|---|
/// | interruptor OFF + `Density` + plano armado | **corre** e refaz o plano | **calada** |
/// | interruptor ON + pilha de multiresolução | **não corre** | **avisa** de um preço que ninguém paga |
///
/// ⚠️ *Um falso negativo e um falso positivo na mesma condição, e nenhum deles
/// é visível a quem lê só um dos dois sítios* — que é a forma exacta que o
/// §5.0 desta casa chama de **a lente do painel mais larga que a do
/// consumidor**, aqui com os papéis trocados numa metade e não na outra.
///
/// ⚠️ **É uma função PURA**, como a [`o_gesto_muda_a_topologia`] de que ela é
/// dona: a cena pede um `wgpu::Device` e um gate ali nasceria `#[ignore]`.
pub(crate) fn o_passe_corre_no_pen_down(
    verbo: ph2d_sculpt3d::Verb,
    dyntopo_armado: bool,
    niveis: usize,
    tinta_fina_armada: bool,
) -> bool {
    // ⚠️ **O `Density` corre SEM o interruptor** (ordem do dono de 14/09:
    // *«Dynamic topology é para os outros pincéis»*), logo perguntar pelo
    // interruptor sozinho deixa-o de fora.
    (dyntopo_armado || verbo.corre_sem_o_interruptor())
        // ⚠️ **Com a pilha montada os dois motores recusam**, e a queixa sai no
        // ARM em vez de por dab — ver o `refine_for_dab`.
        && niveis == 1
        && o_gesto_muda_a_topologia(verbo, tinta_fina_armada)
}

/// ⭐⭐⭐⭐ **BASTA SUBIR AS AMOSTRAS, ou o device precisa do plano INTEIRO?**
///
/// O atalho que tira o custo da tinta fina de cima de `O(plano)` — medido em
/// 2026-09-20: só empacotar o plano custa `21,9 ms` no degrau `8×` da peça de
/// fábrica e `81,7 ms` no `16×`, **por quadro**, contra um quadro de `16,7`.
///
/// ⛔⛔⛔ **A cerca `!mexeu` NÃO diz o que o comentário dela prometia** (medido
/// 2026-09-21, ao lado do pânico do §14): ela é `!dirty.is_empty()`, e o
/// [`crate::Sculpt3dScene::mesh_rebuilt`] — que é quem TODA mudança de
/// topologia chama — faz `dirty.clear()`. *A linha que regista «a topologia
/// mudou» é a mesma que apaga a evidência de que alguma coisa mudou.*
///
/// ⚠️⚠️ **E é alcançável por um gesto comum:** um verbo com ÂNCORA (`Grab`,
/// `Snake Hook`) **não carimba no pen-down, ele PEGA** — logo depois da
/// triangulação do pen-down existe um quadro com a malha NOVA, o `dirty`
/// VAZIO e o plano emprestado. Ali o atalho disparava, o
/// `upload_tinta_at` nunca era chamado, e a entrada de fragmento da tinta
/// ficava a resolver o `@builtin(primitive_index)` da geometria que o
/// `upload_at` acabou de renovar contra o `origem`/`topo` de ANTES.
///
/// ⇒ a quarta cerca é **`malha_por_subir`** (o `SlotJob::Full`), que é o
/// device a dizer *«eu não tenho esta malha»*.
///
/// ⚠️ **É PURA porque a decisão não tem pixel nenhum**, e a lei do módulo é a
/// mesma da [`Rota`]: um gate no laço de upload pediria um `wgpu::Device` e
/// nasceria `#[ignore]`, fora da varredura que o CI corre.
pub(crate) fn so_as_amostras_bastam(
    emprestado: bool,
    mexeu: bool,
    tinta_suja: bool,
    malha_por_subir: bool,
) -> bool {
    emprestado && !mexeu && !tinta_suja && !malha_por_subir
}

impl crate::Sculpt3dScene {
    /// ⭐⭐⭐⭐ **ONDE ESTÁ O PLANO DESTA PEÇA, AGORA?** — a porta, e ela tem
    /// TRÊS consumidores.
    ///
    /// Durante um traço o plano não está na peça: o pen-down **empresta-o** ao
    /// gesto (um `take`, nunca um clone), e quem o devolve é o `close_stroke`.
    /// ⇒ *ler o `Option` da peça durante um traço devolve `None` sobre uma peça
    /// que TEM detalhe fino*, e cada consumidor que o faça inventa o seu
    /// próprio defeito:
    ///
    /// | consumidor | o que ele via sem esta porta |
    /// |---|---|
    /// | o upload | `armado = 0` ⇒ *a tinta fina desaparece ao começar a pintar* |
    /// | a VOZ | o aviso nunca soava (medido 20/09, §10.2) |
    /// | o **SAVE** | um `Ctrl+S` a meio de um traço grava a peça **sem o plano** |
    ///
    /// ⭐⭐ **E a pergunta é pelo DONO, não pelo índice `active`.** O empréstimo
    /// carrega quem o emprestou desde o §13, e é essa a resposta certa: *o
    /// plano que o traço segura é da peça que o emprestou, e de mais nenhuma* —
    /// uma peça que não seja a dona lê o `Option` dela, como deve.
    /// ⚠️ **Ela recebe as PARTES e não o `&self`, e não é arrumação:** o laço
    /// de upload precisa de `&mut self.renderer` ao mesmo tempo, e um método
    /// `&self` empresta a cena INTEIRA. *A assinatura que o compilador aceita é
    /// a mesma que um gate consegue montar sem uma cena* — e uma cena pede um
    /// `wgpu::Device`.
    pub(crate) fn plano_de(&self, i: usize) -> Option<&Tinta> {
        plano_da_peca(&self.objects, self.stroke.tinta_fina.as_ref(), i)
    }

    /// ⭐⭐⭐⭐ **Alguma peça desta cena carrega TINTA FINA?** — a pergunta que o
    /// aviso da exportação faz, e a irmã de cena da [`alguma_peca_tem_plano`].
    pub(crate) fn alguma_peca_tem_tinta_fina(&self) -> bool {
        alguma_peca_tem_plano(&self.objects, self.stroke.tinta_fina.as_ref())
    }

    /// ⭐⭐⭐ **A [`o_passe_corre_no_pen_down`] com as quatro entradas que a cena
    /// tem** — a irmã da [`Self::o_gesto_em_maos_muda_a_topologia`].
    pub(crate) fn o_passe_de_topologia_corre_no_pen_down(&self) -> bool {
        o_passe_corre_no_pen_down(
            self.brush.verb,
            self.dyntopo.armed,
            self.level_count(),
            self.tinta_fina_armada(),
        )
    }

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
/// `nivel = None` desarma: o plano sai do ar e a cor volta a ser a do canal por
/// vértice, que continua a ser escrito pelo mesmo traço.
///
/// ⭐⭐⭐⭐ **E ele NÃO é largado — é PARQUEADO** (ordem do dono, 2026-09-23).
/// Até aqui, voltar a fileira para `Mesh` **destruía** o detalhe fino: o canal
/// por vértice guarda a mesma cor em BAIXA resolução (a [`devolve`] escreve-o
/// no fim de cada traço), logo re-armar o degrau re-semeava daí e o que vivia
/// *entre* os vértices tinha-se ido. *Um gesto de um clique não pode apagar
/// trabalho sem o dizer* — e o roteiro dizia-o, o que não é curar.
///
/// ⭐⭐⭐ **A pergunta que decide se o parque serve NÃO é «alguma coisa
/// mudou?» — é «este plano ainda é um REFINAMENTO do que a peça mostra?»**, e
/// os dados para a responder já existem: as primeiras `verts` amostras de um
/// plano **são** a projecção por vértice dele ([`Tinta::plano_por_vertice`], uma
/// fatia — custo zero), e a [`devolve`] mantém `Mesh::colors` igual a ela. ⇒ o
/// parque volta se, e só se, as duas ainda baterem **ao bit**.
/// ⛔ *Isso é melhor do que uma impressão digital da cor:* não guarda campo
/// nenhum, não tem colisão, e **acerta no caso em que o artista pintou no nível
/// da malha e desfez** — ali a cor volta a bater e o plano volta a ser válido.
///
/// ⚠️ **O parque tem UMA ranhura por peça, e ela serve os dois gestos:** largar
/// (`8x → Mesh → 8x`) e TROCAR de degrau (`8x → 4x → 8x`). Trocar guarda o que
/// estava e tenta buscar o do degrau novo — logo o único caso que ainda perde é
/// passar por um TERCEIRO degrau pelo meio, que despeja a ranhura.
///
/// ⛔⛔ **E o preço está medido e é memória:** um plano `8x` na peça de fábrica
/// pesa `75,5 MB`, e parqueá-lo mantém isso vivo com a feature DESLIGADA. Ele
/// entra no [`crate::objects::SceneObject::footprint_bytes`], que é o orçamento
/// da fila de desfazer — *um plano que a conta não vê é memória que ninguém
/// sabe que tem*.
///
/// ⛔⛔ **O plano NOVO nasce SEMEADO da cor por vértice e nunca branco**, e
/// este é o mesmo defeito que a [`Tinta::semeada`] existe para impedir um nível
/// abaixo: uma peça pintada que ganha detalhe fino veria a tinta **desaparecer
/// no instante em que o artista pede mais resolução para ela**. A semente é
/// exacta nos vértices e interpolada no resto, que é a única resposta que a
/// malha sabe dar.
///
/// ⛔⛔⛔ **O PLANO QUE ELA CONSTRÓI É SEMPRE UNIFORME desde 2026-09-23, por
/// ordem do dono** (*«Even Detail derruba muito a resolução. retiro!»*): o
/// interruptor que graduava o plano por ÁREA de face saiu do painel, e com ele
/// o escolhedor. O que fica está no handoff §26.
///
/// ⚠️⚠️ **E ela CONTINUA a aceitar um plano GRADUADO, de propósito, e essa
/// metade é a que faz a retirada ser honesta:** a identidade pergunta só *«é o
/// degrau pedido, e ele descreve esta malha?»*. Um plano graduado que veio de
/// um FICHEIRO gravado antes da retirada passa nas duas e **sobrevive** —
/// *retirar o botão tira a capacidade de CRIAR, nunca o direito de abrir o que
/// já está gravado*. Reconstruí-lo uniforme apagaria a tinta fina que o artista
/// já tem no disco.
///
/// ⚠️ **Esta é a porta SEM tecto de placa, e é o arnês dos gates da lei**
/// (`#[cfg(test)]`): o quadro usa a [`garante_no_orcamento`], que é a mesma lei
/// com o degrau descido ao que a placa aceita para ESTA peça.
#[cfg(test)]
pub(crate) fn garante(
    mesh: &Mesh,
    tinta: &mut Option<Tinta>,
    parque: &mut Option<Tinta>,
    nivel: Option<u8>,
) -> bool {
    garante_no_orcamento(mesh, tinta, parque, nivel, u64::MAX)
}

// ⭐ O TECTO DA PLACA (quantas amostras cabem, e o degrau que cabe) vive num
// filho: aqui fica a lei do plano, lá o recurso do dispositivo que a limita.
#[path = "tinta_da_peca_placa.rs"]
mod placa;
pub(crate) use placa::{degrau_que_cabe, orcamento_da_placa};

/// ⭐⭐⭐ **A [`garante`] com o tecto da PLACA** — o que o quadro chama.
///
/// ⛔ **Um degrau que não cabe DESCE, não estoura nem é recusado em
/// silêncio:** o plano nasce no maior que cabe (ou não nasce, se nem o `2x`
/// cabe), e **quem chama lê o nível que ficou** em `tinta` e devolve-o ao
/// painel — senão o chip mostraria um degrau que a peça não tem e o quadro
/// seguinte tentaria outra vez. *Um tecto que aceita e entrega menos sem dizer
/// é o «aceita e mente» que esta casa já pagou três vezes.*
pub(crate) fn garante_no_orcamento(
    mesh: &Mesh,
    tinta: &mut Option<Tinta>,
    parque: &mut Option<Tinta>,
    nivel: Option<u8>,
    orcamento: u64,
) -> bool {
    let k = nivel.map(|k| k.min(NIVEL_MAX));
    if let (Some(k), Some(t)) = (k, tinta.as_ref())
        && t.nivel() == k
        && concorda_com(t, mesh)
    {
        return false;
    }
    let ja = tinta
        .as_ref()
        .filter(|t| concorda_com(t, mesh))
        .map(Tinta::topologia);
    let k = k
        .map(|k| degrau_que_cabe(mesh, ja, k, orcamento))
        .filter(|&j| j > 0);
    if let (Some(k), Some(t)) = (k, tinta.as_ref())
        && t.nivel() == k
        && concorda_com(t, mesh)
    {
        return false;
    }

    // ⚠️ **A ORDEM é load-bearing:** tira-se o que está no ar ANTES de olhar
    //    para o parque, e guarda-se DEPOIS — senão o que acabou de sair
    //    sobrescreve aquele que se ia buscar.
    let antigo = tinta.take();
    let mudou = antigo.is_some() || k.is_some();
    *tinta = k.map(|k| {
        desparqueia(parque, mesh, k).unwrap_or_else(|| {
            let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
            match mesh.colors() {
                Some(c) => Tinta::semeada(c, faces(), k),
                None => Tinta::nova(mesh.vert_count(), faces(), k),
            }
        })
    });
    if let Some(t) = antigo {
        *parque = Some(t);
    }
    mudou
}

/// ⭐⭐⭐ **O plano parqueado serve este degrau?** — e ele só sai do parque se
/// servir, nunca «o que houver».
///
/// ⛔⛔ **As TRÊS perguntas, e nenhuma é dispensável:**
/// 1. **o degrau** — um plano `4x` não é o `8x` que o artista pediu;
/// 2. **a malha** ([`concorda_com`]) — esculpir enquanto o plano dorme muda a
///    contagem de faces, e instalá-lo depois é tinta no sítio errado;
/// 3. ⭐ **a COR** — se o artista pintou no nível da malha enquanto o plano
///    dormia, devolvê-lo **ressuscitava tinta velha por cima da nova**, que é
///    pior do que a perda que esta wave existe para curar.
///
/// ⛔⛔ **E sem canal por vértice a `3` é VACUAMENTE verdadeira — ela não
/// recusa.** A 1.ª redacção recusava ali *«por conservadorismo»*, e o gate
/// reprovou: **numa peça sem cor por vértice o plano é a ÚNICA cor que ela
/// tem**, logo não existe tinta nova que devolvê-lo possa tapar, e recusar era
/// destruir o detalhe exactamente onde destruí-lo custa mais. *Um valor
/// «conservador» escolhido sem olhar o que ele protege pode ser o valor que
/// causa o dano.*
fn desparqueia(parque: &mut Option<Tinta>, mesh: &Mesh, k: u8) -> Option<Tinta> {
    let serve = parque.as_ref().is_some_and(|t| {
        t.nivel() == k
            && concorda_com(t, mesh)
            && mesh.colors().is_none_or(|c| c == t.plano_por_vertice())
    });
    serve.then(|| parque.take()).flatten()
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
///
/// ⭐⭐⭐⭐ **E ele carrega QUEM emprestou.** O `dono` é o [`crate::ObjectId`]
/// da peça, e é ele que a [`devolve_ao_dono`] usa para achar o caminho de
/// volta — *o empréstimo deixa de depender de o índice `active` não se mexer
/// entre o pen-down e o pen-up*.
pub(crate) fn empresta(
    tinta: &mut Option<Tinta>,
    dono: crate::ObjectId,
) -> Option<ph2d_sculpt3d::tinta_fina::TintaDoTraco> {
    tinta
        .take()
        .map(|t| ph2d_sculpt3d::tinta_fina::TintaDoTraco::nova(t, dono.0))
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

/// ⭐⭐⭐⭐ **O CAMINHO DE VOLTA, e ele é pelo DONO e nunca pela peça activa.**
///
/// ⛔⛔ **O achado da auditoria de 21/09 (§10.5):** o empréstimo tem duas
/// pontas — `empresta(&mut objects[active].tinta)` no pen-down e a devolução
/// no `close_stroke` — e **nada as prendia à mesma peça**. Com o índice a
/// mudar entre as duas, o plano da peça A aterra na B, a A fica sem ele e a
/// [`garante`] reconstrói-a **grosso**: *o mesmo sintoma do report do dono,
/// por outra porta*.
///
/// ⚠️ **Hoje nenhum gesto o alcança** (o `a_stroke_belongs_to_the_piece_it_started_on`
/// proíbe que um consumidor de `pick` mova a peça activa a meio de uma
/// pincelada, e a Hierarquia só troca na mudança de selecção) — ⛔ *mas a
/// cerca que o protegia é de OUTRO assunto*: ela existe contra um pânico de
/// índice, não contra este.
///
/// ⚠️ **Se a peça já não existe, o plano MORRE com ela, e isso é a resposta
/// certa** — um plano é paramétrico nas faces de UMA malha, e não há segunda
/// peça a que ele pudesse pertencer.
pub(crate) fn devolve_ao_dono(
    objects: &mut [crate::SceneObject],
    do_traco: ph2d_sculpt3d::tinta_fina::TintaDoTraco,
) -> bool {
    let dono = do_traco.dono();
    let Some(obj) = objects.iter_mut().find(|o| o.id.0 == dono) else {
        return false;
    };
    let crate::objects::SceneObject {
        stack,
        tinta,
        tinta_suja,
        ..
    } = obj;
    devolve(stack.mesh_mut(), tinta, Some(do_traco));
    *tinta_suja = true;
    true
}

/// O tecto da PLACA (24/09) — irmão dos dois de cima, pelo tecto de LOC deles.
#[cfg(test)]
#[path = "tinta_da_peca_placa_tests.rs"]
mod placa_tests;
/// ⚠️ **Irmão do [`tests`], cortado dele pelo tecto de LOC e pelo ASSUNTO** —
/// lá o PLANO, aqui as PORTAS que decidem. Ver o cabeçalho do ficheiro.
#[cfg(test)]
#[path = "tinta_da_peca_portas_tests.rs"]
mod portas_tests;
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
