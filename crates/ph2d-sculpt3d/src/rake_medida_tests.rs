//! **A MEDIÇÃO QUE A ESPEC §14.5 EXIGE** — o `G-2` sobre a NOSSA saída.
//!
//! A pergunta que a espec deixa **em aberto** é se o pente é *uma* lei no sítio
//! certo ou *duas*. Ela não é decidível de fora do alvo: o instrumento dele
//! nunca produz uma célula em que o operador de topologia tenha trabalhado **e**
//! as arestas sejam comparáveis. ⇒ a espec manda fazer a hipótese **(b)** — a
//! relaxação dentro do laço por-carimbo — e **medir contra o `G-2`**. Se a nossa
//! malha ficar abaixo da barra com a lei já no sítio certo, a causa é a **(a)** e
//! há uma segunda lei a construir.
//!
//! ⚠️ **A régua é a do corpus e é PLANAR**: `Q = média(cos 4α)` sobre as arestas
//! cujo ponto médio cai na pegada, com a aresta e a direcção do traço projectadas
//! no plano do ecrã. As fixturas do oráculo são planas vistas de topo, logo a
//! projecção XY **é** o ecrã — e a dívida de uma régua para malha curva está
//! nomeada no `README` delas.
//!
//! ⚠️ **A região é `raio/2`**, não o raio inteiro: medido no corpus, ali a
//! separação é `2`–`3×` maior, porque o efeito concentra-se na faixa central do
//! traço e a orla dilui-o.

use ph2d_mesh::Mesh;

use crate::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};

/// Uma chapa plana triangulada, `n × n` vértices no plano `z = 0`, com o
/// interior **sacudido**.
///
/// ⚠️ **Plana de propósito:** é o que torna a projecção XY igual ao ecrã, que é
/// a premissa da régua do corpus.
///
/// ⛔⛔ **E SACUDIDA de propósito, porque a 1.ª redacção media a FIXTURA.** Uma
/// grade regular triangulada pelos eixos já É uma grade alinhada com um traço ao
/// longo de `+x`: com o pente desligado ela lia `Q = +0,2885`, seis vezes acima
/// da barra, e o gate `G-3` reprovava sobre uma lei correcta. *O lado desligado
/// tem de conter o fenómeno tanto quanto o ligado* — o corpus do oráculo parte de
/// uma malha de topologia dinâmica, que é irregular por construção.
///
/// O sacudir é **determinístico** (um hash dos índices): ele não é ruído de
/// corrida, é a fixtura.
fn chapa(n: usize, lado: f32) -> Mesh {
    let passo = lado / (n - 1) as f32;
    let meio = lado * 0.5;
    let mut pos = Vec::with_capacity(n * n);
    let sacode = |i: usize, j: usize, sal: u32| -> f32 {
        let mut h = (i as u32).wrapping_mul(0x9E37_79B9)
            ^ (j as u32).wrapping_mul(0x85EB_CA6B)
            ^ sal.wrapping_mul(0xC2B2_AE35);
        h ^= h >> 15;
        h = h.wrapping_mul(0x2545_F491);
        h ^= h >> 13;
        (h & 0xFFFF) as f32 / 65535.0 - 0.5
    };
    for j in 0..n {
        for i in 0..n {
            // ⚠️ A borda fica QUIETA: sacudi-la deformaria o contorno da chapa,
            // e o que se quer irregular é o INTERIOR, que é onde a régua mede.
            let dentro = i > 0 && j > 0 && i < n - 1 && j < n - 1;
            let (dx, dy) = if dentro {
                (sacode(i, j, 1) * passo * 0.7, sacode(i, j, 2) * passo * 0.7)
            } else {
                (0.0, 0.0)
            };
            pos.push([
                i as f32 * passo - meio + dx,
                j as f32 * passo - meio + dy,
                0.0,
            ]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let (a, b, c, d) = (
                (j * n + i) as u32,
                (j * n + i + 1) as u32,
                ((j + 1) * n + i + 1) as u32,
                ((j + 1) * n + i) as u32,
            );
            faces.push(ph2d_mesh::Face::tri(a, b, c));
            faces.push(ph2d_mesh::Face::tri(a, c, d));
        }
    }
    Mesh::from_parts(pos, faces).expect("a chapa e' uma malha valida")
}

/// `Q = média(cos 4α)` sobre as arestas da pegada — a régua do corpus.
fn q_da_faixa(malha: &Mesh, centros: &[[f32; 3]], raio: f32) -> (f64, usize) {
    let pos = malha.positions();
    let mut vistas = std::collections::BTreeSet::new();
    let mut soma = 0.0f64;
    let mut n = 0usize;
    for f in malha.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (vs[k], vs[(k + 1) % vs.len()]);
            if !vistas.insert((a.min(b), a.max(b))) {
                continue;
            }
            let (pa, pb) = (pos[a as usize], pos[b as usize]);
            let meio = [(pa[0] + pb[0]) * 0.5, (pa[1] + pb[1]) * 0.5];
            // O troço de percurso mais perto, e a direcção LOCAL dele.
            let mut melhor = f32::INFINITY;
            let mut direccao = [1.0f32, 0.0];
            for par in centros.windows(2) {
                let d = (meio[0] - par[1][0]).hypot(meio[1] - par[1][1]);
                if d < melhor {
                    melhor = d;
                    direccao = [par[1][0] - par[0][0], par[1][1] - par[0][1]];
                }
            }
            if melhor > raio * 0.5 {
                continue;
            }
            let aresta = [pb[0] - pa[0], pb[1] - pa[1]];
            let c = f64::from(aresta[0] * direccao[0] + aresta[1] * direccao[1]);
            let s = f64::from(aresta[0] * direccao[1] - aresta[1] * direccao[0]);
            if c == 0.0 && s == 0.0 {
                continue;
            }
            soma += (4.0 * s.atan2(c)).cos();
            n += 1;
        }
    }
    (soma / n.max(1) as f64, n)
}

