//! ⭐⭐⭐ **O PASSE QUE PINTA** — do G-buffer que ficou no dispositivo aos bytes que a tela recebe.
//!
//! # ⛔⛔ Porque ele existe: o barramento, medido
//!
//! Até aqui o dispositivo marchava e **devolvia o G-buffer**: `t`, normal, sombra e oclusão por
//! pixel, `49,8 MB` a `1920×1080`. A CPU reconstruía o ponto de cada pixel, resolvia o material,
//! corria o OpenPBR, aplicava o olhar e escrevia os bytes — `36` dos `61 ms` do quadro assente.
//!
//! ⇒ as três leis do pintor já atravessaram ([`ph2d_material::wgsl`], o céu do chamador e o
//! [`ph2d_view_transform::wgsl`]) e a quarta é a do **dono** ([`ph2d_field_eval::owners::wgsl`]).
//! Com as quatro no dispositivo, o que volta é a **imagem**: `8,3 MB`.
//!
//! # ⚠️ Ele lê o grupo `0` da marcha e não o refaz
//!
//! O centro, a luz e a lista de bordas já lá estão. O pintor acrescenta um **grupo `1`** com o que
//! só ele lê — o céu, as tabelas, os materiais e a saída. *Uma segunda declaração do centro seria a
//! segunda resposta à mesma pergunta, e ela divergiria no dia em que um dos dois mudasse de
//! formato.*
//!
//! # ⚠️⚠️ As DUAS aproximações da borda são as da CPU, declaradas
//!
//! Um pixel de borda tem quatro sub-amostras marchadas, e delas guarda-se a **normal** — não o
//! ponto. ⇒ o **ponto** e o **material** do centro servem às quatro, exactamente como o
//! [`ph2d_field_render::shade_render`] faz. Numa silhueta entre duas peças de cores diferentes isto
//! pinta a borda com a cor da que o centro apanhou.

use ph2d_field_eval::owners::{Owners, wgsl::OwnersWgsl};

