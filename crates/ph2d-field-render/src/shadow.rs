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

use crate::march::{Scene, march_shadow_saindo};
use crate::{Gbuffer, Ground, Orbit, Sharpness, Stencil};
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
    /// ⭐⭐⭐ **A OUTRA METADE do integral do hemisfério: a luz que a CENA devolve** (a `W5`, ver
    /// [`crate::bounce`]).
    ///
    /// O [`Self::ambient`] mede a parte do CÉU e **atenua**; esta é a parte das SUPERFÍCIES e
    /// **soma**. As duas saem do mesmo conjunto de direcções e do mesmo peso `max(0, n·d)` — dois
    /// conjuntos diferentes fariam a soma deixar de ser o integral de coisa nenhuma.
    ///
    /// ⚠️⚠️ **Fora de alcance devolve `[0, 0, 0]`, que é o OPOSTO da lei das irmãs** — e a razão é
    /// que a pergunta é outra. Uma sombra que não foi calculada é *ausência de sombra* (`1,0`);
    /// uma luz que não foi calculada é **ausência de luz**. *Inventar luz é a única das duas que
    /// acende o que devia estar escuro.*
    bounce: Vec<[f32; 3]>,
    pixels: usize,
    /// ⭐⭐⭐ **O CHÃO para o qual os pixels de FUNDO foram calculados** — ver [`crate::ground`].
    ///
    /// ⚠️ Com ele, um pixel que **falha** a peça e vê o chão guarda nos mesmos canais a luz que chega
    /// **ao chão** — as lâmpadas e o céu. *O canal responde sempre «quanto desta fonte chega ao que
    /// este pixel mostra»*, e o que um pixel de fundo mostra passa a ser o chão.
    ///
    /// ⚠️ **`None` é o caminho de sempre, ao bit**: os pixels de fundo guardam `1,0` e o pintor
    /// copia os bytes do fundo. O pintor lê o chão DAQUI, e não de um argumento ao lado: a altura a
    /// que os canais foram calculados e a altura a que se pinta são a MESMA por construção.
    ground: Option<Ground>,
    /// ⭐⭐⭐ **A COR QUE A PEÇA DEVOLVE AO CHÃO** — ver [`crate::ground_bounce`].
    ///
    /// ⚠️⚠️ **Ele é um CAMPO e não um canal por pixel**, ao contrário de todos os vizinhos desta
    /// struct, e a razão é medida: o chão é um PLANO, logo a resposta dele é função de `(x, z)` e
    /// **não** da câmera. *Um canal por pixel voltaria a ser assado a cada movimento da vista para
    /// devolver exactamente os mesmos números.*
    ///
    /// ⚠️ **Vazio é o caminho de sempre, ao bit** — a consulta devolve `[0,0,0]` e o pintor soma
    /// zero.
    ground_bounce: crate::ground_bounce::GroundBounce,
}

impl Shadows {
    /// O chão que os pixels de fundo destes canais recebem — ver o campo.
    #[must_use]
    pub fn ground(&self) -> Option<Ground> {
        self.ground
    }

    /// O campo da luz que a peça devolve ao chão — ver o campo [`Shadows::ground_bounce`].
    #[must_use]
    pub fn ground_bounce(&self) -> &crate::ground_bounce::GroundBounce {
        &self.ground_bounce
    }

    /// ⭐ Declara o campo do chão — para quem o assou (o [`crate::ground_bounce::bake_ground_bounce`]).
    pub fn set_ground_bounce(&mut self, campo: crate::ground_bounce::GroundBounce) {
        self.ground_bounce = campo;
    }

    /// Declara o chão destes canais — para quem os calculou noutro sítio (o traçador de GPU).
    pub fn set_ground(&mut self, ground: Option<Ground>) {
        self.ground = ground;
    }

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

    /// A luz que a CENA devolve ao pixel `i` — ver o campo [`Shadows::bounce`].
    ///
    /// ⚠️ **Fora de alcance devolve `[0, 0, 0]`** — ausência de luz, nunca luz inventada. É o que
    /// mantém o quadro sem esta passagem **byte a byte** o de sempre.
    #[must_use]
    pub fn bounce_at(&self, i: usize) -> [f32; 3] {
        self.bounce.get(i).copied().unwrap_or([0.0; 3])
    }

