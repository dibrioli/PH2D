//! **A TOPOLOGIA QUE O ENDEREÇO PRECISA** — as arestas canónicas e os saltos.
//!
//! ⭐ Ela é derivada das FACES e de mais nada. A `Tinta` guarda-a porque o
//! endereço de uma amostra de aresta depende de **qual** aresta e **de que
//! lado** a face a percorre, e recalcular isso por amostra seria uma busca por
//! dab.

use std::collections::BTreeMap;

/// O sentinela do 4.º índice de um TRIÂNGULO.
///
/// ⚠️ **Ele é o mesmo valor que a `ph2d_mesh::TRI`** — esta crate não a pode
/// importar (ela declara zero dependências), e duas constantes que têm de
/// concordar sem ninguém a verificar é a forma que esta casa já pagou. ⇒ o
/// gate vive na primeira crate que vê as duas:
/// `ph2d-sculpt3d` :: `o_sentinela_do_triangulo_e_o_mesmo_nas_duas_crates`.
///
/// ⛔ **Esta linha já apontou para um gate que NÃO existia** (`ph2d-mesh` ::
/// `o_sentinela_do_triangulo_e_o_mesmo`, que o `git log -S` não encontra) — a
/// família dos oito de 13/09, reintroduzida numa crate fora do alcance do
/// censo que a cura. *Um gate citado e um gate escrito leem-se igual num
/// cabeçalho.*
pub const TRI: u32 = u32::MAX;

/// Quantos cantos uma face tem, lida com o sentinela.
#[must_use]
pub fn cantos(face: &[u32]) -> usize {
    match face.len() {
        4 if face[3] == TRI => 3,
        n => n,
    }
}

/// ⭐ **As arestas canónicas da malha, e por onde cada face passa nelas.**
///
/// ⛔⛔ **A ORIENTAÇÃO é a metade que não se pode esquecer.** Uma aresta `(u,v)`
/// guarda as amostras dela **uma vez**, numeradas de `u` para `v` com `u < v` —
/// é isso que faz a fronteira ser PARTILHADA e não duplicada, que é a diferença
/// de espécie entre esta família e o Ptex. Uma face que a percorra ao contrário
/// tem de ler `lado − t`, e quem esquecer isso escreve tinta espelhada ao longo
/// de metade das arestas da peça — **um defeito que só aparece com `lado > 1`**.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Topologia {
    /// Quantos vértices a malha tem.
    pub(crate) verts: usize,
    /// Quantas arestas distintas.
    pub(crate) arestas: usize,
    /// Para cada `(face, lado)` — `4 * face + lado` — o valor `id << 1 | virada`.
    /// `virada = 1` quer dizer que a face percorre a aresta do vértice MAIOR
    /// para o menor, logo `t` conta ao contrário.
    pub(crate) lado_da_face: Vec<u32>,
    /// Quantos cantos cada face tem (`3` ou `4`).
    pub(crate) cantos_da_face: Vec<u8>,
    /// Prefixo das amostras de INTERIOR: `off[f]..off[f+1]` são as da face `f`,
    /// contadas a partir de zero (o bloco das faces começa depois das arestas).
    pub(crate) off_interior: Vec<u32>,
    /// ⭐⭐ **A PRIMEIRA face que reclamou cada aresta.**
    ///
    /// Ela existe por uma razão só, e é a da [`crate::vizinhanca`]: um par de
    /// amostras que corre **ao longo** de uma aresta da malha é visto pelas
    /// DUAS faces que ali se tocam, e quem some pesos sobre pares tem de o
    /// contar **uma vez**. *Sem um dono, uma média do anel pesa o dobro
    /// exactamente na fronteira — e o defeito lê-se como um fio mais escuro ao
    /// longo de metade das arestas, que é o sintoma que esta família existe
    /// para não ter.*
    pub(crate) dono_da_aresta: Vec<u32>,
}

