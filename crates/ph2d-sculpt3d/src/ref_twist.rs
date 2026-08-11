//! **O TWIST DA REFERÊNCIA** — o único kernel que gira em vez de deslocar, e o
//! único que paga TRANSCENDENTAIS.
//!
//! Filho de [`super`] (`ref_kernels`) pela mesma razão dos irmãos
//! [`super::smooth`] e [`super::mask`]: é o mesmo porte do mesmo original, e a
//! superfície pública fica plana porque quem chama fala com **um** porte.
//!
//! # ⚠️ Aqui a paridade bit-a-bit deixa de ser uma escolha de estrutura e passa
//! a depender de QUAL biblioteca de matemática o porte usa — e isso foi MEDIDO
//!
//! Os nove kernels de deslocamento só somam, multiplicam e tiram raiz — e
//! `sqrt` é **exatamente especificada** pelo IEEE-754, então dois runtimes dão o
//! mesmo bit. Este chama `Math.sin`, `Math.cos`, `Math.atan2` e `Math.hypot`, e
//! o ECMAScript declara os quatro **implementation-approximated**: *não existe
//! resposta exata para espelhar* — é a mesma frase que o
//! [`ph2d_wet_paint::jsmath`] já carrega sobre o `Math.pow`.
//!
//! O que existe é **medir qual libm chega mais perto do V8**. 20 000 amostras,
//! comparadas bit a bit contra o Node:
//!
//! | função  | `std` (libm do SISTEMA) | `libm` (o crate, porte do MUSL) |
//! |---------|-------------------------|---------------------------------|
//! | `sin`   | 3,300 % a 1 ulp         | **1,005 % a 1 ulp**             |
//! | `cos`   | 3,280 % a 1 ulp         | **0,845 % a 1 ulp**             |
//! | `atan2` | 18,645 % a 1 ulp        | **0,000 % — EXATO**             |
//! | `hypot` | 36,935 % a 2 ulp        | 37,400 % a 2 ulp                |
//!
//! Daí as três decisões, cada uma com o número ao lado:
//!
//! - **`atan2` sai do `libm`, e é EXATO contra o V8.** Zero divergências em
//!   20 000 amostras, contra quase um quinto delas no `std`. Como o ângulo
//!   multiplica a curva de TODO vértice, ele é a única entrada cujo erro se
//!   espalharia pela pegada inteira de uma vez.
//! - **`sin`/`cos` também saem do `libm`** (3× melhor que o `std`), e o 1 ulp
//!   que resta não é observável através do `f32` — ver a tabela de baixo.
//! - **`hypot` não é melhor em lugar nenhum, e não precisa ser:** o único
//!   consumidor dele é a comparação `< 30` da zona morta. Dois ulps só trocam o
//!   ramo para um gesto que caia a dois ulps de exatamente 30 píxeis do centro —
//!   e nesse ponto o ângulo produzido é o de um vetor de comprimento 30, que a
//!   normalização apaga. Fica no `libm` pela razão inversa da dos outros: para
//!   este porte falar **uma** biblioteca de matemática, em vez de duas.
//!
//! # ⚠️ O QUE O GATE DEFENDE, E O QUE ELE NÃO PODE DEFENDER
//!
//! **Seis mutações deste arquivo sobrevivem à suíte inteira, e elas são UM
//! fato, não seis buracos:** a saída deste kernel passa por uma arredondada para
//! `f32`, que descarta 29 bits, e **toda escolha de nível `f64` aqui pousa
//! abaixo dela**. Medido sobre 3 M avaliações com gesto, eixo e pegada
//! plausíveis — a coluna que decide é a última:
//!
//! | escolha | diverge em `f64` | pior erro relativo | atravessa o `f32` |
//! |---|---|---|---|
//! | `atan2` do `std` no lugar do `libm` | 1,032 % | 1,49e-9 | **0 de 3 000 000** |
//! | `sin`/`cos` do `std` | 0,306 % | 4,67e-12 | **0** |
//! | associação `(f·a)·m` no lugar de `f·(a·m)` | 2,386 % | 3,78e-12 | **0** |
//! | dividir pelo raio em vez do recíproco | 1,078 % | 6,13e-11 | **0** |
//! | `transformQuat` do gl-matrix **2.x** | 28,930 % | 1,49e-9 | **0** |
//!
//! ⚠️ **A linha da associação precisou de fixture própria e isso é uma lição
//! sobre o oráculo:** com a máscara valendo `{0, 0,5, 1}` — que é o que a
//! fixture do gate usa — ela diverge **0,000 %**, porque multiplicar por
//! potência de dois é EXATO. A associação é inobservável ali *por construção*,
//! não por equivalência; o `2,386 %` é com máscara geral.
//!
//! **Então a paridade que o gate prova é a que o produto tem** (a saída em
//! `f32`, sobre a pegada inteira, contra o JS executando), e as cinco escolhas
//! acima são **defesa em camadas documentada em vez de gateada** — o precedente
//! do ADR-0145. Elas ficam por serem o que a referência faz; o que **não** se
//! pode escrever ao lado delas é que um teste as vigia.
//!
//! ⚠️ **A sexta sobrevivente é de outra espécie** (inalcançável, não
//! sub-ulp): ver a guarda de comprimento zero no [`normalise`].
//!
//! # ⚠️ A zona morta é em PÍXEIS, e isso é um acoplamento real
//!
//! `if (vec2.len(vecMouse) < 30) return;` (`Twist.js:96`) — trinta **píxeis de
//! tela**, não unidades de mundo nem fração de raio. Quem chamar
//! [`twist_angle`] tem de alimentar a mesma régua, senão a zona morta passa a
//! significar outra coisa em silêncio: num gizmo cujo raio de arrasto é medido
//! em unidades de mundo, `30` seria o mundo inteiro.
//!
//! Fonte: `src/editing/tools/Twist.js`, `src/math3d/Geometry.js` e o
//! `gl-matrix` **3.3.0** que o `package-lock.json` da referência trava.

