//! ⭐⭐⭐⭐ **A TINTA DEIXA DE SER DOS VÉRTICES** — *mesh colors*: a cor mora em
//! amostras nos **vértices**, nas **arestas** e no **interior** das faces.
//!
//! Clean-room de dois papers públicos — Yuksel, Keyser, House, *Mesh Colors*
//! (ACM TOG 29(2), 2010) e Yuksel, *Mesh Color Textures* (HPG 2017). A pesquisa
//! que escolheu esta família, a triagem de licença e a medição estão em
//! `docs/3D/27_o_estado_da_arte_de_onde_a_tinta_mora.md`.
//!
//! # ⭐⭐⭐ Por que esta família, em duas frases com número
//!
//! *A resolução da tinta deixa de ser a resolução da MALHA.* Medido no corpus
//! do dono (doc 27 §4): ao mesmo orçamento de memória, uma tinta sem costuras
//! lê **`379`** amostras por unidade de mundo no pior sítio da peça contra as
//! **`113`** do atlas `2048²` — e a dispersão do atlas **não é do solver, é o
//! Teorema Egregium**: quem achata um pedaço CURVO distorce área por obrigação.
//! Aqui nada é achatado: cada amostra vive na face, e um mapa afim num
//! triângulo preserva a razão de áreas **exactamente**.
//!
//! # ⭐⭐⭐ E o caso base É o produto que já shipa
//!
//! O nível é `lado = 2^k` **intervalos por aresta**, e a `lado = 1` só existem
//! as amostras de canto — **uma por vértice, na numeração da malha**. ⇒
//! [`Tinta`] ao nível zero é a `colors()` de hoje, ao bit, e não há migração
//! nenhuma a escrever. *Uma família nova cujo caso base é o produto actual
//! entra sem um degrau de formato.*
//!
//! # ⭐⭐⭐⭐ E desde 23/09 o nível é POR FACE (a P2)
//!
//! Com um nível só para a peça, uma face grande e uma pequena recebem o mesmo
//! número de amostras ⇒ a densidade por área dispersa `3,1×` a `18,3×` nas
//! peças do dono (medido — ver [`niveis_por_area`]). Hoje cada face tem o
//! nível dela e **cada aresta leva o MÁXIMO dos dois vizinhos**, o que mantém
//! a fronteira partilhada: a face grossa lê um **subconjunto EXACTO** das
//! amostras da aresta fina, sem arredondar e com as duas pontas preservadas.
//!
//! ⚠️ **Um plano UNIFORME é o mesmo de antes, ao bit** — os dois prefixos
//! voltam a ser produtos (`id × (lado − 1)`, `f × interior(lado)`), que é a
//! aritmética que o shader ainda faz; é isso que deixa o caminho da placa
//! correcto sem uma linha de WGSL nova. ⛔ E quem ainda assume um lado só
//! **recusa** um plano graduado em voz alta em vez de adivinhar: o assado por
//! [`assar::Recusa::Graduado`], o device por [`Tinta::lado_uniforme`].
//!
//! ⚠️ **A escada é de potências de dois porque METADE tem de ser exacta:** as
//! amostras estão em `i/L`, e ficar com as de `i` par dá exactamente `i/(L/2)`,
//! **com as duas pontas preservadas**. Uma escada `2^r − 1` (a que o paper usa
//! para contar as amostras INTERIORES de uma aresta) descreve a mesma retícula,
//! e `lado` conta os intervalos porque é o que a aritmética do endereço usa.
//!
//! # ⛔⛔ A fronteira é PARTILHADA, e é isso que a separa do Ptex
//!
//! Uma amostra de aresta é guardada **uma vez**, e as duas faces que se tocam
//! ali leem **a mesma célula** — logo não há costura, nem sequer de filtragem,
//! e não há adjacência a consultar durante a leitura. O Ptex e o Htex duplicam
//! essa fronteira e pagam-na em buscas (`5` e `3` por ponto) e em
//! descontinuidade nos cantos. *É por isso que esta é a família certa para um
//! pincel e não só para um renderizador.*
//!
//! # O que esta crate NÃO faz
//!
//! Ela não sabe o que é uma `Mesh`, um pincel, uma câmera ou um device. Ela
//! recebe **faces** (fatias de índices) e posições, e devolve **índices de
//! amostra** e pesos. Quem os pinta, quem os sobe à placa e quem os re-amostra
//! quando a topologia muda são os chamadores.

