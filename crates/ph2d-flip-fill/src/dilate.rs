//! **A DILATAÇÃO do contorno** — com que largura o contorno que o solver devolve é
//! DESENHADO, para a cor entrar por baixo do line-art (BUGS #15).
//!
//! # Por que isto mora aqui, e não no shell
//!
//! Morou no shell até 2026-07-18, e o preço foi **oito oráculos de pixel cegos**: o
//! `gpu_fill_fit` (a suíte que rasteriza a cena e MEDE o encaixe da cor na linha —
//! inclusive um gate chamado *"a cor nunca transborda para fora da linha"*) montava a
//! **própria** dilatação, com uma cópia da constante e uma cópia da fórmula. Os dois
//! números eram os mesmos por acordo tácito, não por construção.
//!
//! O resultado é a assinatura do problema: quando a dilatação do PRODUTO ficou 100×
//! grande demais (BUGS #20 — uma constante em px somada a uma largura em unidades de
//! mundo), **os oito gates continuaram verdes**, porque nenhum deles olhava para o
//! número do produto. O defeito foi achado por um humano olhando a tela.
//!
//! Um oráculo que reconstrói o que deveria verificar não verifica nada: ele afirma que
//! *a sua própria* aritmética é consistente. A lei mora aqui — junto de quem produz o
//! contorno — para que a pergunta *"que largura o contorno veste?"* tenha **uma
//! resposta** e o oráculo tenha de perguntá-la em vez de respondê-la.
//!
//! (Irmão de `feedback_two_doors_to_the_same_question_diverge`.)
//!
//! **A REFERÊNCIA da dilatação é o Draw:Filled** — e ele não dilata nada.
//!
//! > *"Diferente do Draw:Filled que faz exatamente como eu estou dizendo."* (Enio,
//! > 2026-07-18)
//!
//! O Draw:Filled põe `fill` no PRÓPRIO traço: a cor é a triangulação dos pontos da linha,
//! então ela termina **no eixo** e a metade externa do traço composita sobre o que houver
//! atrás. Zero dilatação — e é o desenho aprovado. É também o que a rota
//! `filled_shape_target` do balde já fazia para UMA forma fechada, sem ninguém reclamar.
//!
//! A rota do contorno dilatava por `w` (a espessura da linha), levando a cor até o raio
//! GEOMÉTRICO. Medido contra a referência (`probe_bucket_vs_draw_filled`, pincel macio,
//! escala do produto — pior delta de canal e nº de pixels que diferem de mais de 8/255):
//!
//! | linha | dureza | `w + 2s` (a lei antiga) | **`2s`** (esta) |
//! |---|---|---|---|
//! | 8 px | 0,80 | 166 · 2.721 px | **48 · 29 px** |
//! | 16 px | 0,80 | 166 · 5.623 px | **14 · 8 px** |
//! | 32 px | 0,80 | 166 · 11.685 px | **3 · 0 px** |
//! | 32 px | 0,50 | 166 · 12.223 px | **15 · 11 px** |
//! | 32 px | 1,00 | 76 · 435 px | **0 · 0 px** |
//!
//! Com pincel DURO a lei nova é **byte-idêntica** ao Draw:Filled. Com pincel macio a lei
//! antiga difere em doze mil pixels — é a franja que o Enio viu em quatro smokes seguidos.
//!
//! # Por que o `w` estava ali, e por que ele é contagem dupla
//!
//! O **BUGS #15** o introduziu para que a metade externa da linha (translúcida, num
//! pincel macio) tivesse cor por baixo. Mas o defeito que ele curava não era a metade
//! externa: era o **contorno caindo AQUÉM do eixo** — o traçado sai de um marching
//! squares + RDP, então ele erra em torno do eixo, e onde ele errava para dentro sobrava
//! um fio de linha sobre o fundo.
//!
//! O termo `2s` passou a curar exatamente isso, **por ponto e com sinal**. A partir dali o
//! `w` não corrigia mais nada — só empurrava a cor além da silhueta. A tabela mostra a
//! prova: a coluna `zero` (sem `w` E sem `2s`) é **pior** que a `2s` em toda linha do
//! varrimento, então a compensação se paga; o `w` só transborda.
//!
//! É a terceira vez que este projeto paga por isto: ao acrescentar uma defesa, pergunte o
//! que ela torna **desnecessário**. O mecanismo velho não fica errado — fica obsoleto, e
//! obsoleto não quebra gate nenhum ([[feedback_a_new_remedy_makes_the_old_one_double_counting]]).
//!
//! ⚠️ **A margem extra (`FILL_TUCK_FRACTION`) morreu junto**, e tinha de morrer: ela era
//! uma FRAÇÃO de `w`. Sem o termo que ela multiplicava, ela não tem o que significar.

