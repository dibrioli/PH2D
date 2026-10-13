// ⭐ W5g (F5) — o PÉ do texel do interior na borda EXATA: o ponto de fronteira que a busca acha (o
// cruzamento rumo a um texel de fora da janela 5×5, ou a uma semente do JFA) só é o mais próximo à
// resolução da grade, e salta ao longo da face de texel para texel — junto ao ponto TANGENTE, onde a
// luz muda em poucos texels ao longo da face, saía a fronteira acesa/escura em degraus, e o salto
// depende da fase da grade (logo do zoom). A lei (o gabarito, `fora_dos_opacos`) é o ponto mais
// próximo: o raio de `P0` pela normal (secante) da borda exata (a do `c3_borda.wgsl`), até assentar.
// A semente do JFA é a da CÉLULA (4 texels finos): sem isto, o miolo saía em blocos dela.
//
// Concatenado depois de `c3_borda.wgsl` (lê `cobertura` com a saída dele: `(a, normal, recuo)`).

override PE_NA_BORDA: bool = false;
// ⭐ W5g (F5) — a lâmpada vista do ponto de fronteira: a própria face tapa a metade da abertura atrás
// da TANGENTE (conta exata), e os raios da metade da frente marcham a partir de meio texel da captura
// para fora. A marcha a partir do ponto rente lia os raios RASANTES — que roçam a borda capturada por
// uma fração de texel ao longo de um trecho comprido — ora livres ora tapados conforme a fase da
// grade, e o ruído ao longo da face projetava-se no interior em degraus (a foto do dono, cena 7).
override VIS_NA_FACE: bool = false;
override AFASTA_NA_FACE: f32 = 1.5;
// ⭐ W5o — a abertura e os raios saem do PRÓPRIO ponto da face `P`; cada raio só se testa a partir de
// onde está a `AFASTA_NA_FACE` texels do plano da face (o roçar na própria borda fica de fora como
// antes). Partir de `P + afasta` deslocava a penumbra: lá a visibilidade varia com a posição.
override FACE_PELO_PONTO: bool = false;

// Passos da busca do pé: perto da borda assenta num passo; a `r` do centro de um disco de raio `R`, o
// erro de ângulo cai para (1 − r/R) por passo.
const PASSOS_DO_PE: u32 = 6u;

// A reta da borda (texels da captura ligada) no texel `t`: `(normal para dentro, recuo)` e o centro
// do texel em `w = 1`; `w = 0` sem reta.
fn reta_da_borda(t: vec2<i32>, n: vec2<i32>) -> vec4<f32> {
    if any(t < vec2<i32>(0)) || any(t >= n) {
        return vec4<f32>(0.0);
    }
    let e = textureLoad(cobertura, t, 0);
    if e.r <= 0.0 || e.r >= 1.0 || (e.g == 0.0 && e.b == 0.0) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(e.gb, e.a, 1.0);
}

// O pé de `P0` (dentro) na borda exata, a partir do ponto de fronteira `b` (fora, rente) — texels das
// cascatas: o cruzamento do raio de P0 pela normal de fora do ponto anterior, enquanto se aproxima
// (o ponto mais próximo é onde o raio de P0 sai pela normal). Projetar na reta do texel não serve
// longe da borda: o pé cai fora do texel dela, num cheio ou num vazio, e a iteração perde-se.
fn pe_na_borda(P0: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    if !PE_NA_BORDA || !BORDA_EXATA {
        return b;
    }
    let h = 1.0 / f32(fina.fina);
    var q = b;
    var d = distance(b, P0);
    for (var i = 0u; i < PASSOS_DO_PE; i++) {
        let nf = normal_secante(q, P0, h);
        if all(nf == vec2<f32>(0.0)) {
            break;
        }
        let fim = P0 + nf * (d + 2.0 * h);
        if num_corpo(fim) {
            break;
        }
        let c = cruzamento(P0, fim, h);
        let dc = distance(c, P0);
        if dc >= d - 1e-3 * h {
            break;
        }
        q = c;
        d = dc;
    }
    return q;
}

