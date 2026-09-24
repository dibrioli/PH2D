//! ⭐⭐⭐⭐ **O DAB POR AMOSTRA** — a tinta deixa de ser dos vértices.
//!
//! A lei da retícula vive na [`ph2d_mesh_colors`] (clean-room dos papers de
//! *mesh colors*); aqui mora o que um PINCEL faz com ela. A pesquisa que
//! escolheu esta família está em
//! `docs/3D/27_o_estado_da_arte_de_onde_a_tinta_mora.md`.
//!
//! # ⛔⛔⛔ As TRÊS cercas de VÉRTICES do dab, e porque elas cediam
//!
//! Report do dono, 2026-09-21: *«a tinta só é depositada se o pincel está
//! sobre um vertex»*. O [`crate::stroke_dab_core`] tem **três** saídas cedo em
//! fila — a **pegada** (nenhum vértice na esfera de consulta), a **máscara**
//! de alcance e os **movidos** —, e as três contam VÉRTICES enquanto a unidade
//! que este módulo escreve é a AMOSTRA. Numa malha grossa há dezenas de
//! amostras entre dois vértices.
//!
//! ⚠️ **A régua é o MAPA e não a quantidade.** Varrendo o ecrã de 5 em 5
//! píxeis com um clique em cada sítio, na cena `=52` (`738` vértices, aresta
//! média `0,1304`), a tinta caía em **`3` de `61`** posições com o pincel fino
//! e em `23` de `61` com o seguinte — *o mapa das que pintavam é a rede de
//! vértices*. Somar as amostras da varredura inteira esconde isso atrás dos
//! sítios que funcionavam.
//!
//! ⚠️⚠️ **E a atribuição custou duas tentativas erradas:** curar a cerca dos
//! MOVIDOS deixou o mapa idêntico carácter a carácter, e a da PEGADA também.
//! *Uma cura que não move a régua não é a cura* — quem apontou a cerca certa
//! (a do meio, em `96` das corridas) foi um rasto por `PH2D_DAB_TRACE`.
//!
//! ⭐ **A lei que fica: a máscara decide quando tem PROVA.** Se ela CORTOU
//! vértices e não sobrou nenhum, decidiu — e o dab morre para todos. Se a
//! pegada já estava vazia, não havia um único vértice sobre que julgar, e a
//! unidade que sobra é a amostra. As cercas ficam inteiras para quem escreve
//! posição, máscara ou cor por-vértice.
//!
//! # ⛔⛔ E a máscara já era INERTE para a cor fina
//!
//! A `Connected Only` — que o dono mandou shipar LIGADA — corta VÉRTICES, e a
//! cura do report de 19/09 (*«Snake Hook … deformando a face POSTERIOR»*)
//! protege o vértice. Medido na barbatana de `0,06` de espessura, o MESMO
//! traço com ela armada e desarmada:
//!
//! | raio | antes: armada / desarmada | depois: armada / desarmada |
//! |---|---|---|
//! | 64 px | `66 179` / `66 179` | **`33 841`** / `66 179` |
//! | 24 px | `13 035` / `13 035` | **`7 603`** / `13 035` |
//! | 10 px | `2 536` / `2 536` | `2 536` / `2 536` ← **CONTROLO** |
//!
//! ⭐ *A linha do raio pequeno é o controlo que dá direito às outras duas:* ali
//! a esfera do dab não alcança as costas da folha, logo não há o que cortar.
//!
//! ⚠️ **Das três condições daquela máscara só a NORMAL se transplanta:** a
//! conectividade e a razão `superfície/ar` são um passeio por ARESTAS, e uma
//! amostra não tem vizinhos no grafo.
//!
//! # ⭐⭐⭐ O que muda, e o que NÃO muda
//!
//! **Não muda a lei do pincel.** O peso de uma amostra sai das MESMAS portas do
//! dab por-vértice ([`crate::peso_do_ponto`]), e a composição é a MESMA
//! ([`crate::GripLaw`]): o [`Verb::Paint`] compõe como tinta a partir da cor
//! congelada, e os dois que leem o anel compõem **por dab** com o `w` cru. O
//! que muda é ONDE a distância é medida — na amostra, e não no vértice. *É daí
//! que vem a resolução, e é só daí.*
//!
//! # ⚠️ De onde vêm as outras entradas do peso
//!
//! | entrada | no dab por-vértice | aqui |
//! |---|---|---|
//! | **distância** | a posição do vértice | ⭐ **a posição da AMOSTRA** |
//! | normal (front-face) | `base_nrm[s]` congelada | interpolada dos cantos |
//! | máscara | `base_mask[s]` congelada | interpolada dos cantos |
//! | padrão (alpha) | na posição congelada | na posição da amostra |
//!
//! ⭐⭐ **Ler a malha VIVA aqui é correcto, e é propriedade do VERBO e não uma
//! conveniência:** os três verbos de cor **não movem um vértice, não mudam uma
//! normal e não escrevem a máscara** — durante um traço de cor o vivo **é** o
//! congelado. Há gate a afirmá-lo; no dia em que um verbo de cor mexer em
//! geometria, ele reprova.
//!
//! # ⛔⛔ O anel na BORDA da pegada: uma MARGEM foi escrita, MEDIDA e RETIRADA
//!
//! O anel de uma amostra sai das faces que a consulta devolveu, logo uma
//! amostra no bordo da esfera poderia perder um vizinho que vive numa face que
//! a consulta não trouxe. Escrevi uma `MARGEM_DO_ANEL` para o cobrir e depois
//! varri-a contra o caminho por-vértice (que lê a adjacência INTEIRA da malha):
//!
//! | margem | desvio do `Blur` a `lado = 1` |
//! |---|---|
//! | `1,00` | `5,96e-8` |
//! | `1,25` · `1,50` · `2,00` | `5,96e-8` |
//!
//! ⇒ **ela não compra um bit** — `5,96e-8` é um ULP de `f32` perto de `1`, e é
//! o mesmo nas quatro. A razão é que a consulta do octree já é **conservadora**
//! (ela devolve as faces das FOLHAS que a esfera toca, não as que a esfera
//! corta). *Uma constante que a medição não consegue mover é um comentário com
//! sintaxe de código* ⇒ ela saiu, e quem garante a propriedade é o gate
//! `os_dois_que_leem_o_anel_concordam_com_o_caminho_por_vertice`, que a mede
//! contra a adjacência inteira.

