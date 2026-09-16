//! ⭐⭐⭐ **A MALHA REFINA-SE NA HORA DE DESENHAR, contra uma TOLERÂNCIA EM PIXELS.**
//!
//! # O report que a trouxe (dono, 2026-09-10, com foto e três setas)
//!
//! > *«malha bem desenhada. Contudo não é a solução perfeita em termos de deformação pois ao dobrar
//! > a articulação temos arestas retas na imagem. Estude um algoritmo com opção de um tipo de smooth
//! > na imagem e coloque como alternativa»*
//!
//! # ⭐⭐⭐ A lei, e a razão de ela existir
//!
//! **A malha NÃO é a deformação — ela é uma AMOSTRAGEM dela.** O campo
//! `Φ(p) = Σ wᵢ(p)·Mᵢ·p` está definido em **todo** ponto da imagem, porque os pesos são
//! **derivados** e não guardados. O que produz a aresta reta é o DESENHO: cada triângulo é pintado
//! com **um afim**, que é a aproximação de 1.ª ordem de um campo curvo.
//!
//! ⇒ O desvio entre a silhueta desenhada e o campo verdadeiro é **`O(h²)`**, e MEDIU-SE
//! (cápsula do smoke, cadeia de 3 ossos, silhueta analítica de 2000 pontos):
//!
//! | grelha | triângulos | desvio p99 a 60° | a 105° | a 150° |
//! |---|---:|---:|---:|---:|
//! | a de hoje | `216` | `3,85 px` | `6,85 px` | **`9,84 px`** |
//! | 2× mais fina | `768` | `1,09` | `1,98` | `2,79` |
//! | 4× mais fina | `3 086` | `0,29` | `0,54` | **`0,78`** |
//! | 8× mais fina | `11 472` | `0,07` | `0,13` | `0,19` |
//!
//! ⛔⛔ **E o erro NÃO mora nas articulações**, que era a premissa da grelha graduada: com as juntas
//! em `x = 0 · 107 · 213 · 320`, o pior desvio cai em **`x = 119 · 190 · 266`**. Apertar a banda à
//! volta das juntas leva `3,99 px` a `3,39 px` e mais nada — *a densidade que a DOBRA precisa e a
//! densidade que o DESENHO precisa não estão no mesmo sítio.* O erro vive onde os **pesos** variam
//! depressa, e com `raio = comprimento do osso` isso é o membro inteiro.
//!
//! # ⚠️ Porque isto é feito a DESENHAR e não a PRENDER
//!
//! No instante do *bind* a pose é a de repouso: **não há dobra nenhuma**, logo não há erro para
//! medir. A densidade necessária é função da POSE, que só existe no quadro. *Uma malha escolhida no
//! bind é escolhida antes de a pergunta ser feita.* ⭐ E a malha guardada fica pequena — o save e o
//! undo continuam a fotografar a mesma coisa.
//!
//! # ⚠️ A tolerância é em pixels de ECRÃ, e é por isso que ela precisa da câmara
//!
//! Um desvio de meia unidade local é meio pixel a zoom `1` e **quatro** pixels a zoom `8`. *A
//! suavidade que o olho vê é um facto de espaço de ECRÃ*, então quem chama converte a tolerância
//! pela escala da câmara antes de entrar aqui.
//!
//! # ⭐⭐⭐ A conformidade é EXACTA, e não uma tolerância
//!
//! O refinamento **uniforme** usa o mesmo `k` para toda a malha, e cada ponto novo é nomeado
//! pela **aresta canónica** que o gera (`o vértice de índice menor primeiro`) — logo os dois
//! triângulos que partilham uma aresta calculam o ponto do meio dela a partir da **mesma expressão,
//! na mesma ordem**, e obtêm os **mesmos bits**.
//!
//! ⛔⛔⛔ **E ESTE CABEÇALHO AFIRMAVA, ATÉ 2026-09-16, QUE UM `k` POR TRIÂNGULO ERA IMPOSSÍVEL**
//! (*«abriria nós pendurados, que é a fenda que a `grid.rs` recusa por escrito»*). A afirmação
//! estava certa sobre **uma** maneira de o fazer — dar a cada triângulo a sua própria grelha
//! baricêntrica — e foi lida como se fosse sobre a pergunta inteira. ⭐ A
//! [`crate::refine_adaptive`] faz `k` por triângulo **sem nó pendurado nenhum**, porque a operação
//! dela não é *«partir um triângulo»* e sim *«partir uma ARESTA»*: os dois donos da aresta partem-se
//! **ao mesmo tempo**, e um ponto no meio de uma aresta que deixou de existir não fica pendurado em
//! coisa nenhuma.
//!
//! ⚠️ **O preço da afirmação errada foi o `Smooth` INERTE:** o `k` global tem tecto
//! `⌊√(orçamento/peças)⌋` ([`max_split`]), logo toda malha acima de `orçamento/4` peças só admite
//! `k = 1` — e o botão do painel prometia uma coisa que a aritmética proibia. Ver [`RefineLaw`].