// O ponto de fronteira entre `dentro` e `fora`, do lado de FORA, a um centésimo de texel fino `h`.
fn cruzamento(dentro: vec2<f32>, fora: vec2<f32>, h: f32) -> vec2<f32> {
    var a = dentro;
    var b = fora;
    let passos = u32(ceil(log2(max(distance(a, b) / (0.01 * h), 2.0))));
    for (var i = 0u; i < passos; i++) {
        let m = 0.5 * (a + b);
        if num_corpo(m) {
            a = m;
        } else {
            b = m;
        }
    }
    return b;
}

// A normal de FORA da borda exata no ponto de fronteira `P` (rente, fora) achado de `P0` (dentro):
// a reta do texel de P ou do seguinte rumo a P0; sem reta, a direção P0 → P.
fn normal_de_fora(P: vec2<f32>, P0: vec2<f32>) -> vec2<f32> {
    let s = ESCALA_DA_CAPTURA;
    let n = vec2<i32>(vec2<f32>(cas.tamanho) * s);
    let x = P * s;
    let v = P0 - P;
    var dentro = vec2<f32>(0.0, -1.0);
    if dot(v, v) > 1e-12 {
        dentro = normalize(v);
    }
    var r = reta_da_borda(vec2<i32>(floor(x)), n);
    if r.w == 0.0 {
        r = reta_da_borda(vec2<i32>(floor(x + 0.5 * dentro)), n);
    }
    if BORDA_EXATA && r.w != 0.0 {
        return -r.xy;
    }
    return -dentro;
}

// A fração à vista da lâmpada `k` do ponto de fronteira `P` com a normal de fora `nf`: a abertura
// cortada pela tangente, a marcha a partir de meio texel da captura para fora.
// ⭐ W5k: `translucido` — o corpo atrás da face deixa passar a sua parte (τ > 0): a metade de trás
// da abertura não é tapada, marcha-se a partir do ponto espelhado DENTRO (a partida de dentro conta a
// entrada × τ, a lei do gabarito). De `Q`, fora, o raio quase rente a uma face convexa nunca entra.
fn visibilidade_na_face(P: vec2<f32>, nf: vec2<f32>, k: u32, translucido: bool) -> f32 {
    let L = lampadas.itens[k];
    let afasta = nf * AFASTA_NA_FACE * max(1.0 / ESCALA_DA_CAPTURA, 1.0 / f32(fina.fina));
    if FACE_PELO_PONTO && !translucido {
        return visibilidade_pelo_ponto(P, nf, length(afasta), k);
    }
    let Q = P + afasta;
    let v = L.centro - Q;
    let d = length(v);
    if L.raio <= 0.0 || d <= L.raio {
        if dot(L.centro - P, nf) <= 0.0 {
            if !translucido {
                return 0.0;
            }
            return visibilidade(P - afasta, k, 0.0);
        }
        return visibilidade(Q, k, 0.0);
    }
    let alfa = asin(min(L.raio / d, 1.0));
    let base = atan2(v.y, v.x);
    var delta = atan2(nf.y, nf.x) - base;
    delta -= TAU * round(delta / TAU);
    let lo = max(-alfa, delta - 0.25 * TAU);
    let hi = min(alfa, delta + 0.25 * TAU);
    var visto = 0.0;
    if hi > lo {
        let meia = 0.5 * (hi - lo);
        visto = visivel_do_disco(Q, v, d, L.raio, base + 0.5 * (lo + hi), meia, k) * meia / alfa;
    }
    if !translucido {
        return visto;
    }
    // As duas pontas de trás, [−α, lo) e (hi, α] (uma vazia quando a face corta a abertura de um lado),
    // pelos mesmos ângulos a partir de dentro.
    let Qd = P - afasta;
    let vd = L.centro - Qd;
    let dd = length(vd);
    let tras = array<vec2<f32>, 2>(vec2<f32>(-alfa, min(lo, alfa)), vec2<f32>(max(hi, -alfa), alfa));
    for (var i = 0u; i < 2u; i++) {
        let meia = 0.5 * (tras[i].y - tras[i].x);
        if meia > 0.0 {
            visto += visivel_do_disco(Qd, vd, dd, L.raio, base + 0.5 * (tras[i].x + tras[i].y), meia, k) * meia / alfa;
        }
    }
    return visto;
}

