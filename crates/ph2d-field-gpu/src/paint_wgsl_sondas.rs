//! ⭐⭐⭐ **AS SONDAS E O RICOCHETE, em WGSL** — a segunda metade do corpo do pintor.
//!
//! # Por que um TERCEIRO ficheiro
//!
//! O [`super::paint`] monta, o [`super::paint_wgsl`] declara o que o pintor LÊ, e este integra o
//! **hemisfério**: as sondas de irradiância, a recolha por pixel, a assadura e os dois borrões.
//!
//! ⛔ **Split, nunca allowlist** (`CLAUDE.md` §5.0). ⚠️ E a fronteira **não foi escolhida** — ela já
//! estava escrita no próprio texto, entre *«o que o pintor lê»* e *«o que ele integra»*.
//!
//! ⚠️ **As duas metades são UM shader**, concatenadas por quem monta: o corte é o tecto a pedir uma
//! fronteira, e não duas leis.

/// ⭐⭐⭐ **AS SONDAS e o RICOCHETE, em WGSL** — a segunda metade do corpo do pintor.
///
/// ⚠️ **Ela é CONCATENADA com a [`PINTOR`] por quem monta**, e não substituída: as duas são o mesmo
/// shader, e o corte é só o tecto de LOC a pedir uma fronteira. *A fronteira que ele escolheu é a
/// que já estava lá: acima, o que o pintor LÊ; abaixo, o hemisfério que ele INTEGRA.*
pub(crate) const PINTOR_SONDAS: &str = r"
// ⭐⭐⭐ **AS SONDAS DE IRRADIÂNCIA** — `ph2d_field_render::probes`, linha a linha (`docs/Render3d/08`
// §14). ⛔ Aqui viveu a recolha POR PIXEL (`ricochete_em`), que o report do dono de 2026-09-17
// (*«um reflexo mal feito»*) mostrou ser uma soma de projecções DURAS da peça — a fita está no
// cabeçalho daquele módulo. A cura é trocar ONDE se recolhe: em pontos fixos, com o pixel a
// interpolar.
fn sonda_raio() -> f32 { return max(s.ball_radius, 1e-3) * PROBE_MARGIN; }
fn sonda_passo() -> f32 { return 2.0 * sonda_raio() / f32(PROBE_GRID - 1u); }
fn sonda_canto() -> vec3<f32> { return s.ball_center - vec3<f32>(sonda_raio()); }
fn sonda_pos(x: u32, y: u32, z: u32) -> vec3<f32> {
    return sonda_canto() + vec3<f32>(f32(x), f32(y), f32(z)) * sonda_passo();
}
fn sonda_idx(x: u32, y: u32, z: u32) -> u32 { return (z * PROBE_GRID + y) * PROBE_GRID + x; }

// A base real de harmónicas esféricas até `l = 2` — os mesmos nove literais da CPU.
fn sh_base(d: vec3<f32>) -> array<f32, 9> {
    var y: array<f32, 9>;
    y[0] = 0.282095;
    y[1] = 0.488603 * d.y;
    y[2] = 0.488603 * d.z;
    y[3] = 0.488603 * d.x;
    y[4] = 1.092548 * d.x * d.y;
    y[5] = 1.092548 * d.y * d.z;
    y[6] = 0.315392 * (3.0 * d.z * d.z - 1.0);
    y[7] = 1.092548 * d.x * d.z;
    y[8] = 0.546274 * (d.x * d.x - d.y * d.y);
    return y;
}

// A irradiância (a média da radiância pesada pelo cosseno) que a sonda `k` entrega à normal `n`.
fn sh_irradiancia(k: u32, n: vec3<f32>) -> vec3<f32> {
    let b = k * SH_STRIDE;
    var y = sh_base(n);
    var e = vec3<f32>(0.0);
    for (var m: u32 = 0u; m < 9u; m = m + 1u) {
        var a = y[m];
        if (m >= 4u) { a = a * SH_A2; } else if (m >= 1u) { a = a * SH_A1; }
        e = e + a * vec3<f32>(sondas[b + m * 3u], sondas[b + m * 3u + 1u], sondas[b + m * 3u + 2u]);
    }
    return max(e, vec3<f32>(0.0));
}