    /// ⭐ Declara o canal do ricochete — para quem o calculou (o [`crate::bounce::bounce_pass`], ou
    /// o traçador de GPU quando ele o souber fazer).
    pub fn set_bounce(&mut self, bounce: Vec<[f32; 3]>) {
        if self.pixels == 0 {
            self.pixels = bounce.len();
        }
        self.bounce = bounce;
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
    shadow_pass_on(doc, reg, cam, g, lamps_world, None)
}

/// ⭐⭐⭐ **O mesmo passe, com o CHÃO** — os pixels de fundo que o vêem recebem a sombra que cai nele.
///
/// ⚠️ **As mesmas três cercas**, com a normal do chão ([`crate::GROUND_UP`]) no lugar da do pixel: uma
/// lâmpada abaixo do chão não o ilumina, o raio pára na lâmpada e na saída da bola. E uma quarta,
/// só do chão: **um raio que nem toca a bola não marcha** — ali a peça não pode tapar nada, e o chão
/// é quase toda a imagem. O dispositivo faz o mesmo salto, e é isso que mantém os dois motores no
/// mesmo número (um raio marchado com cerca `0` ainda avalia o campo uma vez).
#[must_use]
pub fn shadow_pass_on(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    lamps_world: &[[f32; 3]],
    ground: Option<Ground>,
) -> Shadows {
    let pixels = g.hit.len();
    let chao = crate::ground::ground_points(cam, g, ground);
    // ⭐ **O céu do chão sai daqui, e não do refinamento** — ver [`crate::ground::ground_sky`]. Nos
    // pixels de peça ele lê `1,0`, que é exactamente o que um canal vazio significa.
    let ambient = if chao.is_empty() {
        Vec::new()
    } else {
        crate::ground::ground_sky(doc, reg, cam, &chao)
    };
    if lamps_world.is_empty() || pixels == 0 {
        return Shadows {
            per_lamp: Vec::new(),
            ambient,
            // ⚠️ Vazio: este passe não o calcula. Quem o quiser chama o
            // [`crate::bounce::bounce_pass`] e declara-o com o [`Shadows::set_bounce`] — é o mesmo
            // desenho do `ambient`, que também vem de outro passe.
            bounce: Vec::new(),
            pixels,
            ground,
            ground_bounce: crate::ground_bounce::GroundBounce::vazio(),
        };
    }
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let (right, up, toward_eye) = cam.basis();
    // ⭐ A base resolve-se UMA vez por passe — ver o doc dela.
    let base = crate::shade_render::ViewBasis::of(cam);
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
            let mut sair = Vec::new();
            let mut origens = Vec::new();
            let mut dirs = Vec::new();
            let mut cercas = Vec::new();
            for i in 0..pixels {
                if !g.hit[i] {
                    // ⭐ **O CHÃO que este pixel de fundo vê**, se algum — ver a nota do passe.
                    let Some(q) = chao.get(i).copied().flatten() else {
                        continue;
                    };
                    let d = [luz[0] - q[0], luz[1] - q[1], luz[2] - q[2]];
                    let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    // A normal do chão é `+y`: `N·L > 0` é a lâmpada estar ACIMA dele.
                    if dist <= f32::EPSILON || d[1] <= 0.0 {
                        continue;
                    }
                    let dir = [d[0] / dist, d[1] / dist, d[2] / dist];
                    // ⛔⛔ **A bola ALARGADA pela penumbra, e não a bola** — ver [`cerca_com`].
                    let ate = cerca_com(bola.as_ref(), dist / HARDNESS, q, dir, dist);
                    if ate <= 0.0 {
                        continue;
                    }
                    quais.push(i);
                    // ⚠️ O chão é um plano e o raio dele parte POR CIMA: nunca está dentro de nada.
                    sair.push(false);
                    origens.push([q[0], q[1] + lift, q[2]]);
                    dirs.push(dir);
                    cercas.push(ate);
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
                let nm = base.view_to_world(g.normal[i]);
                // ⭐⭐⭐ **De costas para a luz: o raio PARTE, e só conta o que estiver depois de
                // ele SAIR do próprio corpo.**
                //
                // ⛔⛔ **Aqui esteve um `continue` que escrevia `1,0`, e o comentário dele previa
                // por escrito o dia em que isso mordesse** — *«até ao dia em que alguém ler este
                // canal para outra coisa (a OpenPBR tem termos que recebem luz com `N·L < 0`)»*. O
                // dia foi 2026-09-18: a subsuperfície MACIÇA é o primeiro consumidor desta casa que
                // lê luz do lado escuro, e com `1,0` ali a sombra que um vizinho projecta era
                // **TRUNCADA** no terminador — a linha dura que o dono fotografou (medido: `+4,4`
                // bytes num pixel, p99 da quebra `9,21` contra `1,00` sem sombra).
                //
                // ⛔ **E a cura NÃO é marchar o raio como os outros:** ele atravessaria o próprio
                // corpo e a metade de parede FINA — a folha com o sol atrás, que é a razão de ser
                // da wave — apagava-se (`83,7 → 73,8` de luminância média, `vis` mínima `0,000`,
                // medido). *A pergunta certa não é «a minha peça está no caminho?», é «há mais
                // alguma coisa no caminho?»* ⇒ [`crate::march::march_shadow_saindo`].
                let de_costas = nm[0] * dir[0] + nm[1] * dir[1] + nm[2] * dir[2] <= 0.0;
                quais.push(i);
                sair.push(de_costas);
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
                //
                // ⚠️⚠️ **E quem está de costas parte do PONTO, sem erguer.** O ergue existe para o
                // raio se afastar da superfície; de costas ele tem de ENTRAR, e um raio erguido que
                // roça o terminador pode nunca tocar no corpo — nunca sairia, nunca acusaria, e a
                // sombra do vizinho voltava a ser truncada exactamente no pixel que se está a curar.
                let erguido = if de_costas {
                    p
                } else {
                    [
                        p[0] + nm[0] * lift,
                        p[1] + nm[1] * lift,
                        p[2] + nm[2] * lift,
                    ]
                };
                origens.push(erguido);
                dirs.push(dir);
                cercas.push(cerca(bola.as_ref(), p, dir, dist));
            }
            let v = march_shadow_saindo(&scene, &origens, &dirs, &cercas, HARDNESS, &sair);
            for (j, &i) in quais.iter().enumerate() {
                vis[i] = v[j];
            }
            vis
        })
        .collect();

