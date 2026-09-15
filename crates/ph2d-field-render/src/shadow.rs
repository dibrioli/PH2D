//! ⭐⭐⭐ **A SOMBRA É UM CANAL DO G-BUFFER** — quem tem a cena calcula-a; quem pinta só a multiplica.
//!
//! # Porque não é uma conta do pintor
//!
//! O [`crate::shade_render`] não conhece campo nenhum, de propósito: ele recebe geometria já
//! resolvida e devolve bytes. Uma sombra precisa de **marchar o campo**, que é exactamente o que
//! aquela fronteira mantém do lado de fora. ⇒ o passe corre onde o traçado corre, em LOTE (a
//! [`crate::march::march_shadow_to`] quer raios aos milhares, não um de cada vez), e o resultado
//! viaja como mais um plano de pixels, como a normal e o ponto já viajam.
//!
//! # ⭐ O preço, MEDIDO (2026-09-14, `load 2,53`, peça de três cilindros cruzados)
//!
//! | px | traçado | sombra (de frente + cerca da bola) | razão |
//! |---|---:|---:|---:|
//! | `640×360` — o quadro de MOVIMENTO (piso D=3) | `3,51 ms` | `6,09 ms` | `1,74×` |
//! | `1920×1080` — o quadro ASSENTE | `28,61 ms` | `56,23 ms` | `1,97×` |
//!
//! **Um raio de sombra custa `29,3` amostras de campo contra `8,7` de um raio de câmera — `3,4×`.**
//!
//! # ⛔⛔ As três alavancas óbvias foram MEDIDAS, e duas quase não compram nada
//!
//! | alavanca | população | relógio | veredito |
//! |---|---:|---:|---|
//! | só os pixels que VÊEM a luz (`N·L > 0`) | `−55,3 %` | `−6 %` | fica, mas não é ela que paga |
//! | cerca na LÂMPADA em vez de `2r` | — | `−5 %` | fica: é a cerca honesta |
//! | cerca na BOLA que contém a peça | — | `−10 %` | fica — ⚠️ **e não pelo relógio**, ver abaixo |
//! | viés de partida `4 → 256` | — | `−54 %` | ⛔ **RECUSADA: apaga sombra** (`27,3 % → 20,0 %` de pixels tapados) |
//!
//! ⭐⭐⭐ *O achado é a primeira linha:* cortar `55 %` dos raios corta `6 %` do relógio — **os raios
//! de costas para a luz terminam no primeiro passo, dentro da própria peça: são os BARATOS.** O caro
//! é o raio que viaja; e apertar-lhe a cerca `4,7×` (mediana `3,200 → 0,678`) compra `11 %`, o que
//! diz que ele também não paga na viagem. *Ele paga a RASTEJAR à saída da superfície*, onde `d ≈ 0`
//! e a lei `t += d·passo` mal o move. Não há alavanca barata: a sombra custa o que custa.
//!
//! # ⚠️⚠️ A cerca da BOLA não é uma optimização — é o DOMÍNIO DA PERGUNTA
//!
//! Eu escrevi-a como poupança (*«um raio que saiu da bola nunca lá volta»*) e a prova de mutação
//! desmentiu-me: apagá-la põe **1 554 dos 28 640 pixels de uma ESFERA** a vir sombreados, e uma
//! esfera é convexa. O mecanismo é o **estimador de penumbra**, `vis = min(k·d/t)`: numa saída
//! RASANTE de um corpo convexo a tangente afasta-se da superfície como `d ≈ t²/2R`, logo
//! `k·d/t ≈ k·t/2R` é **menor que `1`** enquanto `t < 2R/k` — *o estimador está a ler a superfície
//! de ONDE O RAIO SAIU e a chamar-lhe oclusor.*
//!
//! ⇒ a cerca certa é a que diz **onde a pergunta tem sentido**: fora da bola que contém a peça não
//! existe geometria nenhuma, logo nada do que se leia ali é informação sobre oclusão. Ela é parte
//! da definição do passe, e quem a defende é o gate da esfera.
//!
//! ⛔ *Uma cerca que corrige um artefacto e se documenta como poupança é a mais perigosa que há:
//! o primeiro que a apertar por relógio apaga a correcção sem saber que ela existia.*
//!
//! # ⭐⭐⭐ E é por isso que ela viaja na bandeira que JÁ EXISTE
//!
//! O módulo tem, desde a W73, uma lei escrita e com dois consumidores — *grosso a mexer, nítido ao
//! assentar* (o contorno engrossado e o anti-serrilhado desligado saem da **mesma** bandeira, com a
//! nota a avisar que *«uma segunda pergunta para o mesmo facto podia divergir dela»*). A sombra é o
//! **terceiro passageiro**, e não uma pergunta nova:
//!
//! - o quadro de MOVIMENTO fica **byte-idêntico** ao de hoje — zero risco de empurrar uma peça
//!   pesada contra o piso do divisor, que a [`crate::super::preview`] declara ficar *preso e lento*;
//! - o quadro ASSENTE paga `56 ms` sobre os `28,6` que já custava, **noutra thread** — a janela
//!   continua a 60 Hz, que é a cerca que aquele módulo inteiro protege.

