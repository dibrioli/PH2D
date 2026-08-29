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
    // `(largura, altura, _, _)` em pixels — a régua que converte um empurrão em
    // PIXELS num empurrão em NDC. Ver `CameraRaw::viewport`.
    viewport: vec4<f32>,
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
    // **O WIREFRAME PODE REMOVER LINHA ESCONDIDA PELA NORMAL?** `1` só numa malha
    // FECHADA. Ver `ObjectRaw::wire_cull`.
    wire_cull: f32,
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
    // **QUANTO DO AMBIENTE COM DIREÇÃO ENTRA.** `0` = o piso escalar de ontem,
    // ao byte; `1` = o estúdio (`ph2d_light::env_ambient`).
    //
    // ⚠️ **Ele ocupou o ÚLTIMO `_pad`** — o `trans_scale` tinha levado o outro, e
    // o comentário de lá dizia *"sobra um"*. O `size_of` continua em 32 B.
    env: f32,
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

// **O AMBIENTE COM DIREÇÃO** — o piso acima, redistribuído por onde a normal
// olha. ⚠️ São `ph2d_light::ENV_BASE` e `ph2d_light::ENV_SLOPE`, e a igualdade é
// gateada pelo MESMO teste que já pina o AMBIENT: duas cópias dariam uma sombra
// de estúdio no barro diferente da que a tinta ao lado recebe, sob o mesmo card.
//
// A irradiância de um ambiente linear na altura é `c + (2/3)·k·(n·cima)`, e o
// `2/3` é o `Â₁` da convolução zonal com o lóbulo cosseno (Ramamoorthi &
// Hanrahan 2001) — já embutido no SLOPE. Um ambiente linear **não tem** grau 2,
// então isto é a resposta EXATA e não a barata (medido: 3e-6 contra a integral).
const ENV_BASE: vec3<f32> = vec3<f32>(0.946, 1.002, 1.137);
const ENV_SLOPE: vec3<f32> = vec3<f32>(0.3, 0.383, 0.518);

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

// O tinto do PREVIEW do padrão — o que o próximo traço vai depositar, mostrado
// no barro antes de o artista tocá-lo.
//
// ⚠️ **VIOLETA, e a escolha não é gosto:** o barro é claro e quente, a máscara é
// AZUL-FRIA, e o cursor é ÂMBAR. Um preview em qualquer um desses três leria
// como *"isto está protegido"* ou como *"aqui está a mira" — as duas frases
// erradas. Ele tem de ler como uma quarta coisa.
const PREVIEW_TINT: vec3<f32> = vec3<f32>(0.62, 0.34, 0.66);

// Quanto o barro cede ao tinto no pico do padrão.
//
// ⚠️ **MENOR que o da máscara, e é o ponto inteiro:** o artista olha o preview
// para julgar se o padrão cabe na FORMA dele, então o que ele não pode perder é
// a forma. A máscara pode ser opaca porque a pergunta dela é *"cobri a dobra?"*;
// a deste é *"esta densidade serve para esta peça?"*, que se responde vendo o
// relevo por baixo.
const PREVIEW_STRENGTH: f32 = 0.45;