/// Tudo o que o pintor precisa e que **não** sai da marcha.
pub struct PaintSetup<'a> {
    /// A lei do dono desta peça. `None` numa peça de UMA folha — ver [`Owners`].
    pub owners: Option<&'a Owners>,
    /// Um material por folha, empacotado com o [`ph2d_material::wgsl::pack`] e concatenado.
    ///
    /// ⚠️ **A ordem é a das folhas do [`Owners`]**, e quem constrói um constrói o outro: uma lista
    /// com outra ordem pinta cada peça com a cor da vizinha, sem erro nenhum.
    pub materials: &'a [f32],
    /// O céu de quem chama, que tem de declarar **exactamente**:
    ///
    /// ```wgsl
    /// fn ceu_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32>
    /// fn ceu_irradiance(n: vec3<f32>) -> vec3<f32>
    /// ```
    ///
    /// ⚠️ **`ceu_*` e não `env_*`, desde o ricochete** (`docs/Render3d/08` §12): quem preenche o
    /// [`ph2d_material::wgsl::ENV_SLOT`] é o [`ambiente`] desta crate, e o céu é **um dos dois
    /// braços** dele — o outro é a luz que as superfícies devolvem.
    ///
    /// ⚠️ **Ele vem de fora de propósito:** o céu do produto vive na família `field3d`, que é
    /// composição, e esta crate não desenha estúdio nenhum.
    pub env_source: &'a str,
    /// O uniforme que o [`Self::env_source`] lê.
    pub env_consts: &'a [f32],
    /// O armazém que o [`Self::env_source`] lê.
    pub env_tables: &'a [f32],
    /// ⭐⭐⭐ **O BRILHO da cena** (`docs/Render3d/12`) — o gémeo do que a cauda do sombreamento de
    /// CPU lê. Desligado, nada da cadeia é criado e o quadro é o de sempre **ao bit**.
    pub bloom: ph2d_bloom::Bloom,
    /// A radiância que cada lâmpada entrega a **uma** unidade de distância — o
    /// [`ph2d_field_render::PointLamp::radiance_at_one`]. As posições são as
    /// [`crate::trace::MarchSetup::lamps`], e as duas listas **têm de ter o mesmo comprimento e a
    /// mesma ordem**: quem monta uma monta a outra.
    pub lamp_radiance: [[f32; 3]; crate::trace::MAX_LAMPS],
    /// ⭐ **Quantas direcções o RICOCHETE percorre** — o mesmo `ao_rays` da marcha
    /// (`docs/Render3d/08`). `0` não compila nem despacha a passagem, e o canal fica vazio.
    ///
    /// ⚠️ **Ele vem do chamador e não do uniforme** porque quem decide COMPILAR um pipeline é o
    /// Rust, e o uniforme só é lido dentro do shader.
    pub ao_rays: u32,
    /// ⭐⭐⭐ **Este quadro tem BORDA MOLE?** (`docs/Render3d/10` §12) — o gémeo do
    /// [`crate::trace::MarchSetup::mole`], do lado de quem COMPILA.
    ///
    /// ⚠️ **Ele vem do chamador e não do uniforme**, pela mesma razão que o `ao_rays`: quem decide
    /// compilar e despachar um pipeline é o Rust, e o uniforme só é lido dentro do shader. ⛔ Os dois
    /// têm de concordar — um `true` aqui com o `mole` do `MarchSetup` a `None` despacha duas
    /// passagens sobre slots que o buffer não tem.
    pub mole: bool,
    /// ⭐⭐⭐⭐ **ALGUMA COISA NESTE QUADRO LÊ O CAMPO DA PEÇA DENTRO DO PINTOR?**
    ///
    /// Medido 2026-09-21 sobre o grafo de chamadas do WGSL (`docs/Render3d/03` §W9): das seis
    /// entradas deste passe, o `pinta` e o `pinta_bordas` alcançam a fita da peça **por UM caminho
    /// só** — a CURVATURA (`curvatura_em` / `curvatura_do_estilo_em`) —, o `assa_sondas` alcança-a
    /// pela marcha, e o `pinta_ricochete` e as duas metades da borda mole **não a alcançam de todo**.
    ///
    /// ⇒ quando nada lê a curvatura e o ricochete está desligado, **o pintor não precisa da fita**,
    /// e a [`pinta`] entrega-lhe uma inerte. O texto do shader deixa de levar a peça, logo deixa de
    /// mudar quando o artista acrescenta uma forma — e o cache, que tem por chave o TEXTO, acerta.
    /// **Medido: acrescentar uma forma passa de `1 406 ms` para `74 ms`.**
    ///
    /// ⚠️ **A imagem é byte-idêntica por CONSTRUÇÃO, não por promessa:** as chamadas a `field()` que
    /// sobram no texto estão atrás das guardas que este predicado descreve, logo nunca correm.
    ///
    /// ⚠️ **Ele vem do chamador e não do uniforme**, pela mesma razão que o [`Self::ao_rays`] e o
    /// [`Self::mole`]: quem decide COMPILAR um pipeline é o Rust. E é o **mesmo predicado** que o
    /// caminho de referência já usa para decidir se assa os canais da curvatura
    /// (`ph2d_field_render::curvatura::assar_canais`): `Surface::reads_curvature` ∪
    /// `Presentation::reads_curvature`. *Duas respostas à mesma pergunta divergiriam no dia em que
    /// um terceiro consumidor da curvatura nascesse.*
    pub le_o_campo: bool,
    /// A exposição, em paragens.
    pub stops: f32,
    /// A vista, no código do [`ph2d_view_transform::wgsl::view_code`].
    pub view: u32,
    /// Os bytes EXACTOS que um pixel de fundo recebe — copiados, nunca reconvertidos.
    pub background: [u8; 4],
    /// A largura em MUNDO da fronteira entre dois materiais — ver o `BOUNDARY_PIXELS` do
    /// [`ph2d_field_render::shade_render`], que é quem a deriva.
    pub pixel_world: f32,
    /// ⭐⭐⭐ **O PASSO da segunda diferença que dá a CURVATURA** — o
    /// [`ph2d_field_render::curvatura::eps_para`], e nunca o da normal: uma primeira diferença
    /// divide por `ε` e uma segunda por `ε²`, logo o cancelamento em `f32` entra `1/ε` vezes mais
    /// cedo (com o passo da normal uma face plana lia curvatura `1,49`).
    ///
    /// ⚠️ Ele **vem da CPU** e não se deriva aqui pela razão de sempre: duas respostas à mesma
    /// pergunta divergem, e a paridade desta linha é `100,000 %`.
    pub curv_eps: f32,
    /// ⭐⭐⭐ **O PASSO da curvatura que o ESTILO lê** — `softness × raio_da_peça`, calculado pela
    /// [`ph2d_field_render::Presentation::curvature_eps`].
    ///
    /// ⛔⛔ **Ele é CALCULADO na CPU e LIDO aqui, e não derivado no shader** — ver
    /// [`ph2d_style::wgsl::EPS_DO_ESTILO`], onde a bissecção está: derivá-lo lá dentro custava `2`
    /// bytes em `4 519` píxeis do miolo, **não** por a aritmética ser diferente, mas por o TEXTO da
    /// função mudar e a placa contrair `a*b + c` de outra maneira.
    pub curv_eps_estilo: f32,
    /// ⭐⭐⭐ **A CAMADA DE ESTILO da cena** (`docs/Render3d/03`, a `W8`) — os botões da direcção de
    /// arte, que entram entre a física e o olhar.
    ///
    /// ⚠️ **Ela viaja como a STRUCT e não como floats**, porque quem a arruma é a porta do
    /// dispositivo ([`ph2d_style::wgsl::pack`]), que também **saneia**. Um `[f32; 20]` aqui poria o
    /// chamador a lembrar-se de duas coisas, e *o dispositivo não pode receber um bloco sujo por
    /// alguém se ter esquecido de uma chamada.*
    ///
    /// Com o estilo de fábrica o quadro é o de antes, **ao bit** — por construção (ver
    /// [`ph2d_style`]) e com gate de paridade contra a referência de CPU.
    pub style: ph2d_style::Style,
    /// ⭐⭐ **O raio da bola que envolve a PEÇA** — o que torna a curvatura adimensional antes de
    /// chegar ao estilo. Ver [`ph2d_field_render::Presentation::piece_radius`].
    ///
    /// ⛔ **Ele NÃO se deriva do [`PaintSetup::curv_eps`]**, apesar de aquele sair deste: seria a
    /// segunda resposta à mesma pergunta, e a que passa a mentir no dia em que a fracção do
    /// `eps_para` mudar.
    pub piece_radius: f32,
    /// ⭐⭐⭐ **A difusa BRANCA com que o CHÃO mede a luz** (`docs/Render3d/07`) — a
    /// [`ph2d_field_render::catcher_surface`], empacotada como as outras.
    ///
    /// ⚠️ Ela viaja **depois** dos materiais da peça, no índice `materiais`, e o guarda do
    /// [`PaintSetup::materials`] não a alcança de propósito: ela não é o material de folha nenhuma.
    pub catcher: &'a [f32],
    /// ⭐⭐⭐ **A COR QUE A PEÇA DEVOLVE AO CHÃO** — o campo 2D do
    /// [`ph2d_field_render::ground_bounce`]. Vazio = o caminho de sempre, **ao bit**.
    ///
    /// # ⛔⛔ Porque ele é assado na CPU e ENVIADO, ao contrário das sondas
    ///
    /// As sondas da peça têm kernel próprio (`assa_sondas`) porque custam `8,4 M` raios. Este campo
    /// custa `131 072` — **`1,6 %`** — e o número que decidiu foi o RELÓGIO da assadura de CPU:
    ///
    /// | | num lote só | em lotes paralelos |
    /// |---|---:|---:|
    /// | `48² × 128` (a grelha de então) | `106,5 ms` | **`11,8 ms`** (a `load 82`) |
    /// | **`32² × 128`** (a que shipa) | — | **`5,33 ms`** (a `load 21`) |
    ///
    /// ⇒ com o lote repartido ele cabe onde o quadro assente já espera, e **compra UMA lei em vez
    /// de duas**: não há kernel de assadura em WGSL para divergir do da CPU, logo não há paridade
    /// de ASSADURA para falhar — só a da CONSULTA, que é uma bilinear.
    ///
    /// ⏳ **O kernel próprio fica NOMEADO com o preço:** na placa ele valeria `~0,04 ms`, e a cache
    /// por cena-e-luz vale mais (o campo **não depende da câmera**, logo orbitar reutiliza-o).
    pub ground_bounce: &'a ph2d_field_render::ground_bounce::GroundBounce,
    /// ⭐⭐⭐ **AS GÉMEAS FOSCAS** — os mesmos materiais de [`PaintSetup::materials`], na MESMA
    /// ordem, sem o lóbulo especular ([`ph2d_material::Surface::matte`]).
    ///
    /// ⚠️⚠️ **Elas viajam empacotadas e não se derivam aqui**, e a razão é que o pacote é
    /// **preparado**: o `specular_weight` entra no `modulated_eta_s`, no `main_alpha` e no
    /// escurecimento da base, e zerar o campo `a` do `emission_specweight` no shader **não** é a
    /// mesma superfície que a `matte()` prepara. *Uma segunda derivação da mesma lei, num shader, é
    /// como as duas divergem no dia em que o OpenPBR ganhar uma camada.*
    ///
    /// Vazia = o caminho de antes, ao bit: o ricochete lê o material de sempre.
    pub matte: &'a [f32],
}

