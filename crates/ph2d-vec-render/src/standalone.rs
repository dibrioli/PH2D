//! **DESENHAR UMA FORMA FORA DO DOCUMENTO** — irmão de [`super`] pelo teto de 700 LOC, e o corte é
//! por RESPONSABILIDADE: ali fica o que compõe a CENA (a pilha de z, os clips, os produtores
//! vivos); aqui, as duas portas que desenham UMA forma isolada num alvo qualquer — o bake de um
//! tile, uma prévia, um carimbo.
//!
//! ⚠️ **O `standalone_tests.rs` já existia** e apontava para cá antes de este ficheiro existir: os
//! gates desta família foram cortados em 2026-08-XX e as funções ficaram para trás. *Um ficheiro de
//! testes que nomeia um módulo que não existe é um corte que parou a meio.*

use super::*;

/// **Desenha UM caminho avulso**, pela mesma [`draw_path`] do `dispatch` — a porta
/// que o bake de tile de uma forma usa, e que garante que o tile é o que a forma
/// PARECE, não uma segunda rasterização.
///
/// ⚠️ **Sem tint de instância, de propósito.** O `tint` é por-CÓPIA e multiplica o
/// tile no shader de sprite; aplicá-lo aqui pintaria a cor duas vezes. É por isso
/// que o BRANCO abaixo é a identidade certa, e não uma escolha de cor.
///
/// ⚠️ **Passa pela porta de INSTÂNCIA e não pela do DOCUMENTO, e a diferença foi um
/// bug** (Enio, 2026-08-20: *"não funcionou"*). Um `source.shape` nu — sem fill nem
/// stroke autorados — é um **PRIMITIVO**, e as duas portas discordam sobre ele: a
/// de instância ([`draw_shape_instance_tessellated`]) tem um ramo que preenche a
/// SILHUETA, e a do documento ([`draw_path`]) não desenha nada. O tile saía
/// **totalmente transparente**, o quad desenhava nada, e o modo de falha é mudo —
/// medido pelo `PH2D_GLOW_DIAG`, que mostrava a camada CERTA (`tile_forma=1`,
/// `camada=120`) e nenhum halo.
///
/// ⚠️ *Escolher a porta pelo que ela É (uma forma de instância) e não pelo que ela
/// PARECE (um caminho) é a regra; o crispo desta mesma forma passa por aqui.*
pub fn draw_path_standalone(path: &VecPath, transform: Affine, target: &mut VectorScene) {
    let tess = instance::tessellate_shape_instance(path);
    instance::draw_shape_instance_tessellated(
        path,
        &tess,
        None,
        transform,
        [1.0, 1.0, 1.0, 1.0],
        target,
    );
}

/// ⭐⭐⭐ **OS COEFICIENTES DO TRANSBORDO — o que depende só do CAMINHO.**
///
/// ⛔⛔ Ela nasceu do RECORTE POR CÂMARA (ordem do dono, 2026-09-21) e de uma MEDIÇÃO que me
/// acusou: com o recorte a deitar fora `94 %` das cópias, o encode caía só `2,3×` — porque o
/// [`inflate_for_stroke`] percorre a pilha de tinta **DUAS vezes e constrói um `Stroke` da kurbo**,
/// e o recorte chamava-o **por cópia**. *Uma cura cujo teste custa o que ela poupa não é uma cura.*
///
/// ⭐ A lei inteira é **linear em `(sx, sy)`**, logo os coeficientes são da FORMA: a meia-largura
/// vezes o alcance da junta do contorno mais gordo, e o maior deslocamento/dilatação da pilha.
/// Calculam-se UMA vez por geometria (a tesselação já é cacheada) e por cópia sobram três
/// multiplicações.
///
/// ⚠️⚠️ **A ORDEM DAS OPERAÇÕES É PRESERVADA e isso é deliberado:** o `m` é escrito
/// `((meia_largura · smax) · alcance)`, exactamente como estava, porque reassociar três factores em
/// `f64` muda o último bit — e há gate a exigir igualdade **AO BIT** com a versão de referência.
/// ⭐ O `max` sobre as camadas pôde subir porque `smax >= 0` e a multiplicação é monótona: o
/// vencedor não depende da escala.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Transbordo {
    /// `(meia-largura, alcance da junta)` do contorno MAIS GORDO da pilha — `None` sem traço.
    traco: Option<(f64, f64)>,
    /// O maior deslocamento (ou dilatação) local, em `x` e em `y`.
    kx: f64,
    ky: f64,
}

