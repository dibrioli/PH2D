// Hospedado por `ph2d-mesh-render` (crates/ph2d-mesh-render/src/shaders/mesh.wgsl).
//
// O passe da malha: rasteriza a forma e a acende com O RIG DO ARTISTA — as mesmas
// quatro lâmpadas que iluminam a tinta do Painter, resolvidas pela mesma função
// (`ph2d-light`), dobradas pelo mesmo piso ambiente.
//
// ISTO SUBSTITUI O MATCAP DA W1, e a troca é a tese da W3 (docs/3D/05.2): a malha
// não pede um sistema de luz novo, ela pede uma segunda fonte de NORMAL para o que
// já existe. O matcap era sombreamento função-da-normal com lâmpadas cravadas no
// shader; era certo para "haver forma na tela" e é errado para "um documento, uma
// iluminação" -- sob ele, mover a lâmpada do card não mexia na escultura.
//
// O MODELO É RELATIVO, e essa é a propriedade que faz a doação funcionar: a
// resposta de um ponto é dividida pela resposta de uma superfície PLANA sob o
// mesmo rig. Uma face virada para o olho devolve exatamente a cor do barro; o que
// se inclina escurece até o piso ambiente ou clareia até o dobro. É o mesmo
// contrato que mantém tinta plana byte-idêntica no Painter -- e é por isso que a
// mesma lâmpada vai poder multiplicar a pintura por baixo da forma, na M4, sem
// que nada precise concordar por acidente.
//
// O QUE NÃO É COMPARTILHADO É O MATERIAL. A tinta tem rugosidade, metal e cera
// por-pixel com LUT baked; o barro tem uma cor e um expoente. O material da malha
// é a wave do shader (docs/3D/05.1, W7). A fronteira está tabelada em
// `src/lighting.rs`.
//
// A saída é LINEAR e pode passar de 1.0: o alvo é o `game_rt` (Rgba16Float) e o
// tonemap do shell vem depois. Escrever já-tonemapeado aqui apagaria o realce --
// e é por isso que o realce SOMA aqui e faz `screen` no Painter, que escreve
// unorm8 e cujo pixel É a arte.

struct Camera {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
};

// Uma lâmpada resolvida — o mesmo dado que sobe para o passe de luz da tinta.
struct Lamp {
    dir: vec4<f32>,
    hlf: vec4<f32>,
    tint: vec4<f32>,
};

struct Rig {
    lamps: array<Lamp, 4>,
    n: u32,
};

// **ONDE ESTE OBJETO ESTÁ** (`ph2d_mesh::Pose`). Grupo PRÓPRIO, e não uma
// terceira entrada do grupo 0, porque a frequência é outra: a câmera e o rig são
// da CENA (um write por frame) e isto é do objeto (um bind por desenho). Juntá-los
// obrigaria um bind group por objeto carregando cópias da câmera, que é a forma
// de as duas ficarem em desacordo no dia em que uma delas for escrita a menos.
struct Object {
    model: mat4x4<f32>,
};

// **AS OPÇÕES DE SOMBREAMENTO** (`crate::shade`). Uniform e não `const` de
// permutação porque é uma QUANTIDADE que o artista arrasta, não uma capacidade
// que muda o corpo do shader: recompilar um pipeline por passo de slider é meio
// segundo de trava por toque.
struct Shade {
    cavity: f32,
    // **QUAL MATCAP acende o barro** — `0` é o RIG DO ARTISTA (o caminho da W3,
    // byte-idêntico), `n` é o material `n − 1` da tabela abaixo.
    //
    // ⚠️ Um `u32` no uniform e não uma permutação de pipeline: trocar de material
    // é um clique, e recompilar um pipeline por clique é meia-tela de trava. É a
    // mesma decisão que a cavidade já tomou, pelo motivo vizinho.
    matcap: u32,
    // **QUANTO DO AO ASSADO ENTRA.** `0` = o barro sem oclusão, **ao byte** — e
    // é o default, porque um canal que nem foi assado não pode escurecer nada.
    //
    // ⚠️ Ele pousa no primeiro dos dois `f32` que o `ShadeRaw` já reservava
    // dizendo *"é aqui que SSS e AO vão pousar sem mexer no layout"*. O layout
    // não mudou: a promessa foi cobrada.
    ao: f32,
    // **QUANTO DO AO DE TELA ENTRA.** `1` = todo ele, e é o default — o oposto do
    // vizinho, porque este canal é MEDIDO todo frame em vez de assado uma vez.
    ssao: f32,
    // **QUANTO DO ESPALHAMENTO SUB-SUPERFICIAL ENTRA.** `0` = o barro de sempre,
    // ao byte — e é o default, porque isto é um MATERIAL e barro não é pele.
    sss_strength: f32,
    // O `scatter` do artista já dividido pelo teto da tabela: a coordenada `v`
    // sai de `|kappa| *` isto. ⚠️ A divisão mora no `SssRaw::pack` e SÓ lá —
    // reproduzi-la aqui seria a segunda cópia do teto, e ela divergiria no dia
    // em que o teto mudasse.
    sss_scale: f32,
    // `1 / scatter` — o coeficiente da transmitancia. Ver `sss::SssRaw::pack`.
    trans_scale: f32,
    // ⚠️ **UM `_pad` só.** O `trans_scale` ocupou o outro, que era exatamente o
    // que o comentário do `ShadeRaw` reservava — e o `size_of` continua em 32 B,
    // que é o que o gate de layout afirma.
    _pad0: f32,
};