use crate::Mesh2d;

/// ⭐ **O CAMPO, com os atributos do ponto na mão** — `deform(ponto_de_repouso, atributos)`.
///
/// ⚠️ A fatia é **vazia** quando não há atributos, e é essa a forma que faz o caminho sem carga ser
/// o de sempre: quem não os quer escreve `|q, _| …` e não paga nada.
pub type DeformAttrs<'a> = dyn FnMut([f64; 2], &[f64]) -> [f64; 2] + 'a;

/// ⭐ **Os números do refinamento.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RefineOptions {
    /// Quanto a silhueta desenhada pode afastar-se do campo verdadeiro, **em pixels de ecrã**.
    pub tolerance_px: f64,
    /// ⛔⛔⛔ **O TECTO É DO NÚMERO DE PEÇAS, e não do `k`** — e a diferença foi um report do dono
    /// (*«Smooth bugado quebrando a forma»*, 2026-09-10, com foto).
    ///
    /// ⛔⛔ **A 1.ª redacção deste doc atribuía o limite à CAMADA DE RECORTE do Vello, e a medição
    /// de 2026-09-13 refutou-a:** o que partia a imagem era o ATLAS — a pele desenhava cada peça
    /// pela porta crua, que guardava uma cópia inteira da imagem por peça (`7 776` peças perdiam
    /// `5 651`). Curado isso, o recurso que sobra é o buffer FIXO de informação por desenho do
    /// Vello, que é do QUADRO inteiro — e quem conhece o quadro é o chamador: o produto passa aqui a
    /// PARTE desta malha do orçamento do quadro (`ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES`).
    /// Um tecto no `k` não é um tecto nesse recurso: ele é **quadrático** nele, e a malha de partida
    /// pode ter qualquer tamanho.
    ///
    /// ⚠️⚠️ **A experiência que o dono correu sem saber é a prova:** com o braço quase RECTO (`2°`)
    /// o desvio já é `0,499 px`, logo o `k` saltava para o tecto e desenhava **7 776** recortes —
    /// *a mesma geometria que o `Fast` desenha em 216, e partida*. A malha estava provadamente
    /// correcta: área conservada ao cêntimo, zero triângulos saltados, zero arestas com mais de
    /// dois donos.
    ///
    /// ⭐ **E o custo está MEDIDO dos dois lados** (2026-09-13, W4 do plano `docs/Skeleton/03`;
    /// `load 3,7`–`3,9`, o MÍNIMO de 40/60 corridas — acima de `load ~5` uma leitura de relógio
    /// desta workstation não vale nada). Por peça ENTREGUE, num quadro com `Smooth`: **`1,08 µs`**
    /// (descodificar a malha `0,134` · deformar e montar `0,200` no `Fast` · recolher, costurar,
    /// enviar e desenhar `0,039`). ⇒ o tecto do produto é uma FATIA do quadro dividida por esse
    /// número: `ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES`.
    ///
    /// ⛔ A tabela anterior (`deformar`/`encodar`, até `10`–`16 %` a `7 776` peças) media o caminho
    /// do **Vello**, que a W2 retirou.
    pub max_pieces: usize,
    /// ⭐⭐⭐ **QUAL DAS DUAS LEIS** — `true` (o padrão) parte só os triângulos que a dobra pede.
    ///
    /// ⚠️ **Ele existe para BISSECAR, não para escolher gosto:** a lei uniforme é
    /// provadamente inerte acima de `max_pieces / 4` peças (ver o cabeçalho e [`max_split`]), e o
    /// produto lê-o de `PH2D_SKIN_REFINE=uniforme`. *Um caminho antigo sem porta não se mede
    /// contra o novo.*
    pub adaptativo: bool,
}