impl Transbordo {
    /// **Esta forma não transborda nada** — o caminho comum (um primitivo com tinta e sem traço),
    /// e o que permite ao recorte saltar as duas raízes quadradas por cópia.
    pub(crate) fn e_nulo(&self) -> bool {
        self.traco.is_none() && self.kx == 0.0 && self.ky == 0.0
    }
}

/// Os coeficientes de [`Transbordo`] deste caminho — a metade CARA, uma vez por geometria.
pub(crate) fn transbordo_do_caminho(path: &VecPath) -> Transbordo {
    let mut traco: Option<(f64, f64)> = None;
    let mut melhor = f64::NEG_INFINITY;
    for camada in path.paint_stack() {
        let ph2d_vec_scene::PaintRef::Stroke(s) = camada.paint else {
            continue;
        };
        // Só a JUNTA e o `miter_limit` são lidos aqui; um tracejado não muda o transbordo.
        let k = kurbo_stroke(s, None);
        let reach = if matches!(k.join, Join::Miter) {
            k.miter_limit.max(1.0)
        } else {
            1.0
        };
        let meia = 0.5 * s.width;
        // ⚠️ O vencedor mede-se SEM a escala (ela é comum e positiva ⇒ não inverte a ordem).
        let peso = meia * reach;
        if peso > melhor {
            melhor = peso;
            traco = Some((meia, reach));
        }
    }
    let (mut kx, mut ky) = (0.0_f64, 0.0_f64);
    for camada in path.paint_stack() {
        let [dx, dy] = camada.offset;
        kx = kx.max(dx.abs());
        ky = ky.max(dy.abs());
        if camada.dilate > 0.0 {
            kx = kx.max(camada.dilate);
            ky = ky.max(camada.dilate);
        }
    }
    Transbordo { traco, kx, ky }
}

/// **Aplica o transbordo a uma caixa sob um afim** — a metade BARATA, por cópia.
pub(crate) fn inflate_com(t: &Transbordo, xf: Affine, r: Rect) -> Rect {
    if t.e_nulo() {
        return r;
    }
    let [a, b, c, d, _, _] = xf.as_coeffs();
    let sx = (a * a + b * b).sqrt();
    let sy = (c * c + d * d).sqrt();
    // ⚠️ A ordem é a da referência: `((meia · smax) · alcance)`.
    let m = t
        .traco
        .map_or(0.0, |(meia, reach)| meia * sx.max(sy) * reach);
    let ox = t.kx * sx;
    let oy = t.ky * sy;
    if m > 0.0 || ox > 0.0 || oy > 0.0 {
        r.inflate(m + ox, m + oy)
    } else {
        r
    }
}