// ⭐⭐⭐ **RECOLHER nos pixels**: as oito sondas da célula, pesadas por trilinear × «está à frente»
// — a `gather_probes` da CPU, na mesma ordem.
fn recolhe_sondas(p: vec3<f32>, n_vista: vec3<f32>) -> vec3<f32> {
    let n = vista_para_mundo(n_vista);
    let passo = sonda_passo();
    let canto = sonda_canto();
    let ng = f32(PROBE_GRID - 1u);
    let u = clamp((p - canto) / passo, vec3<f32>(0.0), vec3<f32>(ng));
    let c0 = min(vec3<u32>(floor(u)), vec3<u32>(PROBE_GRID - 2u));
    let f = clamp(u - vec3<f32>(c0), vec3<f32>(0.0), vec3<f32>(1.0));
    let lift = s.hit_eps * 4.0;
    var soma = vec3<f32>(0.0);
    var peso = 0.0;
    for (var dz: u32 = 0u; dz < 2u; dz = dz + 1u) {
        for (var dy: u32 = 0u; dy < 2u; dy = dy + 1u) {
            for (var dx: u32 = 0u; dx < 2u; dx = dx + 1u) {
                let x = c0.x + dx;
                let y = c0.y + dy;
                let z = c0.z + dz;
                let k = sonda_idx(x, y, z);
                if (sondas[k * SH_STRIDE + 27u] > 0.5) { continue; }
                let tri = select(1.0 - f.x, f.x, dx == 1u)
                        * select(1.0 - f.y, f.y, dy == 1u)
                        * select(1.0 - f.z, f.z, dz == 1u);
                let sp = sonda_pos(x, y, z);
                let para = sp - p;
                let dist = length(para);
                if (dist <= lift) { continue; }
                let dir = para / dist;
                let hf = (dot(n, dir) + 1.0) * 0.5;
                let w = tri * hf * hf;
                if (w <= 1e-6) { continue; }
                // ⛔ Aqui viveu um raio de visibilidade pixel→sonda; saiu por medição — ver a
                // recusa no doc da `gather_probes_por` da CPU (piorava o pé das paredes).
                soma = soma + w * sh_irradiancia(k, n);
                peso = peso + w;
            }
        }
    }
    if (peso > 0.0) { return soma / peso; }
    return vec3<f32>(0.0);
}

// ⭐⭐⭐ **ASSAR as sondas** — um GRUPO por sonda, uma thread por direcção, e a projecção em nove
// coeficientes reduzida na memória partilhada. Não há buffer de radiância: cada direcção marcha,
// pergunta a cor fosca de onde bateu (`devolvida_de`, a mesma do pixel) e some-se no coeficiente.
//
// ⚠️ **O controlo é UNIFORME de propósito:** todas as threads avaliam o mesmo `dentro` (o campo na
// posição da sonda), logo o `return` cedo é o mesmo para as 256 e as barreiras ficam legais.
var<workgroup> parcial: array<vec3<f32>, 256>;
@compute @workgroup_size(256, 1, 1)
fn assa_sondas(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
    let k = wg.x;
    let j = lid.x;
    let x = k % PROBE_GRID;
    let y = (k / PROBE_GRID) % PROBE_GRID;
    let z = k / (PROBE_GRID * PROBE_GRID);
    let pos = sonda_pos(x, y, z);
    let base = k * SH_STRIDE;
    // Colada à superfície conta como dentro: a marcha acertaria em todas as direcções no 1.º passo.
    let dentro = field(pos) < s.hit_eps * 4.0;
    if (dentro) {
        if (j == 0u) {
            for (var m: u32 = 0u; m < 27u; m = m + 1u) { sondas[base + m] = 0.0; }
            sondas[base + 27u] = 1.0;
        }
        return;
    }
    let d = direccao_do_cone(j, PROBE_DIRS);
    var r: Raio;
    r.o = pos;
    r.d = d;
    // Até onde um raio vai: a diagonal da caixa da grelha.
    let alcance = 2.0 * sonda_raio() * SQRT3;
    let h = marcha_ate(r, alcance);
    var l = vec3<f32>(0.0);
    if (h.x >= 0.0) { l = devolvida_de(pos + d * h.x, h.yzw, d); }
    var y9 = sh_base(d);
    for (var m: u32 = 0u; m < 9u; m = m + 1u) {
        workgroupBarrier();
        parcial[j] = (SH_PESO * y9[m]) * l;
        workgroupBarrier();
        for (var salto: u32 = 128u; salto > 0u; salto = salto >> 1u) {
            if (j < salto) { parcial[j] = parcial[j] + parcial[j + salto]; }
            workgroupBarrier();
        }
        if (j == 0u) {
            let v = parcial[0];
            sondas[base + m * 3u] = v.x;
            sondas[base + m * 3u + 1u] = v.y;
            sondas[base + m * 3u + 2u] = v.z;
        }
    }
    if (j == 0u) { sondas[base + 27u] = 0.0; }
}

