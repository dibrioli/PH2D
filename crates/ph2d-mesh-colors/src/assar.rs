//! ⭐⭐⭐⭐ **O ASSADO** — a retícula vira uma TEXTURA com UV, para sair daqui.
//!
//! A [`crate::Tinta`] é onde a tinta MORA; isto é como ela **viaja para outro
//! programa**. O consumidor é qualquer motor: um `.obj` com `vt`, um `.mtl` e
//! um `.png`.
//!
//! # ⭐⭐⭐ Porque não há solver de UV nenhum aqui, e isso é um TEOREMA
//!
//! O [doc 27 §5](../../../docs/3D/27_o_estado_da_arte_de_onde_a_tinta_mora.md)
//! mede a razão: um atlas achata um pedaço **CURVO** da superfície e paga
//! distorção de área por Egregium; esta família achata **uma face plana de cada
//! vez**, e um mapa afim num triângulo preserva a razão de áreas — distorção
//! **zero, por construção**.
//!
//! ⇒ *o parametrizador é a disposição*: cada face recebe um **LADRILHO** seu, e
//! o endereço `(face, i, j, k)` que a [`crate::indice`] já resolve é o texel.
//! ⛔ **E é por isso que isto NÃO exige a retopologia**: não há desenrolamento
//! para correr, logo uma escultura crua sai tão bem como um quad limpo.
//!
//! # ⚠️ O que o assado PERDE, e é o formato que o obriga
//!
//! A fronteira de uma face é **PARTILHADA** na retícula (é a diferença de
//! espécie para o Ptex) e uma textura não sabe partilhar: cada ladrilho leva a
//! **cópia** da borda dele. ⭐ Os dois lados escrevem o MESMO valor — a amostra
//! é uma só —, logo a costura é invisível em cor e o que ela custa é
//! **filtragem**, que é o que a [`FOLGA_EM_TEXELS`] paga.
//!
//! ⚠️⚠️ **E ele NÃO iguala densidades entre faces, de propósito:** o ladrilho
//! tem o tamanho do `lado` da retícula, que hoje é UM para a peça toda, logo
//! uma face grande e uma pequena recebem o mesmo número de texels. *Isso não é
//! distorção do assado — é a retícula que ele copia fielmente*, e curá-la é a
//! **P2** do plano (o `R` por face). Medido no corpus do dono, a dispersão da
//! densidade é `3,1×` a `18,3×`.

use crate::{Tinta, cantos};

/// ⭐ **A FOLGA à volta de cada ladrilho, em texels**, preenchida por
/// dilatação.
///
/// ⚠️ **O recurso tem nome: a FILTRAGEM.** Um texel de folga é o que a
/// interpolação bilinear alcança para fora da borda; o segundo é o que
/// sobrevive a **um** nível de mip. ⛔ Ele **não** é o [`crate::topo::TRI`] do
/// atlas (`VAO_EM_TEXELS = 8`, que compra três níveis) — ali o vão separa
/// ILHAS de uma peça inteira e aqui separa faces vizinhas cuja borda tem o
/// mesmo valor dos dois lados. *Uma cadeia de mips profunda sobre este assado
/// mistura faces vizinhas, e isso está DECLARADO.*
pub const FOLGA_EM_TEXELS: u32 = 2;

/// Porque é que uma peça não assou.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recusa {
    /// Uma peça sem faces não tem onde pôr um ladrilho.
    SemFaces,
    /// ⛔ **A textura que esta peça pede não cabe no tecto do chamador.**
    ///
    /// ⚠️ O tecto **não é escolhido aqui** — ele é do consumidor (o lado
    /// máximo que uma placa amostra, ou o que um ficheiro deve pesar), e quem
    /// o sabe é quem chama. *Um limite escrito nesta crate seria um palpite
    /// sobre o programa de outra pessoa.*
    NaoCabe {
        /// O lado que a peça precisava, em texels.
        preciso: u32,
        /// O lado que o chamador permite.
        tecto: u32,
    },
    /// ⛔⛔⛔ **O plano não descreve esta malha.**
    ///
    /// ⚠️⚠️ **É a lei que esta casa pagou com um `panic` na cara do dono** (o
    /// report de 21/09): a [`crate::Topologia`] guarda `4` entradas por face,
    /// logo uma lista de faces **mais longa** do que a que a construiu indexa
    /// fora de alcance — `index out of bounds` no meio de uma exportação.
    /// ⛔ E a metade **CURTA** é a pior: com menos faces nada sai de alcance, o
    /// laço acaba sozinho, e a textura fica com tinta **válida no sítio
    /// errado**, em silêncio.
    ///
    /// ⭐ A cura é a mesma porta que o `payload` já usa
    /// ([`crate::Topologia::descreve`]) — *um assado nasceu uma wave depois
    /// daquela cura e sem nenhuma das duas metades dela*.
    NaoDescreve {
        /// Quantas faces a malha que chegou tem.
        faces: usize,
        /// Quantas o plano conhece.
        plano: usize,
    },
}