/// ⭐ **QUAL LEI CORREU, e o que ela usou como grandeza de trabalho** — a metade do
/// [`RefineReport`] que só faz sentido dentro de uma lei.
///
/// ⛔ **Um `k = 0` para dizer «não houve `k`» seria um número a mentir**: o `0` é um valor legal do
/// tipo, e todo leitor teria de saber que naquele caso ele não quer dizer *zero partes*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefineLaw {
    /// **Uniforme:** cada triângulo partido numa grelha baricêntrica de `k × k`.
    Uniform {
        /// `1` = não refinou (o caminho de omissão, byte-idêntico à malha de entrada).
        k: u32,
    },
    /// **Adaptativo:** `rondas` bissecções da aresta mais longa, escolhidas pelo pior desvio.
    Adaptive {
        /// Quantas vezes o laço escolheu um triângulo e o partiu (⚠️ **não** é o número de arestas
        /// partidas: a propagação de conformidade parte mais do que uma por ronda).
        rondas: usize,
    },
}

/// ⭐⭐ **O QUE O REFINAMENTO FEZ** — a resposta que o produto imprime no diagnóstico.
///
/// ⚠️⚠️ **`desvio` é `Option` de propósito, e é a parte honesta deste tipo.** A lei uniforme mede o
/// desvio da malha que construiu **só quando não precisa de corrigir o `k`** — depois da correcção
/// ela não volta a medir, porque isso seria uma travessia inteira da malha por quadro para um
/// número que ninguém consome no desenho. *Um campo que devolvesse o valor PRÉ-correcção seria um
/// número que mente exactamente no caso que interessa*, e `None` diz o que é: **não medido**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RefineReport {
    /// Quantas peças a malha devolvida tem.
    pub pecas: usize,
    /// O pior desvio ao campo que FICOU, em unidades de entrada — `None` = não medido (ver acima).
    pub desvio: Option<f64>,
    /// ⛔ **O orçamento parou o refinamento antes da tolerância.** É esta a linha que separa
    /// *«a dobra não pediu nada»* de *«o quadro não paga»*, e o produto imprime-a.
    pub travado_pelo_orcamento: bool,
    /// Qual lei correu — ver [`RefineLaw`].
    pub lei: RefineLaw,
}

impl RefineReport {
    /// O `k` da lei uniforme; `None` na adaptativa (lá não há `k`).
    #[must_use]
    pub fn k(&self) -> Option<u32> {
        match self.lei {
            RefineLaw::Uniform { k } => Some(k),
            RefineLaw::Adaptive { .. } => None,
        }
    }
}

impl Default for RefineOptions {
    fn default() -> Self {
        Self {
            // Meio pixel: abaixo disto o anti-aliasing da própria arte é mais largo que o erro.
            tolerance_px: 0.5,
            // ⚠️ **O neutro desta folha, NÃO o tecto do produto:** a folha não sabe o que é um
            // quadro nem um renderer. O produto substitui-o pela parte da malha no orçamento do
            // QUADRO (`ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES`, hoje derivado do TEMPO
            // do quadro com o custo por peça MEDIDO — W4 do plano `docs/Skeleton/03`).
            max_pieces: 1024,
            // ⭐ A lei que de facto refina. A uniforme fica alcançável para bissecar.
            adaptativo: true,
        }
    }
}