use ph2d_mesh::{DEFAULT_MASK, Mesh};
use ph2d_mesh_colors::Tinta;

use crate::AlphaFrame;
use crate::{Brush, Dab, Footprint, Verb};

/// Uma amostra das faces que o dab tocou.
#[derive(Debug, Clone, Copy)]
pub struct Apanhada {
    /// Índice global no plano.
    pub idx: u32,
    /// Onde ela está no mundo.
    pub pos: [f32; 3],
    /// A normal interpolada dos cantos da face.
    pub nrm: [f32; 3],
    /// Quanto a máscara deixa passar.
    pub keep: f32,
    /// Está DENTRO da esfera do dab? As de fora entram na conta do anel e não
    /// pagam o cálculo do peso.
    ///
    /// ⚠️⚠️ **Esta é uma cerca de TRABALHO e não de correcção, e uma mutação
    /// SOBREVIVENTE é que o disse:** pô-la sempre a `true` não muda um bit da
    /// saída, porque toda curva de queda desta casa passa pela
    /// [`crate::falloff::fora_da_pegada`] e devolve `0` em `t >= 1` — e o laço
    /// de escrita salta `w <= 0`. *Ela fica porque poupa o peso de metade das
    /// amostras que a consulta traz, e fica DECLARADA como tal em vez de se
    /// apresentar como guarda.*
    pub dentro: bool,
}

/// O que o dab já calculou e a tinta fina reaproveita.
///
/// ⭐ **Nada aqui é recalculado**: os quatro saem do preâmbulo do
/// [`crate::stroke_dab_core`], que é a mesma chamada. *Reconstruí-los seria a
/// segunda resposta à mesma pergunta.*
pub struct ContextoDoDab<'a> {
    pub footprint: &'a Footprint,
    pub alpha_frame: &'a AlphaFrame,
    pub inv_r: f32,
    pub intensity: f32,
}

