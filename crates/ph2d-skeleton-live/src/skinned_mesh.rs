//! ⭐⭐⭐ **A MALHA DE UMA IMAGEM PRESA, COM OS PESOS DENTRO** — o que o `SkinBind::source` guarda
//! desde que a pele de imagem passou a usar o padrão-ouro.
//!
//! # Porque os pesos passaram a ser GUARDADOS
//!
//! ⛔ **A lei deste repo era o contrário, e está escrita:** *«os pesos não se guardam, derivam-se —
//! uma tabela por ordem de varredura é o vector paralelo que o `corner_radius` proíbe»*. Ela foi
//! escrita para a lei **euclidiana** (`bump` sobre a distância ao eixo), que é uma **função de um
//! ponto**: derivar custava `0,146 %` de um quadro e guardar não comprava nada.
//!
//! ⭐ **Os [*Bounded Biharmonic Weights*](ph2d_skin_weights) não são função de um ponto.** Eles são
//! a solução de um problema variacional **sobre a arte inteira** (minimizar `∫‖Δw‖²` com caixas e
//! Dirichlet nos ossos), e resolvê-lo custa dezenas de milissegundos. ⇒ *quem quer o padrão-ouro
//! resolve uma vez, ao prender, e guarda* — que é o que Spine, Blender e Maya fazem.
//!
//! # ⭐⭐⭐ E eles NÃO são um vector paralelo: são o MESMO objecto
//!
//! A objecção da lei antiga era real — *duas listas que uma edição pode dessincronizar*. A cura é
//! não ter duas listas: a malha e os pesos viajam **numa struct só**, o comprimento de uma é
//! função do da outra, e a porta [`SkinnedMesh::valida`] recusa o par que não fecha. ⛔ Não há
//! como gravar uma e não a outra, porque não há duas coisas para gravar.
//!
//! ⚠️ **O número de ossos DERIVA-SE** ([`SkinnedMesh::ossos`]) — guardá-lo seria a terceira
//! grandeza a poder discordar das outras duas, exactamente o defeito que a
//! [`ph2d_poly2d::Mesh2d`] já evita ao derivar a UV do tamanho.
//!
//! # ⚠️ A tabela é por TENDÃO, nunca por pose resolvida
//!
//! Um osso que dobra dá `N` poses à pele, e um osso apagado dá **zero**. A coluna `j` desta tabela
//! é o osso **autorado** — o `j`-ésimo [`ph2d_skeleton_ecs::Tendon`] da mesma [`SkinBind`] —, e é o
//! [`ph2d_skeleton::SkinBone::tendon`] que reencontra as poses dele no quadro. *Guardar por pose
//! resolvida faria a arte saltar no dia em que alguém apagasse um osso.*
//!
//! [`SkinBind`]: ph2d_skeleton_ecs::SkinBind

use ph2d_poly2d::Mesh2d;

/// ⭐⭐⭐ **A malha de repouso de uma imagem mais a fracção de cada vértice que pertence a cada osso.**
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SkinnedMesh {
    /// A malha em pixels da imagem, no repouso.
    pub mesh: Mesh2d,
    /// **Achatado**: `pesos[v * ossos + j]` é a fracção do vértice `v` que pertence ao tendão `j`.
    ///
    /// ⚠️ **Achatado e não `Vec<Vec<f64>>`**, e o motivo é o quadro: o refinamento do `Smooth`
    /// interpola isto por vértice inventado, e um `Vec` por vértice seria uma alocação por vértice
    /// **por quadro**. A fatia contígua atravessa a [`crate::skin_refine::refine_skinned`] com UMA
    /// alocação além do buffer de saída — a tabela com os gradientes atrás dos pesos (`3×` esta), que
    /// a lei de Hermite pede e que se recupera por quadro.
    ///
    /// ⭐ **Vazio é legal e significa *«esta malha não traz pesos»***: a jusante ela cai na lei
    /// derivada, que é o caminho da 1.ª mídia. ⛔ Não é um erro silencioso — é o único estado em
    /// que uma malha gravada antes desta wave pode chegar aqui, e ele é nomeado.
    pub pesos: Vec<f64>,
}

impl SkinnedMesh {
    /// A malha sem pesos — a leitura de *«resolve pela lei derivada»*.
    #[must_use]
    pub fn sem_pesos(mesh: Mesh2d) -> Self {
        Self {
            mesh,
            pesos: Vec::new(),
        }
    }