use crate::march::{Scene, march_shadow_to};
use crate::{Gbuffer, Orbit, Sharpness, Stencil};
use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::Registry;

/// A visibilidade de cada lâmpada em cada pixel: `1,0` = a luz chega, `0,0` = tapado.
///
/// ⚠️ **`per_lamp[l][i]`, e não `[i][l]`** — a marcha é um LOTE por lâmpada, e é assim que ela sai.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shadows {
    per_lamp: Vec<Vec<f32>>,
    /// ⭐⭐⭐ **O CÉU É UMA FONTE COMO AS OUTRAS, e a oclusão é a sombra DELE.**
    ///
    /// Quanto do ambiente chega a cada pixel — ver [`occlusion_passes`]. Vazio = chega inteiro.
    ///
    /// ⚠️ Ela mora aqui e não num campo novo do [`crate::Lighting`] porque é a MESMA pergunta que
    /// as lâmpadas respondem: *«quanto desta fonte chega a este pixel?»*. Um segundo canal ao lado
    /// faria o pintor perguntar duas vezes a mesma coisa, e é assim que dois canais divergem.
    ambient: Vec<f32>,
    pixels: usize,
}

impl Shadows {
    /// Quanto da lâmpada `lamp` chega ao pixel `i`. ⚠️ **Fora de alcance devolve `1,0`** — uma
    /// sombra que não foi calculada é ausência de sombra, nunca escuridão.
    #[must_use]
    pub fn at(&self, lamp: usize, i: usize) -> f32 {
        self.per_lamp
            .get(lamp)
            .and_then(|v| v.get(i))
            .copied()
            .unwrap_or(1.0)
    }

    /// Quanto do CÉU chega ao pixel `i`. ⚠️ **Fora de alcance devolve `1,0`**, pela mesma lei do
    /// [`Shadows::at`]: uma oclusão que não foi calculada é ausência de oclusão, nunca escuridão.
    #[must_use]
    pub fn ambient_at(&self, i: usize) -> f32 {
        self.ambient.get(i).copied().unwrap_or(1.0)
    }

    /// ⭐ **O canal de UMA lâmpada**, para quem o calculou noutro sítio (o traçador de GPU).
    ///
    /// ⚠️ Ela existe porque o `per_lamp` é privado de propósito: a forma dele (`[lâmpada][pixel]`)
    /// é uma decisão do passe, e um campo público congelá-la-ia.
    pub fn set_lamp(&mut self, lamp: usize, vis: Vec<f32>) {
        if self.pixels == 0 {
            self.pixels = vis.len();
        }
        if self.per_lamp.len() <= lamp {
            self.per_lamp.resize(lamp + 1, Vec::new());
        }
        self.per_lamp[lamp] = vis;
    }

    /// Põe a oclusão do céu neste canal — a saída de [`occlusion_passes`].
    pub fn set_ambient(&mut self, ambient: Vec<f32>) {
        if self.pixels == 0 {
            self.pixels = ambient.len();
        }
        self.ambient = ambient;
    }

    #[must_use]
    pub fn lamps(&self) -> usize {
        self.per_lamp.len()
    }

    #[must_use]
    pub fn pixels(&self) -> usize {
        self.pixels
    }
}

/// ⭐ **A dureza da penumbra** — quanto a sombra endurece com a distância ao oclusor.
///
/// ⚠️ Ela **não** é um recurso nem um teto: é a constante da lei de penumbra `min(k·d/t)`, e o
/// número é o do idioma (`8`) que dá uma borda visível sem a esborratar. Subi-la endurece; descê-la
/// alarga. ⛔ Não a confunda com o viés de partida, que compra relógio APAGANDO sombra.
pub const HARDNESS: f32 = 8.0;

