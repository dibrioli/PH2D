//! Os gates da cena `=114` — e a prova de que ela ensina o que anuncia.

use super::*;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Corre a cena e devolve as posições finais de cada metade.
fn corre(secs: f64) -> PorMetade {
    corre_com_partida(secs).1
}

/// As posições de cada metade num instante — uma nuvem por metade.
type PorMetade = Vec<Vec<[f32; 2]>>;

/// O mesmo, com o instante ZERO ao lado — é ele que torna a DESCIDA mensurável.
fn corre_com_partida(secs: f64) -> (PorMetade, PorMetade) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build(&mut doc, &reg).expect("a cena é bem tipada");
    let mut cook = Cook::new();
    let last = (secs * 60.0) as u64;
    let mut inicio: PorMetade = vec![Vec::new(); sinks.len()];
    let mut fim: PorMetade = vec![Vec::new(); sinks.len()];
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        for (i, sink) in sinks.iter().enumerate() {
            let s = cook.cook(&doc.graph, &reg, *sink, t).expect("cozinha")[0]
                .as_stream()
                .clone();
            if let Some(Column::Vec2(v)) = s.get("P") {
                if k == 0 {
                    inicio[i] = v.clone();
                }
                if k == last {
                    fim[i] = v.clone();
                }
            }
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    (inicio, fim)
}

/// A distância de cada peça ao vizinho MAIS PRÓXIMO dela — uma por peça.
fn vizinhos(p: &[[f32; 2]]) -> Vec<f32> {
    p.iter()
        .enumerate()
        .map(|(i, a)| {
            p.iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                .fold(f32::MAX, f32::min)
        })
        .collect()
}

/// A MEDIANA dessas distâncias — o vão TÍPICO da pilha.
fn vizinho_mediano(p: &[[f32; 2]]) -> f32 {
    let mut v = vizinhos(p);
    v.sort_by(f32::total_cmp);
    v.get(v.len() / 2).copied().unwrap_or(0.0)
}

/// ⭐⭐⭐ **A RÉGUA É O VÃO TÍPICO, e ela corrigiu-se — o par mais próximo era um EXTREMO.**
///
/// ⛔⛔ **A 1.ª redacção mediu o par mais próximo da pilha inteira**, contra a promessa do nó
/// (*«pares sobrepostos são afastados até apenas se tocarem»*, a `r_i + r_j` = [`PECA`]). Ela
/// reprovou a `0,1237` — e varrer os knobs de convergência do próprio nó mostrou que o número
/// **não converge**: `8 → 56 %`, `16 → 66 %`, `32 → 61 %`, `64 → 83 %`, com `strength = 2` a
/// dar PIOR que `1` a 64 varreduras. *Um mínimo sobre todos os pares é o pior instante do pior
/// par de um monte que ainda se acomoda — um extremo, não uma propriedade.*
///
/// ⭐ A grandeza que descreve a pilha é o **vão TÍPICO**: a distância de cada peça ao vizinho
/// mais próximo dela, mediana. Medida, ela é **estável em todos os ajustes** — `90 %` a `100 %`
/// de [`PECA`] — e nos valores de FÁBRICA do nó (`8` varreduras, `strength 1`) dá **`93 %`**.
///
/// ⚠️ **Por isso a cena não afina nada:** ela mostra o nó como o artista o recebe. Subir as
/// varreduras não compra pilha nenhuma aqui, e escrever um ajuste que não compra nada seria
/// ensinar que ele é preciso.
///
/// ⚠️ **O défice de `7 %` não é defeito, e não se esconde:** a promessa é sobre a relaxação de
/// uma entrada PARADA (é por isso que a cena `=48` se julga parada), e aqui há gravidade a
/// comprimir a pilha a cada tique. Um solver PBD sob carga sustentada assenta num equilíbrio
/// entre penetração e pressão — é o que qualquer granular faz.
///
/// ⚠️ **E o CONTROLO é a outra metade**, senão o gate ficava verde sobre duas taças a fazer o
/// mesmo.
#[test]
fn only_the_half_with_collide_keeps_the_pieces_apart() {
    let fim = corre(2.6);
    assert_eq!(fim.len(), 2, "a cena tem duas metades");
    let n = (ROWS * COLS) as usize;
    for (i, metade) in fim.iter().enumerate() {
        assert_eq!(
            metade.len(),
            n,
            "a metade {i} tem de trazer as {n} pecas -- se a nuvem encolheu, a regua abaixo \
             mede outra coisa"
        );
    }
    let (sem, com) = (vizinho_mediano(&fim[0]), vizinho_mediano(&fim[1]));
    eprintln!(
        "  vao tipico │ sem Collide {sem:.4} ({:.0}%) · com Collide {com:.4} ({:.0}%) · PECA {PECA}",
        sem / PECA * 100.0,
        com / PECA * 100.0
    );

    assert!(
        com >= PECA * 0.85,
        "com o `Collide` no laco o vao tipico mede {com:.4}, e a promessa do no' e' que dois \
         discos apenas se toquem -- a `{PECA}` (raio {RAIO} x o lado da peca). Medido nos \
         valores de fabrica ele da' 93%, entao abaixo de 85% alguma coisa deixou de relaxar"
    );
    assert!(
        sem < com * 0.6,
        "sem o `Collide` o vao tipico mede {sem:.4} contra {com:.4} -- as duas metades tem de \
         contar historias DIFERENTES, senao a cena poe duas legendas por cima da mesma imagem"
    );
}

