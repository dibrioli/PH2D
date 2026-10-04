//! ⭐⭐⭐ **A INCLINAÇÃO DO RELEVO POR AMOSTRA** — o gradiente da altura levado
//! às amostras e lido pelos MESMOS pesos da cor ([`docs/3D/29`] §9).
//!
//! A altura é linear (triângulos) ou bilinear (quads) dentro de cada célula da
//! retícula, logo a derivada EXACTA dela é constante por célula: numa encosta
//! inclinada a luz lê-se em degraus do tamanho da célula. A cura é a das
//! normais por vértice (Gouraud/Phong): cada célula dá o seu gradiente às
//! amostras dos cantos, pesado pela ÁREA dela no objecto, e o fragmento
//! interpola os gradientes das amostras — contínuo dentro da face e ATRAVÉS das
//! arestas da malha, porque as amostras da fronteira são partilhadas.
//!
//! ⚠️ O gradiente é um vector do OBJECTO (o declive da altura por unidade de
//! comprimento), e é por isso que ele precisa das POSIÇÕES: quem esculpe a peça
//! muda-o sem tocar numa altura. Ele é DERIVADO — não entra no documento nem na
//! fila de desfazer.
//!
//! [`docs/3D/29`]: ../../../docs/3D/29_plano_o_relevo_do_impasto_na_peca.md

use std::sync::atomic::{AtomicUsize, Ordering as Ord};
use std::sync::{Condvar, Mutex};

use crate::relevo::ALTURA;
use crate::{Tinta, cantos as n_cantos, indice, sitio_quad, sitio_tri};

/// Abaixo deste número de células o trabalho corre na thread que chama: lançar
/// as threads custa mais do que ele (`docs/3D/29` §9.3, medido).
const LIMIAR_PARALELO: usize = 32_768;

