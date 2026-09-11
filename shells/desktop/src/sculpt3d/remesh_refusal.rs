//! **POR QUE UM REMESH RECUSOU** — o tipo, e a explicação em prosa.
//!
//! Irmão (`#[path]`) do [`super`], e o corte foi **forçado pela HR-18** (o pai
//! chegou a 616 LOC). ⭐ **Mas ele pagou uma dívida que já lá estava:** a mesma
//! recusa era explicada em **dois** sítios — o painel e as teclas — com dois
//! textos diferentes, e nada obrigava os dois a concordar.
//!
//! ⚠️ **A exaustividade não se perdeu, mudou de sítio.** Antes, um `match` em cada
//! chamador obrigava cada um a decidir quando nascia uma variante; agora quem
//! obriga são o [`RemeshRefusal::explain`] e o
//! [`RemeshRefusal::reaches_voxel_remesh`], que são exaustivos os dois. *Uma
//! decisão em vez de três, e nenhuma delas por omissão.*

/// Por que o botão de reconstruir RECUSOU.
///
/// ⚠️ **Três causas, três nomes.** Elas cabiam num `Option` e o chamador tinha
/// de eleger UMA mensagem para as três: elegeu a da pilha de multires, então um
/// campo que vazou mandava o artista reverter níveis inexistentes. O conserto
/// não é uma mensagem melhor — é o tipo deixar de perder a informação.
// ⚠️ **Deixou de ser `Copy` em 2026-08-21**: a `Fill` carrega um `FillError`, que
// tem um `String` no braço `Mesh`. Clonar uma recusa é barato e acontece uma vez
// por gesto recusado.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sculpt3d) enum RemeshRefusal {
    /// A pilha de multires está montada: o remesh troca a topologia, e todo
    /// nível acima é subdivisão da base.
    MultiresStack,
    /// Não há peça na cena para reconstruir.
    EmptyScene,
    /// O motor recusou — hoje, o campo sem interior.
    Engine(ph2d_sdf::RemeshError),
    /// **A RETOPOLOGIA não fechou uma malha bem formada.**
    ///
    /// ⚠️ Caso próprio e não um `Engine` reaproveitado: as duas reconstruções
    /// falham por motivos que não se parecem — aquela por um campo sem interior,
    /// esta por um grafo de células degenerado —, e uma variante só faria o
    /// chamador eleger UMA mensagem para as duas. É a mesma lição que partiu o
    /// `Option` original em três casos.
    Quad(ph2d_mesh::MeshError),
    /// **A GRADE NÃO TEM ONDE POUSAR** — a extração devolveu zero faces.
    ///
    /// ⚠️ **Caso próprio porque a peça ficaria INVISÍVEL, e sem uma palavra.**
    /// Uma retopologia não pode resolver uma grade mais fina que a malha que ela
    /// lê; abaixo desse piso a extração não devolve erro nenhum — ela devolve uma
    /// malha **vazia**, e o artista vê o objeto sumir. Foi o que o painel oferecia
    /// no extremo esquerdo do slider até 2026-08-19, e o que o Enio fotografou.
    /// Com o knob de `detail` isto passou a ser inalcançável; a variante fica
    /// porque *o próximo chamador pode não passar pelo knob*.
    TooCoarseToResolve,
    /// **O LAYOUT do traçado não fecha** — cadeia GLOBAL (ADR-0162).
    ///
    /// ⚠️ Um arco que só um patch usa, ou um patch com menos de três lados. É
    /// recusa da fase de **decomposição**, e ela tem nome próprio pela mesma lei
    /// que partiu o `Option` original: a cura de um layout aberto não se parece
    /// nada com a de uma quantização inviável.
    Layout(ph2d_quantize::LayoutError),
    /// **A QUANTIZAÇÃO não fecha** — cadeia GLOBAL (ADR-0162).
    ///
    /// ⚠️ Ela distingue *inviável* de *orçamento esgotado*, e a diferença é o que
    /// o artista precisa: a primeira pede outra malha, a segunda pede paciência.
    Quantize(ph2d_quantize::SolveError),
    /// **A MONTAGEM recusou** — a lei do patch não bate com a quantização.
    ///
    /// ⚠️ É bug a montante e não uma propriedade da peça: o F4 devolve inteiros
    /// que satisfazem a lei por construção. Recusar em vez de remendar é o que
    /// impede uma malha torcida chegar à tela.
    Fill(ph2d_quadfill::FillError),
    /// **A EXTRACÇÃO recusou** — o caminho do mapa de grade inteira, que desde
    /// 2026-08-25 é o de **omissão** (`PH2D_RETOPO_EXTRACT=0` volta ao de sempre).
    ///
    /// ⚠️ **Caso próprio e não um [`Self::Fill`] reaproveitado:** aquele fala de uma
    /// montagem por patch, este de um domínio que não cabe na grade exacta ou de um
    /// mapa com uma coordenada não finita. *Uma variante partilhada faria o artista
    /// ler a cura da outra fase.*
    Extract(ph2d_quadextract::ExtractError),
    /// **A RETOPOLOGIA ESTILHAÇOU A PEÇA** — a saída tem mais pedaços do que a entrada.
    ///
    /// ⛔⛔⛔ **Report do artista com foto, 2026-08-30** (*«péssimo»*): um quad a flutuar
    /// solto ao lado de uma ponta. Reproduzido ao carregar no botão uma **segunda** vez
    /// sobre a saída da primeira: `2` peças, um pedaço solto de `22` faces, e a ponta mais
    /// longa cortada de `−0,2 %` para `−35,0 %`.
    ///
    /// ⚠️ **Caso próprio e não um [`Self::Quad`] reaproveitado:** aqui a malha é bem
    /// formada — fechada, sem bordo, sem não-manifold —, e é por isso que nenhuma das
    /// recusas que existiam a alcança. *O defeito não é a malha estar mal construída; é ela
    /// ter deixado de ser UMA peça.*
    Shattered {
        /// Quantos pedaços a retopologia devolveu.
        pieces: usize,
        /// Quantos a escultura tinha.
        was: usize,
    },
    /// ⛔⛔⛔ **A TENTATIVA ESTOUROU** — um `panic` apanhado pela rede do botão.
    ///
    /// ⛔ **Até 2026-08-30 este caso era devolvido como [`Self::TooCoarseToResolve`]**, e o
    /// artista lia *«a malha é grossa demais para uma grade de quads: subdivida antes»* —
    /// **uma frase sobre a peça dele para um defeito nosso**, que o manda fazer exactamente
    /// a coisa que não ajuda. *Reproduzido com `PH2D_ISO_ADAPT=1` na peça dele: um estouro
    /// em `ph2d-gridmap`.*
    ///
    /// ⚠️ **Caso próprio pela mesma lei que partiu o `Option` original em casos nomeados:**
    /// a `ph2d-quadchain` já distinguia (`Verdict::Panicked`) e **esta porta não** — *duas
    /// portas para o mesmo botão, e só uma sabia dizer o que tinha acontecido.*
    Panicked,
}