// ============================ O MATCAP ============================
//
// **O que um matcap É:** sombreamento que é função APENAS da normal em espaço de
// vista. A luz viaja com a câmera, então orbitar não muda a leitura da forma — é
// por isso que todo app de escultura o oferece, e é a razão de ele NÃO ser
// substituível pelo rig: o rig é do DOCUMENTO (a mesma lâmpada acende a tinta ao
// lado), o matcap é do OLHO.
//
// ⚠️ **UMA IMAGEM, e não mais a fórmula.** Até 2026-08-10 isto era um punhado de
// cores e expoentes avaliados aqui, e o doc deste bloco defendia a escolha com
// uma premissa que era verdade na época: *"seria o caminho certo se houvesse
// matcaps AUTORADOS para carregar; não há — seriam assets novos, com licença,
// num repo que não os tem"*. A premissa caiu por MEDIÇÃO e não por gosto: os
// oito do Blender trazem um `license.txt` dizendo **CC0** e o do SculptGL vem de
// um repositório **MIT**. Com os assets em mãos, o resto do argumento antigo
// (*"uma textura sintetizada seria a MESMA função assada e depois interpolada"*)
// deixou de valer — estas imagens **não são** aquela função, e nenhuma
// quantidade de lâmpadas analíticas as alcança. Ver `crate::matcap`.
//
// ⚠️ **A coordenada é `n.xy * 0.5 + 0.5`, SEM flip, e é o que só um render
// revela.** O `canvas_normal` já devolve o normal em espaço de TELA, onde `y`
// cresce para BAIXO; a linha 0 de uma textura também é o topo. Os dois eixos já
// concordam. Com um flip a escultura acenderia por BAIXO enquanto a tinta ao
// lado, no mesmo documento e sob a mesma lâmpada, acenderia por cima.
//
// ⚠️ **Os cantos da imagem nunca são amostrados, e isso é geometria:** `|n.xy|`
// nunca passa de 1 num vetor unitário, então o domínio é o disco INSCRITO. É por
// isso que estes arquivos podem ter fundos diferentes entre si (o `Basic Side` é
// preto, o `Studio` é cinza) sem que a diferença signifique coisa alguma.
fn matcap_uv(n: vec3<f32>) -> vec2<f32> {
    return n.xy * 0.5 + vec2<f32>(0.5, 0.5);
}

// A esfera autorada, amostrada pela normal — em LINEAR, porque a textura é
// `Rgba8UnormSrgb` e o hardware desfaz a transferencia de graca na leitura. O
// alvo do passe e' HDR e o tonemap do shell vem depois, exatamente como no
// caminho do rig.
//
// ⚠️ **`textureSampleLevel` e nao `textureSample`.** Esta funcao e' chamada duas
// vezes por pixel — uma na normal e outra no CENTRO, para o divisor plano — e a
// segunda tem coordenada CONSTANTE. Uma amostragem com derivada implicita num
// uv constante pede mip 0 de qualquer forma, mas so' e' legal fora de fluxo
// divergente; o nivel explicito torna as duas chamadas validas onde elas estao,
// dentro do `if (shade.matcap > 0u)`. A textura nao tem mips.
fn matcap_shade(n: vec3<f32>) -> vec3<f32> {
    return textureSampleLevel(matcap_tex, sss_samp, matcap_uv(n), 0.0).rgb;
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

// **A IMAGEM DO MATCAP** — a esfera autorada que o `matcap_shade` amostra.
//
// ⚠️ **Mora no grupo 3 e divide o sampler do SSS, e as duas coisas são
// deliberadas.** Os quatro grupos que o wgpu garante já estavam ocupados (0 =
// uniforms · 1 = por-objeto · 2 = AO · 3 = SSS), então não havia um quinto para
// pedir; e o sampler que o SSS precisa — LINEAR com `ClampToEdge` nos dois eixos
// — é exatamente o que uma imagem de matcap quer. Um segundo sampler idêntico
// seria uma segunda resposta para *"como se amostra uma tabela deste passe"*.
//
// ⚠️ **É UMA textura, não um array de nove.** O artista vê um matcap por vez, e
// o `ensure_matcap` reescreve ESTA textura quando ele troca — 1 MB de VRAM em
// vez de 9, e oito PNGs que nunca são decodificados enquanto ninguém os pede.
// O preço é um upload por clique, que é o gesto mais lento que existe na UI.
@group(3) @binding(2) var matcap_tex: texture_2d<f32>;

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
    @location(6) preview: f32,
    @location(3) ao: f32,
    @location(4) curv_world: f32,
    @location(5) thickness: f32,
    // **DE QUE LADO DA PEÇA ESTE PONTO ESTÁ** — `n · (olho − p)`, no espaço de
    // vista. Só o wireframe a lê; o barro a ignora.
    //
    // ⚠️ **Ela é perspectiva-correta de propósito, e o atalho `n_view.z` foi
    // MEDIDO:** ele erra proporcionalmente ao ângulo do raio contra o eixo da
    // câmera — ou seja **erra mais na borda do quadro**, que é exatamente onde
    // esta grandeza decide alguma coisa. Vazamento com ele: **0,3 %** na esfera
    // grossa e 0,5 % no toro, contra **0,0 %** nos dois com a forma correta.
    @location(7) facing: f32,
};