/// **O ÂNGULO** — a metade de TELA do `Twist.twist`, separada do laço.
///
/// ⚠️ **Na referência os dois vivem no mesmo método**, e a separação aqui é
/// deliberada: o cabeçalho de [`super`] promete portar *o laço por-vértice, e só
/// ele*, e esta metade não é um laço — é a lei de AUTORIA (como um arrasto vira
/// um ângulo). Mantê-la junta obrigaria o kernel a receber quatro coordenadas de
/// tela para computar um escalar que ele multiplica uma vez, e faria a promessa
/// do módulo deixar de ser verdadeira.
///
/// `None` é a **zona morta**: perto do centro de rotação um pixel de tremor vira
/// um ângulo enorme, e o original simplesmente não gira (`Twist.js:96`).
///
/// ⚠️ **Um vetor de comprimento zero normaliza para ZERO, não para `NaN`** — o
/// `vec2.normalize` do gl-matrix guarda a divisão atrás de `if (len > 0)` e
/// deixa o `len = 0` passar como fator. É alcançável pelo `last`: o cursor pode
/// estar exatamente sobre o centro no quadro anterior, e sem a guarda a pegada
/// inteira sairia `NaN` — que o `f32` guardaria.
///
/// ⚠️ **Mas a mutação que apaga a guarda SOBREVIVE ao gate, e a razão é a
/// fixture:** o caso do oráculo põe o cursor anterior a 100 px do centro, então
/// o ramo `len == 0` nunca roda. Um caso que o alcançasse teria de pôr `last`
/// exatamente sobre `centre` — construível, e **não construído**: ele provaria
/// uma linha do gl-matrix, não uma do porte. Fica documentada, como as cinco
/// sub-ulp do cabeçalho.
#[must_use]
pub fn twist_angle(mouse: [f64; 2], last: [f64; 2], centre: [f64; 2]) -> Option<f64> {
    let cur = [mouse[0] - centre[0], mouse[1] - centre[1]];
    if libm::hypot(cur[0], cur[1]) < 30.0 {
        return None;
    }
    let old = [last[0] - centre[0], last[1] - centre[1]];
    let cur = normalise(cur);
    let old = normalise(old);
    // `Geometry.signedAngle2d(v1, v2)` — e a ORDEM importa: o atual é `v1`, o
    // anterior é `v2`, então o sinal é o do giro que ACABOU de acontecer.
    Some(libm::atan2(
        cur[0] * old[1] - cur[1] * old[0],
        cur[0] * old[0] + cur[1] * old[1],
    ))
}