/// Quem é um vértice da malha refinada — a chave que faz dois triângulos vizinhos concordarem.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum No {
    /// Um vértice da malha original.
    Canto(u32),
    /// O passo `s` de `k` sobre a aresta `(menor, maior)`, contado **a partir do menor**.
    Aresta(u32, u32, u32),
    /// Um ponto do miolo do triângulo `t`, na grelha baricêntrica `(i, j)`.
    Miolo(u32, u32, u32),
}

/// **Quanto o afim de cada triângulo erra o campo** — o maior desvio nos meios das arestas.
///
/// ⚠️ Os meios das arestas são o sítio certo para perguntar: é onde um interpolador linear erra
/// mais, e é onde o refinamento vai pôr o primeiro ponto novo.
#[must_use]
pub fn deviation(
    mesh: &Mesh2d,
    posed: &[[f64; 2]],
    deform: &mut dyn FnMut([f64; 2]) -> [f64; 2],
) -> f64 {
    deviation_attrs(mesh, posed, &[], 0, &mut |p, _| deform(p))
}

/// [`deviation`] com os **atributos por vértice** na mão — ver [`refine_posed_attrs`].
///
/// ⚠️ **No meio de uma aresta o atributo é a MÉDIA das pontas**, que é exactamente o que o
/// refinamento lá vai pôr ([`attrs_canonicos`]). *Medir o desvio com um atributo que a subdivisão
/// não vai produzir mede outro campo* — e o `k` sairia calibrado para uma malha que ninguém desenha.
#[must_use]
pub fn deviation_attrs(
    mesh: &Mesh2d,
    posed: &[[f64; 2]],
    attrs: &[f64],
    stride: usize,
    deform: &mut DeformAttrs<'_>,
) -> f64 {
    let mut pior = 0.0_f64;
    let mut meio_attrs = vec![0.0; stride];
    for t in &mesh.tris {
        for k in 0..3 {
            let (i, j) = (t[k] as usize, t[(k + 1) % 3] as usize);
            let (Some(&ra), Some(&rb)) = (mesh.rest.get(i), mesh.rest.get(j)) else {
                continue;
            };
            let (Some(&pa), Some(&pb)) = (posed.get(i), posed.get(j)) else {
                continue;
            };
            for (c, v) in meio_attrs.iter_mut().enumerate() {
                *v = f64::midpoint(
                    attrs.get(i * stride + c).copied().unwrap_or(0.0),
                    attrs.get(j * stride + c).copied().unwrap_or(0.0),
                );
            }
            let meio = deform([(ra[0] + rb[0]) / 2.0, (ra[1] + rb[1]) / 2.0], &meio_attrs);
            let reta = [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0];
            pior = pior.max((meio[0] - reta[0]).hypot(meio[1] - reta[1]));
        }
    }
    pior
}

/// ⭐ **Quantas vezes partir cada aresta** para o desvio caber na tolerância.
///
/// ⚠️ **A raiz quadrada é a lei `O(h²)`, medida** (a tabela no cabeçalho): partir ao meio divide o
/// desvio por `~3,6`, não por `2`. Um `k` linear no desvio pediria triângulos a mais.
#[must_use]
pub fn splits_for(desvio: f64, pecas: usize, opts: RefineOptions) -> u32 {
    let tol = opts.tolerance_px.max(f64::MIN_POSITIVE);
    if !desvio.is_finite() || desvio <= tol {
        return 1;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a raiz de uma razão finita e positiva, limitada logo a seguir pelo orçamento"
    )]
    let k = (desvio / tol).sqrt().ceil() as u32;
    k.clamp(1, max_split(pecas, opts))
}

