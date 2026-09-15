//! ⭐⭐⭐ **O ESCALONADOR DA FITA** — a mesma aritmética, na ordem que cabe no ficheiro de registos.
//!
//! # ⛔⛔ O defeito que este módulo cura, e a nota que ele DESMENTE
//!
//! `docs/Render3d/05` §42.5 escreveu, em 2026-09-15, que a cura da peça desenhada na placa era *«o
//! contorno deixa de ser uma cadeia de `min` desenrolada e passa a ser uma **consulta**»*, com o
//! preço declarado ao lado: *«uma folha de dados não passa pela pilha de modificadores ⇒ só uma peça
//! **sem modificadores no contorno** a poderia usar»*. ⛔ **Esse preço não era necessário, e a nota
//! media uma PROCURAÇÃO.**
//!
//! O que faz a peça desenhada não caber não é a cadeia de `min` **existir** — é a **ORDEM** em que
//! ela é emitida. A travessia de `PointTape::build_with_vars` é uma DFS que
//! empilha os filhos e emite o de cima primeiro; sobre a cadeia esquerda
//! `min(min(min(s₀, s₁), s₂), s₃)` que o [`crate::profile`] constrói, isso dá:
//!
//! ```text
//! ordem da DFS :  s₃  s₂  s₁  s₀  min  min  min      ⇒ os N segmentos VIVOS ao mesmo tempo
//! ordem deste  :  s₀  s₁  min  s₂  min  s₃  min      ⇒ um acumulador e um segmento: DOIS
//! ```
//!
//! ⭐ **Os dois calculam o mesmo número, nó a nó** — o valor de um nó só depende dos filhos, e as
//! duas ordens são topológicas. O que muda é **quantos valores estão vivos ao mesmo tempo**, que é o
//! `vivos` do [`crate::point_tape::TapeShape`] e o scratch por thread no dispositivo.
//!
//! ⚠️ **E é uma LEI NUMA PORTA, não no perfil.** Arranjar o `sd_profile_inner` para construir a
//! cadeia noutra forma curaria o contorno e deixaria a superfórmula, as booleanas e as 62 primitivas
//! com a mesma ordem — *uma lei escrita em dois sítios ainda não é uma lei*. O escalonador corre
//! sobre a fita, que é onde todas elas já estão.
//!
//! # A lei de escolha, e por que a segunda metade dela não é gosto
//!
//! É um escalonamento de lista com duas chaves, nesta ordem:
//!
//! | chave | o que é | por que |
//! |---|---|---|
//! | **`delta`** | `1 −` quantos operandos MORREM ao emitir este nó | é a variação exacta do número de vivos: um nó que consome dois valores em fim de vida BAIXA a pressão |
//! | **`crítico`** | o comprimento do maior caminho deste nó até à raiz | desempata a favor de quem está mais longe do fim — e é isso que faz a cadeia acumular à medida que anda |
//!
//! ⚠️ **A segunda chave não é decoração: sem ela o `delta` empata em toda a partida.** No primeiro
//! passo os prontos são `x`, `y`, `z` e **todas** as constantes, e todos têm `delta = +1`. O
//! caminho crítico é o que sabe que o segmento `s₀` é o que a cadeia quer primeiro (ele está a `N`
//! `min`s da raiz; o `s_{N-1}` está a um).
//!
//! ⛔ **A ordem ORIGINAL da fita seria um desempate mudo e errado:** ela é a ordem de empilhamento da
//! `fidget`, que põe `s_{N-1}` primeiro — exactamente ao contrário do que a cadeia consome. *Um
//! desempate que depende da ordem de iteração de outra crate não é uma lei, é um acidente.*
//!
//! # ⚠️ O `vivos` é uma PROCURAÇÃO — e o relógio confirmou-a
//!
//! O `vivos` é a pressão de registos **da fita**, e quem aloca registos de verdade é o compilador da
//! placa: este módulo baixa a grandeza que se **conta**, e que ela mova o **relógio** era uma
//! afirmação sobre o `naga` e o driver. ⭐ Medida A/B na mesma corrida (`1920×1080`, CPU a `91 %`
//! ociosa), ela confirma-se:
//!
//! | arestas | quadro na ordem CRUA | escalonada | ganho |
//! |---:|---:|---:|---:|
//! | `32` | `27,7 ms` | `20,7 ms` | `1,3×` |
//! | `64` | `132,0` | `38,9` | `3,4×` |
//! | `128` | `233,6` | `140,0` | `1,7×` |
//! | `256` | `2 816,8` | `302,5` | **`9,3×`** |
//!
//! ⚠️ **A travessia com a CPU passou de `72`–`80` arestas para `128`** — e o tecto que a guarda
//! mudou de eixo (`ph2d_field_gpu::MAX_GUARDADOS`), porque o `vivos` deixou de discriminar: ele lê
//! `33` e `35` nos dois lados dela. Tabelas: `docs/Render3d/05` §43.