/// ⭐⭐⭐ **O passe: uma visibilidade por lâmpada por pixel de peça.**
///
/// As três cercas da nota do módulo, todas aplicadas:
/// 1. só os pixels que **vêem** a luz recebem raio (um de costas já está escuro pelo `N·L`);
/// 2. o raio pára **na lâmpada** — nada além dela pode tapar;
/// 3. e pára na **saída da bola** que contém a peça — que é o DOMÍNIO da pergunta.
///
/// ⚠️⚠️ **A (3) não é redundante com a (2), e não é uma poupança** — ver a nota do módulo. As
/// duas dão a mesma imagem na peça de cilindros cruzados (`discordam 0`, medido), e é
/// precisamente por isso que só uma fixtura CONVEXA as separa: sem ela, `1 554` dos `28 640`
/// pixels de uma esfera vêm sombreados. O gate que a defende é o `uma_esfera_nao_se_tapa_a_si_propria`.
#[must_use]
pub fn shadow_pass(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    lamps_world: &[[f32; 3]],
) -> Shadows {
    let pixels = g.hit.len();
    if lamps_world.is_empty() || pixels == 0 {
        return Shadows {
            per_lamp: Vec::new(),
            ambient: Vec::new(),
            pixels,
        };
    }
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let (right, up, toward_eye) = cam.basis();
    let scene = Scene {
        shape: &shape,
        cam,
        basis: (right, up, toward_eye),
        sharp: Sharpness::for_frame(cam.half_extent, g.width.min(g.height) as usize),
        clip: None,
        step: ph2d_field_eval::safe_march_step(doc),
        shrink: ph2d_field_eval::field_shrink(doc, reg),
        stencil: Stencil::Tetra4,
    };
    let bola = ph2d_field_eval::bounds::bounding_ball(doc, reg);
    // Quanto o ponto se ergue da superfície antes de o raio partir — ver a nota no laço.
    let lift = scene.sharp.hit * crate::march::BIAS;

    let per_lamp = lamps_world
        .iter()
        .map(|luz| {
            let mut vis = vec![1.0f32; pixels];
            let mut quais = Vec::new();
            let mut origens = Vec::new();
            let mut dirs = Vec::new();
            let mut cercas = Vec::new();
            for i in 0..pixels {
                if !g.hit[i] {
                    continue;
                }
                let p = g.point[i];
                let d = [luz[0] - p[0], luz[1] - p[1], luz[2] - p[2]];
                let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if dist <= f32::EPSILON {
                    continue;
                }
                let dir = [d[0] / dist, d[1] / dist, d[2] / dist];
                // ⚠️ **A normal do G-buffer está em espaço de VISTA e a luz em MUNDO.** A base
                // converte — e é a MESMA conversão que o `shade_render` faz para o `N·L`, senão a
                // sombra e a luz discordariam sobre quem vê quem.
                let n = g.normal[i];
                let nm = [
                    n[0] * right[0] + n[1] * up[0] + n[2] * toward_eye[0],
                    n[0] * right[1] + n[1] * up[1] + n[2] * toward_eye[1],
                    n[0] * right[2] + n[1] * up[2] + n[2] * toward_eye[2],
                ];
                // ⭐⭐ **De costas para a luz: sem raio — e a visibilidade fica em `1,0`.**
                //
                // ⚠️ **`1,0` e não `0,0`, e a diferença NÃO é visível hoje**: o `N·L ≤ 0` já anula
                // a contribuição da lâmpada, logo as duas respostas pintam o mesmo pixel. Mas o
                // canal diz *«quanto da lâmpada CHEGA»*, e a resposta verdadeira é *«ela não está
                // tapada por nada»* — escrever `0` seria uma mentira que por acaso não se nota, até
                // ao dia em que alguém ler este canal para outra coisa (a OpenPBR tem termos que
                // recebem luz com `N·L < 0`). O gate da esfera reprova a mentira: uma mutação que
                // apague este filtro põe METADE da esfera a vir sombreada.
                if nm[0] * dir[0] + nm[1] * dir[1] + nm[2] * dir[2] <= 0.0 {
                    continue;
                }
                quais.push(i);
                // ⭐⭐⭐ **O RAIO PARTE AO LONGO DA NORMAL, e não ao longo de si próprio.**
                //
                // ⛔⛔ **Acne de sombra, apanhada pelo gate da ESFERA:** um corpo convexo não se pode
                // tapar a si próprio, e `10,5 %` dos pixels de uma esfera saíam **DUROS** (`vis = 0`,
                // e não penumbra — a sonda `probe_de_onde_vem_a_sombra_da_esfera` separa as duas).
                // A causa é o ângulo RASANTE: o raio arrancava em `p` e andava `hit·BIAS` **na
                // direcção da luz**, logo com `N·L ≈ 0` ele viaja quase tangente e, ao fim daquele
                // troço, ainda está a menos de `hit` da superfície de onde saiu — *o campo responde
                // «acertaste» sobre a peça em que o raio já estava.*
                //
                // ⚠️ **A cura NÃO é um viés maior**, e isso está MEDIDO: subir o viés de `4` para
                // `256` compra `54 %` de relógio e apaga sombra a sério (`27,3 % → 20,0 %` de pixels
                // tapados). Um viés ao longo do RAIO tem de crescer com `1/(N·L)` para limpar a
                // superfície, e na rasante isso diverge.
                //
                // ⭐ Erguer o ponto `ε` pela NORMAL resolve-o **em qualquer ângulo e com um `ε`
                // só**: no ponto erguido o campo vale `≈ ε` seja qual for a direcção do raio.
                let erguido = [
                    p[0] + nm[0] * lift,
                    p[1] + nm[1] * lift,
                    p[2] + nm[2] * lift,
                ];
                origens.push(erguido);
                dirs.push(dir);
                cercas.push(cerca(bola.as_ref(), p, dir, dist));
            }
            let v = march_shadow_to(&scene, &origens, &dirs, &cercas, HARDNESS);
            for (j, &i) in quais.iter().enumerate() {
                vis[i] = v[j];
            }
            vis
        })
        .collect();

    Shadows {
        per_lamp,
        ambient: Vec::new(),
        pixels,
    }
}