/// ⭐ **A TINTA FINA de um traço** — o plano emprestado, mais o estado que a lei
/// do envelope pede.
///
/// ⚠️⚠️ **Ela é EMPRESTADA ao traço e não copiada**, e é isso que decide a
/// assinatura de tudo o que está abaixo: o plano vive na PEÇA e é *movido* para
/// o [`crate::SculptStroke`] enquanto o gesto dura (o campo `tinta_fina`, que é
/// um `Option` por isso mesmo). ⛔ A alternativa — o dab receber um
/// `&mut Tinta` — mudaria a assinatura do [`crate::SculptStroke::dab`], **que é
/// a porta por onde TODOS os corpora de oráculo desta casa entram**. *Um corpus
/// que não pode ser tocado por uma feature nova é o que torna a feature nova
/// barata de provar.*
#[derive(Debug, Clone)]
pub struct TintaDoTraco {
    tinta: Tinta,
    /// ⭐⭐⭐⭐ **QUEM emprestou o plano** — um símbolo OPACO do chamador, que
    /// esta crate nunca interpreta.
    ///
    /// ⛔⛔ **Ele existe porque o empréstimo tem duas pontas e nada as prendia
    /// à mesma peça** (achado da auditoria de 21/09, §10.5 do handoff): o
    /// pen-down pede o plano à peça **ACTIVA** e o `close_stroke` devolvia-o à
    /// peça **ACTIVA** — se o índice mudar entre os dois, o plano da peça A
    /// aterra na B e a A fica sem ele, *que é o mesmo sintoma do report do
    /// dono por outra porta*.
    ///
    /// ⚠️ **Ele viaja DENTRO do empréstimo e não ao lado dele**, de propósito:
    /// um `Option<ObjectId>` na cena ao lado de um `Option<TintaDoTraco>` no
    /// traço são **dois campos que têm de concordar**, e esta casa já pagou
    /// essa forma — aqui a pergunta *«de quem é este plano?»* tem uma resposta
    /// só, e ela morre com o plano.
    dono: u32,
    /// Por amostra: `slot + 1`, com `0` a querer dizer *nunca tocada*.
    slot: Vec<u32>,
    accum: Vec<f32>,
    base: Vec<[f32; 3]>,
    tocadas: Vec<u32>,
    /// ⭐⭐⭐⭐ **Que amostras mudaram desde o último upload** — paralelo a
    /// [`Self::tocadas`], e indexado por SLOT e não por amostra.
    ///
    /// ⚠️⚠️ **O índice é o slot de propósito, e é isso que a torna afordável:**
    /// um vector por AMOSTRA custaria `O(plano)` — `25 M` entradas no degrau
    /// mais fino da peça de fábrica —, e este custa `O(pegada do traço)`, que é
    /// a mesma grandeza que a janela do desfazer que já vive ao lado.
    suja: Vec<bool>,
    // ── o rascunho de um dab ──
    faces: Vec<u32>,
    carimbo: Vec<u32>,
    local: Vec<u32>,
    epoca: u32,
    amostras: Vec<Apanhada>,
    pares: Vec<(u32, u32)>,
}

impl TintaDoTraco {
    /// Empresta o plano ao traço — `dono` é o símbolo de quem o emprestou, e
    /// esta crate **não o interpreta**: ver [`Self::dono`].
    #[must_use]
    pub fn nova(tinta: Tinta, dono: u32) -> Self {
        let n = tinta.amostras().len();
        Self {
            tinta,
            dono,
            slot: vec![0; n],
            accum: Vec::new(),
            base: Vec::new(),
            tocadas: Vec::new(),
            suja: Vec::new(),
            faces: Vec::new(),
            carimbo: vec![0; n],
            local: vec![0; n],
            epoca: 0,
            amostras: Vec::new(),
            pares: Vec::new(),
        }
    }

    /// Devolve o plano ao dono.
    #[must_use]
    pub fn entregar(self) -> Tinta {
        self.tinta
    }

    /// **Quem emprestou este plano** — o símbolo opaco que o [`Self::nova`]
    /// recebeu.
    #[must_use]
    pub fn dono(&self) -> u32 {
        self.dono
    }

    /// O plano, para quem o sobe ao device.
    #[must_use]
    pub fn tinta(&self) -> &Tinta {
        &self.tinta
    }

    /// As amostras que este traço tocou, na ordem em que foram tocadas.
    #[must_use]
    pub fn tocadas(&self) -> &[u32] {
        &self.tocadas
    }

