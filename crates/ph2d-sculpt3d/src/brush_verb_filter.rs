//! **QUE LEIS UM FILTRO PODE RODAR** — o catálogo, a faixa de cada uma e o
//! verbo que a semeia; irmão do [`super`], que responde *o que um verbo É*.
//!
//! ⚠️ **A divisão não é de tamanho, é de assunto.** O arquivo pai é o CATÁLOGO
//! DE PINCÉIS (quais verbos existem, o que cada um lê, com que gesto se pega);
//! aqui mora o catálogo de **FILTROS**, que é uma lista diferente — sete leis
//! contra vinte e três verbos, e nem toda lei tem verbo.
//!
//! ⚠️ **A tabela é UMA, e é ela que torna o motor do filtro EXAUSTIVO.** O
//! [`crate::SculptStroke::filter`] casa sobre [`FilterKind`], não sobre
//! [`Verb`] — então uma lei nova que entre aqui e não seja implementada lá é um
//! **erro de compilação**, e não um filtro que aparece na lista e não move um
//! vértice. É a mesma razão pela qual a [`crate::GripLaw`] é uma tabela.
//!
//! # Por que a lei NÃO é um verbo
//!
//! Três das sete (`Scale`, `Sphere`, `Random`) não têm pincel correspondente e
//! **não podem ter**: elas leem a posição do vértice contra a ORIGEM DO OBJETO
//! (as duas primeiras) ou o hash dela (a terceira), e nenhuma delas tem sentido
//! sob uma pegada de dab — não há *"o quanto isto está perto do cursor"* que as
//! module. Empurrá-las para dentro do [`Verb::ALL`] daria três chips de pincel
//! que ninguém pode carimbar.
//!
//! ⚠️ **E é isso que decide a UI:** a lista que o artista escolhe é o
//! [`FilterKind::ALL`], **não** os verbos que filtram. O
//! [`Verb::filter_kind`] segue vivo como **SEMENTE** (*o verbo na mão sabe qual
//! lei ele seria*), que é o padrão do `arm_verb_defaults` — nunca uma segunda
//! porta para escolher a mesma lei.

use super::*;

