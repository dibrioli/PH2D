# ADR-0141 — Motion path: a posição é UM canal com trajetória, e eixos separados são um MODO

**Status:** proposto — aguardando aceite do Enio (nada é construído antes)
**Data:** 2026-07-23 · **Linha:** `line/anim`
**Pesquisa:** [`docs/Timeline/05_pesquisa_motion_path.md`](../../Timeline/05_pesquisa_motion_path.md)
**Plano:** [`docs/Timeline/06_plano_motion_path.md`](../../Timeline/06_plano_motion_path.md)

⚠️ **O número é PROVISÓRIO.** 0141 é o próximo livre nesta árvore e nenhuma linha viva
(`line/FLIP`, `line/physics`) toca `decisions/` — mas quem chegar ao `main` primeiro fica com o
número (gate `architecture_adr_numbers_are_unique`; já aconteceu 3 vezes no repo).

---

## Contexto

`PropKind::TranslationX` e `TranslationY` são **duas tracks escalares independentes**. Uma curva no
espaço *emerge* delas, mas não é **autorável**, não é **visível** e não é **agarrável**: fazer um
arco é brigar com dois gráficos até o canvas parecer certo. Arcos são um dos 12 princípios da
animação, e a tabela de pesquisa da Parte A do plano geral marca *interp espacial (motion path)*
como o último item `backlog` da categoria que ela mesma chama de **coração amado**.

E o módulo já construiu a **sombra** disto sem perceber: o
[`rove.rs`](../../../crates/ph2d-anim/src/rove.rs) diz de si mesmo, na linha 6, que roving é *"o
modelo espacial do AE aplicado a uma track escalar"*. No AE, *Rove Across Time* **só existe para
propriedades espaciais** — portamos o parceiro júnior e deixamos o sênior de fora.

---

## A pesquisa fecha a pergunta antes de o desenho começar

Dois produtos, independentemente, dizem que **os dois modos são mutuamente exclusivos e a escolha é
do OBJETO** (§1 da pesquisa):

- **After Effects:** *"Separate Dimensions **precludes** having Spatial Keyframes"* — separar X/Y
  **remove** as alças de bézier da trajetória.
- **Harmony:** o peg tem *Position mode* **Separate** | **3D Path**; no 3D Path os eixos vivem numa
  **única função** com uma **única função de velocidade**, e o Separate é o preferido para rigging.

⚠️ E o contraste mais útil é o **Blender**, cujo modelo de dados é o mais parecido com o nosso (uma
F-curve escalar por canal): ele **não** enxertou trajetória autorável nesse modelo — ofereceu
*visualização* read-only e mandou quem quer o caminho usar um objeto `Curve` + constraint.

**Logo: não existe "adicionar motion path às tracks que temos".** Uma tangente espacial é um objeto
**2D**; duas curvas escalares independentes não têm onde guardá-la, e editar a curva de X sozinha
moveria uma trajetória que o artista não autorou.

---

## Decisão

### 1. O MODO é o `PropKind` — não um flag novo

Um binding para `TranslationX`/`TranslationY` **é** o modo Separate. Um binding para o
**`PropKind::Position` novo** (append, discriminante **8**) **é** o modo Path.

Não há campo `mode` em lugar nenhum: **o modo é uma propriedade do binding que já existe**, e por
isso não há como o flag e a realidade discordarem — a classe de bug que este módulo catalogou meia
dúzia de vezes. Converter entre modos é trocar o binding, um gesto explícito e nomeado (§5).

⚠️ **O modo de hoje NÃO é um erro a corrigir.** É o que a Harmony recomenda para cut-out/rigging, e
segue sendo o default. Documento sem binding `Position` é **byte-idêntico** ao de hoje.

### 2. A trajetória é GEOMETRIA; o tempo continua sendo a track escalar que já existe

Esta é a espinha do ADR, e é o que torna a feature barata:

> Em modo Path a track escalar **deixa de medir "x" e passa a medir "quanto do caminho"** — o
> comprimento de arco acumulado. A geometria (âncoras + tangentes espaciais) vive ao lado; a
> amostragem é `ponto = caminho.em_arco(track.sample(t))`.

O que isso entrega **sem uma linha nova**:

| Peça que já existe | O que ela vira em modo Path |
|---|---|
| Graph editor + weighted tangents + presets | a curva de **timing** ao longo do caminho |
| **Speed graph** (W5) | a `Position: Velocity` da Harmony, literalmente — a derivada de "distância percorrida" **é** a velocidade |
| **Roving** | o *Rove Across Time* do AE, no significado do produto de origem: velocidade constante ao longo da trajetória |
| Time remap, clips, containers, stack, undo, save | inalterados — continua sendo uma track |
| `ph2d-anim` | **intocada**: nenhum tipo de track 2D nasce |