    /// A cor de ANTES do traço, na ordem de [`Self::tocadas`] — a janela do
    /// desfazer.
    #[must_use]
    pub fn base(&self) -> &[[f32; 3]] {
        &self.base
    }

    /// ⭐⭐ **As amostras das faces que o dab tocou, cada uma UMA vez.**
    ///
    /// ⚠️ **A deduplicação é obrigatória e não é arrumação:** uma amostra de
    /// aresta pertence às DUAS faces que ali se tocam, e uma de canto a todas
    /// as faces do anel. Aplicar uma mistura duas vezes no mesmo sítio dá uma
    /// cor diferente de a aplicar uma — *a tinta ficaria mais forte exactamente
    /// nas arestas da malha*, ou seja o desenho da malha a aparecer na pintura.
    /// O carimbo de época é o idioma do [`ph2d_mesh::QueryScratch`].
    fn apanha(&mut self, mesh: &Mesh, centro: [f32; 3], raio: f32) {
        self.amostras.clear();
        self.pares.clear();
        let mut faces = std::mem::take(&mut self.faces);
        mesh.octree().faces_in_sphere(centro, raio, &mut faces);
        self.epoca = self.epoca.wrapping_add(1);
        if self.epoca == 0 {
            self.epoca = 1;
            self.carimbo.fill(0);
        }
        let r2 = raio * raio;
        for &fi in &faces {
            // ⭐ **O lado é da FACE e não da peça** (a P2): ler um `lado` só
            //   para o laço inteiro daria a uma face grossa os pesos de uma
            //   fina, e a tinta sairia no sítio errado **sem nada a acusar**.
            let lado =
                f32::from(u16::try_from(self.tinta.lado_da_face(fi as usize)).unwrap_or(u16::MAX));
            let face = mesh.faces()[fi as usize];
            let cantos = face.verts();
            let (tinta, carimbo, local, amostras) = (
                &self.tinta,
                &mut self.carimbo,
                &mut self.local,
                &mut self.amostras,
            );
            let epoca = self.epoca;
            let mut pousa = |idx: u32, w: &[f32], pos: [f32; 3], nrm: [f32; 3], m: &[f32]| {
                if carimbo[idx as usize] == epoca {
                    return;
                }
                carimbo[idx as usize] = epoca;
                local[idx as usize] = u32::try_from(amostras.len()).unwrap_or(u32::MAX);
                let d = [pos[0] - centro[0], pos[1] - centro[1], pos[2] - centro[2]];
                amostras.push(Apanhada {
                    idx,
                    pos,
                    nrm: unitario(nrm),
                    // ⭐ A lei partilhada com o `Fill` — ver [`crate::preenche`].
                    keep: crate::preenche::keep_da_amostra(w, m),
                    dentro: d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r2,
                });
            };
            // ⚠️ **As DUAS formas, e elas não partilham a retícula:** um
            // triângulo é baricêntrico e um quad é bilinear. Saltar os quads
            // aqui foi o meu primeiro defeito nesta wave, e ele é MUDO — a
            // `uv_sphere` desta casa é quase toda de quads, logo *o pincel não
            // pintava nada e o gate leu «o nível zero divergiu»*.
            let m: Vec<f32> = cantos.iter().map(|&v| mascara(mesh, v)).collect();
            if cantos.len() == 3 {
                let p = tres(cantos, mesh.positions());
                let n = tres(cantos, mesh.normals());
                tinta.para_cada_amostra_tri(fi as usize, cantos, |idx, (i, j, k)| {
                    let w = [i as f32 / lado, j as f32 / lado, k as f32 / lado];
                    pousa(idx, &w, mistura3(&p, w), mistura3(&n, w), &m);
                });
            } else {
                let p = quatro(cantos, mesh.positions());
                let n = quatro(cantos, mesh.normals());
                tinta.para_cada_amostra_quad(fi as usize, cantos, |idx, (i, j)| {
                    let w = bilinear(i as f32 / lado, j as f32 / lado);
                    pousa(idx, &w, mistura4(&p, w), mistura4(&n, w), &m);
                });
            }
        }
        // Os PARES, em índices locais — a segunda passagem porque ela precisa
        // do `local` de TODAS as amostras, inclusive das da face seguinte.
        for &fi in &faces {
            let face = mesh.faces()[fi as usize];
            let cantos = face.verts();
            let (tinta, carimbo, local, pares) =
                (&self.tinta, &self.carimbo, &self.local, &mut self.pares);
            let epoca = self.epoca;
            let mut par = |a: u32, b: u32| {
                if carimbo[a as usize] == epoca && carimbo[b as usize] == epoca {
                    pares.push((local[a as usize], local[b as usize]));
                }
            };
            if cantos.len() == 3 {
                tinta.para_cada_par_tri(fi as usize, cantos, &mut par);
            } else {
                tinta.para_cada_par_quad(fi as usize, cantos, &mut par);
            }
        }
        self.faces = faces;
    }