/// Onde o raio pode parar: na lâmpada, ou na saída da bola — o que vier primeiro.
fn cerca(
    bola: Option<&ph2d_field_eval::bounds::Ball>,
    p: [f32; 3],
    dir: [f32; 3],
    ate_a_luz: f32,
) -> f32 {
    let Some(b) = bola else {
        return ate_a_luz;
    };
    let oc = [p[0] - b.center[0], p[1] - b.center[1], p[2] - b.center[2]];
    let bb = oc[0] * dir[0] + oc[1] * dir[1] + oc[2] * dir[2];
    let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - b.radius * b.radius;
    let disc = bb * bb - c;
    if disc <= 0.0 {
        // O raio nem toca a bola: nada o pode tapar. ⚠️ `0` e não `ate_a_luz` — marchar seria
        // perguntar a um campo que já se sabe vazio.
        return 0.0;
    }
    (-bb + disc.sqrt()).max(0.0).min(ate_a_luz)
}

/// ⭐ **O ALCANCE da oclusão**, em unidades do enquadramento — até onde um vizinho ainda escurece.
///
/// # ⛔ O número foi MEDIDO depois de eu o ter ESCOLHIDO
///
/// A 1.ª redacção dizia `0,35`, a olho. Varrido (`measure_the_occlusion_reach`, três cilindros
/// cruzados), a resposta **satura** — e o que a mostra é a CAUDA, nunca a média:
///
/// | alcance | mín | p05 | abaixo de `0,8` | céu MÉDIO |
/// |---:|---:|---:|---:|---:|
/// | `0,15` | `0,688` | `1,000` | `0,3 %` | `0,997` |
/// | `0,35` | `0,188` | `0,938` | `2,4 %` | `0,985` |
/// | `0,60` | `0,062` | `0,875` | `4,1 %` | `0,973` |
/// | **`1,00`** | `0,062` | `0,750` | **`5,8 %`** | `0,965` |
/// | `1,60` | `0,062` | `0,750` | `5,8 %` | `0,965` |
/// | `2,50` | `0,062` | `0,750` | `5,8 %` | `0,965` |
///
/// ⚠️⚠️ **A coluna da MÉDIA é a que engana, e foi ela que quase me fez dar o passe por partido:**
/// numa cruz de cilindros a fenda é `~5 %` dos pixels visíveis, logo uma oclusão de `94 %` no fundo
/// dela move a média da peça de `1,000` para `0,965`. *Ler `0,965` é ler «quase nada»; ler a cauda
/// é ler o que o olho vê.* É o «extremo global» que este repositório já pagou cinco vezes, do lado
/// oposto.
///
/// ⇒ **`1,0`, que é onde a resposta deixa de mudar.** Acima disso não há mais nada para encontrar —
/// e a cerca da bola ([`occlusion_slice`] fecha na saída dela) já o bordava de qualquer forma.
pub const OCCLUSION_REACH: f32 = 1.0;

