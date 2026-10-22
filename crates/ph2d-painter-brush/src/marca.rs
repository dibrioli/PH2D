//! **O jitter da MARCA que faltava** (L4, `docs/Painter/Pinceis/03_MOTORES.md` §4 e `06_LEIS_DAS_LACUNAS.md` §3):
//! a opacidade por dab, a contagem sorteada do spray, os espelhos X/Y, a redondeza, o ângulo sorteado UMA vez
//! por traço, o espalhamento só ATRAVÉS do traço e o tamanho da partícula do spray. O Position / Scale /
//! Rotation / Spacing / Count de sempre ficam em [`crate::jitter`] e no `BrushSpec`.
//!
//! ⭐ **Cada campo neutro = o traço de hoje ao bit**: um campo a zero não sorteia nada, e os ligados sorteiam
//! de um fluxo PRÓPRIO do traço (`PorTraco::rng`, semeado pela `seed`), logo ligar um não desloca os sorteios
//! de sempre (escala → rotação → cor → posição): eles saem ao bit. Transcendental-free (HR-5).
//!
//! ⛔ **Fluxo por dab NÃO é um campo**: o motor multiplica a Strength e o Flow no MESMO produto por dab
//! (`dab.rs`, BUGS #41), e a [`DabJitter::opacity`] já é esse produto — um segundo seria um controlo que mente
//! (a mesma lei da L2 para a pressão→fluxo).

use crate::jitter::next_f32;

/// O jitter da marca (os campos novos da L4). [`DabJitter::NEUTRO`] = o traço de sempre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DabJitter {
    /// **Opacidade por dab** `0..1`: a cobertura de cada marca × `1 ± amount` (simétrico: a média fica — o
    /// Splats do Rebelle, opacidade por dab cv 0,08–0,20 em torno de 0,33–0,37).
    pub opacity: f32,
    /// **Contagem sorteada** `0..1` (Photoshop *Count Jitter*): cada ponto do caminho deixa `n − ⌊amount·u·n⌉`
    /// marcas do spray, nunca menos de uma. Sem spray (`spray_count = 1`) não há o que sortear.
    pub count: f32,
    /// **Espelho X** `0..1`: a probabilidade de a ponta sair espelhada no eixo X dela (Photoshop *Flip X
    /// Jitter* = `0,5`). Só se VÊ numa ponta que não é simétrica (Shape de imagem, grão que roda com o dab).
    pub flip_x: f32,
    /// **Espelho Y** `0..1`: idem no eixo Y da ponta.
    pub flip_y: f32,
    /// **Redondeza** `0..1` (Photoshop *Roundness Jitter*): o eixo MENOR de cada marca × `1 − amount·u` — a
    /// ponta achata-se ao acaso, marca a marca.
    pub roundness: f32,
    /// **Ângulo do traço** `0..1` (Procreate *Rotation → Randomized*): a ponta roda `±amount·180°` sorteados UMA
    /// vez por traço — o traço inteiro com o mesmo ângulo, o seguinte com outro.
    pub angle_per_stroke: f32,
    /// **Espalhar através** `0..2` diâmetros (Photoshop *Scatter* sem *Both Axes*): cada marca desvia-se
    /// `±amount·D` só na perpendicular ao traço — o spatter do Rebelle espalha 0,6–0,9·D através, ~3× o ao longo.
    pub across: f32,
    /// **Partícula** `0,05..1` (o tamanho de cada marca do spray em fracção do pincel; o espalhamento continua o
    /// do pincel): `1` = a marca do tamanho do pincel. Só com spray (`spray_count > 1`).
    pub particle: f32,
    /// **Partícula em PÍXEIS** (J4 `texturas`): o diâmetro de cada marca do spray na TELA, o mesmo a 5 e a 200 px
    /// — o espalhamento continua o do pincel (a gotícula do Spray e do Pointillism do Krita não cresce com o
    /// pincel: a 200 px são gotas soltas, não discos maiores; a 5 px um cordão de gotas MAIORES que o pincel).
    /// `0` = desligada (a [`Self::particle`] relativa de sempre). Só com spray (`spray_count > 1`).
    pub particle_px: f32,
    /// ⭐ **O TECTO da partícula em PÍXEIS** (J4 `texturas`, «Motor J4»): o diâmetro máximo de cada marca do spray na
    /// TELA — o espalhamento continua o do pincel. O Spatter/Star do Rebelle a 200 px espalha as estrelas por
    /// ~234 px mas nenhuma passa de 56 px (a caixa da imagem dele: as que passariam saem QUADRADAS). `0` =
    /// desligado. Só com spray (`spray_count > 1`).
    pub particle_max_px: f32,
    /// ⭐ **O PISO da partícula em PÍXEIS** (J4 `texturas`, «Motor J4»): o diâmetro mínimo de cada marca do spray — a
    /// 5 px o raminho do Foliage a 0,5 do pincel tem 2,5 px e a linha parte-se (revisões r8–r9: o Krita é uma linha
    /// cheia). `0` = desligado. Só com spray (`spray_count > 1`).
    pub particle_min_px: f32,
    /// ⭐ **O VÉU** (J4 `texturas`, «Motor J4»): a fracção `0..1` das marcas do spray que sai da 2.ª população — maior
    /// ([`Self::veu_escala`]) e pálida ([`Self::veu_opacidade`]): a névoa clara por baixo das gotas do Texture Spray
    /// do Krita. Sorteado de um fluxo PRÓPRIO do traço. `0` = desligado (uma população, a de sempre).
    pub veu: f32,
    /// ⭐ **O PÓ SOLTO** (J4 `texturas`, «Motor J4»; pedido da família `pastel_carvao`): grãos de pó à VOLTA do
    /// traço, fora da borda — grãos por 100 px² da área do traço (`0` = desligado). Medido no Rebelle (o recto de 200
    /// e o de 40 px da bateria): o Soft Pastel `Dry Thin` 0,85–0,9, o Charcoal 0,6, o Flat Soft 1,25 — a densidade
    /// por área é a mesma a 40 e a 200 px. Contados sem sorteio (o resto acumula de ponto em ponto); a posição, o
    /// tamanho e o tom de cada grão de um fluxo PRÓPRIO. Vale para todo meio que pinta os dabs do traço (Digital, Wet).
    pub po: f32,
    /// Até onde o pó vai além da borda, em raios do dab: a distância é `raio·(1 + po_alcance·u²)` (o Rebelle: a
    /// mediana a ~0,3 e o p90 a ~0,9 da meia-largura no Soft Pastel ⇒ ~1,1; o Charcoal ~0,5; o Flat Soft ~3).
    pub po_alcance: f32,
    /// O diâmetro de um grão em px (× `0,5..1,5` sorteado).
    pub po_px: f32,
    /// A cobertura de um grão em fracção da do dab (× `0,4..1` sorteado; o Rebelle: o tom médio dos grãos ~0,17).
    pub po_opacidade: f32,
    /// ⭐ O LADO do pó (J4 `pastel_carvao` 2.ª ronda): a fracção `-1..1` dos grãos do lado ESQUERDO que passa para o
    /// DIREITO do rumo do traço (`<0`: o contrário). Medido no Rebelle (recto de 200 e verticais da bateria): o pó cai
    /// à direita de quem anda — Soft Pastel 411 × 191 grãos (`0,37`), Charcoal 231 × 91 (`0,43`), Flat Soft ~0,15. `0` =
    /// à volta por igual, sem sorteio a mais (ao bit).
    pub po_lado: f32,
    /// O raio de uma marca do véu × isto (`> 0`; só com [`Self::veu`]).
    pub veu_escala: f32,
    /// A cobertura de uma marca do véu × isto (`0..1`; só com [`Self::veu`]).
    pub veu_opacidade: f32,
}

