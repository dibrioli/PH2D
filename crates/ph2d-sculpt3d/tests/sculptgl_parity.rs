//! **A PARIDADE BIT A BIT COM O SCULPTGL** — os kernels de
//! [`ph2d_sculpt3d::ref_kernels`] contra os bits que o **JS da referência
//! produziu**.
//!
//! # Por que este gate é diferente de todos os outros desta crate
//!
//! Os outros afirmam propriedades que eu escolhi (*"o Grab devolve o barro"*,
//! *"o suporte da bola é o conjunto tocado"*). Este afirma um número que **eu
//! não escolhi e não posso escolher**: ele vem de
//! `docs/3D/ferramentas/sculptgl_oracle.txt`, gerado por
//! `sculptgl_oracle.mjs`, que **extrai o corpo dos métodos dos arquivos do
//! SculptGL e os executa**. Nenhuma linha daquele oráculo é uma transcrição
//! minha dos kernels — se fosse, ele só poderia confirmar a minha leitura, que é
//! exatamente o modo de falha do gate que espelha o produto em vez de o
//! interrogar.
//!
//! # ⚠️ A fixture é COMPACTA e isso é exato, não uma amostragem
//!
//! Todo kernel indexa a malha **através** de `iVerts` e nenhum deles lê um
//! vizinho fora da lista, então a pegada re-indexada de 0 a n produz os mesmos
//! bits que a esfera de 8256 vértices de onde ela saiu. O que está no arquivo é
//! a pegada REAL de um pincel de raio 0,45 sobre uma esfera unitária: **272
//! vértices**, com máscara em três regimes (livre, meio e travado).
//!
//! # ⚠️ O que este gate NÃO cobre, dito para ninguém o ler como mais do que é
//!
//! Ele cobra o **laço por-vértice**. A pegada (o nosso octree), a simetria, o
//! `pre` congelado, a janela de undo e o espaçamento são nossos e têm gates
//! próprios — o original não tem equivalente da metade deles.
//!
//! # Regenerar
//!
//! ```text
//! node docs/3D/ferramentas/sculptgl_oracle.mjs <dir-do-SculptGL> \
//!      docs/3D/ferramentas/sculptgl_oracle.txt
//! ```
//!
//! O arquivo é COMMITADO, então o gate roda sem node e sem o clone da
//! referência — é ele que torna a paridade uma propriedade do repositório em
//! vez de uma da máquina de quem a mediu.

use ph2d_sculpt3d::ref_kernels as rk;

// ---------------------------------------------------------------------------
// O PARSER — bits, nunca decimais.
// ---------------------------------------------------------------------------

/// Um caso do oráculo: as entradas e a saída que o JS produziu.
struct Case {
    name: String,
    params: std::collections::BTreeMap<String, Vec<f64>>,
    verts: usize,
    in_pos: Vec<f32>,
    in_nrm: Vec<f32>,
    /// A máscara **na polaridade da REFERÊNCIA**: `1` é livre, `0` é travado.
    free: Vec<f32>,
    sel: Vec<u32>,
    out_pos: Vec<f32>,
}

struct Oracle {
    center: [f64; 3],
    radius2: f64,
    eye: [f64; 3],
    cases: Vec<Case>,
}

fn f32s(rest: &str) -> Vec<f32> {
    rest.split_whitespace()
        .map(|t| f32::from_bits(u32::from_str_radix(t, 16).expect("hex f32")))
        .collect()
}

fn f64s(rest: &str) -> Vec<f64> {
    rest.split_whitespace()
        .map(|t| f64::from_bits(u64::from_str_radix(t, 16).expect("hex f64")))
        .collect()
}

fn load() -> Oracle {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/3D/ferramentas/sculptgl_oracle.txt"
    );
    let text = std::fs::read_to_string(path).expect("o oráculo do SculptGL tem de estar commitado");
    let mut o = Oracle {
        center: [0.0; 3],
        radius2: 0.0,
        eye: [0.0; 3],
        cases: Vec::new(),
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "sphere" => {}
            "center" => o.center = f64s(rest).try_into().expect("center 3"),
            "radius2" => o.radius2 = f64s(rest)[0],
            "eye" => o.eye = f64s(rest).try_into().expect("eye 3"),
            "case" => o.cases.push(Case {
                name: rest.to_string(),
                params: std::collections::BTreeMap::new(),
                verts: 0,
                in_pos: Vec::new(),
                in_nrm: Vec::new(),
                free: Vec::new(),
                sel: Vec::new(),
                out_pos: Vec::new(),
            }),
            _ => {
                let c = o.cases.last_mut().expect("um campo antes de `case`");
                match key {
                    "param" => {
                        let (k, v) = rest.split_once(' ').expect("param <k> <v>");
                        c.params.insert(k.to_string(), f64s(v));
                    }
                    "verts" => c.verts = rest.parse().expect("verts"),
                    "in.pos" => c.in_pos = f32s(rest),
                    "in.nrm" => c.in_nrm = f32s(rest),
                    "in.mask" => c.free = f32s(rest),
                    "sel" => {
                        c.sel = rest
                            .split_whitespace()
                            .map(|t| t.parse().expect("índice"))
                            .collect();
                    }
                    "out.pos" => c.out_pos = f32s(rest),
                    other => panic!("campo desconhecido no oráculo: {other}"),
                }
            }
        }
    }
    assert!(!o.cases.is_empty(), "o oráculo veio vazio");
    o
}

