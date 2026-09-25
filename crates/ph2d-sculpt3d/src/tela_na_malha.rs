//! ⭐⭐⭐ **A TELA DO PAINTER POUSADA NA PEÇA** — a metade de LEI da integração
//! do Painter na malha (ordem do dono, 2026-09-24: *«a integração total do
//! módulo Painter já existente para que consiga pintar com os mesmos features
//! na malha 3d»*).
//!
//! # O desenho, e porque é este
//!
//! O Painter pinta numa **imagem do tamanho da vista** — a mesma imagem sobre
//! que ele pinta numa sprite, só que transparente e alinhada ao ecrã. Esta
//! crate não sabe o que é o Painter: ela recebe a imagem e pousa-a na peça.
//! ⇒ *todo o motor do Painter (pincéis, textura, forma, pressão, espaçamento)
//! corre sem uma linha nova*, porque a superfície onde ele pinta É uma imagem
//! verdadeira. É a via do `docs/3D/25` §11.
//!
//! # ⭐⭐ Cada amostra RE-DERIVA-SE da cor de antes do traço
//!
//! `nova = base·(1 − a·k) + c·a·k`, com `(c·a, a)` lidos da tela em
//! PRÉ-MULTIPLICADO e `k` a liberdade da máscara ([`crate::preenche::keep_da_amostra`],
//! a mesma lei do pincel e do `Fill`). ⚠️ **A partida é a `base` e nunca a cor
//! viva**, e é isso que deixa esta função ser chamada a cada quadro sobre o
//! mesmo rectângulo sem a tinta engrossar: pousar duas vezes a mesma tela dá
//! EXACTAMENTE o que pousar uma — há gate. É também o que a torna o «over» de
//! uma camada: o traço inteiro do Painter compõe-se sobre a peça como a camada
//! dele se compõe sobre a sprite.
//!
//! # ⭐⭐ A tela SEMEADA: a lei passa a ser a DIFERENÇA (etapa 2)
//!
//! Os modos que lêem a cor debaixo do pincel (borrar, esfumar, clonar, o balde,
//! a aquarela…) começam com o RETRATO da peça na tela ([`crate::tela_semente`]),
//! e aí cada amostra recebe `nova = base + k·(c − s)` — `c` a cor que a tela
//! ficou, `s` a do retrato, as duas DIRECTAS. ⚠️ O que o pincel não tocou tem
//! `c − s = 0` exactamente (os mesmos bytes dos dois lados) e sai ANTES do raio
//! de oclusão; e a diferença SOMA-se à base, logo o detalhe fino debaixo fica.
//! A pintura simples continua no «over»: com uma tinta opaca a diferença
//! deixaria passar o detalhe mais fino do que um píxel.
//!
//! # ⚠️ Só pinta o que se VÊ — o preço declarado
//!
//! Uma amostra entra quando (1) a face dela está de FRENTE para o olho, (2) cai
//! dentro do rectângulo que o Painter mudou, e (3) o raio do olho até ela não
//! bate noutra superfície antes ([`FOLGA_DA_OCLUSAO`]). ⇒ o lado de trás fica
//! por pintar até o artista rodar a peça, como na pintura por projecção de
//! todo programa de referência.
//!
//! # ⚠️ A visibilidade decide-se UMA vez por traço
//!
//! Um raio custa `~0,44 µs` (medido na wave do `Scene Project`), e a câmera e a
//! forma não mudam durante uma pincelada de cor ⇒ a resposta de cada amostra é
//! guardada na primeira vez que ela é pedida, e o traço paga um raio por
//! amostra que TOCA, nunca por quadro.

use ph2d_mesh::{Mesh, Ray};

/// Lado de uma célula da grelha de faces, em píxeis da tela.
const CELULA: f32 = 32.0;

/// ⚠️ **Quanto mais perto do olho um obstáculo tem de estar para ESCONDER a
/// amostra**, em fracção da distância olho→amostra. O raio que acerta a própria
/// face devolve `t ≈ dist` a menos do erro de `f32` da interseção; `1e-3` é
/// folga para esse erro e ainda separa uma dobra da peça a `0,1 %` da distância
/// da câmera.
pub const FOLGA_DA_OCLUSAO: f32 = 1e-3;

