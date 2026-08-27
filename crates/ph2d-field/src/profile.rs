//! **O perfil 2D** — a figura plana fechada de onde saem `Extrude` e `Revolve` ([ADR-0161]).
//!
//! É a peça que liga o modelador 3D à **caneta que a casa já tem**: o artista desenha no editor
//! vetorial, e o desenho vira sólido. O fluxo do MoI (*desenhar o contorno, depois extrudar ou
//! revolucionar*) nasce daqui.
//!
//! # ⚠️ O perfil é COZIDO, e é isso que ele guarda
//!
//! O que mora aqui é uma **polilinha fechada**, não uma Bézier. Não é preguiça: a distância exata a
//! uma cúbica exige resolver uma quíntica, que não é exprimível na árvore de avaliação — nem o
//! `libfive` o faz. O que se faz é **achatar com tolerância declarada**, e é por isso que a
//! tolerância viaja **dentro** do perfil ([`Profile::tolerance`]): sem ela, "este perfil está bom?"
//! é uma pergunta sem resposta, e re-cozinhar a fonte com outro número passa despercebido.
//!
//! Isto é a lei **fonte ≠ cozido** do editor vetorial ([ADR-0121]/[ADR-0132]) aplicada uma camada
//! acima: a **fonte** continua a ser o path do documento vetorial, com os handles e o raio vivo de
//! quina; o **cozido** é o que este tipo guarda.
//!
//! ⭐ **O arredondamento de quina do perfil vem de graça** — quem coze usa a geometria já cozida do
//! path, então o *corner widget* do editor vetorial já entregou os arcos. O módulo 3D não tem, e
//! não deve ter, uma segunda resposta para "arredondar a quina de um contorno".
//!
//! # Por que a regra de preenchimento é COPIADA e não importada
//!
//! [`FillRule`] repete o tipo homónimo da `ph2d-vec-scene` de propósito. Esta crate é **o
//! documento** e não pode depender do documento de outro módulo: um `ph2d-field` que importasse o
//! modelo vetorial faria um arquivo salvo do modelador depender do schema do editor de vetores, e
//! um passaria a quebrar o outro. A conversão é trabalho de quem coze (`ph2d-field-profile`).
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md
//! [ADR-0121]: ../../../docs/architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md
//! [ADR-0132]: ../../../docs/architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md

use serde::{Deserialize, Serialize};

/// ⭐⭐ **O NÍVEL DE RESOLUÇÃO de omissão** de uma forma ainda ligada ao desenho (W55).
///
/// É o joelho que a tabela do `ph2d_field_profile::TOLERANCE_RATIO` mediu: a silhueta erra 0,009 %
/// da peça e o salto de normal fica em 2,14°, o que apagou 98 % dos pixels em degrau. *O default é
/// o número certo; o knob existe para a peça que é grande ou vista de perto.*
pub const DEFAULT_PROFILE_RESOLUTION: u32 = 1;