/// Os gradientes das baricêntricas de um triângulo `(a, b, c)` no objecto e a
/// área dele — `∇βa = n × (c − b)/|n|²`, a mesma geometria do `tinta.wgsl`.
fn geometria(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> ([[f32; 3]; 3], f32) {
    let n = cruz(sub(b, a), sub(c, a));
    let dd = ponto(n, n);
    if dd <= 0.0 {
        return ([[0.0; 3]; 3], 0.0);
    }
    let ga = escala(cruz(n, sub(c, b)), 1.0 / dd);
    let gb = escala(cruz(n, sub(a, c)), 1.0 / dd);
    let gc = escala(soma(ga, gb), -1.0);
    ([ga, gb, gc], 0.5 * dd.sqrt())
}

/// O rascunho de UMA face: o endereço global de cada ponto da retícula dela
/// (`u32::MAX` onde não foi pedido) e o acumulador `[Σ A·g, Σ A]` local.
#[derive(Default)]
struct Rascunho {
    tab: Vec<u32>,
    acc: Vec<[f32; 4]>,
}

impl Tinta {
    /// ⭐⭐ **Cada célula da face no rascunho dela** — o gradiente EXACTO da
    /// célula no objecto, pesado pela área, somado nos pontos dos cantos. A lei
    /// que as duas portas das [`Inclinacoes`] partilham.
    ///
    /// `so_borda` visita só as células com um canto na fronteira da face — as
    /// únicas que uma face VIZINHA de uma mudança dá às amostras partilhadas —
    /// e só resolve os endereços dessa faixa.
    fn rascunho_da_face(
        &self,
        face: usize,
        cantos: &[u32],
        pos: &[[f32; 3]],
        alturas: &[[f32; 2]],
        so_borda: bool,
        r: &mut Rascunho,
    ) {
        let l = self.lado_da_face(face);
        let lf = l as f32;
        let w = l as usize + 1;
        let topo = self.topologia();
        r.tab.clear();
        r.tab.resize(w * w, u32::MAX);
        r.acc.clear();
        r.acc.resize(w * w, [0.0; 4]);
        let (tab, acc) = (&mut r.tab, &mut r.acc);
        let h = |tab: &[u32], k: usize| alturas[tab[k] as usize][ALTURA];
        let p = |c: usize| pos[cantos[c] as usize];
        if n_cantos(cantos) == 3 {
            // A chave de `(i, j, k)` é `i·w + j`.
            if so_borda {
                for i in 0..=l {
                    for j in 0..=l - i {
                        let k = l - i - j;
                        if i.min(j).min(k) <= 1 {
                            tab[i as usize * w + j as usize] =
                                indice(topo, face, sitio_tri(l, i, j, k), cantos);
                        }
                    }
                }
            } else {
                self.para_cada_amostra_tri(face, &cantos[..3], |s, (i, j, _)| {
                    tab[i as usize * w + j as usize] = s;
                });
            }
            let ([ga, gb, gc], area) = geometria(p(0), p(1), p(2));
            let a = area / (lf * lf);
            let mut celula = |c: [usize; 3], sinal: f32| {
                let g = soma(
                    soma(escala(ga, h(tab, c[0])), escala(gb, h(tab, c[1]))),
                    escala(gc, h(tab, c[2])),
                );
                let g = escala(g, sinal * lf);
                for k in c {
                    acumula(&mut acc[k], g, a);
                }
            };
            let ch = |i: u32, j: u32| i as usize * w + j as usize;
            for i in 0..l {
                for j in 0..l - i {
                    // A célula DIREITA de piso `(i, j, k)`, `i + j + k = L − 1`.
                    let k = l - 1 - i - j;
                    if !so_borda || i == 0 || j == 0 || k == 0 {
                        celula([ch(i + 1, j), ch(i, j + 1), ch(i, j)], 1.0);
                    }
                    // A INVERTIDA de piso `(i, j, k − 1)`, `i + j + k = L − 2`.
                    if k >= 1 && (!so_borda || i == 0 || j == 0 || k == 1) {
                        celula([ch(i, j + 1), ch(i + 1, j), ch(i + 1, j + 1)], -1.0);
                    }
                }
            }
            return;
        }
        // A chave de `(i, j)` é `j·w + i`.
        if so_borda {
            for j in 0..=l {
                for i in 0..=l {
                    if i.min(j).min(l - i).min(l - j) <= 1 {
                        tab[j as usize * w + i as usize] =
                            indice(topo, face, sitio_quad(l, i, j), cantos);
                    }
                }
            }
        } else {
            self.para_cada_amostra_quad(face, cantos, |s, (i, j)| {
                tab[j as usize * w + i as usize] = s;
            });
        }
        // O quad desenha-se em duas metades: `(a, b, c)` com `u = βb + βc`,
        // `v = βc`, e `(a, c, d)` com `u = βc`, `v = βc + βd` — os mesmos
        // `∇u`/`∇v` do `tinta_no_ponto4`.
        let ([ga0, _, gc0], a0) = geometria(p(0), p(1), p(2));
        let ([ga1, gb1, _], a1) = geometria(p(0), p(2), p(3));
        let (gu0, gv0) = (escala(ga0, -1.0), gc0);
        let (gu1, gv1) = (gb1, escala(ga1, -1.0));
        let ch = |i: u32, j: u32| j as usize * w + i as usize;
        for j in 0..l {
            for i in 0..l {
                if so_borda && i != 0 && j != 0 && i != l - 1 && j != l - 1 {
                    continue;
                }
                let c = [ch(i, j), ch(i + 1, j), ch(i + 1, j + 1), ch(i, j + 1)];
                let (ha, hb, hd, he) = (h(tab, c[0]), h(tab, c[1]), h(tab, c[2]), h(tab, c[3]));
                // A derivada da bilinear no CENTRO da célula.
                let hu = 0.5 * lf * ((hb - ha) + (hd - he));
                let hv = 0.5 * lf * ((he - ha) + (hd - hb));
                let g0 = soma(escala(gu0, hu), escala(gv0, hv));
                let g1 = soma(escala(gu1, hu), escala(gv1, hv));
                // Uma célula com `i > j` vive inteira na metade `(a, b, c)`; com
                // `i == j` a diagonal corta-a ao meio.
                let (g, area) = match i.cmp(&j) {
                    std::cmp::Ordering::Greater => (g0, 2.0 * a0),
                    std::cmp::Ordering::Less => (g1, 2.0 * a1),
                    std::cmp::Ordering::Equal => (escala(soma(g0, g1), 0.5), a0 + a1),
                };
                let a = area / (lf * lf);
                for k in c {
                    acumula(&mut acc[k], g, a);
                }
            }
        }
    }

    /// ⭐⭐⭐ **A inclinação num ponto baricêntrico de um triângulo** — os
    /// gradientes das amostras pelos pesos da cor. O gémeo da leitura do
    /// `tinta.wgsl`.
    #[must_use]
    pub fn inclinacao_tri(
        &self,
        face: usize,
        cantos: &[u32],
        bar: [f32; 3],
        g: &[[f32; 3]],
    ) -> [f32; 3] {
        let mut o = [0.0f32; 3];
        for (idx, peso) in self.pesos_tri(face, cantos, bar) {
            o = soma(o, escala(g[idx], peso));
        }
        o
    }

    /// A irmã para QUADS.
    #[must_use]
    pub fn inclinacao_quad(
        &self,
        face: usize,
        cantos: &[u32],
        uv: [f32; 2],
        g: &[[f32; 3]],
    ) -> [f32; 3] {
        let mut o = [0.0f32; 3];
        for (idx, peso) in self.pesos_quad(face, cantos, uv) {
            o = soma(o, escala(g[idx], peso));
        }
        o
    }
}

/// ⭐⭐⭐ **O gradiente de cada amostra, e a adjacência que o mantém em dia por
/// pedaços.**
///
/// ⚠️ Um traço muda poucas amostras por quadro; recalcular a peça inteira seria
/// pagar o plano todo por quadro. [`Self::atualiza`] refaz só as faces que as
/// amostras sujas tocam — e, para as amostras da fronteira delas, junta as
/// células das faces VIZINHAS, sem as quais a média da fronteira mentiria.
///
/// ⭐ Cada face soma num rascunho PRÓPRIO e os rascunhos juntam-se pela ORDEM
/// das faces: com muitas células as faces correm em paralelo, e o resultado é
/// o mesmo ao bit com qualquer número de threads.
#[derive(Debug, Clone, Default)]
pub struct Inclinacoes {
    g: Vec<[f32; 3]>,
    /// As faces de cada vértice, em CSR: `faces_v[ini_v[v]..ini_v[v + 1]]`.
    ini_v: Vec<u32>,
    faces_v: Vec<u32>,
    /// As (até) duas faces de cada aresta; `u32::MAX` onde falta uma.
    faces_a: Vec<[u32; 2]>,
    /// O acumulador `[Σ A·g, Σ A]`, válido só onde `marca == epoca`.
    acc: Vec<[f32; 4]>,
    marca: Vec<u32>,
    marca_face: Vec<u32>,
    epoca: u32,
    /// Quantas threads o espalhar pode usar (`0` = as da máquina).
    threads: usize,
}

impl Inclinacoes {
    /// ⭐ **O gradiente de todas as amostras** — `cantos_de(f)` dá os cantos da
    /// face `f` (com ou sem o sentinela [`crate::TRI`]) e `pos` as posições dos
    /// vértices. Um plano sem relevo dá gradientes nulos.
    #[must_use]
    pub fn nova<'a>(
        tinta: &Tinta,
        cantos_de: impl Fn(usize) -> &'a [u32] + Sync,
        pos: &[[f32; 3]],
    ) -> Self {
        Self::com_threads(tinta, cantos_de, pos, 0)
    }

    /// A [`Self::nova`] com o número de threads fixo — `1` corre tudo na thread
    /// que chama (a régua de que o paralelo dá o mesmo ao bit).
    #[must_use]
    pub fn com_threads<'a>(
        tinta: &Tinta,
        cantos_de: impl Fn(usize) -> &'a [u32] + Sync,
        pos: &[[f32; 3]],
        threads: usize,
    ) -> Self {
        let topo = tinta.topologia();
        let n_faces = topo.faces();
        let mut ini_v = vec![0u32; topo.verts() + 1];
        let mut faces_a = vec![[u32::MAX; 2]; topo.arestas()];
        for f in 0..n_faces {
            let c = cantos_de(f);
            for (s, &v) in c[..n_cantos(c)].iter().enumerate() {
                ini_v[v as usize + 1] += 1;
                let (id, _) = topo.aresta(f, s);
                let par = &mut faces_a[id as usize];
                if par[0] == u32::MAX {
                    par[0] = f as u32;
                } else {
                    par[1] = f as u32;
                }
            }
        }
        for v in 0..topo.verts() {
            ini_v[v + 1] += ini_v[v];
        }
        let mut cursor = ini_v.clone();
        let mut faces_v = vec![0u32; ini_v[topo.verts()] as usize];
        for f in 0..n_faces {
            let c = cantos_de(f);
            for &v in &c[..n_cantos(c)] {
                faces_v[cursor[v as usize] as usize] = f as u32;
                cursor[v as usize] += 1;
            }
        }
        let n = tinta.amostras().len();
        let mut me = Self {
            g: vec![[0.0; 3]; n],
            ini_v,
            faces_v,
            faces_a,
            acc: vec![[0.0; 4]; n],
            marca: vec![0; n],
            marca_face: vec![0; n_faces],
            epoca: 0,
            threads,
        };
        // ⭐ Só as faces com altura dão inclinação: uma célula sem altura em
        //   nenhum canto tem gradiente nulo, e as amostras de fora delas ficam
        //   em zero — o 1.º toque de impasto não paga o plano inteiro.
        if let Some(alt) = tinta.relevo() {
            let sujas: Vec<u32> = (0..n as u32)
                .filter(|&i| alt[i as usize][ALTURA] != 0.0)
                .collect();
            me.refaz(tinta, &cantos_de, pos, &sujas, &mut Vec::new());
        }
        me
    }

    /// O gradiente de cada amostra, em unidades de altura por unidade de objecto.
    #[must_use]
    pub fn por_amostra(&self) -> &[[f32; 3]] {
        &self.g
    }

    /// Estas inclinações ainda são do plano `tinta`? — a mesma contagem de
    /// amostras e de faces (a cerca barata; a forte é de quem troca o plano).
    #[must_use]
    pub fn serve(&self, tinta: &Tinta) -> bool {
        self.g.len() == tinta.amostras().len() && self.marca_face.len() == tinta.topologia().faces()
    }

    /// ⭐ **Refaz todas** — depois de a peça MUDAR DE FORMA (as posições entram
    /// em todo gradiente) ou de o relevo chegar inteiro.
    pub fn recalcula<'a>(
        &mut self,
        tinta: &Tinta,
        cantos_de: &(impl Fn(usize) -> &'a [u32] + Sync),
        pos: &[[f32; 3]],
    ) {
        let Some(alt) = tinta.relevo() else {
            self.g.fill([0.0; 3]);
            return;
        };
        self.acc.fill([0.0; 4]);
        let faces: Vec<u32> = (0..self.marca_face.len() as u32).collect();
        self.espalha(tinta, cantos_de, pos, alt, &faces, faces.len(), None);
        for (g, acc) in self.g.iter_mut().zip(&self.acc) {
            *g = media(*acc);
        }
    }

    /// ⭐⭐ **Refaz só o que as amostras `sujas` mudam**, e devolve em `mudadas`
    /// as amostras cujo gradiente foi reescrito (as que o device tem de
    /// receber). As sujas podem vir com repetidos e por qualquer ordem — e o
    /// índice da amostra de um VÉRTICE é o dele, logo uma peça que mudou de
    /// forma atualiza-se pelos vértices que se moveram.
    /// ⭐ **Refaz as inclinações das amostras `sujas`** — pelo incremental
    /// ([`Self::atualiza`]) ou, com mais de metade do plano sujo, do zero
    /// ([`Self::recalcula`], em paralelo): a mesma escolha para quem nasce e
    /// para quem sobe um relevo inteiro novo. Devolve `true` se refez TODAS
    /// (`mudadas` fica vazia: são todas).
    pub fn refaz<'a>(
        &mut self,
        tinta: &Tinta,
        cantos_de: &(impl Fn(usize) -> &'a [u32] + Sync),
        pos: &[[f32; 3]],
        sujas: &[u32],
        mudadas: &mut Vec<u32>,
    ) -> bool {
        if sujas.len() * 2 > self.g.len() {
            mudadas.clear();
            self.recalcula(tinta, cantos_de, pos);
            true
        } else {
            self.atualiza(tinta, cantos_de, pos, sujas, mudadas);
            false
        }
    }

    pub fn atualiza<'a>(
        &mut self,
        tinta: &Tinta,
        cantos_de: &(impl Fn(usize) -> &'a [u32] + Sync),
        pos: &[[f32; 3]],
        sujas: &[u32],
        mudadas: &mut Vec<u32>,
    ) {
        mudadas.clear();
        let Some(alt) = tinta.relevo() else {
            return;
        };
        // Duas épocas por chamada (a das amostras e a do anel), e nenhuma pode
        // dar a volta até ao `0` com que as marcas nascem.
        if self.epoca >= u32::MAX - 2 {
            self.marca.fill(0);
            self.marca_face.fill(0);
            self.epoca = 0;
        }
        let ep = self.epoca + 1;
        // 1. As faces que contêm uma amostra suja: todas as células mudadas
        //    vivem nelas, logo todos os gradientes mudados são amostras delas.
        let mut tocadas = Vec::new();
        for &s in sujas {
            self.faces_da_amostra(tinta, s, |f| tocadas.push(f));
        }
        tocadas.retain(|&f| {
            let m = &mut self.marca_face[f as usize];
            let nova = *m != ep;
            *m = ep;
            nova
        });
        for &f in &tocadas {
            let c = cantos_de(f as usize);
            tinta.para_cada_amostra(f as usize, c, |i| {
                if self.marca[i as usize] != ep {
                    self.marca[i as usize] = ep;
                    self.acc[i as usize] = [0.0; 4];
                    mudadas.push(i);
                }
            });
        }
        // 2. A média de uma amostra da fronteira junta as células de TODAS as
        //    faces que a tocam: as tocadas e o anel delas pelos vértices (o anel
        //    de um vértice cobre também as faces da aresta).
        let ep_anel = ep + 1;
        self.epoca = ep_anel;
        let mut anel = Vec::new();
        for &f in &tocadas {
            self.marca_face[f as usize] = ep_anel;
            anel.push(f);
        }
        for &f in &tocadas {
            let c = cantos_de(f as usize);
            for &v in &c[..n_cantos(c)] {
                let (a, b) = (
                    self.ini_v[v as usize] as usize,
                    self.ini_v[v as usize + 1] as usize,
                );
                for &g in &self.faces_v[a..b] {
                    if self.marca_face[g as usize] != ep_anel {
                        self.marca_face[g as usize] = ep_anel;
                        anel.push(g);
                    }
                }
            }
        }
        self.espalha(tinta, cantos_de, pos, alt, &anel, tocadas.len(), Some(ep));
        for &i in mudadas.iter() {
            self.g[i as usize] = media(self.acc[i as usize]);
        }
    }

    /// ⭐⭐ **Soma as células das `faces` no acumulador** — as `inteiras`
    /// primeiras inteiras, as restantes só na borda; com `so_marca`, só nas
    /// amostras dessa época. Cada face soma no rascunho dela, e os rascunhos
    /// juntam-se pela ordem da lista — em paralelo quando há células que
    /// paguem as threads.
    #[allow(clippy::too_many_arguments)]
    fn espalha<'a>(
        &mut self,
        tinta: &Tinta,
        cantos_de: &(impl Fn(usize) -> &'a [u32] + Sync),
        pos: &[[f32; 3]],
        alt: &[[f32; 2]],
        faces: &[u32],
        inteiras: usize,
        so_marca: Option<u32>,
    ) {
        let (acc, marca) = (&mut self.acc, &self.marca);
        let mut junta = |r: &Rascunho| {
            for (k, a) in r.acc.iter().enumerate() {
                if a[3] > 0.0 {
                    let s = r.tab[k] as usize;
                    if so_marca.is_none_or(|ep| marca[s] == ep) {
                        let o = &mut acc[s];
                        o[0] += a[0];
                        o[1] += a[1];
                        o[2] += a[2];
                        o[3] += a[3];
                    }
                }
            }
        };
        let faz = |k: usize, r: &mut Rascunho| {
            let f = faces[k] as usize;
            tinta.rascunho_da_face(f, cantos_de(f), pos, alt, k >= inteiras, r);
        };
        let celulas: usize = faces
            .iter()
            .enumerate()
            .map(|(k, &f)| {
                let l = tinta.lado_da_face(f as usize) as usize;
                if k < inteiras { l * l } else { 4 * l }
            })
            .sum();
        let threads = match self.threads {
            0 => std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
            t => t,
        }
        .min(faces.len());
        if threads <= 1 || celulas < LIMIAR_PARALELO {
            let mut r = Rascunho::default();
            for k in 0..faces.len() {
                faz(k, &mut r);
                junta(&r);
            }
            return;
        }
        // As threads tiram faces de um contador; a que chama junta os
        // rascunhos PELA ORDEM, à medida que ficam prontos.
        let prontos: Vec<Mutex<Option<Rascunho>>> =
            (0..faces.len()).map(|_| Mutex::new(None)).collect();
        let sinal = Condvar::new();
        let proxima = AtomicUsize::new(0);
        std::thread::scope(|sc| {
            for _ in 0..threads {
                sc.spawn(|| {
                    loop {
                        let k = proxima.fetch_add(1, Ord::Relaxed);
                        if k >= faces.len() {
                            break;
                        }
                        let mut r = Rascunho::default();
                        faz(k, &mut r);
                        *prontos[k]
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(r);
                        sinal.notify_all();
                    }
                });
            }
            for celula in &prontos {
                let mut guarda = celula
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                while guarda.is_none() {
                    guarda = sinal
                        .wait(guarda)
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                }
                if let Some(r) = guarda.take() {
                    drop(guarda);
                    junta(&r);
                }
            }
        });
    }

    /// As faces que contêm a amostra `s` — pelo bloco onde ela mora.
    fn faces_da_amostra(&self, tinta: &Tinta, s: u32, mut f: impl FnMut(u32)) {
        let topo = tinta.topologia();
        let v = topo.verts() as u32;
        if s < v {
            let (a, b) = (
                self.ini_v[s as usize] as usize,
                self.ini_v[s as usize + 1] as usize,
            );
            self.faces_v[a..b].iter().for_each(|&g| f(g));
            return;
        }
        let e = s - v;
        if e < topo.arestas_amostras() {
            let id = topo.off_aresta.partition_point(|&o| o <= e) - 1;
            self.faces_a[id]
                .iter()
                .filter(|&&g| g != u32::MAX)
                .for_each(|&g| f(g));
            return;
        }
        let n = e - topo.arestas_amostras();
        f((topo.off_interior.partition_point(|&o| o <= n) - 1) as u32);
    }
}

impl Tinta {
    /// Cada amostra da face, pela forma dela.
    fn para_cada_amostra(&self, face: usize, cantos: &[u32], mut f: impl FnMut(u32)) {
        if n_cantos(cantos) == 3 {
            self.para_cada_amostra_tri(face, &cantos[..3], |i, _| f(i));
        } else {
            self.para_cada_amostra_quad(face, cantos, |i, _| f(i));
        }
    }
}

fn acumula(acc: &mut [f32; 4], g: [f32; 3], a: f32) {
    acc[0] += a * g[0];
    acc[1] += a * g[1];
    acc[2] += a * g[2];
    acc[3] += a;
}

fn media(acc: [f32; 4]) -> [f32; 3] {
    if acc[3] > 0.0 {
        [acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3]]
    } else {
        [0.0; 3]
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn soma(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn escala(a: [f32; 3], k: f32) -> [f32; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn ponto(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cruz(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