    /// O slot de uma amostra, registando a base na PRIMEIRA vez que ela é vista
    /// neste traço. É esta primeira vez que enche a janela do desfazer.
    fn slot_de(&mut self, idx: u32) -> usize {
        let s = self.slot[idx as usize];
        if s != 0 {
            return (s - 1) as usize;
        }
        let novo = self.tocadas.len();
        self.slot[idx as usize] = u32::try_from(novo + 1).unwrap_or(u32::MAX);
        self.tocadas.push(idx);
        self.base.push(self.tinta.amostras()[idx as usize]);
        self.accum.push(0.0);
        // Nasce suja: quem pede um slot é quem está prestes a escrever nele.
        self.suja.push(true);
        novo
    }

    /// ⭐ **Este traço já tocou esta amostra?** — sem a capturar.
    #[must_use]
    pub fn tocou(&self, idx: u32) -> bool {
        self.slot[idx as usize] != 0
    }

    /// ⭐⭐ **Escreve a amostra a partir da cor de ANTES do traço** — a porta do
    /// Painter sobre a peça ([`crate::tela_na_malha`]). Capturar e escrever são
    /// um acto só, logo a janela do desfazer e a do upload enchem-se como as
    /// do pincel de pintura. Devolve se a amostra mudou.
    pub fn repinta(&mut self, idx: u32, cor: impl FnOnce([f32; 3]) -> [f32; 3]) -> bool {
        let s = self.slot_de(idx);
        let nova = cor(self.base[s]);
        let viva = &mut self.tinta.amostras_mut()[idx as usize];
        if *viva == nova {
            return false;
        }
        *viva = nova;
        self.suja[s] = true;
        true
    }

    /// ⭐⭐⭐⭐ **As amostras escritas desde a última vez que isto foi chamado**
    /// — a janela que o upload do device consome, e a razão de ele deixar de
    /// ser `O(plano)`.
    ///
    /// ⛔⛔ **MEDIDO em 2026-09-20, e é o muro que o degrau novo tornaria
    /// intransponível:** subir o plano INTEIRO por quadro custa, só para
    /// empacotar, `21,9 ms` no degrau `8×` da peça de fábrica (`72 MB`) e
    /// `81,7 ms` no `16×` (`288 MB`) — contra um quadro de `16,7`. *A escrita
    /// da tinta fina é por AMOSTRA e nunca passou pelo `dirty`, que é uma
    /// janela de VÉRTICES; era essa a dívida nomeada.*
    ///
    /// ⚠️ **Ela LIMPA as marcas**, logo dois consumidores no mesmo quadro
    /// dividiriam a janela entre si e o device perderia metade. O único
    /// chamador é o laço de upload.
    pub fn drena_sujas(&mut self, out: &mut Vec<u32>) {
        out.clear();
        for (s, suja) in self.suja.iter_mut().enumerate() {
            if *suja {
                *suja = false;
                out.push(self.tocadas[s]);
            }
        }
    }
}

/// Os pesos bilineares dos quatro cantos, na ordem `a, b, c, d` do percurso.
pub(crate) fn bilinear(u: f32, v: f32) -> [f32; 4] {
    [(1.0 - u) * (1.0 - v), u * (1.0 - v), u * v, (1.0 - u) * v]
}

fn quatro(cantos: &[u32], v: &[[f32; 3]]) -> [[f32; 3]; 4] {
    [
        v[cantos[0] as usize],
        v[cantos[1] as usize],
        v[cantos[2] as usize],
        v[cantos[3] as usize],
    ]
}