// ---------------------------------------------------------------------------
// A COMPARAÇÃO
// ---------------------------------------------------------------------------

/// Compara bit a bit e devolve uma frase útil quando diverge.
///
/// ⚠️ **Ele conta os DIVERGENTES e mede o PIOR, e as duas perguntas são
/// diferentes** — a lição que o gate de paridade da luz do impasto pagou: um
/// limite só de magnitude deixou passar 2375 bytes errados por um nível.
fn assert_bit_identical(name: &str, got: &[f32], want: &[f32]) {
    assert_eq!(got.len(), want.len(), "[{name}] comprimentos diferentes");
    let mut diff = 0usize;
    let mut worst = 0.0f64;
    let mut first = None;
    for (i, (&g, &w)) in got.iter().zip(want).enumerate() {
        if g.to_bits() != w.to_bits() {
            diff += 1;
            let d = (f64::from(g) - f64::from(w)).abs();
            if d > worst {
                worst = d;
            }
            if first.is_none() {
                first = Some((i, g, w));
            }
        }
    }
    assert!(
        diff == 0,
        "[{name}] {diff} de {} componentes divergem dos bits do SculptGL \
         (pior delta absoluto {worst:.3e}); o primeiro é o índice {:?}",
        got.len(),
        first
    );
}

/// A fixture do caso, pronta para ser mexida.
fn scratch(c: &Case) -> Vec<f32> {
    c.in_pos.clone()
}

fn front_with(c: &Case, eye: [f64; 3]) -> Vec<u32> {
    let mut out = Vec::new();
    rk::front_vertices(&c.in_nrm, &c.sel, eye, &mut out);
    out
}

fn case<'a>(o: &'a Oracle, name: &str) -> &'a Case {
    o.cases
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("o oráculo não tem o caso `{name}`"))
}

/// O centro deste caso — o do arquivo, ou o que o caso sobrescreveu.
fn center_of(o: &Oracle, c: &Case) -> [f64; 3] {
    c.params
        .get("center")
        .map_or(o.center, |v| [v[0], v[1], v[2]])
}

/// O olho deste caso — idem. Só o caso da terminadora o sobrescreve.
fn eye_of(o: &Oracle, c: &Case) -> [f64; 3] {
    c.params.get("eye").map_or(o.eye, |v| [v[0], v[1], v[2]])
}

// ---------------------------------------------------------------------------
// OS GATES
// ---------------------------------------------------------------------------

