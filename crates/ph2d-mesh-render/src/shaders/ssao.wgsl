// Hospedado por `ph2d-mesh-render` (crates/ph2d-mesh-render/src/shaders/ssao.wgsl).
//
// **O AO DE TELA — GTAO**, o passe que mede oclusão TODO FRAME e por isso nunca
// fica velho.
//
// `docs/3D/05.1` §3 pede as duas metades e esta é a segunda:
//
//   - **Assado:** cone tracing contra o campo SDF (`ph2d-sdf::bake_ao`). Exato,
//     enxerga o corpo inteiro em qualquer direção, custo zero em runtime — e
//     obsoleto no instante em que a forma muda.
//   - **Em tela:** isto. Vê só o que está na tela, custa milissegundos por frame,
//     e é sempre a resposta sobre a forma que o artista tem debaixo do pincel.
//
// ⚠️ **Elas não competem, e a diferença não é qualidade — é ALCANCE.** O assado
// mede metros de campo; este mede um raio em torno do pixel. Quem compõe os dois
// é o barro (`mesh.wgsl`), com um `min`, e o porquê está lá.
//
// # O método é PORTADO, não inventado
//
// **GTAO** — Jimenez, Wu, Pesce, Jarabo, *"Practical Real-Time Strategies for
// Accurate Indirect Occlusion"* (SIGGRAPH 2016 Courses). A escolha é do plano, e
// a razão de ele ser o certo aqui: o SSAO clássico (Crytek 2007) conta pontos
// ocluídos numa hemisfera e devolve ruído que precisa de blur; o HBAO
// (Bavoil & Sainz 2008) acha o horizonte mas integra a visibilidade com uma
// aproximação. O GTAO acha o MESMO horizonte e resolve o arco **analiticamente**
// (Eq. 7), ponderado por cosseno — que é exatamente a mesma quantidade que o
// `bake_ao` mede com cones. As duas fontes respondem a mesma pergunta.
//
// # O que este passe lê, e por que ele não reconstrói a normal
//
// Ele lê a PROFUNDIDADE (para saber onde cada pixel está) e o **G-buffer de
// normais** que a doação já produz. Reconstruir a normal a partir da
// profundidade é o caminho usual — e aqui seria uma **segunda resposta** a *"para
// onde esta superfície aponta"*, com a primeira já rasterizada ao lado. Derivada
// da profundidade ela erra na silhueta e num vinco; vinda do vértice ela é a
// mesma que acende o barro.
//
// ⚠️ **A única conversão é a negação do `y`**, e ela é o inverso exato da última
// linha do `canvas_normal`: o G-buffer guarda a normal no espaço do RIG (`y` para
// baixo, como a tela) e a geometria deste passe vive em espaço de VISTA (`y` para
// cima). Sem ela o horizonte seria procurado do lado errado da superfície — e o
// sintoma é oclusão que aparece na crista em vez de na fresta.

struct Ssao {
    /// A inversa da projeção — é ela que devolve a posição de vista a partir da
    /// profundidade. Montada na CPU (`SsaoRaw::pack`) porque a `Camera3d` já é
    /// dona da perspectiva e invertê-la no shader seria a segunda cópia dela.
    proj_inv: mat4x4<f32>,
    /// `x` = raio em unidades de MUNDO · `y` = a escala que leva um comprimento
    /// de vista a pixels (`0.5 * altura / tan(fov/2)`) · `z` = fatias · `w` = passos.
    params: vec4<f32>,
    /// `xy` = o tamanho do alvo em pixels · `z` = a potência que ajusta o
    /// contraste · `w` = a espessura assumida de um oclusor.
    screen: vec4<f32>,
};

@group(0) @binding(0) var<uniform> cfg: Ssao;
@group(0) @binding(1) var depth_tex: texture_depth_2d;
@group(0) @binding(2) var normal_tex: texture_2d<f32>;

const PI: f32 = 3.14159265359;
const HALF_PI: f32 = 1.57079632679;

// **O TRIÂNGULO DE TELA CHEIA.** Três vértices e nenhum buffer: as coordenadas
// saem do índice. Um quad de dois triângulos custaria uma aresta diagonal onde os
// dois se encontram — e nela o rasterizador roda os quads de fragmento duas vezes.
@vertex
fn vs_fullscreen(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i32(i) & 1) << 2) - 1.0;
    let y = f32((i32(i) & 2) << 1) - 1.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

