//! **O decaimento da umidade: o passe row-parallel + o rect que encolhe** (2026-08-02).
//!
//! O log do produto (`PH2D_PAINT_PERF`, canvas 4096², pincel 250) mediu `secagem 10-16 ms` **em todo
//! quadro** — o maior item isolado do quadro do artista, e ele roda pinte-se ou não.
//!
//! ⚠️ **TRÊS curas foram construídas antes desta e as TRÊS mediram ~1,00×.** A hipótese era que o
//! snapshot do rect (um `vec![0; rw*rh]` por quadro + a cópia) fosse o custo: uma **janela deslizante**
//! o substituiu (`up` de uma linha de scratch, `left` de um escalar, `down`/`right` do próprio mapa) e
//! entregou **1,02×**; o **piso da erosão** e o **rect que encolhe** entregaram o mesmo. O custo é o
//! CAMINHAR — 2,2 ns/texel — e nada disso o toca. A janela deslizante foi **revertida**, porque a
//! dependência entre linhas que ela cria é exatamente o que impede o paralelo que funciona.
//!
//! O que ficou:
//!
//! 1. **O passe é row-parallel** (ADR-0109): 28,87 -> 3,45 ms a 4096² (**8,4×**), byte-idêntico.
//! 2. **O rect ENCOLHE** para a bbox do não-zero — a secagem é edges-to-centre por desenho, então a
//!    poça recua. Vale ~1,0× no relógio e é real no véu de umidade do shell, que lê o mesmo rect.
//! 3. **O piso da erosão** — `erode = gap*step*2/255` só alcança 1 acima de `ceil(255/(step*2))`, e
//!    `gap <= o`, então abaixo do piso os quatro vizinhos não mudam um byte e não são lidos.
//!
//! ⚠️ **O oráculo destes gates é a rotina que SHIPAVA**, congelada abaixo verbatim — não uma
//! reescrita minha. Comparar duas versões que eu escrevi na mesma tarde prova que eu fui consistente,
//! não que o produto não mudou.
//!
//! ⚠️ E a fixture contém o fenômeno de propósito: a poça tem **estrutura interior** (sem gradiente o
//! `gap` é zero e um vizinho errado não muda um byte), o rect nasce **maior que a poça** (é o que a
//! união cumulativa de verdade é) e o `step` é **varrido** — ver o gate.

use super::*;

/// A rotina de decaimento que shipou até 2026-08-02, **congelada**: snapshot do rect inteiro e rect
/// que nunca encolhe. Devolve o `wettest`, como a original. É o oráculo, e por isso é uma cópia
/// literal — qualquer "melhoria" aqui destrói o que o gate mede.
fn decay_snapshot(wet: &mut [u8], fw: usize, rect: (usize, usize, usize, usize), step: u8) -> u8 {
    let (x0, y0, x1, y1) = rect;
    let (rw, rh) = (x1 - x0, y1 - y0);
    let mut old = vec![0u8; rw * rh];
    for oy in 0..rh {
        let src = (y0 + oy) * fw + x0;
        old[oy * rw..oy * rw + rw].copy_from_slice(&wet[src..src + rw]);
    }
    let mut wettest = 0u8;
    for oy in 0..rh {
        for ox in 0..rw {
            let o = old[oy * rw + ox];
            if o == 0 {
                continue;
            }
            let up = if oy > 0 { old[(oy - 1) * rw + ox] } else { 0 };
            let down = if oy + 1 < rh {
                old[(oy + 1) * rw + ox]
            } else {
                0
            };
            let left = if ox > 0 { old[oy * rw + ox - 1] } else { 0 };
            let right = if ox + 1 < rw {
                old[oy * rw + ox + 1]
            } else {
                0
            };
            let gap = o.saturating_sub(up.min(down).min(left).min(right));
            let erode =
                ((u32::from(gap) * u32::from(step) * super::watercolor_backdrop::WET_ERODE_GAIN)
                    / 255) as u8;
            let nv = o.saturating_sub(step.saturating_add(erode));
            wet[(y0 + oy) * fw + x0 + ox] = nv;
            wettest = wettest.max(nv);
        }
    }
    wettest
}