impl DabJitter {
    /// O jitter de sempre: nada sorteia, nada muda.
    pub const NEUTRO: Self = Self {
        opacity: 0.0,
        count: 0.0,
        flip_x: 0.0,
        flip_y: 0.0,
        roundness: 0.0,
        angle_per_stroke: 0.0,
        across: 0.0,
        particle: 1.0,
        particle_px: 0.0,
        particle_max_px: 0.0,
        particle_min_px: 0.0,
        veu: 0.0,
        veu_escala: 1.0,
        veu_opacidade: 1.0,
        po: 0.0,
        po_alcance: 1.0,
        po_px: 2.0,
        po_opacidade: 0.3,
        po_lado: 0.0,
    };

    /// A menor partícula (`5 %` do pincel): abaixo disso o dab do raio mínimo (`0,5 px`) já é o que se pinta.
    pub const PARTICULA_MIN: f32 = 0.05;
    /// O maior espalhamento através, em diâmetros (o Spray do Krita mede 1,2·D; o dobro dá a margem do artista).
    pub const ACROSS_MAX: f32 = 2.0;

    /// Algum dos sorteios de FORMA está ligado (espelho ou redondeza) — o dab leva então a sua pegada própria.
    #[must_use]
    pub fn muda_a_forma(&self) -> bool {
        self.flip_x > 0.0 || self.flip_y > 0.0 || self.roundness > 0.0
    }

    /// ⭐ **A partícula efectiva de um pincel de raio `raio_px`** (o factor do raio de cada marca): a em píxeis
    /// ([`Self::particle_px`]) quando ligada — `diâmetro ÷ (2·raio)`, acima de `1` num pincel menor que ela —, senão
    /// a relativa (`≤ 1`).
    #[must_use]
    pub fn particula_no(&self, n: u32, raio_px: f32) -> f32 {
        if n > 1 && self.particle_px > 0.0 && raio_px > 0.0 {
            self.particle_px / (2.0 * raio_px)
        } else {
            self.particula(n)
        }
    }

