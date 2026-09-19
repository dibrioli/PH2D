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
/// ⭐⭐ **A rampa é `Info → Danger`, e os dois extremos são TOKENS e não cores.** Ela é a rampa que
/// toda ferramenta de rig usa (azul = nada, vermelho = tudo), e usá-la aqui quer dizer que um
/// artista que já pesou um rig noutro programa lê esta tela sem aprender nada.
///
/// ⛔ **Um peso de `0` é pintado, e é a metade que importa.** Sem ele o artista vê os pontos que o
/// osso já governa e **não vê onde ele devia governar e não governa** — que é exactamente o defeito
/// que ele veio corrigir. *Desenhar só o que já está certo é desenhar o problema invisível.*
pub fn draw_weights(
    pontos: &[([f64; 2], f64)],
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    if pontos.is_empty() {
        return;
    }
    let frio = ColorToken::Info.resolve(theme);
    let quente = ColorToken::Danger.resolve(theme);
    for &(p, w) in pontos {
        if !(p[0].is_finite() && p[1].is_finite() && w.is_finite()) {
            continue;
        }
        let t = w.clamp(0.0, 1.0);
        let c = mistura(frio, quente, t);
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

/// A rampa, em `sRGB` directo.
///
/// ⚠️ **Sem correcção de gama de propósito:** isto é uma LEITURA de um número, e não tinta — o que
/// o artista precisa é de distinguir `0,3` de `0,6`, e a interpolação linear nos bytes tem contraste
/// mais uniforme para esse fim do que a linear na luz.
fn mistura(a: ph2d_tokens::Color, b: ph2d_tokens::Color, t: f64) -> VelloColor {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o `t` vem clampado a 0..1 e os canais sao bytes, logo a soma cabe em u8"
    )]
    let mix = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as u8;
    VelloColor::from_rgba8(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b), 255)
}