fn vs_core(
    pos: vec3<f32>,
    normal: vec3<f32>,
    mask: f32,
    curv: f32,
    ao: f32,
    curv_world: f32,
    thickness: f32,
    preview: f32,
) -> VsOut {
    var out: VsOut;
    out.clip = cam.view_proj * obj.model * vec4<f32>(pos, 1.0);
    out.mask = mask;
    // Um peso adimensional, como a máscara: `obj.model` não o toca.
    out.preview = preview;
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
    // O olho é a ORIGEM do espaço de vista, então o vetor ponto→olho é `-p_view`.
    let p_view = (cam.view * obj.model * vec4<f32>(pos, 1.0)).xyz;
    out.facing = dot(normalize(out.n_view), normalize(-p_view));
    return out;
}

// **QUANTO A ARESTA SE APROXIMA DO OLHO** — em profundidade NDC, e é o número
// que faz o wireframe existir.
//
// ⚠️ **O viés de profundidade do pipeline NÃO alcança uma LINHA, e isso é spec,
// não bug do driver:** ele é definido para POLÍGONOS. Medido em 2026-08-12,
// varrendo `constant` de `0` a `-4096` e `slope_scale` de `0` a `-16`: a tinta
// que chega à tela é **exatamente a mesma** nos quatro pontos. Enquanto o corte
// foi lido como z-fighting, a cura óbvia era um viés maior — e ela é INERTE.
//
// ⚠️ **É por isso que a nudge mora AQUI e não no `DepthBiasState`:** o vertex
// shader é o único lugar que o wgpu garante alcançar toda topologia, em todo
// backend.
//
// ⚠️ **Ela é um deslocamento em `z` de CLIP proporcional a `w`**, o que a torna
// um deslocamento CONSTANTE em NDC — e por isso não move um pixel em `x`/`y`:
// a linha continua exatamente sobre a aresta, ela só ganha a disputa de
// profundidade. Um empurrão no espaço de VISTA (aproximar o vértice do olho)
// deslocaria a linha na tela por perspectiva, e a aresta passaria a desenhar ao
// lado de si mesma na silhueta.
//
// ⚠️ **A NUDGE SOZINHA NÃO BASTA, e a razão é geometria:** perto da silhueta a
// face da frente e a de trás CONVERGEM em profundidade, então qualquer
// deslocamento constante grande o bastante para uma linha de frente vencer o
// próprio triângulo é grande o bastante para o fio do outro lado da peça
// atravessar. As duas metades do mesmo número, puxadas em sentidos opostos.
// Quem separa as duas é o descarte por normal do [`fs_wire`], e é ELE que
// libertou este valor de ter um orçamento de vazamento.
//
// ⚠️ **O valor é MEDIDO** (`probe_wire_continuity.rs`), na régua do MIOLO
// ESTRITO — as arestas cujas duas pontas encaram o olho com folga, num sólido
// convexo, onde toda a aresta tem de chegar sob qualquer lei:
//
// | nudge | miolo | vazada |
// |---|---|---|
// | 0 (o que shipava antes do 1º report) | **45 %** | 0,0 % |
// | **3e-3** | **86 %** | **0,0 %** |
// | 6e-3 · 1,2e-2 · 2,4e-2 · 4,8e-2 | 86 % | 0,0 % |
//
// ⇒ **Ela SATURA em 3e-3**, e é isso que diz que os 14 % que faltam não são
// disputa de profundidade (mais empurrão não os compra): são a sobreposição de
// pixels entre arestas vizinhas dentro da máscara do próprio oráculo.
//
// ⚠️ **E subir a nudge PIORA a peça não-convexa**, que é o que fecha a escolha:
// num TORO o miolo estrito inclui arestas que encaram o olho e estão ATRÁS do
// tubo da frente. A 3e-3 elas ficam corretamente escondidas (50 %); a 6e-3 elas
// atravessam e o número SOBE para 87 % — um oráculo melhorando enquanto a
// remoção de superfície escondida piora.
//
// ⚠️ **DUAS formas mais espertas foram construídas, MEDIDAS e REJEITADAS** — não
// as refaça:
//
// 1. **`nudge / sec(ângulo)`**, reproduzindo o termo de INCLINAÇÃO que o viés de
//    polígono teria a partir da normal do vértice: um `normalize` e uma divisão
//    por vértice por ~1 %;
// 2. **`nudge × facing`** (e a versão em DEGRAU, `facing > 0 ? nudge : 0`), para
//    o vértice de costas não ganhar empurrão nenhum: as duas zeram o vazamento
//    — e as duas derrubam o miolo para **73 %**, porque uma aresta que CRUZA a
//    silhueta tem uma ponta de cada lado e o empurrão interpolado morre no meio
//    dela. O vazamento tem de ser cortado no FRAGMENTO, onde a pergunta é feita
//    por pixel; no vértice ela leva a metade da frente junto.
const WIRE_DEPTH_NUDGE: f32 = 3.0e-3;