/// **A LEI que um filtro roda** — uma por LEI, nunca uma por verbo.
///
/// A ordem é a do `prop_mesh_filter_types` da referência
/// (`sculpt_filter_mesh.cc:241`), porque é a ordem em que o artista já viu esta
/// lista noutro programa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterKind {
    /// `calc_smooth_filter` — anda a fração `f` na direção da média do anel.
    ///
    /// ⚠️ **Com `f` NEGATIVO ele AFIA**, e é por isso que a faixa da referência
    /// é `[−1, 1]` e não `[0, 1]`: `smooth(−f)` **é** `sharpen(f)` (a álgebra
    /// está no gate `the_sharpen_filter_is_the_smooth_filter_dragged_backwards`,
    /// que a mede em vez de a afirmar).
    Smooth,
    /// `calc_scale_filter` — `t = pre × f`, uma **homotetia em torno da ORIGEM
    /// DO OBJETO**.
    ///
    /// ⚠️ **Não é o gesto de escala do [`crate::MaskTransform`], e a distinção
    /// foi MEDIDA** (`tests/measure_scale_filter.rs`): aquele escala em torno do
    /// **centroide ponderado do que está livre** e este em torno da origem, o
    /// que sobre uma esfera com máscara em gradiente dá **48,0% do movimento**
    /// de desvio — e **90,1%** com a peça deslocada três unidades. As duas
    /// coincidem (a um ULP, `1,4e-7`) exactamente quando o peso é uniforme *e* a
    /// peça está na origem, que é o controle daquela sonda.
    ///
    /// ⚠️ **A ORIGEM é a do objeto, e a consequência fica NOMEADA:** uma peça
    /// cujo pivô não está no meio dela escala *para longe do pivô*. É o
    /// comportamento da referência (`orig_positions` são coordenadas de objeto),
    /// e quem o move é o gesto de re-centrar, que já existe.
    Scale,
    /// `calc_inflate_filter` — `t = orig_normals × f`, ao pé da letra.
    Inflate,
    /// `calc_sphere_filter` — puxa cada vértice para a **esfera unitária** de
    /// raio 1 centrada na origem do objeto.
    ///
    /// ⚠️ **A força é o VALOR ABSOLUTO do fator** (`calc_sphere_translations`:
    /// `midpoint(normalize(p), −p) × abs(factors[i])`), então **arrastar para
    /// qualquer lado esferiza** — a lei não tem inverso, e a referência a
    /// escreve assim de propósito: *"desesferizar"* não é uma operação (não há
    /// para onde voltar sem lembrar de onde se veio).
    ///
    /// ⚠️ **O `midpoint` é METADE do caminho, não o caminho inteiro:** com
    /// `|f| = 1` o vértice anda `(p̂ − p)/2`, ou seja pousa no MEIO entre onde
    /// estava e a esfera. Um arrasto chega lá; um arrasto por gesto, não.
    Sphere,
    /// `calc_random_filter` — `t = orig_normals × f × (hash(p, seed) − 0,5)`.
    ///
    /// ⚠️ **O deslocamento é ao longo da NORMAL, não numa direção sorteada** —
    /// a referência sorteia a *magnitude* e mantém a direção, e é isso que
    /// produz *rugosidade* em vez de *nuvem de pontos*.
    ///
    /// ⚠️ **DIVERGÊNCIA DECLARADA no HASH, e ela é estrutural:** o
    /// `randomize_factors` chama `BLI_hash_int_2d`, **cuja definição não existe
    /// neste clone da referência** (medido: `grep -rl BLI_hash_int_2d` sobre
    /// `source/` devolve **um** arquivo, o que a USA). Não há como portar o que
    /// não se pode ler, então o hash é o [`crate::alpha`] desta crate — o mesmo
    /// que os alphas procedurais já usam, porque *uma crate, um hash*. O que se
    /// perde é paridade de BITS com o Blender; o que se mantém são as quatro
    /// propriedades que fazem de um ruído um ruído, e elas são gateadas:
    /// determinismo, faixa `[−0,5, 0,5)`, estabilidade ao longo do arrasto e
    /// descorrelação entre vizinhos.
    Random,
    /// `calc_relax_filter` — a mesma média, com a componente normal REMOVIDA.
    Relax,
    /// `calc_surface_smooth_filter` — o HC (Vollmer et al.), que devolve parte
    /// do que o laplaciano tirou.
    SurfaceSmooth,
    /// `calc_enhance_details_filter` — **o MESMO kernel do [`Self::Smooth`],
    /// com o sinal trocado e SEM teto**.
    ///
    /// ⚠️ **Ela nao e' lei nova, e isso foi MEDIDO antes de existir** (a sonda
    /// `tests/measure_sharpen_filter.rs`): contra a lei da referencia escrita a`
    /// mao, o nosso [`Self::Smooth`] em forca NEGATIVA concorda a **1,2e-7 a`
    /// 2,4e-7** — um a dois ULP de `f32` (`2^-23 = 1,2e-7`) — em toda a faixa
    /// `0,10..1,00`, com o CONTROLE em forca zero a dar `0,000e0`. *Duas
    /// expressoes da mesma lei, nao dois modelos.*
    ///
    /// ⚠️ **Entao o que ela ACRESCENTA e' exactamente o TETO, e ele e'
    /// ALCANCAVEL.** O `calc_smooth_filter` chama `clamp_factors(factors, -1,
    /// 1)` (`sculpt_filter_mesh.cc:375`) e o `calc_enhance_details_filter`
    /// **nao passa pelo `clamp_factors`** (conferido no fonte, `:1885-1925`: a
    /// cadeia e' `fill_factor_from_hide_and_mask` -> `auto_mask::calc_vert_factors`
    /// -> `scale_factors` -> `gather_data_mesh(detail_directions)` ->
    /// `scale_translations` -> `zero_disabled_axis_components` ->
    /// `clip_and_lock_translations`). Medido nas forcas 1,5 / 2,0 / 3,0: o
    /// [`Self::Smooth`] fica preso em **0,072617** e a referencia alcanca
    /// **0,108926 / 0,145235 / 0,217852**.
    ///
    /// ⚠️ **E o teto e' alcancavel porque o ARRASTO nao tem clamp proprio:**
    /// [`crate::FILTER_DRAG_PER_PX`] vale `0,001`, entao 1000 px de arrasto sao
    /// forca 1 e, dali para a frente, o kernel satura em silencio — *o filtro
    /// para de responder*. Sem esta lei, `|f| > 1` e' um gesto que o artista faz
    /// e que nada honra.
    ///
    /// ⚠️ **NAO se alarga a faixa do [`Self::Smooth`] em vez disto**, e a razao
    /// e' fidelidade: a referencia clampa o `SMOOTH` dela, e alargar o nosso
    /// seria a nossa lei a vestir o nome dela. As duas leis existem **na
    /// referencia** precisamente para separar *o alisamento com teto* de *o
    /// realce sem teto*.
    ///
    /// ⚠️ **O SINAL e' o da referencia** (`t = detail_directions x -strength`):
    /// arrastar para a DIREITA realca, ao contrario do [`Self::Smooth`], que
    /// alisa. Os dois chips respondem em sentidos opostos ao mesmo gesto, e e'
    /// assim no Blender — cada um e' uma entrada propria do
    /// `prop_mesh_filter_types`.
    EnhanceDetails,
    /// `calc_sharpen_filter` — **a única lei desta família que nenhum outro
    /// verbo exprime**, e a única com PRÉ-PASSE: o deslocamento de um vértice é
    /// pesado pela curvatura dos VIZINHOS, então nenhum pode ser escrito antes
    /// de todos serem medidos.
    ///
    /// ⚠️ **Ela não desloca uma feição, ela muda o CONTRASTE entre a feição e a
    /// vizinhança** — onde há detalhe o vértice alisa, onde não há ele é puxado
    /// pelos que têm. É por isso que ela não é exprimível compondo as outras
    /// oito: nenhuma delas lê a curvatura de um vizinho.
    ///
    /// ⚠️ **É a única cuja força é FATIADA em iterações**, porque a referência
    /// a clampa em `0,5` com o motivo escrito num comentário (*"needs multiple
    /// iterations to reach a stable state"*) — e porque a lei DELA depende da
    /// taxa de polling do rato, que esta casa recusa. O mecanismo inteiro está
    /// no cabeçalho do `stroke_filter_sharpen.rs`.
    Sharpen,
}

