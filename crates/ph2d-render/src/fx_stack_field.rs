//! **O CAMPO DE DISTÂNCIA do `FxStackPass`** — a semente sub-texel, os saltos do JFA e o finalize
//! que serve os CINCO tipos que perguntam *a que distância da borda estou, e de que lado?*.
//!
//! Irmão de [`super::fx_stack_shader`] pelo teto de LOC, e o corte é por responsabilidade: aquele
//! arquivo é o **fold** (a ingestão, a Gaussiana separável, o op pontual e a saída), este é a
//! **régua** — o único bloco do shader que mede geometria em vez de misturar cor.
//!
//! ⚠️ Ele é concatenado DEPOIS do `FX_STACK_MID_WGSL` porque consome o `dst` que aquele declara, e
//! DEPOIS do `FX_STACK_WGSL` porque consome o `tap_img`/`inner_tint`/`exact_foot` da prelude. A
//! ordem é a das dependências, não gosto.

/// O bloco WGSL do campo de distância.
pub(crate) const FIELD_WGSL: &str = r#"
// ── O CAMPO DE DISTÂNCIA (JFA limitado) ───────────────────────────────────────────────────────
//
// Por que ele existe: o modo PROXIMITY mede *quanto de fora há por perto* (o alfa invertido
// borrado). Numa reentrância o "fora" subtende um ângulo pequeno, então a sombra quase não nasce
// lá — foi o que o smoke reportou: a estrela ficava com sombra só nas pontas. A DISTÂNCIA à borda
// não tem essa dependência de ângulo: ela é 0 em TODO ponto do contorno, reentrâncias incluídas.
//
// `t1` guarda por texel o OFFSET inteiro até o texel de FORA mais próximo (`.xy`), com `.z = 1`
// quando já há semente. ⚠️ Os offsets são limitados pela banda, e f16 representa inteiros até 2048
// EXATAMENTE — o campo é exato na faixa que nos interessa, não "aproximado porque é f16".

// **Onde a fronteira REALMENTE está dentro deste texel.** O alfa anti-aliased é, perto da borda,
// uma rampa de ~1 px ao longo da normal, então a fronteira (onde ele cruza 0,5) fica a `a - 0,5` do
// centro, na direção em que o alfa DECRESCE.
//
// ⚠️ **É isto que mata o PENTE.** Semear no centro do texel faz a distância saltar em degraus
// inteiros ao andar paralelo a uma aresta obliqua — medido, 33 níveis de oscilação numa aresta a
// 21,8° (a 45° o artefato some por simetria, e foi assim que ele passou pelo primeiro gate). Com a
// semente sub-texel a distância é contínua, e a correção de meio texel que existia à mão
// DESAPARECE: ela era o caso particular disto para uma borda dura.
// ⚠️ **Só o caminho SEM geometria chega aqui.** Havendo silhueta, o finalize computa o pé exato
// por texel e os passes de semente e salto nem são despachados — deixar um braço de geometria
// nesta função seria código morto que uma mutação não faz sangrar.
fn edge_offset(p: vec2<i32>, a: f32) -> vec2<f32> {
    let gx = tap_img(t0, p.x + 1, p.y).a - tap_img(t0, p.x - 1, p.y).a;
    let gy = tap_img(t0, p.x, p.y + 1).a - tap_img(t0, p.x, p.y - 1).a;
    let g = vec2<f32>(gx, gy);
    let m = length(g);
    if (m < 1.0e-5) { return vec2<f32>(0.0); }
    // ⚠️ A rampa de anti-aliasing NÃO tem 1 px de largura: numa aresta oblíqua ela é mais larga
    // (~|nx|+|ny|), então o alfa cai mais devagar e a fronteira está mais longe do que `a − 0,5`
    // sugere. A inclinação real é `|g|/2` (diferença central), logo a distância é `2(a−0,5)/|g|`.
    // Com a suposição de 1 px o campo errava ~0,09 px, e numa borda DURA isso lê como serrilha —
    // medido, 24 níveis de variação entre texels à mesma distância na borda do contorno.
    let t = clamp(2.0 * (a - 0.5) / m, -1.5, 1.5);
    return (-g / m) * t;
}

