//! ⭐⭐⭐ **A SUBDIVISÃO DO BIND — os pontos nascem VISÍVEIS quando a forma é presa.**
//!
//! Ordem do dono (2026-09-19): *«sem saber onde os pontos estão não fica legal. Melhor criar a
//! subdivisão visível logo na associação com os ossos»*. É a lei que a 2.ª mídia já tinha — uma
//! imagem presa ganha uma **malha graduada pelas articulações** no bind —, agora também para uma
//! forma vectorial.
//!
//! # ⛔⛔ O que ela substitui
//!
//! A barra da cena do dono tem **oito** nós, os oito nas duas pontas. Pintar peso no meio dela era
//! pintar num sítio sem ponto nenhum à vista: o artista não sabe onde o peso mora, e o indicador
//! mostra oito pontos parados enquanto a arte dobra. *Uma ferramenta cujo efeito não tem onde ser
//! visto lê-se como partida.*
//!
//! # ⭐⭐ O alvo é DERIVADO, e a régua é a lei que já existe
//!
//! O passo é **o osso mais curto a dividir por [`DIVISOES_POR_OSSO`]**. ⚠️ *A lei é a do peso, não
//! a do desenho:* a escala fina de um campo de pesos é o comprimento de um osso, e é por isso que
//! o passo se mede contra ele e não contra o tamanho da forma.
//!
//! # ⛔⛔⛔ O `K = 3` foi calibrado numa grandeza que os DOIS lados partilhavam (corrigido 2026-09-20)
//!
//! A 1.ª redacção escolheu `3` como *«o menor valor em que a lei dos pontos de controlo passa a
//! concordar com a lei da CURVA dentro da [`ph2d_vec_skin::curva::TOLERANCIA`]»*, e mediu:
//!
//! | ossos | osso | `K=1` | `K=2` | **`K=3`** |
//! |---|---|---|---|---|
//! | 2 | `3,200` | — | `0,0606` | **`0,0000`** |
//! | 3 | `2,133` | `0,2100` | `0,0496` | **`0,0000`** |
//! | 4 | `1,600` | `0,0699` | `0,0000` | **`0,0000`** |
//! | 6 | `1,067` | `0,0567` | `0,0000` | **`0,0000`** |
//!
//! ⛔ **Aquela régua mede a CONCORDÂNCIA de dois caminhos, e ela é cega ao defeito que os DOIS
//! têm.** As duas leis concordam a `0,0000` **e as duas ONDULAM**: medido com a régua nova
//! ([`crate::skinned_mesh::ondulacao_regua_tests`]), o contorno sai com **seis serpentinas de
//! `4,9 %` da espessura da barra** — e a mídia IMAGEM, que é o padrão-ouro, tem `0,12 %` no mesmo
//! sítio. *Um zero de «as duas dão o mesmo» e um zero de «está liso» são o mesmo byte.*
//!
//! # ⭐⭐⭐ O `K` re-medido na grandeza que o dono vê, com o preço ao lado
//!
//! A barra a `90°` em S, a flecha MEDIANA das ondas mais curtas que a espessura da peça, e o
//! recook em `--release` (o que corre **todo quadro**, por forma):
//!
//! | `K` | nós | flecha p50 | recook | de um quadro |
//! |---|---|---|---|---|
//! | `3` (o que shipava) | `34` | **`0,04863`** | `267 µs` | `1,60 %` |
//! | `3,6` | `38` | `0,04863` | `298 µs` | `1,78 %` |
//! | `4,3` | `50` | `0,00345` | `400 µs` | `2,40 %` |
//! | **`5`** | **`54`** | **`0,00175`** | **`434 µs`** | **`2,60 %`** |
//! | `5,3` | `62` | `0,00175` | `493 µs` | `2,95 %` |
//! | `6` / `7,1` | `66` | `0,00175` | `520 µs` | `3,12 %` |
//! | `8` | `86` | `0,00095` | `679 µs` | `4,07 %` |
//!
//! ⇒ **`K = 5`** é o PLANALTO: ele corta a serpentina **`28×`** — até ao nível do próprio
//! padrão-ouro — por **`+167 µs`**, que é `1,0 %` de um quadro; e `K = 6` e `K = 7,1` leem
//! **exactamente o mesmo** número, logo o joelho não é escolhido, é onde a coluna deixa de descer.
//! ⚠️ `K = 8` compra mais `1,8×` por `+57 %` de relógio, e é aí que a curva vira preço.
//!
//! ⛔ **A coluna `flecha MÁX` NÃO converge** (`0,053` em toda a tabela, até `446` nós): essa é a
//! **QUINA** que a lei de mistura cria na aresta de dentro, e *uma cúbica nunca faz uma quina —
//! mais nós só a espremem*. Ela é assunto do [`ph2d_skeleton::MisturaDoAngulo`] e não deste passo.
//!
//! ⭐⭐ **E subdividir torna o desenho MAIS BARATO na outra ponta:** o refit da lei da curva só
//! corre onde a lei ingénua se afasta, e com a forma subdividida ele deixa de correr — medido em
//! `debug`, o recook da barra passou de **`732 µs`** (8 nós, refit a arder) para **`58 µs`**.
//!
//! # ⛔ O tecto, e de que recurso ele é
//!
//! [`VERTICES_MAX`] é o **relógio do recook**, que corre todo quadro. Medido em `--release` na
//! barra da cena:
//!
//! | vértices | recook | de um quadro |
//! |---|---|---|
//! | `510` | `64,9 µs` | `0,39 %` |
//! | `2 054` | `244,1 µs` | `1,46 %` |
//! | **`4 094`** | **`608,1 µs`** | **`3,65 %`** |
//! | `8 190` | `1 230,7 µs` | `7,38 %` |
//!
//! ⇒ um décimo de quadro compra **~11 100** vértices, e o tecto fica em `4096` porque o orçamento é
//! **partilhado pelas formas presas da cena**. ⚠️ **Ele é generoso e não apertado** — a barra da
//! cena sai com `34` — e o que ele impede é o caso degenerado: um osso minúsculo sobre uma forma
//! enorme. ⛔ **Medir isto em `debug` daria um tecto `11×` mais baixo** (`8 190` vértices custam lá
//! `7,4 %` de um quadro contra os `7,38 %` que aqui custam... em `release` são `0,67 %`): o §0.0
//! outra vez — *o caminho mais lento não define o tecto do mais rápido*.