/// **O TETO DA FORÇA TOTAL DO SHARPEN — `4,0`, e o recurso é a CONVERGÊNCIA.**
///
/// ⚠️ **Ele NÃO é o `0,5` da referência:** aquele limita uma ITERAÇÃO (e
/// sobrevive intacto como o teto de cada fatia — ver `MAX_STEP`), este limita a
/// força TOTAL que o arrasto pede.
///
/// ⚠️ **A PRIMEIRA tabela que esteve aqui MEDIU-SE A SI MESMA, e a auditoria
/// pegou-a.** Ela dizia que a lei *"satura em 4,0 — de 4 para cima os números
/// são idênticos a todos os dígitos"* e nomeava o recurso como *"a
/// CONVERGÊNCIA"*. Mas o [`crate::SculptStroke::filter_sharpen`] **clampa a
/// entrada por este teto antes de qualquer aritmética**, então toda leitura
/// acima dele via o CLAMP e não a lei: a identidade era o teto a devolver o
/// mesmo número para qualquer entrada maior. *O recurso nomeado não existia.*
/// Ela citava ainda a sonda e a fixture erradas — os números eram de outra malha
/// que a atribuição não mencionava.
///
/// ⚠️ **Medido pela porta que NÃO clampa** (`sharpen_total_for_measurement`,
/// `tests/measure_sharpen_intensify.rs`, malha do produto, 98306 vértices):
/// a lei **não satura** — o degrau sobe `0,990× → 1,398×` de força 1 a 64,
/// monotónico e sem joelho, com a excursão a quase **dobrar** de 4 para 8.
///
/// ⚠️ **O RECURSO REAL É TEMPO, e é ele que fixa o número:** uma fatia percorre
/// a malha INTEIRA (pré-passe + gather), e o filtro corre **a cada evento de
/// ponteiro**:
///
/// | força | fatias | tempo |
/// |---|---|---|
/// | 0,5 | 1 | 7,72 ms |
/// | 2,0 | 4 | 11,47 ms |
/// | **4,0** | **8** | **17,17 ms** |
/// | 8,0 | 16 | 27,93 ms |
/// | 16,0 | 32 | 49,16 ms |
///
/// ⚠️ **Um quadro de 60 fps são 16,7 ms**, então o teto de hoje já está **no
/// limite** — e é por isso que ele não sobe sem uma wave de custo. A tabela
/// antiga, medida na fixture de 3010 vértices, escondia isto por completo.
///
/// | força | excursão | degrau (razão) |
/// |---|---|---|
/// | 1,0 | 0,00154 | 0,990× |
/// | 2,0 | 0,00303 | 1,003× |
/// | 3,0 | 0,00446 | 1,021× |
/// | **4,0** | **0,00585** | **1,046×** |
/// | 6,0 | 0,00585 | 1,046× |
/// | 8,0 | 0,00585 | 1,046× |
///
/// ⚠️ **⏸️ E O NÚMERO É DECISÃO DO ENIO, com a tabela acima na mão:** subir o
/// teto compra excursão real (a lei não satura) e **paga em milissegundos por
/// evento de ponteiro**. Não é uma escolha de engenharia — é *quanto arrasto um
/// gesto vale*, e a cura barata do outro lado (cortar o custo por fatia) é uma
/// wave própria.
///
/// ⚠️ **A PRIMEIRA tabela que escrevi aqui era de uma lei ERRADA**, e ficou uma
/// wave inteira no arquivo: ela media o `sharpen_factor` recomputado a cada
/// sub-passo, e nesse regime a lei ALISA (o degrau caía a `0,667×`). Com o
/// factor congelado — como no `filter_cache` da referência — ele **sobe**, que é
/// o que a coluna acima mostra. *Uma tabela medida sobre um produto que mudou é
/// pior que tabela nenhuma: ela justifica o número com o fenómeno errado.*
///
/// ⚠️ **E a divergência mora no `MAX_STEP` mais a guarda de valência**, não
/// aqui: sem a guarda esta mesma varredura numa esfera UV chega a **1098×** de
/// degrau e a uma excursão de **33** num raio um.
const SHARPEN_MAX: f32 = 4.0;