/// ⭐⭐⭐ **QUANTAS PASSAGENS até a oclusão assentar** — e o recurso que ele nomeia é **a precisão
/// do byte de saída**.
///
/// ⚠️ **A régua é o erro NO PIXEL, e não no canal.** A oclusão multiplica só o termo do ambiente,
/// que é uma fracção do pixel — medir o erro no canal mede-o onde ele não aterra. Medido contra
/// uma referência de `64` raios (`measure_how_many_passes_the_occlusion_needs`):
///
/// | passagens | erro no canal | **erro no PIXEL** | pior pixel |
/// |---:|---:|---:|---:|
/// | `1` | `0,210` | `26,8 bytes` | `165` |
/// | `4` | `0,056` | `5,99` | `43` |
/// | `8` | `0,027` | `2,85` | `19` |
/// | `16` | `0,012` | `1,33` | `9` |
/// | **`32`** | `0,004` | **`0,63`** | `4` |
///
/// ⇒ **`32`, porque é onde o erro médio cai abaixo de UM byte** — abaixo do que a saída consegue
/// mostrar. ⛔ Não é um número escolhido: é o ponto em que continuar a refinar deixa de mudar a
/// imagem.
pub const OCCLUSION_PASSES: u32 = 16;

/// ⭐ **Quão parecidas duas normais têm de ser para a suavização as misturar** — o cosseno entre
/// elas.
///
/// ⚠️ Ele é uma **guarda**, não um teto: borrar através de uma quina esborrata a quina. `0,9` é
/// `~26°`, que separa faces de um vinco e mantém junta a variação suave de um cilindro.
pub const OCCLUSION_BLUR_COS: f32 = 0.9;

/// ⭐⭐⭐ **A SUAVIZAÇÃO GUIADA — e ela não é batota: a oclusão é de BAIXA FREQUÊNCIA.**
///
/// O que uma oclusão tem de respeitar é a fronteira entre superfícies **diferentes**; dentro de uma
/// superfície ela varia devagar por construção. ⇒ uma média `3×3` **guardada pela normal** remove o
/// ruído de amostragem sem tocar em nada que a resposta verdadeira tenha.
///
/// # ⭐ E ela compra METADE das passagens (medido, referência de `256` raios)
///
/// | passagens | erro no pixel | **com suavização** |
/// |---:|---:|---:|
/// | `4` | `7,22 bytes` | `2,19` |
/// | `8` | `4,07` | `1,35` |
/// | **`16`** | `2,34` | **`0,89`** |
/// | `32` | `1,42` | `0,67` |
///
/// ⇒ `16` passagens **com** ela ficam abaixo de um byte, onde `32` **sem** ela ficavam em `1,42`.
/// *Metade da espera e melhor imagem* — e é por isso que o [`OCCLUSION_PASSES`] desceu de `32`
/// para `16` no mesmo dia em que ela entrou.
#[must_use]
pub fn blur_occlusion(g: &Gbuffer, oc: &[f32]) -> Vec<f32> {
    let (w, h) = (g.width as usize, g.height as usize);
    let mut out = oc.to_vec();
    if w == 0 || h == 0 {
        return out;
    }
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !g.hit[i] {
                continue;
            }
            let n0 = g.normal[i];
            let (mut soma, mut n) = (0.0f32, 0u32);
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                    if xx < 0 || yy < 0 || xx >= w as i32 || yy >= h as i32 {
                        continue;
                    }
                    #[allow(clippy::cast_sign_loss)]
                    let j = yy as usize * w + xx as usize;
                    if !g.hit[j] {
                        continue;
                    }
                    let nj = g.normal[j];
                    if n0[0] * nj[0] + n0[1] * nj[1] + n0[2] * nj[2] < OCCLUSION_BLUR_COS {
                        continue;
                    }
                    soma += oc[j];
                    n += 1;
                }
            }
            if n > 0 {
                #[allow(clippy::cast_precision_loss)]
                let inv = 1.0 / n as f32;
                out[i] = soma * inv;
            }
        }
    }
    out
}

