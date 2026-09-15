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

/// ⭐⭐⭐ **QUANTAS DIRECÇÕES TEM O CONJUNTO DE CONES** — e o recurso que ele nomeia é **o relógio
/// do quadro assente**.
///
/// # ⛔⛔⛔ O que este número deixou de significar em 2026-09-15
///
/// Até àquele dia ele eram **passagens de Monte Carlo**: raios binários sorteados por pixel, e o
/// número saía de *«onde o erro médio cai abaixo de um byte»*. ⚠️ **Essa régua media a MÉDIA de um
/// estimador com ruído, e o olho não vê médias — vê o vizinho.** O report do dono (foto do interior
/// de um furo, *«baixíssima qualidade»*) é exactamente o que aquela tabela não media.
///
/// Hoje o número é o tamanho de um conjunto **FIXO** de direcções de mundo ([`cone_dir`]), cada uma
/// traçada como CONE. Não há ruído para promediar: o que `N` compra é **resolução angular**, e o que
/// ele evita é **banda** — terraços de nível onde a resposta devia variar devagar.
///
/// # ⭐ A tabela MEDIDA (`ph2d-field-gpu`, sonda `ao_grain`, `1920×1080`, RTX 5060 Ti)
///
/// A peça é a da foto: a cruz de três cilindros com um furo passante. `ms` é o **quadro inteiro** no
/// dispositivo (marcha + sombra + oclusão + bordas); `Δ ref` é contra o mesmo estimador com `1024`
/// direcções; `bytes` é o erro **no pixel sombreado**, que é o que ele vê.
///
/// | cones | `ms` | `Δ ref` p99 | **bytes p99** | máx |
/// |---:|---:|---:|---:|---:|
/// | `16` | `22,0` | `0,0855` | `7` | `9` |
/// | `32` | `27,9` | `0,0412` | `4` | `5` |
/// | **`48`** | **`36,5`** | `0,0254` | **`2`** | `4` |
/// | `64` | `43,9` | `0,0206` | `2` | `3` |
/// | `96` | `56,0` | `0,0163` | `1` | `3` |
///
/// ⇒ **`48`, que é o joelho**: é onde o erro no pixel chega a `2` bytes — abaixo do que um degrau
/// de gradiente mostra — e onde os terraços somem à vista (`16` e `32` ainda os têm, com a imagem
/// esticada `2×`). `64` custa mais `7,4 ms` e compra `4 → 3` no máximo; `96` custa mais `19,5 ms`
/// para tirar um byte do `p99`.
///
/// ⚠️ **Só cerca de METADE das direcções corre por pixel** — o conjunto é da esfera inteira e as que
/// ficam abaixo do horizonte têm peso `0`. É esse o preço de não ter referencial tangente, e a razão
/// está em [`cone_dir`].
///
/// ⛔ **O quadro de MOVIMENTO não paga nada disto:** o dispositivo só toma o quadro **assente**
/// (`ph2d_app_field3d::gpu_frame::takes_the_frame`) e a oclusão de CPU está desligada por omissão.
///
/// ⏳ **A alavanca que fica, medida e NÃO construída:** a oclusão é de baixa frequência (é a mesma
/// premissa que legitima o [`blur_occlusion`]), logo cabe em **meia resolução** com reconstrução
/// guiada pela normal — `4×` mais barata, o que poria `96` cones abaixo do preço dos `16` raios de
/// ontem. Ela traz uma classe de artefacto própria (halo na descontinuidade de profundidade) e é
/// wave com espec própria.
pub const OCCLUSION_PASSES: u32 = 48;

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
/// # ⚠️⚠️ A RAZÃO DELA MUDOU em 2026-09-15, e a tabela abaixo é HISTÓRIA
///
/// Ela nasceu para apagar **ruído de amostragem**, e a tabela que a justificou media isso (medida
/// contra uma referência de `256` raios, com a lei de Monte Carlo que shipou até àquele dia):
///
/// | passagens | erro no pixel | **com suavização** |
/// |---:|---:|---:|
/// | `4` | `7,22 bytes` | `2,19` |
/// | `8` | `4,07` | `1,35` |
/// | **`16`** | `2,34` | **`0,89`** |
/// | `32` | `1,42` | `0,67` |
///
/// ⛔ **Esse ruído já não existe:** a oclusão é hoje determinística e há dois gates a afirmá-lo
/// (`a_oclusao_de_um_ponto_nao_depende_do_pixel_em_que_ele_cai` e o irmão da câmera). O que ela
/// suaviza agora é outra coisa — as **estrias** do conjunto discreto de direcções, que são a
/// assinatura de `48` cones em vez de infinitos.
///
/// ⚠️ **A premissa que a legitima é a MESMA** (*a oclusão é de baixa frequência dentro de uma
/// superfície*), e é por isso que ela continua; ⏳ o que **não** foi re-medido é **quanto** ela vale
/// contra a lei nova. *Uma cerca cuja razão mudou e cuja tabela não foi refeita é uma nota a
/// envelhecer — esta fica NOMEADA como tal, e não em silêncio.*
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