@compute @workgroup_size(8, 8, 1)
fn cs_sdf_seed(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let me = vec2<i32>(i32(id.x), i32(id.y));
    let a = tap_img(t0, me.x, me.y).a;
    var v = vec4<f32>(0.0);
    // **A CASCA da fronteira**: um texel de DENTRO com algum vizinho de fora. Uma regra só, e ela
    // dá o campo dos DOIS lados — o sinal vem do alfa de quem pergunta, não de outra semeadura.
    // (Fora da TEXTURA conta como fora da forma: o `tap_img` devolve transparente, então uma forma
    // encostada na borda tem casca ali, que é o que mantém o campo certo no limite do scratch.)
    if (seeds_shell()) {
        if (a > 0.5) {
            let l = tap_img(t0, me.x - 1, me.y).a;
            let r = tap_img(t0, me.x + 1, me.y).a;
            let u = tap_img(t0, me.x, me.y - 1).a;
            let d = tap_img(t0, me.x, me.y + 1).a;
            if (l <= 0.5 || r <= 0.5 || u <= 0.5 || d <= 0.5) {
                v = vec4<f32>(edge_offset(me, a), 1.0, 0.0);
            }
        }
    } else if (a <= 0.5) {
        v = vec4<f32>(edge_offset(me, a), 1.0, 0.0);
    }
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), v);
}

@compute @workgroup_size(8, 8, 1)
fn cs_sdf_jump(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let me = vec2<i32>(i32(id.x), i32(id.y));
    var best = textureLoad(t1, me, 0);
    var bd = 1.0e30;
    if (best.z > 0.5) { bd = dot(best.xy, best.xy); }
    for (var j = -1; j <= 1; j = j + 1) {
        for (var i = -1; i <= 1; i = i + 1) {
            let step = vec2<i32>(i * g.jump, j * g.jump);
            let s = me + step;
            let delta = vec2<f32>(f32(step.x), f32(step.y));
            var off = vec2<f32>(0.0);
            var has = false;
            if (s.x < 0 || s.y < 0 || s.x >= i32(g.dims.x) || s.y >= i32(g.dims.y)) {
                // Fora da textura não há CASCA (a casca é feita de texels da forma), então não há
                // semente — a assimetria que a semeadura por-lado exigia morreu com ela.
            } else {
                let n = textureLoad(t1, s, 0);
                if (n.z > 0.5) { off = n.xy + delta; has = true; }
            }
            if (has) {
                let dd = dot(off, off);
                if (dd < bd) { bd = dd; best = vec4<f32>(off, 1.0, 0.0); }
            }
        }
    }
    textureStore(dst, me, best);
}

// A distância guardada no campo (ou "longe" onde o JFA não chegou).
fn field_dist(x: i32, y: i32) -> f32 {
    if (x < 0 || y < 0 || x >= i32(g.dims.x) || y >= i32(g.dims.y)) { return 1.0e6; }
    let f = textureLoad(t1, vec2<i32>(x, y), 0);
    if (f.z <= 0.5) { return 1.0e6; }
    return length(f.xy);
}

