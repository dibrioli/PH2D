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
            | Self::Extract(_) => false,
        }
    }
}