// ⭐⭐⭐ **O RICOCHETE SUAVIZADO — a MESMA vizinhança do céu**, canal a canal.
//
// ⚠️⚠️ **Ele é suavizado pela mesma razão que a oclusão**, e o doc do
// `ph2d_field_render::blur_occlusion` escreve-a: o que aquele borrão apaga hoje não é ruído de
// amostragem — são as **estrias do conjunto discreto de direcções**. O ricochete corre no MESMO
// conjunto, logo tem a mesma assinatura. *E é por isso que ele tem de ser um CANAL: um valor
// calculado e consumido na mesma invocação não tem vizinhos para suavizar.*
// ⭐ **UMA passagem da recolha `3×3` guardada pela normal.** `liso = 0` lê os slots CRUS (o que a
// `borra_ricochete` faz); `liso = 1` lê os já borrados uma vez (o que o pintor faz ao ler) — e as
// duas juntas são as `ph2d_field_render::BOUNCE_BLUR_PASSES` passagens da lei.
//
// ⚠️ **É UMA função com dois chamadores de propósito:** duas cópias da mesma recolha divergiriam no
// dia em que alguém afinasse o `BLUR_COS` numa delas, e a segunda passagem deixaria de ser a mesma
// lei da primeira. *É a mesma razão que a `ph2d_field_render::occlusion::para_cada_vizinhanca` já
// escreve do lado da CPU.*
fn borra_de(x: u32, y: u32, i: u32, n0: vec3<f32>, liso: u32) -> vec3<f32> {
    var soma = vec3<f32>(0.0);
    var cont = 0u;
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let xx = i32(x) + dx;
            let yy = i32(y) + dy;
            if (xx < 0 || yy < 0 || xx >= i32(s.w) || yy >= i32(s.h)) { continue; }
            let j = u32(yy) * s.w + u32(xx);
            let c = centro[j];
            if (c.x < 0.0) { continue; }
            if (dot(n0, c.yzw) < BLUR_COS) { continue; }
            let b = base_do_ricochete(j) + liso * 3u;
            soma = soma + vec3<f32>(luz[b], luz[b + 1u], luz[b + 2u]);
            cont = cont + 1u;
        }
    }
    let b = base_do_ricochete(i) + liso * 3u;
    if (cont > 0u) { return soma / f32(cont); }
    return vec3<f32>(luz[b], luz[b + 1u], luz[b + 2u]);
}

fn borra_uma_vez(x: u32, y: u32, i: u32, n0: vec3<f32>) -> vec3<f32> {
    return borra_de(x, y, i, n0, 0u);
}

fn ricochete_no_pixel(x: u32, y: u32, i: u32, n0: vec3<f32>) -> vec3<f32> {
    return borra_de(x, y, i, n0, 1u);
}

// A luz que UM material devolve ao olho, já com o olhar — o `shade_render::radiance` da CPU.
// ⭐⭐⭐ **AS DUAS METADES DE UM PIXEL** — o que vai para o ECRÃ e o que a CENA tem.
//
// ⚠️ **O brilho lê a segunda** (`docs/Render3d/12` §11): ele nasce em cena-linear, **antes** do
// olhar, e é isso que o torna honesto — ler depois dele seria colher o que o tonemapper já
// comprimiu. ⛔ Devolver só o ecrã obrigaria o passe do halo a desfazer o olhar, que não é
// invertível.
//
// ⚠️ **Com o brilho desligado isto não custa nada:** a `cena` é o valor que o `vt_to_display` já
// consome, e o que muda é só ele viajar mais uma função acima.
struct Luz {
    ecra: vec3<f32>,
    cena: vec3<f32>,
}