/// ⭐⭐⭐ **O QUE UMA FATIA DE CONES ENTREGA** — a soma pesada E o peso que a produziu.
///
/// # ⛔⛔ Porque o denominador não é a contagem de cones
///
/// O peso de um cone é `n·d`, e duas direcções do conjunto fixo contribuem com pesos **diferentes**
/// para a mesma normal. Enquanto a oclusão foram raios sorteados no hemisfério, cada amostra valia
/// o mesmo e a média era `soma / contagem` — a fatia podia devolver um número só.
///
/// ⚠️ **Com cones isso passa a MENTIR no meio do refinamento**, e foi o gate
/// `o_refinamento_nunca_abre_a_peca_preta` que o disse: as primeiras direcções do reticulado caem
/// perto do pólo `+z`, que para uma face virada a `+z` são as de **maior** cosseno, e a média
/// parcial sobre a contagem lia **`1,763`** — céu a mais do que existe. *Uma média parcial tem de
/// dividir pelo peso que já entrou, nunca por quantas vezes se entrou.*
#[derive(Clone, Debug, Default)]
pub struct ConeSlice {
    /// `Σ w·vis`, por pixel.
    pub sum: Vec<f32>,
    /// `Σ w`, por pixel — o denominador.
    pub weight: Vec<f32>,
}

impl ConeSlice {
    /// Acumula outra fatia nesta.
    pub fn add(&mut self, outra: &Self) {
        for (a, b) in self.sum.iter_mut().zip(&outra.sum) {
            *a += b;
        }
        for (a, b) in self.weight.iter_mut().zip(&outra.weight) {
            *a += b;
        }
    }