impl std::fmt::Display for Recusa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SemFaces => write!(f, "a peca nao tem faces"),
            Self::NaoCabe { preciso, tecto } => write!(
                f,
                "a textura pedia {preciso}x{preciso} texels e o tecto e' {tecto}"
            ),
            Self::NaoDescreve { faces, plano } => write!(
                f,
                "o plano de tinta fina conhece {plano} faces e a malha tem {faces}"
            ),
        }
    }
}

/// O que o assado mediu de si próprio.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Relatorio {
    /// Faces assadas — uma por ladrilho.
    pub faces: usize,
    /// O lado de um ladrilho **com** a folga, em texels.
    pub ladrilho: u32,
    /// Texels que receberam uma amostra.
    pub com_amostra: usize,
    /// Texels da textura inteira.
    pub texels: usize,
}

impl Relatorio {
    /// A fracção da textura que carrega uma amostra de verdade.
    ///
    /// ⚠️ **O resto NÃO é lixo** — é folga e é a metade vazia do ladrilho de um
    /// triângulo, e as duas são preenchidas por dilatação. *Uma coluna de
    /// «aproveitamento» que se lê como desperdício é a régua do
    /// [doc 26 §10.1](../../../docs/3D/26_a_parametrizacao_como_atlas.md) outra
    /// vez.*
    #[must_use]
    pub fn aproveitamento(&self) -> f32 {
        if self.texels == 0 {
            return 0.0;
        }
        self.com_amostra as f32 / self.texels as f32
    }
}

/// A textura e as coordenadas que saem de uma peça.
#[derive(Debug, Clone, PartialEq)]
pub struct Assado {
    /// O lado da textura, em texels. ⚠️ **Não é potência de dois** — ver
    /// [`assar`].
    pub lado_px: u32,
    /// `lado_px² × 4` bytes, RGBA, linha `0` **em cima** (a convenção do PNG).
    pub rgba: Vec<u8>,
    /// As coordenadas de cada CANTO de cada face, na ordem do percurso da face
    /// e das faces. ⭐ Já em convenção de ficheiro (`v` conta de BAIXO) — ver
    /// [`assar`], onde a inversão vive **uma vez**.
    pub uv: Vec<[f32; 2]>,
    /// `off_uv[f]..off_uv[f + 1]` são os cantos da face `f` dentro de
    /// [`Self::uv`].
    pub off_uv: Vec<u32>,
    /// Ver [`Relatorio`].
    pub relatorio: Relatorio,
}

impl Assado {
    /// ⭐⭐ **Os píxeis como um FICHEIRO os quer: TRÊS canais.**
    ///
    /// ⚠️⚠️ **O alfa do [`Self::rgba`] não é transparência — é a COBERTURA**, a
    /// marca de quem recebeu uma amostra ou uma dilatação, e é por ela que os
    /// gates sabem distinguir um texel preto de um texel vazio. *Escrevê-la num
    /// `.png` entrega ao destino uma grandeza interna com cara de alfa:* medido
    /// com o alvo a correr, o Blender 5.2.2 lê o canal, põe o material em
    /// `blend_method: HASHED` e passa a tratar a peça como translúcida — ⛔ e
    /// isso não é uma decisão que uma textura de albedo deva tomar por ninguém.
    ///
    /// ⭐ *A cobertura fica onde ela é medida; o ficheiro leva a cor.*
    #[must_use]
    pub fn rgb(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.rgba.len() / 4 * 3);
        for p in self.rgba.as_chunks::<4>().0 {
            out.extend_from_slice(&p[..3]);
        }
        out
    }
}