/// Uma poça com ESTRUTURA num canvas `n`×`n`: um domo com ondulação, dentro de um rect folgado.
/// Devolve `(mapa, rect)` — o rect é maior que a poça de propósito (a união cumulativa é assim).
fn puddle(n: usize) -> (Vec<u8>, (usize, usize, usize, usize)) {
    let mut wet = vec![0u8; n * n];
    let (cx, cy) = (n as f32 * 0.5, n as f32 * 0.5);
    let r = n as f32 * 0.34;
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = (x as f32 - cx, y as f32 - cy);
            let d = (dx * dx + dy * dy).sqrt();
            if d >= r {
                continue;
            }
            // Domo + ondulação: o interior tem gradiente em toda parte, então o termo de erosão
            // (que lê os 4 vizinhos) está VIVO no miolo e não só na borda.
            let dome = 1.0 - d / r;
            let ripple = 0.18 * (((x * 7 + y * 11) % 23) as f32 / 23.0);
            wet[y * n + x] = ((dome * 0.82 + ripple) * 255.0).clamp(1.0, 255.0) as u8;
        }
    }
    // Rect folgado: 6 px de margem em torno da poça, como uma união cumulativa que já secou nas bordas.
    let m = (n as f32 * 0.10) as usize;
    (wet, (m, m, n - m, n - m))
}

const DT: f32 = 0.5;

/// Um tool com a poça instalada e o relógio armado para dar EXATAMENTE `step` por chamada
/// (`DT * rate = step`, com `rate = 2*step` ⇒ exato em `f32`, sem deriva de carry entre as rotas).
fn dry_tool(n: usize, step: u8) -> (PainterTool, Vec<u8>, (usize, usize, usize, usize)) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; n * n * 4], n as u32, n as u32);
    let (wet, rect) = puddle(n);
    t.paint.canvas_wet = wet.clone();
    t.paint.canvas_wet_rect = Some(rect);
    t.paint.canvas_wet_carry = 0.0;
    t.paint.dry_rate_per_s = f32::from(step) * 2.0;
    (t, wet, rect)
}

#[test]
fn the_parallel_pass_decays_exactly_like_the_serial_one_it_replaced() {
    // ⚠️ **O `step` é PARTE da fixture, e a 1ª versão deste gate não sabia disso.** A erosão é
    // `gap * step * 2 / 255` em inteiros, e um vizinho errado erra o `gap` por ~`step`; o erro só
    // atravessa a quantização quando `step² * 2 >= 255`, ou seja `step >= 12`. Com o `step = 5` que eu
    // havia escolhido, a mutação *"leia um vizinho já escrito"* **SOBREVIVEU** — verde sobre um vizinho
    // errado. O produto normalmente anda a `step = 1`, onde o erro é invisível; mas *"invisível no
    // ponto de operação de hoje"* é como um vizinho errado vive até alguém mexer no Drying Time.
    // O gate pina a LEI, então varre o `step` — e é ele que cruza o piso da erosão nos dois sentidos.
    // ⚠️ **E o TAMANHO é fixture tanto quanto o `step`.** O produto roda serial abaixo do piso do
    // pool (`DRY_PAR_MIN`) e paralelo acima dele; a 1ª versão deste gate usava só 128² = 10.816
    // texels e portanto **nunca entrou na rota paralela que a wave existe para instalar** — verde
    // sobre o caminho que não mudou. Os dois lados do piso, então.
    for &n in &[128usize, 384] {
        for &step in &[1u8, 5, 17, 51] {
            let (mut t, mut reference, rect) = dry_tool(n, step);
            let passes = (200 / u32::from(step)).max(3);
            for pass in 0..passes {
                t.dry_canvas_wet(DT);
                // A rota congelada NUNCA encolhe o rect — é o comportamento que ela tinha.
                let want = decay_snapshot(&mut reference, n, rect, step);
                assert_eq!(
                    t.paint.canvas_wet, reference,
                    "n {n}, step {step}, passe {pass}: o passe paralelo divergiu do serial congelado"
                );
                assert!(
                    want > 0,
                    "step {step}, passe {pass}: a fixture secou antes do fim — sem poça não há o que medir"
                );
            }
            assert!(
                t.paint.canvas_wet.iter().any(|&v| v > 0),
                "n {n}, step {step}: a poça tem de sobreviver — comparar dois zeros é verde por vácuo"
            );
        }
    }
}