use ph2d_vec_scene::VecPath;

/// Em quantos pedaços o osso mais curto é dividido. Ver o cabeçalho.
///
/// ⚠️ **`3 → 5` em 2026-09-20**, e a razão não é afinação: o `3` foi calibrado contra a
/// concordância de duas leis que **ondulam as duas**, e o `5` é o planalto da grandeza que o dono
/// vê. O gate que o defende é o
/// [`crate::skinned_mesh::ondulacao_regua_tests::a_serpentina_do_desenho_morre_com_os_nos`], e ele
/// leva o CONTROLO do `3` dentro — *sem ele, alguém baixa este número e nada fica vermelho.*
pub const DIVISOES_POR_OSSO: f64 = 5.0;

/// Quantos vértices uma forma presa pode ter. Ver o cabeçalho — o recurso é o **relógio do recook**.
pub const VERTICES_MAX: usize = 4096;

/// Quantas passagens de refinamento, no máximo.
///
/// ⚠️ Cada ronda divide ao meio todo segmento longo, logo `16` são `65 536×` de refinamento — o que
/// termina o laço é o [`VERTICES_MAX`] ou não haver segmento longo. *Ela existe para o laço ser
/// finito por construção, e não como orçamento.*
pub const RONDAS_MAX: usize = 16;