/// A cor de uma amostra em bytes — **a mesma conta que o `write_ply` faz**.
///
/// ⚠️ O `clamp` vem ANTES do cast pela razão que aquele escreve: um `as u8`
/// satura, e uma cor fora de `[0,1]` é HDR num canal de 8 bits, que aqui vira
/// um número honesto em vez de um wrap.
fn byte(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// ⭐⭐⭐ **ASSA a retícula de uma peça numa textura quadrada.**
///
/// `tecto_px` é o maior lado que o chamador aceita — ver [`Recusa::NaoCabe`].
///
/// ⚠️ **O lado NÃO é arredondado a potência de dois.** Arredondar dobraria a
/// memória no pior caso para agradar a uma cadeia de mips que a
/// [`FOLGA_EM_TEXELS`] já declara não suportar em profundidade. *Um ficheiro
/// que ninguém amostra não paga o alinhamento de quem amostra.*
///
/// ⚠️⚠️ **A inversão do eixo `v` vive AQUI e só aqui.** Uma textura conta as
/// linhas de cima para baixo e um `.obj` conta `v` de baixo para cima; escrever
/// a inversão no escritor do ficheiro poria a mesma lei em dois sítios, e o dia
/// em que nascesse o segundo formato ela viajaria para um só.
pub fn assar<'a>(
    tinta: &Tinta,
    faces: impl Iterator<Item = &'a [u32]>,
    tecto_px: u32,
) -> Result<Assado, Recusa> {
    let faces: Vec<&[u32]> = faces.collect();
    if faces.is_empty() {
        return Err(Recusa::SemFaces);
    }
    // ⛔⛔⛔ **A GUARDA, e ela vem ANTES de qualquer indexação.** Ver
    //    [`Recusa::NaoDescreve`]: sem ela uma malha com mais faces do que o
    //    plano conhece estoura no meio de uma exportação, e uma com menos sai
    //    com tinta válida no sítio errado.
    //
    // ⚠️ A pergunta é a do `payload` e não uma cerca nova — *uma lei escrita em
    //    dois sítios ainda não é uma lei; só uma PORTA é.*
    if !tinta
        .topologia()
        .descreve(tinta.plano_por_vertice().len(), faces.len())
    {
        return Err(Recusa::NaoDescreve {
            faces: faces.len(),
            plano: tinta.topologia().faces(),
        });
    }
    let l = tinta.lado();
    let ladrilho = l + 1 + 2 * FOLGA_EM_TEXELS;
    // ⭐ `cols` é o lado de uma grelha quadrada que cabe as faces todas, e as
    // linhas nunca passam as colunas porque `rows = ceil(n / cols) <= cols`
    // quando `cols = ceil(sqrt(n))`. *É isso que torna a textura quadrada sem
    // uma segunda conta.*
    let cols = (faces.len() as f64).sqrt().ceil() as u32;
    let lado_px = cols * ladrilho;
    if lado_px > tecto_px {
        return Err(Recusa::NaoCabe {
            preciso: lado_px,
            tecto: tecto_px,
        });
    }
    let texels = (lado_px as usize) * (lado_px as usize);
    let mut rgba = vec![0u8; texels * 4];
    let mut coberto = vec![false; texels];
    let mut uv = Vec::with_capacity(faces.len() * 4);
    let mut off_uv = Vec::with_capacity(faces.len() + 1);
    off_uv.push(0);

    let pouse = |rgba: &mut Vec<u8>, coberto: &mut Vec<bool>, x: u32, y: u32, c: [f32; 3]| {
        let p = (y as usize) * (lado_px as usize) + (x as usize);
        rgba[p * 4] = byte(c[0]);
        rgba[p * 4 + 1] = byte(c[1]);
        rgba[p * 4 + 2] = byte(c[2]);
        rgba[p * 4 + 3] = 255;
        coberto[p] = true;
    };

    for (f, face) in faces.iter().enumerate() {
        let n = cantos(face);
        let ox = (f as u32 % cols) * ladrilho + FOLGA_EM_TEXELS;
        let oy = (f as u32 / cols) * ladrilho + FOLGA_EM_TEXELS;
        // ⚠️⚠️ **O CANTO de um triângulo é `(i = L)`, `(j = L)`, `(k = L)`** —
        // a ordem que a [`crate::sitio_tri`] declara. A disposição no ladrilho
        // é `(u, v) = (j, k)`, logo o canto `0` cai na origem, o `1` em `(L, 0)`
        // e o `2` em `(0, L)`. ⛔ *Trocar dois deles espelha meia peça e nada
        // no ecrã acusa*, que é o aviso que o cabeçalho da [`crate::topo`] já
        // escreve para as arestas.
        if n == 3 {
            for k in 0..=l {
                for j in 0..=(l - k) {
                    let i = l - j - k;
                    let idx = tinta.indice_tri(f, face, i, j, k) as usize;
                    pouse(
                        &mut rgba,
                        &mut coberto,
                        ox + j,
                        oy + k,
                        tinta.amostras()[idx],
                    );
                }
            }
            for (u, v) in [(0, 0), (l, 0), (0, l)] {
                uv.push(coord(ox + u, oy + v, lado_px));
            }
        } else {
            for j in 0..=l {
                for i in 0..=l {
                    let idx = tinta.indice_quad(f, face, i, j) as usize;
                    pouse(
                        &mut rgba,
                        &mut coberto,
                        ox + i,
                        oy + j,
                        tinta.amostras()[idx],
                    );
                }
            }
            for (u, v) in [(0, 0), (l, 0), (l, l), (0, l)] {
                uv.push(coord(ox + u, oy + v, lado_px));
            }
        }
        off_uv.push(uv.len() as u32);
    }

    let com_amostra = coberto.iter().filter(|c| **c).count();
    dilata(&mut rgba, &mut coberto, lado_px, FOLGA_EM_TEXELS);
    Ok(Assado {
        lado_px,
        rgba,
        uv,
        off_uv,
        relatorio: Relatorio {
            faces: faces.len(),
            ladrilho,
            com_amostra,
            texels,
        },
    })
}