#![forbid(unsafe_code)]

pub mod amostragem;
pub mod assar;
pub mod enderecos;
pub mod topo;
pub mod vizinhanca;

#[cfg(test)]
#[path = "assar_tests.rs"]
mod assar_tests;

#[cfg(test)]
#[path = "enderecos_tests.rs"]
mod enderecos_tests;
#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
#[cfg(test)]
#[path = "p2_tests.rs"]
mod p2_tests;
#[cfg(test)]
#[path = "topo_tests.rs"]
mod topo_tests;
#[cfg(test)]
#[path = "vizinhanca_tests.rs"]
mod vizinhanca_tests;

pub use assar::{Assado, Recusa, Relatorio, assar};
pub use enderecos::{Sitio, indice, sitio_quad, sitio_tri, total};
pub use topo::{TRI, Topologia, cantos, interior_por_face};

/// A cor de quem ninguém pintou.
///
/// ⚠️ **Ela é o mesmo valor que a `ph2d_mesh::DEFAULT_COLOR`, e as duas têm de
/// concordar**: a `Tinta` ao nível zero é aquele plano. Há gate do lado da
/// `ph2d-mesh`, que é quem pode importar as duas.
pub const BRANCO: [f32; 3] = [1.0, 1.0, 1.0];

/// ⛔⛔ **O NÍVEL MÁXIMO, e ele nomeia DOIS recursos.**
///
/// | `k` | `lado` | amostras na peça de fábrica (`98 306` V) | plano em `f32×3` |
/// |---|---|---|---|
/// | `0` | `1` | `0,10 M` | `1,2 MB` |
/// | `1` | `2` | `0,39 M` | `4,7 MB` |
/// | `2` | `4` | `1,57 M` | `18,9 MB` |
/// | `3` | `8` | `6,29 M` | `75,5 MB` |
/// | `4` | `16` | `25,2 M` | `302 MB` |
/// | `5` | `32` | `101 M` | ⛔ `1,2 GB` |
///
/// A contagem é `≈ V · lado²` numa malha de triângulos (`V` vértices, `3V`
/// arestas, `2V` faces — a aritmética fecha e está no gate).
///
/// ⭐ **`5` é onde o ÍNDICE ainda cabe:** a `lado = 64` uma malha de `1 M`
/// vértices pede `4,1e9` amostras, e o `u32` que indexa este plano — e o buffer
/// da placa — para em `4,29e9`. ⚠️ **O tecto de MEMÓRIA é mais apertado que este
/// e é do CHAMADOR**, que é quem sabe quanta placa a cena tem: a tabela acima
/// existe para ele escolher, e é por isso que ela está aqui e não num comentário.
pub const NIVEL_MAX: u8 = 5;

/// ⭐ **O PLANO DE TINTA de uma malha.**
///
/// ⚠️ Ele guarda a [`Topologia`] porque o endereço de uma amostra de aresta
/// depende de qual aresta e de que lado — recalcular isso por amostra seria uma
/// busca por dab.
#[derive(Debug, Clone, PartialEq)]
pub struct Tinta {
    nivel: u8,
    topo: Topologia,
    amostras: Vec<[f32; 3]>,
}