fn luz_do_material(m: Mat, n: vec3<f32>, v: vec3<f32>, p: vec3<f32>, i: u32, ceu_vis: f32, ric: vec3<f32>, k: f32, k_estilo: f32) -> Luz {
    // ⭐⭐⭐ **A OCLUSÃO É A SOMBRA DO CÉU** — ela multiplica o que o AMBIENTE entrega, e mais nada.
    // Não toca nas lâmpadas (que têm sombra a sério) nem na emissão.
    var rgb = mx_indirect(m, n, v) * ceu_vis;
    // ⭐⭐⭐ **E A OUTRA METADE DO HEMISFÉRIO: a luz que as SUPERFÍCIES devolvem.**
    //
    // ⚠️ **É uma SEGUNDA chamada à mesma lei indirecta, com o ambiente trocado** — exactamente o
    // que o `shade_render` faz com o `SoIrradiancia`. Ela não leva o `ceu_vis`: a oclusão é a
    // sombra do CÉU, e a luz que vem das superfícies não é céu.
    //
    // ⛔ **Fora deste ramo o quadro é o de sempre, AO BIT** — `ricochete` nasce a zero e o
    // `ambiente_e_ricochete` a `false`.
    if (ric.x > 0.0 || ric.y > 0.0 || ric.z > 0.0) {
        ricochete = ric;
        ambiente_e_ricochete = true;
        rgb = rgb + mx_indirect(m, n, v);
        ambiente_e_ricochete = false;
    }
    // ⭐⭐⭐ **A SATURAÇÃO DA LUZ INDIRECTA** (`ph2d_style`, a `W8`) — aqui, sobre as DUAS parcelas
    // de ambiente e **antes das lâmpadas**: saturar no fim saturaria também o realce do sol.
    // ⚠️ Com o valor de fábrica (`1`) isto é a identidade AO BIT, por construção.
    rgb = st_saturate_indirect(pintor.estilo, rgb);
    let piso = PISO_LUZ * PISO_LUZ;
    let base = i * passo_da_luz();
    // ⭐⭐⭐ **AS LUZES-OBJECTO, uma a uma** — a direcção e a distância de cada saem do PONTO deste
    // pixel, e a soma é sobre a RADIÂNCIA, como a CPU faz.
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        let d = s.lamps[l].xyz - p;
        let cru = dot(d, d);
        // ⚠️ **O piso protege DUAS grandezas.** Abaixo dele a direcção é a NORMAL: com a luz sobre
        // o ponto, `d` é o vector ZERO, a direcção normalizada sai `(0,0,0)` e o pixel ficaria PRETO.
        var to_light = n;
        if (cru > piso) {
            let inv = 1.0 / sqrt(cru);
            to_light = mundo_para_vista(d * inv);
        }
        // ⭐ **A sombra entra na radiância que CHEGA** — não no `N·L` e não no resultado.
        let chega = pintor.lamp[l].rgb * luz[base + 1u + l] / max(cru, piso);
        rgb = rgb + mx_direct(m, n, v, to_light, chega);
    }
    // ⭐⭐⭐ **O ESTILO ENTRA AQUI, entre a física e o olhar** — o gémeo exacto do
    // `ph2d_field_render::shade_render`. ⛔ Depois do olhar seria tinta sobre um valor já cortado, e
    // um contorno que não respira com a exposição separa-se da peça ao expor.
    //
    // ⚠️ **`|N·V|`**, e o módulo não é defensivo: numa silhueta o produto passa por zero e muda de
    // sinal com o ruído da normal, e um contorno que pisca não é um contorno.
    // ⚠️ **A curvatura viaja em unidades da PEÇA** (`knobs.w` é o raio), senão o mesmo botão daria
    // outra tinta numa peça grande e numa pequena.
    //
    // ⛔⛔ **E ela é a `k_estilo`, NUNCA a `k` do material** (auditoria de 2026-09-19): as duas são
    // a mesma grandeza medida a distâncias DIFERENTES — o material pede o óptimo de precisão, o
    // estilo pede a escala que o artista escolheu, e é essa a única alavanca sobre a dureza da
    // borda. *Escrever `k` aqui traria o degrau de volta sem uma linha de lei ter mudado.*
    let cena = st_apply(
        pintor.estilo,
        rgb + mx_emission(m, n, v),
        abs(dot(n, v)),
        k_estilo * pintor.knobs.w,
    );
    return Luz(vt_to_display(cena, pintor.knobs.x, pintor.modo.x), cena);
}

// ⭐⭐ **Sombreia DUAS vezes e mistura o RESULTADO**, nunca os materiais: um metal e um dieléctrico
// a meio caminho não são um meio-metal. E só paga o dobro onde há fronteira.
fn radiancia(p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, i: u32, ceu_vis: f32, ric: vec3<f32>) -> Luz {
    let d = dono_mix(p, pintor.knobs.y);
    let ma = ler_mat(d.a);
    let mb = ler_mat(d.b);
    // ⭐⭐⭐ **A CURVATURA mede-se quando ALGUÉM a lê, e UMA vez** — o material deste pixel (a
    // subsuperfície maciça) ou o ESTILO da cena (`modo2.z`, a tinta por curvatura da `W8`).
    //
    // ⚠️⚠️ **São DUAS portas somadas e não uma:** perguntar só ao material faria o artista mexer na
    // tinta de aresta e a peça não mudar um pixel — *a grandeza que o botão escolhe nunca teria sido
    // medida*, que é a forma de knob morto que esta casa mede desde 30/08. O gémeo da CPU faz a
    // mesma soma no `smoke_draw_thread`.
    //
    // ⛔ **Sem leitor, ZERO amostras de campo** — o `curv_eps` nem chega a ser lido, e o quadro é o
    // de sempre ao bit.
    //
    // ⭐⭐⭐ **E SÃO DUAS MEDIÇÕES, porque são DUAS PERGUNTAS** — o gémeo exacto do que o
    // `smoke_draw_thread` faz na CPU. Cada uma só corre se o consumidor DELA estiver vivo, logo o
    // caminho de omissão continua a não pagar amostra nenhuma.
    var k = 0.0;
    var k_estilo = 0.0;
    let mat_le = mat_le_curvatura(ma) || (d.t > 0.0 && mat_le_curvatura(mb));
    if (mat_le) { k = curvatura_em(p); }
    // ⚠️ **O `ε` do estilo DERIVA-SE aqui do que já viaja** — a suavidade (fracção, `estilo.extra.y`)
    // vezes o raio da peça (`knobs.w`) —, e é a MESMA aritmética que a
    // `Presentation::curvature_eps` faz na CPU. ⛔ Enviá-lo já multiplicado seria a segunda resposta
    // à mesma pergunta, e a que passa a discordar no dia em que uma das duas mude.
    if (pintor.modo2.z != 0u) { k_estilo = curvatura_do_estilo_em(p); }
    // ⚠️ O gatherer do ricochete (`ler_mat_fosca`) NÃO recebe curvatura, e a CPU também não: ali a
    // subsuperfície maciça lê curvatura `0`, que o piso transforma no raio de `100`. *A paridade
    // daquele caminho é por construção, e não por um número.*
    let ca = luz_do_material(com_a_curvatura(ma, k), n, v, p, i, ceu_vis, ric, k, k_estilo);
    if (d.t <= 0.0) { return ca; }
    let cb = luz_do_material(com_a_curvatura(mb, k), n, v, p, i, ceu_vis, ric, k, k_estilo);
    // ⚠️ **Cada metade mistura-se na SUA unidade**, e não é a mesma conta: o ecrã mistura valores
    // JÁ passados pelo olhar (o `mixed_radiance` da CPU) e a cena mistura os de antes dele (o
    // `mixed_radiance_scene`). ⛔ Misturar uma e derivar a outra daria um terceiro programa — o
    // olhar não é linear.
    return Luz(
        ca.ecra + (cb.ecra - ca.ecra) * d.t,
        ca.cena + (cb.cena - ca.cena) * d.t,
    );
}