impl RemeshRefusal {
    /// **A FRASE que o artista lê** — e ela nomeia o CONSERTO, não só a causa.
    ///
    /// ⚠️ **A diferença entre uma recusa útil e uma muda é dizer o que fazer a
    /// seguir**, e foi ela que partiu o `Option` original em casos nomeados: com
    /// um `None` só, o chamador elegia a mensagem da pilha de multires — e um
    /// campo que vazava mandava o artista *"reverter os níveis"* que ele não tem.
    #[must_use]
    pub(in crate::sculpt3d) fn explain(&self) -> String {
        match self {
            Self::MultiresStack => String::from(
                "nao' reconstroi com a pilha montada: o remesh troca a TOPOLOGIA, e todo nivel \
                 acima e' subdivisao dela -- ACHATE a pilha antes",
            ),
            Self::EmptyScene => String::from("nao' reconstroi: nao ha' peca na cena"),
            // ⚠️ **A mensagem NÃO manda o artista mexer na peça** — ver [`Self::Panicked`].
            // *Um defeito nosso não se conserta subdividindo a escultura dele*, e a frase
            // anterior (a do `TooCoarseToResolve`) dizia exactamente isso.
            Self::Panicked => String::from(
                "a retopologia falhou por um defeito NOSSO e a escultura fica como esta': \
                 tente outro Detail -- e se puder, guarde a peca e avise",
            ),
            Self::Engine(e) => format!(
                "nao' reconstroi, e a escultura fica como esta': {e} -- tente outra resolucao"
            ),
            Self::Quad(e) => {
                format!("a retopologia nao fechou uma malha, e a escultura fica como esta': {e}")
            }
            // ⚠️ **A mensagem nomeia o CONSERTO**: o `Detail` não alcança este
            // estado, então quem chega aqui tem uma malha grossa demais para ser
            // retopologizada de todo.
            Self::TooCoarseToResolve => String::from(
                "a malha e' grossa demais para uma grade de quads, e a escultura fica como \
                 esta': subdivida (ou use o Remesh) antes",
            ),
            // ⭐⭐ **O género perdido tem CONSERTO PRÓPRIO, e é por isso que ele
            // não cai no braço geral.** ⛔ *"tente outro Detail"* seria mandar o
            // artista para o sítio errado: a decomposição não depende do alvo de
            // densidade nenhum — ela é do campo e do traçado, e mexer no slider
            // devolve exactamente a mesma recusa. Medido em 2026-08-22: o mesmo
            // toro falha em **todos** os pesos, e o toro do lado passa em todos.
            Self::Layout(ph2d_quantize::LayoutError::GenusLost { complex, surface }) => format!(
                "esta peca tem um buraco (ou uma alca) que o tracado nao soube contornar, e a \
                 escultura fica como esta': a decomposicao fechou como {complex} e a peca e' \
                 {surface}. Mexer no Detail nao muda isto -- use PH2D_RETOPO_LEGACY=1, ou passe \
                 o Remesh na peca antes"
            ),
            Self::Layout(e) => format!(
                "o tracado nao fechou um layout, e a escultura fica como esta': {e:?} -- \
                 tente outro Detail, ou PH2D_RETOPO_LEGACY=1"
            ),
            // ⚠️ **As duas causas de dentro têm consertos diferentes**, e por isso
            // o `{e:?}` viaja: *inviável* pede outra malha, *orçamento esgotado*
            // pede outro alvo — a busca cresce com a sujidade do layout, não com o
            // tamanho da peça.
            Self::Quantize(e) => format!(
                "a quantizacao nao fechou, e a escultura fica como esta': {e:?} -- tente outro \
                 Detail, ou PH2D_RETOPO_LEGACY=1"
            ),
            Self::Fill(e) => {
                format!("a montagem recusou (bug a montante), e a escultura fica como esta': {e:?}")
            }
            Self::Extract(e) => format!(
                "a extraccao do mapa de grade inteira recusou, e a escultura fica como esta': \
                 {e} -- ponha PH2D_RETOPO_EXTRACT=0 para voltar ao caminho de sempre"
            ),
            // ⚠️ **A frase nomeia o que o artista VÊ** (pedaço solto a flutuar) e o conserto que
            // de facto o resolve. ⛔ *«tente outro Detail»* sozinho seria adivinhar: medido na
            // peça dele, a re-entrada parte a peça em qualquer ponto do slider, e o que a cura é
            // **não voltar a carregar sobre a saída** — daí o Ctrl+Z vir primeiro.
            Self::Shattered { pieces, was } => format!(
                "a retopologia partiu a peca em {pieces} pedacos soltos (ela entrou com {was}), e \
                 a escultura fica como esta' -- desfaca (Ctrl+Z) ate' voltar a' escultura \
                 original antes de carregar outra vez, ou baixe o Detail"
            ),
        }
    }