    /// A oclusão média. ⚠️ Um pixel de FUNDO — e um sem peso nenhum — lê `1,0`: céu inteiro.
    #[must_use]
    pub fn average(&self, hit: &[bool]) -> Vec<f32> {
        self.sum
            .iter()
            .zip(&self.weight)
            .zip(hit)
            .map(|((s, w), h)| if *h && *w > 0.0 { s / w } else { 1.0 })
            .collect()
    }
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
    occlusion_slice_with_reach(doc, reg, cam, g, 0, total, total, reach).average(&g.hit)
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
) -> ConeSlice {
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
) -> ConeSlice {
    let pixels = g.hit.len();
    let mut fatia = ConeSlice {
        sum: vec![0.0f32; pixels],
        weight: vec![0.0f32; pixels],
    };
    if pixels == 0 || quantos == 0 || total == 0 {
        return fatia;
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
    // ⭐⭐⭐ **A CERCA DA BOLA — o DOMÍNIO da pergunta, e o dispositivo já a tinha.**
    //
    // ⚠️ Ela viveu só no WGSL desde que o traçado foi para lá, e a paridade não a acusava porque
    // com raios BINÁRIOS ela quase nunca vincula. Com CONES vincula: o estimador é um mínimo de
    // `d/(t·cos)` ao longo do raio, logo tudo o que ele lê para lá da peça entra na resposta.
    // *Uma cerca escrita num motor só é uma LEI diferente nos dois, e a régua que a não vê é a que
    // corre com o estimador errado.*
    let bola = ph2d_field_eval::bounds::bounding_ball(doc, reg)
        .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
    let ate_sair_da_bola = |o: [f32; 3], d: [f32; 3]| -> f32 {
        let oc = [0, 1, 2].map(|c| o[c] - bola.center[c]);
        let b = oc[0] * d[0] + oc[1] * d[1] + oc[2] * d[2];
        let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - bola.radius * bola.radius;
        let disc = b * b - c;
        if disc <= 0.0 {
            return 0.0;
        }
        (-b + disc.sqrt()).clamp(0.0, alcance)
    };

    let mut quais = Vec::new();
    let mut origens = Vec::new();
    let mut dirs = Vec::new();
    let mut cercas = Vec::new();
    let mut durezas = Vec::new();
    let mut pesos = Vec::new();
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
        let cos_de = |k: u32| {
            let d = cone_dir(k, total);
            (n[0] * d[0] + n[1] * d[1] + n[2] * d[2]).max(0.0)
        };
        // ⭐ O peso acumula-se AQUI, dentro da fatia — ver [`ConeSlice`] para porque ele não pode
        // ser a contagem de cones.
        let erguido = [p[0] + n[0] * lift, p[1] + n[1] * lift, p[2] + n[2] * lift];
        for j in 0..quantos {
            let k = primeiro + j;
            if k >= total {
                break;
            }
            let c = cos_de(k);
            if c <= 0.0 {
                continue;
            }
            quais.push(i);
            origens.push(erguido);
            dirs.push(cone_dir(k, total));
            cercas.push(ate_sair_da_bola(erguido, cone_dir(k, total)));
            // ⭐⭐⭐ O CONE QUE ROÇA O PLANO TANGENTE — ver [`crate::march::march_cone_to`].
            durezas.push(1.0 / c);
            pesos.push(c);
            fatia.weight[i] += c;
        }
    }
    let v = crate::march::march_cone_to(&scene, &origens, &dirs, &cercas, &durezas);
    for (j, &i) in quais.iter().enumerate() {
        fatia.sum[i] += pesos[j] * v[j];
    }
    fatia
}

// ⛔ **RECUSA MEDIDA (2026-09-15): um PISO no cosseno não compra paridade.** A hipótese era que um
// cone quase tangente, com dureza `1/(n·d)`, amplificasse a diferença de campo entre os dois
// motores. Com o piso a `0,05` o `p99` da pior cena ficou **exactamente onde estava** (`0,0201`) ⇒
// o amplificador não é a dureza. *Uma cerca que não move a medição é ruído no código.*

/// A razão áurea — a sequência aditiva de baixa discrepância que espalha o azimute do reticulado.
const PHI: f32 = 0.618_034;

/// ⭐⭐⭐ **A direcção `k` do conjunto de cones** — reticulado de Fibonacci esférico sobre a esfera
/// INTEIRA, em coordenadas de MUNDO.
///
/// # ⛔⛔⛔ Porque ela não depende do pixel, nem da câmera, nem de um referencial tangente
///
/// O report do dono de 2026-09-14 (*«baixíssima qualidade»*, foto do interior de um furo) é **ruído
/// de estimador**: até àquele dia a oclusão eram `OCCLUSION_PASSES` raios BINÁRIOS **sorteados por
/// pixel**, e um estimador binário de `N` amostras tem desvio-padrão `√(p(1−p)/N)` — a `N = 16` isso
/// é `12,5 %` da faixa **por pixel**. Medido na peça da foto (`ph2d-field-gpu`, sonda `ao_grain`):
/// o canal de oclusão sai com uma **textura tecida** visível a olho, e o vizinho de 3×3 não a apaga.
///
/// ⚠️⚠️ **E havia um defeito PIOR que a variância: a oclusão dependia do PIXEL.** O sorteio era
/// semeado no índice do pixel, logo **o mesmo ponto da peça recebia outra oclusão ao rodar a
/// câmera** — a imagem fervia ao assentar de um ângulo diferente. Um conjunto de direcções de
/// MUNDO não tem como fazer isso, e há gate a afirmá-lo.
///
/// ⛔ **Um referencial TANGENTE foi considerado e recusado**: ele dá `N` direcções úteis em vez de
/// `N/2`, mas toda construção de base a partir de `n` tem uma **descontinuidade** (a do
/// [`base_do_hemisferio`] salta quando `|n.x|` cruza `0,9`; a de Duff salta em `n.z = 0`). Com
/// direcções SORTEADAS a descontinuidade é invisível — o sorteio já embaralha o azimute. Com um
/// conjunto FIXO ela vira uma **costura** desenhada na peça. *Uma base que só era aceitável porque
/// o ruído a escondia deixa de ser aceitável quando se apaga o ruído.*
///
/// ⭐ Com o conjunto de mundo a continuidade sai de graça: uma direcção entra no hemisfério com
/// peso `n·d → 0`, logo entra **sem degrau**.
#[must_use]
pub fn cone_dir(k: u32, total: u32) -> [f32; 3] {
    #[allow(clippy::cast_precision_loss)]
    let n = total.max(1) as f32;
    #[allow(clippy::cast_precision_loss)]
    let ki = k as f32;
    let z = 1.0 - (2.0 * ki + 1.0) / n;
    let r = (1.0 - z * z).max(0.0).sqrt();
    // ⚠️⚠️ **A FRACÇÃO vem ANTES do seno, e não é cosmética:** a `k = 47` o ângulo cru é `~182 rad`,
    // e o WGSL só garante precisão de `sin`/`cos` perto da origem — fora dela a redução de argumento
    // é do driver. *Duas implementações da mesma fórmula deixam de dar a mesma direcção, e a
    // paridade CPU↔dispositivo mede exactamente isso.*
    let (sp, cp) = (std::f32::consts::TAU * (ki * PHI).fract()).sin_cos();
    [r * cp, r * sp, z]
}