    /// ⭐ **Quantos ossos a tabela cobre** — `0` quando não há tabela.
    ///
    /// ⚠️ **DERIVADO**, e é isso que impede uma terceira grandeza de discordar das outras duas.
    #[must_use]
    pub fn ossos(&self) -> usize {
        self.pesos
            .len()
            .checked_div(self.mesh.rest.len())
            .unwrap_or(0)
    }

    /// ⛔ **O par fecha?** — `pesos` tem de ser um múltiplo exacto do número de vértices.
    ///
    /// ⚠️ Um resto diferente de zero quer dizer que a malha e a tabela descrevem coisas
    /// diferentes, e a resposta certa é **recusar**, nunca ler a tabela deslocada: uma tabela
    /// deslocada por um vértice dá pesos plausíveis e arte errada.
    #[must_use]
    pub fn valida(&self) -> bool {
        let n = self.mesh.rest.len();
        n > 0 && self.pesos.len().is_multiple_of(n)
    }

    /// Os pesos do vértice `v`. Vazio quando não há tabela — ⛔ nunca uma fatia de outro vértice.
    #[must_use]
    pub fn pesos_de(&self, v: usize) -> &[f64] {
        let m = self.ossos();
        if m == 0 {
            return &[];
        }
        self.pesos.get(v * m..(v + 1) * m).unwrap_or(&[])
    }
}

/// ⭐⭐⭐ **A FORMA VECTORIAL PRESA, COM OS PESOS DENTRO** — o gémeo da [`SkinnedMesh`] para a 1.ª
/// mídia.
///
/// ⚠️ **Ela é uma struct própria e não um genérico**, e a razão é o que cada mídia GUARDA: uma
/// imagem guarda uma malha (a amostragem da arte) e um caminho guarda o **caminho** (a arte
/// exacta). ⛔ A malha do domínio de um caminho é um **andaime do bind** — ela não sobrevive, e
/// guardá-la seria guardar o andaime em vez da obra.
///
/// ⚠️ **A tabela é por PONTO DE CONTROLO**, na ordem do `for_each_vert_mut`: `3k`, `3k+1`, `3k+2`
/// são âncora, alça de entrada e alça de saída do vértice `k`. Ver
/// [`ph2d_vec_skin::pesos::pesos_do_caminho`].
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SkinnedPath {
    /// A geometria autorada, como ela foi presa.
    pub path: ph2d_vec_scene::VecPath,
    /// **Achatado**: `pesos[c * ossos + j]` é a fracção do ponto de controlo `c` que pertence ao
    /// tendão `j`. ⭐ Vazio ⇒ a lei derivada (um caminho ABERTO não tem interior, logo não tem
    /// domínio, logo não tem pesos — e isso é uma resposta, não um erro).
    pub pesos: Vec<f64>,
    /// ⭐⭐⭐ **O CAMPO DO DOMÍNIO — a malha do bind, que até 2026-09-20 era deitada fora.**
    ///
    /// Com ele a lei da curva lê a linha de **qualquer ponto** do interior, em vez de traçar uma
    /// recta entre as linhas dos dois nós; a tabela [`Self::pesos`] continua a ser o que a lei dos
    /// NÓS lê, e as duas concordam ao bit nas âncoras **porque ela é amostrada deste campo**.
    ///
    /// ⭐ **`None` é legal e é o que um bind ANTERIOR a esta wave carrega** — ali a lei da curva
    /// volta à mistura, que é exactamente o que ela fazia. ⛔ *Nenhuma migração:* re-prender a forma
    /// preenche-o, e até lá o desenho é o de sempre.
    pub campo: Option<ph2d_vec_skin::pesos::CampoDoDominio>,
}

