//! ⭐⭐⭐ **A PELE DE UMA MALHA DE SPRITE — o que a PLACA precisa para posar** (F9 W2, ordem do dono
//! de 2026-09-20: *«implemente se esse é o padrão ouro»*).
//!
//! # ⛔⛔ Porque isto existe: a CPU era o TECTO do produto
//!
//! Até aqui o [`crate::SpriteMesh`] chegava com as posições **já posadas**, e quem as posava era a
//! CPU, **vértice a vértice, todo quadro**. Medido na arte do dono
//! (`ph2d-app-vec`, `de_que_e_feito_o_quadro_da_pele`, `--release`, `load 4,05`):
//!
//! | | ms | % de um quadro |
//! |---|---:|---:|
//! | `attach_skin_meshes` INTEIRO, 8 imagens | `5,980` | **`35,9 %`** |
//! | só a CÓPIA (dois clones por instância) | `0,033` | `0,2 %` |
//! | só a DEFORMAÇÃO por vértice | `5,774` | **`34,6 %`** |
//!
//! ⇒ **`96 %` do custo é a lei a correr por vértice**, e ele é **exactamente linear no número de
//! imagens** (`1,012` · `4,064` · `8,164` ms no perfil `smoke`). ⚠️ A minha hipótese era a CÓPIA, e
//! a medição derrubou-a: *uma sonda que não parte o relógio acusa o suspeito errado.*
//!
//! ⛔⛔ **E isso REFUTA a nota que parou esta fila em 2026-09-17**, que lia `1,824 ms` / `10,9 %`
//! nas mesmas 8 imagens e concluía *«não se gasta uma wave a comprar 11 % de um quadro que hoje
//! sobra»*. O quadro **não** o tem para dar: a `16` imagens são `72 %`, e a `23` é o quadro inteiro
//! sem nada mais desenhado. *§0.0 — quem move o número que tornava algo inalcançável tem de
//! reconferir a nota; aqui o número moveu-se por baixo da decisão de parar.*
//!
//! ⭐ **O tecto do dispositivo está medido e é outro mundo:** `100 352` triângulos deformados e
//! desenhados custam-lhe `0,73 ms` (`4,4 %`) — `ph2d-render/tests/it/skin_mesh_gpu_ceiling.rs`. É
//! o caso canónico do §0.0: *o caminho mais lento a definir o tecto do mais rápido.*
//!
//! # ⛔⛔⛔ A LEI NÃO É UMA MISTURA LINEAR — e o gate de paridade apanhou-o antes de shipar
//!
//! O desenho desta wave nasceu de um cabeçalho de 2026-09-17 que dizia *«a lei é uma mistura
//! LINEAR de afins»*. Em 2026-09-19 a CPU deixou de a usar: o [`ph2d_skeleton::Skin::blend`] passou
//! a **rodar em torno da JUNTA** para curar o entalhe do cotovelo, e a linear ficou como
//! [`ph2d_skeleton::Skin::blend_linear`], o CONTROLO. A primeira redacção do shader implementou a
//! antiga e o gate de paridade leu **`2,315e-3 m`** de desvio — *a arte desenharia num sítio e o
//! ponteiro apontaria noutro*.
//!
//! ⚠️ **A dívida estava NOMEADA no repo** (`ph2d_skeleton_live::skin_gpu_tests`, *«a lei da placa é
//! a mistura linear, e o produto já não a usa»*) e eu não a li antes de desenhar. *Uma paridade
//! medida contra a lei ERRADA teria shipado o defeito com um gate verde por cima.*
//!
//! # ⭐⭐⭐ O que a placa recebe
//!
//! `p' = R(θ̄)·(p − c) + Σ ŵ_i · (M_i · c)`, com `c` a média dos pares de juntas pesada por `w_i·w_j`
//! e `θ̄` a média em CÍRCULO dos ângulos das poses. ⇒ por vértice viajam `K` pesos e `K` índices; por
//! BIND, a tabela `n × n` de juntas; por QUADRO, `N` afins mais `N` pares `(cos θ, sin θ)`.
//!
//! ⚠️ **As duas tabelas novas são do BIND e do QUADRO, nunca do vértice** — a das juntas sai dos
//! eixos de REPOUSO ([`ph2d_skeleton::Skin::tabela_de_juntas`]) e os ângulos da pose. *A dobra
//! inteira continua a colapsar na tabela, do lado da CPU.*
//!
//! ⛔⛔ **E a truncagem a `K` passa a tocar o CENTRO, não só os pesos:** o `c` da CPU soma sobre
//! TODOS os pares de ossos com peso, e aqui sobre os pares dos `K` que ficaram. Num vértice com
//! `≤ K` ossos — a arte do dono, `3` tendões — é a mesma soma AO BIT; acima disso é uma
//! aproximação **declarada**, medida no gate `a_truncagem_a_quatro_ossos_renormaliza`.
//!
//! # ⚠️ Os afins são LOCAL→LOCAL e a conjugação é do DESENHO
//!
//! Eles são os `M_i` da pele, no espaço LOCAL da sprite — o mesmo em que o [`crate::SpriteMesh`]
//! guarda as posições. ⛔ Guardá-los já conjugados para o espaço do quad (`(l − anchor)/size`)
//! amarraria o payload ao `anchor`/`size` da instância, e o mesmo bind serve **nove** instâncias num
//! 9-slice. ⇒ quem conjuga é o [`crate::sprite_mesh::MeshFrame`], que é quem tem o quad na mão.
//!
//! # ⚠️ `K = 4` é o formato da indústria, e a truncagem é DECLARADA
//!
//! Um vértice com mais de `K` ossos de peso não-nulo perde os mais fracos e **renormaliza**. ⚠️ Na
//! arte do dono isso nunca acontece (o rig tem `3` tendões), e a lei que a CPU corre para as
//! costuras é a **MESMA** tabela truncada ([`SpriteMeshSkin::posa`]) — *desenhar por um mapa e
//! apontar por outro é o defeito que esta fila já pagou uma vez*.

