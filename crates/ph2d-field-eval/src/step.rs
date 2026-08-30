//! ⭐⭐⭐ **O PASSO DA MARCHA — a lei que impede a peça de FURAR.**
//!
//! Irmão do [`crate`] por responsabilidade (teto de LOC da workspace): o `lib.rs` fica com a
//! **compilação** do documento em árvore; este ficheiro responde a uma pergunta só — *quanto é
//! seguro andar?*
//!
//! ⚠️ **O corte é o assunto certo, e não a conveniência:** as duas funções aqui são a única coisa na
//! crate cujo erro não fica lento — **fura**. Ter uma casa própria é o que faz a auditoria delas
//! (a tabela de `‖∇f‖` construtor a construtor, e a das composições) caber ao lado da lei em vez de
//! no meio do compilador.

use ph2d_field::{Blend, FieldDoc, NodeKind};

/// ⭐⭐⭐ **O PASSO SEGURO DESTE DOCUMENTO** (W56f) — e ele é do documento, não uma constante.
///
/// A marcha de esferas anda `d · s` e é segura enquanto `s · ‖∇f‖ ≤ 1`: se o campo sobe mais
/// depressa que a distância, um passo do tamanho do valor **atravessa** a superfície, e o furo
/// aparece como pixel de fundo no meio da peça.
///
/// ⇒ o passo é o **recíproco do tecto de `‖∇f‖`**, e quem o calcula é a [`gradient_bound`].
///
/// # ⭐ A auditoria, medida construtor a construtor
///
/// O traçador andava `1/√2` **em tudo**, e o número é o recíproco de uma constante medida na W0 —
/// `‖∇f‖ = √2` no arredondamento exacto. ⚠️ Mas o [`ph2d_field::Xform::scale`] deste módulo é
/// **uniforme de propósito**, e o doc dele já diz porquê: *"‖∇f‖ = 1 é a fundação de tudo neste
/// módulo"*. Se quase todo construtor honra a fundação, o passo curto é **o caminho mais lento a
/// definir o teto do mais rápido** (`CLAUDE.md` §0).
///
/// Medido (`the_table_of_who_inflates_the_gradient` + a varredura irmã, pior `‖∇f‖` sobre uma
/// grelha de 48³):
///
/// | construtor | pior `‖∇f‖` |
/// |---|---|
/// | as 6 primitivas, com e sem `round` | `1,000` |
/// | `Union` / `Intersection` / `Difference` **`Sharp`** | `1,000` |
/// | `Shell`, `Offset`, `Mirror` | `1,000` |
/// | `Array` (espaçamento `0,1`–`1,0`) · `Radial` (2–64) | `1,000` |
/// | `Organic` (`k` de `0` a `1,2`), nas três operações | `1,000` |
/// | escala uniforme (`0,2`–`4,0`) | `1,000` |
/// | **`Taper`** (declive `0` a `4`) | `1,000` → **`0,844`** |
/// | ⛔ **`Union`/`Intersection` `Exact`**, todo `r > 0` | **`1,4142`** |
/// | ⛔ **`Difference` `Exact`** | `1,000` até `r = 0,1`, **`1,143`** a `r = 0,6` |
///
/// ⭐ **O `Taper` DESCE** — ele subestima a distância, o que é seguro para a marcha. ⚠️ E a
/// `Difference Exact` é a que a régua quase deixou passar: a 1.ª fixtura usava `r = 0,1` e leu
/// `1,000` **exacto**. *Um valor não é uma família.*
#[must_use]
pub fn safe_march_step(doc: &FieldDoc) -> f32 {
    1.0 / gradient_bound(doc)
}