    Shadows {
        per_lamp,
        ambient,
        bounce: Vec::new(),
        pixels,
        ground,
        ground_bounce: crate::ground_bounce::GroundBounce::vazio(),
    }
}

/// Onde o raio pode parar: na lâmpada, ou na saída da bola — o que vier primeiro.
fn cerca(
    bola: Option<&ph2d_field_eval::bounds::Ball>,
    p: [f32; 3],
    dir: [f32; 3],
    ate_a_luz: f32,
) -> f32 {
    cerca_com(bola, 0.0, p, dir, ate_a_luz)
}

/// Onde o raio pode parar, com a bola ALARGADA por `folga` — a cerca de um raio que parte do CHÃO
/// usa `folga = distância à luz / HARDNESS`.
///
/// ⛔⛔ **A bola simples cortava a penumbra numa elipse DURA** (medido 2026-09-16, a cruz pousada: um
/// disco com a borda preta à volta da sombra). O estimador escurece um raio que passa a menos de
/// `t/k` da peça — e um raio do chão viaja `t ~ 1` até lá, logo a penumbra estende-se `~0,12` para
/// fora da bola. Um raio que só passava a raspar a ponta de um cilindro, fora da bola, não marchava
/// e lia `1`; o vizinho que a tocava marchava e lia `0,4`.
///
/// ⭐ **Com a folga a cerca é EXACTA num campo de distância**: a distância à peça é pelo menos a
/// distância à bola, logo um raio que fica a mais de `t_máx/k` dela nunca lê abaixo de `1`.
///
/// ⚠️ **Os pixels de PEÇA ficam com a bola simples, e não por economia**: ali ela é o que impede o
/// estimador de ler a superfície de onde o raio saiu (a acne da esfera, ver a nota do módulo). Um
/// ponto do chão não está sobre superfície nenhuma.
fn cerca_com(
    bola: Option<&ph2d_field_eval::bounds::Ball>,
    folga: f32,
    p: [f32; 3],
    dir: [f32; 3],
    ate_a_luz: f32,
) -> f32 {
    let Some(b) = bola else {
        return ate_a_luz;
    };
    let raio = b.radius + folga;
    let oc = [p[0] - b.center[0], p[1] - b.center[1], p[2] - b.center[2]];
    let bb = oc[0] * dir[0] + oc[1] * dir[1] + oc[2] * dir[2];
    let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - raio * raio;
    let disc = bb * bb - c;
    if disc <= 0.0 {
        // O raio nem toca a bola: nada o pode tapar. ⚠️ `0` e não `ate_a_luz` — marchar seria
        // perguntar a um campo que já se sabe vazio.
        return 0.0;
    }
    (-bb + disc.sqrt()).max(0.0).min(ate_a_luz)
}