/// ⭐ **Quantas partes o ORÇAMENTO ainda paga** — `k` tal que `peças · k² <= max_pieces`.
///
/// ⚠️ É aqui que o tecto deixa de ser um número escolhido e passa a ser uma **divisão**: uma malha
/// de `216` triângulos com orçamento `1024` pode ir a `k = 2`; uma de `50` pode ir a `k = 4`. *Um
/// tecto no `k` daria à segunda o mesmo direito que à primeira, e é a CONTAGEM que o renderer paga.*
#[must_use]
pub fn max_split(pecas: usize, opts: RefineOptions) -> u32 {
    if pecas == 0 {
        return 1;
    }
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "contagens de triângulos de uma malha de imagem, muito abaixo do limite exacto do f64"
    )]
    let k = ((opts.max_pieces as f64) / (pecas as f64)).sqrt() as u32;
    k.max(1)
}

/// ⭐⭐⭐ **A MALHA POSADA, refinada até à tolerância** — devolve `(malha, posições)`.
///
/// `deform` é o campo: ele leva um ponto de **repouso** (pixels da imagem) para onde ele está
/// **agora**. Ele é chamado uma vez por vértice novo, e ⛔ nunca duas vezes para o mesmo ponto —
/// é a chave [`No`] que o garante.
///
/// Sem refinamento devolve a malha original e as posições dela, **ao bit**: é o caminho de omissão,
/// e ele tem de ser byte-idêntico ao que o desenho já fazia.
#[must_use]
pub fn refine_posed(
    mesh: &Mesh2d,
    deform: &mut dyn FnMut([f64; 2]) -> [f64; 2],
    opts: RefineOptions,
) -> (Mesh2d, Vec<[f64; 2]>, RefineReport) {
    let (m, p, _, r) = refine_posed_attrs(mesh, &[], 0, &mut |q, _| deform(q), opts);
    (m, p, r)
}

/// ⭐⭐⭐ **A PORTA DO REFINAMENTO — ela ESCOLHE a lei, e é a única que o produto chama.**
///
/// ⛔⛔ **Ela não é um atalho: é o sítio onde o `opts.adaptativo` é lido.** Com as duas leis
/// exportadas e nenhum despachante, o chamador escolheria a função e o campo das opções ficaria a
/// ser um knob que ninguém lê — *a espécie de controlo morto que o `CLAUDE.md` §5.0 chama de
/// «consumidor que projecta o valor fora»*.
///
/// `attrs` é achatado: `attrs[v * stride + c]` é a componente `c` do vértice `v`; com `stride == 0`
/// não há atributos e o vector de saída sai vazio.
#[must_use]
pub fn refine_posed_attrs(
    mesh: &Mesh2d,
    attrs: &[f64],
    stride: usize,
    deform: &mut DeformAttrs<'_>,
    opts: RefineOptions,
) -> (Mesh2d, Vec<[f64; 2]>, Vec<f64>, RefineReport) {
    if opts.adaptativo {
        crate::refine_adaptive::refine_posed_adaptive(mesh, attrs, stride, deform, opts)
    } else {
        refine_posed_uniform(mesh, attrs, stride, deform, opts)
    }
}