/// ⚠️ **O CONTROLE, e ele vem primeiro.** Um oráculo cujos casos não movem nada
/// aprovaria um kernel deletado — e um cuja pegada é vazia aprovaria qualquer
/// coisa. Antes de comparar um bit, este gate exige que o arquivo descreva um
/// fenômeno: pegada não-trivial, os três regimes de máscara, e deslocamento
/// real em todos os casos.
#[test]
fn the_oracle_describes_a_phenomenon_before_it_judges_anything() {
    let o = load();
    assert_eq!(o.cases.len(), 10, "nove kernels portados, e a terminadora");
    for c in &o.cases {
        let n = c.sel.len();
        assert!(n >= 200, "[{}] pegada rala demais: {n}", c.name);
        assert_eq!(c.in_pos.len(), c.verts * 3, "[{}] posições", c.name);
        assert_eq!(c.in_nrm.len(), c.verts * 3, "[{}] normais", c.name);
        assert_eq!(c.free.len(), c.verts, "[{}] máscaras", c.name);
        let moved = c
            .in_pos
            .iter()
            .zip(&c.out_pos)
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
        assert!(
            moved > n,
            "[{}] só {moved} componentes mexeram — a fixture não contém o fenômeno",
            c.name
        );
        // Os três regimes de máscara, na polaridade da referência.
        assert!(c.free.contains(&1.0), "[{}] sem livre", c.name);
        assert!(c.free.contains(&0.5), "[{}] sem meio", c.name);
        assert!(c.free.contains(&0.0), "[{}] sem travado", c.name);

        // ⚠️ **A seleção NÃO pode ser a identidade, e a asserção nasceu de uma
        // mutação SOBREVIVENTE.** O proxy do `move` é EMPACOTADO (indexado pela
        // posição na lista, `j = i * 3`) e não pelo id do vértice — com
        // `sel = [0..n)` os dois índices coincidem, e trocar um pelo outro
        // passava verde. Embaralhada, a distinção existe.
        assert!(
            c.sel.iter().enumerate().any(|(i, &v)| i as u32 != v),
            "[{}] a seleção é a identidade — o proxy empacotado fica indistinguível \
             do indexado por vértice",
            c.name
        );
        // ⚠️ E ela é um subconjunto PRÓPRIO: os de fora têm de sair
        // byte-idênticos, então um kernel que escrevesse além da lista é pego
        // pela mesma comparação que julga os de dentro.
        assert!(
            n < c.verts,
            "[{}] a seleção ({n}) cobre a malha inteira ({}) — ninguém testa o \
             que fica de FORA do pincel",
            c.name,
            c.verts
        );

        // ⚠️ **As normais NÃO são unitárias, e a fixture tem de garantir isso.**
        // O `Mesh.updateVerticesNormal` da referência guarda a MÉDIA das normais
        // de face, sem normalizar — é por isso que o `Inflate` divide pelo
        // comprimento na hora de usar. Com normais unitárias essa divisão é um
        // no-op, e a mutação que a apaga passa verde (foi o que aconteceu).
        let mut any_short = false;
        for i in 0..c.verts {
            let (x, y, z) = (c.in_nrm[i * 3], c.in_nrm[i * 3 + 1], c.in_nrm[i * 3 + 2]);
            let len = f64::from(x).hypot(f64::from(y)).hypot(f64::from(z));
            if len < 0.99 {
                any_short = true;
            }
        }
        assert!(
            any_short,
            "[{}] toda normal é unitária — a divisão pelo comprimento do Inflate \
             vira um no-op e ninguém a testa",
            c.name
        );
    }
    // ⚠️ **O `eye` só é observável onde a pegada atravessa a TERMINADORA**, e
    // esta asserção nasceu VERMELHA sobre um oráculo correto: com o olho
    // apontando para o centro da pegada, TODO vértice é frontal e o
    // `front_vertices` devolve a lista inteira — um gate que só olhasse para os
    // nove casos normais aprovaria um filtro que não filtra nada.
    //
    // A cura não foi afrouxar a asserção; foi a fixture passar a **conter o
    // fenômeno** (o caso `brush_terminator`). As duas metades ficam:
    let c = case(&o, "brush");
    assert_eq!(
        front_with(c, eye_of(&o, c)).len(),
        c.sel.len(),
        "com o olho sobre a pegada, a frente É a pegada — se isto mudar, \
         os oito casos normais deixaram de medir o que dizem medir"
    );
    let t = case(&o, "brush_terminator");
    let f = front_with(t, eye_of(&o, t));
    assert!(
        f.len() < t.sel.len() && f.len() > t.sel.len() / 4,
        "na terminadora a frente ({}) tem de ser um subconjunto PRÓPRIO e \
         não-trivial da pegada ({}) — é o único caso que exercita o olho",
        f.len(),
        t.sel.len()
    );
}

/// O proxy **EMPACOTADO** do [`rk::r#move`] — indexado pela POSIÇÃO NA LISTA.
///
/// ⚠️ **Ele existe porque o gate me pegou.** A primeira versão passava
/// `c.in_pos.clone()` (indexado pelo id do vértice), que é o que os proxies do
/// Inflate e do Crease são — e ficava verde enquanto a seleção era a identidade.
/// Com ela embaralhada, **699 de 1509 componentes divergiram**: os dois proxies
/// deste módulo não são a mesma coisa, e nada no tipo `&[f32]` os distingue.
fn packed_proxy(c: &Case) -> Vec<f32> {
    let mut out = Vec::with_capacity(c.sel.len() * 3);
    for &v in &c.sel {
        let i = v as usize * 3;
        out.extend_from_slice(&c.in_pos[i..i + 3]);
    }
    out
}