/// ⭐ **Quantos ossos um vértice leva à placa.** Ver a nota da truncagem no cabeçalho.
pub const OSSOS_POR_VERTICE: usize = 4;

/// ⭐⭐ **O índice que diz «este vértice NÃO é posado»** — o que torna o caminho de omissão
/// byte-idêntico **por construção** e não por aritmética.
///
/// ⛔ A alternativa era pôr a identidade na tabela e multiplicar por ela: `x·1 + y·0 + 0` é exacto
/// em `f32` para todo `x` finito, e **deixa de o ser** num `y` infinito (`inf · 0 = NaN`). *Uma
/// igualdade que depende de os dados serem finitos não é uma igualdade.*
pub const SEM_PELE: u32 = u32::MAX;

/// ⭐⭐⭐ **A PELE DE UMA MALHA** — o que a placa lê para posar, e o que a CPU lê para responder às
/// costuras.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpriteMeshSkin {
    /// Por vértice de [`crate::SpriteMesh::local`]: os `K` pesos, **já normalizados**.
    pub pesos: Vec<[f32; OSSOS_POR_VERTICE]>,
    /// Por vértice: os `K` índices em [`Self::afins`]. Um peso `0` torna o índice irrelevante.
    pub ossos: Vec<[u32; OSSOS_POR_VERTICE]>,
    /// Os afins dos ossos, **LOCAL→LOCAL**, na convenção do `ph2d_affine::Xform`
    /// (`[a, b, c, d, e, f]` ⇒ `x' = a·x + c·y + e`, `y' = b·x + d·y + f`).
    pub afins: Vec<[f32; 6]>,
    /// ⭐ **A tabela `n × n` de juntas em ordem de linha**, no espaço LOCAL — `juntas[i·n + j]` é
    /// por onde os ossos `i` e `j` se encontram em REPOUSO
    /// ([`ph2d_skeleton::Skin::tabela_de_juntas`], que é quem a produz).
    ///
    /// ⚠️ **Ela é do BIND** (os eixos de repouso não mudam com a pose) e os `afins` são do QUADRO.
    /// *Duas cadências no mesmo payload, e é por isso que a tabela não se recalcula por quadro.*
    ///
    /// ⛔ **Vazia ⇒ a lei cai na mistura LINEAR**, que é o controlo — e o
    /// [`Self::valida`] recusa a pele antes disso.
    pub juntas: Vec<[f32; 2]>,
    /// ⭐ **`(cos θ_i, sin θ_i)` por osso**, com `θ_i` o ângulo da parte linear da pose
    /// ([`ph2d_skeleton::Skin::angulos_das_poses`]).
    ///
    /// ⚠️ **Resolvido do afim CRU e não do conjugado:** o shader recebe os afins já levados ao
    /// espaço do quad, e o `atan2` deles daria outro ângulo.
    pub angulos: Vec<[f32; 2]>,
}

impl SpriteMeshSkin {
    /// **Esta pele descreve `n` vértices?** — as duas listas têm de os cobrir, e tem de haver osso.
    ///
    /// ⚠️ **É a mesma cerca do [`crate::sprite_mesh::drawn_mesh`]**, e pela mesma razão: uma pele
    /// que não fecha com a malha posaria vértices com o peso do vizinho, **em silêncio**.
    #[must_use]
    pub fn valida(&self, n: usize) -> bool {
        let m = self.afins.len();
        !self.afins.is_empty()
            && self.pesos.len() == n
            && self.ossos.len() == n
            && self.angulos.len() == m
            && self.juntas.len() == m * m
    }