@group(0) @binding(0) var<uniform> cam: Camera;
@group(0) @binding(1) var<uniform> rig: Rig;
@group(0) @binding(2) var<uniform> shade: Shade;
@group(1) @binding(0) var<uniform> obj: Object;

// O piso AMBIENTE: o que uma face totalmente virada para longe da luz ainda
// devolve. Sombra é mais escura, não é preta. ⚠️ É `ph2d_light::AMBIENT`, e a
// igualdade é gateada (`the_clay_folds_the_ratio_by_the_same_ambient_floor`) --
// duas cópias dariam uma escultura mais escura na sombra que a pintura ao lado
// dela, sob a MESMA lâmpada.
const AMBIENT: f32 = 0.35;

// Piso do divisor. Um canal a que o rig não dá luz nenhuma (uma lâmpada pura
// vermelha, no canal azul) dividiria por zero; a difusa dele também é zero, então
// a resposta que mantém o contrato é 1 -- o canal fica intocado.
const FLAT_FLOOR: f32 = 1.0e-4;

// O barro de estúdio: claro e dessaturado, para a FORMA aparecer.
const CLAY: vec3<f32> = vec3<f32>(0.74, 0.70, 0.66);

// **O GANHO DA CAVIDADE** — o que leva a curvatura crua à faixa que o olho usa.
//
// ⚠️ É `ph2d_mesh_render::CAVITY_GAIN`, e a igualdade é gateada pelo MESMO teste
// que já pina o material do barro (`the_clays_material_is_the_same_number...`):
// duas cópias dariam uma cavidade no vivo diferente da do objeto assado.
//
// A curvatura é adimensional e pequena: MEDIDO, o fundo liso de uma esfera fica
// em |k| ~ 0,02-0,05 e um vinco chega a 0,7. E o fundo ENCOLHE com a tesselação
// enquanto o vinco não — é isso que deixa um ganho constante servir malhas de
// densidades diferentes. 4,0 satura em 0,25, entre os dois p99 medidos.
const CAVITY_GAIN: f32 = 4.0;

// O expoente Blinn-Phong do barro. ⚠️ É UM NÚMERO, não um modelo de material: a
// tinta deriva o dela da rugosidade por-pixel por uma LUT baked, e o barro ainda
// não tem rugosidade. 24 é o mesmo ponto neutro que o impasto usa por default
// (a média geométrica de 6 e 96).
const CLAY_EXPONENT: f32 = 24.0;

// Quanto do realce entra. O `Shine` da tinta é per-pixel; aqui é fixo, pelo mesmo
// motivo do expoente.
const CLAY_SHINE: f32 = 0.35;

// **A MÁSCARA.** ⚠️ Convenção INVERTIDA em relação ao SculptGL: aqui `0 = livre`
// e `1 = protegido` (lá é o contrário). É a armadilha nº 1 de todo port desta
// área, e ela está no livro-razão.
//
// O tinto é AZUL-FRIO e escuro porque o barro é claro e quente: a região
// protegida tem de se ler como *outra substância*, não como *o mesmo barro na
// sombra* — senão o artista confunde máscara com a forma que ele acabou de
// esculpir, que é o único erro caro aqui.
const MASK_TINT: vec3<f32> = vec3<f32>(0.30, 0.42, 0.58);

// Quanto a região protegida cede à cor de máscara no teto (`mask = 1`).
// ⚠️ NÃO é 1.0: apagar o barro por completo esconderia o RELEVO embaixo, e a
// pergunta que o artista faz olhando uma máscara é *"cobri a dobra inteira?"* —
// que é sobre a forma, não sobre a máscara.
const MASK_STRENGTH: f32 = 0.75;