// A curva do `ph2d_color::srgb::linear_to_srgb_byte`, com o mesmo arredondamento.
fn srgb_byte(linear: f32) -> u32 {
    let v = clamp(linear, 0.0, 1.0);
    var e = v * 12.92;
    if (v > 0.0031308) { e = 1.055 * pow(v, 1.0 / 2.4) - 0.055; }
    return u32(clamp(e * 255.0 + 0.5, 0.0, 255.0));
}

fn empacota(c: vec4<f32>) -> u32 {
    let a = u32(clamp(clamp(c.w, 0.0, 1.0) * 255.0 + 0.5, 0.0, 255.0));
    return srgb_byte(c.x) | (srgb_byte(c.y) << 8u) | (srgb_byte(c.z) << 16u) | (a << 24u);
}

// A direcção PARA o observador — o raio do traçado, ao contrário.
fn direccao_de_vista(d: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(-dot(d, s.right), -dot(d, s.up), -dot(d, s.fwd));
}

// ⭐⭐⭐ **A LEI DO CHÃO QUE SÓ RECEBE** — `luz que chega com a peça / luz que chegaria sem ela`, em
// luminância, sobre a difusa branca. É o `shade_render::catcher` da CPU, linha a linha.
//
// ⚠️ **As duas somas correm as mesmas contas na mesma ordem**, e onde nada tapa elas são o MESMO
// número: a razão sai exactamente `1` e o pixel fica com os bytes do fundo.
fn fator_do_chao(i: u32, q: vec3<f32>, v: vec3<f32>) -> f32 {
    let m = mat_do_chao();
    let n = mundo_para_vista(vec3<f32>(0.0, 1.0, 0.0));
    let ceu = mx_indirect(m, n, v);
    let base = i * passo_da_luz();
    var livre = ceu;
    var chega = ceu * luz[base];
    let piso = PISO_LUZ * PISO_LUZ;
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        let d = s.lamps[l].xyz - q;
        let cru = dot(d, d);
        var to_light = n;
        if (cru > piso) { to_light = mundo_para_vista(d * (1.0 / sqrt(cru))); }
        let rad = pintor.lamp[l].rgb / max(cru, piso);
        livre = livre + mx_direct(m, n, v, to_light, rad);
        chega = chega + mx_direct(m, n, v, to_light, rad * luz[base + 1u + l]);
    }
    let luma = vec3<f32>({LUMA_R}, {LUMA_G}, {LUMA_B});
    let a = luma.x * chega.x + luma.y * chega.y + luma.z * chega.z;
    let b = luma.x * livre.x + luma.y * livre.y + luma.z * livre.z;
    if (!(b > 0.0)) { return 1.0; }
    return clamp(a / b, 0.0, 1.0);
}

// O fundo com a sombra do chão por cima, em linear pré-multiplicado — o `shadowed_background` da CPU.
fn fundo_sombreado(f: f32) -> vec4<f32> {
    return vec4<f32>(pintor.fundo.rgb * f, (1.0 - f) + pintor.fundo.a * f);
}