/// ⭐⭐ **O TETO do nível de resolução — e o recurso dele é o TRAÇADO ASSENTE** (W55).
///
/// Cada nível divide a tolerância de cozimento por si (`ph2d_field_profile::tolerance_ratio_for`).
/// Numa curva suave a contagem de arestas anda com `tol^-1/2`, e o custo do traçado é **linear** nas
/// arestas — então o preço de um nível cresce com a **raiz** dele, e não com ele:
///
/// | nível | tolerância | arestas | traçado assente | *idem*, calmo |
/// |---:|---:|---:|---:|---:|
/// | **1** (omissão) | `1e-4` | 168 | 184,1 ms | *139 ms* |
/// | 2 | `5e-5` | 236 | 241,4 ms | *183 ms* |
/// | 4 | `2,5e-5` | 332 | 336,0 ms | *254 ms* |
/// | 8 | `1,25e-5` | 472 | 450,3 ms | *341 ms* |
/// | **16** (teto) | `6,3e-6` | 664 | 648,7 ms | *491 ms* |
/// | 32 | `3,1e-6` | 940 | 900,5 ms | *682 ms* |
///
/// (sonda `field3d_profile::tests::the_table_that_chose_the_resolution_ceiling`: círculo de raio 0,5
/// extrudado, 640×480, mediana de 7.)
///
/// ⚠️ **A coluna «calmo» é DERIVADA, e a razão está medida ao lado.** A corrida saiu com `load ≈ 4,7`
/// — abaixo do inutilizável, acima do ideal —, e a linha do nível 1 é a **mesma configuração** que a
/// W54 mediu a `load < 3` em **139,3 ms**. As duas leituras do mesmo trabalho dão **184,1** e
/// **139,3**: ⭐ *32 % de diferença só de carga, sem uma linha de código mudar* — que é exactamente
/// por que a lei do `CLAUDE.md` §5 existe. A coluna calma é a medida escalada por esse fator (0,757),
/// e o teto escolhido sobre a coluna **medida** é, por isso, conservador.
///
/// ⚠️⚠️ **A PERNA DO RELÓGIO DESTE TETO CAIU** (W60, 2026-08-24). Ele foi escolhido por **duas**
/// razões, e só uma continua de pé.
///
/// ~~*O teto é 16 porque é onde o assentar deixa de parecer instantâneo*~~ — meio segundo por gesto.
/// ⛔ As waves W56e–W59 (fatia de profundidade · passo derivado do documento · corte por casco)
/// baixaram o traçado **~2,2×**, e a mesma escada, medida a `load ≈ 4,2`, dá hoje:
///
/// | nível | arestas | traçado, 23/08 | **traçado, 24/08** | ms/aresta |
/// |---:|---:|---:|---:|---:|
/// | 1 | 168 | 184,1 ms | **81,0 / 83,9 ms** | 0,48–0,50 |
/// | 8 | 472 | 450,3 ms | **201,5 / 209,2 ms** | 0,43–0,44 |
/// | **16** | 664 | 648,7 ms | **288,4 / 317,0 ms** | 0,43–0,48 |
/// | 32 | 940 | 900,5 ms | **439,7 / 460,1 ms** | 0,47–0,49 |
/// | 64 | 1328 | — | **705,6 / 727,8 ms** | 0,53–0,55 |
///
/// ⇒ pela regra de **meio segundo**, o teto de hoje seria **32**. ⚠️ E o `128` da 1.ª corrida deu
/// `2 071,8 ms` (`1,10 ms/aresta`, um joelho aparente) e `1 184,0 ms` na 2.ª (`0,63`) — **era
/// carga**, e não lei. *Uma leitura só não é um joelho.*
///
/// ⭐ **O que segura o 16 é a outra perna: o OLHO.** O nível 32 não compra nada que se veja — e essa
/// afirmação é **mais forte** do que este doc dizia: a régua das bandas põe o joelho em **168
/// arestas**, que é o próprio [`DEFAULT_PROFILE_RESOLUTION`]. ⛔ E ela **não consegue** medir acima
/// disso: o limiar dela é `3°` e o salto do nível 1 já é `2,14°`
/// (`field3d_profile::tests::the_table_of_where_the_banding_knee_moves_with_zoom` regista as três
/// refutações — a régua saturada, o aro a engolir a versão sem limiar, e a câmera a entrar na peça).
///
/// ⛔⛔ **E o zoom NÃO é um eixo:** o cozimento não conhece a câmera (a tolerância é `span × ratio`,
/// com `span` do **desenho**), então o salto de normal de um círculo de `n` lados é `360/n` em
/// qualquer enquadramento. *«O knob existe para a peça vista de perto» é sobre a SILHUETA, não sobre
/// a luz.*
///
/// ⇒ **Subir o teto é decisão de produto e precisa de um contorno de curvatura VARIÁVEL** para ser
/// medida — não de um círculo. Até lá o número fica onde está, agora com uma perna só e ela
/// nomeada.
///
/// ⚠️ **O custo é linear nas arestas** — `0,43` a `0,55 ms/aresta` ao longo da escada inteira em
/// 24/08 (era `0,95`–`1,10` em 23/08) —, então não há joelho onde se esconder: o teto é uma escolha
/// de produto sobre uma reta, e diz de que recurso é.
///
/// ⚠️ **É um limite de RECURSO e não de validade**: um perfil de 940 arestas é perfeitamente
/// correcto, e o documento aceita-o por outra porta (`Profile::new` com a tolerância à mão). O que
/// este número fecha é a **faixa do controle**, que é onde um teto pertence.
///
/// # ⭐⭐⭐ 16 → 64 (2026-08-26), por decisão do Enio e com a régua que faltava
///
/// A W60 concluiu, correctamente, que a régua das bandas **satura**: ela media um **círculo**, e num
/// círculo todo ponto tem a mesma curvatura. ⇒ a fixtura nova é uma **elipse `4:1`**, cuja ponta é
/// `16×` mais curva que o lado, e a régua é o **maior salto de normal** — que é o que a luz mostra
/// (`field3d_profile::tests::the_table_of_the_sharpest_corner`, release, máquina calma):
///
/// | nível | arestas | salto MAX | salto mediano | traçado |
/// |---:|---:|---:|---:|---:|
/// | 1 | 112 | 8,84° | 2,09° | 43,6 ms |
/// | 8 | 320 | 3,11° | 0,73° | 109,6 ms |
/// | ~~16~~ | 448 | 2,21° | 0,52° | 145,1 ms |
/// | **32** | **636** | **1,55°** | **0,37°** | **216,6 ms** |
/// | 128 | 1 268 | 0,78° | 0,19° | 483,4 ms |
///
/// ⭐ **A lei é `θ ≈ √(8·tol/R)`** (a sagitta de um arco), e a tabela confirma-a em quatro pontos:
/// dobrar o nível divide o salto por **`√2`** e multiplica as arestas por `√2`. *Um teto escolhido
/// como se o ganho fosse linear escolhe o número errado.*
///
/// ⭐⭐ **E o que o nível compra, na língua de quem desenha:** como `θ ∝ 1/√R`, cada duplicação do
/// teto deixa o artista desenhar um canto **duas vezes mais apertado** antes de ele facetar. Com a
/// barra de `3°` que as fotos do Enio fixaram, o limite era uma elipse de **~5,5:1** no `16`; no
/// **`64`** é **~22:1**. *O teto nunca foi sobre «detalhe»: é sobre que forma se pode desenhar.*
///
/// ⚠️ **Não há joelho onde parar** — cada degrau custa `×√2` e compra `×2`, e a razão
/// benefício/preço **melhora** ao subir. ⇒ o limite é um **recurso absoluto**, e a 1.ª redacção
/// desta nota nomeou o **errado**.
///
/// ⛔⛔ **O recurso NÃO é o quadro assente: é a PRÉ-VISUALIZAÇÃO** (corrigido no mesmo dia, pelo
/// report do Enio *"queda de fps e lentidão com resoluções altas"*). O traçado assente paga-se
/// **uma vez**, quando a câmera pára; o traçado de movimento paga-se **em cada quadro em que a mão
/// mexe**, e é esse que o artista sente. Ele corre noutra thread e tem orçamento próprio de
/// `16,7 ms`, alcançado por baixar a resolução da imagem
/// ([`crate::field3d_preview`], divisor até `MAX_PREVIEW_DIVISOR`).
///
/// Medido (círculo, 640×480, release) contra o divisor **máximo que a casa mediu como seguro**
/// (`D = 8`, deriva de silhueta `0,15 %`):
///
/// | nível | arestas | traçado cheio | a D=8 | cabe nos 16,7 ms? |
/// |---:|---:|---:|---:|:--|
/// | 16 | 664 | 486 ms | 7,6 ms | ✅ |
/// | **32** | **940** | **700 ms** | **10,9 ms** | ✅ |
/// | 64 | 1 328 | 1 091 ms | 17,0 ms | ⛔ **não** |
///
/// ⛔⛔ **E o divisor NÃO é a cura — a medição matou-a.** O custo do traçado **para de cair** por
/// volta de `D=6` e depois **sobe** (medido: `D=1` 299,5 · `D=3` 75,2 · `D=6` **43,8** · `D=8`
/// 46,6 ms). Há um **piso** que a resolução da imagem não toca:
///
/// | nível | arestas | traçado `D=1` | **piso (`D=6`)** | por aresta |
/// |---:|---:|---:|---:|---:|
/// | 1 | 168 | 214 ms | **39,3 ms** | 0,23 ms |
/// | 8 | 472 | 523 ms | 103,7 ms | 0,22 ms |
/// | 16 | 664 | 761 ms | 144,3 ms | 0,22 ms |
/// | **32** | **940** | 1 090 ms | **212,5 ms** | 0,23 ms |
/// | 64 | 1 332 | 1 825 ms | 338,5 ms | 0,25 ms |
///
/// ⭐⭐⭐ **`0,22 ms por aresta`, constante, e cego aos pixels** — de `D=3` para `D=6` são 4× menos
/// pixels e o tempo cai `1,3×`. *Isso não é marchar raios: é MONTAR.* E a montagem corre em **todo
/// traçado**, enquanto o documento não muda entre dois quadros de uma órbita.
///
/// ⛔ **E o defeito é PRÉ-EXISTENTE, não desta wave:** mesmo na resolução de **omissão** (168
/// arestas) o piso é `39 ms` contra um orçamento de `16,7`. *A pré-visualização nunca alcançou o
/// alvo dela numa peça de perfil* — a lei dela foi calibrada com **cilindros**, que não têm contorno
/// cozido. O teto alto não criou a lentidão: tornou-a impossível de ignorar.
///
/// ⚠️ **A montagem BASE está ilibada** (`Hybrid::new`: `4,3 ms` a 168 arestas, `23,3 ms` a 1 344) —
/// o suspeito que fica é a **especialização por ladrilho**, uma compilação por ladrilho × fatia em
/// cada traçado. ⏸️ É a obra seguinte, e a sonda que a decide é
/// `ph2d_field_eval::tests::measure_building_the_tape_against_marching_it`.
///
/// ⭐⭐⭐ **E A RAZÃO DO `32` DISSOLVEU NO MESMO DIA, porque a cura chegou.** O preview passou a
/// **engrossar o contorno** enquanto a mão mexe ([`crate::coarsen`] +
/// `field3d_preview::coarse_doc`) — a mesma lei que já baixava os pixels, aplicada onde o custo
/// estava. Medido, traçado de movimento a `640×360`:
///
/// | arestas | sem a cura | com a cura | ganho |
/// |---:|---:|---:|---:|
/// | 168 (omissão) | 55,3 ms | 52,1 ms | 1,06× |
/// | 472 | 133,4 ms | 54,6 ms | **2,44×** |
/// | 940 | 266,1 ms | **53,7 ms** | ⭐ **4,96×** |
///
/// ⭐ **O custo de movimento passou a ser CONSTANTE (~53 ms) qualquer que seja o nível** — antes
/// crescia em linha recta. *Subir este knob deixou de ter preço em movimento.*
///
/// ⇒ **o teto volta a `64`**, e agora o recurso que manda é de novo o **quadro assente** (`303 ms`
/// contra a regra de meio segundo) — só que desta vez isso é **verdade**, e não a leitura do relógio
/// errado que a 1.ª redacção fez. ⚠️ **O número mexeu-se três vezes em dois dias** (`16 → 64 → 32 →
/// 64`) e cada passo estava certo com o que se sabia: o 1.º media o relógio errado, o 2.º media o
/// certo **sem a cura**, e o 3.º é o mesmo relógio **com ela**. *Uma constante que se move é uma
/// medição a acontecer; o que não pode mover-se em silêncio é a razão.*
///
/// ⛔ **E o defeito PRÉ-EXISTENTE fica:** mesmo agora o movimento custa `~53 ms` contra um orçamento
/// de `16,7` — a pré-visualização continua a não alcançar `60 Hz` numa peça de perfil, em **qualquer**
/// nível. A cura tirou a contribuição do TETO; a base é outra obra.
///
/// ⚠️ E o quadro **assente** continua confortável em qualquer destes níveis (`217 ms` no `32`
/// contra a regra de meio segundo) — *era ele que a 1.ª redacção media, e por isso ela deixou passar
/// um teto que o artista sentia como lentidão.*
///
/// ⛔ **A 1.ª medição desta wave passou pela porta do produto e mediu a PRÓPRIA TRAVA:** a
/// [`ph2d_field_profile::tolerance_ratio_for`] clampa neste número, então os níveis `32`, `64` e
/// `128` recebiam a tolerância do `16` e a tabela saía com `448` arestas quatro vezes — lida como
/// *"o achatamento saturou"*. *Uma sonda que atravessa o limite que quer medir mede o limite.* A
/// cura foi partir a lei em duas: o **span** continua a ser o do produto
/// ([`ph2d_field_profile::span_of`]), o **teto** é o que a sonda contorna.
pub const MAX_PROFILE_RESOLUTION: u32 = 64;