/// O transbordo do traço sobre a caixa do fill — extraído junto com [`path_bounds_under`].
///
/// ⚠️⚠️ **A PROSA que explica cada termo desta conta vive aqui, INTEIRA, e ela foi PAGA três
/// vezes** (as três ocorrências da ponta CEIFADA). ⛔ A 1.ª tentativa de a mudar para cá
/// colheu-a *linha a linha pelos marcadores* e partiu cada parágrafo a meio — **destruir a
/// prosa é a forma mais comum de perder a memória de um repo em que o porquê vive nos
/// doc-comments**, e o `cargo clippy` foi quem a apanhou (`empty line after doc comment`).
/// *Colhe-se por BLOCO, nunca por linha.*
///
/// O traço transborda o fill por metade da largura; escala com o afim.
///
/// ⚠️ **E uma junta MITER vai MUITO além disso.** Numa quina de ângulo interno `θ` a ponta do
/// miter fica a `½w / sin(θ/2)` do vértice — numa ponta de estrela (36°), **3,24 × ½w** —, e
/// a kurbo só a corta no `miter_limit`. Inflar por meia largura recortava a ponta contra a
/// borda do scratch, e o efeito visível era **a ponta CEIFADA** (reportado no smoke). O
/// limite é lido do MESMO construtor que o renderer usa, não de uma segunda constante.
/// ⭐⭐⭐ **A CAIXA É A DO CONTORNO MAIS GORDO DA PILHA** (v20), e não a do de base. Uma
/// forma com um traço extra de `20` por baixo de um de `2` é **dez vezes** mais gorda do
/// que o `stroke.width` diz — e esta caixa dimensiona o scratch do FX e o rectângulo da
/// camada de mistura. Medir só a base devolvia uma caixa que **corta o desenho**, que é o
/// mesmo sintoma da ponta CEIFADA que o parágrafo acima nomeia, uma tinta adiante.
///
/// ⚠️ A porta [`ph2d_vec_scene::VecPath::paint_stack`] já devolve a base como uma camada,
/// então o caminho comum (uma forma sem pilha) mede exactamente o que media.
///
/// Só a JUNTA e o `miter_limit` são lidos aqui; um tracejado não muda o
/// transbordo, então medir o caminho para o ajustar seria trabalho por nada.
///
/// ⭐⭐⭐ **E A CAMADA DESLOCADA** (v21). Uma camada com `offset` desenha FORA da caixa da
/// forma, e esta caixa dimensiona o scratch do FX e o rectângulo da camada de mistura ⇒ sem
/// isto ela é **recortada**, que é a MESMA ponta CEIFADA que os dois parágrafos acima
/// nomeiam, uma tinta adiante. (A terceira vez que este ficheiro paga a conta.)
///
/// ⚠️ **Simétrica de propósito.** Um deslocamento tem direcção e a caixa não: inflar os dois
/// lados por `|dx|` e `|dy|` cobre qualquer direcção e nunca corta. Uma caixa apertada demais
/// CORTA o desenho (defeito visível); uma folgada custa scratch (memória) — a troca só tem
/// um lado.
///
/// O deslocamento é LOCAL, e a caixa está no espaço de `xf` ⇒ ele escala com o afim,
/// pelo mesmo par (`sx`, `sy`) que a largura acima usa.
///
/// ⭐⭐ **E O OFFSET DE CAD** (v22): uma camada que CRESCE desenha para fora da silhueta
/// e seria recortada pela mesma borda. ⛔ Só o crescer conta — encolher fica DENTRO da
/// forma, e inflar por ele daria folga a troco de nada.
///
/// ⭐ Hoje ela é as duas metades juntas ([`transbordo_do_caminho`] + [`inflate_com`]), e o
/// caminho quente do recorte chama-as separadas.
pub(crate) fn inflate_for_stroke(path: &VecPath, xf: Affine, r: Rect) -> Rect {
    inflate_com(&transbordo_do_caminho(path), xf, r)
}

/// ⭐⭐ **A REFERÊNCIA: o corpo de [`inflate_for_stroke`] como ele era antes de 2026-09-21.**
///
/// ⛔ Ela fica **só sob `cfg(test)`** e existe por uma razão: a versão de produção subiu os
/// coeficientes para fora do laço, e a única forma honesta de afirmar que isso **não muda um bit**
/// é ter as duas a correr lado a lado sobre um corpo de formas e afins. *Uma reescrita de
/// performance que se diz byte-idêntica sem a referência ao lado é uma promessa.*
#[cfg(test)]
pub(crate) fn inflate_for_stroke_referencia(path: &VecPath, xf: Affine, r: Rect) -> Rect {
    let mut r = r;
    {
        let [a, b, c, d, _, _] = xf.as_coeffs();
        let sx = (a * a + b * b).sqrt();
        let sy = (c * c + d * d).sqrt();
        let mut m = 0.0_f64;
        for camada in path.paint_stack() {
            let ph2d_vec_scene::PaintRef::Stroke(s) = camada.paint else {
                continue;
            };
            let k = kurbo_stroke(s, None);
            let reach = if matches!(k.join, Join::Miter) {
                k.miter_limit.max(1.0)
            } else {
                1.0
            };
            m = m.max(0.5 * s.width * sx.max(sy) * reach);
        }
        let (mut ox, mut oy) = (0.0_f64, 0.0_f64);
        for camada in path.paint_stack() {
            let [dx, dy] = camada.offset;
            ox = ox.max((dx * sx).abs());
            oy = oy.max((dy * sy).abs());
            if camada.dilate > 0.0 {
                ox = ox.max(camada.dilate * sx);
                oy = oy.max(camada.dilate * sy);
            }
        }
        if m > 0.0 || ox > 0.0 || oy > 0.0 {
            r = r.inflate(m + ox, m + oy);
        }
    }
    r
}

