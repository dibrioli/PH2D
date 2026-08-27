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
///
/// # A regra, e por que ela é grosseira de propósito
///
/// Um documento **sem nenhum arredondamento exacto** anda `1,0`; com qualquer um, fica no `1/√2` de
/// sempre. ⛔ Não se compõe um limite por nó: encadear misturas pode compor os factores, e essa
/// pergunta **não foi medida**. Ficar no valor de hoje quando há um `Exact` não piora nada — o que
/// esta função faz é **deixar de castigar quem não o usa**. ⚠️ Uma [`ph2d_field::NodeKind::Sampled`]
/// (uma escultura) também fica no curto: o campo dela é interpolado de uma grelha, e ninguém mediu
/// o gradiente da interpolação.
#[must_use]
pub fn safe_march_step(doc: &FieldDoc) -> f32 {
    // `√2` por nível, e o expoente é a PROFUNDIDADE — ver [`inflation_depth`].
    2.0f32.powf(-0.5 * inflation_depth(doc) as f32)
}

/// ⭐⭐⭐ **QUANTOS NÍVEIS INFLANTES há no pior caminho raiz→folha** (W75) — o expoente do passo.
///
/// # ⛔ A cerca que estava aqui era FALSA, e o preço dela era a peça furar
///
/// Até 2026-08-26 este módulo perguntava *«há **algum** arredondamento exacto?»* e respondia com o
/// mesmo `1/√2` para todos os casos, com uma nota a dizer que compor os factores *«não foi
/// medido»*. **Foi medido, e eles compõem** (`the_table_of_the_gradient_of_a_composition`):
///
/// | composição | `‖∇f‖` | `passo × ‖∇f‖` com `1/√2` |
/// |---|---:|---:|
/// | `Union Exact 0,05` × 2 | `1,4142` | `1,00` |
/// | `Union Exact 0,2` × 2 | `1,5076` | **`1,07`** ⛔ |
/// | `Union Exact 0,5` × 2 | `1,6873` | **`1,19`** ⛔ |
/// | `Union Exact 0,2` × 3 | `1,7778` | **`1,26`** ⛔ |
/// | `Union Exact 0,5` × 3 | `1,9588` | **`1,39`** ⛔ |
///
/// ⚠️ Acima de `1` o passo é **maior que a distância até à superfície**: o raio atravessa-a, e o
/// sintoma é pixel de fundo no meio da peça. *Errar a classificação de um construtor não fica lento
/// — fura.*
///
/// # O que NÃO compõe, e também foi medido
///
/// Um arredondamento exacto por **baixo** de qualquer modificador fica em `1,4142` — `Shell`,
/// `Offset`, `Mirror`, `Radial`, `Array` (e o `Taper` **desce**, a `0,8333`). E uma `Difference`
/// exacta sobre uma `Union` exacta também fica em `1,4142`: ⭐ *o que compõe é o exacto que recebe um
/// campo **já inflado** no ramo que ele arredonda*, e a diferença lê o segundo operando pelo lado de
/// fora. ⇒ a profundidade conta **níveis de mistura encadeados**, não nós inflantes soltos.
///
/// # A barra é `√2` por nível, e é a PROVÁVEL, não a medida
///
/// O arredondamento exacto de dois campos `L`-Lipschitz é `√2·L` no pior caso, e encadear `k` deles
/// dá `√2^k`. As medições ficam **abaixo** disso (`1,96` contra `2,83` a `k = 3`) — e é assim que
/// tem de ser: *um teto de segurança prova-se, não se ajusta a um corpus*. Apertá-lo até à medição
/// seria transformar «as peças que eu testei» em «as peças que existem».
///
/// ⚠️ **Uma escultura conta como um nível, e isso agora está MEDIDO** (W77): o campo dela é
/// interpolado de uma grelha, e o pior `‖∇f‖` sobre a caixa inteira é **`1,0852`** (um cubo — a
/// forma com vinco; uma esfera dá `1,0016` e um octaedro `0,9029`). O `√2` que este nível concede
/// tem portanto **30 % de folga**. Gate: `ph2d_field_mesh::tests::a_sculptures_field_never_out_climbs_the_march_step`.
///
/// ⛔ **E a nota que aqui estava — «ninguém mediu» — era falsa:** um gate irmão
/// (`the_sampled_field_marches_like_a_distance`) media desde sempre, **numa esfera e numa banda de
/// três células**. *Uma nota que diz «não medido» quando existe um gate estreito é pior que
/// nenhuma: ela manda medir de novo e esconde o que já se sabe.* O que faltava era a
/// **generalização** — formas com vinco, e a caixa inteira.
///
/// ⚠️ **O limite DEMONSTRÁVEL de uma interpolação trilinear é `√3 ≈ 1,732`**, acima do `√2` que este
/// nível concede: cada componente do gradiente é um quociente de diferenças ≤ `1`, e três somam em
/// quadratura. As medições ficam em `1,09` porque saturar as três ao mesmo tempo exigiria a
/// superfície perpendicular aos três eixos **no mesmo ponto**, e uma distância com sinal não faz
/// isso. *A diferença entre a barra medida e a demonstrável fica escrita em vez de esquecida.*
#[must_use]
pub fn inflation_depth(doc: &FieldDoc) -> u32 {
    // A arena tem os filhos ANTES dos pais, então uma passagem para a frente basta.
    let mut depth = vec![0u32; doc.nodes().len()];
    for (i, node) in doc.nodes().iter().enumerate() {
        let below = match &node.kind {
            NodeKind::Combine { children, .. } => children
                .iter()
                .map(|c| depth.get(c.0 as usize).copied().unwrap_or(0))
                .max()
                .unwrap_or(0),
            NodeKind::Leaf(_) | NodeKind::Sampled { .. } => 0,
        };
        let here = match &node.kind {
            NodeKind::Sampled { .. } => 1,
            // ⭐⭐⭐ **`n` filhos são `n − 1` arredondamentos** — o `combine_trees` dobra da esquerda
            // para a direita, então um nó só já é uma **corrente**. ⛔ Medido: uma união exacta com
            // 3 filhos lê `‖∇f‖ = 1,5411` e com 5 lê `1,9585`, os dois acima do `√2` de um nível — e
            // é essa a forma da **cena 1 do smoke** (três cilindros num nó). *Uma fixtura de dois
            // filhos não vê a corrente que o lowering constrói.*
            NodeKind::Combine { op, children } => match op.blend() {
                Blend::Exact { radius } if radius != 0.0 => {
                    u32::try_from(children.len().saturating_sub(1)).unwrap_or(u32::MAX)
                }
                Blend::Exact { .. } | Blend::Sharp | Blend::Organic { .. } => 0,
            },
            NodeKind::Leaf(_) => 0,
        };
        depth[i] = below + here;
    }
    depth.get(doc.root().0 as usize).copied().unwrap_or(0)
}