/// Como os contornos de um perfil se combinam.
///
/// Para um perfil de **um** contorno as duas regras coincidem — a distinção só existe quando há
/// ilha ou buraco.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FillRule {
    /// Respeita a orientação de cada contorno (winding). É o default, e é o que faz um contorno
    /// desenhado ao contrário virar buraco.
    #[default]
    NonZero,
    /// Alterna dentro/fora a cada cruzamento: um contorno aninhado é buraco **independente de como
    /// foi orientado**. É a regra robusta para geometria vinda de booleana.
    EvenOdd,
}

/// Por que um perfil foi recusado.
///
/// ⚠️ Como o resto desta crate, nenhuma variante é zelo: um perfil inválido não dá erro na
/// avaliação — ele dá um sólido errado, em silêncio.
// Sem `Eq`: as variantes carregam os `f32` que explicam a recusa.
#[derive(Clone, Debug, PartialEq)]
pub enum ProfileError {
    /// Nenhum contorno. Uma figura vazia não delimita sólido nenhum.
    Empty,
    /// Menos de 3 pontos: não fecha área.
    TooFewPoints { contour: u32, points: u32 },
    /// Coordenada não-finita.
    NonFinite { contour: u32 },
    /// O contorno colapsou numa reta ou num ponto — uma das extensões da caixa dele é zero.
    ///
    /// ⚠️ É este o teste, e **não** a área: uma figura em oito tem área líquida zero e é um perfil
    /// perfeitamente legítimo sob [`FillRule::EvenOdd`]. Recusar por área mataria o caso válido e
    /// deixaria passar o degenerado de verdade.
    Collapsed {
        contour: u32,
        width: f32,
        height: f32,
    },
    /// A tolerância de cozimento não é um número positivo finito.
    BadTolerance { tolerance: f32 },
}