/// ⭐⭐⭐ **QUANTOS ARMAZÉNS O PASSE QUE PINTA LIGA** — seis do grupo `0` e três do grupo `1`.
///
/// ⚠️ **O piso garantido da `wgpu` é `8`**, e é por isso que este número é público: quem não o
/// tiver cai na CPU em vez de ver a `wgpu` recusar o layout a meio de um quadro.
pub const ARMAZENS: u32 = 9;

/// Os buffers do grupo `0`, que a marcha já criou e escreveu.
pub(crate) struct Alvos<'a> {
    /// ⭐ **As LEIS da marcha, com as esculturas já substituídas** — o pintor marcha o ricochete
    /// com elas (`docs/Render3d/08` §12), e quem as compõe é quem também compõe o molde da marcha:
    /// *a origem que o texto indexa e a ordem dos vectores são a MESMA decisão.*
    pub leis: &'a str,
    /// A fita do campo — a mesma que a marcha compilou, pela mesma razão.
    pub fita: &'a ph2d_field_eval::wgsl::TapeWgsl,
    pub bgl: &'a wgpu::BindGroupLayout,
    /// As grades das esculturas — o pintor lê-as pela mesma lei que a marcha.
    pub grades: &'a wgpu::Buffer,
    pub setup: &'a wgpu::Buffer,
    pub k: &'a wgpu::Buffer,
    pub centro: &'a wgpu::Buffer,
    pub luz: &'a wgpu::Buffer,
    pub conta: &'a wgpu::Buffer,
    pub borda: &'a wgpu::Buffer,
}

