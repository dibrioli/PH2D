# ADR-0146 — A sim do Wet Paint na GPU é um SEGUNDO MODELO, não o mesmo mais rápido

> ⚠️ **Status: PROPOSTA, aguardando decisão do Enio.** Ele autorizou *"os dois
> (A) e (B)"* em 2026-07-29, e a metade (A) — a grade do fluido desacoplada do
> pixel — **shipou no mesmo dia** (doc 28 §5.41). Este ADR existe porque **(A)
> moveu o número que tornava (B) necessária**, e a regra do CLAUDE.md §0 corta
> nos dois sentidos: *quem move o número que tornava algo inalcançável tem de
> reconferir a nota* — e quem move o número que tornava algo **necessário** tem
> de reconferir a necessidade. O desenho está inteiro aqui; a construção não
> começou.
>
> ⚠️ **O número 0146 é PROVISÓRIO** — linhas paralelas reivindicam ADRs na mesma
> janela e o valor se **CONTA** na integração, não se escolhe (3 precedentes no
> repo: 0134, 0131, 0115).

## Contexto

O ADR-0134 trouxe a sim de fluido de volta como port **1:1** do reference JS,
com duas travas que são o contrato do módulo:

* **aritmética f64 / storage f32, semântica JS só via `jsmath`, transcendentais
  só via `libm`** — o *port law*;
* **`tests/fingerprint.rs`**: uma sessão roteirizada com hash **pinado**, de
  modo que toda reescrita de hot loop se prova **byte-idêntica**.

O ADR-0145 (2026-07-29) paralelizou por linhas os três passes row-disjuntos e
deixou escrito o que sobrava: **93 % do passo são os passes seriais POR
SEMÂNTICA**, sem caminho de CPU. A conclusão de então: *"a próxima alavanca é a
GPU, que quebra o port 1:1 e o fingerprint pinado — ADR próprio."*

## O que (A) mudou, medido

A grade do fluido deixou de ser 1:1 com os pixels (doc 28 §5.41). Medido pela
porta do produto, 4096², pincel r=100:

| razão | células vivas | ms/passo | Hz |
|---|---|---|---|
| 1:1 | 1.607.169 | 32,2 | **31** |
| 2:1 | 486.789 | 12,0 | **83** |
| 4:1 | 128.000 | 3,6 | **282** |
| 8:1 | 32.391 | 0,77 | **1293** |

O nominal da SPEC é **40 Hz**. ⇒ **a razão 2 já sai do regime work-limited**, e a
4 deixa a água 7× mais rápida do que o modelo pede. O `km_mixing` — o report do
mesmo dia — passa de 104,4 ms/passo (9,6 Hz) a 8,3 ms a 4:1, **abaixo do kill de
12** do ADR-0134.

⇒ **O que (B) compraria hoje não é "água utilizável": é água 1:1 em 4K.** Isso é
detalhe sub-célula do FLUIDO (não da tinta — a tinta é sempre pintada em pixels
de canvas, com a silhueta do Painter avaliada em espaço de canvas).

## Decisão a tomar

Duas leituras, e a diferença entre elas não é de esforço:

### O que a GPU custa, passe por passe

Os pesos são os **amortizados pela cadência** (§5.40 — `advect` roda todo passo,
`drying`/`project` ÷3, `flow_field` ÷4, `rebuild` ÷2):

| passe | % do passo | mecanismo | port |
|---|---|---|---|
| `advect` | **42,3 %** | **SUBTRAI nos 4 cantos-fonte** de linhas vizinhas | scatter ⇒ precisa de gather reformulado ou atômicos ⇒ **muda os números** |
| `drying_pass` | **25,9 %** | lê a vizinhança 3×3 de `susp` que ele **escreve** (Gauss-Seidel) | Jacobi em 2 buffers ⇒ **outro modelo de relaxação** |
| `build_flow_field` | **24,9 %** | o freio lê o `wet` **VIVO**; o backrun **espalha** em `susp[nb]`/`sett[nb]` | scatter ⇒ **muda os números** |
| `rebuild_active_region` | 4,1 % | 3 de 4 sub-passadas row-disjuntas; a **saia** é prefix-sum serial | scan on-device ✓ (o `ph2d-gpu-cook::scan` existe) |
| `smooth_velocity` | 1,5 % | gather puro | exato ✓ |
| `project` | 1,0 % | **Jacobi** (lê um buffer, escreve outro) | exato ✓ |
| `apply_boundaries` | 0,3 % | por-borda | exato ✓ |

⚠️ **Os três que MUDAM os números somam 93,1 % do passo.** A GPU não é o mesmo
modelo mais rápido: para 93 % do custo ela é um **segundo modelo**, e o
fingerprint do ADR-0134 morre por construção — não por descuido, por
matemática. O que sobra portável-exato é **2,5 %**.