fn pincel(pente: f32) -> Brush {
    Brush {
        verb: Verb::Draw,
        radius: 0.30,
        strength: 0.25,
        falloff: Falloff::Smooth,
        pente,
        ..Brush::default()
    }
}

/// Um traço recto ao longo de `+x`, com o refino a correr antes de cada carimbo
/// quando `refina`.
fn traco(pente: f32, refina: bool) -> (Mesh, Vec<[f32; 3]>) {
    let mut malha = chapa(61, 3.0);
    let brush = pincel(pente);
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut region = ph2d_mesh::RegionScratch::default();
    let mut centros = Vec::new();
    for k in 0..24 {
        let centro = [-1.2 + 0.1 * k as f32, 0.0, 0.0];
        centros.push(centro);
        if refina {
            let _ = ph2d_mesh::refine_in_sphere(
                &mut malha,
                centro,
                brush.radius,
                0.035,
                &mut births,
                &mut region,
            );
            stroke.grow_with(&malha, &births);
        }
        stroke.dab(
            &mut malha,
            &brush,
            &Dab::at(centro, brush.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    (malha, centros)
}

/// ⭐⭐⭐ **A MEDIÇÃO: o `G-2` sobre a nossa saída.**
///
/// Barra do corpus: `Q ≥ +0,0465` com o pente no máximo e `Q ≤ +0,0465` com ele
/// desligado — a **mesma** barra, o meio de um vale cujos dois lados são saída do
/// próprio alvo.
#[test]
#[ignore = "sonda"]
fn diag_o_g2_sobre_a_nossa_saida() {
    for refina in [false, true] {
        let etiqueta = if refina { "com refino" } else { "sem refino" };
        let (m0, c0) = traco(0.0, refina);
        let (m1, c1) = traco(1.0, refina);
        let (q0, n0) = q_da_faixa(&m0, &c0, 0.30);
        let (q1, n1) = q_da_faixa(&m1, &c1, 0.30);
        println!(
            "{etiqueta}:  Q desligado {q0:+.4} (n={n0})  ·  Q no maximo {q1:+.4} (n={n1})  \
             ·  ΔQ {:+.4}  ·  barra +0,0465  ⇒  {}",
            q1 - q0,
            if q1 >= 0.0465 { "PASSA" } else { "FALHA" }
        );
    }
}

/// ⭐⭐⭐ **A NOSSA MALHA PENTEIA-SE ACIMA DA BARRA DO ORÁCULO — e é isto que
/// responde à pergunta que a espec deixou EM ABERTO.**
///
/// A espec §14.5 manda: *faça a hipótese **(b)** primeiro — a relaxação dentro
/// do laço por-carimbo — e meça contra o `G-2`; se a nossa malha ficar abaixo da
/// barra com a lei já no sítio certo, a causa é a **(a)** e há uma segunda lei a
/// construir.* Medido, no regime em que a barra foi calibrada (com o passe de
/// refino, que é o da fixtura `escada/k_*`):
///
/// | | `Q` desligado | `Q` no máximo | `ΔQ` |
/// |---|---|---|---|
/// | **com refino** | `−0,0192` | **`+0,1471`** | `+0,1664` |
/// | sem refino | `+0,0541` | `+0,3322` | `+0,2781` |
///
/// ⇒ **`+0,1471` contra a barra de `+0,0465`: `3,2×`.** Pelo critério que a
/// própria espec prescreveu, **NÃO há segunda lei a construir.**
///
/// ⚠️⚠️ **E a indicação que sugeria o contrário era falível, e estava errada.** A
/// §14.3.1 extrapola a composição em série em `1/n` e aterra em `+0,0393`,
/// **abaixo** da barra — o que o §14.5.2 lê como *«(b) sozinha pode não
/// chegar»*, com quatro fraquezas declaradas. Ela chegou. *Uma indicação
/// declarada como falível que se mede e cai é a declaração a funcionar.*
///
/// ⭐⭐ **E há uma concordância INDEPENDENTE que ninguém forçou:** a espec §2.2
/// mede que no alvo o efeito é **MAIOR sem o passe de refino** (`ΔQ = +0,2061`
/// contra `+0,1835` com ele). A nossa lei, escrita sem olhar para esse número,
/// lê `+0,2781` contra `+0,1664` — *a mesma ordem, pelo mesmo mecanismo: os
/// vértices que o refino insere não sabem nada do traço.*
///
/// ⛔ **A linha «sem refino» NÃO entra no gate**, e a razão é da fixtura: ali o
/// lado desligado lê `+0,0541`, ligeiramente **acima** da barra, porque uma
/// chapa sacudida a `0,7` do passo ainda é meio alinhada. *A barra foi
/// calibrada no regime COM refino, e é nele que ela afirma.*
#[test]
fn a_nossa_malha_penteia_se_acima_da_barra_do_oraculo() {
    /// A barra do corpus: o MEIO do vale, e ela é uma só — `≥` de um lado e `≤`
    /// do outro. ⛔ Não é um número escolhido: os dois lados do vale
    /// (`+0,0298` e `+0,0632`) são saída do **próprio alvo**.
    const BARRA: f64 = 0.0465;

    let (m0, c0) = traco(0.0, true);
    let (m1, c1) = traco(1.0, true);
    let (q0, n0) = q_da_faixa(&m0, &c0, 0.30);
    let (q1, n1) = q_da_faixa(&m1, &c1, 0.30);

    // (1) — **o controlo, e sem ele as outras duas metades não afirmam nada:**
    // o lado DESLIGADO tem de estar abaixo da barra. Uma fixtura cuja malha já
    // nasce alinhada com o traço passa a metade (2) sem o pente fazer nada — foi
    // exactamente o que a 1.ª redacção desta chapa fazia (`+0,2885`).
    assert!(
        n0 > 3_000 && n1 > 3_000,
        "a populacao encolheu ({n0} e {n1} arestas na faixa) — o arranjo deixou \
         de conter o que este gate mede"
    );
    assert!(
        q0 <= BARRA,
        "com o pente DESLIGADO a nossa malha ja' le' Q {q0:+.4} contra a barra \
         de {BARRA:+.4} (medido −0,0192) — a fixtura nasce alinhada com o traco \
         e a metade (2) passa a afirmar o vazio"
    );

    // (2) — e o lado LIGADO passa a barra do oráculo. **É esta linha que diz que
    // a hipótese (b) chega**, e com ela a pergunta «uma lei ou duas» fecha.
    assert!(
        q1 >= BARRA,
        "com o pente no maximo a nossa malha le' Q {q1:+.4} contra a barra de \
         {BARRA:+.4} (medido +0,1471) — se isto reprovou, a causa e' a (a) da \
         espec §14.4 e ha' uma SEGUNDA lei a construir: o pente do alvo enviesa \
         as decisoes do passe de topologia, e a relaxacao sozinha nao chega"
    );
}