use crate::point_tape::Instr;

/// Os operandos de uma instrução, com a multiplicidade — `v * v` lê o mesmo slot **duas** vezes, e
/// contá-lo uma só faria o `delta` acreditar numa morte que não acontece.
fn operandos(i: &Instr) -> ([u32; 2], usize) {
    match i {
        Instr::X | Instr::Y | Instr::Z | Instr::Var(_) | Instr::Const(_) => ([0, 0], 0),
        Instr::Unary(_, a) => ([*a, 0], 1),
        Instr::Binary(_, a, b) => ([*a, *b], 2),
    }
}

/// ⭐⭐⭐ **A MESMA fita, reordenada para o mínimo de valores vivos.**
///
/// Devolve `(código, raiz)` — os índices são renumerados, e continuam a apontar **para trás**.
///
/// ⚠️ **Bit a bit a mesma resposta, por CONSTRUÇÃO:** a lista de instruções é uma permutação da
/// entrada, cada uma com os mesmos operandos (renumerados), e a ordem continua topológica. O valor
/// de um nó só depende dos filhos — *o que muda é o andaime, nunca a aritmética.*
#[must_use]
pub(crate) fn schedule(code: &[Instr], root: u32) -> (Vec<Instr>, u32) {
    let n = code.len();
    if n == 0 {
        return (Vec::new(), root);
    }

    // Quantos consumidores cada slot ainda tem por emitir. ⚠️ A raiz leva **+1**: ela sobrevive ao
    // fim da fita, e sem esse voto ela «morreria» e o último `delta` mentiria.
    let mut restam = vec![0u32; n];
    // Quantos operandos de cada nó faltam emitir.
    let mut por_emitir = vec![0u32; n];
    // Quem consome cada slot — a lista de pais, achatada.
    let mut pais_de: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (i, instr) in code.iter().enumerate() {
        let (ops, k) = operandos(instr);
        por_emitir[i] = u32::try_from(k).expect("no máximo dois operandos");
        for o in &ops[..k] {
            restam[*o as usize] += 1;
            #[allow(clippy::cast_possible_truncation)]
            pais_de[*o as usize].push(i as u32);
        }
    }
    restam[root as usize] += 1;

    // O caminho crítico: o maior número de passos deste nó até à raiz. ⚠️ Uma varredura DESCENDENTE
    // basta porque os operandos têm sempre índice menor — a fita de entrada já é topológica.
    let mut critico = vec![0u32; n];
    for i in (0..n).rev() {
        let (ops, k) = operandos(&code[i]);
        for o in &ops[..k] {
            critico[*o as usize] = critico[*o as usize].max(critico[i] + 1);
        }
    }

    // ⭐⭐⭐ **A fila por BALDES, e a propriedade que a torna exacta: o `delta` só DESCE.**
    //
    // `delta = nasce − mortes`, e `mortes` conta os operandos cuja leitura é a ÚLTIMA. À medida que
    // outros consumidores são emitidos, o `restam` de um operando só baixa ⇒ mais operandos chegam
    // a `restam == 1` ⇒ `mortes` só sobe ⇒ **`delta` é monótono não-crescente**. É isso que permite
    // guardar cada candidato no balde da chave com que entrou e **reconferi-la ao sair**: se ela
    // melhorou, ele desce de balde; nunca sobe.
    //
    // ⛔⛔ **A 1.ª redacção varria a lista inteira a cada passo** e o doc dela dizia, a defender-se,
    // *«um monte com chaves obsoletas é um defeito mudo»* — verdade para uma chave qualquer, e
    // **falso para uma monótona**. O preço dessa prudência foi medido: a montagem de um contorno de
    // 256 arestas custava `11,2 ms` contra `2,8` da fita crua (**`+301 %`**), e ela corre **a cada
    // quadro** (o `pedido` reconstrói a fita). *Uma cautela que não nomeia o mecanismo cobra caro.*
    //
    // `delta ∈ [-2, +1]` ⇒ quatro baldes. Dentro de cada um, o maior caminho crítico primeiro; o
    // índice desempata, para a ordem ser **determinista**.
    const BALDES: usize = 4;
    let balde_de = |d: i32| (d + 2).clamp(0, BALDES as i32 - 1) as usize;
    let delta_de = |cand: u32, restam: &[u32]| -> i32 {
        let (ops, k) = operandos(&code[cand as usize]);
        let mut mortes = 0i32;
        // ⚠️ Com o mesmo slot nos dois operandos, só há UMA morte — e ela só acontece se as duas
        // leituras forem as últimas.
        // ⚠️ E uma constante que morre não liberta registo nenhum, porque nunca ocupou um
        // ([`Instr::ocupa_registo`]): contá-la faria o escalonador perseguir uma folga que não
        // existe, e o pico de `256` arestas media `93` com `51` deles a serem `k[i]`.
        if k == 2 && ops[0] == ops[1] {
            mortes +=
                i32::from(restam[ops[0] as usize] == 2 && code[ops[0] as usize].ocupa_registo());
        } else {
            for o in &ops[..k] {
                mortes += i32::from(restam[*o as usize] == 1 && code[*o as usize].ocupa_registo());
            }
        }
        i32::from(code[cand as usize].ocupa_registo()) - mortes
    };

    let mut fila: [std::collections::BinaryHeap<(u32, u32)>; BALDES] = Default::default();
    for i in 0..n {
        if por_emitir[i] == 0 {
            #[allow(clippy::cast_possible_truncation)]
            let c = i as u32;
            fila[balde_de(delta_de(c, &restam))].push((critico[i], c));
        }
    }
    let mut novo: Vec<Instr> = Vec::with_capacity(n);
    let mut destino = vec![u32::MAX; n];

    loop {
        // Tira o melhor: o balde mais barato, e dentro dele o caminho crítico mais longo.
        let Some((b, (_, cand))) = (0..BALDES).find_map(|b| fila[b].pop().map(|e| (b, e))) else {
            break;
        };
        // ⚠️ **A chave pode ter melhorado desde que ele entrou** — reconfere e desce de balde.
        let agora = balde_de(delta_de(cand, &restam));
        if agora < b {
            fila[agora].push((critico[cand as usize], cand));
            continue;
        }
        let i = cand as usize;

        let (ops, k) = operandos(&code[i]);
        let instr = match code[i] {
            Instr::Unary(op, _) => Instr::Unary(op, destino[ops[0] as usize]),
            Instr::Binary(op, _, _) => {
                Instr::Binary(op, destino[ops[0] as usize], destino[ops[1] as usize])
            }
            folha => folha,
        };
        for o in &ops[..k] {
            restam[*o as usize] -= 1;
        }
        destino[i] = u32::try_from(novo.len()).expect("a fita cabe num u32");
        novo.push(instr);

        for p in &pais_de[i] {
            por_emitir[*p as usize] -= 1;
            if por_emitir[*p as usize] == 0 {
                fila[balde_de(delta_de(*p, &restam))].push((critico[*p as usize], *p));
            }
        }
    }

    debug_assert_eq!(novo.len(), n, "o escalonamento é uma permutação da fita");
    (novo, destino[root as usize])
}

#[cfg(test)]
#[path = "tape_schedule_tests.rs"]
mod tests;