#[test]
fn the_brush_kernel_is_bit_identical() {
    let o = load();
    let c = case(&o, "brush");
    let mut pos = scratch(c);
    let n = rk::area_normal(&c.in_nrm, &c.free, &front_with(c, eye_of(&o, c))).expect("normal de área");
    rk::brush(
        &mut pos,
        &c.free,
        &c.sel,
        None,
        n,
        center_of(&o, c),
        o.radius2,
        c.params["intensity"][0],
        c.params["negative"][0] != 0.0,
    );
    assert_bit_identical("brush", &pos, &c.out_pos);
}

/// O MESMO kernel com a pegada **atravessando a terminadora** — o único caso em
/// que `front_vertices` filtra alguma coisa, e por isso o único que prova que a
/// normal de área é ajustada sobre o conjunto FRONTAL e não sobre a pegada.
///
/// ⚠️ **Sem ele, um `front_vertices` que devolvesse a lista inteira ficaria
/// verde nos nove irmãos** — era o estado desta suíte até o controle acusar.
#[test]
fn the_brush_kernel_is_bit_identical_across_the_terminator() {
    let o = load();
    let c = case(&o, "brush_terminator");
    let mut pos = scratch(c);
    let n = rk::area_normal(&c.in_nrm, &c.free, &front_with(c, eye_of(&o, c)))
        .expect("normal de área");
    rk::brush(
        &mut pos,
        &c.free,
        &c.sel,
        None,
        n,
        center_of(&o, c),
        o.radius2,
        c.params["intensity"][0],
        c.params["negative"][0] != 0.0,
    );
    assert_bit_identical("brush_terminator", &pos, &c.out_pos);
}

/// ⚠️ **O CLAY é o default de fábrica do Brush** (`Brush.js:12`, `_clay =
/// true`) e ele **não chama o `brush()`**: ele achata contra um plano deslocado
/// por `raio · 0,1`, pulando quem já passou do plano. É uma operação diferente
/// da nossa, não uma afinação dela.
#[test]
fn the_clay_kernel_is_bit_identical() {
    let o = load();
    let c = case(&o, "clay");
    let mut pos = scratch(c);
    let f = front_with(c, eye_of(&o, c));
    let n = rk::area_normal(&c.in_nrm, &c.free, &f).expect("normal de área");
    let mut ctr = rk::area_center(&c.in_pos, &c.free, &f).expect("centro de área");
    let off = rk::clay_plane_offset(o.radius2.sqrt());
    for k in 0..3 {
        ctr[k] += n[k] * off;
    }
    rk::flatten(
        &mut pos,
        &c.free,
        &c.sel,
        None,
        n,
        ctr,
        center_of(&o, c),
        o.radius2,
        c.params["intensity"][0],
        c.params["negative"][0] != 0.0,
    );
    assert_bit_identical("clay", &pos, &c.out_pos);
}

#[test]
fn the_flatten_kernel_is_bit_identical() {
    let o = load();
    let c = case(&o, "flatten");
    let mut pos = scratch(c);
    let f = front_with(c, eye_of(&o, c));
    let n = rk::area_normal(&c.in_nrm, &c.free, &f).expect("normal de área");
    let ctr = rk::area_center(&c.in_pos, &c.free, &f).expect("centro de área");
    rk::flatten(
        &mut pos,
        &c.free,
        &c.sel,
        None,
        n,
        ctr,
        center_of(&o, c),
        o.radius2,
        c.params["intensity"][0],
        c.params["negative"][0] != 0.0,
    );
    assert_bit_identical("flatten", &pos, &c.out_pos);
}

#[test]
fn the_inflate_kernel_is_bit_identical() {
    let o = load();
    let c = case(&o, "inflate");
    let mut pos = scratch(c);
    // ⚠️ O Inflate do original chama `updateProxy` INCONDICIONALMENTE
    // (`Inflate.js:23`) e lê `getVerticesProxy()` — no primeiro dab de um traço
    // o proxy É a posição de entrada.
    let proxy = c.in_pos.clone();
    rk::inflate(
        &mut pos,
        &c.in_nrm,
        &c.free,
        &c.sel,
        Some(&proxy),
        center_of(&o, c),
        o.radius2,
        c.params["intensity"][0],
        c.params["negative"][0] != 0.0,
    );
    assert_bit_identical("inflate", &pos, &c.out_pos);
}

#[test]
fn the_crease_kernel_is_bit_identical() {
    let o = load();
    let c = case(&o, "crease");
    let mut pos = scratch(c);
    let n = rk::area_normal(&c.in_nrm, &c.free, &front_with(c, eye_of(&o, c))).expect("normal de área");
    let proxy = c.in_pos.clone();
    rk::crease(
        &mut pos,
        &c.free,
        &c.sel,
        Some(&proxy),
        n,
        center_of(&o, c),
        o.radius2,
        c.params["intensity"][0],
        c.params["negative"][0] != 0.0,
    );
    assert_bit_identical("crease", &pos, &c.out_pos);
}