/// ⭐⭐⭐ **O CAMPO QUE NINGUÉM CHAMA** — o corpo que substitui a fita da peça quando nada neste
/// quadro a lê (ver [`PaintSetup::le_o_campo`]).
///
/// ⚠️ **Ele tem de existir e não tem de responder:** as chamadas a `field()` que sobram no texto
/// estão todas atrás de guardas que não abrem, e o que este corpo compra é o texto do shader deixar
/// de mudar com a peça. *Uma constante compila em nada e o cache passa a acertar.*
const FITA_INERTE: &str = "fn field(p: vec3<f32>) -> f32 { return 1.0; }\n";

/// ⛔ **A lei do dono numa peça de UMA folha** — a mesma resposta que o `Surfaces::owners: None` dá
/// na CPU, e não um caso especial: não perguntar é exactamente o custo zero.
const DONO_DE_UMA_FOLHA: &str = r"
struct Dono { a: u32, b: u32, t: f32 };
fn dono_mix(p: vec3<f32>, width: f32) -> Dono { return Dono(0u, 0u, 0.0); }
";

/// ⭐ **O corpo do shader vive no irmão** ([`super::paint_wgsl`]) — ver o cabeçalho dele.
use crate::paint_wgsl::PINTOR;
use crate::paint_wgsl_mole::PINTOR_MOLE;
use crate::paint_wgsl_sondas::PINTOR_SONDAS;