// ⭐⭐⭐ **A LUZ QUE A PEÇA DEVOLVE AO PONTO `q` DO CHÃO** — bilinear sobre as quatro células, vezes
// o esmorecimento da orla. É o `GroundBounce::sample` da CPU, linha a linha.
//
// ⚠️ **Fora do campo devolve ZERO**, e a orla já lá pôs zero antes da borda: a saída antecipada é
// guarda de índice, e a LEI é a orla (ver o doc da irmã de CPU).
fn ricochete_do_chao(q: vec3<f32>) -> vec3<f32> {
    let n = pintor.modo2.y;
    if (n < 2u) { return vec3<f32>(0.0); }
    let passo = pintor.chao_campo.z;
    if (!(passo > 0.0)) { return vec3<f32>(0.0); }
    let lado = f32(n - 1u);
    let u = vec2<f32>(
        (q.x - pintor.chao_campo.x) / passo,
        (q.z - pintor.chao_campo.y) / passo,
    );
    if (u.x < 0.0 || u.y < 0.0 || u.x > lado || u.y > lado) { return vec3<f32>(0.0); }
    let c0 = vec2<u32>(
        min(u32(floor(u.x)), n - 2u),
        min(u32(floor(u.y)), n - 2u),
    );
    let f = vec2<f32>(
        clamp(u.x - f32(c0.x), 0.0, 1.0),
        clamp(u.y - f32(c0.y), 0.0, 1.0),
    );
    var soma = vec3<f32>(0.0);
    for (var dz = 0u; dz < 2u; dz = dz + 1u) {
        for (var dx = 0u; dx < 2u; dx = dx + 1u) {
            var w = 1.0 - f.x;
            if (dx == 1u) { w = f.x; }
            var wz = 1.0 - f.y;
            if (dz == 1u) { wz = f.y; }
            let k = ((c0.y + dz) * n + c0.x + dx) * 3u;
            soma = soma + (w * wz) * vec3<f32>(chao_luz[k], chao_luz[k + 1u], chao_luz[k + 2u]);
        }
    }
    // A orla: medida no QUADRADO do campo (Chebyshev), como na CPU.
    let meia = passo * lado * 0.5;
    if (!(meia > 0.0)) { return vec3<f32>(0.0); }
    let centro = vec2<f32>(pintor.chao_campo.x + meia, pintor.chao_campo.y + meia);
    let d = max(abs(q.x - centro.x), abs(q.z - centro.y)) / meia;
    return soma * clamp((1.0 - d) / {FADE}, 0.0, 1.0);
}

// ⭐ **A luz devolvida ao chão DESTE pixel** — `[0,0,0]` quando ele não vê chão nenhum.
fn chao_no_pixel(x: u32, y: u32) -> vec3<f32> {
    if (s.chao == 0u) { return vec3<f32>(0.0); }
    let r = ray_at_plane(raio(f32(x) + 0.5, f32(y) + 0.5));
    let q = chao_em(r);
    if (q.w == 0.0) { return vec3<f32>(0.0); }
    let e = ricochete_do_chao(q.xyz);
    if (e.x <= 0.0 && e.y <= 0.0 && e.z <= 0.0) { return vec3<f32>(0.0); }
    // ⭐⭐ **A MESMA lei indirecta com o ambiente trocado** que a peça usa para o ricochete dela —
    // o espelho exacto do `SoIrradiancia` da CPU. ⚠️ Escrever aqui uma segunda expressão para «a
    // resposta do material a uma irradiância» seria a segunda cópia de uma lei que já tem porta.
    ricochete = e;
    ambiente_e_ricochete = true;
    let saiu = mx_indirect(
        mat_do_chao(),
        mundo_para_vista(vec3<f32>(0.0, 1.0, 0.0)),
        direccao_de_vista(r.d),
    );
    ambiente_e_ricochete = false;
    return saiu;
}

// ⭐⭐ **A luz devolvida de uma BORDA** — a média dos vizinhos de cruz que falham a peça, na MESMA
// ordem do `edge_ground_bounce` da CPU. ⚠️ **Sem vizinho de fundo devolve ZERO** (e o irmão do
// factor devolve `1,0`): ausência de LUZ, nunca luz inventada.
fn chao_da_borda(i: u32, x: u32, y: u32) -> vec3<f32> {
    if (s.chao == 0u) { return vec3<f32>(0.0); }
    if (centro[i].x < 0.0) { return chao_no_pixel(x, y); }
    var soma = vec3<f32>(0.0);
    var n = 0u;
    if (x > 0u && centro[i - 1u].x < 0.0) { soma = soma + chao_no_pixel(x - 1u, y); n = n + 1u; }
    if (x + 1u < s.w && centro[i + 1u].x < 0.0) { soma = soma + chao_no_pixel(x + 1u, y); n = n + 1u; }
    if (y > 0u && centro[i - s.w].x < 0.0) { soma = soma + chao_no_pixel(x, y - 1u); n = n + 1u; }
    if (y + 1u < s.h && centro[i + s.w].x < 0.0) { soma = soma + chao_no_pixel(x, y + 1u); n = n + 1u; }
    if (n == 0u) { return vec3<f32>(0.0); }
    return soma / f32(n);
}