// ⛔⛔⛔ **AQUI VIVIAM O AMOSTRADOR SORTEADO E A BASE TANGENTE — as duas foram APAGADAS em
// 2026-09-15, com o report do dono na mão** (*«baixíssima qualidade»*, foto do interior de um furo).
//
// O que estava aqui era `sample_uv` — estratificação da elevação com rotação de Cranley–Patterson
// por pixel, azimute pela razão áurea — e `base_do_hemisferio`, que construía um referencial
// tangente a partir da normal. As duas serviam um estimador de **MONTE CARLO**, e o defeito não
// estava em nenhuma delas: estava em ser Monte Carlo.
//
// ⚠️ **A §31 curou um ENVIESAMENTO daquele amostrador e deixou a VARIÂNCIA de pé.** Depois da cura
// as listras foram-se e ficou o grão: `√(p(1−p)/N)` a `N = 16` é `12,5 %` da faixa por pixel, e o
// borrão de 3×3 baixa-o para `~4 %` — que num furo ESCURO, depois da curva sRGB, é a dezena de
// níveis que ele fotografou. *A cura de um enviesamento não é a cura da variância, e as duas
// leem-se como «a imagem melhorou».*
//
// ⛔ E havia um segundo defeito que nenhuma régua desta crate media: o sorteio era semeado no
// **índice do pixel**, logo o mesmo ponto da peça mudava de oclusão ao rodar a câmera.
//
// ⇒ a lei que ficou é o CONE determinístico ([`cone_dir`]), e estas duas funções não têm consumidor
// nenhum. *Um amostrador sem estimador que o use é código que ninguém pode acordar.*

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
    let mut acc = ConeSlice {
        sum: vec![0.0f32; pixels],
        weight: vec![0.0f32; pixels],
    };
    for k in 0..OCCLUSION_PASSES {
        acc.add(&occlusion_slice(doc, reg, cam, g, k, 1, OCCLUSION_PASSES));
        // ⚠️ **A média é sobre o PESO já acumulado**, e não sobre o total nem sobre a contagem de
        // passagens — senão a imagem mudaria de nível a cada passo em vez de afinar. Ver
        // [`ConeSlice`], que é onde essa lei vive.
        let cru = acc.average(&g.hit);
        // ⭐ **Suavizada no PUBLICAR, e não no acumulador** — a soma tem de continuar crua, senão
        // cada passagem borraria o que a anterior já borrou e a oclusão espalhar-se-ia.
        shadows.set_ambient(blur_occlusion(g, &cru));
        if !entrega(shadows, k + 1) {
            return k + 1;
        }
    }
    OCCLUSION_PASSES
}