#[test]
fn the_pinch_kernel_is_bit_identical() {
    let o = load();
    let c = case(&o, "pinch");
    let mut pos = scratch(c);
    rk::pinch(
        &mut pos,
        &c.free,
        &c.sel,
        center_of(&o, c),
        o.radius2,
        c.params["intensity"][0],
        c.params["negative"][0] != 0.0,
    );
    assert_bit_identical("pinch", &pos, &c.out_pos);
}

#[test]
fn the_drag_kernel_is_bit_identical() {
    let o = load();
    let c = case(&o, "drag");
    let mut pos = scratch(c);
    let d = &c.params["dir"];
    rk::drag(
        &mut pos,
        &c.free,
        &c.sel,
        [d[0], d[1], d[2]],
        center_of(&o, c),
        o.radius2,
    );
    assert_bit_identical("drag", &pos, &c.out_pos);
}

#[test]
fn the_move_kernel_is_bit_identical() {
    let o = load();
    let c = case(&o, "move");
    let mut pos = scratch(c);
    let d = &c.params["dir"];
    // ⚠️ O proxy do Move é EMPACOTADO — ver [`packed_proxy`], que nasceu de o
    // gate ter pego a minha própria versão indexada pelo id do vértice.
    let proxy = packed_proxy(c);
    rk::r#move(
        &mut pos,
        &c.free,
        &c.sel,
        &proxy,
        [d[0], d[1], d[2]],
        center_of(&o, c),
        o.radius2,
    );
    assert_bit_identical("move", &pos, &c.out_pos);
}

#[test]
fn the_local_scale_kernel_is_bit_identical() {
    let o = load();
    let c = case(&o, "local_scale");
    let mut pos = scratch(c);
    rk::scale(
        &mut pos,
        &c.free,
        &c.sel,
        center_of(&o, c),
        o.radius2,
        c.params["delta"][0],
    );
    assert_bit_identical("local_scale", &pos, &c.out_pos);
}

/// ⚠️ **A CURVA, isolada** — o falloff único da referência, contra os números
/// que a álgebra dele dá em pontos escolhidos à mão.
///
/// Ele não substitui os oito de cima (que rodam o JS); ele existe porque a
/// curva é o que **toda** ferramenta lê, e um gate que a nomeia sozinha diz
/// *onde* olhar quando os oito ficarem vermelhos juntos.
///
/// ⚠️ **E ele é o número que separa a nossa curva da dela**: a meio raio a
/// referência entrega **0,6875** e o nosso `Falloff::Smooth` — `(1 − t²)²` —
/// entrega **0,5625**. Não é afinação: é 22% mais pincel em toda a coroa.
#[test]
fn the_reference_falloff_is_one_curve_and_these_are_its_numbers() {
    assert_eq!(rk::falloff(0.0), 1.0, "cheio no centro");
    assert_eq!(rk::falloff(1.0), 0.0, "zero na borda");
    assert_eq!(rk::falloff(0.5), 0.6875, "3/16 − 1/2 + 1");
    // Derivada `12d²(d − 1)`: zero nas DUAS pontas, e é isso que faz o traço
    // não deixar degrau na fronteira nem bico no meio.
    let e = 1e-6;
    assert!(rk::falloff(e) > 1.0 - 1e-11, "C¹ no centro");
    assert!(rk::falloff(1.0 - e) < 1e-11, "C¹ na borda");
    // ⚠️ **Fora do raio ela CRESCE, e eu tinha escrito o contrário.** A minha
    // primeira versão deste gate afirmava *"negativa fora do raio, −0,71 em
    // d = 1,2"* — falso: fatorando, `3d⁴ − 4d³ + 1 = (d−1)²(3d² + 2d + 1)`, e o
    // quadrático tem discriminante **−8**, logo a curva nunca troca de sinal.
    // A raiz em `d = 1` é DUPLA (é daí que vem a derivada zero na borda) e
    // depois ela sobe.
    //
    // O fato importa porque o [`rk::pinch`] é o único kernel **sem** a guarda
    // `dist >= 1`: entregar a ele uma lista mais larga que a esfera não o
    // deixa um pouco mais forte, deixa-o divergente — e sem sintoma nenhum
    // perto do centro, que é onde se olha.
    assert!(rk::falloff(1.2) > 0.30 && rk::falloff(1.2) < 0.31, "0,3088");
    assert!(rk::falloff(2.0) > 16.9, "17,0 — quem contém isto é a PEGADA");
}
