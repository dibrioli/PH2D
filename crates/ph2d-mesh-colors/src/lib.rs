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
//! # ⭐⭐⭐⭐ O nível POR FACE (a P2) — o SUBSTRATO fica, o produto é uniforme
//!
//! Com um nível só para a peça, uma face grande e uma pequena recebem o mesmo
//! número de amostras ⇒ a densidade por área dispersa `3,1×` a `18,3×` nas
//! peças do dono (medido pelo `examples/mede_o_r_por_face.rs`). ⚠️ **Entre
//! 23/09 e 24/09 o produto CRIOU planos graduados** (o `Even Detail`, com a lei
//! `niveis_por_area` por baixo) e o device desenhava-os; os dois saíram por
//! ordem do dono (handoff §31 e §33). **O que fica é o substrato**, porque há
//! ficheiros desse dia no disco: um plano graduado ainda se constrói, lê,
//! assa e converte — e o carregador converte-o ao abrir. Num plano graduado
//! cada face tem o nível dela e **cada aresta leva o MÁXIMO dos dois vizinhos**, o que mantém
//! a fronteira partilhada: a face grossa lê um **subconjunto EXACTO** das
//! amostras da aresta fina, sem arredondar e com as duas pontas preservadas.
//!
//! ⚠️ **Um plano UNIFORME é o mesmo de antes, ao bit** — os dois prefixos
//! voltam a ser produtos (`id × (lado − 1)`, `f × interior(lado)`), e o passo
//! do subconjunto vale `1` em toda aresta.
//!
//! ⛔⛔ **Quem recusa um plano graduado, e a história tem TRÊS datas.** O assado
//! deixou de o recusar quando o empacotador passou a dispor um ladrilho por
//! face (2026-09-22), e continua a saber dispô-lo. O device deixou de o recusar
//! quando o registo achatado passou a carregar o lado da face e o bloco de cada
//! aresta (2026-09-23, [`topo::PAYLOAD_STRIDE`] `10 → 19`) — ⛔ e **voltou a
//! recusá-lo em 2026-09-24**, quando o registo regressou a `10` palavras por
//! ordem do dono: o `Even Detail`, o único que CRIAVA planos graduados, tinha
//! saído, e as nove palavras ficavam a ser pagas em toda peça para nada.
//!
//! ⭐ **Pelo produto nenhum plano graduado chega à placa:** o carregador
//! converte-o ao abrir ([`Tinta::uniformizada`]), lendo cada amostra do plano
//! gravado. A recusa do device fica para o caso em que um chegue por outro
//! caminho — *a resposta certa se ele chegar, e nenhuma se não chegar*.
//!
//! ⚠️ O [`Tinta::lado_uniforme`] é, outra vez, as duas coisas: a PERGUNTA que
//! as fixturas das réguas fazem e a CERCA que o device usa.
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
mod uniformiza;
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
#[path = "uniformiza_tests.rs"]
mod uniformiza_tests;
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

/// ⛔⛔ **O NÍVEL MÁXIMO da escada, e o tecto REAL não é ele: é o ÍNDICE de
/// cada malha.**
///
/// | `k` | `lado` | amostras na peça de fábrica (`98 306` V) | na peça da cena `=52` (`738` V) |
/// |---|---|---|---|
/// | `3` | `8` | `6,29 M` · `75,5 MB` | `0,05 M` · `0,6 MB` |
/// | `4` | `16` | `25,2 M` · `302 MB` | `0,19 M` · `2,3 MB` |
/// | `5` | `32` | `101 M` · `1,2 GB` | `0,76 M` · `9,1 MB` |
/// | `6` | `64` | `403 M` · `4,8 GB` | `3,0 M` · `36 MB` |
/// | `7` | `128` | `1,6 G` · `19 GB` | `12 M` · `145 MB` |
/// | `8` | `256` | `6,4 G` · ⛔ índice | `48 M` · `580 MB` |
///
/// A contagem é `≈ V · lado²` numa malha de triângulos (`V` vértices, `3V`
/// arestas, `2V` faces — a aritmética fecha e está no gate).
///
/// ⭐⭐ **O recurso é por MALHA, e é por isso que ele não é esta constante**
/// (2026-09-24, report do dono *«16x não chega para o Painter»*): todo endereço
/// desta crate é `u32`, e o que cabe nele depende de `V`. A
/// [`Topologia::nova`] desce o nível de uma malha ao maior que cabe no índice
/// DELA, e a [`Topologia::regraduada`] recusa — os prefixos somavam em `u32`
/// e transbordavam em silêncio. ⚠️ **O tecto de MEMÓRIA da placa é mais
/// apertado que o índice e é do CHAMADOR**, que é quem tem o device.
///
/// ⚠️ `8` é o fim da escada que o produto oferece e não um recurso: a `lado =
/// 256` um quad tem `65 025` amostras de interior, e o que a peça da lição
/// compra ali está medido no `ph2d_app_sculpt3d::tinta_da_peca::NIVEL_MAX`.
pub const NIVEL_MAX: u8 = 8;

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

// ⛔⛔⛔⛔ **E A LEI POR BAIXO (`niveis_por_area`) SAIU COM ELE.**
//
// Ela respondia *«que nível uma face desta área quer, para uma densidade
// alvo»*, e tinha **um** consumidor: o escolhedor acima. Retirado ele, ela
// ficou **viva e órfã** — alcançável só pelos gates dela próprios —, que é a
// forma que o §5.0 manda apagar: *um instrumento que só mede a si mesmo lê-se
// como uma lei coberta*.
//
// ⚠️ **A medição dela NÃO se perdeu:** a tabela das três peças do dono, o
// mecanismo da cerca do salto e as duas âncoras estão no handoff §25–§26.
// *O que foi medido e rejeitado não se reconstrói, e o endereço dele é o
// handoff, nunca código sem chamador.*
//
// ⛔ **E a `Mesh::face_areas` saiu na mesma cascata**, pelo mesmo motivo — ela
// era a entrada desta. A `Mesh::surface_area` FICA (ela alimenta o tecto de
// quads da retopologia e o alvo da topologia dinâmica), e a lei partilhada do
// triângulo com ela.