/// A imagem que o Painter pintou: RGBA8 **não** pré-multiplicado, bytes sRGB —
/// o formato do canvas dele.
#[derive(Clone, Copy)]
pub struct Tela<'a> {
    /// Os píxeis, `largura·altura·4`.
    pub rgba: &'a [u8],
    /// Largura em píxeis.
    pub largura: u32,
    /// Altura em píxeis.
    pub altura: u32,
}

impl Tela<'_> {
    /// ⚠️ **Fora da tela repete-se a BORDA** e não o transparente: um ponto
    /// exactamente na borda da vista está na superfície que o artista vê, e
    /// ler meio píxel de «nada» ali pintava a orla da vista a meia força.
    fn texel(&self, i: i64, j: i64) -> ([f32; 3], f32) {
        if self.largura == 0 || self.altura == 0 {
            return ([0.0; 3], 0.0);
        }
        let i = i.clamp(0, i64::from(self.largura) - 1);
        let j = j.clamp(0, i64::from(self.altura) - 1);
        let o = ((j as usize) * self.largura as usize + i as usize) * 4;
        let Some(px) = self.rgba.get(o..o + 4) else {
            return ([0.0; 3], 0.0);
        };
        let a = f32::from(px[3]) / 255.0;
        (
            [
                f32::from(px[0]) / 255.0 * a,
                f32::from(px[1]) / 255.0 * a,
                f32::from(px[2]) / 255.0 * a,
            ],
            a,
        )
    }

    /// ⭐ **A cor PRÉ-MULTIPLICADA e a cobertura** num ponto contínuo da tela.
    ///
    /// ⚠️ **Bilinear em pré-multiplicado**, e não em cor crua: interpolar a cor
    /// de um píxel pintado com a de um transparente (cujo RGB é lixo) tingiria a
    /// borda do traço. ⚠️ O centro do píxel `i` é `i + 0,5` — a convenção do
    /// ponteiro do Painter, que recebe a mesma coordenada.
    #[must_use]
    pub fn amostra(&self, x: f32, y: f32) -> ([f32; 3], f32) {
        let (fx, fy) = (x - 0.5, y - 0.5);
        let (x0, y0) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - x0, fy - y0);
        let (i, j) = (x0 as i64, y0 as i64);
        let mut c = [0.0f32; 3];
        let mut a = 0.0f32;
        for (di, dj, w) in [
            (0, 0, (1.0 - tx) * (1.0 - ty)),
            (1, 0, tx * (1.0 - ty)),
            (0, 1, (1.0 - tx) * ty),
            (1, 1, tx * ty),
        ] {
            if w == 0.0 {
                continue;
            }
            let (pc, pa) = self.texel(i + di, j + dj);
            for k in 0..3 {
                c[k] += pc[k] * w;
            }
            a += pa * w;
        }
        (c, a)
    }
}

/// A vista em que a tela foi pintada: de espaço LOCAL da peça para píxeis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vista {
    /// Local → clip, coluna a coluna (a pose já dentro).
    local_para_clip: [f32; 16],
    largura: f32,
    altura: f32,
    /// O olho, em espaço LOCAL da peça.
    olho: [f32; 3],
}

impl Vista {
    /// `local_para_clip` em colunas (a ordem do `glam::Mat4::to_cols_array`),
    /// o tamanho da tela e o olho em espaço local.
    #[must_use]
    pub fn nova(local_para_clip: [f32; 16], tamanho: (u32, u32), olho: [f32; 3]) -> Self {
        Self {
            local_para_clip,
            largura: tamanho.0.max(1) as f32,
            altura: tamanho.1.max(1) as f32,
            olho,
        }
    }

    /// Onde um ponto local cai na tela — a conversão do `Camera3d::project`
    /// (`x` para a direita, `y` para BAIXO). `None` atrás do olho.
    #[must_use]
    pub fn ecra(&self, p: [f32; 3]) -> Option<[f32; 2]> {
        let m = &self.local_para_clip;
        let lin = |r: usize| m[r] * p[0] + m[4 + r] * p[1] + m[8 + r] * p[2] + m[12 + r];
        let w = lin(3);
        if w <= 0.0 {
            return None;
        }
        let (nx, ny) = (lin(0) / w, lin(1) / w);
        Some([
            (nx + 1.0) * 0.5 * self.largura,
            (1.0 - ny) * 0.5 * self.altura,
        ])
    }

    /// O olho, em espaço local.
    #[must_use]
    pub fn olho(&self) -> [f32; 3] {
        self.olho
    }

    /// O tamanho da tela, em píxeis.
    #[must_use]
    pub fn tamanho(&self) -> (u32, u32) {
        (self.largura as u32, self.altura as u32)
    }