/// O centro do texel `(x, y)`, em UV de ficheiro.
fn coord(x: u32, y: u32, lado_px: u32) -> [f32; 2] {
    let s = lado_px as f32;
    [
        (x as f32 + 0.5) / s,
        // ⚠️ A inversão do eixo — ver [`assar`].
        1.0 - (y as f32 + 0.5) / s,
    ]
}

/// ⭐⭐ **A DILATAÇÃO** — dá cor aos texels que nenhuma amostra pousou.
///
/// ⛔⛔ **Ela não é acabamento: sem ela o assado tem um defeito visível na
/// primeira olhada.** Um `uv` sobre a borda de uma face cai ENTRE dois centros
/// de texel, logo a interpolação bilinear lê um bloco `2×2` que inclui um texel
/// de fora — e a metade vazia do ladrilho de um triângulo está *dentro* desse
/// bloco ao longo da hipotenusa. Sem dilatação, a borda de toda face sai com
/// uma linha escura.
///
/// ⚠️ **A média dos vizinhos COBERTOS e não o primeiro que aparece:** com o
/// primeiro, o resultado depende da ordem em que os oito são visitados, e dois
/// texels simétricos da mesma borda ficam de cores diferentes.
fn dilata(rgba: &mut [u8], coberto: &mut [bool], lado_px: u32, passos: u32) {
    let w = lado_px as i64;
    for _ in 0..passos {
        let antes = coberto.to_vec();
        for y in 0..w {
            for x in 0..w {
                let p = (y * w + x) as usize;
                if antes[p] {
                    continue;
                }
                let (mut soma, mut n) = ([0u32; 4], 0u32);
                for dy in -1..=1i64 {
                    for dx in -1..=1i64 {
                        let (qx, qy) = (x + dx, y + dy);
                        if qx < 0 || qy < 0 || qx >= w || qy >= w {
                            continue;
                        }
                        let q = (qy * w + qx) as usize;
                        if !antes[q] {
                            continue;
                        }
                        for c in 0..4 {
                            soma[c] += u32::from(rgba[q * 4 + c]);
                        }
                        n += 1;
                    }
                }
                if n == 0 {
                    continue;
                }
                for c in 0..4 {
                    rgba[p * 4 + c] = (soma[c] / n) as u8;
                }
                coberto[p] = true;
            }
        }
    }
}