/// ⭐⭐⭐ **O texto do pintor, composto** — as quatro leis mais o corpo.
///
/// ⚠️ **A ordem é a que o WGSL precisa para o `struct Ceu` existir antes do binding que o nomeia.**
/// O material traz a ranhura do ambiente já preenchida pelo céu de quem chama.
pub(crate) fn fonte(
    pintor: &PaintSetup<'_>,
    lei_do_dono: Option<&OwnersWgsl>,
    leis: &str,
) -> String {
    let dono = lei_do_dono.map_or(DONO_DE_UMA_FOLHA, |l| l.source.as_str());
    let material = ph2d_material::wgsl::SOURCE
        .replace(ph2d_material::wgsl::ENV_SLOT, &ambiente(pintor.env_source));
    // ⚠️ **As TRÊS metades são UM shader** — o corte é o tecto de LOC, e a concatenação é onde ele
    // deixa de se ver. Ver o cabeçalho do [`crate::paint_wgsl`] e o do [`crate::paint_wgsl_mole`].
    // ⛔ Eram duas até 2026-09-19; a `W10` trouxe a terceira, e há gate a exigir que ela seja
    // JUNTA (`o_gemeo_da_borda_mole_esta_ligado_no_dispositivo`): *um fragmento declarado que o
    // `format!` não junta compila e não chega ao shader.*
    let corpo = format!("{PINTOR}{PINTOR_SONDAS}{PINTOR_MOLE}")
        .replace(
            "{BLUR_COS}",
            &formata(ph2d_field_render::OCCLUSION_BLUR_COS),
        )
        .replace(
            "{PISO_LUZ}",
            &formata(ph2d_field_render::POINT_LAMP_MIN_DISTANCE),
        )
        .replace("{PACKED}", &ph2d_material::wgsl::PACKED.to_string())
        // ⚠️ **Lida do sítio que a declara** ([`crate::paint_wgsl::CURVATURA`]), porque ela tem um
        // SEGUNDO leitor: o instrumento que mede a curvatura nos dois motores. *Uma lei com dois
        // leitores não se escreve duas vezes.*
        // ⛔⛔⛔ **DUAS funções do MESMO molde, e a razão é MEDIDA** (2026-09-19): quando o `ε`
        // passou a ser ARGUMENTO, a imagem do dispositivo divergiu da referência em **2–3 bytes
        // sobre `4 519` píxeis do MIOLO**. ⚠️ **Não era a lei nem o `ε`** — bissectado: a divergência
        // é **idêntica** com o ganho antigo e com a suavidade no piso. O que muda é o TEXTO da
        // função: a placa **contrai `a*b + c` num `fma`** de outra maneira, e a segunda diferença
        // amplifica essa última casa por `1/(4ε²)`.
        //
        // ⚠️⚠️ **A recusa que o doc daquela const escrevia tinha uma razão REAL e enunciava OUTRA:**
        // ela dizia *«mudaria o texto do produto para servir o instrumento»*, e o perigo não era o
        // instrumento — era **a aritmética da placa mudar com o texto**. ⇒ o molde dá à medição do
        // MATERIAL um corpo byte-idêntico ao de ontem (`let e = pintor.knobs.z;`) e ao ESTILO o
        // mesmo corpo com outra primeira linha. *Uma lei, dois leitores, e nenhum texto novo no
        // caminho que já shipava.*
        .replace("{CURVATURA}", &{
            let molde = |nome: &str, eps: &str| {
                crate::paint_wgsl::CURVATURA
                    .replace("{NOME}", nome)
                    .replace("{EPS}", eps)
            };
            format!(
                "{}{}",
                molde("curvatura_em", "pintor.knobs.z"),
                molde("curvatura_do_estilo_em", "pintor.estilo.extra.z"),
            )
        })
        // ⚠️ **Lido do ficheiro que o declara**, nunca transcrito — a mesma lei do `{BLUR_COS}`.
        .replace(
            "{FADE}",
            &formata(ph2d_field_render::ground_bounce::GROUND_BOUNCE_FADE),
        )
        .replace(
            "{PROBE_GRID}",
            &ph2d_field_render::probes::PROBE_GRID.to_string(),
        )
        .replace(
            "{PROBE_DIRS}",
            &ph2d_field_render::probes::PROBE_DIRS.to_string(),
        )
        .replace(
            "{PROBE_MARGIN}",
            &formata(ph2d_field_render::probes::PROBE_MARGIN),
        )
        // ⚠️ O MESMO f32 que a CPU calcula em tempo de execução — `4π/N` em `f32`.
        .replace(
            "{SH_PESO}",
            &formata(4.0 * std::f32::consts::PI / ph2d_field_render::probes::PROBE_DIRS as f32),
        )
        .replace(
            "{SH_A1}",
            &formata(ph2d_field_render::probes::SH_COSSENO[1]),
        )
        .replace(
            "{SH_A2}",
            &formata(ph2d_field_render::probes::SH_COSSENO[4]),
        )
        .replace("{SQRT3}", &formata(3.0f32.sqrt()))
        .replace("{MAX_LAMPS}", &crate::trace::MAX_LAMPS.to_string())
        .replace("{LUMA_R}", &formata(ph2d_field_render::GROUND_LUMA[0]))
        .replace("{LUMA_G}", &formata(ph2d_field_render::GROUND_LUMA[1]))
        .replace("{LUMA_B}", &formata(ph2d_field_render::GROUND_LUMA[2]))
        // ⭐ O MESMO tecto de raio que a CPU clampa — `ph2d_field_render::sss_shadow::MAX_RAIO_PX`.
        .replace(
            "{MAX_RAIO_MOLE}",
            &formata(ph2d_field_render::sss_shadow::MAX_RAIO_PX),
        );
    // ⚠️⚠️ **As LEIS da marcha entram aqui desde o ricochete** (`docs/Render3d/08` §12): ele marcha
    // a partir da superfície, logo o passe que PINTA precisa do campo, da marcha e da
    // visibilidade. ⛔ **E só as leis** — os dois kernels da marcha ficam de fora, senão este
    // módulo teria pontos de entrada que ninguém despacha.
    // ⭐⭐⭐ **A camada de ESTILO entra ANTES do corpo, e a ordem é load-bearing:** com a
    // `struct Estilo` já declarada, o `Pintor` pode tê-la como CAMPO — e a arrumação dos vinte
    // números deixa de estar escrita uma segunda vez neste ficheiro. *Um adaptador que copiasse
    // `array[0] → rim`, `array[1] → convexo` … seria a segunda resposta à tabela do
    // `ph2d_style::wgsl::pack`, e a que envelhece.*
    format!(
        "{}{leis}{material}\n{}\n{}\n{dono}\n{corpo}",
        crate::trace_wgsl::comum(),
        ph2d_view_transform::wgsl::SOURCE,
        ph2d_style::wgsl::source()
    )
}