// ============================ O MATCAP ============================
//
// **O que um matcap É:** sombreamento que é função APENAS da normal em espaço de
// vista. A luz viaja com a câmera, então orbitar não muda a leitura da forma — é
// por isso que todo app de escultura o oferece, e é a razão de ele NÃO ser
// substituível pelo rig: o rig é do DOCUMENTO (a mesma lâmpada acende a tinta ao
// lado), o matcap é do OLHO.
//
// ⚠️ **ANALÍTICO, e não uma textura.** A forma canônica é uma imagem de esfera
// amostrada por `n.xy * 0.5 + 0.5`, e ela seria o caminho certo se houvesse
// matcaps AUTORADOS para carregar. Não há: seriam assets novos, com licença, num
// repo que não os tem. Uma textura sintetizada na CPU seria a MESMA função
// avaliada uma vez por texel e depois interpolada — mais bindings, mais VRAM,
// mais uma resolução a escolher, e uma SEGUNDA cópia da fórmula (a de assar e a
// de ler). Avaliada aqui ela é exata por pixel, custa cinco produtos escalares e
// não tem resolução. O dia em que um matcap de imagem chegar, ele entra como uma
// FONTE a mais — não como a correção desta.
//
// ⚠️ O espaço é o do rig (`canvas_normal`): `y` cresce para BAIXO, como na tela.
// É por isso que a luz principal aponta para `-y` — "em cima, à esquerda" tem de
// querer dizer a mesma coisa nos dois modos, senão trocar de material vira o
// modelo de cabeça para baixo.
struct Mat {
    base: vec3<f32>,
    key_dir: vec3<f32>,
    key: vec3<f32>,
    fill_dir: vec3<f32>,
    fill: vec3<f32>,
    rim: vec3<f32>,
    rim_pow: f32,
    spec_pow: f32,
    spec: f32,
};

// A principal vem de cima-à-esquerda-frente e o preenchimento do lado oposto,
// mais fraco e frio: é o estúdio de duas luzes que toda foto de escultura usa, e
// o que separa "há forma" de "há forma e eu consigo ler a virada dela".
const KEY_DIR: vec3<f32> = vec3<f32>(-0.40, -0.55, 0.73);
const FILL_DIR: vec3<f32> = vec3<f32>(0.50, 0.35, 0.79);

fn material(id: u32) -> Mat {
    var m: Mat;
    m.key_dir = KEY_DIR;
    m.fill_dir = FILL_DIR;
    switch id {
        // **BARRO** — o mesmo cinza quente do rig, para a troca de modo mostrar
        // a diferença de LUZ e não a de cor.
        case 0u: {
            m.base = vec3<f32>(0.74, 0.70, 0.66);
            m.key = vec3<f32>(1.00, 0.98, 0.94);
            m.fill = vec3<f32>(0.30, 0.34, 0.42);
            m.rim = vec3<f32>(0.10, 0.11, 0.13);
            m.rim_pow = 3.0; m.spec_pow = 24.0; m.spec = 0.14;
        }
        // **PÉROLA** — claro e macio: o material que menos esconde a forma, e o
        // que o escultor usa para julgar silhueta.
        case 1u: {
            m.base = vec3<f32>(0.88, 0.87, 0.90);
            m.key = vec3<f32>(1.00, 1.00, 1.00);
            m.fill = vec3<f32>(0.42, 0.46, 0.58);
            m.rim = vec3<f32>(0.28, 0.30, 0.36);
            m.rim_pow = 3.0; m.spec_pow = 48.0; m.spec = 0.40;
        }
        // **PELE** — a translucidez lida na borda: o `rim` quente é o que a luz
        // atravessando a orelha faz, e é o teste de uma cabeça.
        case 2u: {
            m.base = vec3<f32>(0.90, 0.71, 0.62);
            m.key = vec3<f32>(1.00, 0.95, 0.88);
            m.fill = vec3<f32>(0.40, 0.24, 0.22);
            m.rim = vec3<f32>(0.72, 0.26, 0.20);
            m.rim_pow = 2.2; m.spec_pow = 20.0; m.spec = 0.12;
        }
        // **JADE** — verde translúcido, borda acesa: o material que mais mostra
        // curvatura fina, porque o `rim` responde ao que vira de perfil.
        case 3u: {
            m.base = vec3<f32>(0.32, 0.58, 0.45);
            m.key = vec3<f32>(0.92, 1.00, 0.95);
            m.fill = vec3<f32>(0.12, 0.28, 0.24);
            m.rim = vec3<f32>(0.42, 0.86, 0.66);
            m.rim_pow = 2.0; m.spec_pow = 64.0; m.spec = 0.55;
        }
        // **METAL** — base escura e realce apertado: é o que revela ONDULAÇÃO,
        // porque um realce estreito varre a superfície e denuncia toda barriga.
        case 4u: {
            m.base = vec3<f32>(0.24, 0.26, 0.30);
            m.key = vec3<f32>(1.00, 1.00, 1.00);
            m.fill = vec3<f32>(0.16, 0.20, 0.30);
            m.rim = vec3<f32>(0.55, 0.60, 0.72);
            m.rim_pow = 4.0; m.spec_pow = 128.0; m.spec = 1.30;
        }
        // **CERA VERMELHA** — o barro de escultor clássico; contraste alto e cor
        // saturada, para ver o volume geral de longe.
        default: {
            m.base = vec3<f32>(0.70, 0.20, 0.16);
            m.key = vec3<f32>(1.00, 0.90, 0.85);
            m.fill = vec3<f32>(0.30, 0.09, 0.09);
            m.rim = vec3<f32>(0.85, 0.38, 0.26);
            m.rim_pow = 2.2; m.spec_pow = 32.0; m.spec = 0.26;
        }
    }
    return m;
}