    /// ⭐⭐⭐ **A LEI, na CPU** — `p' = R(θ̄)·(p − c) + Σ ŵ_i·(M_i·c)` para UM vértice.
    ///
    /// ⚠️ **É a referência contra a qual o shader é medido** (o molde é o do Flip: dois motores, uma
    /// lei), e é também o que as costuras correm quando alguém pergunta onde a arte está. ⛔ Duas
    /// aritméticas aqui seriam *«desenhado por um mapa e agarrado por outro»*.
    ///
    /// ⚠️ **É o porte da [`ph2d_skeleton::Skin::blend`] sobre a tabela TRUNCADA**, e não uma
    /// segunda redacção dela: as juntas e os ângulos chegam prontos no payload, produzidos pela
    /// crate que implementa a lei. *A única coisa que este corpo decide é a ARITMÉTICA.*
    ///
    /// ⚠️ **Um peso zero SALTA o osso** — sem isso um índice de enchimento (`0`) somaria a pose do
    /// primeiro osso com peso nulo, o que é exacto em `f32` mas deixa de o ser com um afim
    /// não-finito. *A guarda é sobre os DADOS, não sobre a aritmética.*
    ///
    /// ⛔ **As três degenerescências caem na mistura LINEAR**, exactamente como a lei original:
    /// nenhum PAR de ossos (não há junta), soma de pesos nula, e a média em círculo a cancelar-se
    /// (duas poses a `180°` com pesos iguais).
    #[must_use]
    pub fn posa(&self, v: usize, p: [f32; 2]) -> [f32; 2] {
        let (Some(w), Some(b)) = (self.pesos.get(v), self.ossos.get(v)) else {
            return p;
        };
        let Some(c) = self.centro(w, b) else {
            return self.linear(w, b, p);
        };
        let (mut sx, mut sy, mut soma) = (0.0f32, 0.0f32, 0.0f32);
        for k in 0..OSSOS_POR_VERTICE {
            let Some(&[co, si]) = self.angulos.get(b[k] as usize) else {
                continue;
            };
            if w[k] == 0.0 {
                continue;
            }
            sx = w[k].mul_add(co, sx);
            sy = w[k].mul_add(si, sy);
            soma += w[k];
        }
        if soma == 0.0 || (sx == 0.0 && sy == 0.0) {
            return self.linear(w, b, p);
        }
        let n = sx.hypot(sy);
        let (co, si) = (sx / n, sy / n);
        let base = self.linear(w, b, c);
        let d = [p[0] - c[0], p[1] - c[1]];
        [
            si.mul_add(-d[1], co.mul_add(d[0], base[0])),
            si.mul_add(d[0], co.mul_add(d[1], base[1])),
        ]
    }

    /// O centro em torno do qual um vértice com estes pesos roda — `None` quando não há PAR.
    ///
    /// ⚠️ **Os índices de [`Self::ossos`] são os da própria malha**, logo indexam a
    /// [`Self::juntas`] directamente; o deslocamento pela base do quadro é do
    /// [`crate::sprite_mesh::MeshFrame`] e vive só na tabela que sobe ao dispositivo.
    fn centro(
        &self,
        w: &[f32; OSSOS_POR_VERTICE],
        b: &[u32; OSSOS_POR_VERTICE],
    ) -> Option<[f32; 2]> {
        let n = self.afins.len();
        let (mut num, mut den) = ([0.0f32, 0.0], 0.0f32);
        for i in 0..OSSOS_POR_VERTICE {
            if w[i] <= 0.0 || b[i] as usize >= n {
                continue;
            }
            for j in (i + 1)..OSSOS_POR_VERTICE {
                if w[j] <= 0.0 || b[j] as usize >= n {
                    continue;
                }
                let Some(&junta) = self.juntas.get(b[i] as usize * n + b[j] as usize) else {
                    continue;
                };
                let q = w[i] * w[j];
                num[0] = q.mul_add(junta[0], num[0]);
                num[1] = q.mul_add(junta[1], num[1]);
                den += q;
            }
        }
        (den > 0.0 && num[0].is_finite() && num[1].is_finite())
            .then(|| [num[0] / den, num[1] / den])
    }

    /// A mistura LINEAR `Σ ŵ_i·(M_i·p)` — o CONTROLO, e o que as três degenerescências devolvem.
    fn linear(
        &self,
        w: &[f32; OSSOS_POR_VERTICE],
        b: &[u32; OSSOS_POR_VERTICE],
        p: [f32; 2],
    ) -> [f32; 2] {
        let mut out = [0.0f32; 2];
        let mut soma = 0.0f32;
        for k in 0..OSSOS_POR_VERTICE {
            if w[k] == 0.0 {
                continue;
            }
            let Some(&[a, bb, c, d, e, f]) = self.afins.get(b[k] as usize) else {
                continue;
            };
            out[0] += w[k] * (a * p[0] + c * p[1] + e);
            out[1] += w[k] * (bb * p[0] + d * p[1] + f);
            soma += w[k];
        }
        // ⛔ **Soma zero devolve o ponto INTACTO**, nunca a origem — a mesma lei do
        // `ph2d_skeleton::Skin::blend`, e o que impede um vértice sem dono de saltar para o zero.
        if soma == 0.0 { p } else { out }
    }
}