/// ⭐⭐⭐ **O AMBIENTE QUE O MATERIAL VÊ — o céu de quem chama, OU o ricochete.**
///
/// A lei indirecta do OpenPBR pergunta duas coisas ao ambiente (`env_radiance` e `env_irradiance`)
/// e é **linear** nas duas: o termo difuso entra pela irradiância, o especular pela radiância, e o
/// empilhamento é linear nas respostas. ⇒ *chamar a lei duas vezes com dois ambientes é somar os
/// dois*, que é exactamente o que o [`ph2d_field_render::shade_render`] faz com o `SoIrradiancia`.
///
/// ⚠️⚠️ **E tinham de ser DUAS chamadas e não um ambiente somado**, porque a parcela do céu leva a
/// oclusão por cima (`* ceu_vis`) e a do ricochete não: *a oclusão é a sombra do CÉU, e a luz que
/// vem das superfícies não é céu.*
///
/// ⛔ **É por isso que o [`PaintSetup::env_source`] declara `ceu_*` e não `env_*`:** quem manda no
/// ambiente é este despacho, e o céu de quem chama é um dos dois braços dele. Com o
/// `ambiente_e_ricochete` a `false` — o valor de nascença — o texto gerado entrega o céu **ao
/// bit**.
fn ambiente(ceu: &str) -> String {
    format!(
        r"{ceu}
// ⭐ O estado deste pixel, por invocação: o WGSL dá `var<private>`, que é exactamente isso.
var<private> ricochete: vec3<f32> = vec3<f32>(0.0);
var<private> ambiente_e_ricochete: bool = false;

fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {{
    // ⚠️ **O ricochete NÃO tem direcção**: ele é uma irradiância por pixel, e o lóbulo especular
    // pergunta *«que luz vem DAQUELA direcção»*. A resposta honesta é zero — a mesma que o
    // `SoIrradiancia` da CPU dá.
    if (ambiente_e_ricochete) {{ return vec3<f32>(0.0); }}
    return ceu_radiance(dir, alpha, shrink);
}}

fn env_irradiance(n: vec3<f32>) -> vec3<f32> {{
    if (ambiente_e_ricochete) {{ return ricochete; }}
    return ceu_irradiance(n);
}}
"
    )
}

/// ⚠️ Um `f32` que o WGSL leia como `f32` — sem isto um `0.9` inteiro sairia `0.9` e um `2` sairia
/// `2`, que ali é um literal **inteiro**.
fn formata(v: f32) -> String {
    let s = format!("{v:?}");
    if s.contains('.') || s.contains('e') {
        s
    } else {
        format!("{s}.0")
    }
}