#[test]
fn the_rect_follows_the_puddle_in_and_nothing_wet_is_left_outside_it() {
    let n = 128;
    let (mut t, _, rect) = dry_tool(n, 5);
    for _ in 0..20 {
        t.dry_canvas_wet(DT);
    }
    let now = t.paint.canvas_wet_rect.expect("a poça ainda está molhada");
    assert!(
        now.0 > rect.0 && now.1 > rect.1 && now.2 < rect.2 && now.3 < rect.3,
        "o rect tem de recuar nos QUATRO lados: {rect:?} -> {now:?}"
    );
    // A metade que torna o encolhimento seguro: fora do rect novo não pode sobrar umidade, senão o
    // decaimento do próximo quadro deixa de alcançá-la e ela fica molhada para sempre.
    for y in 0..n {
        for x in 0..n {
            let inside = x >= now.0 && x < now.2 && y >= now.1 && y < now.3;
            if !inside {
                assert_eq!(
                    t.paint.canvas_wet[y * n + x],
                    0,
                    "texel molhado em ({x},{y}), fora do rect {now:?}"
                );
            }
        }
    }
}

/// **O que o passe de secagem custa, pelas DUAS rotas, na mesma corrida.**
///
/// ⚠️ **Costas-com-costas e DENTRO da corrida, de propósito** (a lição da §5.46): esta worktree divide
/// 32 núcleos com outras linhas, e o mesmo passo do produto já variou 2× entre corridas sem uma linha
/// de código mudar. Um A/B cross-run atribuiria a carga da máquina ao ganho. Aqui a carga é fator
/// comum, e o estado é RESTAURADO antes de cada amostra (repetir o passe sobre o mesmo mapa o deixa
/// mais seco e mais barato — a fixture envenenada da §5.41).
///
/// Rode com `cargo test -p ph2d-tool-painter --release the_cost_of_the_drying_pass -- --ignored
/// --nocapture --test-threads=1`, e **só com a máquina calma** (`uptime` abaixo de ~5).
#[test]
#[ignore = "sonda de tempo: rode sozinha, com a maquina calma"]
fn the_cost_of_the_drying_pass_by_both_routes() {
    for n in [2048usize, 4096] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; n * n * 4], n as u32, n as u32);
        // A poça do produto: quase toda a tela, funda (step 1 leva 255 quadros para secar, então o
        // rect mal encolhe durante a amostragem e as duas rotas caminham a MESMA área).
        let m = n / 16;
        let rect = (m, m, n - m, n - m);
        let mut wet = vec![0u8; n * n];
        for y in rect.1..rect.3 {
            for x in rect.0..rect.2 {
                wet[y * n + x] = 200 + ((x * 3 + y * 5) % 55) as u8;
            }
        }
        t.paint.dry_rate_per_s = 2.0; // DT 0.5 -> step 1, o regime do produto
        let (mut door, mut frozen) = (Vec::new(), Vec::new());
        for _ in 0..5 {
            t.paint.canvas_wet = wet.clone();
            t.paint.canvas_wet_rect = Some(rect);
            t.paint.canvas_wet_carry = 0.0;
            let a = std::time::Instant::now();
            t.dry_canvas_wet(DT);
            door.push(a.elapsed().as_secs_f64() * 1e3);

            let mut m2 = wet.clone();
            let b = std::time::Instant::now();
            decay_snapshot(&mut m2, n, rect, 1);
            frozen.push(b.elapsed().as_secs_f64() * 1e3);
        }
        door.sort_by(f64::total_cmp);
        frozen.sort_by(f64::total_cmp);
        let (d, f) = (door[2], frozen[2]);
        println!(
            "UM passe {n}x{n} area {:.2} M | antes {f:.2} ms | agora {d:.2} ms | {:.2}x",
            ((rect.2 - rect.0) * (rect.3 - rect.1)) as f64 / 1e6,
            f / d
        );

        // E agora o que o produto DE FATO faz: uma poça SECANDO ao longo de muitos quadros. É aqui
        // que o piso da erosão e o rect que encolhe vivem — a poça cheia acima está inteiramente
        // acima do piso e dentro do rect original, então ela mede só a janela deslizante.
        t.paint.canvas_wet = wet.clone();
        t.paint.canvas_wet_rect = Some(rect);
        t.paint.canvas_wet_carry = 0.0;
        let a = std::time::Instant::now();
        let mut ran = 0u32;
        for _ in 0..120 {
            if t.paint.canvas_wet_rect.is_none() {
                break;
            }
            t.dry_canvas_wet(DT);
            ran += 1;
        }
        let now_total = a.elapsed().as_secs_f64() * 1e3;

        let mut m2 = wet.clone();
        let b = std::time::Instant::now();
        for _ in 0..ran {
            decay_snapshot(&mut m2, n, rect, 1);
        }
        let before_total = b.elapsed().as_secs_f64() * 1e3;
        println!(
            "  SECANDO ({ran} quadros)          | antes {before_total:.1} ms | agora {now_total:.1} ms | {:.2}x  ({:.2} -> {:.2} ms/quadro)",
            before_total / now_total,
            before_total / f64::from(ran),
            now_total / f64::from(ran)
        );
    }
}