/// ⭐⭐⭐ **O TECTO DE `‖∇f‖` DESTE DOCUMENTO** (2026-08-30) — a soma dos QUADRADOS, não uma potência.
///
/// # ⛔ O report que isto cura: *«quanto mais objetos, mais artefatos e mais largos os vãos»*
///
/// Até hoje a lei era `passo = 2^(−profundidade/2)` — `√2` **por nível**, com a profundidade a
/// contar `n − 1` para um grupo de `n` formas filetadas (o `combine_trees` dobra aos pares, logo um
/// nó só já é uma corrente). ⚠️ **Isso é exponencial no número de peças na cena**, e a marcha tem um
/// tecto de passos (`MAX_STEPS = 400`): a partir de certo ponto o raio **acaba os passos antes de
/// chegar à superfície** e é largado em silêncio, o que se lê como fundo.
///
/// Medido pela porta do traçador (`measure_holes_versus_object_count`, 320×320, pixels acertados):
///
/// | formas | passo de ontem | acertos | passo de hoje | acertos |
/// |---:|---:|---:|---:|---:|
/// | 9 | `0,0625` | `21 795` | `0,3333` | `23 646` |
/// | 10 | `0,0442` | `19 793` | `0,3162` | `25 906` |
/// | 11 | `0,0312` | `13 992` | `0,3015` | `34 592` |
/// | 12 | `0,0221` | **`688`** | `0,2887` | `34 737` |
/// | 13 | `0,0156` | **`0`** | `0,2774` | `35 585` |
///
/// ⛔ **A peça DESAPARECE por inteiro a 13 formas.** *Uma lei conservadora de mais não fica lenta —
/// ela apaga o produto*, e o sintoma sobe com o número de peças exactamente como o report diz.
///
/// # ⭐⭐⭐ A lei certa, e ela DEMONSTRA-SE
///
/// Na região de mistura da união exacta (`ops::union_round`), com `u = max(r−a,0)` e
/// `v = max(r−b,0)`:
///
/// ```text
/// ∇f = (u·∇a + v·∇b) / ‖(u, v)‖        (o termo `max(min(a,b), r)` é constante ali)
/// ‖∇f‖ ≤ (u·L_a + v·L_b) / ‖(u,v)‖ ≤ √(L_a² + L_b²)      [Cauchy–Schwarz]
/// ```
///
/// ⇒ **os quadrados somam-se**, e não os logaritmos. Uma corrente de `n` folhas dá `√n`, e não
/// `√2^(n−1)`: a `n = 12` isso é `3,46` contra `45,3` — **13× de passo** que a lei antiga deitava
/// fora. E o `min` de uma junta viva não compõe nada: ali o tecto é o **maior** dos dois.
///
/// ⚠️ **O CHANFRO cabe na mesma lei, e também se demonstra:** o termo dele é `(a+b−r)/√2`, com
/// tecto `(L_a + L_b)/√2`, e `(L+1)²/2 ≤ L²+1 ⟺ (L−1)² ≥ 0`. *Não é uma coincidência aritmética —
/// é a mesma desigualdade, com o caso de igualdade em `L = 1`.*
///
/// # ⭐ Medido nas quatro formas de árvore (`measure_the_chain_of_fillets`, grelha 44³)
///
/// | árvore | `‖∇f‖` medido | tecto `√(ΣL²)` | folga |
/// |---|---:|---:|---:|
/// | plana `n = 2` | `1,4133` | `1,4142` | `0,1 %` |
/// | plana `n = 4` | `1,9913` | `2,0000` | `0,4 %` |
/// | plana `n = 8` | `2,7675` | `2,8284` | `2,2 %` |
/// | **equilibrada** (4 folhas, profundidade 2) | `1,9852` | `2,0000` | `0,7 %` |
/// | irmãs por junta **viva** | `1,4027` | `1,4142` | `0,8 %` |
///
/// ⛔⛔ **É a linha da EQUILIBRADA que prova que a PROFUNDIDADE não serve** — nem a antiga nem uma
/// `√(1+profundidade)`: ela mede `1,985` com profundidade `2`, e `√3 = 1,732` seria **furo**. *O que
/// conta é quantas folhas chegam ao ponto por misturas, não quantos níveis a árvore tem.*
///
/// ⚠️ **O tecto é PROVADO e a medição fica ABAIXO dele em todas as linhas** — é assim que tem de
/// ser: *um tecto de segurança prova-se, não se ajusta a um corpus*. Apertá-lo até à medição
/// transformaria «as peças que eu testei» em «as peças que existem».
///
/// ⚠️ **Uma escultura vale `L² = 2`, e isso mantém o passo que ela já tinha** (W77): o campo dela é
/// interpolado de uma grelha, e o pior `‖∇f‖` medido sobre a caixa inteira é **`1,0852`** (um cubo —
/// a forma com vinco; uma esfera dá `1,0016` e um octaedro `0,9029`). Gate:
/// `ph2d_field_mesh::tests::a_sculptures_field_never_out_climbs_the_march_step`.
/// ⚠️ **O limite DEMONSTRÁVEL de uma interpolação trilinear é `√3 ≈ 1,732`**, acima do `√2` que este
/// nível concede: cada componente do gradiente é um quociente de diferenças ≤ `1`, e três somam em
/// quadratura. As medições ficam em `1,09` porque saturar as três ao mesmo tempo exigiria a
/// superfície perpendicular aos três eixos **no mesmo ponto**, e uma distância com sinal não faz
/// isso. *A dívida fica escrita em vez de esquecida.*
#[must_use]
pub fn gradient_bound(doc: &FieldDoc) -> f32 {
    // A arena tem os filhos ANTES dos pais, então uma passagem para a frente basta.
    let mut sq = vec![1.0f32; doc.nodes().len()];
    for (i, node) in doc.nodes().iter().enumerate() {
        sq[i] = match &node.kind {
            // ⭐⭐⭐ **UMA PRIMITIVA TAMBÉM PODE INFLAR — e esta linha valia `1` para todas até à
            // W103.**
            //
            // ⛔ O filete do cone, do prisma, da cunha e da estrela é uma **interseção arredondada**
            // (é a única saída quando as paredes não são ortogonais), e ela infla exactamente como a
            // junta entre duas formas: medido `‖∇f‖ = 1,1943` num cone de declive `0,47`. Enquanto
            // aquele filete era **inerte** — e era, até àquela wave — a linha estava certa por
            // acidente; no dia em que ele passou a fazer alguma coisa, ela passou a mentir.
            //
            // ⚠️ A pergunta atravessa a porta do documento ([`ph2d_field::fillet_inflates`]) para a
            // lista ficar num sítio só.
            NodeKind::Leaf(p) => {
                if ph2d_field::fillet_inflates(p) {
                    2.0
                } else {
                    1.0
                }
            }
            // Ver o doc da função: `√2` de tecto, com `30 %` de folga sobre o medido.
            NodeKind::Sampled { .. } => 2.0,
            // ⭐⭐⭐ **A DOBRA, com o verbo EFECTIVO de cada filho** (2026-08-29 — o report do Enio).
            //
            // ⛔ Esta pergunta era `op.blend()`, a mistura DO GRUPO, e isso deixou de ser a única
            // que existe na W97. Com o verbo em cada forma e o raio de junção com ele (W98), o
            // filete de cada passo sai de `fold_verb(parent, filho.verb)` — que é exactamente o que
            // o [`crate::combine_trees`] lê ao construir a árvore.
            //
            // ⇒ com o grupo em `Sharp` e cada filho a trazer o seu `Exact`, o tecto lia `1`, o passo
            // ficava em `1,0`, e o raio **atravessava** a superfície: medido `passo 1,0000 ×
            // ‖∇f‖ 1,1717 = 1,17` com duas esferas e junta `0,05`. O sintoma que o Enio viu é o que
            // um furo dependente da direcção **é**: *«ao rotacionar, as áreas do joint mudam de
            // aspecto»*.
            //
            // ⚠️ **É o verbo EFECTIVO e não um dos dois lados**: ler só `node.verb` trocaria o
            // defeito pelo simétrico (o grupo com filete e os filhos calados voltaria a ler zero), e
            // somar todos os filhos sempre que o GRUPO arredonda castigaria a peça em que metade das
            // juntas foi pedida viva — o caminho lento a definir o teto do rápido (§0).
            //
            // ⚠️ **O primeiro filho SEMEIA e o verbo dele não é perguntado** — `n` filhos são `n−1`
            // passos de dobra. É a mesma lei do `fold_verb`, e escrevê-la aqui outra vez seria a
            // segunda cópia; por isso a pergunta atravessa a **porta** dele.
            NodeKind::Combine { op, children } => {
                let mut acc: Option<f32> = None;
                for c in children {
                    let child = sq.get(c.0 as usize).copied().unwrap_or(1.0);
                    let Some(a) = acc else {
                        acc = Some(child);
                        continue;
                    };
                    let verb = doc.node(*c).and_then(|n| n.verb);
                    acc = Some(if inflates(ph2d_field::fold_verb(*op, verb).blend()) {
                        // ⭐ **Cauchy–Schwarz**: os quadrados somam-se. Ver o doc da função.
                        a + child
                    } else {
                        // Uma junta viva é um `min`, e o gradiente dele é o de um dos dois.
                        a.max(child)
                    });
                }
                acc.unwrap_or(1.0)
            }
        };
    }
    // ⚠️ Os modificadores (`Shell`, `Offset`, `Mirror`, `Array`, `Radial`, `Taper`) não entram, e
    // isso é **medido**: os cinco primeiros lêem `1,000` por cima de um exacto e o `Taper` **desce**
    // a `0,8333`. *Eles lêem o campo, não o voltam a arredondar.*
    sq.get(doc.root().0 as usize)
        .copied()
        .unwrap_or(1.0)
        .max(1.0)
        .sqrt()
}