// ⛔⛔ **E o `DepthBiasState` do pipeline de arestas era CÓDIGO MORTO — confirmado
// pela plataforma em 2026-08-29, na subida do `wgpu` 28 → 29.**
//
// O `pipeline_build` declarava, para o passe de arestas,
// `DepthBiasState { constant: -4, slope_scale: -1.0 }`, com um comentário longo a
// explicar que *"o viés negativo é o que faz a aresta ganhar da face"*. O
// raciocínio está certo **para triângulos**; aquela topologia é `LineList`. O
// WebGPU exige `depthBias`, `depthBiasSlopeScale` e `depthBiasClamp` **iguais a
// zero** fora de topologia de triângulos, e o Vulkan aplica viés só a primitivas
// de polígono ⇒ aquele valor **nunca foi aplicado por backend nenhum**.
//
// ⭐ **A casa já o tinha medido** — a sonda `probe_wire_continuity` traz, escrito,
// que *"não há um gate afirmando «o viés do pipeline não alcança uma linha», que é
// o achado mais caro desta investigação — porque eu não consigo fazê-lo falhar
// pelo motivo que ele alegaria"*, com a varredura ao lado: `constant` de **0 a
// −4096**, tinta **idêntica ao pixel**. A cura verdadeira é esta constante (mais o
// empurrão lateral abaixo, que ataca a outra metade). O campo do pipeline ficou
// para trás porque **nada o obrigava a sair**.
//
// ⇒ O `wgpu` 29 obrigou: ele valida o que o 28 tolerava, e a criação do pipeline
// passou a falhar com *"Depth bias is not compatible with non-triangle topology
// LineList"* — **48 gates de uma vez**, e foi assim que o cadáver apareceu.
// ⚠️ *Um valor inerte não custa nada até ao dia em que a plataforma deixa de o
// tolerar; nesse dia custa a suíte inteira — e enquanto lá está, quem o lê pela
// primeira vez acredita no comentário.*
//
// ⚠️ O gate que faltava agora existe, e não é nosso: **é a própria validação do
// `wgpu`**. Quem repuser um viés naquele pipeline não vê um teste vermelho — vê a
// malha 3D inteira deixar de desenhar.

// **A ARESTA** — a mesma posição do [`vs_main`], um passo mais perto do olho.
//
// ⚠️ **Ele delega em vez de recalcular:** duas expressões para *"onde este
// vértice cai na tela"* divergiriam no dia em que a `Pose` ganhasse rotação, e o
// wireframe passaria a anotar uma forma que não é a desenhada.
@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) mask: f32,
    @location(3) curv: f32,
    @location(4) ao: f32,
    @location(5) curv_world: f32,
    @location(6) thickness: f32,
    @location(7) preview: f32,
) -> VsOut {
    return vs_core(pos, normal, mask, curv, ao, curv_world, thickness, preview);
}