// O factor do chão DESTE pixel — `1,0` quando ele não vê chão nenhum.
fn fator_no_pixel(j: u32, x: u32, y: u32) -> f32 {
    let r = ray_at_plane(raio(f32(x) + 0.5, f32(y) + 0.5));
    let q = chao_em(r);
    if (q.w == 0.0) { return 1.0; }
    return fator_do_chao(j, q.xyz, direccao_de_vista(r.d));
}

// ⭐⭐ **O factor de uma BORDA** — o do próprio pixel quando ele falha a peça; senão, a média dos
// vizinhos de cruz que a falham (esquerda, direita, cima, baixo — a ordem da CPU).
fn fator_da_borda(i: u32, x: u32, y: u32) -> f32 {
    if (s.chao == 0u) { return 1.0; }
    if (centro[i].x < 0.0) { return fator_no_pixel(i, x, y); }
    var soma = 0.0;
    var n = 0u;
    if (x > 0u && centro[i - 1u].x < 0.0) { soma = soma + fator_no_pixel(i - 1u, x - 1u, y); n = n + 1u; }
    if (x + 1u < s.w && centro[i + 1u].x < 0.0) { soma = soma + fator_no_pixel(i + 1u, x + 1u, y); n = n + 1u; }
    if (y > 0u && centro[i - s.w].x < 0.0) { soma = soma + fator_no_pixel(i - s.w, x, y - 1u); n = n + 1u; }
    if (y + 1u < s.h && centro[i + s.w].x < 0.0) { soma = soma + fator_no_pixel(i + s.w, x, y + 1u); n = n + 1u; }
    if (n == 0u) { return 1.0; }
    return soma / f32(n);
}

// ⭐⭐⭐ **O INTERIOR: um pixel, um material, uma escrita.**
// ⭐⭐⭐ **A PASSAGEM QUE RECOLHE O RICOCHETE NAS SONDAS E O GUARDA** (`docs/Render3d/08` §12, §14).
//
// ⚠️⚠️ **Ela vive no PINTOR e não na marcha, e é uma decisão:** as sondas precisam do MATERIAL do
// ponto acertado, e a tabela de materiais é o grupo `1` deste passe. ⇒ *quem marcha não sabe de que
// cor é o que ele acertou.*
//
// ⚠️ **E ela é um despacho SEPARADO porque a suavização precisa dos VIZINHOS** — a mesma razão que
// faz a re-amostragem da borda ser um segundo despacho e não uma linha dentro do primeiro.
@compute @workgroup_size(8, 8, 1)
fn pinta_ricochete(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    if (c.x < 0.0) { return; }
    let r = ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5));
    let devolvida = recolhe_sondas(r.o + r.d * c.x, c.yzw);
    let b = base_do_ricochete(i);
    luz[b] = devolvida.x;
    luz[b + 1u] = devolvida.y;
    luz[b + 2u] = devolvida.z;
}

// ⭐⭐⭐ **A PRIMEIRA DAS DUAS PASSAGENS DE BORRÃO** — ver `ph2d_field_render::BOUNCE_BLUR_PASSES`.
//
// ⚠️⚠️ **Ela tem de ser um DESPACHO com destino PRÓPRIO.** A `ricochete_no_pixel` já faz uma recolha
// de `3×3` ao LER, o que dá a segunda passagem de graça; a primeira não pode ser feita no mesmo
// sítio porque escrever onde os vizinhos ainda estão a ler é uma **corrida**. ⇒ os seis slots do
// canal: o CRU e o de uma passagem.
//
// ⛔ **E ela não pode ser um núcleo MAIOR numa recolha só**, que seria de graça: medido na peça do
// dono, `5×5` e `7×7` numa passagem baixam o `p99` (`0,401 → 0,33`/`0,32`) e **sobem o MÁXIMO**
// (`1,81 → 2,84`/`3,21`), enquanto duas passagens de `3×3` baixam os dois (`0,293` / `1,50`).
// *A guarda da normal aplicada a CADA salto é transitiva — uma vizinhança GEODÉSICA, que não
// atravessa um vinco; uma recolha larga guardada pelo centro atravessa-o quando as duas pontas por
// acaso concordam.*
@compute @workgroup_size(8, 8, 1)
fn borra_ricochete(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    let d = base_do_ricochete_liso(i);
    if (c.x < 0.0) {
        luz[d] = 0.0; luz[d + 1u] = 0.0; luz[d + 2u] = 0.0;
        return;
    }
    let v = borra_uma_vez(g.x, g.y, i, c.yzw);
    luz[d] = v.x;
    luz[d + 1u] = v.y;
    luz[d + 2u] = v.z;
}