/// ⭐⭐ **A LEI UNIFORME: o mesmo `k` para toda a malha** — ver o cabeçalho do módulo.
///
/// Cada vértice NOVO recebe o atributo **interpolado baricentricamente** dos vértices originais que
/// o geraram, e o campo passa a ser chamado com ele: `deform(ponto, atributos)`.
///
/// `attrs` é achatado: `attrs[v * stride + c]` é a componente `c` do vértice `v`. Cada vértice NOVO
/// recebe o atributo **interpolado baricentricamente** dos vértices originais que o geraram, e o
/// campo passa a ser chamado com ele: `deform(ponto, atributos)`.
///
/// ⭐⭐⭐ **Ela existe porque o padrão-ouro dos pesos NÃO é derivável de uma posição.** O *bump*
/// euclidiano era uma função do ponto, logo bastava dar-lhe o ponto; os *Bounded Biharmonic
/// Weights* são a solução de um problema **global** sobre a arte, resolvida uma vez ao prender e
/// **guardada nos vértices**. ⇒ um vértice que a subdivisão inventa não tem peso — ele tem de o
/// herdar, e a única resposta certa é a do triângulo que o gerou.
///
/// ⛔ **A localização do ponto NÃO serve.** Procurar a que triângulo pertence um ponto novo custa
/// `O(n)` por ponto e devolve a mesma resposta que a proveniência já sabe de graça — *a subdivisão
/// conhece o triângulo de origem porque foi ela que o partiu*.
///
/// ⚠️ **A conformidade dos atributos é a mesma dos pontos, e pela mesma razão:** um ponto de aresta
/// é nomeado pela [`No::Aresta`] canónica, então os dois triângulos vizinhos interpolam o atributo
/// **das mesmas duas pontas, na mesma ordem** — mesmos bits, logo nenhuma costura de peso.
///
/// ⛔ **O tecto dela é `⌊√(orçamento/peças)⌋`** ([`max_split`]), logo acima de `orçamento/4` peças
/// ela é **inerte** — é essa a razão de ela ter deixado de ser o caminho de omissão em 2026-09-16.
#[must_use]
pub fn refine_posed_uniform(
    mesh: &Mesh2d,
    attrs: &[f64],
    stride: usize,
    deform: &mut DeformAttrs<'_>,
    opts: RefineOptions,
) -> (Mesh2d, Vec<[f64; 2]>, Vec<f64>, RefineReport) {
    let posed: Vec<[f64; 2]> = mesh
        .rest
        .iter()
        .enumerate()
        .map(|(v, &p)| deform(p, fatia(attrs, stride, v)))
        .collect();
    let tecto = max_split(mesh.tris.len(), opts);
    let cru = deviation_attrs(mesh, &posed, attrs, stride, deform);
    let k = splits_for(cru, mesh.tris.len(), opts);
    if k <= 1 {
        let pecas = mesh.tris.len();
        return (
            mesh.clone(),
            posed,
            attrs.to_vec(),
            RefineReport {
                pecas,
                desvio: Some(cru),
                // ⭐ O `k` saturou no tecto do orçamento: a tolerância pedia mais e não há onde.
                travado_pelo_orcamento: tecto <= 1 && cru > opts.tolerance_px,
                lei: RefineLaw::Uniform { k: 1 },
            },
        );
    }
    let (r, p, a) = build(mesh, attrs, stride, deform, k);
    // ⭐⭐⭐ **O ESTIMADOR CONFERE O QUE ENTREGOU, e corrige UMA vez.**
    //
    // ⚠️⚠️ **Medido, e foi um gate vermelho que o exigiu:** a lei `O(h²)` descreve a tendência, não
    // o valor — com a tolerância a pedir `0,954 px` o `k` de um só passo entregava `0,980`. *Um
    // número que se chama tolerância e não é honrado é um número que mente ao artista.*
    //
    // ⚠️ **UMA correcção, nunca um laço até convergir:** a segunda estimativa parte da medição já
    // feita **na malha refinada** (`k · √(d/tol)`), que é a lei aplicada onde ela vale — e um laço
    // seria trabalho por quadro sem tecto, exactamente o que o orçamento existe para impedir.
    let d = deviation_attrs(&r, &p, &a, stride, deform);
    if d <= opts.tolerance_px || k >= tecto {
        let pecas = r.tris.len();
        return (
            r,
            p,
            a,
            RefineReport {
                pecas,
                desvio: Some(d),
                travado_pelo_orcamento: d > opts.tolerance_px,
                lei: RefineLaw::Uniform { k },
            },
        );
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a razão é finita e positiva, e o clamp logo abaixo é o tecto"
    )]
    let k2 = ((f64::from(k) * (d / opts.tolerance_px.max(f64::MIN_POSITIVE)).sqrt()).ceil() as u32)
        .clamp(k + 1, tecto.max(k + 1));
    let (r2, p2, a2) = build(mesh, attrs, stride, deform, k2);
    let pecas = r2.tris.len();
    (
        r2,
        p2,
        a2,
        RefineReport {
            pecas,
            // ⚠️ **NÃO medido**: uma segunda travessia da malha já refinada, por quadro, para um
            // número que só o diagnóstico lê. Ver o doc do [`RefineReport`].
            desvio: None,
            travado_pelo_orcamento: k2 >= tecto,
            lei: RefineLaw::Uniform { k: k2 },
        },
    )
}