/// `vec2.normalize` do gl-matrix, com a guarda de comprimento zero dele.
#[inline]
fn normalise(a: [f64; 2]) -> [f64; 2] {
    let mut len = a[0] * a[0] + a[1] * a[1];
    if len > 0.0 {
        len = 1.0 / len.sqrt();
    }
    [a[0] * len, a[1] * len]
}

/// `vec3.transformQuat` do **gl-matrix 3.3.0**, na formulação `uv`/`uuv`.
///
/// O gl-matrix 2.x fazia `q · v · q⁻¹` expandido em duas multiplicações de
/// quaternion; o 3.x faz `v + 2w(q⃗ × v) + 2(q⃗ × (q⃗ × v))`. A referência declara
/// `"gl-matrix": "^3.1.0"` e o `package-lock.json` resolve **3.3.0** — é essa a
/// versão que o oráculo carrega, então é essa que este `fn` reproduz.
///
/// ⚠️ **E a minha primeira versão desta nota afirmava que escolher a errada
/// *"falha no gate"* — MEDIDO, é FALSO.** As duas formas são a mesma rotação em
/// álgebra e divergem em **28,930 %** das avaliações em `f64`, por até
/// **1,49e-9** relativo — e **zero de 3 000 000** dessas divergências sobrevive
/// à arredondada para `f32`. Escrever a de 2.x aqui passaria em tudo. A versão
/// certa fica por ser a que a referência resolve, **não** porque um teste a
/// vigia; a tabela inteira está no cabeçalho de [`super::twist_mod`].
#[inline]
fn transform_quat(a: [f64; 3], q: [f64; 4]) -> [f64; 3] {
    let (qx, qy, qz, qw) = (q[0], q[1], q[2], q[3]);
    let (x, y, z) = (a[0], a[1], a[2]);
    let mut uvx = qy * z - qz * y;
    let mut uvy = qz * x - qx * z;
    let mut uvz = qx * y - qy * x;
    let mut uuvx = qy * uvz - qz * uvy;
    let mut uuvy = qz * uvx - qx * uvz;
    let mut uuvz = qx * uvy - qy * uvx;
    let w2 = qw * 2.0;
    uvx *= w2;
    uvy *= w2;
    uvz *= w2;
    uuvx *= 2.0;
    uuvy *= 2.0;
    uuvz *= 2.0;
    [x + uvx + uuvx, y + uvy + uuvy, z + uvz + uuvz]
}