    /// **ESTA RECUSA PODE VIR DO VOXEL REMESH?**
    ///
    /// ⚠️ **Ela existe para o `debug_assert` do chamador**, e é o que sobrou da
    /// propriedade que os `match` espalhados davam: uma variante nova obriga a
    /// decidir **aqui** se ela alcança aquele gesto — e um `false` errado grita no
    /// sítio onde acontece em vez de virar uma mensagem estranha na consola.
    #[must_use]
    pub(in crate::sculpt3d) fn reaches_voxel_remesh(&self) -> bool {
        match self {
            Self::MultiresStack | Self::EmptyScene | Self::Engine(_) => true,
            // As cinco da família da retopologia — as duas do porte local e as
            // três da cadeia global.
            Self::Quad(_)
            | Self::TooCoarseToResolve
            | Self::Layout(_)
            | Self::Quantize(_)
            | Self::Fill(_)
            | Self::Extract(_)
            | Self::Shattered { .. }
            // ⚠️ **`false`, e a razão é o SÍTIO da rede:** o `catch_unwind` que produz esta
            // variante vive na cadeia de extracção
            // ([`crate::sculpt3d::history::retopo_extract`]); o remesh por voxels não tem
            // rede nenhuma e um estouro lá derrubaria a janela. *Se alguém lhe puser uma,
            // esta linha é o sítio onde tem de mudar.*
            | Self::Panicked => false,
        }
    }
}