use crate::Vec2;

/// **A linha que este ponto do contorno está vestindo**: `(espessura, distância)`, na
/// unidade do documento.
///
/// O contorno do balde termina no EIXO da linha (BUGS #14), então o line-art mais
/// próximo de um ponto do contorno é, por construção, a linha que ele veste.
///
/// `strokes` é a MESMA lista que o `fill_at` recebeu — `(pontos, MEIA-espessura por
/// ponto, fechado)`. Perguntar a dilatação à mesma lista que delimitou a região não é
/// conveniência: a versão anterior re-derivava o conjunto de linhas do documento com um
/// filtro **próprio** (`!hide_stroke`, contra o `!(hide_stroke && fill.is_some())` das
/// fronteiras), e os dois só concordavam por acidente — um fechamento de gap tem
/// espessura zero, então caía no filtro `w > 0` mais adiante. Acidente enumerado é a
/// forma que um bug futuro toma.
#[must_use]
pub fn local_line(strokes: &[(Vec<Vec2>, Vec<f32>, bool)], p: Vec2) -> Option<(f32, f32)> {
    nearest_on_axis(strokes, p).map(|(w, q)| {
        let (dx, dy) = (p.x - q.x, p.y - q.y);
        (w, (dx * dx + dy * dy).sqrt())
    })
}

/// A linha mais próxima **e o ponto exato do eixo** que a representa: `(espessura, q)`.
///
/// A distância sozinha não basta para compensar o erro de vetorização, porque ela não diz
/// **de que lado**. Quem quer o sinal precisa de `q`.
#[must_use]
pub fn nearest_on_axis(strokes: &[(Vec<Vec2>, Vec<f32>, bool)], p: Vec2) -> Option<(f32, Vec2)> {
    nearest_on_axis_indexed(strokes, p).map(|(w, q, ..)| (w, q))
}