/// Uma figura plana fechada, já achatada em polilinhas.
///
/// Os campos são privados e a única porta é [`Profile::new`]: um `Profile` que exista está válido.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// Contornos **fechados**. O segmento de fecho (último → primeiro) é implícito: o primeiro
    /// ponto **não** se repete no fim. Repeti-lo produziria uma aresta de comprimento zero, e uma
    /// aresta de comprimento zero é uma divisão por zero na distância ponto-segmento.
    contours: Vec<Vec<[f32; 2]>>,
    fill: FillRule,
    tolerance: f32,
}

impl Profile {
    /// Constrói e **valida**.
    ///
    /// Pontos consecutivos repetidos são **removidos** (inclusive o fecho, se quem chamou repetiu o
    /// primeiro ponto no fim) — é limpeza de entrada, não uma decisão de forma: um ponto repetido
    /// não muda a figura e só existe para quebrar a distância ponto-segmento.
    ///
    /// # Errors
    /// Ver [`ProfileError`].
    pub fn new(
        contours: Vec<Vec<[f32; 2]>>,
        fill: FillRule,
        tolerance: f32,
    ) -> Result<Self, ProfileError> {
        if !tolerance.is_finite() || tolerance <= 0.0 {
            return Err(ProfileError::BadTolerance { tolerance });
        }
        if contours.is_empty() {
            return Err(ProfileError::Empty);
        }
        let mut cleaned: Vec<Vec<[f32; 2]>> = Vec::with_capacity(contours.len());
        for (i, raw) in contours.into_iter().enumerate() {
            let idx = i as u32;
            if raw.iter().any(|p| !p[0].is_finite() || !p[1].is_finite()) {
                return Err(ProfileError::NonFinite { contour: idx });
            }
            let c = dedup_closed(&raw);
            let n = c.len() as u32;
            if n < 3 {
                return Err(ProfileError::TooFewPoints {
                    contour: idx,
                    points: n,
                });
            }
            let (min, max) = contour_bounds(&c);
            let (w, h) = (max[0] - min[0], max[1] - min[1]);
            if w <= 0.0 || h <= 0.0 {
                return Err(ProfileError::Collapsed {
                    contour: idx,
                    width: w,
                    height: h,
                });
            }
            cleaned.push(c);
        }
        Ok(Self {
            contours: cleaned,
            fill,
            tolerance,
        })
    }