// **O EMPURRÃO LATERAL** — o fio sai meio pixel para FORA, e só onde a face
// raspa o olho.
//
// ⚠️ **Ele ataca uma metade DIFERENTE do problema que a [`WIRE_DEPTH_NUDGE`]
// ataca, e a medição é quem diz isso.** Separando o miolo estrito por quanto a
// face encara o olho (sonda `where_the_interior_miss_lives`), a tinta que falta
// **não** está espalhada: ela mora no bin RASANTE.
//
// | facing | cobertura (esfera 32x64 / 64x128) | peso deste empurrão |
// |---|---|---|
// | **0,20-0,40** | **79 % / 75 %** | **0,91** |
// | 0,40-0,60 | 100 % / 96 % | 0,75 |
// | 0,60-0,80 | 103 % / 100 % | 0,51 |
// | 0,80-1,00 | 100 % / 97 % | 0,19 |
//
// ⇒ O buraco está exatamente onde `1 − facing²` é quase 1. Uma nudge maior não o
// compra (ela satura em 3e-3); um deslocamento LATERAL sim, porque perto da
// silhueta a linha não perde a disputa de profundidade — ela é **coberta** pelo
// próprio triângulo, que ali se projeta quase de perfil.
//
// ⚠️ **O MEIO PIXEL é medido, não herdado do Blender** (embora caia no mesmo
// número), e o que fecha a escolha é o que acontece do outro lado dele:
//
// | px | continuidade (32x64 / 64x128 / toro) | arestas inteiras | vazada |
// |---|---|---|---|
// | 0 (o controle) | 43,0 / 48,6 / 28,6 % | 699 | 0,0 % |
// | **0,5** | **45,4 / 49,0 / 33,4 %** | **786** | **0,0 %** |
// | 0,75 | 43,7 / 47,4 / 31,3 % | 827 | 0,3 % |
// | 1,0 | 39,8 / 44,6 / 28,8 % | 766 | 1,4 % |
//
// A 0,75 o vazamento já cruza a barra de 0,1 % do gate, e a 1,0 a **continuidade
// cai abaixo do controle**: passado meio pixel o fio deixa de estar sobre a
// própria aresta e passa a mentir sobre onde a geometria está. O ganho no bin
// rasante da esfera fina é **75,1 % -> 79,2 %**, e no toro a continuidade sobe
// **28,6 -> 33,4 %** — a peça não-convexa é a que mais ganha, porque é a que tem
// mais superfície de perfil.
//
// ⚠️ **A direção sai de uma DIFERENÇA FINITA, não da normal projetada à mão.**
// Um segundo caminho de "onde este vértice cai na tela" divergiria do
// [`vs_core`] no dia em que a `Pose` ganhasse rotação — o mesmo argumento que
// faz o [`vs_wire`] delegar em vez de recalcular. Só a DIREÇÃO é usada, então a
// não-linearidade da divisão perspectiva sobre o passo é um erro de fração de
// grau.
//
// ⚠️ **O passo é proporcional a `clip.w`** (a profundidade de vista), não um
// comprimento de mundo fixo: uma peça pequena ou uma câmera afastada mudariam o
// condicionamento da diferença, e a régua tem de acompanhar a cena.
//
// ⚠️ **`sign(facing)` empurra cada lado para FORA**, que é o que faz disto um
// alargamento da silhueta e não um deslize. Numa peça FECHADA o lado de trás é
// descartado no fragmento e o sinal nunca se vê; numa casca ABERTA ele é o que
// impede as duas metades de se empurrarem para o mesmo lado.
const WIRE_PUSH_PX: f32 = 0.5;
const WIRE_PUSH_EPS_REL: f32 = 1.0e-3;