/// Os atributos do vértice `v`. Vazio quando não há atributos — ⛔ **nunca** um índice fora da
/// fatia: uma tabela mais curta que a malha é um defeito do chamador, e `&[]` di-lo em vez de
/// entregar os pesos do vizinho.
pub(crate) fn fatia(attrs: &[f64], stride: usize, v: usize) -> &[f64] {
    if stride == 0 {
        return &[];
    }
    attrs.get(v * stride..(v + 1) * stride).unwrap_or(&[])
}

/// A grelha baricêntrica de `k` partes por aresta, com os pontos das arestas **partilhados**.
fn build(
    mesh: &Mesh2d,
    attrs: &[f64],
    stride: usize,
    deform: &mut DeformAttrs<'_>,
    k: u32,
) -> (Mesh2d, Vec<[f64; 2]>, Vec<f64>) {
    let kf = f64::from(k);
    let mut indice: std::collections::BTreeMap<No, u32> = std::collections::BTreeMap::new();
    let mut rest: Vec<[f64; 2]> = Vec::new();
    let mut agora: Vec<[f64; 2]> = Vec::new();
    let mut saida_attrs: Vec<f64> = Vec::new();
    let mut scratch = vec![0.0; stride];
    let mut tris: Vec<[u32; 3]> = Vec::new();

    for (t_idx, t) in mesh.tris.iter().enumerate() {
        let (Some(&a), Some(&b), Some(&c)) = (
            mesh.rest.get(t[0] as usize),
            mesh.rest.get(t[1] as usize),
            mesh.rest.get(t[2] as usize),
        ) else {
            continue;
        };
        // A grelha baricêntrica `P(i,j) = A + (B−A)·i/k + (C−A)·j/k`, com `i + j <= k`.
        let mut no_de = |i: u32, j: u32| -> u32 {
            let chave = chave_do_no(t, k, i, j, t_idx);
            if let Some(&v) = indice.get(&chave) {
                return v;
            }
            let p = ponto_canonico(chave, [a, b, c], t, mesh, kf, i, j);
            attrs_canonicos(chave, t, attrs, stride, (kf, i, j), &mut scratch);
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a malha refinada não passa de 2^32 nós: o orçamento de peças limita-a muito antes"
            )]
            let v = rest.len() as u32;
            rest.push(p);
            agora.push(deform(p, &scratch));
            saida_attrs.extend_from_slice(&scratch);
            indice.insert(chave, v);
            v
        };
        for j in 0..k {
            for i in 0..(k - j) {
                let (p00, p10, p01) = (no_de(i, j), no_de(i + 1, j), no_de(i, j + 1));
                tris.push([p00, p10, p01]);
                if i + j + 1 < k {
                    let p11 = no_de(i + 1, j + 1);
                    tris.push([p10, p11, p01]);
                }
            }
        }
    }
    (
        Mesh2d {
            rest,
            tris,
            size: mesh.size,
        },
        agora,
        saida_attrs,
    )
}