    #[must_use]
    pub fn contours(&self) -> &[Vec<[f32; 2]>] {
        &self.contours
    }

    #[must_use]
    pub fn fill(&self) -> FillRule {
        self.fill
    }

    /// A tolerância com que este perfil foi achatado a partir da fonte — **o erro máximo entre esta
    /// polilinha e a curva que a originou**, em unidades do documento.
    #[must_use]
    pub fn tolerance(&self) -> f32 {
        self.tolerance
    }

    /// Quantas arestas o perfil tem ao todo.
    ///
    /// ⚠️ **É o número que manda no custo**: cada aresta vira **~26 nós** na árvore de avaliação
    /// (medido, `docs/3DModeling/04_resultados_perfis.md` §3), e o traçado avalia a árvore inteira
    /// por pixel. Quem mexer na tolerância mexe aqui.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.contours.iter().map(Vec::len).sum()
    }

    /// A caixa envolvente `(min, max)` de todos os contornos.
    #[must_use]
    pub fn bounds(&self) -> ([f32; 2], [f32; 2]) {
        let mut min = [f32::INFINITY; 2];
        let mut max = [f32::NEG_INFINITY; 2];
        for c in &self.contours {
            let (a, b) = contour_bounds(c);
            for k in 0..2 {
                min[k] = min[k].min(a[k]);
                max[k] = max[k].max(b[k]);
            }
        }
        (min, max)
    }
}