/// Quanto o RECUO de uma quina viva pode encolher para um corte ainda ser aceite.
///
/// ⛔⛔ **A grandeza é o recuo e não o desenho, e isso é a diferença entre um guarda `O(1)` e um
/// `O(n²)`**: a 1.ª redacção amostrava o desenho inteiro antes e depois de CADA corte e comparava
/// ponto a polilinha — numa forma com quinas vivas ela pendurou a suíte. ⭐ O que um corte pode
/// estragar é uma coisa só: o recuo da quina é clampado a **metade da menor corda vizinha**
/// ([`ph2d_vec_scene::corner_live::corner_at`]), logo partir o segmento ao lado dela encolhe-o.
/// *Medir a consequência custa o desenho todo; medir a CAUSA custa uma subtracção.*
///
/// ⚠️ **O `setback` NÃO muda com um corte interior** — de Casteljau preserva as tangentes das
/// pontas, e é delas que ele sai. Só o `max_setback` encolhe.
const RECUO_PERDIDO: f64 = 1e-9;

/// **O passo que este esqueleto pede**, no espaço da forma — `None` quando não há osso com
/// comprimento.
///
/// ⚠️ **Os eixos já vêm no espaço da forma** (é o que o `bind` resolve com a inversa da pose dela),
/// e tem de ser assim: o alvo compara-se com comprimentos de segmento, que vivem nesse espaço.
#[must_use]
pub fn alvo_dos_eixos(eixos: &[ph2d_skin_weights::Handle]) -> Option<f64> {
    let curto = eixos
        .iter()
        .map(|h| (h.b[0] - h.a[0]).hypot(h.b[1] - h.a[1]))
        .filter(|l| *l > 0.0 && l.is_finite())
        .fold(f64::INFINITY, f64::min);
    (curto.is_finite() && curto > 0.0).then_some(curto / DIVISOES_POR_OSSO)
}

/// O comprimento do polígono de controlo de um segmento — um **majorante** do arco.
///
/// ⚠️ Majorante de propósito: ele erra para o lado de subdividir a mais, que é o lado seguro.
/// *Uma estimativa que erra para baixo deixaria um segmento longo passar por curto.*
fn comprimento(a: &ph2d_vec_scene::VecVertex, b: &ph2d_vec_scene::VecVertex) -> f64 {
    let d = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).hypot(p[1] - q[1]);
    d(a.anchor, a.out_handle) + d(a.out_handle, b.in_handle) + d(b.in_handle, b.anchor)
}

/// Os comprimentos de todos os segmentos, por índice **plano** (o que o
/// [`ph2d_vec_scene::split_segment`] recebe).
fn comprimentos(p: &VecPath) -> Vec<f64> {
    let mut out = Vec::new();
    for c in 0..p.contour_count() {
        let Some((v, fechado)) = p.contour(c) else {
            continue;
        };
        let n = v.len();
        let ultimo = if fechado { n } else { n.saturating_sub(1) };
        for i in 0..ultimo {
            out.push(comprimento(&v[i], &v[(i + 1) % n]));
        }
        // ⚠️ Um contorno ABERTO tem `n-1` segmentos e o índice plano conta-os assim — ver o
        // `locate_segment`. Uma contagem a mais aqui apontaria o corte para o contorno seguinte.
    }
    out
}

/// **O recuo que a quina do vértice `i` de facto usa** — `min(o que o raio pede, o que a geometria
/// absorve)`. `0` quando aquele vértice não tem quina viva.
fn recuo(verts: &[ph2d_vec_scene::VecVertex], fechado: bool, i: usize) -> f64 {
    if verts.get(i).is_none_or(|v| v.corner_size() <= 0.0) {
        return 0.0;
    }
    ph2d_vec_scene::corner_live::corner_at(verts, fechado, i)
        .map_or(0.0, |f| f.setback.min(f.max_setback))
}