/// Os atributos de um ponto da grelha baricêntrica, calculados **da chave** — o gémeo exacto do
/// [`ponto_canonico`], e escrito ao lado dele de propósito: os dois têm de responder pela MESMA
/// proveniência, senão um vértice recebe a posição de um sítio e o peso de outro.
fn attrs_canonicos(
    chave: No,
    t: &[u32; 3],
    attrs: &[f64],
    stride: usize,
    // `(k, i, j)` — a grelha baricêntrica do ponto, como o [`ponto_canonico`] a recebe.
    grelha: (f64, u32, u32),
    out: &mut [f64],
) {
    let (kf, i, j) = grelha;
    if stride == 0 {
        return;
    }
    let de = |v: u32, c: usize| -> f64 {
        attrs
            .get(v as usize * stride + c)
            .copied()
            .unwrap_or_default()
    };
    match chave {
        No::Canto(v) => {
            for (c, o) in out.iter_mut().enumerate() {
                *o = de(v, c);
            }
        }
        No::Aresta(u, v, s) => {
            let f = f64::from(s) / kf;
            for (c, o) in out.iter_mut().enumerate() {
                let (pu, pv) = (de(u, c), de(v, c));
                *o = (pv - pu).mul_add(f, pu);
            }
        }
        No::Miolo(..) => {
            let (u, v) = (f64::from(i) / kf, f64::from(j) / kf);
            for (c, o) in out.iter_mut().enumerate() {
                let a = de(t[0], c);
                *o = (de(t[2], c) - a).mul_add(v, (de(t[1], c) - a).mul_add(u, a));
            }
        }
    }
}

/// A chave canónica de um ponto da grelha baricêntrica.
fn chave_do_no(t: &[u32; 3], k: u32, i: u32, j: u32, t_idx: usize) -> No {
    let aresta = |u: u32, v: u32, s: u32| -> No {
        // ⭐ Sempre a partir do vértice de índice MENOR: os dois triângulos que partilham a aresta
        // chegam à mesma expressão, na mesma ordem, e logo aos mesmos bits.
        if u < v {
            No::Aresta(u, v, s)
        } else {
            No::Aresta(v, u, k - s)
        }
    };
    match (i, j) {
        (0, 0) => No::Canto(t[0]),
        _ if i == k => No::Canto(t[1]),
        _ if j == k => No::Canto(t[2]),
        (_, 0) => aresta(t[0], t[1], i),
        (0, _) => aresta(t[0], t[2], j),
        _ if i + j == k => aresta(t[1], t[2], j),
        #[expect(
            clippy::cast_possible_truncation,
            reason = "um índice de triângulo de uma malha de imagem cabe em u32 muito antes de a imagem caber em memória"
        )]
        _ => No::Miolo(t_idx as u32, i, j),
    }
}

/// A posição de repouso de um ponto, calculada **da chave** e nunca do triângulo em mãos.
fn ponto_canonico(
    chave: No,
    abc: [[f64; 2]; 3],
    t: &[u32; 3],
    mesh: &Mesh2d,
    kf: f64,
    i: u32,
    j: u32,
) -> [f64; 2] {
    match chave {
        No::Canto(v) => mesh.rest.get(v as usize).copied().unwrap_or(abc[0]),
        No::Aresta(u, v, s) => {
            let (Some(&pu), Some(&pv)) = (mesh.rest.get(u as usize), mesh.rest.get(v as usize))
            else {
                return abc[0];
            };
            let f = f64::from(s) / kf;
            [
                (pv[0] - pu[0]).mul_add(f, pu[0]),
                (pv[1] - pu[1]).mul_add(f, pu[1]),
            ]
        }
        No::Miolo(..) => {
            let (u, v) = (f64::from(i) / kf, f64::from(j) / kf);
            let _ = t;
            [
                (abc[2][0] - abc[0][0]).mul_add(v, (abc[1][0] - abc[0][0]).mul_add(u, abc[0][0])),
                (abc[2][1] - abc[0][1]).mul_add(v, (abc[1][1] - abc[0][1]).mul_add(u, abc[0][1])),
            ]
        }
    }
}
