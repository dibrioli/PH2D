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

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) n_view: vec3<f32>,
    @location(1) mask: f32,
    @location(2) curv: f32,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) mask: f32,
    @location(3) curv: f32,
) -> VsOut {
    var out: VsOut;
    out.clip = cam.view_proj * obj.model * vec4<f32>(pos, 1.0);
    out.mask = mask;
    // ⚠️ **A curvatura NÃO cruza `obj.model`, e a normal cruza.** Ela é
    // adimensional por construção (a divisão pelo raio médio do anel, em
    // `ph2d_mesh::curvature`), então a escala da `Pose` já se cancelou na CPU —
    // e a rotação, quando ela chegar, não move um escalar. Aplicar a matriz aqui
    // seria a mesma classe de erro do ângulo cru sobre altura no impasto: uma
    // grandeza que não tem direção passando por uma transformação que só sabe
    // falar de direção.
    out.curv = curv;
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

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let nc = canvas_normal(in.n_view);

    // Sem lâmpada acesa não há razão a computar: o barro cru é a leitura honesta
    // de "o artista apagou tudo" para uma superfície opaca.
    if (rig.n == 0u) {
        return vec4<f32>(CLAY, 1.0);
    }

    var diffuse = vec3<f32>(0.0);
    var spec = vec3<f32>(0.0);
    var flat_d = vec3<f32>(0.0);
    for (var i = 0u; i < rig.n; i = i + 1u) {
        let l = rig.lamps[i];
        // A resposta PLANA (N = (0,0,1) ⇒ N·L = L.z), que é o divisor de tudo.
        let lz = max(l.dir.z, 0.0);
        flat_d = flat_d + l.tint.rgb * lz;
        diffuse = diffuse + l.tint.rgb * max(dot(nc, l.dir.xyz), 0.0);
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

    // **A CAVIDADE** — o canal que faz a escultura ser LIDA (`docs/3D/05.1` §4).
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
    let cav = 1.0 - shade.cavity * clamp(in.curv * CAVITY_GAIN, -1.0, 1.0);

    // Soma, não `screen`: o destino é HDR e quem faz o roll-off é o tonemap.
    var c = CLAY * m * cav + CLAY_SHINE * spec;

    // A máscara entra DEPOIS da luz, sobre a cor já acesa: ela tinge o que se vê
    // em vez de mudar como a superfície responde à lâmpada. Tingir antes faria a
    // região protegida acender diferente — e a máscara passaria a ser um
    // material, que é a metade que o G-buffer recusa logo acima.
    // ⚠️ O tinto leva o MESMO `cav`: sem ele a região protegida perderia a
    // leitura de forma que o resto da peça tem, e o artista veria a máscara
    // *achatar* o relevo que ela deveria só cobrir — que é exatamente o que o
    // `MASK_STRENGTH` de 0,75 existe para não fazer.
    c = mix(c, MASK_TINT * m * cav, clamp(in.mask, 0.0, 1.0) * MASK_STRENGTH);
    return vec4<f32>(c, 1.0);
}