### O bloqueador que não é sobre o solver

O stamp do dab recebe a **silhueta do HOST** por closure — é o que faz o Wet
Paint honrar o pincel do Painter (doc 21/22). Ela chama:

`dab::silhouette_at` → `texture::sample_shape_silhouette` (a **imagem de Shape**
do artista + basis + rotação + tiling) → `texture::remap_shape_value` (o **ramp
LUT**) → `BrushSpec::compose_shape_silhouette` → `BrushSpec::falloff_weight`.

Duas saídas, as duas caras:

* **portar isso para WGSL** = levar o sistema de pincel do Painter ao device
  (Shape image, footprint de flatten/rotate, ramp, os 8 falloffs) — e então
  passam a existir **duas respostas** a *"que forma tem este dab?"*, divergindo
  no único lugar onde ninguém lê um número: uma screenshot. É o defeito que o
  ADR do passe de luz na GPU evitou explicitamente ao manter o **fold** na CPU;
* **round-trip por batch de dabs** (carimbar na CPU, subir a região) — mata parte
  do ganho e reintroduz a sincronização que a wave off-thread removeu.

### E a memória que decide o formato, se for construída

`feedback_two_engines_one_state_is_worse_than_a_slow_engine` — *"assume o LAÇO
inteiro ou nada"*. Um solver metade-GPU com o estado atravessando a fronteira
por passe é o pior dos dois. ⇒ **(B) é all-or-nothing:** os 14 planos passam a
ser **device-resident**, o composite lê do device, e o CPU passa a ser o caminho
de referência (que só precisa **computar a mesma resposta**, CLAUDE.md §0).

## Recomendação

**Não construir (B) agora, e a razão é um número, não uma preferência:** com (A)
a água corre a 83-282 Hz contra um nominal de 40, e o item que (B) resolveria —
*water-limited a 4096² com grade 1:1* — deixou de ser o caminho que o artista
percorre. O preço não encolheu junto: 93 % do passo vira outro modelo, o
fingerprint (o contrato de fidelidade que o ADR-0134 escolheu) morre, e o stamp
exige o pincel do Painter no device ou um round-trip por batch.

**O que RE-ABRE (B), com o gatilho escrito para não depender de memória:**

1. o smoke do Enio reprovar a razão 2-4 **no desenho** (a granulação do banco de
   cerdas, doc 28 §5.41 — e nesse caso a cura barata é a tile em escala de
   canvas, não a GPU);
2. ele querer **1:1 a 4096² sem concessão** como requisito de produto — aí (B) é
   a única via e este ADR já traz o desenho;
3. uma feature nova pedir campos que a CPU não alcança (mais camadas de fluido
   simultâneas, papel 3D com relevo acoplado).

## Se for construída — a ordem, para não haver duas metades vivas

1. **Fase 0 — o ADR aceito** com a decisão explícita sobre o fingerprint: ele é
   **aposentado** (o hash pinado deixa de ser o contrato) e substituído por um
   par de gates de **PROPRIEDADE** (conservação de massa, monotonicidade da
   secagem, a poça parada que não deriva) mais paridade CPU×GPU por **épsilon
   documentado** — o template que o passe de luz na GPU já usa.
2. **Fase 1 — os 2,5 % exatos** (`project`, `smooth_velocity`,
   `apply_boundaries`) + o scan do `rebuild`, com os planos ainda na CPU e
   round-trip: mede o custo da FRONTEIRA antes de qualquer reformulação. Se a
   fronteira já come o ganho, o resto não se paga.
3. **Fase 2 — os planos device-resident** e o composite lendo do device (é aqui
   que a wave deixa de ser reversível).
4. **Fase 3 — os três reformulados** (`advect` gather, `drying` Jacobi,
   `flow_field` sem scatter), um por vez, cada um com o número de **quanto o
   desenho mudou** ao lado.
5. **Fase 4 — o stamp:** a decisão do §"bloqueador" acima, tomada com o custo da
   Fase 1 na mesa.

⚠️ **Nenhuma fase depois da 1 é reversível de graça**, e é por isso que a Fase 1
existe: ela é a medição que decide se as outras valem.

## Consequências de NÃO construir agora

* a água a 4096² **grade 1:1** fica em 31 Hz (o passo de 32,2 ms), abaixo do
  nominal de 40 — e isso está **nomeado**, não escondido: o slider é a resposta;
* o `km_mixing` a grade 1:1 fica em 9,6 Hz. Também nomeado, com a mesma resposta;
* o ADR-0134 e o `tests/fingerprint.rs` **sobrevivem intactos** — e é isso que
  mantém o módulo provável byte a byte a cada reescrita de hot loop, que foi o
  que permitiu as waves do ADR-0145 e da §5.41 serem seguras.