@compute @workgroup_size(8, 8, 1)
fn pinta(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    if (c.x < 0.0) {
        // ⭐ **O CHÃO**: onde ele é tapado o fundo escurece; onde nada o tapa a razão é exactamente
        // `1`. ⚠️ **O fundo é COPIADO** nesse caso, e não passa pela conversão — a cerca da CPU.
        let f = fator_no_pixel(i, g.x, g.y);
        // ⭐⭐⭐ **A LUZ QUE A PEÇA PÕE NO CHÃO** — somada em pré-multiplicado e com alfa ZERO, que
        // é o que um compositor lê como luz acrescentada. ⛔ Ela NÃO pode multiplicar o fundo: o do
        // modelador é transparente, e a wave inteira sairia num pixel que não muda um bit.
        let posta = chao_no_pixel(g.x, g.y);
        let tem_posta = posta.x != 0.0 || posta.y != 0.0 || posta.z != 0.0;
        if (f < 1.0) {
            let base = fundo_sombreado(f);
            saida[i] = empacota(vec4<f32>(base.rgb + posta, base.a));
        } else if (tem_posta) {
            saida[i] = empacota(vec4<f32>(pintor.fundo.rgb + posta, pintor.fundo.a));
        } else {
            saida[i] = pintor.modo.z;
        }
        // ⭐⭐⭐ **O FUNDO NÃO ENTRA NA CADEIA DO BRILHO** — o gémeo exacto do `campo_de_cena` da
        // CPU: *«só os píxeis da PEÇA entram; o fundo é uma cor de bytes que nunca passou pelo
        // olhar, e pô-lo aqui seria inventar uma luz de cena que ninguém autorou»*.
        if (pintor.modo2.w != 0u) { cena_hdr[i] = vec4<f32>(0.0); }
        return;
    }
    let r = ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5));
    // ⭐ O PONTO reconstrói-se do `t` — a mesma álgebra do `Rays::point_at`.
    let p = r.o + r.d * c.x;
    let luz_px = radiancia(
        p,
        c.yzw,
        direccao_de_vista(r.d),
        i,
        ceu_em(g.x, g.y, i, c.yzw),
        ricochete_no_pixel(g.x, g.y, i, c.yzw),
    );
    saida[i] = empacota(vec4<f32>(luz_px.ecra, 1.0));
    // ⭐⭐⭐ **E a CENA deste pixel fica guardada para o brilho** — ver `Luz`. ⚠️ Quem escreve é
    // **só** este passe: o `campo_de_cena` da CPU lê o CENTRO mesmo nos píxeis de silhueta, logo o
    // `pinta_bordas` não pode tocar aqui. *Uma segunda escrita faria a cadeia ler outro quadro.*
    if (pintor.modo2.w != 0u) { cena_hdr[i] = vec4<f32>(luz_px.cena, 0.0); }
}

// ⭐⭐⭐ **A BORDA: quatro sub-amostras, média em LINEAR DE ECRÃ.**
//
// ⚠️ A média é das luzes **já transformadas** — é isso que o olho vê, e transformar a média de duas
// luzes da cena não é a mesma coisa.
@compute @workgroup_size(64, 1, 1)
fn pinta_bordas(@builtin(global_invocation_id) g: vec3<u32>) {
    let slot = g.x;
    if (slot >= pintor.modo.y) { return; }
    let i = bitcast<u32>(borda[slot * 5u].x);
    if (i >= s.w * s.h) { return; }
    let x = i % s.w;
    let y = i / s.w;
    let c = centro[i];
    let r = ray_at_plane(raio(f32(x) + 0.5, f32(y) + 0.5));
    let v = direccao_de_vista(r.d);
    let p = r.o + r.d * c.x;
    let ceu_vis = ceu_em(x, y, i, c.yzw);
    // ⚠️ **As sub-amostras partilham o ricochete do CENTRO**, exactamente como partilham o ponto e
    // o material — a mesma aproximação declarada da borda, e pela mesma razão.
    let ric = ricochete_no_pixel(x, y, i, c.yzw);
    // ⭐⭐ **O fundo de uma sub-amostra que falha é o fundo COM o chão** — sem isto a silhueta de
    // baixo pinta um fio do fundo limpo entre a peça e a sombra de contacto.
    let f_chao = fator_da_borda(i, x, y);
    var fundo = pintor.fundo;
    if (f_chao < 1.0) { fundo = fundo_sombreado(f_chao); }
    // ⭐ **E a borda recebe a luz devolvida pela MESMA média dos vizinhos** — ver `chao_da_borda`.
    fundo = vec4<f32>(fundo.rgb + chao_da_borda(i, x, y), fundo.a);
    var acc = vec4<f32>(0.0);
    for (var j = 0u; j < 4u; j = j + 1u) {
        let q = borda[slot * 5u + 1u + j];
        var cor = fundo;
        if (q.x >= 0.0) { cor = vec4<f32>(radiancia(p, q.yzw, v, i, ceu_vis, ric).ecra, 1.0); }
        acc = acc + cor * 0.25;
    }
    saida[i] = empacota(acc);
}
";