/// Remove pontos consecutivos idênticos, **tratando a lista como fechada** (o último é vizinho do
/// primeiro).
fn dedup_closed(pts: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut out: Vec<[f32; 2]> = Vec::with_capacity(pts.len());
    for &p in pts {
        if out.last() != Some(&p) {
            out.push(p);
        }
    }
    // O fecho: se o último coincide com o primeiro, ele é a aresta de comprimento zero.
    while out.len() > 1 && out.last() == out.first() {
        out.pop();
    }
    out
}

fn contour_bounds(c: &[[f32; 2]]) -> ([f32; 2], [f32; 2]) {
    let mut min = [f32::INFINITY; 2];
    let mut max = [f32::NEG_INFINITY; 2];
    for p in c {
        for k in 0..2 {
            min[k] = min[k].min(p[k]);
            max[k] = max[k].max(p[k]);
        }
    }
    (min, max)
}

/// ⭐⭐⭐ **O MESMO CONTORNO, MAIS GROSSO — para a pré-visualização.**
///
/// # ⚠️ Por que ela existe
///
/// O traçado custa **`0,22 ms` por aresta do contorno**, medido, e esse custo é **cego aos pixels**:
/// numa imagem 4× menor ele cai `1,3×`. ⇒ a pré-visualização, que baixa a **resolução da tela** para
/// caber no orçamento de um quadro, **não baixava as arestas** — e subir o `Resolution` custava fps
/// enquanto a mão mexia (report do Enio, 2026-08-26).
///
/// ⭐ Esta é a **mesma lei que o módulo já ship**, aplicada onde faltava: *grosso a mexer, nítido ao
/// assentar*. O que o artista pediu em detalhe aparece quando ele **pára**, que é quando ele olha.
///
/// # ⚠️ Ela DECIMA, não recoze — e a diferença é o que a torna possível
///
/// Recozer exigiria a curva de origem, que vive na cena vetorial e **não** no documento. O que há
/// aqui é a polilinha já achatada, e um contorno achatado por **tolerância** tem os pontos densos
/// onde a curvatura é alta — então tirar um em cada `k` preserva o carácter da forma em vez de a
/// achatar por igual.
///
/// ⚠️ **Um contorno decimado pode auto-intersectar-se** numa feição fina, e é por isso que o
/// resultado passa pelo [`Profile::new`]: se ele recusar, volta o original. *Uma pré-visualização
/// que estraga a peça é pior do que uma lenta.*
///
/// ⛔ E ela **nunca sobe**: um `max_edges` maior do que o contorno devolve o próprio contorno, sem
/// inventar pontos que a curva não tem.
#[must_use]
pub fn coarsen(profile: &Profile, max_edges: usize) -> Profile {
    let total = profile.segment_count();
    if total <= max_edges || max_edges < 3 {
        return profile.clone();
    }
    // ⚠️ **O orçamento é o mesmo para todos os contornos**: um furo e a borda de fora têm de
    // encolher JUNTOS, senão o furo escapa da peça que o continha.
    let giro_total: f64 = profile.contours().iter().map(|c| total_abs_turn(c)).sum();
    // ⚠️ Um contorno FECHADO gira sempre `2π`, então isto não acontece — mas um `NaN` que viesse de
    // um ponto degenerado passaria por um `<= 0.0` ingénuo, e o orçamento sairia `NaN`.
    if !giro_total.is_finite() || giro_total <= 0.0 {
        return profile.clone();
    }
    let orcamento = giro_total / max_edges as f64;
    let thinner: Vec<Vec<[f32; 2]>> = profile
        .contours()
        .iter()
        .map(|c| {
            // ⚠️ Um contorno que já é pequeno fica INTEIRO: decimá-lo levá-lo-ia abaixo do triângulo,
            // e um furo de três lados é melhor do que um furo que desapareceu.
            if c.len() <= 8 {
                return c.clone();
            }
            decimate_by_turn(c, orcamento)
        })
        .collect();
    if thinner.iter().any(|c| c.len() < 3) {
        return profile.clone();
    }
    // ⚠️ A tolerância declarada sobe com a decimação: ela é o erro contra a curva de origem, e a
    // polilinha decimada erra mais. Mentir aqui envenenaria quem a usa para escolher uma grade.
    let ficou: usize = thinner.iter().map(Vec::len).sum();
    let passo_medio = (total as f32 / ficou.max(1) as f32).max(1.0);
    let tol = profile.tolerance() * passo_medio;
    Profile::new(thinner, profile.fill(), tol).unwrap_or_else(|_| profile.clone())
}

