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

    Shadows { per_lamp, pixels }
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