// **A normal do rebordo, pelo GRADIENTE do campo.**
//
// ⚠️ **Não use `normalize(off)`.** O vetor até a semente aponta para UMA semente, então ele salta na
// fronteira entre células de Voronoi — a distância continua exata (texels à mesma distância dão o
// mesmo número, há gate), mas a DIREÇÃO fica em degraus, e é ela que o bevel lê. Era esse o PENTE.
// O gradiente é uma diferença central de uma grandeza que já é suave, então não tem esse salto.
fn field_normal(x: i32, y: i32, fallback: vec2<f32>) -> vec2<f32> {
    let dx = field_dist(x + 1, y) - field_dist(x - 1, y);
    let dy = field_dist(x, y + 1) - field_dist(x, y - 1);
    let grad = vec2<f32>(dx, dy);
    // O gradiente cresce PARA DENTRO; a normal externa é o oposto.
    if (dot(grad, grad) < 1.0e-8 || abs(dx) > 1.0e5 || abs(dy) > 1.0e5) {
        return fallback;
    }
    return -normalize(grad);
}

// **De que COR é um texel que a cobertura passou a alcançar?** A porta única de uma pergunta com
// DOIS consumidores: o feather (que estende a borda para fora com uma rampa) e a morfologia (que
// engorda a silhueta). Os dois criam área onde a fonte não tem tinta, e os dois têm de a vestir com
// a mesma resposta — duas cópias divergiriam exatamente na franja, que é o único lugar onde isto se
// vê.
//
// ⚠️ **Onde a fonte existe, ela É a resposta** — e só onde não há nada é que se busca a borda, para
// onde `off` já aponta. A busca NÃO pode ter modo de falha: devolver preto e ainda escrever alfa
// pinta um DENTE escuro (medido, na wave do feather). A extensão é a média das cores RETAS da
// vizinhança do ponto de fronteira PESADA pela cobertura — que é exatamente
// `Σ rgb_premultiplicado / Σ alfa`, porque cada termo já vem multiplicado pelo próprio peso. Basta
// UM vizinho com tinta para a resposta existir, e no ponto de fronteira isso é garantido por
// construção.
fn straight_colour(over: vec4<f32>, src: vec2<f32>, off: vec2<f32>) -> vec3<f32> {
    if (over.a > 0.1) {
        return over.rgb / over.a;
    }
    let b = round(src + off);
    var acc = vec3<f32>(0.0);
    var wsum = 0.0;
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let s = tap_img_at(b + vec2<f32>(f32(dx), f32(dy)));
            acc = acc + s.rgb;
            wsum = wsum + s.a;
        }
    }
    if (wsum > 1.0e-4) { return acc / wsum; }
    return vec3<f32>(0.0);
}