    /// O ponto na tela e `1/w` — o inverso da profundidade de clip, que é
    /// LINEAR no ecrã e por isso o que um rasterizador interpola (maior = mais
    /// perto). `None` atrás do olho.
    pub(crate) fn ecra_e_inverso(&self, p: [f32; 3]) -> Option<([f32; 2], f32)> {
        let m = &self.local_para_clip;
        let w = m[3] * p[0] + m[7] * p[1] + m[11] * p[2] + m[15];
        if w <= 0.0 {
            return None;
        }
        Some((self.ecra(p)?, 1.0 / w))
    }
}

/// Um rectângulo da tela, `[x, y, largura, altura]` em píxeis.
pub type Rectangulo = [u32; 4];

/// ⭐⭐ **O estado de UMA pincelada do Painter sobre a peça** — construído no
/// pen-down, sobre a forma e a câmera desse instante.
pub struct TelaNaMalha {
    vista: Vista,
    ecra: Vec<Option<[f32; 2]>>,
    colunas: usize,
    linhas: usize,
    celulas: Vec<Vec<u32>>,
    /// Por amostra (ou por vértice, sem plano): `0` por decidir · `1` vê-se ·
    /// `2` escondida.
    visivel: Vec<u8>,
    /// ⭐ **A oclusão decidida por (face, PÍXEL)** — cache de mapeamento
    /// directo, uma entrada por píxel da vista: `face + 1` nos 31 bits de
    /// baixo, o veredito no bit de cima; `0` = vazia. Ver [`Self::ve_se_no_pixel`].
    visivel_no_pixel: Vec<u32>,
    carimbo_face: Vec<u32>,
    carimbo_amostra: Vec<u32>,
    epoca: u32,
    raios: usize,
    projetadas: usize,
    /// ⭐⭐ **O retrato da peça com que a tela do Painter COMEÇOU** — só nos
    /// modos que lêem a cor debaixo do pincel ([`crate::tela_semente`]). Com
    /// ele a lei deixa de ser o «over» e passa a ser a DIFERENÇA.
    semente: Option<Vec<u8>>,
}

impl TelaNaMalha {
    /// Congela a vista sobre `mesh`. `amostras` é o tamanho do destino — as
    /// amostras do plano de tinta fina, ou os vértices sem ele.
    #[must_use]
    pub fn nova(mesh: &Mesh, vista: Vista, amostras: usize) -> Self {
        let ecra: Vec<Option<[f32; 2]>> = mesh.positions().iter().map(|&p| vista.ecra(p)).collect();
        let colunas = (vista.largura / CELULA).ceil().max(1.0) as usize;
        let linhas = (vista.altura / CELULA).ceil().max(1.0) as usize;
        let mut celulas = vec![Vec::new(); colunas * linhas];
        let pos = mesh.positions();
        for (fi, face) in mesh.faces().iter().enumerate() {
            let cantos = face.verts();
            if !de_frente(pos, cantos, vista.olho) {
                continue;
            }
            let Some(caixa) = caixa(&ecra, cantos) else {
                continue;
            };
            if caixa[2] < 0.0
                || caixa[3] < 0.0
                || caixa[0] > vista.largura
                || caixa[1] > vista.altura
            {
                continue;
            }
            let c0 = (caixa[0].max(0.0) / CELULA) as usize;
            let l0 = (caixa[1].max(0.0) / CELULA) as usize;
            let c1 = ((caixa[2] / CELULA) as usize).min(colunas - 1);
            let l1 = ((caixa[3] / CELULA) as usize).min(linhas - 1);
            for l in l0..=l1 {
                for c in c0..=c1 {
                    celulas[l * colunas + c].push(fi as u32);
                }
            }
        }
        Self {
            vista,
            ecra,
            colunas,
            linhas,
            celulas,
            visivel: vec![0; amostras],
            visivel_no_pixel: vec![
                0;
                (vista.largura.max(1.0) as usize)
                    * (vista.altura.max(1.0) as usize)
            ],
            carimbo_face: vec![0; mesh.faces().len()],
            carimbo_amostra: vec![0; amostras],
            epoca: 0,
            raios: 0,
            projetadas: 0,
            semente: None,
        }
    }

    /// A vista em que o traço foi congelado.
    #[must_use]
    pub fn vista(&self) -> &Vista {
        &self.vista
    }

