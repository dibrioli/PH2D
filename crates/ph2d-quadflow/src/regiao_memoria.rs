//! ⭐⭐⭐⭐ **A MEMÓRIA DO CAMPO DE POSIÇÃO AO LONGO DO TRAÇO** — a FASE que um
//! pincel pode repetir.
//!
//! Filho (`#[path]`) do [`super`], pelo mesmo corte que o
//! [`super::cerca`]: no pai mora *o domínio de um dab*, aqui *o que sobrevive
//! entre dois dabs*.
//!
//! # Porque ela existe, e porque é do campo de POSIÇÃO e não do de orientação
//!
//! O §86 desta linha construiu a hierarquia, mediu-a e recusou-a, e a frase que
//! ficou escrita foi: ***um extractor precisa de fase GLOBAL; um pincel precisa
//! de uma fase que ele possa REPETIR.*** Esta é essa fase.
//!
//! ⭐⭐⭐ **Os dois campos da mancha nascem com sementes de qualidade OPOSTA, e
//! isso explica a tabela do relógio inteira:**
//!
//! | campo | semente | consistência da semente | custo medido por alternância |
//! |---|---|---|---|
//! | orientação | a direcção do traço projectada em cada normal | **global por construção** (é UMA direcção do mundo) | `0,553 ms` |
//! | posição | **a própria posição de cada vértice** | **a pior possível** — cada vértice declara-se a origem da sua própria retícula | **`2,797 ms`** |
//!
//! ⇒ *o campo de orientação já chega quase resolvido e o de posição recomeça do
//! zero a cada carimbo.* É por isso que a memória é **só do de posição**: não é
//! uma escolha de economia, é onde a informação de facto falta.
//!
//! ⛔ **E é também por isso que o «conjunto activo» deu `0 %`** (a 3.ª das cinco
//! recusas da caça ao relógio): com a semente a ser a posição do vértice, a 1.ª
//! varredura move **toda a gente**, logo não há quem saltar. *A cura não era
//! saltar trabalho — era não deitar fora a resposta do carimbo anterior.*
//!
//! # ⭐⭐ A FRANJA é onde a memória faz o trabalho
//!
//! A franja de uma mancha está **PREGADA** ([`super::mancha`]): ela não se move
//! e serve de condição de fronteira a quem se move. Sem memória, a condição de
//! fronteira que ela impõe é *«cada um de vocês é a sua própria origem»* — ou
//! seja, nenhuma. Com memória, a franja de um dab é o **miolo já resolvido do
//! dab anterior**, e ela passa a impor a retícula que a zona penteada já tem.
//!
//! *É a diferença entre pentear vinte manchas e pentear um traço.*
//!
//! # ⚠️ O que ela NÃO é
//!
//! - **Não é uma cache.** Ela não evita trabalho: ela muda o ponto de partida.
//!   Quem a lê corre exactamente as mesmas varreduras.
//! - **Não atravessa traços nem peças.** O pen-down [`CampoDoTraco::esquece`].
//! - **Vazia, o caminho é BYTE-IDÊNTICO ao de antes** — a semente cai na
//!   posição do vértice, que é o que a [`super::posicao_da_mancha_com`] fazia; e
//!   há gate a afirmá-lo.

use ph2d_mesh::{Birth, Mesh, Remap};

use super::Mancha;

/// ⭐⭐⭐ **Quantos anéis a memória TRANSBORDA para o território virgem.**
///
/// O mecanismo está em [`CampoDoTraco::semente`]: a pegada anda, logo o
/// crescente da frente é sempre virgem, e sem transbordo há uma
/// **descontinuidade de fase desenhada ao longo da testa do traço** — exactamente
/// onde as fileiras novas se formam.
///
/// **Medido pela porta do produto** (`2` rondas, `2` alternâncias, oito rumos,
/// e a orientação a nascer do traço):
///
/// | anéis | grade % | fil p90 | 4 braços % |
/// |---|---|---|---|
/// | `0` (a 1.ª redacção) | `65,02` | `71,0` | `90,97` |
/// | `1` | `65,24` | `69,2` | `92,41` |
/// | **`2`** (shipa) | **`65,91`** | **`71,9`** | **`95,38`** |
/// | `3` | `65,65` | `71,0` | `94,83` |
/// | `4` | `65,85` | `71,5` | `95,15` |
///
/// ⇒ *o salto está entre `0` e `2` e acima disso a coluna satura.* ⛔ **`3` é
/// pior que `2`, e não por pouco** — transbordar demais deixa a semente da
/// franja a descrever uma retícula que ela já não tem por baixo.
///
/// ⚠️ **A régua da FILEIRA foi lida em OITO rumos e não em quatro**, porque ela
/// salta entre células vizinhas (o percurso é guloso; §86) — a `p50` dela
/// dispersa `±10` a `±19` entre rumos, e a escada de quatro células chegou a
/// pôr `34,5` ao lado de `52,2` sobre a mesma alavanca.
pub const ANEIS_DO_TRANSBORDO: usize = 2;

