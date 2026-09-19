//! ⭐⭐⭐ **O PESO À VISTA** — irmão do [`super::limit`] pelo tecto de LOC, cortado por assunto.
//!
//! Enquanto o pincel de peso está na mão, cada ponto da arte presa é um **ponto colorido pela
//! influência do osso em foco**: frio onde ele não manda, quente onde ele manda sozinho.
//!
//! ⛔⛔ **Ele existe porque o pincel era CEGO.** Corrigir um peso sem o ver é apontar para um número
//! que não está na tela — *o artista via a arte a dobrar mal e não tinha como saber qual osso a
//! puxava, nem quanto*. É a mesma razão do indicador da pose e do do contorno: aqueles mostram a
//! REGIÃO de um gesto, este mostra o VALOR que o gesto edita.
//!
//! ⚠️ **O RAIO do ponto é em píxeis de ecrã e a posição é em MUNDO**, que é a gramática desta crate
//! inteira (o cabeçalho do [`super`] escreve-a): *o ponto sobe pelo afim, a espessura não*. Com o
//! raio em mundo, afastar o zoom colaria os pontos numa mancha e aproximar fá-los-ia desaparecer.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, Brush, Circle, Color as VelloColor, Fill, Point, Stroke, VectorScene};

/// O raio de cada ponto, em píxeis de ecrã.
///
/// ⚠️ **Ele tem de deixar ver a ARTE por baixo** — um ponto que a tapa responde *«qual é o peso
/// aqui»* e apaga *«o que está aqui»*, e o artista precisa dos dois ao mesmo tempo para saber se a
/// correcção caiu no sítio certo.
///
/// ⛔⛔ **Mas `2,5` era pequeno demais para a cor ser LEGÍVEL, e o número que o diz é o vão entre
/// dois nós desenhados** (report do dono, 2026-09-19: *«nada fica vermelho e nada fica azul»*):
///
/// | grandeza | píxeis |
/// |---|---|
/// | dois nós vizinhos da barra da cena | `70,7` |
/// | o ponto de então (raio) | `2,5` — **`3,5 %`** do vão |
/// | o ponto de hoje (raio) | `5,0` — `14 %` do vão, `60,7` px de folga entre vizinhos |
///
/// ⚠️ **E o diâmetro de `10` px não é escolhido:** é a família das alças de gradiente desta casa
/// (`~9` px), que é o tamanho a que um ponto de UI deste app já é apontável e legível.
pub const WEIGHT_DOT_R_PX: f64 = 5.0; // LITERAL-PX-OK: raio de ecrã, tabela medida acima

/// A opacidade dos pontos.
///
/// ⚠️ **Eles são uma LEITURA sobre o desenho e não parte dele** — opacos, um rig denso viraria uma
/// segunda arte por cima da primeira.
const WEIGHT_DOT_ALPHA: f32 = 0.85;

/// **Os pontos da pele, pintados pelo peso** — `(mundo, peso 0..1)`.
///
/// ⭐⭐ **A rampa é a de PESO da indústria** — ver [`Rampa`]. ⛔ Ela **não** é feita de tokens do
/// tema, e a razão está medida lá: um token semântico é escolhido para ser CALMO no chrome, e uma
/// leitura de valor é escolhida para ser DISTINGUÍVEL — são requisitos opostos.
///
/// ⛔ **Um peso de `0` é pintado, e é a metade que importa.** Sem ele o artista vê os pontos que o
/// osso já governa e **não vê onde ele devia governar e não governa** — que é exactamente o defeito
/// que ele veio corrigir. *Desenhar só o que já está certo é desenhar o problema invisível.*
///
/// ⚠️ **O `theme` fica na assinatura e não é lido pela COR** — ele é a assinatura de família desta
/// crate (todo `draw_*` a recebe), e tirá-lo daqui faria deste o único passe com outra forma. A cor
/// de um número não muda com o tema, e isso é a lei da [`Rampa`].
pub fn draw_weights(
    pontos: &[([f64; 2], f64)],
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let _ = theme;
    if pontos.is_empty() {
        return;
    }
    // ⚠️ **Construída UMA vez por passe e não por ponto** — ela resolve cinco conversões de cor e
    // quatro distâncias; por ponto isso seria `925 ×` o trabalho na arte medida do smoke.
    let rampa = Rampa::nova();
    for &(p, w) in pontos {
        if !(p[0].is_finite() && p[1].is_finite() && w.is_finite()) {
            continue;
        }
        let c = rampa.cor(w);
        let centro = transform * Point::new(p[0], p[1]);
        target.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(c.multiply_alpha(WEIGHT_DOT_ALPHA)),
            None,
            &Circle::new(centro, WEIGHT_DOT_R_PX),
        );
    }
}