/// **TWIST** (`Twist.js:88-129`) — cada vértice **GIRA** em torno do eixo do
/// olhar, por um ângulo que a curva atenua.
///
/// O vértice do centro roda o `angle` inteiro e o da borda roda **zero**: é a
/// [`super::falloff`] aplicada ao ÂNGULO em vez de a um deslocamento, e é isso
/// que produz o redemoinho em vez de um giro rígido da pegada.
///
/// `axis` é o `twistData.normal` do original — a direção do olhar **NEGADA**
/// (`Twist.js:41`), capturada no início do traço e fixa até o fim; girar a
/// câmera no meio do gesto não vira o eixo. Ele é unitário porque a direção do
/// olhar é, e o `quat.setAxisAngle` do gl-matrix **não normaliza**: um eixo de
/// comprimento diferente de um produz um quaternion não-unitário, e a
/// transformação deixa de ser uma rotação (ela escala junto). Quem monta o eixo
/// é quem manda.
///
/// ⚠️ **SEM `if dist >= 1.0 continue`**, como o [`super::pinch`] — e aqui a
/// consequência é diferente e pior. Fora do raio a curva **CRESCE** (`17` em
/// `d = 2`, ver o doc da [`super::falloff`]), então um vértice fora da pegada
/// giraria dezessete vezes o ângulo do centro: não um exagero, um nó. Quem o
/// contém é a PEGADA, e este kernel não a verifica porque o original não a
/// verifica.
///
/// ⚠️ **A distância é `sqrt(d²) * (1/raio)`, e NÃO `sqrt(d²) / raio`** —
/// `Twist.js:106` pré-computa o recíproco fora do laço, e em ponto flutuante
/// `x · (1/r)` não é `x / r` (o recíproco arredonda uma vez a mais). É por isso
/// que este kernel **não é** o `normalised_dist` que os nove irmãos usam —
/// ⚠️ *não é*, e não *"não pode ser"*: medido, as duas rotas divergem em
/// **1,078 %** das avaliações por até **6,13e-11** relativo, e **nenhuma** delas
/// atravessa o `f32`. A distinção é sobre reproduzir a referência, não sobre um
/// gate. A curva, essa sim, é a mesma expressão na mesma associação, e vem de
/// [`super::falloff`].
#[allow(clippy::too_many_arguments)]
pub fn twist(
    pos: &mut [f32],
    free: &[f32],
    verts: &[u32],
    center: [f64; 3],
    radius_squared: f64,
    angle: f64,
    axis: [f64; 3],
) {
    let inv_radius = 1.0 / radius_squared.sqrt();
    for &v in verts {
        let ind = v as usize * 3;
        let vx = f64::from(pos[ind]);
        let vy = f64::from(pos[ind + 1]);
        let vz = f64::from(pos[ind + 2]);
        let dx = vx - center[0];
        let dy = vy - center[1];
        let dz = vz - center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt() * inv_radius;
        // O ângulo DESTE vértice.
        //
        // ⚠️ **O AGRUPAMENTO é o da referência:** `fallOff *= angle *
        // mAr[ind+2] * alpha` computa o produto do gesto PRIMEIRO e só então
        // multiplica a curva, ou seja `f · (a · m)`. Escrever `f * a * m` dá
        // `(f · a) · m`, que é outro número — **2,386 % das avaliações, por até
        // 3,78e-12, e ZERO delas atravessa o `f32`** (a tabela do cabeçalho de
        // [`super::twist_mod`]); e sobre a fixture do gate, cuja máscara vale
        // `{0, 0,5, 1}`, a divergência é **0,000 %** — multiplicar por potência
        // de dois é exato. Fica pela mesma razão que a ordem da
        // [`super::falloff`] fica: é a da referência.
        // O `alpha` sai por ser `1.0` (ver o cabeçalho de [`super`]), e
        // multiplicar por um é exato.
        let fall = super::falloff(dist) * (angle * f64::from(free[v as usize]));
        // `quat.setAxisAngle` — meia-volta no seno, o cosseno no escalar, e o
        // eixo entra sem normalizar (ver o doc acima). O `0.5` é exato: dois é
        // potência de dois.
        let rad = fall * 0.5;
        let s = libm::sin(rad);
        let rot = [s * axis[0], s * axis[1], s * axis[2], libm::cos(rad)];
        // `vec3.set` + `vec3.sub` — a MESMA expressão que já deu `dx/dy/dz`
        // (`vx - center[0]`), logo os mesmos bits; a referência a escreve duas
        // vezes porque lá são duas chamadas de biblioteca.
        let c = transform_quat([dx, dy, dz], rot);
        pos[ind] = (c[0] + center[0]) as f32;
        pos[ind + 1] = (c[1] + center[1]) as f32;
        pos[ind + 2] = (c[2] + center[2]) as f32;
    }
}