/// ⛔⛔ **A ORIENTAÇÃO TAMBÉM SER LEMBRADA foi CONSTRUÍDA, MEDIDA e RECUSADA.**
///
/// A hipótese era simétrica à da posição, e ela cai pelo mecanismo que o
/// cabeçalho deste módulo já escreve: **a semente do campo de orientação já é
/// globalmente coerente por construção** — uma direcção do mundo projectada em
/// cada normal —, logo não há fase por lembrar ali.
///
/// **Medido** (`2`/`2`, `2` anéis, oito rumos):
///
/// | orientação | grade % | fil p90 | 4 braços % | ms/traço |
/// |---|---|---|---|---|
/// | **nasce do traço** (shipa) | **`65,91`** | **`71,9`** | **`95,38`** | **`161,4`** |
/// | lembrada | `65,74` | `71,9` | `94,98` | `176,4` |
///
/// ⇒ ela **paga `9 %` de relógio para entregar menos**. ⚠️ E há uma segunda
/// razão, de produto: a semente da orientação é *a direcção que o artista
/// pediu* — **o dab IMPÕE** —, e é ela que faz o pente acompanhar uma pincelada
/// que CURVA. Lembrá-la poria o pente a resistir à mão.
///
/// ⚠️ **A lei fica viva e alcançável** ([`CampoDoTraco::lembra_orientacao`]), com
/// a sonda a exercitá-la: *apagá-la levava a medição junto*.
pub const LEMBRA_A_ORIENTACAO: bool = false;

/// **O ponto de retícula que cada vértice trouxe do carimbo anterior.**
///
/// Indexada pelo **id de malha**, como o `SculptStroke`, e mantida em dia pelas
/// **mesmas duas portas** que ele ([`Self::cresceu`] / [`Self::encolheu`]) —
/// *uma terceira resposta para «a malha mudou de tamanho» é como os dois lados
/// passam a descrever vértices diferentes*.
#[derive(Clone, Debug)]
pub struct CampoDoTraco {
    /// O ponto da retícula lembrado, por id de malha.
    o: Vec<[f32; 3]>,
    /// A direcção do campo de orientação lembrada, por id de malha.
    q: Vec<[f32; 3]>,
    /// Quem tem memória. Um vértice que nunca foi **miolo** não tem.
    visto: Vec<bool>,
    /// **Quantos anéis a memória TRANSBORDA** para o território virgem.
    ///
    /// Ver [`Self::semente`] para o mecanismo — em `0` a costura da frente do
    /// traço volta, e ela custou `fil50` `38,8 → 29,0` quando foi medida.
    pub aneis: usize,
    /// **A orientação também é lembrada?**
    ///
    /// ⚠️ Ela é a alavanca MENOR das duas, e a razão está no cabeçalho: a
    /// semente do campo de orientação já é **globalmente coerente por
    /// construção** (uma direcção do mundo projectada em cada normal), logo há
    /// muito menos por lembrar. O número está na tabela do
    /// [`crate::regiao::arruma_na_grelha_lembrando`].
    pub lembra_orientacao: bool,
}

impl Default for CampoDoTraco {
    fn default() -> Self {
        Self {
            o: Vec::new(),
            q: Vec::new(),
            visto: Vec::new(),
            aneis: ANEIS_DO_TRANSBORDO,
            lembra_orientacao: LEMBRA_A_ORIENTACAO,
        }
    }
}

impl CampoDoTraco {
    /// Quantos vértices de malha esta memória descreve.
    #[must_use]
    pub fn len(&self) -> usize {
        self.visto.len()
    }