/// **E o passe de secagem caminha em PARALELO — é a única coisa que segura os 9,3×.**
///
/// ⚠️ **Nenhum outro gate pega esta regressão**, e é literalmente a lição que o irmão do fold do
/// impasto já carrega neste crate: serial e paralelo produzem os MESMOS bytes — que é precisamente a
/// propriedade que torna a cura segura —, então o gate de identidade fica verde sobre um passe que
/// voltou a custar 28 ms. Trocar `par_chunks_mut` por `chunks_mut` é **uma letra**.
///
/// ⚠️ **Arquitetural de propósito:** *"este laço roda em paralelo"* é afirmação sobre a FORMA do
/// código. Uma barra de milissegundos mediria o perfil do build (o `ci-test` compila em `opt-level=1`)
/// e, pior, mediria as OUTRAS sondas rodando ao lado — a razão de duas rotas onde só uma é paralela
/// não sobrevive a uma suíte concorrente. O número vive na sonda `#[ignore]` acima; o que este gate
/// guarda é que o mecanismo que o produziu continua lá.
///
/// **Mutação que deve sangrar:** `par_chunks_mut` → `chunks_mut` no walk do decaimento.
#[test]
fn the_drying_pass_walks_in_parallel_because_the_rows_are_disjoint() {
    let src = include_str!("watercolor_backdrop.rs");
    // Controle positivo: a função tem de ser ENCONTRADA, senão o gate passa por não achar nada.
    let at = src
        .find("fn dry_canvas_wet_inner(")
        .expect("controle: o corpo do decaimento tem de existir");
    let body = &src[at..];
    let end = body
        .find("\n    }\n")
        .expect("controle: a funcao tem de terminar");
    let body = &body[..at + end - at];
    assert!(
        body.contains("par_chunks_mut("),
        "o walk do decaimento tem de percorrer as linhas em paralelo — elas sao disjuntas (ADR-0109) \
         e este e o mecanismo dos 28,50 -> 2,93 ms/quadro a 4096². Nenhum gate de bytes pega esta \
         regressao: serial e paralelo dao os MESMOS bytes, por desenho"
    );
    // E a rota serial CONTINUA lá, com o piso do pool a escolher: um passe pequeno pagaria mais pelo
    // fork do que pelo trabalho, e é por isso que o gate de identidade varre os dois lados do piso.
    assert!(
        body.contains("chunks_mut(fw)") && body.contains("DRY_PAR_MIN"),
        "o piso do pool e a rota serial abaixo dele fazem parte do desenho, nao sao resto"
    );
}

#[test]
fn the_pour_table_is_the_expression_it_replaces() {
    // ⚠️ **O oráculo é a expressão ESCRITA AQUI, não a função do produto.** A 1ª versão deste gate
    // comparava a tabela com o `pour_hardening` que a CONSTRÓI — uma tautologia: mudar a expressão
    // muda os dois lados e o gate fica verde. É a forma exata que a `line/physics` documentou em três
    // gates ("um oráculo que usa a função sob teste para computar o que espera é sempre verde").
    // Este oráculo é a lei que o despejo avaliava por texel até 2026-08-02, congelada em prosa:
    // `smoothstep(SS0, SS1, cov/255) * 255`, truncado.
    for c in 0..=255u8 {
        let cov = f32::from(c) / 255.0;
        let want = (super::watercolor_field::smoothstep(
            super::watercolor_render::SS0,
            super::watercolor_render::SS1,
            cov,
        ) * 255.0) as u8;
        assert_eq!(
            super::watercolor_backdrop::pour_hardening_lut()[usize::from(c)],
            want,
            "a tabela de dureza do despejo divergiu da expressão em coverage={c}"
        );
    }
}