fn mistura4(v: &[[f32; 3]; 4], w: [f32; 4]) -> [f32; 3] {
    let mut o = [0.0f32; 3];
    for (q, k) in v.iter().zip(w) {
        for e in 0..3 {
            o[e] += q[e] * k;
        }
    }
    o
}

fn tres(cantos: &[u32], v: &[[f32; 3]]) -> [[f32; 3]; 3] {
    [
        v[cantos[0] as usize],
        v[cantos[1] as usize],
        v[cantos[2] as usize],
    ]
}

fn mascara(mesh: &Mesh, v: u32) -> f32 {
    mesh.masks().map_or(DEFAULT_MASK, |k| k[v as usize])
}

fn mistura3(v: &[[f32; 3]; 3], w: [f32; 3]) -> [f32; 3] {
    [
        v[0][0] * w[0] + v[1][0] * w[1] + v[2][0] * w[2],
        v[0][1] * w[0] + v[1][1] * w[1] + v[2][1] * w[2],
        v[0][2] * w[0] + v[1][2] * w[1] + v[2][2] * w[2],
    ]
}

fn unitario(v: [f32; 3]) -> [f32; 3] {
    let q = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if q == 0.0 {
        return [0.0, 1.0, 0.0];
    }
    let inv = 1.0 / q.sqrt();
    [v[0] * inv, v[1] * inv, v[2] * inv]
}