impl FilterKind {
    /// **A lista que o artista escolhe**, na ordem da referência.
    ///
    /// ⚠️ **Esta é a fonte da UI, e a contagem NÃO é citada em prosa nenhuma** —
    /// o `Verb::ALL` já pagou essa lição duas vezes em dois parágrafos vizinhos
    /// do plano.
    pub const ALL: [Self; 9] = [
        Self::Smooth,
        Self::Scale,
        Self::Inflate,
        Self::Sphere,
        Self::Random,
        Self::Relax,
        Self::SurfaceSmooth,
        Self::EnhanceDetails,
        Self::Sharpen,
    ];

    /// O rótulo que o painel mostra — os nomes do `prop_mesh_filter_types`.
    ///
    /// ⚠️ Eles saem **daqui**, nunca de uma tabela paralela no painel: é a mesma
    /// regra do [`Verb::label`] e do [`crate::TransformKind::label`].
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Smooth => "Smooth",
            Self::Scale => "Scale",
            Self::Inflate => "Inflate",
            Self::Sphere => "Sphere",
            Self::Random => "Random",
            Self::Relax => "Relax",
            Self::SurfaceSmooth => "Surface Smooth",
            Self::EnhanceDetails => "Enhance Details",
            Self::Sharpen => "Sharpen",
        }
    }

    /// **A FAIXA do fator** — `clamp(máscara × arrasto, lo, hi)`, na ordem da
    /// referência (`scale_factors` e depois `clamp_factors`).
    ///
    /// ⚠️ **Ela é propriedade da LEI, não do verbo que a semeia**, e é por isso
    /// que mora aqui: ela sai do `clamp_factors` que cada `calc_*_filter` chama
    /// (ou não chama), e uma lei sem verbo tem faixa do mesmo jeito.
    #[must_use]
    pub fn range(self) -> (f32, f32) {
        match self {
            // `clamp_factors(factors, -1.0f, 1.0f)` — `sculpt_filter_mesh.cc:375`.
            Self::Smooth => (-1.0, 1.0),
            // ⚠️ **SEM clamp, e a ausência é da referência** (nem
            // `calc_inflate_filter` nem `calc_scale_filter` nem
            // `calc_random_filter` chamam `clamp_factors`): o deslocamento é
            // `strength` em unidades de OBJETO, e um teto aqui seria um número
            // que ninguém mediu. Quem calibra é a escala do ARRASTO, que é
            // nossa — ver [`crate::FILTER_DRAG_PER_PX`].
            Self::Inflate | Self::Scale | Self::Random => (f32::MIN, f32::MAX),
            // ⚠️ **O Sphere também não clampa na referência**, e não precisa: a
            // lei toma `abs(f)` e o alvo é uma posição FIXA (a esfera unitária),
            // então força grande satura em vez de divergir — ao contrário do
            // Inflate, que anda sem teto.
            Self::Sphere => (f32::MIN, f32::MAX),
            // `clamp_factors(factors, 0.0f, 1.0f)` — `:1019` e `:1358`.
            //
            // ⚠️ **Arrastar para o lado errado não faz NADA nestes dois, e é a
            // lei da referência:** um relax negativo não é *"desrelaxar"* — não
            // existe a operação inversa de redistribuir —, e o HC negativo
            // amplificaria o próprio erro que ele existe para devolver.
            Self::Relax | Self::SurfaceSmooth => (0.0, 1.0),
            // ⚠️ **O teto NÃO é o `0,5` da referência, e a diferença é a wave.**
            // Aquele número é `clamp_factors(factors, 0.0f, 0.5f)` (`:1662`) e
            // ele limita **uma ITERAÇÃO**, com o motivo escrito ao lado dele no
            // fonte (*"needs multiple iterations to reach a stable state"*); a
            // referência alcança forças maiores acumulando um passo por evento
            // de rato. Nós entregamos a mesma força em sub-passos
            // determinísticos (ver `steps_for`), então o teto aqui é o da FORÇA
            // TOTAL e o `0,5` sobrevive intacto como o teto de cada fatia.
            //
            // ⚠️ **E arrastar para trás não faz NADA, como nos dois acima:** o
            // `clamp` da referência começa em ZERO, e um *desafiar* não existe
            // — a lei mede contraste e o inverso dela seria outro kernel.
            Self::Sharpen => (0.0, SHARPEN_MAX),
            // ⚠️ **SEM teto, e aqui a ausencia e' a FEATURE inteira** — ao
            // contrario dos tres acima, esta lei tem uma irma CLAMPADA que roda
            // o mesmo kernel (o [`Self::Smooth`]). O `calc_enhance_details_filter`
            // nao chama `clamp_factors` (`:1885-1925`), e e' so' isso que o
            // separa do `calc_smooth_filter` negado. Ver o doc do variant.
            Self::EnhanceDetails => (f32::MIN, f32::MAX),
        }
    }
}