/// **Desenha exatamente UM caminho, como o [`dispatch`] o desenharia, transladado por `offset`
/// (px de tela)** — a rasterização da forma isolada que o produtor de FX (plano 24) lê de volta.
///
/// Honra a geometria DERIVADA (`live`) e a pose, igual ao `dispatch`, então o que o FX borra é
/// exatamente o que a forma É na tela; o `offset` leva o bbox da forma à origem `(0,0)` do scratch.
/// Passa pela MESMA [`draw_path`] do `dispatch` — desenhar por uma 2ª porta faria o FX divergir do
/// que a forma parece de verdade.
///
/// ⛔⛔ **REPORT DO ENIO (2026-08-27): *"filters anula pattern"*.** O `tile` é obrigatório e não
/// opcional por isso: sem ele esta função chamava a `draw_path`, que passa `None`, e uma forma com
/// padrão era rasterizada com a **cor de recurso**. Como no `dispatch` a imagem de FX **toma o
/// lugar** do desenho, ligar um filtro apagava o padrão.
///
/// ⚠️ O doc acima já declarava a lei que isso partia — *"passa pela MESMA `draw_path`"* —, e a
/// segunda porta apareceu **dentro** da primeira quando o ladrilho virou um parâmetro que só o
/// `dispatch` sabia preencher. *Um argumento novo com um default é uma porta nova sem nome.*
///
/// ⛔⛔⛔ **REPORT DO ENIO (2026-09-04): *"a da direita não ficou transparente"*.** A MESMA falha,
/// uma tinta adiante: `bound` é obrigatório pelo mesmo motivo que o `tile` é. A opacidade viva, a
/// cor de um token e a espessura de um token vivem num [`BoundStyle`], que **não é campo do
/// `VecPath`** — o `dispatch` aplica-o com `painted()` no ponto de desenho, e esta porta desenhava
/// o AUTORADO. Uma forma filtrada que a linha do tempo desvanece re-cozinhava a cada quadro (a
/// chave do memo já carregava o estilo) e re-cozinhava **os mesmos pixels opacos**.
///
/// ⚠️ *Corrigir a CHAVE de um memo e não corrigir o DESENHO deixa o defeito intacto e gasta o
/// relógio a re-produzi-lo* — o miss passa a acontecer todo quadro e a resposta não muda.
#[allow(clippy::too_many_arguments)]
pub fn draw_path_isolated(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    patterns: &crate::PatternTiles,
    brushes: &crate::BrushArts,
    // O estilo resolvido DESTA forma neste quadro (`VecViewState::bound_style`). `None` = desenhe
    // o autorado — e é a resposta certa para quem não tem projecção de quadro nenhuma, nunca um
    // atalho para quem tem.
    bound: Option<&BoundStyle>,
    id: VecPathId,
    camera: Affine,
    offset: Affine,
    target: &mut VectorScene,
) {
    let tile = patterns.get(&(id, crate::PatternSlot::Fill));
    let stroke_tile = patterns.get(&(id, crate::PatternSlot::Stroke));
    let art = brushes.get(&id).map(Vec::as_slice);
    if let Some(items) = live.get(&id) {
        for item in items {
            // A derivada leva o MESMO estilo da fonte, como no `dispatch`: as cópias de
            // offset/pattern/espelho têm id próprio, então o estilo pára na borda do 1.º efeito se
            // for procurado por elas.
            crate::draw_path_tiled(
                &item.painted(bound),
                offset * camera,
                target,
                crate::Derived {
                    tile,
                    stroke_tile,
                    brush_art: art,
                    dilated: None,
                },
            );
        }
    } else if let Some(path) = scene.paths().iter().find(|p| p.id == id) {
        crate::draw_path_tiled(
            &path.painted(bound),
            offset * path_to_screen(xforms, id, camera),
            target,
            crate::Derived {
                tile,
                stroke_tile,
                brush_art: art,
                dilated: None,
            },
        );
    }
}