/// **O ANEL do pincel** — onde ele vai pintar, e com que tamanho.
///
/// ⛔⛔ **O raio deste era em MUNDO, com um argumento escrito aqui, e a medição refutou-o**
/// (2026-09-19): *«o raio do pincel É uma distância do desenho, logo aproximar o zoom tem de o
/// mostrar maior»*. Verdade sobre o que a MANCHA guarda e falso sobre o que o ARTISTA escolhe — e o
/// preço da confusão foi um valor de fábrica de `2 000` px (`2,85 ×` a peça inteira), porque **um
/// número de mundo não pode ter um valor de fábrica**: ele teria de saber a escala da cena.
///
/// ⇒ o artista escolhe **píxeis de ecrã** ([`ph2d_tool_vector::WEIGHT_RADIUS_DEFAULT`], com a
/// tabela medida), quem chama converte a mundo com o factor do pick, e este anel desenha o que o
/// artista escolheu, sem escala nenhuma. ⚠️ **A propriedade declarada que isso traz:** a mesma
/// posição do slider pinta uma área de MUNDO diferente em dois zooms — o que é precisamente o que
/// faz aproximar-se para corrigir um sítio fino funcionar, e é o que toda ferramenta de pintura
/// desta casa já faz.
pub fn draw_weight_brush(
    centro: Option<[f64; 2]>,
    raio_ecra: f64,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let Some(c) = centro else { return };
    if !(raio_ecra.is_finite() && raio_ecra > 0.0) {
        return;
    }
    let cor = ColorToken::Accent.resolve(theme);
    let p = transform * Point::new(c[0], c[1]);
    target.inner_mut().stroke(
        &Stroke::new(1.5),
        Affine::IDENTITY,
        &Brush::Solid(VelloColor::from_rgba8(cor.r, cor.g, cor.b, cor.a)),
        None,
        &Circle::new(p, raio_ecra),
    );
}

/// ⭐⭐⭐ **A RAMPA DE PESO** — as cinco paradas que todo programa de rig usa, percorridas a
/// velocidade PERCEPTUAL constante.
///
/// # ⛔⛔⛔ Porque a rampa de dois tokens não servia (report do dono, 2026-09-19)
///
/// *«as cores não ficam tão boas como no blender»* — e a causa é uma **confusão de categoria**: a
/// rampa era `ColorToken::Info → ColorToken::Danger`, e um token semântico é escolhido para ser
/// **CALMO** dentro do chrome (`Info` é `oklch(0,720 0,110 235)`, um azul de baixa croma). Uma
/// leitura de VALOR é escolhida para ser **DISTINGUÍVEL**. São requisitos opostos, e o preço
/// mede-se: interpolada entre dois tons calmos, a rampa passa por um **cinza-lilás** a meio
/// (croma `0,048`) onde `0,45` e `0,55` são a mesma cor.
///
/// # ⭐ A medição, em OKLab, com 21 amostras (passo de `0,05`)
///
/// | rampa | caminho total | **pior passo** | uniformidade |
/// |---|---|---|---|
/// | `Info → Danger` (a de ontem) | `0,305` | `0,0131` | `0,739` |
/// | 5 paradas espaçadas em `t` (o porte ingénuo) | `1,435` | `0,0082` | **`0,057`** |
/// | 5 paradas em OKLCH, por arco | `1,397` | `0,0383` | `0,443` |
/// | **5 paradas em OKLab, por ARCO** (esta) | `1,393` | **`0,0497`** | `0,651` |
///
/// ⇒ **`3,8 ×` o pior passo da rampa de ontem** e `4,6 ×` o caminho percorrido. *O pior passo é o
/// número que decide, porque é ele que diz se dois pesos vizinhos se distinguem.*
///
/// # ⛔⛔ E o porte INGÉNUO da rampa da indústria seria PIOR que o que havia
///
/// Com as paradas em `0`, `¼`, `½`, `¾`, `1` — que é como elas são publicadas — o verde puro é um
/// **planalto**: o pior passo cai para `0,0082`, `0,63 ×` o da rampa de ontem, exactamente a meio
/// da faixa. ⭐ A cura não é mudar as cores: é **parametrizar por comprimento de arco**, o que
/// deixa as paradas em `0,000 · 0,380 · 0,546 · 0,680 · 1,000`. O verde — a única referência que um
/// artista de facto lê («metade») — desloca-se `+0,046`, e o ciano, que ninguém lê como número,
/// paga os `+0,130`.
///
/// # ⚠️ Ela não muda com o TEMA, e isso é a lei
///
/// *Uma rampa que muda com o tema deixa de ser a leitura de um número.* Verde tem de querer dizer
/// «metade» nos quatro temas, senão o artista tem de aprender quatro rampas.
struct Rampa {
    /// As cinco paradas em OKLab (`L`, `a`, `b`) — o espaço em que a interpolação é recta.
    lab: [[f64; 3]; 5],
    /// Onde cada parada cai em `0..1`, proporcional ao comprimento de arco do troço que a precede.
    pos: [f64; 5],
}