impl Verb {
    /// **Que lei este verbo SEMEIA quando o filtro é armado?**
    ///
    /// ⚠️ **É uma SEMENTE, não a porta de escolha.** Quem o artista escolhe é
    /// um [`FilterKind`] da lista inteira; este mapa existe para o interruptor
    /// nascer no lugar que o verbo na mão sugere, do mesmo jeito que o
    /// `arm_verb_defaults` semeia um pincel. Duas portas para escolher a mesma
    /// lei seriam o defeito que o próprio [`FilterKind`] existe para evitar.
    ///
    /// ⚠️ **A propriedade que a lista exprime é MEDÍVEL, não uma opinião:** o
    /// alvo destes quatro é função do vértice, do `pre` congelado e do anel, e
    /// de **mais nada** — nenhum deles lê o centro do dab, a normal de área, o
    /// plano ajustado à pegada, o caminho do traço ou uma âncora. O gate
    /// `the_verb_in_hand_does_not_change_the_law_a_filter_runs` o afirma pela
    /// porta do produto, varrendo `FilterKind::ALL × Verb::ALL` com `assert_eq!`
    /// byte a byte.
    ///
    /// ⚠️ **Esta linha citava um `a_filtering_verb_reads_nothing_from_the_dab`
    /// que NUNCA existiu** — `grep` na árvore devolve só a própria citação, e
    /// `git log -S` só o commit que a escreveu. A propriedade **está** defendida,
    /// por um gate mais forte que o nome prometido; e o cenário que o nome
    /// descrevia é hoje inexprimível **por tipo** (as leis de filtro não recebem
    /// um `Dab`).
    ///
    /// ⚠️ **WHITELIST e não blacklist**, pelo motivo que o
    /// [`Self::honours_invert`] já pagou uma vez: derivada por negação
    /// (`!uses_plane() && !anchors() && !paints_mask()`) ela deixaria de fora
    /// exactamente os verbos que o `uses_plane` **não** nomeia e que mesmo assim
    /// leem o dab — o [`Verb::Draw`] e o [`Verb::ClayStrips`] leem a normal de
    /// ÁREA, o [`Verb::ClayThumb`] lê o caminho, os quatro do
    /// [`crate::kelvinlet`] leem o centro. Um verbo novo nasceria reivindicando
    /// que filtra a malha, **em silêncio**, e o filtro o rodaria com um dab que
    /// não existe.
    ///
    /// ⚠️ **ESTE PARÁGRAFO ESTÁ RETRATADO — o [`Verb::Sharpen`] ESTÁ na tabela**
    /// (o `match` abaixo semeia-o em [`FilterKind::EnhanceDetails`]). A razão da
    /// retratação está no gate que a mediu
    /// (`the_seed_table_has_content_and_no_two_verbs_seed_the_same_law`): *duas
    /// portas para o MESMO deslocamento é o que se recusa; duas portas para
    /// conjuntos DIFERENTES é o que a referência tem.* O texto abaixo é o mundo
    /// pré-W9c-a e fica como registo do que caiu.
    ///
    /// ⚠️ ~~O [`Verb::Sharpen`] fica de FORA, e a exclusão é medida.~~ Ele
    /// passa na propriedade acima (só lê o anel), mas o alvo dele é
    /// `live + (live − média)·w`, que **é** o `target_smooth` com o sinal do
    /// peso trocado — e num filtro o sinal vem do ARRASTO. Semeá-lo apontaria
    /// para o mesmo [`FilterKind::Smooth`] que o Smooth semeia, o que é
    /// inofensivo mas inútil: quem quer afiar arrasta para o outro lado. No
    /// pincel ele é um chip próprio com razão: ali o gesto **não tem sinal** (o
    /// `Ctrl` é a única fonte, e o [`Self::honours_invert`] não nomeia nem
    /// Smooth nem Sharpen), então sem o chip a lei seria inalcançável.
    ///
    /// ⚠️ **O [`Verb::Layer`] fica de fora por MEDIÇÃO, não por herança.** O
    /// irmão 2-D deste módulo (o *Filter Layer* do Painter) o recusou porque
    /// filtrar uma camada com ele é uma **translação uniforme** de um campo de
    /// altura, e a luz de lá lê `∇h` — não moveria um pixel. Aqui a razão **não
    /// transfere**: a nossa normal é por-vértice, então `base + n·h` sobre a
    /// malha inteira é precisamente o [`FilterKind::Inflate`], com um degrau de
    /// saturação a mais. A recusa fica de pé pela OUTRA razão — semeá-lo
    /// apontaria para uma lei que o Inflate já semeia.
    #[must_use]
    pub fn filter_kind(self) -> Option<FilterKind> {
        Some(match self {
            Self::Smooth => FilterKind::Smooth,
            Self::Inflate => FilterKind::Inflate,
            Self::SlideRelax => FilterKind::Relax,
            Self::SurfaceSmooth => FilterKind::SurfaceSmooth,
            // ⚠️ **O verbo que ja' rodava esta lei.** O
            // [`crate::SculptStroke::target_sharpen`] e' o kernel deste pincel
            // desde que ele nasceu, e e' *a mesma expressao* do
            // `calc_enhance_details_filter`; ate' a
            // [`FilterKind::EnhanceDetails`] existir, ele era o unico verbo cuja
            // lei nao tinha filtro correspondente.
            Self::Sharpen => FilterKind::EnhanceDetails,
            _ => return None,
        })
    }

    /// **Este verbo semeia um filtro?** — uma leitura de [`Self::filter_kind`]
    /// em vez de um segundo predicado, o mesmo corte que [`Self::anchors`] faz
    /// sobre o [`Self::grip`].
    #[must_use]
    pub fn filters_mesh(self) -> bool {
        self.filter_kind().is_some()
    }
}