/// ⭐⭐⭐ **A OCLUSÃO TRAÇADA CONTRA O CAMPO** — quanto do céu chega a cada pixel de peça.
///
/// `1,0` = o céu chega inteiro; `0,0` = a peça tapa-se a si própria por completo. Nos pixels que não
/// são peça devolve `1,0`.
///
/// # ⭐ Porque ela é ACUMULÁVEL, e é assim que ela cabe
///
/// `passagens` raios por pixel, com a sequência **deslocada por `desde`**: chamar com
/// `(0, 1)`, `(1, 1)`, `(2, 1)`… e tirar a média dá exactamente o mesmo que `(0, N)` — é isso que
/// permite ao quadro assente **refinar em passagens** em vez de pagar tudo de uma vez.
///
/// ⛔ Medido (`docs/Render3d/05` §29.2): `16` raios por pixel de uma só vez custam **`1,35 s`** a
/// `1920×1080`. Em `16` passagens de `84 ms` a imagem afina com a mão parada, que é o idioma de
/// toda viewport de render.
///
/// # ⚠️ A direcção é COSSENO-distribuída, e o deslocamento é POR PIXEL
///
/// A elevação sai de `(j + salto(pixel)) / N` (rotação de Cranley–Patterson). Sem o salto, `N = 1`
/// dispararia **um só ângulo** (45°) na imagem inteira — a sonda da §29.4 leu `5,3 %` de oclusão
/// contra os `17,3 %` verdadeiros, e isso lia-se como uma conclusão sobre AO em vez de um furo da
/// régua. *Uma estratificação que muda com o N não compara os N.*
#[must_use]
pub fn occlusion(doc: &FieldDoc, reg: &Registry, cam: &Orbit, g: &Gbuffer, total: u32) -> Vec<f32> {
    occlusion_with_reach(doc, reg, cam, g, total, OCCLUSION_REACH)
}

/// A mesma, com o alcance por parâmetro — a porta que a sonda varre.
#[must_use]
pub fn occlusion_with_reach(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    total: u32,
    reach: f32,
) -> Vec<f32> {
    let soma = occlusion_slice_with_reach(doc, reg, cam, g, 0, total, total, reach);
    if total == 0 {
        return soma;
    }
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / total as f32;
    soma.iter()
        .zip(&g.hit)
        .map(|(v, hit)| if *hit { v * inv } else { 1.0 })
        .collect()
}

/// ⭐⭐⭐ **UMA FATIA da sequência** — a SOMA da visibilidade dos raios `primeiro..primeiro+quantos`
/// de uma sequência de `total`, sem dividir.
///
/// ⚠️ **Os três números são obrigatórios e nenhum é redundante.** `total` fixa a **estratificação**
/// (que elevações a sequência inteira vai cobrir) e `primeiro`/`quantos` dizem que pedaço dela esta
/// chamada paga. *Uma API com `(desde, quantos)` só não consegue exprimir «o raio `k` de uma
/// sequência de 32»* — ela ancoraria a estratificação na FATIA, e somar fatias daria uma
/// distribuição diferente de a sequência inteira. Foi o 1.º desenho, e ele não acumulava.
///
/// ⭐ Somar todas as fatias e dividir por `total` dá **exactamente** o que [`occlusion`] devolve, e
/// há gate a prová-lo — é isso que autoriza o quadro assente a refinar em passagens.
///
/// Devolve `0,0` nos pixels que não são peça (eles não entram em soma nenhuma).
#[must_use]
pub fn occlusion_slice(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    primeiro: u32,
    quantos: u32,
    total: u32,
) -> Vec<f32> {
    occlusion_slice_with_reach(doc, reg, cam, g, primeiro, quantos, total, OCCLUSION_REACH)
}