/// ⭐⭐⭐⭐ **O NÍVEL DE CADA FACE PARA UMA DENSIDADE ALVO** — a lei da P2.
///
/// `areas[f]` é a área da face `f` no mundo, e `alvo` é a densidade LINEAR
/// pedida (amostras por unidade de comprimento). O nível é
/// `round(log2(alvo · √área))`, cortado na escada.
///
/// ⚠️⚠️ **O CHÃO desta lei é `2×` e não `√2`, e a medição desmente o handoff de
/// 21/09.** Ele escreveu *«com o `R` por face quantizado a potências de dois o
/// pior caso é `√2 = 1,41×`»* — ⛔ **as duas grandezas não são a mesma**: o `√2`
/// é o desvio ao alvo de UMA face (meia escada) e a dispersão é uma razão entre
/// DUAS, logo `√2 × √2 = 2`. Medido pelo [`examples/mede_o_r_por_face`] sobre o
/// corpus do dono, `p99/p1` da densidade linear:
///
/// | peça | faces | hoje (uniforme) | com esta lei |
/// |---|---:|---:|---:|
/// | `nossa_com_calota` | 21 914 | `3,12×` | **`1,90×`** |
/// | `Sculpt_Blender` | 8 291 | `4,88×` | **`1,97×`** |
/// | `_base_sculpt` | 18 432 | `6,74×` | **`1,95×`** |
/// | `sculpt_antes` | 13 824 | **`18,26×`** | **`1,92×`** |
///
/// ⚠️ **A única leitura acima de `2` é o CHÃO da escada a morder**: a `k` baixo
/// a `sculpt_antes` lê `2,37×` porque as faces mais pequenas pedem um nível
/// NEGATIVO e o corte em `0` deixa-as mais finas do que o alvo. *Uma face mais
/// pequena que `1/alvo` não tem como ser mais grossa do que uma amostra por
/// canto* — e isso é o fim da escada, não um defeito da lei.
///
/// `tecto_de_salto` é a cerca entre VIZINHAS: a face grossa lê um subconjunto
/// da aresta fina, logo um salto grande é detalhe que o lado grosso não
/// consegue mostrar. ⭐ **Medida, ela é quase inerte e quase de graça:** no
/// corpus do dono o salto máximo já é `2` sem cerca nenhuma (e só em `1`–`6`
/// arestas de `16 582`–`43 828`), e pô-la a `1` custa entre `+0` e `+360`
/// amostras num plano de `1,4 M`. ⇒ *ela fica como GUARDA, e o gate dela precisa
/// de uma fixtura construída para isso — o corpus não contém o fenómeno.*
#[must_use]
pub fn niveis_por_area(topo: &Topologia, areas: &[f32], alvo: f32, tecto_de_salto: u8) -> Vec<u8> {
    let mut k: Vec<u8> = areas
        .iter()
        .map(|a| {
            // ⚠️ Uma face DEGENERADA (área zero, e elas existem — o `collapse`
            //    desta casa deixa-as) pede `log2(0) = −∞`: o corte apanha-a, e
            //    o valor conservador é o nível mais grosso.
            let ideal = (alvo * a.max(0.0).sqrt()).log2();
            if ideal.is_finite() {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    ideal.round().clamp(0.0, f32::from(NIVEL_MAX)) as u8
                }
            } else {
                0
            }
        })
        .collect();
    // ⛔ **Sem `resize`, de propósito:** uma lista de áreas mais curta que a
    //    malha é o chamador a passar a peça errada, e a recusa é da
    //    [`Topologia::regraduada`], que compara os comprimentos. *Preencher com
    //    um valor de omissão aqui transformaria essa recusa num plano errado.*
    if k.len() != topo.faces() {
        return k;
    }

    // ⚠️ A adjacência ARESTA → faces é construída UMA vez: escrita dentro do
    //    laço ela é `O(F²)`, e numa peça de `20 k` faces isso é uma exportação
    //    que nunca acaba.
    let mut por_aresta: Vec<Vec<u32>> = vec![Vec::new(); topo.arestas()];
    for f in 0..topo.faces() {
        for s in 0..topo.cantos_de(f) {
            let (id, _) = topo.aresta(f, s);
            por_aresta[id as usize].push(f as u32);
        }
    }

    // ⭐ A cerca SOBE a vizinha grossa e nunca desce a fina: descer apagaria
    //   detalhe que o artista pediu. Ela corre até ao PONTO FIXO — subir uma
    //   vizinha pode obrigar a seguinte — e ele existe porque o nível só sobe e
    //   está preso em [`NIVEL_MAX`].
    loop {
        let mut mexeu = false;
        for vizinhas in &por_aresta {
            let Some(hi) = vizinhas.iter().map(|&g| k[g as usize]).max() else {
                continue;
            };
            for &g in vizinhas {
                if hi - k[g as usize] > tecto_de_salto {
                    k[g as usize] = hi - tecto_de_salto;
                    mexeu = true;
                }
            }
        }
        if !mexeu {
            return k;
        }
    }
}