    /// A memória está vazia?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.visto.is_empty()
    }

    /// Quantos vértices têm memória — o que uma sonda conta para saber se a
    /// fase está de facto a viajar.
    #[must_use]
    pub fn lembrados(&self) -> usize {
        self.visto.iter().filter(|v| **v).count()
    }

    /// **O nó que ESTE vértice lembra**, ou `None` se ele nunca foi miolo.
    ///
    /// ⚠️ É a leitura CRUA, sem a re-quantização ao passo de agora — quem quer a
    /// semente chama a [`Self::semente`]. Ela existe para os gates das duas
    /// portas de tamanho poderem afirmar *«a âncora seguiu o vértice»* sem
    /// reconstruir uma mancha.
    #[must_use]
    pub fn ancora(&self, v: u32) -> Option<[f32; 3]> {
        let vi = v as usize;
        (self.visto.get(vi).copied().unwrap_or(false)).then(|| self.o[vi])
    }

    /// **O PEN-DOWN** — a memória não atravessa traços.
    ///
    /// ⚠️ Ela também não atravessa PEÇAS: quem troca de peça activa a meio de
    /// uma sessão chama isto, porque os ids de vértice de duas malhas não têm
    /// nada que ver um com o outro e a memória é indexada por id.
    pub fn esquece(&mut self) {
        self.o.clear();
        self.q.clear();
        self.visto.clear();
    }

    /// **A malha CRESCEU** — cada vértice novo herda a âncora de um pai.
    ///
    /// ⚠️ **A âncora de um pai, nunca a MÉDIA das duas:** o ponto médio de dois
    /// nós de uma retícula **não é um nó dela** (é o centro de uma aresta ou de
    /// uma célula), e semear com ele poria o vértice novo a meia célula de fase
    /// de toda a vizinhança — exactamente o `0,375` que a hierarquia mediu e que
    /// a fez ser recusada. Os dois pais descrevem a MESMA retícula, logo
    /// qualquer um serve; escolher sempre o primeiro é o que torna isto
    /// determinístico.
    ///
    /// ⚠️ **Sem pai lembrado o filho nasce SEM memória**, e é a resposta certa:
    /// ele está em terreno que a lei ainda não resolveu, e a semente de lá é a
    /// própria posição.
    ///
    /// ⚠️ **A ORDEM de `births` é load-bearing**, pelo mesmo facto que o
    /// `SculptStroke::grow_with` já escreve: um passe parte arestas cujo extremo
    /// pode ser um vértice do passe anterior.
    pub fn cresceu(&mut self, mesh: &Mesh, births: &[Birth]) {
        let verts = mesh.vert_count();
        self.o.resize(verts, [0.0; 3]);
        self.q.resize(verts, [0.0; 3]);
        self.visto.resize(verts, false);
        for b in births {
            let (vi, a, bb) = (b.vert as usize, b.a as usize, b.b as usize);
            if vi >= self.visto.len() {
                continue;
            }
            let herda = if self.visto.get(a).copied().unwrap_or(false) {
                Some(a)
            } else if self.visto.get(bb).copied().unwrap_or(false) {
                Some(bb)
            } else {
                None
            };
            if let Some(pai) = herda {
                self.o[vi] = self.o[pai];
                self.q[vi] = self.q[pai];
                self.visto[vi] = true;
            }
        }
    }

    /// **A malha ENCOLHEU** — a renumeração do colapso, aplicada à memória.
    ///
    /// Irmã exacta da [`Self::cresceu`], e ela existe pela mesma razão que o
    /// `SculptStroke::shrink_with`: ⛔ **saltá-la não deixa a memória
    /// incompleta, deixa-a a MENTIR** — a âncora de um vértice morto passaria a
    /// descrever o vértice que ocupou a casa dele, e a fase viajaria para o
    /// sítio errado **em silêncio**.
    pub fn encolheu(&mut self, remap: &Remap) {
        if self.visto.is_empty() || remap.verts == self.visto.len() {
            return;
        }
        for &(from, to) in &remap.vert_moves {
            let (from, to) = (from as usize, to as usize);
            if from >= self.visto.len() || to >= self.visto.len() {
                continue;
            }
            self.o[to] = self.o[from];
            self.q[to] = self.q[from];
            self.visto[to] = self.visto[from];
        }
        self.o.truncate(remap.verts);
        self.q.truncate(remap.verts);
        self.visto.truncate(remap.verts);
    }

    /// **A SEMENTE do campo de posição desta mancha.**
    ///
    /// Quem tem memória parte do ponto de retícula que trouxe; quem não tem
    /// parte **da retícula de um vizinho que tem**, e só em último caso da
    /// própria posição — que é o que a referência faz.
    ///
    /// # ⛔⛔⛔ A COSTURA DA FRENTE, e porque a 1.ª redacção PIOROU o produto
    ///
    /// A 1.ª versão desta função semeava o que não tem memória com a própria
    /// posição, e **medida ela custou `fil50` `38,8 → 29,0`** — mais cadeias e
    /// mais curtas, ou seja *mais quebras*, exactamente a coluna que a memória
    /// existia para subir.
    ///
    /// ⭐⭐⭐ **O mecanismo é a pegada ANDAR:** a cada carimbo o crescente da
    /// frente é território virgem e o resto vem lembrado, logo havia uma
    /// **descontinuidade de fase** desenhada ao longo da testa do traço — e é
    /// precisamente ali que as fileiras novas se formam. Sem memória nenhuma não
    /// havia costura porque **toda** a mancha nascia em fase zero: *uma semente
    /// uniformemente má não tem costura; uma semente boa pela metade tem.*
    ///
    /// ⇒ a memória **transborda um anel**: quem chega novo adopta a retícula do
    /// vizinho lembrado de menor índice local (a escolha é arbitrária e tem de
    /// ser **determinística** — os vizinhos lembrados descrevem a mesma
    /// retícula, senão o dab anterior não tinha convergido).
    ///
    /// # ⚠️ E ela RE-QUANTIZA ao passo de AGORA — CONSERVADORA, não medida
    ///
    /// O nó lembrado é de uma retícula do carimbo anterior, cujo lado saía da
    /// aresta média daquela pegada. [`crate::position::position_round_4`] devolve
    /// o nó **da retícula ancorada nele** mais perto do vértice, com o lado e a
    /// direcção de hoje ⇒ *a FASE viaja e o ESPAÇAMENTO não*. Sem isto a franja
    /// — que está pregada — imporia um espaçamento que a mancha de agora não
    /// pediu.
    ///
    /// ⛔⛔ **A MUTAÇÃO QUE A APAGA NÃO SANGRA, e o número está aqui** (oito
    /// rumos, na configuração que shipa): `grade 65,86` contra `65,91`, `fil p90
    /// 71,8` contra `71,9`, `4 braços 95,34 %` contra `95,38 %` — *dentro da
    /// dispersão entre rumos, que é `±0,7` a `±1,0`*. ⇒ **neste corpus o passo
    /// da pegada mal deriva ao longo de um traço**, e a re-quantização não é uma
    /// alavanca.
    ///
    /// ⚠️ **Ela fica porque é a leitura CONSERVADORA e não por ser grátis:** um
    /// traço em que o artista mexa no `Detail` a meio, ou uma peça com aresta
    /// muito desigual, fazem o lado da célula andar — e ali um nó lembrado de
    /// uma retícula de outro espaçamento é uma condição de fronteira que a
    /// mancha de agora não pode satisfazer. *A mutação é NOMEADA, com a medição,
    /// em vez de silenciosa.*
    #[must_use]
    pub fn semente(&self, m: &Mancha, dirs: &[[f32; 3]], passo: f32) -> Vec<[f32; 3]> {
        let n = m.len();
        if dirs.len() != n {
            return m.pos.clone();
        }
        let mut tem: Vec<bool> = m
            .ids
            .iter()
            .map(|&v| self.visto.get(v as usize).copied().unwrap_or(false))
            .collect();
        let mut s: Vec<[f32; 3]> = (0..n)
            .map(|i| {
                if tem[i] {
                    crate::position::position_round_4(
                        self.o[m.ids[i] as usize],
                        dirs[i],
                        m.nrm[i],
                        m.pos[i],
                        passo,
                    )
                } else {
                    m.pos[i]
                }
            })
            .collect();
        // ⚠️ **Cada anel lê o anterior de um buffer À PARTE**: um Gauss-Seidel
        // aqui faria a fase atravessar a mancha inteira numa passagem, e a
        // semente deixaria de ser *«N anéis»* — passaria a depender da ordem dos
        // índices, que é o defeito que todo este módulo evita.
        for _ in 0..self.aneis {
            let (antes, tinha) = (s.clone(), tem.clone());
            let mut mudou = false;
            for i in 0..n {
                if tinha[i] {
                    continue;
                }
                if let Some(j) = m.adj[i].iter().map(|l| l.id as usize).find(|&j| tinha[j]) {
                    s[i] = crate::position::position_round_4(
                        antes[j], dirs[i], m.nrm[i], m.pos[i], passo,
                    );
                    tem[i] = true;
                    mudou = true;
                }
            }
            if !mudou {
                break;
            }
        }
        s
    }

    /// **A SEMENTE do campo de ORIENTAÇÃO desta mancha**, ou `None` quando a
    /// memória dela está desligada ou vazia.
    ///
    /// ⚠️ **A direcção lembrada é RE-PROJECTADA na normal de agora** — o barro
    /// mexeu-se debaixo dela, e uma tangente do carimbo anterior já não é
    /// tangente. ⛔ Sem isto a 4-RoSy receberia um vector com componente normal e
    /// o representante escolhido seria o de outra superfície.
    ///
    /// ⚠️ **Quem não lembra continua a nascer da direcção do TRAÇO**, que é a
    /// lei declarada (*o dab IMPÕE*) — e é ela que faz o pente seguir uma
    /// pincelada que curva.
    #[must_use]
    pub fn semente_da_orientacao(&self, m: &Mancha, direccao: [f32; 3]) -> Option<Vec<[f32; 3]>> {
        if !self.lembra_orientacao || self.visto.is_empty() {
            return None;
        }
        Some(
            m.ids
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    let vi = v as usize;
                    let d = if self.visto.get(vi).copied().unwrap_or(false) {
                        self.q[vi]
                    } else {
                        direccao
                    };
                    crate::orientation::project_tangent(d, m.nrm[i])
                })
                .collect(),
        )
    }

    /// **GUARDA o que a lei resolveu** — só o **MIOLO**.
    ///
    /// ⛔ **A franja NÃO escreve:** ela está pregada, logo o que sai dela é o que
    /// entrou. Guardar a semente de uma franja virgem seria gravar *«este vértice
    /// tem fase zero»* como se fosse uma resposta — e no carimbo seguinte ele
    /// contaria como **lembrado** para o transbordo, bloqueando o anel de chegar
    /// a ele e espalhando ele próprio uma fase que ninguém resolveu.
    ///
    /// ⛔⛔ **A MUTAÇÃO QUE DEIXA A FRANJA ESCREVER NÃO SANGRA, e o número está
    /// aqui** (oito rumos, na configuração que shipa): `grade 65,77` contra
    /// `65,91`, `fil p90 71,8` contra `71,9`, `4 braços 95,05 %` contra
    /// `95,38 %` — *a diferença é da ordem da dispersão entre rumos*. ⇒ o
    /// argumento acima é **mecanicamente certo e não é discriminado por este
    /// corpus**, e fica registado assim em vez de como uma lei que a medição não
    /// sustenta. ⚠️ *A leitura conservadora é a que fica: a memória é do que a
    /// lei RESOLVEU.*
    pub fn guarda(&mut self, m: &Mancha, dirs: &[[f32; 3]], grelha: &[[f32; 3]]) {
        if grelha.len() != m.len() || dirs.len() != m.len() {
            return;
        }
        // ⚠️⚠️ **Ela CRESCE para caber no que lhe contam, e a 1.ª redacção
        // não crescia — ela saltava o que não coubesse.** O produto entra pela
        // [`crate::regiao::arruma_na_grelha_lembrando`], que acomoda antes, logo
        // ali estava certo; mas uma sonda que chamasse a lei directamente
        // guardava **ZERO** e o lado *«com memória»* dela media o lado *«sem»*.
        // *Um no-op silencioso numa porta de escrita lê-se como uma medição.*
        if let Some(&maior) = m.ids.iter().max() {
            let n = maior as usize + 1;
            if n > self.visto.len() {
                self.o.resize(n, [0.0; 3]);
                self.q.resize(n, [0.0; 3]);
                self.visto.resize(n, false);
            }
        }
        for (i, &v) in m.ids.iter().enumerate() {
            if m.fronteira[i] {
                continue;
            }
            let vi = v as usize;
            self.o[vi] = grelha[i];
            self.q[vi] = dirs[i];
            self.visto[vi] = true;
        }
    }

    /// **Acomoda a memória ao tamanho da malha**, sem inventar conteúdo.
    ///
    /// ⚠️ É a rede de quem chama a lei sem ter passado pelas duas portas — ela
    /// degrada para *«sem memória»* em vez de indexar fora, e **nunca** encolhe
    /// em silêncio: encolher é o [`Self::encolheu`], que sabe para onde cada
    /// vértice foi.
    pub(super) fn acomoda(&mut self, verts: usize) {
        if self.visto.len() == verts {
            return;
        }
        if verts < self.visto.len() {
            // A malha encolheu sem a renumeração ter chegado: a memória que
            // sobra descreve vértices que já não existem ⇒ deitá-la fora é a
            // única leitura honesta.
            self.esquece();
        }
        self.o.resize(verts, [0.0; 3]);
        self.q.resize(verts, [0.0; 3]);
        self.visto.resize(verts, false);
    }
}