fn wire_lateral_push(
    pos: vec3<f32>,
    normal: vec3<f32>,
    clip: vec4<f32>,
    facing: f32,
) -> vec4<f32> {
    let ratio = 1.0 - facing * facing;
    if ratio <= 0.0 || clip.w <= 0.0 {
        return clip;
    }
    let world = obj.model * vec4<f32>(pos, 1.0);
    let wn = (obj.model * vec4<f32>(normal, 0.0)).xyz;
    let nl = length(wn);
    if nl <= 0.0 {
        return clip;
    }
    let step = wn / nl * (clip.w * WIRE_PUSH_EPS_REL);
    let ahead = cam.view_proj * (world + vec4<f32>(step, 0.0));
    if ahead.w <= 0.0 {
        return clip;
    }
    // Medida em PIXELS: num viewport não-quadrado normalizar em NDC torceria a
    // direção, e o empurrão sairia mais largo num eixo que no outro.
    let d_px = (ahead.xy / ahead.w - clip.xy / clip.w) * cam.viewport.xy;
    let dl = length(d_px);
    if dl <= 0.0 {
        return clip;
    }
    // ⚠️ **PARA DENTRO, e o sinal foi um defeito MEDIDO desta wave.** `d_px` é a
    // direção da normal EXTERNA na tela; empurrar ao longo dela leva o fio para
    // FORA da silhueta, sobre o fundo, e a medição diz o que isso custa: a tinta
    // total cai monotonicamente (11984 -> 11750 -> 11495 -> 11071 em 0 / 0,5 /
    // 1 / 2 px) e o bin rasante cai de 75,1 % para 72,9 %. Invertido, os dois
    // sobem. O fio tem de correr para o CORPO da peça, que é o lado onde a
    // superfície de fato está.
    //
    // ⚠️ **E é `-sign(facing)`, não `-1`** — é isso que torna o empurrão
    // INVARIANTE à orientação. Virar o enrolamento de uma casca vira a normal
    // **e** o sinal do `facing`, e o produto dos dois não se move: as duas faces
    // da mesma folha ganham o mesmo empurrão, que é o que o
    // `an_open_shell_keeps_its_wireframe` cobra.
    let off_px = d_px / dl * (WIRE_PUSH_PX * ratio * -sign(facing));
    let off_ndc = off_px * 2.0 / cam.viewport.xy;
    return vec4<f32>(clip.xy + off_ndc * clip.w, clip.z, clip.w);
}