/// **ONDE ESTE PIXEL ESTÁ, em espaço de vista.**
///
/// ⚠️ O `y` do NDC é invertido em relação ao do framebuffer (o clip do wgpu tem
/// `y` para cima e o pixel `(0,0)` é o de cima), e é a linha abaixo que faz a
/// ponte. Sem ela a peça inteira fica espelhada na vertical e a oclusão aparece
/// simétrica ao lugar certo — o modo de falha que parece "quase certo".
fn view_pos(px: vec2<i32>) -> vec3<f32> {
    let d = textureLoad(depth_tex, px, 0);
    let uv = (vec2<f32>(px) + vec2<f32>(0.5)) / cfg.screen.xy;
    let ndc = vec3<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d);
    let p = cfg.proj_inv * vec4<f32>(ndc, 1.0);
    return p.xyz / p.w;
}

/// O horizonte que uma amostra propõe, já atenuado pela distância.
///
/// ⚠️ **A atenuação não é enfeite, ela é o que separa AO de silhueta.** Sem ela
/// um objeto no fundo da tela oclui um em primeiro plano só por estar na frente
/// dele em projeção — o *halo* que todo SSAO ingênuo desenha. Amarrando o
/// horizonte ao raio, o que está além dele deixa de ter voto.
fn horizon(p: vec3<f32>, v: vec3<f32>, s: vec3<f32>, best: f32) -> f32 {
    let d = s - p;
    let len = length(d);
    if (len < 1e-6) {
        return best;
    }
    let c = dot(d / len, v);
    // Peso 1 dentro do raio, caindo a 0 na borda. `saturate` de propósito: além
    // do raio a amostra não empurra o horizonte de volta, ela simplesmente não é
    // ouvida.
    let w = clamp(1.0 - len / cfg.params.x, 0.0, 1.0);
    return max(best, mix(best, c, w));
}