/// ⭐⭐ **A FORMA ANTERIOR do registo, para a [`le`] não perder os pesos de um bind já gravado.**
///
/// ⛔⛔ **O postcard é POSICIONAL e não auto-descritivo:** apendar um campo faz os bytes antigos
/// acabarem cedo, o `from_bytes` devolve `Err` e a [`le`] devolveria `None` — que quem chama lê
/// como *«esta fonte não se lê»* e resolve pela **lei derivada**. ⇒ toda forma já presa perderia a
/// tabela do padrão-ouro **em silêncio**, e o desenho mudava sem ninguém ter tocado nela.
///
/// ⚠️ **A ordem das tentativas é load-bearing:** o novo PRIMEIRO. Bytes novos lidos como antigos
/// deixariam cauda por consumir, e bytes antigos lidos como novos **falham a meio de um campo** —
/// só a ordem *novo → antigo* recusa exactamente o que tem de recusar.
#[derive(serde::Deserialize)]
struct SkinnedPathV1 {
    path: ph2d_vec_scene::VecPath,
    pesos: Vec<f64>,
}

impl SkinnedPath {
    /// Quantos ossos a tabela cobre — `0` quando não há tabela. ⚠️ **DERIVADO**, como na irmã.
    #[must_use]
    pub fn ossos(&self) -> usize {
        self.pesos.len().checked_div(self.pontos()).unwrap_or(0)
    }

    /// Quantos pontos de controlo o caminho tem — três por vértice.
    #[must_use]
    pub fn pontos(&self) -> usize {
        self.path.verts_all().count() * 3
    }

    /// ⛔ **O par fecha?** — `pesos` tem de ser um múltiplo exacto do número de pontos de controlo.
    #[must_use]
    pub fn valida(&self) -> bool {
        let n = self.pontos();
        n > 0 && self.pesos.len().is_multiple_of(n)
    }

    /// ⭐⭐⭐ **A LINHA DE PESOS DO NÓ `k`** — a única que o desenho lê.
    ///
    /// ⚠️ **O peso é do NÓ** ([`ph2d_vec_skin::dono_do_peso`], ordem do dono de 2026-09-19): as duas
    /// linhas das alças continuam gravadas e **nunca são lidas**. ⛔ Devolve `None` quando a tabela
    /// não fecha com o caminho — *pesos plausíveis sobre os pontos errados dão arte errada sem um
    /// erro*, que é a cerca que o `recook` já aplica.
    #[must_use]
    pub fn linha_do_no(&self, k: usize) -> Option<&[f64]> {
        if !self.valida() {
            return None;
        }
        let n = self.ossos();
        self.pesos.get(k * 3 * n..k * 3 * n + n)
    }
}

/// ⭐⭐⭐ **A PORTA do formato guardado** — o `SkinBind::source` lê-se por aqui, e só por aqui.
///
/// ⛔⛔ **Ela existe porque o formato era descodificado À MÃO em SETE sítios** (medido 2026-09-19),
/// cada um com a sua cerca e a sua maneira de desistir. *Uma lei escrita em sete sítios ainda não é
/// uma lei* — e a oitava chamada seria a que esqueceria uma das cercas.
///
/// `None` é a resposta para uma fonte que não se lê: quem chama cai na lei derivada, que é a decisão
/// que os sete já tomavam cada um por si.
#[must_use]
pub fn le(bytes: &[u8]) -> Option<SkinnedPath> {
    if let Ok(g) = postcard::from_bytes::<SkinnedPath>(bytes) {
        return Some(g);
    }
    // ⭐ A forma ANTERIOR — ver [`SkinnedPathV1`]. Um bind gravado antes de 2026-09-20 não tem
    // campo, e a lei da curva volta à mistura das linhas dos nós, que é o que ele já desenhava.
    let v1 = postcard::from_bytes::<SkinnedPathV1>(bytes).ok()?;
    Some(SkinnedPath {
        path: v1.path,
        pesos: v1.pesos,
        campo: None,
    })
}

/// ⭐⭐ **O sentido inverso da [`le`].** `None` quando a serialização falha — e aí quem chama **não
/// escreve**, porque meia fonte é pior do que a antiga.
#[must_use]
pub fn grava(g: &SkinnedPath) -> Option<Vec<u8>> {
    postcard::to_allocvec(g).ok()
}

#[cfg(test)]
#[path = "skinned_mesh_tests.rs"]
mod tests;

/// ⭐⭐ **A LENTE B — as réguas contra o PADRÃO-OURO** (auditoria de 2026-09-20).
///
/// ⚠️ Saiu do [`tests`] por tecto de LOC, e o corte é por RESPONSABILIDADE: o irmão mede *«as duas
/// leis concordam?»* e este mede *«a nossa lei bate a mesma forma deformada como malha densa?»*.
#[cfg(test)]
#[path = "skinned_mesh_ouro_reguas_tests.rs"]
pub(crate) mod ouro_reguas_tests;