    /// O raio de uma marca do spray de `n` entre o piso [`Self::particle_min_px`] e o tecto
    /// [`Self::particle_max_px`] (o próprio `raio` sem eles).
    #[must_use]
    pub fn com_tecto(&self, n: u32, raio: f32) -> f32 {
        if n <= 1 {
            return raio;
        }
        let raio = if self.particle_min_px > 0.0 {
            raio.max(0.5 * self.particle_min_px)
        } else {
            raio
        };
        if self.particle_max_px > 0.0 {
            raio.min((0.5 * self.particle_max_px).max(0.5))
        } else {
            raio
        }
    }

    /// A partícula efectiva para um spray de `n` marcas (`1` sem spray: o campo não age).
    #[must_use]
    pub fn particula(&self, n: u32) -> f32 {
        if n > 1 {
            self.particle.clamp(Self::PARTICULA_MIN, 1.0)
        } else {
            1.0
        }
    }
}

impl Default for DabJitter {
    fn default() -> Self {
        Self::NEUTRO
    }
}

/// O que os sorteios da marca decidiram para UM dab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Marca {
    /// O multiplicador da cobertura (`1` = o de sempre).
    pub cobertura: f32,
    /// O multiplicador do eixo menor (`1` = a ponta do pincel).
    pub redondeza: f32,
    /// A ponta espelhada no eixo Y dela (o espelho X é este + meia-volta, [`Marca::meia_volta`]).
    pub espelho: bool,
    /// Meia-volta da ponta (o espelho X = espelho Y + 180°; os dois espelhos juntos = só a meia-volta).
    pub meia_volta: bool,
    /// O desvio através do traço, em px.
    pub desvio: [f32; 2],
}

/// Os sorteios da marca, pela ordem fixa opacidade → redondeza → espelho X → espelho Y → através; cada um só
/// quando o seu campo está ligado (`None` = nenhum ligado: nada sorteado). `dir` é o heading do traço (unitário;
/// `[0, 0]` sem direcção ainda — o desvio através fica `0`, mas o sorteio acontece para o fluxo ser o mesmo).
pub(crate) fn sorteia(rng: &mut u64, j: &DabJitter, diametro: f32, dir: [f32; 2]) -> Option<Marca> {
    if j.opacity <= 0.0 && !j.muda_a_forma() && j.across <= 0.0 {
        return None;
    }
    let simetrico = |rng: &mut u64| next_f32(rng) * 2.0 - 1.0;
    let cobertura = if j.opacity > 0.0 {
        1.0 + simetrico(rng) * j.opacity.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let redondeza = if j.roundness > 0.0 {
        1.0 - next_f32(rng) * j.roundness.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let fx = j.flip_x > 0.0 && next_f32(rng) < j.flip_x.clamp(0.0, 1.0);
    let fy = j.flip_y > 0.0 && next_f32(rng) < j.flip_y.clamp(0.0, 1.0);
    let desvio = desvio_atraves(rng, j, diametro, dir);
    Some(Marca {
        cobertura,
        redondeza,
        espelho: fx != fy,
        meia_volta: fx,
        desvio,
    })
}

/// O desvio ATRAVÉS do traço (`dir` = o heading; `[0, 0]` sem direcção), sorteado do fluxo da marca só com o
/// campo ligado — a UMA conta do dab do caminho ([`sorteia`]) e dos que preenchem o vão do arremesso (`speed.rs`).
pub(crate) fn desvio_atraves(
    rng: &mut u64,
    j: &DabJitter,
    diametro: f32,
    dir: [f32; 2],
) -> [f32; 2] {
    if j.across <= 0.0 {
        return [0.0, 0.0];
    }
    let d = (next_f32(rng) * 2.0 - 1.0) * j.across.clamp(0.0, DabJitter::ACROSS_MAX) * diametro;
    [-dir[1] * d, dir[0] * d]
}

/// A contagem sorteada de um ponto do caminho com `n` marcas (`n` sem sorteio quando o campo está a zero ou
/// não há spray).
pub(crate) fn conta(rng: &mut u64, j: &DabJitter, n: u32) -> u32 {
    if j.count <= 0.0 || n <= 1 {
        return n;
    }
    let tira = (next_f32(rng) * j.count.clamp(0.0, 1.0) * n as f32).round() as u32;
    n.saturating_sub(tira).max(1)
}

/// O rotor do ângulo do traço (`[1, 0]` com o campo a zero), sorteado do fluxo PRÓPRIO do traço (`seed`), como
/// a cor por traço — o fluxo por dab que os outros sorteios leem fica intacto.
pub(crate) fn rotor_do_traco(j: &DabJitter, seed: u64) -> [f32; 2] {
    if j.angle_per_stroke <= 0.0 {
        return [1.0, 0.0];
    }
    let mut rng = seed ^ 0x616E_676C_6550_5354;
    let deg = ((next_f32(&mut rng) * 2.0 - 1.0) * j.angle_per_stroke.clamp(0.0, 1.0) * 180.0)
        .round() as i32;
    crate::texture::rotate_by_degrees((((deg % 360) + 360) % 360) as u16)
}

#[cfg(test)]
#[path = "marca_tests.rs"]
mod tests;