// O material aceso, em linear e podendo passar de 1 — o alvo é HDR e o tonemap
// do shell vem depois, exatamente como no caminho do rig.
fn matcap_shade(n: vec3<f32>, id: u32) -> vec3<f32> {
    let m = material(id);
    // ⚠️ **`abs` e não `max(.., 0)` no preenchimento.** O preenchimento existe
    // para a face virada para longe da principal não cair no preto; clampá-lo
    // deixaria uma calota inteira no piso e a forma sumiria ali — que é o
    // defeito que o piso AMBIENTE resolve no caminho do rig, por outra via.
    let kd = max(dot(n, normalize(m.key_dir)), 0.0);
    let fd = max(dot(n, normalize(m.fill_dir)), 0.0);
    // A borda: quanto a superfície vira de perfil. `n.z` é o cosseno com o olho,
    // então `1 − n.z` é zero de frente e um na silhueta.
    let rim = pow(clamp(1.0 - n.z, 0.0, 1.0), m.rim_pow);
    // O realce das DUAS lâmpadas, cada um pelo próprio meio-vetor com o olho
    // (`+z`), que é a construção Blinn-Phong que o barro do rig já usa.
    let hk = normalize(normalize(m.key_dir) + vec3<f32>(0.0, 0.0, 1.0));
    let hf = normalize(normalize(m.fill_dir) + vec3<f32>(0.0, 0.0, 1.0));
    let sp = pow(max(dot(n, hk), 0.0), m.spec_pow)
        + 0.35 * pow(max(dot(n, hf), 0.0), m.spec_pow);
    return m.base * (m.key * kd + m.fill * fd) + m.rim * rim + m.spec * sp;
}

// **A OCLUSÃO DE TELA** (`crate::ssao`) — grupo próprio porque a frequência é
// outra: o grupo 0 são três uniforms estáveis e isto é uma TEXTURA recriada a
// cada resize.
//
// ⚠️ Sem sampler: `textureLoad` com as coordenadas inteiras do próprio fragmento.
// A correspondência é 1:1 com a tela, então filtrar seria interpolar uma medição
// consigo mesma.
@group(2) @binding(0) var ssao_tex: texture_2d<f32>;

// **A TABELA PRÉ-INTEGRADA DO SSS** (`crate::sss`) — grupo próprio porque a
// frequência é a mais baixa de todas: ela é assada UMA vez na vida do processo,
// enquanto o grupo 2 é recriado a cada resize.
//
// ⚠️ **COM sampler, ao contrário do AO** — aqui a textura é uma FUNÇÃO tabelada
// e a consulta cai ENTRE os nós; é a interpolação que torna 128 linhas
// suficientes. No AO a correspondência é 1:1 com a tela, e filtrar seria
// interpolar uma medição consigo mesma.
@group(3) @binding(0) var sss_lut: texture_2d<f32>;
@group(3) @binding(1) var sss_samp: sampler;

/// **A resposta difusa deste ponto, dado `N·L` e a curvatura de mundo.**
///
/// Sem espalhamento é `max(N·L, 0)` — o Lambert de sempre, nos três canais. Com
/// espalhamento é a tabela de Penner, que devolve TRÊS números diferentes: o
/// vermelho viaja mais dentro da carne, e é só isso que faz um rosto parecer
/// carne em vez de plástico.
///
/// ⚠️ **`abs(curv)`, e não é economia:** a pré-integração é PAR em `x`, então
/// `D(θ, r) = D(θ, |r|)` — uma narina e uma ponta de nariz do mesmo raio
/// espalham igual. Quem usa o SINAL da curvatura é a cavidade, logo acima.
///
/// ⚠️ E o `mix` é o que faz `sss_strength = 0` ser **byte-idêntico** ao barro
/// anterior a este canal: o segundo braço nem é consultado quando o peso é zero
/// — mas ele É avaliado, e é por isso que a tabela zerada de antes do
/// `ensure_sss_lut` não pode escurecer nada (ela entra multiplicada por 0).
// **A ATENUACAO POR CANAL na TRANSMITANCIA**, relativa ao vermelho.
//
// ⚠️ Os dois numeros sao DERIVADOS do mesmo `PROFILE` que a tabela usa
// (`sss::channel_attenuation`), e o gate
// `the_transmittance_channels_are_the_same_number_in_the_shader_and_in_rust`
// e' quem impede esta copia de derivar. O vermelho e' 1 por construcao: e' ele
// a referencia de normalizacao do perfil inteiro.
//
// E' esta assimetria -- o azul se apagando 7,5x mais depressa que o vermelho --
// que faz uma mao contra a lanterna ficar VERMELHA em vez de cinza.
const TRANS_K_G: f32 = 4.505294;
const TRANS_K_B: f32 = 7.471505;