impl Topologia {
    /// Constrói a topologia a partir das faces.
    ///
    /// `faces` é um iterador de fatias de índices — ⭐ é assim que esta crate
    /// não precisa de conhecer a `ph2d_mesh::Face`. Um triângulo pode vir como
    /// `[a, b, c]` **ou** como `[a, b, c, TRI]`; os dois leem `3` cantos.
    ///
    /// ⚠️ **`lado` entra aqui porque o prefixo do interior depende dele** — e é
    /// por isso que mudar de nível reconstrói a topologia em vez de a reaproveitar.
    #[must_use]
    pub fn nova<'a>(verts: usize, faces: impl Iterator<Item = &'a [u32]>, lado: u32) -> Self {
        let mut mapa: BTreeMap<(u32, u32), u32> = BTreeMap::new();
        let mut dono_da_aresta: Vec<u32> = Vec::new();
        let mut lado_da_face = Vec::new();
        let mut cantos_da_face = Vec::new();
        let mut off_interior = Vec::new();
        let mut acc = 0u32;
        let mut fi = 0u32;
        for f in faces {
            let n = cantos(f);
            off_interior.push(acc);
            acc += interior_por_face(n, lado);
            cantos_da_face.push(n as u8);
            for s in 0..4 {
                if s >= n {
                    lado_da_face.push(u32::MAX);
                    continue;
                }
                let (a, b) = (f[s], f[(s + 1) % n]);
                let (u, v, virada) = if a <= b { (a, b, 0) } else { (b, a, 1) };
                let proximo = mapa.len() as u32;
                let id = *mapa.entry((u, v)).or_insert(proximo);
                if id as usize == dono_da_aresta.len() {
                    dono_da_aresta.push(fi);
                }
                lado_da_face.push(id << 1 | virada);
            }
            fi += 1;
        }
        off_interior.push(acc);
        Self {
            verts,
            arestas: mapa.len(),
            lado_da_face,
            cantos_da_face,
            off_interior,
            dono_da_aresta,
        }
    }

    /// ⭐ **Os dois globais que o payload não traz** — quantos vértices e
    /// quantas arestas distintas a malha tem. Um leitor do
    /// [`Self::payload`] precisa dos dois para saber onde cada bloco começa,
    /// e eles são da MALHA e não da face, logo não cabem num registo.
    #[must_use]
    pub fn verts(&self) -> usize {
        self.verts
    }

    /// Quantas arestas distintas — ver [`Self::verts`].
    #[must_use]
    pub fn arestas(&self) -> usize {
        self.arestas
    }

    /// Quantas faces.
    #[must_use]
    pub fn faces(&self) -> usize {
        self.cantos_da_face.len()
    }

    /// Cantos da face `f`.
    #[must_use]
    pub fn cantos_de(&self, f: usize) -> usize {
        self.cantos_da_face[f] as usize
    }

    /// A aresta do lado `s` da face `f`, e se a face a percorre ao contrário.
    #[must_use]
    pub fn aresta(&self, f: usize, s: usize) -> (u32, bool) {
        let v = self.lado_da_face[4 * f + s];
        (v >> 1, v & 1 == 1)
    }

    /// ⭐ **Esta face é a dona do lado `s`?** — ver [`Self::dono_da_aresta`].
    #[must_use]
    pub fn dona_do_lado(&self, f: usize, s: usize) -> bool {
        let (id, _) = self.aresta(f, s);
        self.dono_da_aresta[id as usize] as usize == f
    }
}

/// Quantas amostras de INTERIOR uma face de `n` cantos tem ao nível `lado`.
///
/// ⭐ **Triângulo `(L−1)(L−2)/2`, quad `(L−1)²`** — é a contagem dos pontos da
/// retícula com nenhuma coordenada baricêntrica (ou bilinear) nula. A `L = 1`
/// os dois dão **zero**, que é a linha que faz o nível base ser *cor
/// por-vértice ao bit*.
#[must_use]
pub fn interior_por_face(cantos: usize, lado: u32) -> u32 {
    if lado <= 1 {
        return 0;
    }
    let l = lado - 1;
    if cantos == 3 {
        l.saturating_mul(l.saturating_sub(1)) / 2
    } else {
        l * l
    }
}

/// Quantos `u32` um registo de face ocupa no [`Topologia::payload`].
pub const PAYLOAD_STRIDE: usize = 10;

impl Topologia {
    /// ⭐⭐⭐ **A TOPOLOGIA ACHATADA para quem não tem `Vec`** — o registo por
    /// face que um shader lê para resolver um endereço.
    ///
    /// Por face, `PAYLOAD_STRIDE` palavras:
    ///
    /// | fatia | o quê |
    /// |---|---|
    /// | `0..4` | os cantos, com o sentinela [`TRI`] no slot `3` de um triângulo |
    /// | `4..8` | `id << 1 \| virada` de cada lado, e [`TRI`] no slot `7` de um triângulo |
    /// | `8` | o início do bloco de interior desta face |
    /// | `9` | quantos cantos (`3` ou `4`) |
    ///
    /// ⚠️ **Nenhuma posição fica sem dono**, e as duas que um triângulo não usa
    /// levam o sentinela em vez de lixo — *uma posição sem dono e sem régua é
    /// onde o campo seguinte aterra por engano*, e há gate a exigir o valor.
    ///
    /// ⛔ Os cantos NÃO vivem na [`Topologia`] (o [`crate::indice`] recebe-os
    /// de quem chama), logo ela precisa das faces **outra vez** — e têm de ser
    /// as MESMAS, na mesma ordem, senão o payload descreve outra malha. O gate
    /// confere o registo contra o [`crate::indice`] face a face.
    pub fn payload<'a>(&self, faces: impl Iterator<Item = &'a [u32]>, out: &mut Vec<u32>) {
        out.clear();
        out.reserve(self.faces() * PAYLOAD_STRIDE);
        for (f, cantos) in faces.enumerate() {
            let n = crate::cantos(cantos);
            debug_assert_eq!(n, self.cantos_de(f), "o payload recebeu outra face");
            // ⚠️ `iter().take(n)` e não um índice: o clippy recusa a indexação
            //   por variável de laço, e aqui ela seria mesmo pior — o `n` vem
            //   do sentinela e a fatia pode ser mais curta que `4`.
            for c in cantos.iter().take(n) {
                out.push(*c);
            }
            for _ in n..4 {
                out.push(TRI);
            }
            for s in 0..4 {
                out.push(if s < n {
                    self.lado_da_face[4 * f + s]
                } else {
                    TRI
                });
            }
            out.push(self.off_interior[f]);
            out.push(n as u32);
        }
    }
}