// **O finalize sobre o CAMPO DE DISTÂNCIA** — serve QUATRO tipos: os degraus de dentro em modo
// Contour, o contorno, o feather e o bevel. Todos perguntam a mesma coisa (*a que distância da
// borda estou, e de que lado?*) e cada um responde com uma lei diferente.
@compute @workgroup_size(8, 8, 1)
fn cs_op_field(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let over = tap_img(t0, i32(id.x), i32(id.y));
    // ⚠️ O par de offset quer dizer coisas DIFERENTES conforme o tipo, e é por isso que a tabela o
    // ROTULA: numa sombra ele é um DESLOCAMENTO (amostra-se o campo mais adiante, e a banda anda
    // para o lado da luz); num bevel é uma DIREÇÃO (a luz), e deslocar por ela moveria o relevo
    // inteiro em vez de o iluminar.
    let disp = select(vec2<i32>(g.off_x, g.off_y), vec2<i32>(0), g.kind == KIND_BEVEL);
    let sx = i32(id.x) - disp.x;
    let sy = i32(id.y) - disp.y;
    let at = tap_img(t0, sx, sy);
    let inside = at.a > 0.5;
    var off = vec2<f32>(0.0);
    var far = true;
    if (g.n_segs > 0u) {
        // ⚠️ **Com geometria o JFA não responde nada — ele só PROPAGA.** Um texel que herda a
        // semente do vizinho recebe o vetor até o pé DAQUELE vizinho, não até o seu próprio: o
        // comprimento erra pouco (a envoltória de cones acerta a distância a menos de `s²/8d`),
        // mas a DIREÇÃO salta ao trocar de célula, e é dela que o bevel vive. Medido: com o campo
        // já exato pela semente, o feather caiu para 1,28 níveis e o bevel ficou em 117 — a prova
        // de que o que sobrava era a direção herdada, não o campo.
        //
        // O pé exato POR TEXEL custa o laço de segmentos onde a semente já o custava, e responde
        // as duas perguntas de uma vez.
        off = exact_foot(vec2<f32>(f32(sx), f32(sy)));
        far = false;
    } else if (sx >= 0 && sy >= 0 && sx < i32(g.dims.x) && sy < i32(g.dims.y)) {
        let f = textureLoad(t1, vec2<i32>(sx, sy), 0);
        if (f.z > 0.5) { off = f.xy; far = false; }
    }
    // ⚠️ MEIO TEXEL, e ele é DERIVADO: a casca é a primeira fileira DE DENTRO, cujo centro está a
    // 0,5 px da fronteira. Somando de dentro e subtraindo de fora, os dois lados começam em 0,5 —
    // o campo fica simétrico, que é o que um feather centrado na borda exige.
    // ⚠️ Sem correção nenhuma: a semente já aponta para a FRONTEIRA dentro do próprio texel
    // (`edge_offset`), então `|off|` É a distância. O meio texel que se somava à mão era o caso
    // particular disto para uma borda dura — e era ele que deixava o campo em degraus.
    var dist = 1.0e6;
    if (!far) {
        dist = length(off);
    }
    let sdist = select(-dist, dist, inside);
    let w = max(g.band, 1.0e-4);
    var outc: vec4<f32>;
    if (g.kind == KIND_FEATHER) {
        // A borda vira uma RAMPA CENTRADA na fronteira, sem borrar o miolo — é o que separa isto
        // de um Blur.
        //
        // ⚠️ **O ALFA é função da distância e a COR é RETA.** É a lei das três implementações
        // canônicas, e nenhuma delas reamostra cor: o feather do GIMP é um blur gaussiano da
        // MÁSCARA (σ = raio/3,5), o do Krita é uma gaussiana com `channelFlags(false, true)` — só
        // alfa —, e nos layer styles a cor entra DEPOIS, como fill.
        //
        // A lei anterior compunha `base * f` com `base` PREMULTIPLICADO, ou seja o alfa saía
        // `a_fonte · f` quando devia sair `f`: a cobertura era contada DUAS vezes, e só na fileira
        // do contorno (a única com `a_fonte` parcial). E a cor da metade de fora era buscada
        // andando `dir·0,5` a partir de `off`, com `dir` derivado de um `off` quase nulo — perto do
        // contorno ele desandava (medido: 50° fora) e o passo caía em texel transparente, com o
        // fallback disparando de forma intermitente. O resultado renderizado não era uma linha
        // escura: eram **459 texels de alfa ZERO** espalhados por 206 linhas, cercados por forma
        // dos dois lados. Um FURO, e a intermitência é o que o olho lê como tracejado.
        //
        // Agora não há direção a adivinhar: **onde a fonte existe, ela É a resposta** (o contorno
        // inteiro cai aqui, que é exatamente onde o furo nascia), e só onde não há nada é que se
        // busca a borda — para onde `off` já aponta, sem passo e sem fallback.
        // ⚠️ Um peso ao QUADRADO foi construído dentro do `straight_colour` e REMOVIDO por medição,
        // para ninguém o reintroduzir: o argumento era que um vizinho de alfa 1/255 carrega uma cor
        // reta destruída pela quantização (a tinta premultiplicada arredonda para (1,1,0), cuja cor
        // reta é (255,255,0)). Verdade — e IRRELEVANTE: esse vizinho pesa 1/255 sobre um
        // `Σ alfa ≈ 4`, ou seja 0,1% de uma cor 4× errada = **1 nível**. A mutação que troca o
        // quadrado pelo linear NÃO sangra, e foi ela que expôs que o número que eu usara para
        // justificar o quadrado (7255 níveis) era um defeito do GATE, não do peso.
        let straight = straight_colour(over, vec2<f32>(f32(sx), f32(sy)), off);
        let f = smoothstep(-w * 0.5, w * 0.5, sdist);
        outc = mix(over, vec4<f32>(straight * f, f), g.opacity);
    } else if (g.kind == KIND_MORPHOLOGY) {
        // **GROW / SHRINK — a silhueta anda `grow_px` ao longo da própria normal.** Com o campo de
        // distância na mão isto não é um kernel, é um LIMIAR: o conjunto novo é `{sdist + r > 0}`,
        // e é por isso que o elemento estruturante sai um DISCO EUCLIDIANO exato — o `feMorphology`
        // do SVG usa um RETÂNGULO (quinas quadradas) e o Photoshop tem de oferecer *Preserve:
        // Roundness* como opção; aqui a régua já é a distância, então a forma redonda é a barata.
        //
        // ⚠️ **A rampa é LINEAR e de um texel, e isso é o anti-aliasing, não um gosto:** a
        // cobertura de uma aresta reta a distância `d` do centro do texel é `d + 0,5` recortada em
        // `[0,1]`, que é a mesma lei que todo renderer de SDF usa. Um `smoothstep` aqui daria uma
        // borda visivelmente mais macia que a da fonte, e o degrau anunciaria a própria passagem.
        if (g.grow_px == 0.0) {
            // ⚠️ **ZERO é o NEUTRO, e o neutro é BYTE-IDÊNTICO.** O slider é BIPOLAR, então o
            // artista atravessa o zero a arrastar — e re-derivar a cobertura a partir do campo
            // devolveria um anti-aliasing *quase* igual ao da fonte, ou seja um pisca na passagem.
            // Um degrau que não faz nada não pode tocar num texel.
            outc = over;
        } else {
            let a = clamp(sdist + g.grow_px + 0.5, 0.0, 1.0);
            let straight = straight_colour(over, vec2<f32>(f32(sx), f32(sy)), off);
            outc = mix(over, vec4<f32>(straight * a, a), g.opacity);
        }
    } else if (g.kind == KIND_BEVEL) {
        // O relevo da borda: a face virada para a LUZ clareia, a oposta escurece, e o efeito morre
        // para o miolo. `off` aponta para a borda mais próxima, então ele É a normal 2D do rebordo.
        var shade = 0.0;
        if (!far && inside) {
            // ⚠️ **Com o pé exato, a normal NÃO se estima: ela É `off`.** Por definição de ponto
            // mais próximo, o vetor do texel até o pé é perpendicular à silhueta — então derivar
            // um gradiente do campo aqui seria estimar por diferenças finitas o que já se tem
            // exato. O `field_normal` fica para o caminho sem geometria.
            var n = normalize(off + vec2<f32>(1.0e-6, 0.0));
            if (g.n_segs == 0u) { n = field_normal(sx, sy, n); }
            let lit = vec2<f32>(f32(g.off_x), f32(g.off_y));
            let l = select(vec2<f32>(0.0, -1.0), normalize(lit), dot(lit, lit) > 0.0);
            // ⚠️ **O relevo é a INCLINAÇÃO do rebordo, e ela é ZERO na silhueta.**
            //
            // O perfil antigo (`1 − smoothstep(0, w, dist)`) vale **1 em `dist = 0`**, ou seja
            // punha o valor EXTREMO do sombreado no texel mais externo da forma: o lado escuro
            // saía preto no fio da borda e o claro saía branco. Era isso que o smoke reportou como
            // "linhas pretas" — não um artefato numérico, mas o perfil errado.
            //
            // Um bevel é uma quina arredondada: a superfície começa PLANA na silhueta, sobe pela
            // banda e volta a ficar plana no miolo. Com a altura `h(t) = smoothstep(0,1,t)`, a
            // componente horizontal da normal é `h'(t) = 6t(1−t)` — que se anula nas DUAS pontas e
            // pica no meio da banda. Normalizada ao pico: `4t(1−t)`.
            //
            // É a mesma figura que o Bevel & Emboss do Photoshop desenha (a faixa de luz mora
            // DENTRO da banda, não no contorno), e mata a linha dura sem tocar no campo.
            let t = clamp(dist / w, 0.0, 1.0);
            shade = dot(n, l) * (4.0 * t * (1.0 - t)) * g.opacity;
        }
        let colour = select(tint_lin(), vec3<f32>(1.0), shade > 0.0);
        outc = inner_tint(over, colour, abs(shade) * g.tint.a);
    } else if (is_inner()) {
        // ⚠️ **`sdist`, com SINAL — e é aqui que o Inner Shadow deslocado se conserta.**
        //
        // Com a distância sem sinal, um texel cujo ponto amostrado cai FORA da forma tem `dist`
        // grande outra vez, então a sombra DESVANECE justamente do lado onde ela devia estar
        // saturada: a banda descola do contorno e deixa uma tira clara entre a borda e a sombra.
        // Medido numa aresta reta com deslocamento 8 (luminância por profundidade, tinta crua 180):
        // `110 96 81 64 45 24 3 9 31 52 …` — o ponto MAIS ESCURO ficava 7 texels dentro, e a borda
        // saía 3,6× mais clara que ele. Uma sombra interna é mais escura NA BORDA, sempre.
        //
        // Com sinal, o lado de fora satura (`smoothstep` de negativo é 0 ⇒ força 1) e o perfil
        // volta a ser monótono a partir da borda — que é o que a máscara-invertida-deslocada do
        // Photoshop desenha, e o que o modo Proximity (que borra uma REGIÃO, não uma distância)
        // sempre desenhou.
        //
        // ⚠️ Sem deslocamento é **byte-idêntico** ao anterior: para um texel de dentro
        // `sdist == +dist`, e um de fora é morto pelo `over.a` do `inner_tint`.
        outc = inner_tint(over, tint_lin(), (1.0 - smoothstep(0.0, w, sdist)) * g.tint.a * g.opacity);
    } else if (g.kind == KIND_GLOW) {
        // **GLOW em modo Contour**: uma banda de largura constante ao longo de TODO o contorno.
        //
        // O irmão Proximity (o borrão da silhueta) mede *quanta forma há por perto*, então o vão
        // entre duas pontas de uma estrela quase não brilha — o mesmo ângulo-subtendido que faz o
        // Inner Shadow não escurecer uma reentrância. A distância não tem essa dependência: ela é
        // zero em todo ponto do contorno.
        //
        // A queda vale exatamente 0 em `w` (por isso o `op_reach` deste caso é `w`, não `3σ`), e o
        // halo entra POR BAIXO da entrada — a mesma composição do irmão e do contorno, porque um
        // op tem de devolver UMA camada.
        let a = (1.0 - smoothstep(0.0, w, max(-sdist, 0.0))) * g.tint.a * g.opacity;
        let halo = vec4<f32>(tint_lin() * a, a);
        outc = over + halo * (1.0 - over.a);
    } else {
        // CONTORNO: a borda cai exatamente em `w`, com ~1 px de anti-aliasing. Isto é uma DILATAÇÃO
        // de verdade (`d <= w`), ao contrário do corte num campo borrado, que ENCOLHE na quina
        // convexa — medido, uma ponta de 36° não recebia contorno NENHUM.
        let outward = max(-sdist, 0.0);
        let cov = 1.0 - smoothstep(w - 0.5, w + 0.5, outward);
        let a = cov * g.tint.a * g.opacity;
        let halo = vec4<f32>(tint_lin() * a, a);
        outc = over + halo * (1.0 - over.a);
    }
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), outc);
}
"#;