impl Tinta {
    /// Uma tinta em branco sobre a topologia dada.
    ///
    /// ⚠️ `nivel` é **cortado** em [`NIVEL_MAX`] em vez de recusado: o chamador
    /// que pede `9` tem um defeito, e entregar-lhe um plano de `1,2 GB` ou um
    /// `panic` são as duas respostas piores. O nível efectivo lê-se em
    /// [`Self::nivel`].
    #[must_use]
    pub fn nova<'a>(verts: usize, faces: impl Iterator<Item = &'a [u32]>, nivel: u8) -> Self {
        let nivel = nivel.min(NIVEL_MAX);
        let topo = Topologia::nova(verts, faces, nivel);
        let n = total(&topo);
        Self {
            nivel,
            topo,
            amostras: vec![BRANCO; n],
        }
    }

    /// ⭐⭐⭐⭐ **Uma tinta em branco com um nível POR FACE** — a P2.
    ///
    /// ⛔ Ela **RECUSA** (`None`) uma lista que não descreve esta malha, pela
    /// mesma razão da [`Topologia::regraduada`]: *um plano com o tamanho errado
    /// instalado numa malha é tinta no sítio errado*.
    ///
    /// ⚠️ **O `nivel` que ela guarda é o MAIS FINO do plano**, e isso é uma
    /// definição e não um acidente: o campo é *o degrau que o artista pediu*, e
    /// num plano graduado o pedido é o tecto — as faces mais pequenas recebem
    /// menos porque a ÁREA delas não justifica mais, nunca porque alguém baixou
    /// o pedido.
    #[must_use]
    pub fn graduada<'a>(
        verts: usize,
        faces: impl Iterator<Item = &'a [u32]>,
        niveis: &[u8],
    ) -> Option<Self> {
        let topo = Topologia::nova(verts, faces, 0).regraduada(niveis)?;
        let n = total(&topo);
        Some(Self {
            nivel: topo.nivel_mais_fino(),
            topo,
            amostras: vec![BRANCO; n],
        })
    }

    /// ⭐ **A tinta do nível ZERO a partir do plano por-vértice que já existe.**
    ///
    /// ⛔ Ela copia e não converte, porque **não há conversão**: ao nível zero as
    /// duas coisas são o mesmo vector.
    #[must_use]
    pub fn do_plano_por_vertice<'a>(
        cores: &[[f32; 3]],
        faces: impl Iterator<Item = &'a [u32]>,
    ) -> Self {
        let topo = Topologia::nova(cores.len(), faces, 0);
        Self {
            nivel: 0,
            topo,
            amostras: cores.to_vec(),
        }
    }

    /// ⭐⭐⭐ **A TINTA QUE JÁ EXISTE, LEVADA PARA UM NÍVEL** — a porta de subir
    /// (e de descer) a resolução sem perder o que está pintado.
    ///
    /// Cada amostra recebe a mistura das cores dos CANTOS da face em que ela
    /// cai. ⭐ **As amostras de vértice recebem o valor do vértice, exactamente**
    /// (ali a mistura é `1` num canto e `0` nos outros), logo subir e descer o
    /// nível **nunca mexe** na tinta que o nível zero já continha.
    ///
    /// ⛔⛔ **Sem esta porta, armar a tinta fina sobre uma peça JÁ PINTADA
    /// apagava-a** — a [`Self::nova`] nasce branca, e o defeito lê-se como *«o
    /// pincel apagou o meu trabalho»*. Foi um gate que o apanhou, com um desvio
    /// de `1,0` num canal, que é uma cor inteira e nunca um arredondamento.
    #[must_use]
    pub fn semeada<'a>(
        cores: &[[f32; 3]],
        faces: impl Iterator<Item = &'a [u32]> + Clone,
        nivel: u8,
    ) -> Self {
        let mut t = Self::nova(cores.len(), faces.clone(), nivel);
        for (fi, f) in faces.enumerate() {
            let l = t.lado_da_face(fi);
            let n = topo::cantos(f);
            let c: Vec<[f32; 3]> = f[..n].iter().map(|&v| cores[v as usize]).collect();
            if n == 3 {
                for i in 0..=l {
                    for j in 0..=(l - i) {
                        let k = l - i - j;
                        let w = [
                            i as f32 / l as f32,
                            j as f32 / l as f32,
                            k as f32 / l as f32,
                        ];
                        let idx = indice(&t.topo, fi, sitio_tri(l, i, j, k), &f[..n]) as usize;
                        t.amostras[idx] = mistura(&c, &w);
                    }
                }
            } else {
                for j in 0..=l {
                    for i in 0..=l {
                        let (u, v) = (i as f32 / l as f32, j as f32 / l as f32);
                        let w = [(1.0 - u) * (1.0 - v), u * (1.0 - v), u * v, (1.0 - u) * v];
                        let idx = indice(&t.topo, fi, sitio_quad(l, i, j), &f[..n]) as usize;
                        t.amostras[idx] = mistura(&c, &w);
                    }
                }
            }
        }
        t
    }

    /// ⭐ **Quantos bytes este plano segura** — as amostras mais a topologia.
    ///
    /// ⚠️ Ver o doc da [`Topologia::footprint_bytes`]: ela existe porque uma
    /// peça apagada leva o plano para a fila de desfazer, e o orçamento
    /// daquela fila soma bytes.
    #[must_use]
    pub fn footprint_bytes(&self) -> usize {
        self.amostras.capacity() * size_of::<[f32; 3]>() + self.topo.footprint_bytes()
    }

    /// O nível efectivo (`k`).
    #[must_use]
    pub fn nivel(&self) -> u8 {
        self.nivel
    }

    /// Intervalos por aresta da face `f` — `2^k` do nível DELA.
    ///
    /// ⭐ Ela substituiu um `lado()` da PEÇA: com a P2 o lado é propriedade da
    /// FACE, e um `lado()` que devolvesse o de uma qualquer seria a resposta
    /// errada com a confiança da certa.
    #[must_use]
    pub fn lado_da_face(&self, f: usize) -> u32 {
        self.topo.lado_de(f)
    }

    /// ⭐ **O lado da peça inteira, se ele for um só.**
    ///
    /// ⛔ Ela existe para os consumidores que **ainda** assumem um lado — o
    /// caminho da placa é o principal — poderem RECUSAR um plano graduado em
    /// vez de desenharem tinta no sítio errado.
    #[must_use]
    pub fn lado_uniforme(&self) -> Option<u32> {
        self.topo.nivel_uniforme().map(|k| 1u32 << k)
    }

    /// A topologia derivada.
    #[must_use]
    pub fn topologia(&self) -> &Topologia {
        &self.topo
    }

    /// As amostras, na disposição do módulo [`enderecos`].
    #[must_use]
    pub fn amostras(&self) -> &[[f32; 3]] {
        &self.amostras
    }

    /// As amostras, para escrever.
    pub fn amostras_mut(&mut self) -> &mut [[f32; 3]] {
        &mut self.amostras
    }

    /// ⭐ **A amostra de um VÉRTICE é o próprio índice do vértice** — a lei que
    /// faz o nível zero ser a `colors()` de hoje.
    #[must_use]
    pub fn de_vertice(&self, v: usize) -> [f32; 3] {
        self.amostras[v]
    }

    /// O prefixo por-vértice do plano.
    ///
    /// ⭐ A `lado = 1` ele **é** o plano inteiro; acima disso é a fatia que um
    /// renderizador sem `lado` consegue desenhar, que é a rede de quem não tem
    /// a feature do device.
    #[must_use]
    pub fn plano_por_vertice(&self) -> &[[f32; 3]] {
        &self.amostras[..self.topo.verts]
    }

    /// O índice global de uma amostra da retícula de `face`.
    ///
    /// `cantos` são os índices de vértice da face, na ordem do percurso.
    #[must_use]
    pub fn indice_tri(&self, face: usize, cantos: &[u32], i: u32, j: u32, k: u32) -> u32 {
        let l = self.lado_da_face(face);
        indice(&self.topo, face, sitio_tri(l, i, j, k), cantos)
    }

    /// O índice global de uma amostra da retícula de um QUAD.
    #[must_use]
    pub fn indice_quad(&self, face: usize, cantos: &[u32], i: u32, j: u32) -> u32 {
        let l = self.lado_da_face(face);
        indice(&self.topo, face, sitio_quad(l, i, j), cantos)
    }
}

/// A mistura de cores por pesos — a aritmética que a [`Tinta::semeada`] usa nas
/// duas retículas.
fn mistura(c: &[[f32; 3]], w: &[f32]) -> [f32; 3] {
    let mut o = [0.0f32; 3];
    for (q, k) in c.iter().zip(w) {
        for e in 0..3 {
            o[e] += q[e] * k;
        }
    }
    o
}