// **A TRANSMITANCIA** -- a luz que ENTRA pelo outro lado e sai aqui.
//
// ⚠️ **Ela SOMA, e e' por isso que ela existe.** O canal pre-integrado
// REDISTRIBUI a luz da frente, e o teto dele e' a media dela: medido, `1/pi` --
// e chegar la' custa a cor inteira (a separacao R-B cai de 0,0375 em `t = 1,5`
// para 0,0001 em `t = 24`, ou seja o disco fica CINZA). Nenhuma quantidade de
// difusao pre-integrada acende o que o lambert deixou em zero; so' um termo
// aditivo acende, e cera/folha/orelha sao todos ELE.
//
// ⚠️ **`-N·L`, e nao `N·L`:** a luz tem de estar ATRAS da superficie. De frente
// o termo e' exatamente zero, entao ele nao clareia nada que ja' esteja aceso --
// o que impede o canal de virar um `Exposure`.
fn transmittance(n_dot_l: f32, thickness: f32) -> vec3<f32> {
    if (shade.sss_strength <= 0.0) {
        return vec3<f32>(0.0);
    }
    let back = max(-n_dot_l, 0.0);
    if (back <= 0.0) {
        return vec3<f32>(0.0);
    }
    let d = max(thickness, 0.0) * shade.trans_scale;
    let through = vec3<f32>(exp(-d), exp(-d * TRANS_K_G), exp(-d * TRANS_K_B));
    return shade.sss_strength * back * through;
}

fn sss_diffuse(n_dot_l: f32, curv: f32) -> vec3<f32> {
    let lambert = vec3<f32>(max(n_dot_l, 0.0));
    if (shade.sss_strength <= 0.0) {
        return lambert;
    }
    // `u` mapeia [-1, 1] em [0, 1] — a tabela cobre o lado ESCURO de propósito,
    // porque é exatamente lá que a luz que vazou aparece.
    let u = n_dot_l * 0.5 + 0.5;
    // `v` satura no teto pelo `ClampToEdge` do sampler: pedir mais espalhamento
    // do que a tabela representa faz o controle parar de responder, nunca voltar
    // ao outro extremo.
    let v = abs(curv) * shade.sss_scale;
    let scattered = textureSample(sss_lut, sss_samp, vec2<f32>(u, v)).rgb;
    return mix(lambert, scattered, shade.sss_strength);
}

/// **QUANTO ESTE PIXEL ESTÁ OCLUÍDO PELO QUE ESTÁ NA TELA.**
///
/// ⚠️ A coordenada é CLAMPADA ao tamanho da textura, e a linha é load-bearing: sem
/// medição o binding é um 1×1 zerado, e um `textureLoad` fora dos limites devolve
/// zero em WGSL — que nesta convenção também é *"nada oclui"*, mas por acidente de
/// especificação em vez de por desenho. Com o clamp a resposta é a MESMA por
/// construção, e o dia em que a convenção mudar de sinal isto não vira uma peça
/// preta.
fn screen_occlusion(frag: vec2<f32>) -> f32 {
    let dim = vec2<i32>(textureDimensions(ssao_tex)) - vec2<i32>(1);
    let px = clamp(vec2<i32>(frag), vec2<i32>(0), dim);
    return clamp(textureLoad(ssao_tex, px, 0).r, 0.0, 1.0);
}

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) n_view: vec3<f32>,
    @location(1) mask: f32,
    @location(2) curv: f32,
    @location(3) ao: f32,
    @location(4) curv_world: f32,
    @location(5) thickness: f32,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) mask: f32,
    @location(3) curv: f32,
    @location(4) ao: f32,
    @location(5) curv_world: f32,
    @location(6) thickness: f32,
) -> VsOut {
    var out: VsOut;
    out.clip = cam.view_proj * obj.model * vec4<f32>(pos, 1.0);
    out.mask = mask;
    // O AO tampouco cruza `obj.model`, e pela razão da curvatura logo abaixo:
    // ele é uma FRAÇÃO (quanto do céu o vértice enxerga), não uma direção nem um
    // comprimento. Mover a peça não muda o quanto ela se auto-oclui.
    out.ao = ao;
    // ⚠️ **A curvatura NÃO cruza `obj.model`, e a normal cruza.** Ela é
    // adimensional por construção (a divisão pelo raio médio do anel, em
    // `ph2d_mesh::curvature`), então a escala da `Pose` já se cancelou na CPU —
    // e a rotação, quando ela chegar, não move um escalar. Aplicar a matriz aqui
    // seria a mesma classe de erro do ângulo cru sobre altura no impasto: uma
    // grandeza que não tem direção passando por uma transformação que só sabe
    // falar de direção.
    out.curv = curv;
    // ⚠️ **A de MUNDO tampouco cruza `obj.model`, e por um motivo DIFERENTE do
    // da irmã.** A adimensional não cruza porque a escala já se cancelou na CPU;
    // esta não cruza porque ela é `1/comprimento` e a `Pose` escala UNIFORME —
    // então uma peça duplicada de tamanho teria a curvatura de mundo dividida
    // por dois, e é a CPU quem já a mediu na geometria local. O dia em que uma
    // `Pose` com escala chegar ao SSS, é AQUI que ela entra (dividida, não
    // multiplicada) — e o gate `the_scatter_is_a_fraction_of_the_piece` é o que
    // mantém o knob na mesma proporção enquanto isso não existe.
    out.curv_world = curv_world;
    // A espessura tampouco cruza o `obj.model`: ela e' um COMPRIMENTO local, e
    // o `Pose` da peca nao escala (`docs/3D/W8.1`). O dia em que escalar, e' aqui
    // que ela cruza -- e o `trans_scale`, que e' `1/comprimento`, cruza junto.
    out.thickness = thickness;
    // `w = 0` ⇒ direção, não ponto: a translação da vista **e a do objeto** não
    // entram. A matriz de vista é ortonormal (sai de uma `look_at`) e a do
    // objeto é uma SIMILARIDADE (`Pose` é translação + escala UNIFORME), então
    // não há inverso-transposto a fazer — a escala sobrevive como um fator
    // comum e o `normalize` do `canvas_normal` a cancela.
    //
    // ⚠️ É esta linha que uma escala por-eixo quebraria, em silêncio: ela
    // inclinaria a normal sem inclinar a superfície, e o sintoma seria uma luz
    // levemente torta que ninguém consegue nomear. A `Pose` recusa esse caso na
    // representação, e não por convenção.
    //
    // ⚠️ **MEDIDO: o `obj.model` daqui é INERTE hoje, e fica assim mesmo.**
    // Tirá-lo deixa os 22 gates de GPU verdes — com translação e escala
    // uniforme a translação some no `w = 0` e a escala é cancelada pelo
    // `normalize` do `canvas_normal`. Ele é a fórmula GERAL, e é ela que passa a
    // carregar peso no dia em que a `Pose` ganhar rotação: um shader que
    // dispensasse a multiplicação continuaria correto até aquele dia e depois
    // acenderia a forma com a normal de antes de ela girar. Mesma decisão, e
    // mesmo motivo, do `Sculpt3dScene::dir_to_local`.
    out.n_view = (cam.view * obj.model * vec4<f32>(normal, 0.0)).xyz;
    return out;
}