// ⭐ W5o — a transmissão do raio de `P` pela direção `dj` até `B`, testada a partir de onde o raio está
// a `h` do plano da face (normal de fora `nf`); 1 se lá já passou de `B`.
fn pelo_ponto_ate(P: vec2<f32>, nf: vec2<f32>, h: f32, dj: vec2<f32>, B: vec2<f32>, k: u32) -> f32 {
    let s = h / max(dot(dj, nf), 1e-4);
    if s >= distance(P, B) {
        return 1.0;
    }
    let t = f32(marchar(P, P + dj * s, B).t);
    if t <= 0.0 || !DISTANTES {
        return t;
    }
    return t * fora_da_grade(P, B, k);
}

// ⭐ W5o — o [`visibilidade_na_face`] com a abertura vista do próprio `P` (opaco): a metade atrás da
// tangente tapada, os raios da frente pelo [`pelo_ponto_ate`] — a fração à vista integrada como no
// `visivel_do_disco` (fatias fixas, a bissecção onde a transmissão muda).
fn visibilidade_pelo_ponto(P: vec2<f32>, nf: vec2<f32>, h: f32, k: u32) -> f32 {
    let L = lampadas.itens[k];
    let v = L.centro - P;
    let d = length(v);
    if L.raio <= 0.0 || d <= L.raio {
        if dot(v, nf) <= 0.0 || d <= 0.0 {
            return select(0.0, 1.0, d <= L.raio && L.raio > 0.0);
        }
        return pelo_ponto_ate(P, nf, h, v / d, L.centro, k);
    }
    let alfa = asin(min(L.raio / d, 1.0));
    let base = atan2(v.y, v.x);
    var delta = atan2(nf.y, nf.x) - base;
    delta -= TAU * round(delta / TAU);
    let lo = max(-alfa, delta - 0.25 * TAU);
    let hi = min(alfa, delta + 0.25 * TAU);
    if hi <= lo {
        return 0.0;
    }
    let n = RAIOS_DIRETOS;
    let largura = (hi - lo) / f32(n);
    var a0 = base + lo;
    var t0 = pelo_ponto_ao_disco(P, nf, h, v, d, L.raio, a0, k);
    var soma = 0.0;
    for (var i = 0u; i < n; i++) {
        let a1 = a0 + largura;
        let t1 = pelo_ponto_ao_disco(P, nf, h, v, d, L.raio, a1, k);
        if abs(t1 - t0) < 1e-4 {
            soma += 0.5 * (t0 + t1) * largura;
        } else {
            var lo_b = a0;
            var hi_b = a1;
            for (var b = 0u; b < BISSECOES; b++) {
                let m = 0.5 * (lo_b + hi_b);
                let tm = pelo_ponto_ao_disco(P, nf, h, v, d, L.raio, m, k);
                if abs(tm - t0) <= abs(tm - t1) {
                    lo_b = m;
                } else {
                    hi_b = m;
                }
            }
            let borda = 0.5 * (lo_b + hi_b);
            soma += t0 * (borda - a0) + t1 * (a1 - borda);
        }
        a0 = a1;
        t0 = t1;
    }
    return soma / (2.0 * alfa);
}

// O [`ate_o_disco`] pelo [`pelo_ponto_ate`].
fn pelo_ponto_ao_disco(P: vec2<f32>, nf: vec2<f32>, h: f32, v: vec2<f32>, d: f32, raio: f32, a: f32, k: u32) -> f32 {
    let dj = vec2<f32>(cos(a), sin(a));
    let b = dot(dj, v);
    let t = b - sqrt(max(b * b - (d * d - raio * raio), 0.0));
    return pelo_ponto_ate(P, nf, h, dj, P + dj * max(t - ANTES_DO_DISCO, 0.0), k);
}