/// A curvatura total de um contorno fechado, em radianos e **sem sinal**.
///
/// ⚠️ **Sem sinal de propósito.** O giro *com* sinal de um contorno fechado é `±2π`, sempre — ele não
/// distingue um círculo de uma estrela. O que a decimação precisa de repartir é **quanta direcção**
/// a forma tem para gastar, e uma ponta de estrela gasta muito em pouco caminho.
fn total_abs_turn(c: &[[f32; 2]]) -> f64 {
    (0..c.len()).map(|i| turn_at(c, i)).sum()
}

/// O ângulo, em radianos, entre a aresta que **chega** ao vértice `i` e a que **sai** dele.
fn turn_at(c: &[[f32; 2]], i: usize) -> f64 {
    let n = c.len();
    if n < 3 {
        return 0.0;
    }
    let (p, q, r) = (c[(i + n - 1) % n], c[i], c[(i + 1) % n]);
    let a = [f64::from(q[0] - p[0]), f64::from(q[1] - p[1])];
    let b = [f64::from(r[0] - q[0]), f64::from(r[1] - q[1])];
    let cross = a[0] * b[1] - a[1] * b[0];
    let dot = a[0] * b[0] + a[1] * b[1];
    // `atan2` do produto vectorial contra o escalar — estável mesmo com arestas muito curtas, que
    // é onde uma versão por `acos` do normalizado devolve `NaN`.
    cross.atan2(dot).abs()
}

