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
    /// ⭐⭐⭐⭐ **O NÍVEL DE CADA FACE** — a P2, e a razão de este vector existir
    /// em vez de um `u32` só.
    ///
    /// Com um `lado` para a peça inteira, uma face grande e uma pequena recebem
    /// o MESMO número de amostras ⇒ a densidade por área dispersa. **Medido no
    /// corpus do dono** (`examples/mede_o_r_por_face.rs`, `p99/p1` da densidade
    /// linear): `3,12×` · `4,88×` · `6,74×` · **`18,26×`**.
    ///
    /// ⚠️ **É um NÍVEL e não um lado, e o tipo carrega a lei:** a escada é de
    /// potências de dois porque a face grossa tem de ler um **subconjunto
    /// EXACTO** das amostras da aresta fina — ver [`Self::aresta_lado`]. Guardar
    /// `lado` como `u32` deixaria alguém escrever `6`, e aí `le / lf` trunca e
    /// a tinta sai no sítio errado **em silêncio**.
    pub(crate) nivel_da_face: Vec<u8>,
    /// ⭐⭐⭐ **O NÍVEL DE CADA ARESTA — o MÁXIMO dos vizinhos.**
    ///
    /// A fronteira é **partilhada** (é a diferença de espécie para o Ptex),
    /// logo ela só pode ter UMA resolução; tomar o máximo é o que deixa a face
    /// fina escrever tudo o que ela sabe. ⛔ Tomar o mínimo apagaria detalhe que
    /// o artista pintou do lado fino, e tomar a média não é uma potência de dois.
    pub(crate) nivel_da_aresta: Vec<u8>,
    /// Prefixo das amostras de ARESTA: a aresta `e` ocupa
    /// `off_aresta[e]..off_aresta[e+1]`, e o comprimento é `lado_e − 1`.
    ///
    /// ⭐ Com um nível uniforme ele vale **exactamente** `e · (lado − 1)`, que é
    /// a aritmética que o shader ainda faz — é isso que mantém o caminho da
    /// placa correcto sem uma linha de WGSL nova enquanto ninguém gradua.
    pub(crate) off_aresta: Vec<u32>,
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
    /// ⚠️ **`nivel` entra aqui porque o prefixo do interior depende dele** — e é
    /// por isso que mudar de nível reconstrói os prefixos em vez de os
    /// reaproveitar. ⭐ Mudar SÓ o nível não precisa desta porta: a adjacência
    /// não depende dele, e quem a reaproveita é a [`Self::regraduada`].
    ///
    /// ⛔ Ele é o **nível** (`k`) e não o **lado** (`2^k`): a escada é de
    /// potências de dois por lei (ver [`Self::nivel_da_face`]), e um argumento
    /// `lado: u32` deixaria alguém escrever `6`.
    #[must_use]
    pub fn nova<'a>(verts: usize, faces: impl Iterator<Item = &'a [u32]>, nivel: u8) -> Self {
        let lado = 1u32 << nivel;
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
        let arestas = mapa.len();
        let faces = cantos_da_face.len();
        let mut off_aresta = Vec::with_capacity(arestas + 1);
        let mut a = 0u32;
        for _ in 0..arestas {
            off_aresta.push(a);
            a += lado - 1;
        }
        off_aresta.push(a);
        Self {
            verts,
            arestas,
            lado_da_face,
            cantos_da_face,
            off_interior,
            dono_da_aresta,
            nivel_da_face: vec![nivel; faces],
            nivel_da_aresta: vec![nivel; arestas],
            off_aresta,
        }
    }

    /// ⭐⭐⭐⭐ **A MESMA MALHA COM UM NÍVEL POR FACE** — a P2.
    ///
    /// ⭐ **A adjacência não depende do nível**, logo tudo o que ela custou a
    /// construir (o mapa das arestas, quem as percorre ao contrário, quem é a
    /// dona) é **reaproveitado ao bit**; o que se refaz são os três prefixos.
    /// *É por isso que esta é uma porta sobre uma topologia e não um segundo
    /// construtor: um segundo construtor voltaria a percorrer as faces, e as
    /// faces são a única coisa que esta crate não guarda.*
    ///
    /// ⛔ **Ela RECUSA uma lista que não descreve esta malha** (`None`) em vez de
    /// a preencher com um valor de omissão: um `niveis` curto é o chamador a
    /// passar a peça errada, e um plano com o tamanho errado instalado numa
    /// malha é **tinta no sítio errado, em silêncio**.
    ///
    /// ⚠️ Cada nível é **cortado** em [`crate::NIVEL_MAX`], pela mesma razão que
    /// a [`crate::Tinta::nova`] o corta.
    #[must_use]
    pub fn regraduada(&self, niveis: &[u8]) -> Option<Self> {
        if niveis.len() != self.faces() {
            return None;
        }
        let nivel_da_face: Vec<u8> = niveis.iter().map(|k| (*k).min(crate::NIVEL_MAX)).collect();

        // A aresta leva o MÁXIMO dos vizinhos — ver `nivel_da_aresta`.
        let mut nivel_da_aresta = vec![0u8; self.arestas];
        for (f, &k) in nivel_da_face.iter().enumerate() {
            for s in 0..self.cantos_de(f) {
                let (id, _) = self.aresta(f, s);
                let e = &mut nivel_da_aresta[id as usize];
                *e = (*e).max(k);
            }
        }

        let mut off_aresta = Vec::with_capacity(self.arestas + 1);
        let mut a = 0u32;
        for k in &nivel_da_aresta {
            off_aresta.push(a);
            a += (1u32 << k) - 1;
        }
        off_aresta.push(a);

        let mut off_interior = Vec::with_capacity(self.faces() + 1);
        let mut acc = 0u32;
        for (f, &k) in nivel_da_face.iter().enumerate() {
            off_interior.push(acc);
            acc += interior_por_face(self.cantos_de(f), 1u32 << k);
        }
        off_interior.push(acc);

        Some(Self {
            verts: self.verts,
            arestas: self.arestas,
            lado_da_face: self.lado_da_face.clone(),
            cantos_da_face: self.cantos_da_face.clone(),
            off_interior,
            dono_da_aresta: self.dono_da_aresta.clone(),
            nivel_da_face,
            nivel_da_aresta,
            off_aresta,
        })
    }

    /// ⭐ **Quantos bytes esta topologia segura.**
    ///
    /// ⚠️ **Ela existe porque uma peça APAGADA entra inteira na fila de
    /// desfazer**, e o orçamento daquela fila soma bytes: sem esta porta o
    /// plano de tinta fina — até `75 MB` na peça de fábrica — viajaria lá
    /// dentro **invisível ao tecto que existe para o impedir**.
    ///
    /// ⛔ **É a crate que POSSUI os vectores que sabe medi-los.** A conta
    /// escrita do lado de fora divergiria no dia em que um campo novo entrasse
    /// aqui, e divergiria em silêncio.
    #[must_use]
    pub fn footprint_bytes(&self) -> usize {
        self.lado_da_face.capacity() * size_of::<u32>()
            + self.cantos_da_face.capacity()
            + self.off_interior.capacity() * size_of::<u32>()
            + self.dono_da_aresta.capacity() * size_of::<u32>()
            + self.nivel_da_face.capacity()
            + self.nivel_da_aresta.capacity()
            + self.off_aresta.capacity() * size_of::<u32>()
    }

    /// O nível (`k`) da face `f`.
    #[must_use]
    pub fn nivel_de(&self, f: usize) -> u8 {
        self.nivel_da_face[f]
    }

    /// O lado (`2^k`) da retícula da face `f` — intervalos por aresta.
    #[must_use]
    pub fn lado_de(&self, f: usize) -> u32 {
        1u32 << self.nivel_da_face[f]
    }

    /// O lado da aresta `id` — o **máximo** dos vizinhos dela.
    ///
    /// ⭐⭐⭐ **É esta função que faz a face grossa ler um SUBCONJUNTO EXACTO.**
    /// Com `lado_e / lado_f` inteiro (as duas são potências de dois e
    /// `lado_e >= lado_f`), a amostra `t` da face cai **em cima** da amostra
    /// `t · (lado_e / lado_f)` da aresta — sem arredondar, sem interpolar, e com
    /// as duas pontas preservadas.
    #[must_use]
    pub fn aresta_lado(&self, id: u32) -> u32 {
        1u32 << self.nivel_da_aresta[id as usize]
    }

    /// Onde as amostras da aresta `id` começam, dentro do bloco das arestas.
    #[must_use]
    pub fn aresta_off(&self, id: u32) -> u32 {
        self.off_aresta[id as usize]
    }

    /// Quantas amostras o bloco das ARESTAS tem ao todo.
    #[must_use]
    pub fn arestas_amostras(&self) -> u32 {
        self.off_aresta.last().copied().unwrap_or(0)
    }

    /// O nível mais FINO do plano — ver [`crate::Tinta::graduada`].
    #[must_use]
    pub fn nivel_mais_fino(&self) -> u8 {
        self.nivel_da_face.iter().copied().max().unwrap_or(0)
    }

    /// ⭐ **O nível da peça inteira, se ele for um só.**
    ///
    /// ⛔ Ela existe para os consumidores que **ainda** assumem um lado — o
    /// caminho da placa é o principal — poderem dizer *«este plano não é para
    /// mim»* em vez de desenhar tinta no sítio errado. *Um `lado()` que devolve
    /// o de uma face qualquer é a resposta errada com a confiança da certa.*
    #[must_use]
    pub fn nivel_uniforme(&self) -> Option<u8> {
        let k = *self.nivel_da_face.first()?;
        self.nivel_da_face.iter().all(|x| *x == k).then_some(k)
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

    /// ⭐⭐⭐⭐ **ESTA TOPOLOGIA AINDA DESCREVE AQUELA MALHA?** — a lei, no
    /// sítio onde a topologia mora.
    ///
    /// ⛔⛔ **Ela existe por um PÂNICO do dono** (2026-09-21, quadro `10216`):
    /// `index out of bounds: the len is 196608 but the index is 196608` no
    /// [`Self::payload`], que é `4 × 49 152` — a lista de faces que lá chegou
    /// tinha **mais** faces do que esta topologia. O caminho é o pen-down de um
    /// gesto de FORMA com o plano armado: o plano é emprestado ao traço e
    /// **logo a seguir** a malha é triangulada, e a rota do plano emprestado é
    /// a única que não reconcilia.
    ///
    /// ⚠️ **Compara vértices E faces, e as duas metades são precisas:** um
    /// colapso seguido de um refino pode devolver a MESMA contagem de vértices
    /// com outras faces, e um refino que só parte quads deixa a contagem de
    /// faces a subir com a de vértices parada. *Uma régua que conta uma
    /// grandeza só aprova a metade das mudanças de topologia.*
    ///
    /// ⚠️ **Ela é NECESSÁRIA e não suficiente** — duas malhas podem ter as
    /// mesmas duas contagens e outras faces. Quem precisa da resposta forte é
    /// o [`Self::payload`], que a confere face a face por construção.
    #[must_use]
    pub fn descreve(&self, verts: usize, faces: usize) -> bool {
        self.verts == verts && self.faces() == faces
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
    ///
    /// ⭐⭐⭐⭐ **E ELA DEVOLVE SE ERAM MESMO AS MESMAS.** `false` = a lista
    /// não é a desta topologia (mais faces, menos faces, ou uma face com outro
    /// número de cantos), e aí `out` fica **vazio** — nunca meio escrito. As
    /// três recusas são a mesma pergunta que a [`Self::descreve`] faz pelas
    /// contagens, agora respondida face a face.
    ///
    /// ⚠️ **Antes de 2026-09-21 esta porta ESTOURAVA**, e o pânico é o do
    /// report do dono — ver a [`Self::descreve`] para o caminho do produto que
    /// lá chega.
    #[must_use]
    pub fn payload<'a>(&self, faces: impl Iterator<Item = &'a [u32]>, out: &mut Vec<u32>) -> bool {
        out.clear();
        out.reserve(self.faces() * PAYLOAD_STRIDE);
        let mut vistas = 0usize;
        for (f, cantos) in faces.enumerate() {
            // ⛔⛔ **A RECUSA, e ela substitui um `debug_assert`.** A linha que
            // aqui estava dizia a mesma coisa e **só em `debug`** — no perfil
            // `smoke`, que é o que o dono corre, ela não existe e o que o
            // artista recebe é o pânico do índice. *Uma promessa escrita num
            // `debug_assert` é uma promessa que o produto não faz.*
            if f >= self.faces() {
                out.clear();
                return false;
            }
            let n = crate::cantos(cantos);
            if n != self.cantos_de(f) {
                out.clear();
                return false;
            }
            vistas = f + 1;
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
        // ⚠️ **A metade CURTA é a que não estoura, e é a pior.** Com menos
        // faces do que esta topologia o laço acaba sozinho e o registo fica
        // truncado: o shader lê o bloco de interior de uma face que já não
        // está lá e pinta tinta **válida no sítio errado**, em silêncio. É por
        // isso que a régua é a IGUALDADE e não um tecto.
        if vistas != self.faces() {
            out.clear();
            return false;
        }
        true
    }
}