// **A normal, no espaço em que o rig vive.** Porta única: o barro a lê para se
// acender, e o G-buffer a ESCREVE para a tinta se acender por ela. Duas versões
// disto seriam duas respostas a "para onde esta superfície aponta", e elas
// divergiriam no unico lugar onde ninguem le um numero de volta.
fn canvas_normal(n_view: vec3<f32>) -> vec3<f32> {
    // A interpolação entre vértices encurta a normal; sem renormalizar, a
    // superfície escurece no meio de cada triângulo (o facetamento clássico).
    var n = normalize(n_view);

    // Em espaço de vista o olho olha por `-Z`, então `n.z > 0` está de frente.
    // Uma face vista por trás (malha aberta, casca fina) tem de acender como
    // frente — senão o interior de uma peça aberta vira um buraco preto e o
    // artista lê isso como geometria faltando.
    if (n.z < 0.0) {
        n = -n;
    }

    // ⚠️ A ÚNICA conversão de espaço do passe, e a que só um render revela: o rig
    // é autorado em espaço de TELA, onde `y` cresce para BAIXO ("a principal está
    // em cima, à esquerda" só quer dizer algo lá). A vista tem `y` para CIMA.
    // Sem esta negação a mesma lâmpada acende a pintura por cima e a escultura por
    // baixo, no mesmo documento, sob o mesmo card, com o mesmo número.
    return vec3<f32>(n.x, -n.y, n.z);
}

// **O G-BUFFER — a DOAÇÃO, do lado de quem doa.**
//
// `docs/3D/05.2` numa frase: *"a malha não pede um sistema de luz novo, ela pede
// uma SEGUNDA FONTE DE NORMAL para o que já existe"*. Isto é essa fonte.
//
// `xyz` = a normal no espaço do rig (a MESMA que o barro usa, pela porta acima).
// `w`  = COBERTURA: 1 onde a malha está. O alvo é limpo em zero, então `w = 0`
//        quer dizer *"aqui não há forma"*, e é isso que deixa o passe de luz da
//        tinta escolher a fonte por PIXEL em vez de por documento.
//
// ⚠️ `Rgba16Float` e não um formato normalizado: as componentes vivem em [-1, 1] e
// um unorm exigiria codificar `n * 0.5 + 0.5` de um lado e decodificar do outro —
// duas metades que precisam concordar, num canal onde a discordância é uma luz
// levemente torta que ninguém consegue nomear.
// ⚠️ **O G-buffer IGNORA a máscara, e isso é decisão e não esquecimento.** A
// máscara é chrome de AUTORIA — ela diz ao escultor onde o pincel não pega — e
// não uma propriedade da forma. Se ela entrasse aqui, a tinta que o Painter
// acende por baixo sairia azulada onde o escultor protegeu, e o artista veria a
// sua ferramenta de trabalho vazar para dentro da obra. Há gate.
@fragment
fn fs_gbuffer(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(canvas_normal(in.n_view), 1.0);
}