impl crate::SculptStroke {
    /// ⭐⭐⭐ **O DAB DE COR sobre a retícula.** Irmão do `apply_color` do
    /// [`crate::stroke_apply`], e o corte é o SUJEITO: lá o vértice, aqui a
    /// amostra.
    ///
    /// Devolve quantas amostras mudaram.
    pub(crate) fn apply_color_fino(
        &mut self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        ctx: &ContextoDoDab,
    ) -> usize {
        let Some(fina) = self.tinta_fina.as_mut() else {
            return 0;
        };
        fina.apanha(mesh, dab.center, dab.radius);
        if fina.amostras.is_empty() {
            return 0;
        }
        // ── 1. O PESO de cada amostra, pelas portas do dab por-vértice. ──
        let amostras = std::mem::take(&mut fina.amostras);
        // ⚠️⚠️ **A SEGUNDA METADE da lei da folha é o ARMAR, e a minha 1.ª
        // redacção largou-a — SEIS gates de lei apanharam-na.** A máscara de
        // alcance só corta *«se alguma coisa na pegada estiver virada ao
        // artista»*; sem essa pergunta, uma fixtura cujo olho é RADIAL
        // (`Dab::at(c, r, c)`, o construtor que os gates deste ficheiro usam)
        // lê toda a gente virada ao contrário e a cor DESAPARECE.
        // *O valor conservador é não cortar quando não há prova.*
        let le =
            (dab.eye[0] * dab.eye[0] + dab.eye[1] * dab.eye[1] + dab.eye[2] * dab.eye[2]).sqrt();
        let olho_bom = le.is_finite() && le > 1e-6;
        let dot_olho = |n: [f32; 3]| -> f32 {
            (n[0] * dab.eye[0] + n[1] * dab.eye[1] + n[2] * dab.eye[2]) / le
        };
        // ⛔⛔⛔ **E a POPULAÇÃO do armar é quem está DENTRO do raio** — a 1.ª
        // redacção varria todas as colhidas e SEIS gates de lei reprovaram com
        // *«o traço não tocou amostra nenhuma»*. A consulta de amostras é por
        // CAIXA (`faces_in_sphere`) e devolve faces do outro lado da peça; a
        // pegada de vértices, que é quem a máscara arma, é uma ESFERA. Com as
        // de fora dentro da conta, uma amostra da face oposta (virada ao
        // artista) ARMAVA a cerca e as do cursor — que numa fixtura de olho
        // radial leem `+1` — eram todas cortadas. *Uma lei transplantada tem
        // de trazer a população dela junto.*
        let corta_a_folha = brush.surface_only
            && brush.offers_surface_only()
            && olho_bom
            && amostras
                .iter()
                .filter(|a| a.dentro)
                .any(|a| dot_olho(a.nrm) <= crate::dab_alcance::NORMAL_LIMIAR);
        let mut pesos: Vec<f32> = Vec::with_capacity(amostras.len());
        for a in &amostras {
            if !a.dentro {
                pesos.push(0.0);
                continue;
            }
            let d = [
                a.pos[0] - dab.center[0],
                a.pos[1] - dab.center[1],
                a.pos[2] - dab.center[2],
            ];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let curve =
                crate::peso_do_ponto::curva_do_ponto(brush, ctx.footprint, a.pos, dist, ctx.inv_r);
            // ⭐⭐⭐⭐ **A LEI DA FOLHA, na unidade que ESTE laço escreve.**
            //
            // ⛔⛔ A máscara de alcance ([`crate::dab_alcance`]) corta VÉRTICES,
            // e o report de 2026-09-19 (*«Snake Hook … deformando a face
            // POSTERIOR do traço»*) foi curado ali. A cor fina escreve
            // AMOSTRAS, e o A/B na barbatana mediu a máscara **inerte** para
            // ela: `66 179`, `13 035` e `2 536` amostras pintadas com ela
            // armada **e** desarmada, aos três raios.
            //
            // ⚠️ **Das três condições daquela máscara só a NORMAL se transplanta
            // honestamente:** a conectividade e a razão `superfície/ar` são um
            // passeio por ARESTAS, e uma amostra não tem vizinhos no grafo. A
            // folha que o olho vê, essa, é uma pergunta POR PONTO — e é a que
            // guarda a parede fina.
            //
            // ⚠️ O olho degenerado **desliga** a condição em vez de cortar ao
            // acaso, que é o valor conservador que a máscara já escolhe.
            if corta_a_folha && dot_olho(a.nrm) > crate::dab_alcance::NORMAL_LIMIAR {
                pesos.push(0.0);
                continue;
            }
            let facing = match brush.mode.kernel_for(brush.verb).front_face {
                crate::FrontFace::Continuous if brush.front_faces_only => {
                    (-(a.nrm[0] * dab.eye[0] + a.nrm[1] * dab.eye[1] + a.nrm[2] * dab.eye[2]))
                        .max(0.0)
                }
                crate::FrontFace::Ignored | crate::FrontFace::Continuous => 1.0,
            };
            let (w, _forma) = crate::peso_do_ponto::peso_e_forma(
                curve,
                brush.alpha_weight(a.pos, ctx.alpha_frame),
                facing,
                ctx.intensity,
                a.keep,
            );
            pesos.push(w);
        }
        // ── 2. Os ALVOS, TODOS antes de qualquer escrita. ──
        let alvos = self.alvos_de_cor_fino(brush, dab, &amostras);
        // ── 3. A escrita. ──
        let fina = self.tinta_fina.as_mut().expect("conferido acima");
        let pintura = brush.verb == Verb::Paint;
        let mut n = 0;
        for ((a, &w), alvo) in amostras.iter().zip(&pesos).zip(alvos) {
            if w <= 0.0 {
                continue;
            }
            let s = fina.slot_de(a.idx);
            // ⭐ A composição é a do [`crate::GripLaw`], verbo a verbo: a pintura
            // é uma TINTA que compõe ao longo do traço a partir da cor
            // congelada; os dois que leem o anel compõem **por dab**, com o `w`
            // cru, porque o alvo deles muda a cada dab.
            let acc = if pintura {
                fina.accum[s] += w * (1.0 - fina.accum[s]);
                fina.accum[s].clamp(0.0, 1.0)
            } else {
                w.clamp(0.0, 1.0)
            };
            let de = if pintura {
                fina.base[s]
            } else {
                fina.tinta.amostras()[a.idx as usize]
            };
            let out = &mut fina.tinta.amostras_mut()[a.idx as usize];
            for k in 0..3 {
                out[k] = de[k] * (1.0 - acc) + alvo[k] * acc;
            }
            // ⚠️ **Marcada a cada escrita e não só na primeira:** um dab
            // seguinte re-escreve uma amostra que o anterior já tocou (o
            // `accum` cresce ao longo do traço), logo *«tocada uma vez»* e
            // *«mudou desde o último upload»* são grandezas diferentes.
            fina.suja[s] = true;
            n += 1;
        }
        fina.amostras = amostras;
        n
    }