#[cfg(test)]
#[path = "skinned_mesh_ouro_tests.rs"]
mod ouro_tests;

/// ⭐⭐ **ONDE O ERRO NASCE** — a atribuição ao substrato, e o que cada wave comprou.
#[cfg(test)]
#[path = "skinned_mesh_ouro_nos_tests.rs"]
mod ouro_nos_tests;

/// ⭐⭐⭐ **A LEI QUE MISTURA OS OSSOS** — o vinco, as três leis lado a lado, e onde cada uma parte.
#[cfg(test)]
#[path = "skinned_mesh_lei_tests.rs"]
mod lei_tests;

/// ⭐⭐⭐ **QUANTO a lei nova move o DESENHO** — o gate do TAMANHO, e a imagem que o produziu.
///
/// ⚠️ Ele é o par obrigatório do [`lei_tests`]: lá mede-se um extremo LOCAL (`5,1×` na aresta do
/// cotovelo) e aqui o tamanho do que se vê (`4,3 %` da espessura). *Uma régua local não diz o
/// tamanho.*
#[cfg(test)]
#[path = "skinned_mesh_lei_tamanho_tests.rs"]
mod lei_tamanho_tests;

/// ⭐⭐⭐ **A REGULARIDADE DO CONTORNO** — o vector contra a IMAGEM, na grandeza que o report de
/// 2026-09-20 apontou: *«deixa tudo irregular»*. ⚠️ Nenhuma régua anterior a via.
#[cfg(test)]
#[path = "skinned_mesh_regularidade_tests.rs"]
mod regularidade_tests;

/// ⭐⭐⭐ **AS ONDULAÇÕES** — a aresta que devia ser um arco e vai para os dois lados
/// (*«várias curvas ao longo do caminho»*, report de 2026-09-20). ⚠️ Irmão do
/// [`regularidade_tests`], e a separação é a dos dois defeitos: **um canto e uma onda não são a
/// mesma coisa, e não têm a mesma cura.**
#[cfg(test)]
#[path = "skinned_mesh_ondulacao_tests.rs"]
mod ondulacao_tests;

/// ⭐⭐ **AS SONDAS das ondulações** — elas imprimem; as leis são o irmão acima. Saíram por tecto
/// de LOC, e o corte é por RESPONSABILIDADE.
#[cfg(test)]
#[path = "skinned_mesh_ondulacao_sondas_tests.rs"]
mod ondulacao_sondas_tests;

/// ⭐⭐⭐ **A RÉGUA DAS ONDAS A PASSO FIXO** — o irmão do [`ondulacao_tests`] conta **por
/// amostra** e a amostragem dele é **por segmento**, logo duas contagens de nós não são
/// comparáveis ali. ⛔ Foi dessa confusão que saiu o mecanismo errado que eu reportei ao dono.
#[cfg(test)]
#[path = "skinned_mesh_ondulacao_regua_tests.rs"]
mod ondulacao_regua_tests;

/// ⭐⭐⭐ **A SERPENTINA** — o TAMANHO de uma onda (a flecha e o arco), o joelho do
/// [`crate::subdivisao::DIVISOES_POR_OSSO`] e os gates dele. ⚠️ Irmão do
/// [`ondulacao_regua_tests`], e a separação é a dos dois trabalhos: **um torna duas densidades
/// comparáveis, o outro pergunta se a onda SE VÊ.**
#[cfg(test)]
#[path = "skinned_mesh_serpentina_tests.rs"]
mod serpentina_tests;

/// ⭐⭐⭐ **A LEI DO RIVE CONTRA A DE HOJE** — as sondas da ordem *«vá investigar como Rive faz»*.
#[cfg(test)]
#[path = "skinned_mesh_rive_tests.rs"]
mod rive_tests;

/// ⭐⭐ **AS SONDAS** da mesma jornada — elas imprimem; as leis são o irmão acima. Saíram por
/// tecto de LOC, e o corte é por RESPONSABILIDADE.
#[cfg(test)]
#[path = "skinned_mesh_rive_sondas_tests.rs"]
mod rive_sondas_tests;