/// Os recuos das duas quinas que o segmento plano `seg` toca, ANTES de ele ser partido.
fn recuos_do_segmento(path: &VecPath, seg: usize) -> (f64, f64) {
    let Some((c, local)) = path.locate_segment(seg) else {
        return (0.0, 0.0);
    };
    let Some((v, fechado)) = path.contour(c) else {
        return (0.0, 0.0);
    };
    let n = v.len();
    (recuo(v, fechado, local), recuo(v, fechado, (local + 1) % n))
}

/// Os mesmos dois recuos DEPOIS do corte — o fim do segmento andou um índice.
fn recuos_depois_do_corte(path: &VecPath, seg: usize) -> (f64, f64) {
    let Some((c, local)) = path.locate_segment(seg) else {
        return (0.0, 0.0);
    };
    let Some((v, fechado)) = path.contour(c) else {
        return (0.0, 0.0);
    };
    let n = v.len();
    (recuo(v, fechado, local), recuo(v, fechado, (local + 2) % n))
}

/// ⭐⭐⭐ **SUBDIVIDE UMA FORMA ATÉ NENHUM SEGMENTO PASSAR DE `alvo`.** Devolve quantos cortes fez.
///
/// O corte é o de de Casteljau ([`ph2d_vec_scene::split_segment`]), que é **exacto**: a curva não
/// se mexe, só ganha um ponto de controlo no meio.
///
/// ⛔⛔ **A excepção é a QUINA VIVA, e ela é verificada e não presumida.** Cada corte ao lado de uma
/// quina é medido pelo RECUO que ela usa, e **revertido** se ele encolher — ver [`RECUO_PERDIDO`].
/// *Escrever a condição à mão — «não cortes ao lado de um vértice com raio» — recusaria todo corte
/// num rectângulo arredondado, que é a forma mais comum que existe.*
pub fn subdivide(path: &mut VecPath, alvo: f64, tecto: usize) -> usize {
    if !(alvo.is_finite() && alvo > 0.0) {
        return 0;
    }
    // ⛔ **Uma forma com EFEITOS não é subdividida, e é uma limitação DECLARADA.** A saída de um
    // efeito é função do contorno inteiro (aparar, tracejar, engrossar), e esta wave não a mediu —
    // *subdividir às cegas ali seria mudar o desenho sem saber de quanto*.
    if !path.effects.is_empty() {
        return 0;
    }
    let quinas = path.has_live_corner();
    let mut cortes = 0;
    for _ in 0..RONDAS_MAX {
        let longos: Vec<usize> = comprimentos(path)
            .into_iter()
            .enumerate()
            .filter(|(_, l)| *l > alvo)
            .map(|(i, _)| i)
            .collect();
        if longos.is_empty() {
            break;
        }
        // ⚠️ **Em ordem DECRESCENTE**: um corte insere um vértice e empurra todos os índices planos
        // a seguir dele. Descendo, os que faltam continuam válidos.
        for i in longos.into_iter().rev() {
            if path.verts.len() >= tecto {
                return cortes;
            }
            let antes = quinas.then(|| (path.clone(), recuos_do_segmento(path, i)));
            if ph2d_vec_scene::split_segment(path, i, 0.5).is_none() {
                continue;
            }
            if let Some((copia, (ra, rb))) = antes {
                // ⚠️ Depois do corte o vértice novo está no MEIO: o fim do segmento andou um
                // índice. *Ler a quina no índice velho mediria a quina que acabou de nascer, que
                // não tem raio nenhum, e o guarda aprovaria tudo.*
                let (da, db) = recuos_depois_do_corte(path, i);
                if ra - da > RECUO_PERDIDO || rb - db > RECUO_PERDIDO {
                    *path = copia;
                    continue;
                }
            }
            cortes += 1;
        }
    }
    cortes
}

#[cfg(test)]
#[path = "subdivisao_tests.rs"]
mod subdivisao_tests;