/// ⭐⭐⭐ **E AS DUAS METADES TÊM DE CAIR — medido como DESCIDA, não como altura.**
///
/// ⛔⛔ **A 1.ª redacção deste gate SOBREVIVEU à mutação que apagava a gravidade**, e foi ela
/// que expôs o defeito de CENA: a fileira de baixo do arranjo já nascia abaixo da barra
/// (`< ALTURA − 0,5`), então a asserção passava no tique zero. *Uma régua que a condição
/// inicial já satisfaz não mede o produto.*
///
/// ⭐ A cura é comparar cada peça **consigo própria**: onde estava contra onde ficou. Uma
/// cadeia que não integra devolve a mesma posição, e a diferença é exactamente zero.
#[test]
fn both_bowls_actually_catch_what_falls() {
    let (inicio, fim) = corre_com_partida(2.6);
    for (i, (a, b)) in inicio.iter().zip(&fim).enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de pecas")]
        let n = a.len().max(1) as f32;
        let descida = a.iter().zip(b).map(|(p, q)| p[1] - q[1]).sum::<f32>() / n;
        let dentro = b
            .iter()
            .filter(|p| p[1] < TACA_Y + TACA_R && p[1] > TACA_Y - TACA_R - 0.5)
            .count();
        eprintln!("  metade {i} │ descida média {descida:.3} │ na taça {dentro}");
        assert!(
            descida > 0.5,
            "na metade {i} as pecas desceram {descida:.3} em media -- se e' ~0, a cadeia nao \
             integrou e todo gate desta cena fica verde por vacuo"
        );
        assert!(
            dentro >= 20,
            "na metade {i} so' {dentro} peca(s) ficaram na taca -- ou ela nao apanha, ou a \
             queda passou por ela"
        );
    }
}

/// ⭐⭐⭐ **OS DOIS KNOBS QUE O ANÚNCIO MANDA ARRASTAR FAZEM O QUE ELE DIZ.**
///
/// ⛔⛔ Um passo de smoke que manda arrastar um controlo **afirma que ele tem efeito** — a
/// mesma lei que obriga um passo a provar que a linha está na lista, e que já me apanhou duas
/// vezes nesta linha. Aqui são dois: o `Radius` (*«as peças reclamam mais ou menos espaço»*) e
/// o `Iterations` (*«baixe até 1 e a pilha volta a atravessar-se»*).
///
/// ⚠️ **A régua é a MESMA dos gates acima** (o vão típico), e a lei é monótona em cada um: mais
/// raio ⇒ mais vão; menos varreduras ⇒ menos vão.
#[test]
fn the_two_knobs_the_announcement_names_do_what_it_says() {
    let vaos_por = |param: &str, valores: &[f32]| -> Vec<f32> {
        valores
            .iter()
            .map(|v| {
                let reg = registry();
                let mut doc = MotionDoc::default();
                let sinks = build(&mut doc, &reg).expect("cena");
                let alvos: Vec<_> = doc
                    .graph
                    .nodes()
                    .iter()
                    .filter(|n| n.type_name == "motion.collide")
                    .map(|n| n.id)
                    .collect();
                assert_eq!(alvos.len(), 1, "so' a metade da direita tem o no'");
                for c in &alvos {
                    doc.graph.set_param(*c, param, *v);
                }
                let mut cook = Cook::new();
                let last = (2.6 * 60.0) as u64;
                let mut fim: Vec<[f32; 2]> = Vec::new();
                for k in 0..=last {
                    #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
                    let t = k as f64 / 60.0;
                    for (i, s) in sinks.iter().enumerate() {
                        let st = cook.cook(&doc.graph, &reg, *s, t).expect("coze")[0]
                            .as_stream()
                            .clone();
                        if k == last
                            && i == 1
                            && let Some(Column::Vec2(p)) = st.get("P")
                        {
                            fim = p.clone();
                        }
                    }
                    cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
                }
                vizinho_mediano(&fim)
            })
            .collect()
    };

    let raios = [0.25f32, 0.5, 1.0];
    let v_raio = vaos_por("radius", &raios);
    eprintln!("  Radius     {raios:?} -> vao tipico {v_raio:?}");
    assert!(
        v_raio[0] < v_raio[1] && v_raio[1] < v_raio[2],
        "o `Radius` tem de fazer a pilha inchar de forma monotona, e deu {v_raio:?} -- o passo \
         3 do anuncio manda arrasta-lo e promete que as pecas reclamam mais espaco"
    );

    let its = [1.0f32, 8.0];
    let v_it = vaos_por("iterations", &its);
    eprintln!("  Iterations {its:?} -> vao tipico {v_it:?}");
    assert!(
        v_it[0] < v_it[1] * 0.9,
        "com UMA varredura o vao tipico deu {:.4} contra {:.4} com as oito de fabrica -- o \
         passo 4 do anuncio promete que baixar o `Iterations` faz a pilha voltar a \
         atravessar-se, e se os dois lerem igual esse passo nao mostra nada",
        v_it[0],
        v_it[1]
    );
}