// A visibilidade da lâmpada `k` no ponto de fronteira `P` que o texel `P0` (dentro) lê.
fn visibilidade_remapeada(P: vec2<f32>, P0: vec2<f32>, k: u32) -> f32 {
    if VIS_NA_FACE {
        var nf = normal_secante(P, P0, 1.0 / f32(fina.fina));
        if all(nf == vec2<f32>(0.0)) {
            // No canto a face não corta a abertura numa reta: a marcha do ponto rente, como antes.
            return visibilidade(P, k, 0.0);
        }
        return visibilidade_na_face(P, nf, k, meio(P0).tau > 0.0);
    }
    return visibilidade(P, k, 0.0);
}

// O [`luz_exata`] do ponto de fronteira `P` que o texel `P0` (dentro) lê.
fn luz_exata_remapeada(P: vec2<f32>, P0: vec2<f32>) -> vec3<f32> {
    var soma = c0_em(P);
    for (var k = 0u; k < lampadas.n; k++) {
        let s = sem_sombra(P, lampadas.itens[k]);
        if any(s > vec3<f32>(0.0)) {
            soma += s * visibilidade_remapeada(P, P0, k);
        }
    }
    return soma;
}

// ⭐ A normal de fora no ponto de fronteira `q` achado de `P0`, pela SECANTE: de um ponto 3 texels finos
// para dentro de q, os raios a ±δ saem a ~2 texels de q ao longo da face, e a corda entre as saídas é
// a tangente. O Sobel de um texel treme ~2° de texel para texel: o pé que ele fixa salta ~ε·R ao longo
// da face, e a tangente que corta a lâmpada lê ~30 % de visibilidade a mais ou a menos (os degraus e o
// serrilhado junto aos pontos tangentes dos discos da cena 7). Sem o ponto de dentro (corpo fino), o Sobel.
fn normal_secante(q: vec2<f32>, P0: vec2<f32>, h: f32) -> vec2<f32> {
    let ns = normal_de_fora(q, P0);
    let d = distance(q, P0);
    var u = ns;
    if d > 0.5 * h {
        u = (q - P0) / d;
    }
    let O = q - u * (3.0 * h);
    if !num_corpo(O) {
        return ns;
    }
    let a = atan2(u.y, u.x);
    let delta = 0.5880026;
    let alcance = 3.0 * h / cos(delta) + 4.0 * h;
    let c1 = saida_na_direcao(O, a + delta, alcance, h);
    let c2 = saida_na_direcao(O, a - delta, alcance, h);
    if distance(c1, q) > 4.0 * h || distance(c2, q) > 4.0 * h {
        return ns;
    }
    let t = c1 - c2;
    if dot(t, t) <= 1e-12 {
        return ns;
    }
    var n = normalize(vec2<f32>(t.y, -t.x));
    if dot(n, u) < 0.0 {
        n = -n;
    }
    // Um CANTO: q fora da corda (numa face lisa só a flecha, 2h²/R ≤ h/8 para R ≥ 16 texels) — a
    // corda liga duas faces e a normal dela é a diagonal. Zero: quem chama fica com o que tinha.
    if abs(dot(q - c1, n)) > 0.25 * h {
        return vec2<f32>(0.0);
    }
    return n;
}

// Onde o raio de `P0` (dentro) pelo ângulo `a` sai do opaco (rente, fora), a menos de `alcance`; longe
// se não sai.
fn saida_na_direcao(P0: vec2<f32>, a: f32, alcance: f32, h: f32) -> vec2<f32> {
    let u = vec2<f32>(cos(a), sin(a));
    let fim = P0 + u * alcance;
    if num_corpo(fim) {
        return P0 + u * (4.0 * alcance);
    }
    return cruzamento(P0, fim, h);
}
