//! ⭐⭐⭐ **DE QUEM É CADA FACE** (plano 40 §11) — resolvido pelas **ÂNCORAS**, sem memória nenhuma
//! do quadro anterior.
//!
//! Módulo irmão do [`crate::vec_bucket`], e o corte é por RESPONSABILIDADE: aquele responde
//! *"quando se recoze"*, este responde *"quem herda o quê"*.
//!
//! # ⛔⛔ O modelo anterior, e porque ele foi SUBSTITUÍDO
//!
//! Report do Enio, quatro vezes seguidas (2026-09-01 e 02): a tinta partia-se ao atravessar uma
//! linha, sumia, deixava resíduo, trocava de área. Cada defeito teve a sua cura, e o report voltava.
//!
//! ⚠️⚠️ **A causa não era nenhum deles: era o modelo.** A receita de um preenchimento era *a região
//! que ele pintou no quadro anterior*, e o dono de cada face saía de comparar as faces de hoje com
//! as regiões de ontem — uma votação por amostragem de área. Isso **deriva**: o que um quadro
//! decide vira a régua do seguinte, e um único quadro de topologia confusa reatribui a tinta **para
//! sempre**, porque nada puxa de volta. Medido nos SVG que ele exportou: a partir do estado
//! `drawing01`, o corpo do círculo direito vota **azul** — o app tinha-o **verde**, ganho num quadro
//! intermédio e nunca devolvido.
//!
//! # A lei nova
//!
//! > **Uma região é o lado de um conjunto de pedaços de linha** — e não uma coordenada, nem uma
//! > área, nem o que estava lá antes.
//!
//! No clique, o balde grava as **âncoras** da face: para cada arco que a cerca, *de que contorno de
//! que caminho ele é um pedaço*, *em que fracção*, e *de que lado*. Arrastar um nó move a curva
//! **sem mudar de que curva o pedaço é** ⇒ a face segue.
//!
//! ⭐⭐⭐ **É STATELESS.** Cada quadro resolve-se do documento sozinho: *o mesmo desenho dá sempre as
//! mesmas cores, seja qual for o caminho por que se lá chegou.* É a propriedade que os quatro
//! reports pediam, e a única que nenhuma heurística sobre o quadro anterior podia dar.
//!
//! ⭐ **E o resto cai de graça:** uma região que se PARTE deixa umas âncoras a cercar uma metade e
//! outras a outra — o preenchimento fica com as duas, sem uma linha de código sobre partir. Uma
//! FUSÃO põe as âncoras de dois na mesma face — ganha quem tem mais lá. Uma área NOVA não tem
//! âncora nenhuma — fica por pintar, que é o que o artista espera de uma região que nunca visitou.

use ph2d_ecs::FillAnchor;
use ph2d_vec_fill::{Face, Rede};

/// A receita de um preenchimento: as âncoras e o ponto do clique.
pub(crate) struct Receita<'a> {
    pub(crate) ancoras: &'a [FillAnchor],
    pub(crate) semente: [f64; 2],
}

/// ⭐⭐⭐ **A RESOLUÇÃO**: por face, o índice do preenchimento dono — ou `None`.
///
/// `tags[i]` diz de que `(caminho, contorno)` veio o contorno `i` da lista que construiu a rede; é
/// por ele que uma âncora reencontra o arco dela.
///
/// ⚠️ **A SEMENTE é a rede de segurança, não a lei.** Ela só entra quando **nenhuma** âncora
/// resolveu — o que acontece quando o artista refez as linhas (uma solda nova, um corte) e os
/// contornos que as âncoras nomeavam deixaram de existir. ⛔ Ela **não** se re-semeia: reescrevê-la
/// a cada quadro seria reintroduzir a deriva com outro nome.
///
/// ⚠️ **O empate desce ao índice do documento** — ao acaso, a cor piscaria entre duas enquanto a
/// mão treme.
pub(crate) fn donos(
    rede: &Rede,
    faces: &[Face],
    tags: &[(u64, u16)],
    fills: &[Receita],
) -> Vec<Option<usize>> {
    let mut votos: Vec<Vec<usize>> = vec![vec![0; fills.len()]; faces.len()];
    for (k, r) in fills.iter().enumerate() {
        let mut achou = false;
        for a in r.ancoras {
            let Some(origem) = tags.iter().position(|t| *t == (a.path, a.contorno)) else {
                continue; // o caminho ou o contorno deixaram de existir
            };
            let Some(arco) = rede.arco_em(origem, f64::from(a.frac)) else {
                continue;
            };
            let Some(fi) = rede.face_de(faces, arco, a.frente) else {
                continue; // do outro lado está a face de FORA
            };
            votos[fi][k] += 1;
            achou = true;
        }
        if !achou
            && let Some(f) = rede.face_em(r.semente)
            && let Some(fi) = faces.iter().position(|g| *g == f)
        {
            votos[fi][k] += 1;
        }
    }
    votos
        .iter()
        .map(|v| {
            let max = v.iter().copied().max().unwrap_or(0);
            (max > 0)
                .then(|| v.iter().position(|x| *x == max))
                .flatten()
        })
        .collect()
}