/// [`nearest_on_axis`] com o ENDEREÇO do ponto: `(espessura, q, índice do traço, índice do
/// segmento)`.
///
/// Quem precisa disto é quem CAMINHA o eixo — o snap do Colorize: a projeção euclidiana
/// sozinha **pula o fundo de um V** do eixo (do lado côncavo de uma dobra, o ponto mais
/// próximo salta por cima do fundo), então seguir a linha exige saber ENTRE quais vértices
/// dela o `q` caiu. Irmã append-only da porta acima — a busca é UMA (esta); a outra delega.
#[must_use]
pub fn nearest_on_axis_indexed(
    strokes: &[(Vec<Vec2>, Vec<f32>, bool)],
    p: Vec2,
) -> Option<(f32, Vec2, usize, usize)> {
    let mut best: Option<(f32, f32, Vec2, usize, usize)> = None; // (dist, largura, q, traço, seg)
    for (si, (pts, half, closed)) in strokes.iter().enumerate() {
        // ⚠️ **Distância ao SEGMENTO, nunca ao VÉRTICE.** O eixo é a polilinha, não a
        // nuvem de pontos dela: um ponto do contorno pousado exatamente sobre o eixo,
        // mas no meio de dois vértices, fica a até meia-amostragem do vértice mais
        // próximo. Medir ao vértice faria a compensação pagar o **espaçamento da
        // amostragem** como se fosse erro de vetorização — e num traço de 64 amostras
        // isso é maior que o erro real (BUGS #18).
        for (i, a, b) in segments(pts, *closed) {
            let ab = Vec2::new(b.x - a.x, b.y - a.y);
            let l2 = ab.x * ab.x + ab.y * ab.y;
            let t = if l2 <= 0.0 {
                0.0
            } else {
                (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
            };
            let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
            let d = (dx * dx + dy * dy).sqrt();
            // A espessura interpolada ao longo do segmento (a pressão varia por ponto).
            // `half` guarda MEIA espessura (a convenção do `fill_at`); a dilatação veste
            // o diâmetro.
            let n = half.len();
            let (ha, hb) = (
                half.get(i).copied().unwrap_or(0.0),
                half.get((i + 1) % n.max(1)).copied().unwrap_or(0.0),
            );
            let w = 2.0 * (ha + (hb - ha) * t);
            if w > 0.0 && best.is_none_or(|(bd, ..)| d < bd) {
                best = Some((d, w, Vec2::new(a.x + t * ab.x, a.y + t * ab.y), si, i));
            }
        }
    }
    best.map(|(_, w, q, si, i)| (w, q, si, i))
}

/// A **normal EXTERNA** do anel em cada vértice, na ordem dos pontos.
///
/// O sinal sai da área com sinal (a orientação do anel), e não de um chute geométrico
/// tipo *"para longe do centroide"* — que só funciona em forma convexa, e uma região
/// preenchida à mão raramente é.
///
/// `pub` desde o 6º smoke do Colorize: o snap da borda ao eixo empurra os trechos
/// colados na linha até a face OPOSTA (a sobreposição sob a linha que o crave sempre
/// deu), e a direção do empurrão é ESTA normal — uma 2ª cópia da convenção divergiria
/// (o gate `the_outward_normal_points_away` pina a convenção aqui).
pub fn outward_normals(ring: &[Vec2]) -> Vec<Vec2> {
    let n = ring.len();
    // Área positiva = uma orientação; negativa = a outra. Qual delas é "anti-horária"
    // depende do eixo y apontar para cima ou para baixo, e este documento usa y para
    // BAIXO — então a constante certa é a que o gate `the_outward_normal_points_away`
    // fixa, medida num círculo, e não a que o livro de geometria assume.
    let orient = if crate::signed_area(ring) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    (0..n)
        .map(|i| {
            // Tangente pela DIFERENÇA CENTRAL (vizinho anterior → próximo): num anel
            // reamostrado ela é muito mais estável que a aresta de um lado só, e é a
            // mesma escolha que o cálculo da normal do relevo faz no Painter.
            let (a, b) = (ring[(i + n - 1) % n], ring[(i + 1) % n]);
            let t = Vec2::new(b.x - a.x, b.y - a.y);
            let len = (t.x * t.x + t.y * t.y).sqrt();
            if len <= 0.0 {
                return Vec2::new(0.0, 0.0);
            }
            Vec2::new(orient * t.y / len, -orient * t.x / len)
        })
        .collect()
}

/// **A LEI**, ponto a ponto: a largura com que cada ponto do contorno é desenhado.
///
/// Ela tem **um termo só**, e o termo não é a espessura da linha: é o erro de VETORIZAÇÃO
/// do contorno, com sinal. O alvo é pôr a borda da cor **no eixo** — nem aquém (sobra um
/// fio de linha sobre o fundo) nem além (a cor transborda a silhueta).
///
/// # Por que a largura corrige uma POSIÇÃO
///
/// O anel do fill é desenhado como um traço fechado, então uma largura `W` empurra a borda
/// externa da cor `W/2` para fora do anel. Se o anel caiu `s` aquém do eixo, `W = 2s` põe
/// a borda exatamente nele. É por isso que um erro de posição é pago em largura.
///
/// O sinal vem da geometria que já está na mão: a **normal externa do anel** contra a
/// direção do eixo. `s = (q − p) · n_out` é positivo quando o eixo está para FORA do
/// contorno (ficou aquém, a cor precisa avançar) e negativo quando ficou além. Onde o
/// contorno acertou o eixo (`s ≈ 0`) a largura é **zero** — e o balde desenha exatamente
/// o que o Draw:Filled desenha.
///
/// ⚠️ **A tentativa anterior (2026-07-18, revertida sem shipar) era `2d`** — o mesmo termo
/// com o sinal sempre positivo. Ela mede **pior** que uma margem fixa (0,0178 contra 0,005
/// na mediana) porque dobra o erro exatamente nos pontos que transbordaram: metade das
/// correções ia para o lado errado. Não era a ideia que estava errada, era o `d` nu.
///
/// ⚠️ **E não há mais termo `w`** — ver o comentário no topo do módulo: ele curava o
/// contorno-aquém, que é o que ESTE termo cura, e sobreviveu como contagem dupla paga em
/// pixels visíveis.
#[must_use]
pub fn contour_widths(strokes: &[(Vec<Vec2>, Vec<f32>, bool)], contour: &[Vec2]) -> Vec<f32> {
    let normals = outward_normals(contour);

    // Passo 1: o desvio COM SINAL até o eixo, ponto a ponto.
    let mut offsets = Vec::with_capacity(contour.len());
    for (i, &p) in contour.iter().enumerate() {
        // Sem linha por perto não há eixo para mirar, e portanto nada a corrigir: zero.
        // (Antes daqui saía a espessura MÉDIA do desenho — uma dilatação larga sem uma
        // linha que a justificasse.)
        let s = match nearest_on_axis(strokes, p) {
            Some((_, q)) => {
                let n = normals[i];
                (q.x - p.x) * n.x + (q.y - p.y) * n.y
            }
            None => 0.0,
        };
        offsets.push(s);
    }

    // Passo 2: **alisa o desvio ao longo do anel.**
    //
    // O erro que se quer compensar é de BAIXA frequência (trechos inteiros onde o
    // contorno ficou aquém do eixo); o que sobra por cima é o tremor de alta frequência
    // do próprio traçado, que o RDP e o alisamento binomial deixam. Corrigir ponto a
    // ponto sem separar os dois faz a largura **seguir o ruído** — e uma largura que
    // segue ruído desenha uma borda serrilhada, que é trocar um defeito por outro (a
    // lição da borda do Inflate, no Painter).
    //
    // É o mesmo binomial [1,2,1] cíclico que o `simplify_ring` já usa, pela mesma razão.
    smooth_ring(&mut offsets, OFFSET_SMOOTH_PASSES);

    offsets
        .iter()
        // O `max(0)` não é um teto disfarçado: largura negativa não existe. Um contorno
        // que passou do eixo já tem cor demais, e a resposta é não desenhar anel nenhum.
        .map(|&s| (2.0 * s).max(0.0))
        .collect()
}

/// Quantos passes de binomial alisam o desvio antes de ele virar largura.
///
/// **Medido**, não escolhido. O critério é a divergência da MESMA arte em duas escalas
/// (`the_same_art_at_the_products_scale_renders_the_same`), que é ruído puro — quanto
/// menor, menos a largura está seguindo o traçado em vez da forma:
///
/// | passes | pior delta entre escalas | transbordo (`sweep_tuck` margem 0: 8/16/32px) |
/// |---|---|---|
/// | 0 | 75 | 11 / 10 / 11 |
/// | **2** | **20** | **8 / 9 / 12** |
/// | 4 | 20 | 8 / 9 / 14 |
/// | 8 | 79 | 7 / 9 / 14 |
/// | 12 | 79 | 7 / 10 / 14 |
///
/// ⚠️ **A curva NÃO é monótona, e é por isso que ela foi medida em vez de arbitrada.** A
/// intuição ("mais alisamento = menos ruído") acerta até 4 e depois **inverte**: em 8
/// passes o desvio já perdeu a forma que devia corrigir, a compensação vira uma constante
/// local — quase a média que ela veio substituir — e o erro entre escalas volta a 79.
/// Um número escolhido no olho teria pousado em 8 com toda a confiança do mundo.
///
/// 2 ganha de 4 no traço grosso (12 contra 14 de transbordo em 32px) com o mesmo delta.
const OFFSET_SMOOTH_PASSES: usize = 2;

/// Binomial `[1,2,1]` **cíclico** — o anel não tem pontas.
fn smooth_ring(v: &mut [f32], passes: usize) {
    let n = v.len();
    if n < 3 {
        return;
    }
    let mut tmp = vec![0.0f32; n];
    for _ in 0..passes {
        for i in 0..n {
            tmp[i] = 0.25 * v[(i + n - 1) % n] + 0.5 * v[i] + 0.25 * v[(i + 1) % n];
        }
        v.copy_from_slice(&tmp);
    }
}

/// Os segmentos de uma polilinha — **um traço `closed` inclui a COSTURA** (último →
/// primeiro); um aberto NUNCA a ganha.
///
/// ⚠️ Isto **espelha `ph2d_flip::FlipStroke::segments()`**, que é a porta única daquela
/// pergunta para um traço do documento. Esta crate não conhece o documento (só depende
/// de `ph2d-core`), então o espelho é inevitável — e por isso ele é **pinado por um
/// gate no shell**, que é o único lugar onde os dois tipos coexistem
/// (`flip_fill_dilate` → `the_two_segment_walks_agree`). Espelho sem gate é como a
/// dilatação duplicada do `gpu_fill_fit` nasceu.
fn segments(pts: &[Vec2], closed: bool) -> impl Iterator<Item = (usize, Vec2, Vec2)> + '_ {
    let n = pts.len();
    let seam = closed && n >= 2;
    (0..n.saturating_sub(1))
        .chain(seam.then(|| n - 1))
        .map(move |i| (i, pts[i], pts[(i + 1) % n]))
}

#[cfg(test)]
#[path = "dilate_tests.rs"]
mod tests;