@vertex
fn vs_wire(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) mask: f32,
    @location(3) curv: f32,
    @location(4) ao: f32,
    @location(5) curv_world: f32,
    @location(6) thickness: f32,
    @location(7) preview: f32,
) -> VsOut {
    var out = vs_core(pos, normal, mask, curv, ao, curv_world, thickness, preview);
    out.clip.z = out.clip.z - WIRE_DEPTH_NUDGE * out.clip.w;
    out.clip = wire_lateral_push(pos, normal, out.clip, out.facing);
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

// **O PISO DA DIFUSA, NA DIREÇÃO DA NORMAL.**
//
// Hoje o barro devolvia `AMBIENT` — o MESMO número em toda direção — para
// qualquer face virada para longe da luz: duas faces na sombra, uma olhando para
// cima e outra para baixo, saíam idênticas, e na região que a lâmpada não
// alcança a escultura não tinha leitura de forma nenhuma.
//
// ⚠️ **O céu é o topo da TELA, e neste referencial o topo é `-y`** — o
// `canvas_normal` acabou de virar a normal para o frame em que o rig é autorado,
// onde `y` cresce para BAIXO. É a mesma negação que aquele comentário nomeia, e
// o oráculo dela é um RENDER: com o sinal trocado o céu ilumina por baixo, e
// isso é uma escultura que parece estar num porão.
//
// ⚠️ **Ancorado na TELA e não no mundo**, porque as lâmpadas são de tela: um
// estúdio cujo céu gira enquanto as luzes ficam paradas não é um estúdio.
fn ambient_floor(n: vec3<f32>) -> vec3<f32> {
    let e = AMBIENT * (ENV_BASE - ENV_SLOPE * n.y);
    return mix(vec3<f32>(AMBIENT), e, shade.env);
}

// **A OCLUSÃO DE FORMA — a porta ÚNICA, e a razão de ela existir.**
//
// Os três canais que este número compõe não são iluminação: são **leitura de
// FORMA** (a cavidade desenha a fresta, os dois AOs medem o quanto do céu um
// ponto enxerga), e é por isso que eles multiplicam o resultado nos DOIS modos —
// e é por isso que eles podem viajar para a tinta 2D, onde não há rig nenhum
// deste lado.
//
// ⚠️ **Uma porta e não duas cópias.** O barro pergunta isto para se acender e o
// G-buffer pergunta para DOAR. Duas expressões seriam duas respostas a *"quão
// escura é esta fresta?"*, e elas divergiriam no único lugar onde ninguém lê um
// número de volta: uma escultura que escurece de um jeito no viewport e de outro
// na tinta que ela acende, no mesmo documento, sob o mesmo card.
fn form_occlusion(curv: f32, ao: f32, frag: vec2<f32>) -> f32 {
    // **A CAVIDADE** vale nos DOIS modos, e é por isso que ela subiu para cá: ela
    // não é iluminação, é leitura de FORMA — o canal que desenha a fresta. Deixá-la
    // no caminho do rig faria o artista perder a curvatura justamente ao trocar
    // para o material que ele escolheu para ler melhor.
    let cav = 1.0 - shade.cavity * clamp(curv * CAVITY_GAIN, -1.0, 1.0);

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
    let baked = mix(1.0, clamp(ao, 0.0, 1.0), shade.ao);

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
    let screen = 1.0 - shade.ssao * screen_occlusion(frag);
    return cav * min(baked, screen);
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
//
// ⚠️ **E o SEGUNDO alvo é a OCLUSÃO DE FORMA**, pela porta acima. Ela é um alvo
// próprio e não um canal do primeiro porque o primeiro **não tem vaga**: `xyz` é
// a normal e `w` é a cobertura, e nenhum dos quatro é derivável dos outros (a
// normal é flipada para a frente pelo `canvas_normal`, então `z` seria
// reconstrutível por `sqrt(1 − x² − y²)` — mas essa raiz é mal-condicionada
// exatamente na silhueta, onde `z → 0`, que é a mesma armadilha que o doc 24 do
// Painter mediu e recusou na razão K/S). Um alvo `R16Float` custa **2 B/texel**
// contra os 8 do primeiro; o preço da doação está medido no `form_plane`.
struct GbufferOut {
    @location(0) form: vec4<f32>,
    @location(1) occlusion: f32,
};

@fragment
fn fs_gbuffer(in: VsOut) -> GbufferOut {
    var out: GbufferOut;
    out.form = vec4<f32>(canvas_normal(in.n_view), 1.0);
    out.occlusion = form_occlusion(in.curv, in.ao, in.clip.xy);
    return out;
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

// **A REMOÇÃO DE LINHA ESCONDIDA** — o que faz a borda de uma peça densa deixar
// de ser uma mancha.
//
// ⚠️ **Ela é feita AQUI e não no vértice, e a diferença é a silhueta:** uma
// aresta que a cruza tem uma ponta de cada lado, então a pergunta *"este ponto
// está do outro lado da peça?"* só tem UMA resposta por FRAGMENTO. Decidi-la no
// vértice leva a metade visível junto — medido, o miolo cai de 86 % para 73 %.
//
// ⚠️ **E ela só se arma numa malha FECHADA** (`obj.wire_cull`): numa casca
// aberta uma normal de costas é a face de TRÁS de uma folha, que o artista vê de
// frente ao girar a câmera. Ver `Mesh::is_closed`.
//
// ⚠️ **O teste de profundidade NÃO substitui isto**, e é o ponto que custou uma
// investigação: a nudge que faz a linha da frente vencer o próprio triângulo é
// exatamente o que deixa o fio de trás atravessar perto da silhueta. Medido na
// esfera 64×128, a tinta que cai onde aresta de frente nenhuma passa vai de
// **2,2 % a 0,0 %** (11,1 % na malha grossa, 14,0 % num toro).
//
// ⚠️ **E ela corrige uma MEDIÇÃO, não só o desenho:** parte do que a régua
// anterior contava como *"a aresta chegou"* era o fio de trás caindo POR CIMA do
// da frente — na esfera densa a metade oculta projeta-se sobre a inteira. Ao
// removê-lo, a mesma cena caiu de `109 %` para `86 %` no miolo estrito: a
// ilusão a sair, não cobertura a perder.
@fragment
fn fs_wire(in: VsOut) -> @location(0) vec4<f32> {
    if obj.wire_cull > 0.5 && in.facing < 0.0 {
        discard;
    }
    return WIRE_RGBA;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let nc = canvas_normal(in.n_view);

    // A leitura de FORMA — cavidade × os dois AOs — pela porta que o G-buffer
    // também atravessa, para que a tinta acesa por esta peça escureça a fresta
    // exatamente onde o barro escurece.
    let cav_occ = form_occlusion(in.curv, in.ao, in.clip.xy);

    // **O MATCAP** — a luz do OLHO, e não a do documento. Ele vem ANTES da recusa
    // por rig apagado: ele não usa o rig, então apagar as lâmpadas do card não
    // pode apagá-lo.
    if (shade.matcap > 0u) {
        // ⚠️ **O `id` não chega mais aqui, e a ausência é a wave inteira:** a
        // imagem residente É a identidade do matcap. Quem escolhe é o
        // `MeshRenderer::ensure_matcap`, na CPU, reescrevendo a textura quando o
        // artista troca de chip — o `shade.matcap` sobrevive só como o *booleano*
        // que este `if` faz, e o `> 0u` continua sendo a mesma pergunta de antes.
        let lit = matcap_shade(nc);
        // ⚠️ **O MESMO modelo relativo do caminho do rig**, e não uma segunda
        // regra: a resposta dividida pela de uma superfície PLANA sob a mesma
        // luz. Lá o divisor é `flat_d`; aqui é a imagem lida na normal frontal —
        // o **CENTRO** do disco, que é o que "plano" quer dizer neste espaço. É
        // esta razão que deixa a máscara tingir com a MESMA lei nos dois modos.
        let flat = max(matcap_shade(vec3<f32>(0.0, 0.0, 1.0)), vec3<f32>(FLAT_FLOOR));
        let ratio = lit / flat;
        var cm = lit * cav_occ;
        cm = mix(cm, PREVIEW_TINT * ratio * cav_occ, clamp(in.preview, 0.0, 1.0) * PREVIEW_STRENGTH);
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
    // ⚠️ **O piso virou um VETOR e a dobra não mudou**, e é isso que preserva o
    // contrato: em `ratio = 1` — uma superfície PLANA de frente para a luz — o
    // resultado é exatamente `1` para QUALQUER piso. O ambiente redistribui a
    // SOMBRA e não toca no que está aceso.
    let floor_e = ambient_floor(nc);
    let m = floor_e + (vec3<f32>(1.0) - floor_e) * ratio;

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
    // ⚠️ **O preview entra ANTES da máscara, e a ordem é a lei dos dois canais.**
    // A proteção é a palavra final sobre o que o traço alcança, então ela tem de
    // vencer no pixel também: se o preview pintasse por cima, o artista veria
    // padrão prometido em barro que o pincel não pode tocar. Na prática os dois
    // raramente disputam — o kernel já zera o preview onde a máscara protege —,
    // e é precisamente por isso que a ordem tem de estar certa no único caso em
    // que eles se encontram: o verbo de MÁSCARA, que não é freado por ela.
    c = mix(c, PREVIEW_TINT * m * cav_occ, clamp(in.preview, 0.0, 1.0) * PREVIEW_STRENGTH);
    c = mix(c, MASK_TINT * m * cav_occ, clamp(in.mask, 0.0, 1.0) * MASK_STRENGTH);
    return vec4<f32>(c, 1.0);
}