/// ⭐⭐ **AS FACES DE CADA PREENCHIMENTO, a MAIOR à frente.**
///
/// ⚠️ **A ordem é load-bearing**: a primeira face vira o contorno **primário** do caminho e as
/// outras os `subpaths`. ⛔ Um preenchimento pode ganhar VÁRIAS faces, e é esse o ponto — é assim
/// que uma região que se partiu fica com as duas metades.
pub(crate) fn por_preenchimento(
    faces: &[Face],
    donos: &[Option<usize>],
    quantos: usize,
) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); quantos];
    for (i, dono) in donos.iter().enumerate() {
        if let Some(k) = dono
            && let Some(lista) = out.get_mut(*k)
        {
            lista.push(i);
        }
    }
    for lista in &mut out {
        lista.sort_by(|a, b| faces[*b].area.total_cmp(&faces[*a].area));
    }
    out
}

/// ⭐⭐⭐ **AS ÂNCORAS DE UMA FACE** — o que o clique grava.
///
/// Uma por arco do ciclo que a cerca: o contorno de origem (traduzido pelo `tags` para
/// `(caminho, contorno)`), o meio da fatia, e o lado.
///
/// ⚠️ **Várias POR ARCO, e não uma.** É a redundância que faz a receita sobreviver a apagar uma das
/// linhas — e é a mesma redundância que reparte a tinta pelas duas metades quando a região se parte.
///
/// ⛔⛔ **Uma por arco NÃO chega, e a medição foi imediata:** um contorno fechado que não cruza
/// ninguém entra na rede como **um único arco** (um laço), então a face que ele cerca daria **uma**
/// âncora — e ao ser partida por uma linha nova só a metade que calhasse conter aquela fracção
/// herdava a tinta. *A redundância tem de estar ao longo da FRONTEIRA, não na contagem de arcos.*
///
/// ⚠️⚠️ **E espalhadas por COMPRIMENTO ABSOLUTO, não por face.** Normalizar por face dá a **todas** o
/// mesmo número — e aí a FUSÃO, que decide por *quem tem mais âncoras na face*, entregaria a região
/// fundida à tira fina tanto quanto à larga (medido: `16` contra `16`). Com um passo absoluto, quem
/// cercava mais traz mais, que é a lei do *Live Paint*.
///
/// ⚠️ O passo sai da **própria rede** (a soma de todos os arcos a dividir por isto), e não de um
/// número de mundo: a escala do documento é do artista.
const PASSOS_NA_REDE: f64 = 128.0;

pub(crate) fn ancoras_da_face(rede: &Rede, tags: &[(u64, u16)], face: &Face) -> Vec<FillAnchor> {
    let passo = (0..rede.arcos.len())
        .map(|i| rede.comprimento(i))
        .sum::<f64>()
        .max(f64::MIN_POSITIVE)
        / PASSOS_NA_REDE;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    face.arcos
        .iter()
        .filter_map(|&(i, frente)| {
            let a = rede.arcos.get(i)?;
            let (path, contorno) = *tags.get(a.origem)?;
            // ⚠️ **Pelo menos UMA por arco**: um arco curto ainda é fronteira, e sem âncora nenhuma
            // ele deixaria de reconhecer a face quando os vizinhos dele mudassem.
            let m = ((rede.comprimento(i) / passo).round() as usize).max(1);
            Some((0..m).map(move |j| FillAnchor {
                path,
                contorno,
                frac: a.em((j as f64 + 0.5) / m as f64) as f32,
                frente,
            }))
        })
        .flatten()
        .collect()
}

#[cfg(test)]
#[path = "vec_bucket_claim_tests.rs"]
mod tests;