    /// Os ALVOS das três leis de cor, sobre amostras.
    ///
    /// ⭐⭐ O [`Verb::Paint`] deposita a cor do pincel; os dois que leem o ANEL
    /// puxam-na da vizinhança — e a vizinhança aqui é a **retícula**, que
    /// atravessa a aresta da malha porque a fronteira é PARTILHADA. *É esta
    /// linha que o atlas não consegue escrever: lá, o vizinho de um texel pode
    /// estar noutra ponta da peça.*
    fn alvos_de_cor_fino(&self, brush: &Brush, dab: &Dab, amostras: &[Apanhada]) -> Vec<[f32; 3]> {
        if brush.verb == Verb::Paint {
            return vec![brush.color; amostras.len()];
        }
        let fina = self.tinta_fina.as_ref().expect("chamado de dentro do dab");
        let cor = |i: usize| fina.tinta.amostras()[amostras[i].idx as usize];
        // ⚠️⚠️ **A PRÓPRIA amostra entra com peso `1`, nas DUAS leis** — é uma
        // relaxação *para* a vizinhança e não uma substituição por ela. Sem
        // ela, um dab a peso cheio apaga a cor de uma vez e o pincel deixa de
        // ter gradação. *Esquecê-la foi o meu segundo defeito nesta wave, e o
        // gate contra o caminho por-vértice mediu-o em `3,8e-2`.*
        let mut soma: Vec<[f32; 3]> = (0..amostras.len()).map(cor).collect();
        let mut peso = vec![1.0f32; amostras.len()];
        let smear = brush.verb == Verb::SmearColor;
        for &(a, b) in &fina.pares {
            let (ia, ib) = (a as usize, b as usize);
            let (ca, cb) = (cor(ia), cor(ib));
            let (pa, pb) = (amostras[ia].pos, amostras[ib].pos);
            let (wa, wb) = if smear {
                // ⚠️ **A direcção é do MODO e não do caminho** — os três modos
                // do esfregão (arrastar · apertar · espalhar) e a
                // degenerescência de cada um vivem na
                // [`crate::SmearMode::direction`], que é a porta que o caminho
                // por-vértice também lê.
                let da = unit_ou_nada(brush.smear_mode.direction(dab.path, dab.center, pa));
                let db = unit_ou_nada(brush.smear_mode.direction(dab.path, dab.center, pb));
                let e = unit_ou_nada([pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]]);
                match (da, db, e) {
                    (_, _, None) => (0.0, 0.0),
                    (da, db, Some(e)) => {
                        // `a` recebe de `b` quando `b` está a MONTANTE de `a`.
                        let ga = da.map_or(0.0, |d| {
                            (-(d[0] * e[0] + d[1] * e[1] + d[2] * e[2])).max(0.0)
                        });
                        let gb =
                            db.map_or(0.0, |d| (d[0] * e[0] + d[1] * e[1] + d[2] * e[2]).max(0.0));
                        (ga, gb)
                    }
                }
            } else {
                (1.0, 1.0)
            };
            if wa > 0.0 {
                for k in 0..3 {
                    soma[ia][k] += cb[k] * wa;
                }
                peso[ia] += wa;
            }
            if wb > 0.0 {
                for k in 0..3 {
                    soma[ib][k] += ca[k] * wb;
                }
                peso[ib] += wb;
            }
        }
        // ⛔⛔ **DIVIDIR, e não multiplicar pelo recíproco** — é a única coisa
        // que torna a lei um NO-OP AO BIT numa peça de cor uniforme, e é uma
        // lei escrita no [`crate::stroke_cor`] com a medição ao lado. *Um
        // pincel que muda a peça onde não há nada a mudar é um passo de undo,
        // um upload de GPU e um ficheiro diferente por nada.*
        (0..amostras.len())
            .map(|i| {
                [
                    soma[i][0] / peso[i],
                    soma[i][1] / peso[i],
                    soma[i][2] / peso[i],
                ]
            })
            .collect()
    }
}

/// O unitário, ou `None` quando o vector **não tem direcção**. Irmão do da
/// [`crate::stroke_cor`], e pela mesma razão: o limiar é o zero EXACTO.
fn unit_ou_nada(v: [f32; 3]) -> Option<[f32; 3]> {
    let q = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if q == 0.0 {
        return None;
    }
    let inv = 1.0 / q.sqrt();
    Some([v[0] * inv, v[1] * inv, v[2] * inv])
}