/// As cinco paradas, como cantos do cubo sRGB.
///
/// LITERAL-COLOR-OK: rampa de dados — cada cor É UM NÚMERO e não um tom do tema; ver [`Rampa`] para
/// a medição que separa as duas coisas. É a mesma escada (azul → ciano → verde → amarelo →
/// vermelho) que Blender, Maya e Max usam para peso, e é isso que faz um artista ler esta tela sem
/// aprender nada.
const PARADAS: [[u8; 3]; 5] = [
    [0, 0, 255],
    [0, 255, 255],
    [0, 255, 0],
    [255, 255, 0],
    [255, 0, 0],
];

impl Rampa {
    /// Resolve as paradas e mede o comprimento de cada troço — ver [`Rampa`].
    fn nova() -> Self {
        let lab = PARADAS.map(|[r, g, b]| {
            // ⚠️ **A conversão é a PORTA da casa** ([`ph2d_tokens::srgb_to_oklch`]) e não uma
            // segunda cópia da matriz do Ottosson: *duas derivações do mesmo espaço de cor
            // divergem no dia em que uma ganhar uma cerca*.
            let (l, c, h) = ph2d_tokens::srgb_to_oklch(r, g, b);
            let rad = h.to_radians();
            [l, c * rad.cos(), c * rad.sin()]
        });
        let mut pos = [0.0; 5];
        let vaos: [f64; 4] = std::array::from_fn(|i| distancia(lab[i], lab[i + 1]));
        let total: f64 = vaos.iter().sum();
        if total > 0.0 {
            for i in 0..4 {
                pos[i + 1] = pos[i] + vaos[i] / total;
            }
        }
        pos[4] = 1.0;
        Self { lab, pos }
    }

    /// A cor do peso `t`, com `t` fora de `0..1` a saturar nas pontas.
    fn cor(&self, t: f64) -> VelloColor {
        let t = t.clamp(0.0, 1.0);
        let i = (0..4).rev().find(|&i| t >= self.pos[i]).unwrap_or(0);
        let vao = self.pos[i + 1] - self.pos[i];
        let u = if vao > 0.0 {
            ((t - self.pos[i]) / vao).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let m: [f64; 3] =
            std::array::from_fn(|k| self.lab[i][k] + (self.lab[i + 1][k] - self.lab[i][k]) * u);
        let [r, g, b] =
            ph2d_tokens::oklch_to_srgb(m[0], m[1].hypot(m[2]), m[2].atan2(m[1]).to_degrees());
        VelloColor::from_rgba8(r, g, b, 255)
    }
}

/// A distância euclidiana em OKLab — a régua da legibilidade desta rampa.
fn distancia(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d: [f64; 3] = std::array::from_fn(|k| a[k] - b[k]);
    d[2].mul_add(d[2], d[1].mul_add(d[1], d[0] * d[0])).sqrt()
}

#[cfg(test)]
#[path = "peso_tests.rs"]
mod peso_tests;