/// A mesma, com o alcance por parâmetro — a porta que a sonda varre.
#[must_use]
#[allow(clippy::too_many_arguments)] // a peça, a vista, a fatia e o alcance
pub fn occlusion_slice_with_reach(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    primeiro: u32,
    quantos: u32,
    total: u32,
    reach: f32,
) -> Vec<f32> {
    let pixels = g.hit.len();
    let mut vis = vec![0.0f32; pixels];
    if pixels == 0 || quantos == 0 || total == 0 {
        return vis;
    }
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let (right, up, toward_eye) = cam.basis();
    let scene = Scene {
        shape: &shape,
        cam,
        basis: (right, up, toward_eye),
        sharp: Sharpness::for_frame(cam.half_extent, g.width.min(g.height) as usize),
        clip: None,
        step: ph2d_field_eval::safe_march_step(doc),
        shrink: ph2d_field_eval::field_shrink(doc, reg),
        stencil: Stencil::Tetra4,
    };
    let lift = scene.sharp.hit * crate::march::BIAS;
    let alcance = reach * cam.half_extent;

    let mut quais = Vec::new();
    let mut origens = Vec::new();
    let mut dirs = Vec::new();
    let mut cercas = Vec::new();
    for i in 0..pixels {
        if !g.hit[i] {
            continue;
        }
        let p = g.point[i];
        let nv = g.normal[i];
        // A normal do G-buffer está em VISTA e o campo vive no MUNDO — a mesma conversão do
        // [`shadow_pass`], e pela mesma razão.
        let n = [
            nv[0] * right[0] + nv[1] * up[0] + nv[2] * toward_eye[0],
            nv[0] * right[1] + nv[1] * up[1] + nv[2] * toward_eye[1],
            nv[0] * right[2] + nv[1] * up[2] + nv[2] * toward_eye[2],
        ];
        let (t1, t2) = base_do_hemisferio(n);
        let erguido = [p[0] + n[0] * lift, p[1] + n[1] * lift, p[2] + n[2] * lift];
        for j in 0..quantos {
            let (u1, u2) = sample_uv(i, primeiro + j, total);
            let r = u1.sqrt();
            let phi = std::f32::consts::TAU * u2;
            let (sp, cp) = phi.sin_cos();
            let z = (1.0 - u1).max(0.0).sqrt();
            quais.push(i);
            origens.push(erguido);
            dirs.push([
                t1[0] * r * cp + t2[0] * r * sp + n[0] * z,
                t1[1] * r * cp + t2[1] * r * sp + n[1] * z,
                t1[2] * r * cp + t2[2] * r * sp + n[2] * z,
            ]);
            cercas.push(alcance);
        }
    }
    // ⛔⛔⛔ **A PERGUNTA DA OCLUSÃO É BINÁRIA, e a 1.ª redacção usou o estimador de PENUMBRA.**
    //
    // Escrevi `hardness = 1` a raciocinar *«k = 1 é a fracção de céu que o vizinho deixa passar»*.
    // Está errado, e o gate da esfera apanhou-o: ela leu **`0,698` de céu contra `0,757` da cruz** —
    // *um corpo CONVEXO a ocluir-se mais que três cilindros cruzados.*
    //
    // O mecanismo é o mesmo da §27.5(b): numa saída RASANTE a tangente afasta-se como `d ≈ t²/2R`,
    // logo `k·d/t ≈ k·t/2R` desce abaixo de `1` **sem haver oclusor nenhum** — o estimador está a
    // ler a curvatura da própria superfície. Num raio de SOMBRA isso é penumbra e é desejável (a
    // lâmpada tem tamanho angular); aqui a pergunta é *«este raio bate em alguma coisa dentro do
    // alcance?»*, que é **sim ou não**.
    //
    // ⇒ `INFINITY` faz o termo mole nunca vincular (`vis.min(∞) = vis`), e o único sítio que
    // escreve `0` é o acerto de facto. ⚠️ Sem NaN: o braço do acerto sai por `continue` antes, logo
    // `d ≥ hit > 0` quando a multiplicação corre.
    let v = march_shadow_to(&scene, &origens, &dirs, &cercas, f32::INFINITY);
    for (j, &i) in quais.iter().enumerate() {
        vis[i] += v[j];
    }
    vis
}

/// ⭐⭐⭐ **As duas coordenadas do raio `k` do pixel `i`** — elevação e azimute.
///
/// # ⛔⛔⛔ O report do dono: *«artefatos de imagem»* (2026-09-14, foto)
///
/// Listras VERTICAIS finas, a alternar coluna a coluna. **Dois defeitos na mesma linha**, e nenhuma
/// régua minha os via:
///
/// 1. **O azimute não dependia do RAIO.** Ele saía só do pixel, logo os `32` raios de um ponto
///    partilhavam o **mesmo `φ`**: cada pixel amostrava um **LEQUE PLANO**, nunca o hemisfério. A
///    estimativa ficava enviesada por pixel — e o enviesamento era *diferente em cada pixel*.
/// 2. **O bit `0` de `i` virava o bit MAIS SIGNIFICATIVO de `u2`** (`i.reverse_bits() >> 8` devolve
///    os bits `23..0` de `i` **ao contrário**). Medido: `i = 1000 → 0,093` e `i = 1001 → 0,593` —
///    *colunas vizinhas a apontar para lados opostos.* Daí a listra ser de **uma** coluna.
///
/// ⚠️⚠️ **E a sonda da convergência era um ESPELHO:** a referência de `64` raios usava o MESMO
/// leque, logo ela media o estimador contra ele próprio e via `1/√N` bonito sobre um resultado
/// enviesado. *Uma régua que partilha a lei do produto não acusa* — a lei está escrita no
/// `CLAUDE.md` e mordeu na mesma.
///
/// # ⭐ A lei que fica
///
/// - **elevação**: estratificada sobre o `total`, com rotação de Cranley–Patterson por pixel — cada
///   passagem cai numa banda própria, e a banda desloca-se de pixel para pixel;
/// - **azimute**: sequência aditiva da **razão áurea** a partir de um arranque por pixel — ela é de
///   baixa discrepância em qualquer corte inicial, que é exactamente o que uma acumulação precisa
///   (a fatia `0..k` tem de ser boa, não só a sequência inteira).
///
/// ⚠️ **O arranque usa uma MISTURA, não `reverse_bits`:** a multiplicação de Knuth leva os bits
/// baixos de `i` aos altos do produto, logo pixels vizinhos arrancam longe um do outro **sem** o
/// acoplamento de paridade que produziu a listra.
pub(crate) fn sample_uv(i: usize, k: u32, total: u32) -> (f32, f32) {
    #[allow(clippy::cast_possible_truncation)]
    let px = i as u32;
    let mistura = |v: u32| -> f32 {
        let h = v
            .wrapping_mul(2_654_435_761)
            .rotate_left(15)
            .wrapping_mul(2_246_822_519);
        (h >> 8) as f32 / 16_777_216.0
    };
    let salto = mistura(px);
    #[allow(clippy::cast_possible_truncation)]
    let u1 = ((f64::from(k) + f64::from(salto)) / f64::from(total.max(1))).fract() as f32;
    /// O conjugado da razão áurea — a sequência aditiva de menor discrepância que há.
    const PHI: f32 = 0.618_034;
    let u2 = (mistura(px ^ 0x9E37_79B9) + k as f32 * PHI).fract();
    (u1, u2)
}