⚠️ **Consequência que TEM de ser uma porta só:** mover uma âncora no canvas muda o comprimento de
arco de todas as âncoras seguintes, logo **reescreve os valores da track**. Geometria e valores são
atualizados na MESMA operação ou passam a discordar. O tempo das keys **não** muda — o mesmo trecho
percorrido num caminho mais longo é o objeto indo **mais rápido**, que é exatamente o que o AE faz.

### 3. As âncoras SÃO as keys

Uma key = **um ponto no caminho + duas tangentes espaciais + um tempo**. Não são duas listas que
podem divergir em contagem ou ordem; é uma lista com dois lados. Apagar uma key apaga a âncora, e
vice-versa. Precedente da casa: `TrackData.roving`, um vec **paralelo e persistente** dentro do
Track.

`DOC_VERSION` **11 → 12** (campo apendado no binding; postcard é posicional ⇒ **quebra dura**, v11
recusado no load — a política que este documento segue desde o ADR-0133). `PROJECT_SCHEMA`
**intocado**: o `TimelineDoc` viaja como blob e carrega a própria versão.

### 4. A tangente espacial default é **Auto Bezier**

É o default do AE, e é o certo: a trajetória nasce **suave** e o artista afia onde quiser, em vez de
nascer poligonal e exigir trabalho para parecer animação. Uma âncora com tangentes colapsadas é uma
quina — nenhum caso especial.

### 5. Converter entre modos é EXPLÍCITO, e diz o que perde

`Separate → Path` e `Path → Separate` são gestos nomeados. A conversão **mede e reporta**: indo para
Separate, as tangentes espaciais **são descartadas** (não há onde guardá-las — é o que o AE faz), e
o gesto diz isso antes, não depois. Nunca automático, nunca silencioso.

### 6. Auto-orient: opt-in, e ele **RECUSA** em vez de vencer em silêncio

`Auto-Orient Along Path` é o acompanhamento canônico. Duas regras, e as duas vêm de falha
publicada:

- ⚠️ **Se a entidade tem track de `Rotation`, o auto-orient é RECUSADO e diz por quê** — dois
  autores de um fato, com o de trás vencendo calado, é a falha mais repetida deste repo
  ([[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]). O artista escolhe: apaga a
  track, ou não usa auto-orient.
- ⚠️ **Na velocidade ZERO a tangente é indefinida** — o bug publicado do próprio AE (*"auto-orient
  flips when stopping motion"*). Segurar o **último ângulo válido** é a resposta, e ela é gateada.

---

## O que NÃO fazemos, e por quê

- **Z / 3D.** O app é 2D. O terceiro eixo da Harmony é dela.
- **Motion path para vetor / painter / escala.** O canal é **posição**. Um "path de escala" não é
  uma coisa que exista em produto nenhum pesquisado.
- **Trajetória como ASSET compartilhado** (dois objetos no mesmo caminho). É o *Follow Path* do
  Blender/Moho, e é outra feature — precisa de dono, de instância e de offset. **Gatilho que a
  acorda:** pedido real de artista, ou o motion path virar o caminho de uma linha de partículas.
- **Visualização read-only do modo Separate** (o *Motion Paths* do Blender). Seria uma segunda
  resposta para "onde este objeto passa" que ninguém pode agarrar; o §0.0 da casa prefere a resposta
  autorável ou nenhuma.

---

## A bifurcação (recomendo, mas é sua)

**Auto-orient entra nesta wave ou na seguinte?**

Recomendo **dentro**: sem ele o caminho fica meio construído (o peixe atravessa a tela de lado), e
ele é a única parte que encosta na track de `Rotation` — descobrir isso depois, com o modelo já
congelado, é pior. Custo: uma fatia, e é a fatia com a decisão de UX (a recusa).

Se preferir fora, o ADR fica idêntico menos a §6, e a fatia 5 do plano sai.

---

## Conjunto de aceitação (concreto e CONGELADO — DIRETIVA §5)

1. `a_position_binding_samples_the_path_not_two_axes` — um caminho com 3 âncoras devolve pontos
   **fora** da reta entre a 1ª e a 3ª (o oráculo é a distância à corda, não `assert_ne!`).
2. `separate_mode_is_byte_identical` — documento sem binding `Position` amostra **ao bit** como
   antes (fingerprint).
3. `moving_an_anchor_rewrites_the_arclengths_in_one_operation` — mutação: atualizar só a geometria
   deixa o objeto fora do caminho.
4. `the_speed_graph_of_a_path_is_the_velocity_along_it` — a derivada da track em modo Path bate a
   velocidade medida por diferenças finitas do ponto amostrado.
5. `roving_gives_constant_speed_along_the_path` — o *Rove Across Time* do AE, agora literal.
6. `the_schema_is_twelve_and_a_v11_blob_is_refused`.
7. `converting_to_separate_reports_the_tangents_it_drops` — a perda é **dita**, não descoberta.
8. `auto_orient_is_refused_when_a_rotation_track_exists` + `auto_orient_holds_the_last_angle_at_zero_speed`.
9. **Seam que CLICA**: arrastar uma alça espacial no canvas muda a trajetória e a cena responde
   ([[feedback_widget_is_done_when_a_test_clicks_it]]).
10. **Perf**: o gate do §Kill.

---

## Kill-criterion (declarado ANTES do build; o baseline é MEDIDO na Fatia 0)

A amostragem em modo Path é uma **inversa de comprimento de arco** (dado `s`, ache o ponto) — no
kurbo, iterativa. É o único custo novo, e é por-binding-por-frame.

**A barra:** o `apply_from_doc` de N entidades em modo Path não pode custar **mais que 2×** o mesmo
N em modo Separate, e o custo por entidade tem de ser **plano em N** (é por-binding; se crescer com
N, algum passe começou a percorrer a lista inteira).

⚠️ **A barra declarada pode falhar, e nesse caso ela é substituída por uma LEI medida, não
afrouxada** — foi o que aconteceu no ADR-0133 (*"< 2×" falhou, virou "dobrar a profundidade não
mais que dobra o sobrecusto"*). A Fatia 0 mede o baseline **antes** de qualquer feature, e o número
que ela produzir é o que fica escrito, com a tabela ao lado (§0.0 da casa).

**Saída se estourar:** tabela de comprimento de arco pré-computada por caminho, invalidada na edição
de geometria — o desconto tem nome e não é inventado na hora.

---

### ⚠️ A barra declarada FALHOU, e a substituta é medida (Fatia 0, 2026-07-23)

Harness: [`crates/ph2d-timeline/tests/measure_motion_path.rs`](../../../crates/ph2d-timeline/tests/measure_motion_path.rs)
(release, custo de UMA amostra em ns).

```text
  baseline: 1 Track::sample           =   5.7 ns
            1 entidade Separate (X+Y) =  11.5 ns
            a BARRA declarada (2x)    =  22.9 ns por entidade Path

  âncoras │  bisseção │    Newton │  LUT K=16 │  err New │  err LUT │ it.New
  ────────┼───────────┼───────────┼───────────┼──────────┼──────────┼───────
        2 │ 1623.4 ns │  203.8 ns │    7.2 ns │ 1.74e-10 │  1.76e-1 │   4.36
        8 │ 1708.1 ns │  184.5 ns │    9.7 ns │ 1.79e-10 │  1.41e-1 │   3.63
       32 │ 1773.4 ns │  184.3 ns │   12.4 ns │ 1.62e-10 │  1.36e-1 │   3.50
      128 │ 1767.7 ns │  188.6 ns │   15.6 ns │ 1.64e-10 │  1.26e-1 │   3.47
```

**Três leituras, e a terceira reescreve a barra:**

1. **O custo é PLANO nas âncoras** nos três métodos — o prefixo somado + busca binária fazem só
   **um** segmento ser invertido. A estrutura certa já existia (`ArcPath` da linha Vector, com o
   mesmo raciocínio escrito: *"construa uma vez e consulte n vezes"*).
2. **A inversa que SHIPA hoje custa 1700 ns** — 75× a barra declarada. Ela é bisseção de 40
   iterações, e cada iteração chama `arclen_to` (Gauss-Legendre de 16 nós = 32 avaliações de
   `|B'|`): **~1300 `sqrt` por amostra**.
3. ⚠️ **A barra declarada era o INSTRUMENTO ERRADO.** Ela mede contra `Track::sample`, que custa
   **5,7 ns** — quase nada. Uma razão contra quase-nada reprova qualquer algoritmo real, e não diz
   **de que recurso** o limite é (§0.0). O recurso é o **frame**:

   | método | 100 entidades Path | % de um frame de 60 Hz |
   |---|---|---|
   | bisseção (hoje) | 170 µs | **1,02 %** |
   | **Newton** | **19 µs** | **0,114 %** |
   | LUT K=16 | 1,2 µs | 0,007 % |

**Decisão: Newton substitui a bisseção.** `ds/dt = |B'(t)|` — a derivada da função que se está a
inverter está disponível de graça, que é exatamente a condição em que Newton bate bisseção:
**9× mais barato, 3,5 iterações, e sem aproximação a defender** (1,7e-10 num caminho de 823
unidades). ⚠️ O ganho é 9× e não os 12× de uma primeira medição porque a tolerância ficou em
**1e-12** — o que a bisseção de 40 iterações entregava —, e apertá-la custa ~1 iteração: quem já
dependia da precisão antiga não a perde. O ADR não compra a LUT: ela é 10× mais barata ainda, mas erra **0,13
unidades** — e uma aproximação que se paga em precisão sem que o orçamento a exija é um modo de
falha comprado sem necessidade.

**A LEI que substitui a barra:** *amostrar 100 entidades em modo Path custa **≤ 0,2 %** de um frame
de 60 Hz, e o custo por entidade é **plano no número de âncoras***. As duas metades importam: a
primeira é o recurso, a segunda é a estrutura (se o custo passar a crescer com as âncoras, alguém
trocou a busca binária por uma varredura).

⚠️ **A LUT fica documentada como a saída, com o número dela já medido** — acorda se uma cena real
precisar de 1000+ entidades em Path *e* o perfil mostrar esta amostragem no topo.

### A lei, VERIFICADA no produto (Fatia 2, 2026-07-23)

Não na extrapolação de um micro-benchmark: no `apply_from_doc` real, com 100 entidades, cada uma
com um binding Position de 64 âncoras e uma key por âncora.

| | medido | a lei |
|---|---|---|
| frame a 100 entidades | **18,77 µs** = **0,113 %** | ≤ 0,2 % |
| custo × âncoras (4 → 512, ponta a ponta) | **1,04×** | plano |
| custo × âncoras (8 → 8192, `at` sozinha) | **1,04×** | plano |

A previsão desta seção (19 µs) errou por menos de 1 %.

⚠️ **A segunda metade da lei precisou de DOIS gates, e o segundo nasceu de uma mutação que
sobreviveu ao primeiro.** Trocar a busca binária de segmento por uma varredura sobre 512 âncoras
move a razão de ponta a ponta para **1,77×** — abaixo de qualquer barra sã — porque `Track::sample`
e a inversa de Newton dominam o frame e **diluem** o defeito. O gate que pega mede `MotionPath::at`
isolada, com contraste de 1024×: **16,45×** com a varredura. Uma lei com duas metades quer um gate
por metade ([[feedback_layered_defenses_need_per_layer_gates]]).

E a razão de ponta a ponta carrega **controle positivo** (`MotionPath::project`, honestamente
`O(âncoras)`, mesmo cronômetro e mesma fixture: **122×**), porque uma razão de 1,0 sobre um
cronômetro cego também dá 1,0.

⚠️ **Consequência cross-linha:** `inv_arclen` é **compartilhada** — Trim, Pattern Along Path, Zig
Zag e texto-em-caminho (linha Vector) chamam a mesma função. Trocar bisseção por Newton **muda os
bits** que ela devolve (1,3e-7). A Fatia 1 roda a suíte da `ph2d-vec-scene` e afina a tolerância
até os gates deles ficarem verdes — a melhoria é para os dois lados, mas quem a faz responde por
ela.

---

## Consequências

- **Positivas:** o coração amado fecha; roving e o speed graph passam a significar o que significam
  no produto de origem; `ph2d-anim` não é tocada; a geometria reusa kurbo e o maquinário de arco que
  a `line/Vector` já integrou.
- **Custo:** dois modos coexistem para sempre (é o que AE e Harmony shipam); a conversão é um gesto
  a manter; `DOC_VERSION` quebra dura.
- **Risco nomeado:** a §2 (geometria e valores numa porta só) é onde este ADR pode nascer com o bug
  que ele mesmo descreve — e por isso o gate 3 existe com mutação.

---

## Alternativas consideradas

| Alternativa | Por que não |
|---|---|
| **Enxertar tangente espacial nas duas tracks escalares** | Não é exprimível (a tangente é 2D); AE diz que *"separate dimensions precludes spatial keyframes"*, e o Blender — mesmo modelo que o nosso — desistiu e ofereceu só visualização |
| **Track 2D nova em `ph2d-anim`** | Duplicaria graph editor, weighted tangents, roving e speed graph para o novo tipo; a §2 entrega tudo isso de graça mantendo a track escalar |
| **Path como objeto do Vector, ligado por constraint** (Blender/Moho) | É a feature de *Follow Path*, com dono e ciclo de vida próprios; não é o motion path do AE, e não fecha a linha da tabela de pesquisa |
| **Substituir `TranslationX/Y` por `Position`** | Quebra todo documento salvo e deleta o modo que a Harmony recomenda para rigging |