/// ⭐⭐⭐ **QUANTO O CAMPO ENCOLHE, no pior nó do documento** (2026-08-30) — o número que diz à
/// marcha quantos passos a mais um deformador custa.
///
/// # ⛔ O defeito que ele cura, e ele é o de 30/08 a entrar por outra porta
///
/// O divisor de um deformador vive no **operador**, e é a escolha certa: no [`gradient_bound`] ele
/// penalizaria a **cena inteira** por causa de uma peça torcida (o §0 ao contrário). A consequência
/// é que `safe_march_step` fica em `1,0` — o documento não infla — enquanto a região torcida devolve
/// `1/σ` da distância e pede `~σ×` mais passos para lá chegar.
///
/// ⛔ **E um raio que acaba os passos é largado em SILÊNCIO** ([`crate`] não o vê; quem o conta é o
/// `march::EXHAUSTED`). Medido numa barra a uma volta por unidade, `320²`: **18 raios esgotados e
/// 336 pixels de fundo dentro da peça**.
///
/// ⇒ o passo continua cheio (rápido onde o campo é honesto) e o **orçamento** é que cresce. ⭐ É de
/// graça pela mesma razão medida em 30/08: um orçamento maior custa o que os raios que dele precisam
/// custam, e não mais.
///
/// ⚠️ **O MÁXIMO sobre os nós, e não o produto pela árvore**: cada nó é dividido pela pilha dele, e
/// o pior encolhimento que a marcha pode encontrar em qualquer ponto é o do nó mais castigado.
#[must_use]
pub fn field_shrink(doc: &FieldDoc, reg: &crate::hybrid::Registry) -> f32 {
    let balls = crate::bounds::local_balls(doc, reg);
    let mut pior = 1.0f64;
    for (i, node) in doc.nodes().iter().enumerate() {
        let mut ball = balls[i].unwrap_or(crate::bounds::Ball::EMPTY);
        let mut aqui = 1.0f64;
        for m in &node.mods {
            aqui *= crate::stack::step_divisor(*m, ball);
            ball = crate::bounds::step_mod(ball, *m);
        }
        pior = pior.max(aqui);
    }
    #[allow(clippy::cast_possible_truncation)]
    let out = pior as f32;
    out.max(1.0)
}

/// **Esta mistura INFLA o gradiente?** — a pergunta de um passo da dobra, num sítio só.
///
/// ⭐⭐⭐ **O CHANFRO conta como um arredondamento exacto** (W99), e o balde é MEDIDO:
/// `the_chamfer_is_measured_against_the_march` lê o `‖∇f‖` dele no mesmo canto de 90º e afirma que
/// ele não passa o `√2` que este balde já paga. A demonstração de que ele **cabe** na lei da soma
/// dos quadrados está no doc da [`gradient_bound`].
///
/// ⚠️ **Pô-lo no balde do `Sharp` seria o erro que fura**: o termo do corte tem gradiente acima de
/// `1` onde as duas normais se alinham, e um passo do tamanho do valor atravessaria a superfície.
///
/// ⚠️ **Raio zero não infla**, e não é um caso de borda: é como o produto exprime *«junta viva»*
/// (o slider do filete a zero), e é o estado em que toda peça nasce.
fn inflates(blend: Blend) -> bool {
    match blend {
        Blend::Exact { radius } | Blend::Chamfer { radius } => radius != 0.0,
        Blend::Sharp | Blend::Organic { .. } => false,
    }
}