/// ⭐⭐⭐ **A imagem, pintada onde os dados estão.**
// O dispositivo, a fila, o cache, o pedido, a lei do dono, os alvos da marcha, a tela e a contagem
// de bordas — oito coisas independentes, e uma struct só as renomearia.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pinta(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &mut crate::FieldPipelines,
    pintor: &PaintSetup<'_>,
    lei_do_dono: Option<&OwnersWgsl>,
    alvos: &Alvos<'_>,
    width: u32,
    height: u32,
    bordas: u64,
) -> Vec<u8> {
    let leis = alvos.leis;
    let fita = alvos.fita;
    use crate::trace::{armazem, uniforme};
    use wgpu::util::DeviceExt;

    let bgl1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("pintor"),
        entries: &[
            uniforme(0),
            uniforme(1),
            armazem(2, true),
            armazem(3, true),
            armazem(4, false),
            armazem(5, false),
            armazem(6, true),
            armazem(7, false),
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pintor"),
        bind_group_layouts: &[Some(alvos.bgl), Some(&bgl1)],
        immediate_size: 0,
    });
    // ⚠️ **O cache é o mesmo do traçado**, e a chave é o TEXTO: um arrasto de slider muda números e
    // não recompila nada, exactamente como na marcha.
    // ⭐⭐⭐⭐ **A FITA INERTE** — ver [`PaintSetup::le_o_campo`].
    //
    // ⚠️⚠️ **As CONSTANTES viajam na mesma, e hoje isso é uma PRECAUÇÃO e não uma cura** — medido
    // 2026-09-21 por mutação: o [`crate::FieldPipelines::entry_with_layout`] lê **só** o `.source`
    // da fita, e o armazém `k` é montado pelo `trace.rs` a partir da fita REAL. ⇒ trocá-las por
    // `Vec::new()` aqui é hoje **inobservável**, e a mutação que o faz SOBREVIVE — está nomeada
    // como tal. Ficam porque `fita.consts.len()` é a ORIGEM dos offsets que a escultura e a lei do
    // dono escrevem, e no dia em que esta porta os ler a versão encolhida fá-los ler as ranhuras
    // erradas **em silêncio**.
    let inerte = ph2d_field_eval::wgsl::TapeWgsl {
        source: FITA_INERTE.to_string(),
        consts: fita.consts.clone(),
    };
    let fita = if pintor.le_o_campo || pintor.ao_rays > 0 {
        fita
    } else {
        &inerte
    };
    let fonte = fonte(pintor, lei_do_dono, leis);
    // ⚠️ **A fita é a MESMA da marcha, e tem de o ser:** o `k` que o grupo `0` liga já traz as
    // constantes dela, e um texto gerado de outra fita indexaria aquele armazém por outra
    // aritmética. *Era a fita VAZIA enquanto o pintor não marchava.*
    let p_pinta = cache
        .entry_with_layout(device, &fonte, fita, "pinta", Some(&layout))
        .clone();
    // ⭐ **O ricochete só compila quando ele vai de facto correr** — a mesma lei que a borda já
    // segue: *compilar é o caro*.
    let p_ricochete = (pintor.ao_rays > 0).then(|| {
        cache
            .entry_with_layout(device, &fonte, fita, "pinta_ricochete", Some(&layout))
            .clone()
    });
    // ⭐⭐⭐ As SONDAS assam-se antes do ricochete as ler (`ph2d_field_render::probes`).
    let p_assa = (pintor.ao_rays > 0).then(|| {
        cache
            .entry_with_layout(device, &fonte, fita, "assa_sondas", Some(&layout))
            .clone()
    });
    // ⭐ A PRIMEIRA das duas passagens de borrão — a segunda é a recolha que o pintor faz ao ler.
    let p_borra = (pintor.ao_rays > 0).then(|| {
        cache
            .entry_with_layout(device, &fonte, fita, "borra_ricochete", Some(&layout))
            .clone()
    });
    // ⭐⭐⭐ **AS DUAS PASSAGENS DA BORDA MOLE** (`docs/Render3d/10` §12) — separáveis, logo DUAS.
    // ⛔ A segunda lê os vizinhos do que a primeira escreveu: escrever no mesmo sítio de onde eles
    // estão a ler é uma corrida, e é por isso que os slots do intermediário existem.
    let p_mole = pintor.mole.then(|| {
        (
            cache
                .entry_with_layout(device, &fonte, fita, "borra_mole_h", Some(&layout))
                .clone(),
            cache
                .entry_with_layout(device, &fonte, fita, "borra_mole_v", Some(&layout))
                .clone(),
        )
    });
    let p_bordas = (bordas > 0).then(|| {
        cache
            .entry_with_layout(device, &fonte, fita, "pinta_bordas", Some(&layout))
            .clone()
    });

    let bytes = |v: &[f32]| -> Vec<u8> { v.iter().flat_map(|f| f.to_le_bytes()).collect() };
    let ceu = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ceu"),
        contents: &bytes(pintor.env_consts),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let tabela = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("tabela"),
        contents: &bytes(pintor.env_tables),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let mut mats = if pintor.materials.is_empty() {
        vec![0.0f32; ph2d_material::wgsl::PACKED]
    } else {
        pintor.materials.to_vec()
    };
    #[allow(clippy::cast_possible_truncation)]
    let n_mats = (mats.len() / ph2d_material::wgsl::PACKED) as u32;
    // ⭐ **O material do chão vai no fim**, e o `n_mats` continua a contar só os da peça — é ele o
    // guarda do `ler_mat`, e o chão não é uma folha.
    mats.extend_from_slice(pintor.catcher);
    // ⭐ **As gémeas foscas vão a seguir ao chão**, e o `n_mats` continua a contar só as da peça —
    // ele é o guarda do `ler_mat`. A gémea de `k` mora em `(n_mats + 1 + k) * PACKED`.
    let tem_foscas = u32::from(!pintor.matte.is_empty());
    mats.extend_from_slice(pintor.matte);
    let materiais = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("materiais"),
        contents: &bytes(&mats),
        usage: wgpu::BufferUsages::STORAGE,
    });

    // ⭐ **A arrumação dos bytes vive no irmão** — ver [`crate::paint_uniforme`]. ⛔ Corte por
    // responsabilidade e por tecto de LOC, nunca por isenção.
    let (u, n_bordas) = crate::paint_uniforme::arruma(pintor, bordas, n_mats, tem_foscas);
    let ub_pintor = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("pintor"),
        contents: &u,
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let n = u64::from(width) * u64::from(height);
    let b_saida = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("imagem"),
        size: (n * 4).max(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    fn recurso(b: &wgpu::Buffer, i: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: i,
            resource: b.as_entire_binding(),
        }
    }
    let bg0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: alvos.bgl,
        entries: &[
            recurso(alvos.setup, 0),
            recurso(alvos.k, 1),
            recurso(alvos.centro, 2),
            recurso(alvos.luz, 3),
            recurso(alvos.conta, 4),
            recurso(alvos.borda, 5),
            recurso(alvos.grades, 6),
        ],
    });
    // ⭐ As sondas: `PROBE_GRID³ × 28` floats. ⚠️ Ele existe mesmo sem ricochete (a bandeira `0`
    // é «fora», e o pintor só o lê quando o despacho das sondas correu).
    let n_sondas = ph2d_field_render::probes::PROBE_GRID.pow(3);
    let b_sondas = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sondas"),
        size: (n_sondas * 28 * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    // ⭐ **O campo do chão**: `n² × 3` floats. ⚠️ Ele existe sempre (um armazém vazio não é ligável),
    // e o pintor só o lê com `modo2.y >= 2`.
    let mut chao: Vec<u8> = Vec::with_capacity(pintor.ground_bounce.value.len() * 12);
    for v in &pintor.ground_bounce.value {
        for f in v {
            chao.extend_from_slice(&f.to_le_bytes());
        }
    }
    if chao.is_empty() {
        chao.extend_from_slice(&[0u8; 16]);
    }
    let b_chao = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("chao_luz"),
        contents: &chao,
        usage: wgpu::BufferUsages::STORAGE,
    });
    // ⭐⭐⭐ **O QUADRO EM CENA-LINEAR, que só o brilho lê** (`docs/Render3d/12`) — e ele **é o
    // mesmo buffer em que o halo acaba por ser escrito**: quando a cadeia chega ao último degrau
    // ninguém volta a ler a cena. *Um quadro de `1898×916` são `27,8 MB`.*
    //
    // ⚠️ **Com o brilho desligado ele tem UM texel** — a rede que o binding exige —, e o `pinta`
    // não lhe toca (`modo2.w`).
    let cena_n = if pintor.bloom.contributes() { n } else { 1 };
    let b_cena = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("cena"),
        size: cena_n * 16,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let bg1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl1,
        entries: &[
            recurso(&ceu, 0),
            recurso(&ub_pintor, 1),
            recurso(&tabela, 2),
            recurso(&materiais, 3),
            recurso(&b_saida, 4),
            recurso(&b_sondas, 5),
            recurso(&b_chao, 6),
            recurso(&b_cena, 7),
        ],
    });

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    // ⚠️⚠️ **O ricochete ANTES da pintura, e a ordem é a lei**: a pintura lê a vizinhança `3×3` do
    // canal para o suavizar, logo ela precisa dele escrito em TODO o quadro — não só neste pixel.
    // *Escrito na mesma passagem, cada pixel leria oito vizinhos de um quadro que ainda não existe.*
    // ⭐⭐⭐ **As SONDAS primeiro**: um grupo de 256 threads por sonda — `PROBE_GRID³` grupos, que
    // cabem no limite de `65 535` por dimensão a `32³`. Elas não dependem da câmera; assá-las por
    // quadro assente custa `~1 ms` e uma cache por cena é a wave seguinte.
    if let Some(p) = &p_assa {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        #[allow(clippy::cast_possible_truncation)]
        cp.dispatch_workgroups(n_sondas as u32, 1, 1);
    }
    if let Some(p) = &p_ricochete {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    // ⚠️ **E o BORRÃO é um TERCEIRO despacho, pela MESMA razão**: ele lê a vizinhança `3×3` do canal
    // CRU, logo precisa dele escrito em todo o quadro. *Escrever no mesmo sítio de onde os vizinhos
    // estão a ler é uma corrida, e é por isso que os slots do liso existem.*
    if let Some(p) = &p_borra {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    // ⭐⭐⭐ **A BORDA MOLE, antes da pintura e em DUAS passagens** — a horizontal escreve o
    // intermediário e a vertical o canal que o pintor lê. ⚠️ A ordem é a lei, e é a mesma do borrão
    // do ricochete acima: cada passagem precisa da anterior escrita em TODO o quadro.
    if let Some((h_pass, v_pass)) = &p_mole {
        for p in [h_pass, v_pass] {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cp.set_pipeline(p);
            cp.set_bind_group(0, &bg0, &[]);
            cp.set_bind_group(1, &bg1, &[]);
            cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
        }
    }
    {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(&p_pinta);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    // ⚠️ **A borda depois do interior, e a ordem é a lei**: ela SOBRESCREVE o pixel que o interior
    // acabou de escrever, exactamente como o laço em série da CPU faz depois das linhas.
    if let Some(p) = &p_bordas {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(n_bordas.div_ceil(64), 1, 1);
    }
    // ⭐⭐⭐ **E O BRILHO, por último** — a cadeia e a composição, no dispositivo. Ver
    // [`crate::brilho`]. ⚠️ Os buffers dela são guardados até à submissão.
    let _brilho = crate::brilho::encadeia(
        device,
        cache,
        fita,
        &mut enc,
        &b_cena,
        &b_saida,
        (width, height),
        &pintor.bloom,
        &crate::brilho::Olhar {
            stops: pintor.stops,
            view: pintor.view,
        },
    );
    let leitura = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leitura"),
        size: (n * 4).max(16),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    enc.copy_buffer_to_buffer(&b_saida, 0, &leitura, 0, (n * 4).max(16));
    queue.submit([enc.finish()]);
    leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let dados = leitura.slice(..).get_mapped_range();
    #[allow(clippy::cast_possible_truncation)]
    let out = dados[..(n as usize) * 4].to_vec();
    drop(dados);
    out
}