/// ⭐⭐⭐ **A decimação por GIRO** — mantém um vértice quando o ângulo acumulado desde o último
/// mantido chega ao orçamento.
///
/// # ⛔ O que ela substitui, e por que a anterior estava errada
///
/// A versão até 2026-08-27 tirava **um em cada `k`** vértices, com este raciocínio no doc do
/// [`coarsen`]: *«um contorno achatado por tolerância tem os pontos densos onde a curvatura é alta —
/// então tirar um em cada `k` preserva o carácter da forma»*. ⭐ Isso é **verdade para curvatura**,
/// que é distribuída por muitos vértices.
///
/// ⚠️ **Uma QUINA não é curvatura distribuída: é um vértice só, com todo o ângulo dentro.** Um passo
/// por índice apaga-a com probabilidade `(k−1)/k`, e o que fica no lugar é um bisel — *e se ela
/// sobrevive depende de o índice dela ser divisível pelo passo, o que é uma lotaria.*
///
/// ⛔ **Medido** (`measure_whether_the_preview_decimation_eats_corners`, uma estrela de 5 pontas com
/// 400 pontos, traçada a `640×360`):
///
/// | tecto | passo | pixels que mudam | normal p99 | normal máx |
/// |---:|---:|---:|---:|---:|
/// | `336` | `2` | `0` | `0,034°` | `0,048°` |
/// | **`168`** | **`3`** | **`509` (`0,87 %`)** | **`28,1°`** | **`126,8°`** |
/// | `84` | `5` | `0` | `0,034°` | `0,048°` |
///
/// ⭐ As quinas caem em múltiplos de `40`: com passo `2` e `5` elas sobrevivem, com `3` **três em
/// cada cinco morrem**. E o `PREVIEW_MAX_EDGES` que ship é justamente `168`.
///
/// # ⭐ Por que o GIRO é a grandeza certa
///
/// O erro de uma corda que substitui um arco é fixado pelo **ângulo** que o arco varre, não pelo
/// número de pontos que ele tinha. E o erro que se **vê** é o da **normal**, que é esse mesmo ângulo
/// (medido: a normal p99 de um círculo decimado é exactamente `∝ 1/n`). ⇒ repartir o giro por igual
/// distribui o erro por igual, e um vértice que sozinho gasta o orçamento — uma quina — é mantido
/// **por construção**, sem uma regra própria a dizê-lo.
fn decimate_by_turn(c: &[[f32; 2]], orcamento: f64) -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity(c.len());
    let mut acc = 0.0f64;
    for i in 0..c.len() {
        let t = turn_at(c, i);
        if out.is_empty() || acc + t >= orcamento {
            out.push(c[i]);
            acc = 0.0;
        } else {
            acc += t;
        }
    }
    out
}
