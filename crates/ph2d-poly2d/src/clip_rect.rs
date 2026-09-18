//! ⭐⭐⭐ **A SUB-MALHA DENTRO DE UM RECTÂNGULO** — cortar uma malha por quatro linhas alinhadas aos
//! eixos, levando os atributos por vértice consigo.
//!
//! # Por que existe (F11, 2026-09-17)
//!
//! Um sprite com **9-slice** desenha-se em até nove quads, e cada um mostra um pedaço DIFERENTE da
//! arte esticado numa caixa de tamanho próprio (é essa a razão de existir do 9-slice: o canto não
//! estica, a borda estica num eixo, o miolo nos dois). Uma imagem presa a ossos precisa então de
//! **uma malha por pedaço**, cortada nas linhas das fatias — e é isso que esta porta faz.
//!
//! ⛔⛔ **A alternativa — recortar o QUAD em vez da malha — está refutada por construção:** o pedaço
//! do meio mostra a faixa central da imagem ESTICADA, então a tinta que lá está não é a tinta que o
//! mapeamento de repouso poria ali. *O corte tem de ser no espaço da IMAGEM e o mapa para o mundo
//! tem de ser o do pedaço.*
//!
//! # ⭐⭐ A lei do corte: quatro divisões por LINHA, nunca um recorte de polígono
//!
//! Dividir um triângulo por uma recta dá 1..3 triângulos e **todo vértice novo nasce numa aresta do
//! triângulo que se dividiu**. Fazendo isso quatro vezes (uma por lado do rectângulo) e deitando
//! fora o lado de fora, nunca se constrói um polígono convexo nem se precisa de o triangular.
//!
//! ⚠️⚠️ **E o vértice novo é calculado a partir da ponta LEXICOGRAFICAMENTE MENOR, sempre.** Dois
//! triângulos vizinhos partilham a aresta e percorrem-na em sentidos opostos; `a + t·(b−a)` e
//! `b + (1−t)·(a−b)` **não** dão os mesmos bits em `f64`, e com bits diferentes a soldadura final
//! deixaria uma FENDA entre os dois — um fio de fundo a atravessar a arte. *A canonicalização é a
//! mesma que a ordem total das arestas do [`crate::refine_adaptive`] já paga, pela mesma razão.*
//!
//! # ⭐ A identidade é um curto-circuito, e é ela que protege toda sprite normal
//!
//! Um rectângulo que contém a malha inteira devolve-a **sem lhe tocar** — mesmos índices, mesmos
//! bits, mesma ordem. É o caminho de toda imagem presa que não é um 9-slice, e ele não passa por
//! nenhuma aritmética deste módulo.

use crate::Mesh2d;

/// Um vértice em trânsito: a posição e os atributos dele.
///
/// ⚠️ Os atributos viajam **por valor** de propósito: um índice para a tabela de origem não sabe
/// responder por um vértice que nasceu no corte.
#[derive(Clone)]
struct V {
    p: [f64; 2],
    a: Vec<f64>,
}

impl V {
    /// O ponto onde o segmento `self → o` cruza `eixo = c`, com os atributos interpolados.
    ///
    /// ⚠️ **A conta parte sempre da ponta menor** (ver o cabeçalho): é o que faz dois triângulos
    /// vizinhos escreverem o MESMO ponto, bit a bit, e a soldadura fechar.
    fn corte(&self, o: &Self, eixo: usize, c: f64) -> Self {
        let (lo, hi) = if (self.p[0], self.p[1]) <= (o.p[0], o.p[1]) {
            (self, o)
        } else {
            (o, self)
        };
        let d = hi.p[eixo] - lo.p[eixo];
        let t = if d == 0.0 { 0.0 } else { (c - lo.p[eixo]) / d };
        let mut p = [
            lo.p[0] + t * (hi.p[0] - lo.p[0]),
            lo.p[1] + t * (hi.p[1] - lo.p[1]),
        ];
        // ⭐ O eixo do corte é **pregado** ao valor exacto: a interpolação pode errar um ulp, e um
        // ulp aqui põe o vértice do lado de fora da divisão seguinte.
        p[eixo] = c;
        V {
            p,
            a: lo
                .a
                .iter()
                .zip(&hi.a)
                .map(|(l, h)| l + t * (h - l))
                .collect(),
        }
    }
}