    /// ⭐⭐ **A tela começou com este retrato da peça** — daqui em diante cada
    /// amostra recebe a DIFERENÇA entre o que a tela ficou e o que ela era
    /// (ver o cabeçalho). ⚠️ Tem de ser os MESMOS bytes com que a tela do
    /// Painter foi semeada, senão o que o pincel não tocou deixa de se anular.
    pub fn com_semente(&mut self, rgba: Vec<u8>) {
        self.semente = Some(rgba);
    }

    /// Há retrato? — o modo em que a lei é a diferença.
    #[must_use]
    pub fn tem_semente(&self) -> bool {
        self.semente.is_some()
    }

    /// ⭐ **O que a tela pede a um ponto** — e se isso é «nada a fazer».
    fn leitura(&self, tela: &Tela<'_>, s: [f32; 2]) -> (Mistura, bool) {
        let (pm, a) = tela.amostra(s[0], s[1]);
        let Some(sem) = self.semente.as_deref() else {
            return (Mistura::Sobre { pm, a }, a <= 0.0);
        };
        let retrato = Tela {
            rgba: sem,
            largura: tela.largura,
            altura: tela.altura,
        };
        let (spm, sa) = retrato.amostra(s[0], s[1]);
        // ⚠️ Sem cobertura de um dos lados não há cor a comparar: um píxel
        // APAGADO (a borracha) e o fundo fora da silhueta não mexem na peça.
        if a <= COBERTURA_MINIMA || sa <= COBERTURA_MINIMA {
            return (Mistura::Diferenca([0.0; 3]), true);
        }
        let d = [
            pm[0] / a - spm[0] / sa,
            pm[1] / a - spm[1] / sa,
            pm[2] / a - spm[2] / sa,
        ];
        (Mistura::Diferenca(d), d == [0.0; 3])
    }

    /// Quantos raios de oclusão este traço já lançou — sonda de custo.
    #[must_use]
    pub fn raios(&self) -> usize {
        self.raios
    }

    /// Quantas amostras do plano este traço já projectou no ecrã — sonda de
    /// custo: com os blocos da retícula ela segue a PEGADA, não a face.
    #[must_use]
    pub fn projetadas(&self) -> usize {
        self.projetadas
    }

    fn proxima_epoca(&mut self) {
        self.epoca = self.epoca.wrapping_add(1);
        if self.epoca == 0 {
            self.epoca = 1;
            self.carimbo_face.fill(0);
            self.carimbo_amostra.fill(0);
        }
    }

    /// As faces de frente cujas células tocam `r`, cada uma uma vez.
    fn faces_em(&mut self, r: [f32; 4]) -> Vec<u32> {
        let c0 = (r[0].max(0.0) / CELULA) as usize;
        let l0 = (r[1].max(0.0) / CELULA) as usize;
        let c1 = ((r[2].max(0.0) / CELULA) as usize).min(self.colunas - 1);
        let l1 = ((r[3].max(0.0) / CELULA) as usize).min(self.linhas - 1);
        let mut out = Vec::new();
        for l in l0..=l1 {
            for c in c0..=c1 {
                for &f in &self.celulas[l * self.colunas + c] {
                    if self.carimbo_face[f as usize] != self.epoca {
                        self.carimbo_face[f as usize] = self.epoca;
                        out.push(f);
                    }
                }
            }
        }
        out
    }

    /// A amostra já foi vista NESTA pousada? (marca-a).
    fn repetida(&mut self, idx: u32) -> bool {
        let c = &mut self.carimbo_amostra[idx as usize];
        if *c == self.epoca {
            return true;
        }
        *c = self.epoca;
        false
    }

    fn ve_se(&mut self, mesh: &Mesh, idx: u32, p: [f32; 3]) -> bool {
        let v = &mut self.visivel[idx as usize];
        if *v == 0 {
            self.raios += 1;
            *v = if desimpedida(mesh, self.vista.olho, p) {
                1
            } else {
                2
            };
        }
        *v == 1
    }