// ⚠️ **A SAÍDA É OCLUSÃO, não visibilidade** — `0` = nada escurece, `1` = fechado.
//
// A inversão parece arbitrária e é o contrário: ela é o que faz a AUSÊNCIA de
// medição ser gratuita. O barro precisa de um canal para ler quando ninguém mediu
// nada, e uma textura recém-criada nasce **zerada**; com esta convenção esse zero
// já quer dizer *nada aqui escurece nada*. Com visibilidade ele diria *tudo é
// sombra*, e o fallback teria de ser escrito — o que arrastaria um `queue` para
// dentro do construtor do renderizador inteiro por causa de um pixel.
@fragment
fn fs_ssao(@builtin(position) frag: vec4<f32>) -> @location(0) f32 {
    let px = vec2<i32>(frag.xy);
    let cover = textureLoad(normal_tex, px, 0);
    // **Fora da forma não há oclusão a medir.** Zero é a resposta certa e não uma
    // desistência: o barro subtrai este número, e um `1` aqui pintaria de preto o
    // fundo por onde a cena 2D aparece.
    if (cover.w < 0.5) {
        return 0.0;
    }

    let p = view_pos(px);
    // O olho está na origem em espaço de vista, então a direção para ele é
    // simplesmente `-P` normalizado.
    let v = normalize(-p);
    // A negação do `y` — o inverso da última linha do `canvas_normal`.
    let n = normalize(vec3<f32>(cover.x, -cover.y, cover.z));

    // **O raio, em PIXELS.** Um raio de mundo constante encolhe na tela quando a
    // peça se afasta, que é o comportamento certo: o que ele delimita é uma
    // vizinhança da SUPERFÍCIE, não da imagem.
    let radius_px = cfg.params.x * cfg.params.y / max(-p.z, 1e-4);
    // Menos de um pixel não tem o que marchar; menos de dois faz todo passo cair
    // no mesmo texel e a resposta vira o próprio pixel.
    if (radius_px < 2.0) {
        return 0.0;
    }

    let slices = i32(cfg.params.z);
    let steps = i32(cfg.params.w);
    let step_px = radius_px / f32(steps);

    // O ruído por pixel que troca BANDA por granulado. Sem ele as fatias caem
    // sempre nos mesmos ângulos e o resultado tem anéis visíveis; com ele o erro
    // vira alta frequência, que é o que o olho perdoa. A hash é a de Jorge
    // Jimenez (a mesma família do paper), inteira e barata.
    let noise = fract(52.9829189 * fract(dot(vec2<f32>(frag.xy), vec2<f32>(0.06711056, 0.00583715))));

    var visibility = 0.0;
    for (var s = 0; s < slices; s = s + 1) {
        let phi = (f32(s) + noise) * PI / f32(slices);
        let dir = vec2<f32>(cos(phi), sin(phi));

        // **O PLANO DA FATIA** é o que contém o olho e a direção de tela. A normal
        // dele é o que projeta a normal da superfície para dentro do plano, que é
        // onde o integral analítico do GTAO vive.
        // ⚠️ **A negação do `y` OUTRA VEZ, e por um motivo diferente do da normal:**
        // a marcha acontece em coordenadas de FRAMEBUFFER (`y` para baixo) e este
        // vetor é lido em espaço de VISTA (`y` para cima). Sem ela a fatia que o
        // integral resolve é o ESPELHO vertical da fatia que foi marchada.
        let dir_view = vec3<f32>(dir.x, -dir.y, 0.0);
        let slice_n = normalize(cross(dir_view, v));
        let proj_n = n - slice_n * dot(n, slice_n);
        let proj_len = length(proj_n);
        if (proj_len < 1e-5) {
            continue;
        }
        let pn = proj_n / proj_len;
        let tangent = cross(v, slice_n);
        let cos_n = clamp(dot(pn, v), -1.0, 1.0);
        // O sinal vem de que lado da tangente a normal caiu — é ele que diz se a
        // superfície está virada para a esquerda ou para a direita DENTRO da
        // fatia, e trocá-lo espelha a oclusão dentro de cada fatia.
        let n_angle = sign(dot(pn, tangent)) * acos(cos_n);

        // A marcha, nos dois sentidos. `best` começa em −1 = "não vejo horizonte
        // nenhum", que é o céu aberto.
        //
        // ⚠️ **`neg` marcha para −`dir` e `pos` para +`dir`, e a associação com o
        // SINAL do ângulo logo abaixo é load-bearing.** A `tangent` aponta para
        // +`dir`, e é contra ela que o `n_angle` é medido; casar o lado errado
        // com o sinal errado **estreita o arco por `2·n_angle`**, o que produz
        // auto-oclusão espúria que CRESCE com a inclinação da superfície.
        // Medido, com os dois trocados: uma parede perfeitamente CHATA de frente
        // para a câmera escurecia **12,7%** — e o modo de falha não parece um
        // sinal invertido, parece um AO "meio forte demais".
        var neg = -1.0;
        var pos = -1.0;
        for (var t = 1; t <= steps; t = t + 1) {
            // O deslocamento leva o ruído junto, senão o primeiro passo de todo
            // pixel amostra o mesmo texel vizinho e a marcha ganha um degrau.
            let off = dir * (f32(t) - 0.5 + noise) * step_px;
            neg = horizon(p, v, view_pos(px - vec2<i32>(off)), neg);
            pos = horizon(p, v, view_pos(px + vec2<i32>(off)), pos);
        }

        // Os horizontes como ÂNGULOS, um de cada lado.
        var h1 = -acos(clamp(neg, -1.0, 1.0));
        var h2 = acos(clamp(pos, -1.0, 1.0));
        // ⚠️ **O grampo contra a normal é o que torna isto uma hemisfera** e não
        // uma esfera: nada atrás da superfície pode ocluí-la, e sem estes dois
        // `max`/`min` uma face plana se auto-ocluiria pela metade.
        h1 = n_angle + max(h1 - n_angle, -HALF_PI);
        h2 = n_angle + min(h2 - n_angle, HALF_PI);

        // **O INTEGRAL ANALÍTICO** (GTAO, Eq. 7) — o arco entre os dois horizontes,
        // ponderado por cosseno. É esta linha que separa o GTAO do HBAO: o mesmo
        // horizonte, resolvido em forma fechada em vez de somado por amostras.
        let a = 0.25 * (-cos(2.0 * h1 - n_angle) + cos(n_angle) + 2.0 * h1 * sin(n_angle))
              + 0.25 * (-cos(2.0 * h2 - n_angle) + cos(n_angle) + 2.0 * h2 * sin(n_angle));
        visibility = visibility + proj_len * a;
    }

    let vis = clamp(visibility / f32(slices), 0.0, 1.0);
    // A potência é o CONTRASTE, e ela é aplicada AQUI e não no barro: o que sai
    // deste passe é a oclusão que o artista escolheu ver, e o knob do lado do
    // barro é *quanto dela entra*. Duas curvas em dois lugares seriam dois
    // controles para a mesma pergunta.
    return 1.0 - pow(vis, cfg.screen.z);
}