// **O WIREFRAME** — o passe de LINHAS sobre a forma já acesa.
//
// ⚠️ Cor fixa e nenhuma iluminação, de propósito: a malha é um instrumento de
// LEITURA DE TOPOLOGIA (*onde o remesh pôs os anéis? o refino chegou aqui?*), e
// acender as linhas com o mesmo modelo do barro as faria sumir exatamente onde a
// superfície é escura — que é onde a densidade importa mais.
//
// Escura e semitransparente: uma malha densa em branco vira uma chapa e apaga a
// forma que ela deveria anotar.
const WIRE_RGBA: vec4<f32> = vec4<f32>(0.05, 0.06, 0.08, 0.55);

@fragment
fn fs_wire() -> @location(0) vec4<f32> {
    return WIRE_RGBA;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let nc = canvas_normal(in.n_view);

    // **A CAVIDADE** vale nos DOIS modos, e é por isso que ela subiu para cá: ela
    // não é iluminação, é leitura de FORMA — o canal que desenha a fresta. Deixá-la
    // no caminho do rig faria o artista perder a curvatura justamente ao trocar
    // para o material que ele escolheu para ler melhor.
    let cav = 1.0 - shade.cavity * clamp(in.curv * CAVITY_GAIN, -1.0, 1.0);

    // **O AO ASSADO**, e ele mora ao lado da cavidade pelo mesmo motivo: também
    // não é iluminação, é leitura de FORMA. Os dois canais respondem a perguntas
    // diferentes e é por isso que somam em vez de competir — a cavidade é LOCAL
    // (a virada de uma aresta, o vinco que o dedo sente) e o AO é GLOBAL (o que
    // um cone enxerga a meio corpo de distância, a axila e o vão entre membros).
    //
    // ⚠️ `mix` e não multiplicação direta: com `shade.ao = 0` o termo é
    // exatamente `1.0`, então o barro sem oclusão é **byte-idêntico** ao de antes
    // deste canal — e é isso que faz o default não mover um pixel de nada que já
    // foi esculpido.
    let baked = mix(1.0, clamp(in.ao, 0.0, 1.0), shade.ao);

    // **O AO DE TELA**, medido nesta vista e nunca velho (`crate::ssao`).
    //
    // ⚠️ **As duas fontes compõem pelo MENOS-OCLUÍDO, não pelo produto**, e a
    // razão é que elas descrevem a MESMA sombra por dois caminhos: o assado
    // enxerga o corpo inteiro em qualquer direção (metros de campo SDF), este vê
    // um raio em torno do pixel. Onde as duas acertam — uma axila funda, que é
    // justamente onde a oclusão importa — um produto escureceria em DOBRO, e o
    // artista veria a peça ficar preta ao assar. `min` diz *"a mais escura das
    // duas medições vale"*, que é o que uma medição de visibilidade significa.
    //
    // ⚠️ E com uma das duas ausente o `min` é a IDENTIDADE da outra (o ausente
    // vale 1), então nem o caminho só-assado nem o caminho só-tela pagam nada por
    // existirem lado a lado.
    let screen = 1.0 - shade.ssao * screen_occlusion(in.clip.xy);
    let occ = min(baked, screen);
    let cav_occ = cav * occ;

    // **O MATCAP** — a luz do OLHO, e não a do documento. Ele vem ANTES da recusa
    // por rig apagado: ele não usa o rig, então apagar as lâmpadas do card não
    // pode apagá-lo.
    if (shade.matcap > 0u) {
        let id = shade.matcap - 1u;
        let lit = matcap_shade(nc, id);
        // ⚠️ **O MESMO modelo relativo do caminho do rig**, e não uma segunda
        // regra: a resposta dividida pela de uma superfície PLANA sob a mesma
        // luz. Lá o divisor é `flat_d`; aqui é o próprio material avaliado na
        // normal frontal, que é o que "plano" quer dizer neste espaço. É esta
        // razão que deixa a máscara tingir com a MESMA lei nos dois modos.
        let flat = max(matcap_shade(vec3<f32>(0.0, 0.0, 1.0), id), vec3<f32>(FLAT_FLOOR));
        let ratio = lit / flat;
        var cm = lit * cav_occ;
        cm = mix(cm, MASK_TINT * ratio * cav_occ, clamp(in.mask, 0.0, 1.0) * MASK_STRENGTH);
        return vec4<f32>(cm, 1.0);
    }

    // Sem lâmpada acesa não há razão a computar: o barro cru é a leitura honesta
    // de "o artista apagou tudo" para uma superfície opaca.
    if (rig.n == 0u) {
        return vec4<f32>(CLAY, 1.0);
    }

    var diffuse = vec3<f32>(0.0);
    var trans = vec3<f32>(0.0);
    var spec = vec3<f32>(0.0);
    var flat_d = vec3<f32>(0.0);
    for (var i = 0u; i < rig.n; i = i + 1u) {
        let l = rig.lamps[i];
        // A resposta PLANA (N = (0,0,1) ⇒ N·L = L.z), que é o divisor de tudo.
        let lz = max(l.dir.z, 0.0);
        flat_d = flat_d + l.tint.rgb * lz;
        let ndl = dot(nc, l.dir.xyz);
        diffuse = diffuse + l.tint.rgb * sss_diffuse(ndl, in.curv_world);
        // ⚠️ **Fora do `diffuse`, e num acumulador proprio.** O `diffuse` e'
        // dividido pelo `flat_c` logo abaixo (a resposta RELATIVA a uma
        // superficie plana), e a transmitancia nao e' relativa a nada: ela e'
        // luz que atravessou. Somar aqui a faria encolher quando o artista
        // acrescentasse uma lampada.
        trans = trans + l.tint.rgb * transmittance(ndl, in.thickness);
        // O realce de cada lâmpada é relativo AO PRÓPRIO realce dela numa
        // superfície plana, e clampado em zero ALI: somar os brutos e subtrair o
        // total plano deixaria uma lâmpada virada para o outro lado tomar
        // emprestada a folga de uma virada para cá, e a forma brilharia no chapado.
        let fs = pow(max(l.hlf.z, 0.0), CLAY_EXPONENT);
        let s = pow(max(dot(nc, l.hlf.xyz), 0.0), CLAY_EXPONENT);
        spec = spec + l.tint.rgb * max(s - fs, 0.0);
    }

    let flat_c = select(vec3<f32>(1.0), flat_d, flat_d > vec3<f32>(FLAT_FLOOR));
    let ratio = clamp(diffuse / flat_c, vec3<f32>(0.0), vec3<f32>(2.0));
    let m = vec3<f32>(AMBIENT) + (1.0 - AMBIENT) * ratio;

    // **A CAVIDADE** — o canal que faz a escultura ser LIDA (`docs/3D/05.1` §4) —
    // é resolvida no topo da função, porque ela vale nos dois modos.
    //
    // Uma linha, simétrica, um knob: `k > 0` é fresta e escurece, `k < 0` é
    // crista e clareia. Escurecer o côncavo (sujeira, sombra de fresta) e clarear
    // o convexo (desgaste, brilho de aresta) são as DUAS METADES da mesma
    // multiplicação, porque a curvatura é *um* número com sinal.
    //
    // ⚠️ **Ela multiplica o DIFUSO e não o realce**, e isso é óptica e não
    // arrumação: uma fresta oclui a luz de AMBIENTE que chegaria por todas as
    // direções, e não o caminho especular de uma lâmpada específica — que ou
    // alcança aquele ponto ou não, e o `N·H` já responde isso. Multiplicar o
    // realce junto faria uma quina viva perder o brilho que ela é a única a ter.
    //
    // ⚠️ **E ela é aplicada ANTES da máscara**, que tinge por cima: a máscara é
    // chrome de autoria e não uma propriedade da tinta — a mesma razão pela qual
    // ela não entra no G-buffer.

    // Soma, não `screen`: o destino é HDR e quem faz o roll-off é o tonemap.
    // ⚠️ **O AO entra no DIFUSO e não no realce**, exatamente onde a cavidade já
    // entrava: `CLAY_SHINE * spec` fica de fora da multiplicação nas duas. É a
    // colocação fisicamente certa — oclusão é sobre a luz AMBIENTE que chega, e
    // um realce especular é a imagem da lâmpada, que ou está visível ou não.
    // ⚠️ **A TRANSMITANCIA entra FORA do `m`, e nao dentro.** O `m` e' a
    // resposta relativa (`AMBIENT + (1-AMBIENT) * ratio`), e a luz que
    // atravessou a peca nao e' relativa a superficie plana nenhuma. E ela
    // tambem nao leva `cav_occ`: uma fresta oclui a luz que vem de FORA, e esta
    // ja' estava dentro da materia -- ocluir aqui seria escurecer a luz pelo
    // caminho que ela nao tomou.
    var c = CLAY * m * cav_occ + CLAY * trans + CLAY_SHINE * spec;

    // A máscara entra DEPOIS da luz, sobre a cor já acesa: ela tinge o que se vê
    // em vez de mudar como a superfície responde à lâmpada. Tingir antes faria a
    // região protegida acender diferente — e a máscara passaria a ser um
    // material, que é a metade que o G-buffer recusa logo acima.
    // ⚠️ O tinto leva o MESMO `cav`: sem ele a região protegida perderia a
    // leitura de forma que o resto da peça tem, e o artista veria a máscara
    // *achatar* o relevo que ela deveria só cobrir — que é exatamente o que o
    // `MASK_STRENGTH` de 0,75 existe para não fazer.
    c = mix(c, MASK_TINT * m * cav_occ, clamp(in.mask, 0.0, 1.0) * MASK_STRENGTH);
    return vec4<f32>(c, 1.0);
}