impl Tinta {
    /// Uma tinta em branco sobre a topologia dada.
    ///
    /// ⚠️ `nivel` é **cortado** em [`NIVEL_MAX`] em vez de recusado: o chamador
    /// que pede `9` tem um defeito, e entregar-lhe um plano de `1,2 GB` ou um
    /// `panic` são as duas respostas piores. O nível efectivo lê-se em
    /// [`Self::nivel`].
    #[must_use]
    pub fn nova<'a>(verts: usize, faces: impl Iterator<Item = &'a [u32]>, nivel: u8) -> Self {
        let topo = Topologia::nova(verts, faces, nivel.min(NIVEL_MAX));
        // ⚠️ O nível EFECTIVO é o da topologia: ela desce-o quando o plano não
        //    cabe no índice desta malha.
        let nivel = topo.nivel_uniforme().unwrap_or(0);
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
    /// ⛔⛔⛔ **O `pedido` é um ARGUMENTO e não se deriva da lista, e a premissa
    /// que morreu para ele existir está escrita aqui porque a morte dela custou
    /// um report do dono (23/09).**
    ///
    /// Este campo dizia-se *«o MAIS FINO do plano»*, com a justificação ao lado:
    /// *«num plano graduado o pedido é o TECTO»*. Isso era verdade da âncora na
    /// MEDIANA, onde ninguém passava de `k`. Desde que o `k` é um **PISO**
    /// (a lei `niveis_igualados`, que saiu com o `Even Detail`) há faces ACIMA
    /// dele, logo `nivel_mais_fino()`
    /// deixou de ser o pedido — e o consumidor que compara os dois (*«este
    /// plano ainda é o que a fileira pede?»*) passava a responder **NÃO em todo
    /// quadro**, reconstruindo o plano e **re-semeando-o da cor por vértice**:
    /// a tinta fina do artista desaparecia a `60 Hz`.
    ///
    /// ⚠️ **Ele é um argumento OBRIGATÓRIO de propósito:** derivá-lo outra vez,
    /// por qualquer regra, seria a segunda resposta à pergunta *«o que é que o
    /// artista pediu?»* — e a primeira é a fileira do painel. Assim, quem
    /// construir um plano graduado sem dizer de que degrau ele é **não
    /// compila**.
    ///
    /// ⛔ Ela **RECUSA** (`None`) uma lista que não descreve esta malha, pela
    /// mesma razão da [`Topologia::regraduada`]: *um plano com o tamanho errado
    /// instalado numa malha é tinta no sítio errado*.
    #[must_use]
    pub fn graduada<'a>(
        verts: usize,
        faces: impl Iterator<Item = &'a [u32]>,
        niveis: &[u8],
        pedido: u8,
    ) -> Option<Self> {
        let topo = Topologia::nova(verts, faces, 0).regraduada(niveis)?;
        let n = total(&topo);
        Some(Self {
            nivel: pedido,
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
        t.semeia(cores, faces);
        t
    }

    /// ⭐⭐⭐⭐ **A irmã GRADUADA da [`Self::semeada`]** — a mesma semente, sobre
    /// um plano com um nível POR FACE (a P2).
    ///
    /// ⛔ Ela **RECUSA** (`None`) uma lista que não descreve esta malha, como a
    /// [`Self::graduada`] de que ela nasce.
    ///
    /// ⚠️⚠️ **A semente é a MESMA função e não uma segunda redacção dela**
    /// ([`Self::semeia`]): ela já lia o `lado_da_face`, logo estava P2-ready
    /// antes de existir uma porta graduada. *Duas cópias divergiriam no dia da
    /// primeira emenda, e o que se perde aí é a tinta do artista.*
    pub fn semeada_graduada<'a>(
        cores: &[[f32; 3]],
        faces: impl Iterator<Item = &'a [u32]> + Clone,
        niveis: &[u8],
        pedido: u8,
    ) -> Option<Self> {
        let mut t = Self::graduada(cores.len(), faces.clone(), niveis, pedido)?;
        t.semeia(cores, faces);
        Some(t)
    }

    /// A semente, partilhada pelas duas portas acima.
    fn semeia<'a>(&mut self, cores: &[[f32; 3]], faces: impl Iterator<Item = &'a [u32]>) {
        let t = self;
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

    /// ⭐ **O degrau que o artista PEDIU** — nunca o mais fino nem o mais
    /// grosso do plano.
    ///
    /// ⚠️ Num plano graduado as duas coisas separam-se (ver [`Self::graduada`]),
    /// e quem comparar isto com a fileira do painel tem a resposta certa.
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
    /// ⭐⭐ **Ela é CERCA outra vez desde 2026-09-24, e é por isso que a razão de
    /// ser dela é a de sempre:** os consumidores que assumem um lado — o device
    /// é o principal — perguntam aqui antes de desenhar, e um plano graduado
    /// desarma em vez de pôr tinta no sítio errado. Entre 23/09 e 24/09 o
    /// device lia o lado da FACE e ela era só uma pergunta; o registo que o
    /// permitia saiu por ordem do dono.
    ///
    /// ⭐ E é também a PERGUNTA do carregador (*«este plano precisa de ser
    /// convertido?»*) e das réguas, que escrevem a expectativa de uma fixtura
    /// uniforme com este número.
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