/// Divide `tris` pela recta `eixo = c`, ficando com o lado que `manter_maior` escolhe.
fn dividir(tris: Vec<[V; 3]>, eixo: usize, c: f64, manter_maior: bool) -> Vec<[V; 3]> {
    let dentro = |v: &V| {
        if manter_maior {
            v.p[eixo] >= c
        } else {
            v.p[eixo] <= c
        }
    };
    let mut out = Vec::with_capacity(tris.len());
    for t in tris {
        let marca = [dentro(&t[0]), dentro(&t[1]), dentro(&t[2])];
        let n = marca.iter().filter(|m| **m).count();
        match n {
            0 => {}
            3 => out.push(t),
            // UM dentro: sobra um triângulo, com dois vértices novos nas arestas que saem dele.
            1 => {
                let i = marca.iter().position(|m| *m).expect("um esta' dentro");
                let (a, b, c2) = (&t[i], &t[(i + 1) % 3], &t[(i + 2) % 3]);
                out.push([a.clone(), a.corte(b, eixo, c), a.corte(c2, eixo, c)]);
            }
            // DOIS dentro: sobra um quadrilátero, partido em dois triângulos.
            _ => {
                let i = marca.iter().position(|m| !*m).expect("um esta' fora");
                let (fora, a, b) = (&t[i], &t[(i + 1) % 3], &t[(i + 2) % 3]);
                let (ca, cb) = (fora.corte(a, eixo, c), fora.corte(b, eixo, c));
                out.push([ca.clone(), a.clone(), b.clone()]);
                out.push([ca, b.clone(), cb]);
            }
        }
    }
    out
}

/// ⭐⭐⭐ **A SUB-MALHA DE `mesh` DENTRO DE `rect`**, com `n_attr` atributos por vértice levados
/// consigo (interpolados linearmente em cada corte).
///
/// `rect` é `[x0, y0, x1, y1]` nas unidades do [`Mesh2d::rest`]. Devolve `None` quando nada
/// sobrevive — *e `None` é a resposta certa, porque o chamador tem de saber que aquele pedaço não
/// desenha nada em vez de desenhar uma malha vazia*.
///
/// ⭐ **Um rectângulo que contém a malha inteira devolve-a BYTE A BYTE** (ver o cabeçalho).
///
/// ⚠️ `attrs` é achatado (`n_attr` por vértice, na ordem de `mesh.rest`); vazio ⇒ a saída também é
/// vazia, que é a tabela «sem pesos guardados» que a lei derivada usa.
#[must_use]
pub fn submesh_in_rect(
    mesh: &Mesh2d,
    attrs: &[f64],
    n_attr: usize,
    rect: [f64; 4],
) -> Option<(Mesh2d, Vec<f64>)> {
    if mesh.rest.is_empty() || mesh.tris.is_empty() {
        return None;
    }
    // ⭐ O curto-circuito da identidade: se a malha inteira cabe, nada aqui lhe toca.
    let dentro_de_todos = mesh
        .rest
        .iter()
        .all(|p| p[0] >= rect[0] && p[0] <= rect[2] && p[1] >= rect[1] && p[1] <= rect[3]);
    if dentro_de_todos {
        return Some((mesh.clone(), attrs.to_vec()));
    }
    let atr = |v: usize| -> Vec<f64> {
        attrs
            .get(v * n_attr..(v + 1) * n_attr)
            .unwrap_or(&[])
            .to_vec()
    };
    let mut tris: Vec<[V; 3]> = mesh
        .tris
        .iter()
        .map(|t| {
            t.map(|i| V {
                p: mesh.rest[i as usize],
                a: atr(i as usize),
            })
        })
        .collect();
    for (eixo, c, maior) in [
        (0, rect[0], true),
        (0, rect[2], false),
        (1, rect[1], true),
        (1, rect[3], false),
    ] {
        tris = dividir(tris, eixo, c, maior);
        if tris.is_empty() {
            return None;
        }
    }
    // ⭐⭐ **A SOLDADURA é por BITS da posição**, e é legítima só por causa da canonicalização do
    // [`V::corte`]: sem ela dois vizinhos escreveriam o mesmo ponto com o último bit diferente e a
    // malha sairia com uma fenda invisível em todo teste de área.
    let mut chaves: std::collections::BTreeMap<[u64; 2], u32> = std::collections::BTreeMap::new();
    let mut rest = Vec::new();
    let mut saida_attrs = Vec::new();
    let mut fora = Vec::with_capacity(tris.len());
    for t in &tris {
        // ⛔ Um triângulo de área NULA não se emite: a divisão produz slivers na linha do corte, e
        // eles não desenham nada e ainda contam para todo orçamento de peças.
        let (a, b, c) = (t[0].p, t[1].p, t[2].p);
        let area2 = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if area2 == 0.0 {
            continue;
        }
        let mut idx = [0u32; 3];
        for (k, v) in t.iter().enumerate() {
            let chave = [v.p[0].to_bits(), v.p[1].to_bits()];
            idx[k] = *chaves.entry(chave).or_insert_with(|| {
                rest.push(v.p);
                saida_attrs.extend_from_slice(&v.a);
                (rest.len() - 1) as u32
            });
        }
        fora.push(idx);
    }
    if fora.is_empty() {
        return None;
    }
    Some((
        Mesh2d {
            rest,
            tris: fora,
            size: mesh.size,
        },
        saida_attrs,
    ))
}

#[cfg(test)]
#[path = "clip_rect_tests.rs"]
mod tests;