    /// ⭐⭐ **A mesma pergunta, decidida UMA vez por (face, píxel do ecrã).**
    ///
    /// A `256x` a peça de fábrica põe `~70` amostras em cada píxel da vista, e
    /// cada uma pagava o seu raio de oclusão — medido, a 1.ª drenagem de um
    /// traço custava `232,7 ms`, quase tudo raios. Duas amostras da MESMA face
    /// que caem no MESMO píxel são, para o que o artista vê, o mesmo ponto: um
    /// oclusor que tapa uma e não a outra tem a borda DENTRO de um píxel, onde
    /// a tela do Painter já não distingue as duas.
    ///
    /// ⚠️ A chave inclui a FACE: dois lados de uma dobra (duas faces de frente
    /// no mesmo píxel, uma à frente da outra) têm vereditos opostos, e é isso
    /// que a lei da oclusão existe para separar. Uma colisão de face no mesmo
    /// píxel só custa um raio, nunca um veredito errado.
    fn ve_se_no_pixel(
        &mut self,
        mesh: &Mesh,
        face: u32,
        s: [f32; 2],
        idx: u32,
        p: [f32; 3],
    ) -> bool {
        let v = self.visivel[idx as usize];
        if v != 0 {
            return v == 1;
        }
        let w = self.vista.largura.max(1.0) as usize;
        let h = self.vista.altura.max(1.0) as usize;
        let (x, y) = (s[0].floor(), s[1].floor());
        let chave = face.wrapping_add(1) & !VEREDITO;
        let slot = (x >= 0.0 && y >= 0.0 && (x as usize) < w && (y as usize) < h)
            .then(|| y as usize * w + x as usize);
        if let Some(i) = slot {
            let c = self.visivel_no_pixel[i];
            if c & !VEREDITO == chave {
                let ve = c & VEREDITO != 0;
                self.visivel[idx as usize] = if ve { 1 } else { 2 };
                return ve;
            }
        }
        let ve = self.ve_se(mesh, idx, p);
        if let Some(i) = slot {
            self.visivel_no_pixel[i] = chave | if ve { VEREDITO } else { 0 };
        }
        ve
    }
}

/// O bit do veredito na cache por píxel ([`TelaNaMalha::ve_se_no_pixel`]).
const VEREDITO: u32 = 1 << 31;

pub(crate) fn de_frente(pos: &[[f32; 3]], cantos: &[u32], olho: [f32; 3]) -> bool {
    let p = |k: usize| pos[cantos[k] as usize];
    let n = if cantos.len() == 3 {
        cruz(sub(p(1), p(0)), sub(p(2), p(0)))
    } else {
        cruz(sub(p(2), p(0)), sub(p(3), p(1)))
    };
    ponto(n, sub(olho, p(0))) > 0.0
}

fn caixa(ecra: &[Option<[f32; 2]>], cantos: &[u32]) -> Option<[f32; 4]> {
    let mut b = [
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    ];
    for &v in cantos {
        let s = ecra[v as usize]?;
        b = [
            b[0].min(s[0]),
            b[1].min(s[1]),
            b[2].max(s[0]),
            b[3].max(s[1]),
        ];
    }
    Some(b)
}

/// O raio do olho até `p` chega a `p` antes de bater noutra superfície?
fn desimpedida(mesh: &Mesh, olho: [f32; 3], p: [f32; 3]) -> bool {
    let d = sub(p, olho);
    let dist = ponto(d, d).sqrt();
    if dist <= 0.0 {
        return true;
    }
    mesh.raycast(&Ray::new(olho, d))
        .is_none_or(|h| h.t >= dist * (1.0 - FOLGA_DA_OCLUSAO))
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cruz(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn ponto(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn combina(pos: &[[f32; 3]], cantos: &[u32], w: &[f32]) -> [f32; 3] {
    let mut o = [0.0f32; 3];
    for (&v, &k) in cantos.iter().zip(w) {
        let q = pos[v as usize];
        for e in 0..3 {
            o[e] += q[e] * k;
        }
    }
    o
}

/// Abaixo desta cobertura um lado não tem cor que se leia (`1/255` é um
/// byte de alfa, e um byte de alfa não carrega uma cor de 8 bits).
const COBERTURA_MINIMA: f32 = 0.5 / 255.0;

/// O que uma amostra recebe da tela.
#[derive(Clone, Copy)]
enum Mistura {
    /// A tela TRANSPARENTE: a camada do traço por cima da base.
    Sobre { pm: [f32; 3], a: f32 },
    /// A tela SEMEADA: a diferença entre o que ela ficou e o retrato.
    Diferenca([f32; 3]),
}

// ⭐ O que POUSA a tela na peça (a lei por amostra e o percurso da retícula)
// vive num filho: este ficheiro é a VISTA e a OCLUSÃO, aquele o DEPÓSITO.
#[path = "tela_na_malha_pousa.rs"]
mod pousa;
