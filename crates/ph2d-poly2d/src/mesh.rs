//! **A PORTA ÚNICA** — de uma grelha de alfa à malha que a deformação move.

/// ⭐⭐⭐ **A MALHA DE UMA IMAGEM** — as posições de repouso e os triângulos que as ligam.
///
/// ⚠️⚠️ **A UV NÃO É GUARDADA, ela DERIVA-SE** ([`Mesh2d::uv`]) — a coordenada de textura de um
/// vértice de repouso é a posição dele sobre o tamanho da imagem, e mais nada. Guardá-la seria o
/// *vector paralelo* que o esqueleto já proíbe por escrito nos pesos: duas listas indexadas em
/// paralelo que uma edição pode dessincronizar, para exprimir uma divisão.
///
/// ⇒ o que viaja é o **tamanho** (um par), não uma lista.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mesh2d {
    /// Os vértices em **pixels da imagem** no repouso — a pose em que a malha foi presa.
    pub rest: Vec<[f64; 2]>,
    /// Triplas de índices em [`Mesh2d::rest`].
    pub tris: Vec<[u32; 3]>,
    /// O tamanho da imagem no bind, em pixels. É o denominador da UV.
    pub size: [u32; 2],
}

impl Mesh2d {
    /// A coordenada de textura do vértice `i`, em `0..1`.
    ///
    /// `None` quando o índice não existe ou a imagem tem lado zero — ⛔ nunca um `0.0` calado, que
    /// se leria como *«o canto superior esquerdo»* e mandaria o triângulo buscar o pixel errado.
    #[must_use]
    pub fn uv(&self, i: usize) -> Option<[f64; 2]> {
        let p = self.rest.get(i)?;
        let (w, h) = (f64::from(self.size[0]), f64::from(self.size[1]));
        (w > 0.0 && h > 0.0).then(|| [p[0] / w, p[1] / h])
    }

    /// Quantos triângulos — o número que o artista paga por quadro.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.tris.len()
    }
}

/// ⭐ **O ÚNICO número que o artista vê**, e o limiar do que conta como tinta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshOptions {
    /// **Densidade**, em pixels: o quanto a silhueta pode desviar-se da forma real. Maior = menos
    /// vértices. ⚠️ Ela é uma TOLERÂNCIA e não uma contagem, e é de propósito: uma contagem fixa
    /// dá a uma forma simples mais vértices do que ela precisa e a uma complicada menos.
    pub tolerance: f64,
    /// A partir de que alfa um pixel conta como tinta.
    ///
    /// ⚠️ **`1`, e não `128`.** Uma borda suavizada tem alfa a subir de `0` a `255` ao longo de
    /// dois ou três pixels; cortar a meio come essa borda e a silhueta fica **por dentro** do
    /// desenho — o artista vê o contorno do próprio desenho a ser aparado ao deformar.
    pub alpha_threshold: u8,
}

impl Default for MeshOptions {
    fn default() -> Self {
        Self {
            // O valor de nascimento: `1,5 px` de desvio. ⚠️ Ele NÃO é medido contra um recurso —
            // é o ponto de partida do artista, e o smoke é quem o julga. O que está medido é a
            // FORMA da resposta (a tabela no gate `the_density_knob_is_the_only_number`).
            tolerance: 1.5,
            alpha_threshold: 1,
        }
    }
}

/// ⭐⭐⭐ **A PORTA** — da cobertura de uma imagem à malha, numa chamada.
///
/// `alpha` é uma amostra por pixel em ordem de leitura. Devolve `None` quando não há tinta
/// suficiente para um triângulo — ⛔ nunca uma malha vazia, que a jusante se leria como *«a
/// imagem não se move»* em vez de *«não havia o que prender»*.
///
/// ⚠️ **Todas as ilhas entram.** Um desenho com duas peças soltas (dois olhos, uma corrente) dá
/// duas sub-malhas na mesma `Mesh2d`, com os índices já deslocados — é a leitura certa de *«esta
/// imagem»*, e tratá-la como uma peça só obrigaria o artista a cortar o ficheiro.
#[must_use]
pub fn mesh_of(alpha: &[u8], width: u32, height: u32, opts: MeshOptions) -> Option<Mesh2d> {
    let (w, h) = (width as usize, height as usize);
    // ⛔ A recusa do rastreio propaga-se: uma silhueta que não fecha não vira malha nenhuma.
    let aneis = crate::contour(alpha, w, h, opts.alpha_threshold)?;
    let mut rest: Vec<[f64; 2]> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    for anel in &aneis {
        let simples = crate::simplify(anel, opts.tolerance);
        if simples.len() < 3 {
            continue;
        }
        let base = u32::try_from(rest.len()).ok()?;
        // ⛔ E a do ear-clipping também: um anel que não triangula não entra pela metade.
        let locais = crate::triangulate(&simples)?;
        if locais.is_empty() {
            continue;
        }
        rest.extend_from_slice(&simples);
        tris.extend(
            locais
                .iter()
                .map(|t| [t[0] + base, t[1] + base, t[2] + base]),
        );
    }
    (!tris.is_empty()).then_some(Mesh2d {
        rest,
        tris,
        size: [width, height],
    })
}