/// Dois eixos perpendiculares a `n`, sem trigonometria por amostra.
fn base_do_hemisferio(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let a = if n[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let t1 = [
        a[1] * n[2] - a[2] * n[1],
        a[2] * n[0] - a[0] * n[2],
        a[0] * n[1] - a[1] * n[0],
    ];
    let l = (t1[0] * t1[0] + t1[1] * t1[1] + t1[2] * t1[2])
        .sqrt()
        .max(1e-6);
    let t1 = [t1[0] / l, t1[1] / l, t1[2] / l];
    let t2 = [
        n[1] * t1[2] - n[2] * t1[1],
        n[2] * t1[0] - n[0] * t1[2],
        n[0] * t1[1] - n[1] * t1[0],
    ];
    (t1, t2)
}

/// ⭐⭐⭐ **O REFINAMENTO: `OCCLUSION_PASSES` passagens sobre o MESMO G-buffer.**
///
/// Cada passagem acrescenta **um raio por pixel**, publica a média acumulada por `entrega`, e pára
/// assim que ela devolver `false` — que é como a mão a volta a mexer cancela o trabalho.
///
/// ⚠️⚠️ **Ela existe como PORTA, e não como laço dentro da thread do traçado, por causa do gate.**
/// O laço vivia no `std::thread::spawn` do `smoke_draw`, onde nenhum teste lhe chega: a lei da
/// acumulação, a ordem das passagens e a paragem por cancelamento ficavam todas **inalcançáveis**.
/// *A costura não-testada é a causa nº 1 da `DIRETIVA_IMPLEMENTACAO` §1, e um laço dentro de uma
/// thread é a forma mais fácil de a produzir sem dar por isso.*
///
/// Devolve quantas passagens correram — `< OCCLUSION_PASSES` quer dizer que foi cancelado.
pub fn refine_occlusion(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    shadows: &mut Shadows,
    mut entrega: impl FnMut(&Shadows, u32) -> bool,
) -> u32 {
    let pixels = g.hit.len();
    let mut soma = vec![0.0f32; pixels];
    for k in 0..OCCLUSION_PASSES {
        let fatia = occlusion_slice(doc, reg, cam, g, k, 1, OCCLUSION_PASSES);
        for (a, f) in soma.iter_mut().zip(&fatia) {
            *a += f;
        }
        // ⚠️ **A média é sobre as passagens JÁ CORRIDAS**, e não sobre o total — senão a imagem
        // abriria preta e iria clareando, que é o contrário do que uma acumulação deve parecer.
        #[allow(clippy::cast_precision_loss)]
        let inv = 1.0 / f32::from(u16::try_from(k + 1).unwrap_or(u16::MAX));
        let cru: Vec<f32> = soma
            .iter()
            .zip(&g.hit)
            .map(|(v, hit)| if *hit { v * inv } else { 1.0 })
            .collect();
        // ⭐ **Suavizada no PUBLICAR, e não no acumulador** — a soma tem de continuar crua, senão
        // cada passagem borraria o que a anterior já borrou e a oclusão espalhar-se-ia.
        shadows.set_ambient(blur_occlusion(g, &cru));
        if !entrega(shadows, k + 1) {
            return k + 1;
        }
    }
    OCCLUSION_PASSES
}
