# 34 — O plano: os smokes da otimização, e depois a CERCA

> **A ordem é a do documento**, e ela não é arbitrária: os smokes vêm primeiro porque é o log deles
> que decide qual frente de otimização abre. A cerca (a mudança de arquitetura, Parte B) vem **depois**,
> e só entra se a Parte A não tiver aberto uma frente que pague mais.
>
> Estado ao escrever: `line/Painter`, 30 commits, árvore limpa, gate de fechamento verde.
> As três waves da jornada do carimbo estão **pendentes de smoke** — validá-las é metade da Parte A.

---

# PARTE A — os smokes

## §1 Antes de qualquer número: a máquina

```
uptime
```

⚠️ **Nenhum número deste log significa coisa alguma com o `load average` acima de ~5.** O registro
tem o caso medido: o MESMO binário, a MESMA fixture, **14,240 → 46,633 ms/passo** sem uma linha de
código mudar, sob `load average 74` (doc 28 §5.49). Um log tirado de máquina carregada não é um log
ruim — ele é um log **sobre outra coisa**, e as conclusões que ele sugere são plausíveis e erradas.

O detector interno é a linha `poca:`: **um dígito de `ns/celula` = máquina sã; três dígitos = o log
não fala sobre o código.**

## §2 A bateria — UMA corrida, quatro meios

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 PH2D_FLUID_PROFILE=1 \
cargo run -p ph2d-host-desktop --release
```

Canvas **4096²**, e ele abre em **Digital** — o `the_smokes_open_the_painter_in_digital` garante isso,
então a mesma tela serve os quatro meios e quem escolhe é o **dropdown Paint Mode**.

⚠️ **Uma corrida só, e isto é metodologia, não conveniência.** O `[frame]` agrega por janela de 120
quadros, então quatro gestos na mesma corrida dão **quatro janelas** com a máquina no mesmo estado —
a comparação entre meios fica **dentro da corrida**, que é a única forma limpa de comparar neste
hardware compartilhado. Quatro corridas separadas carregariam a máquina junto com o resultado.

Em cada meio, **o gesto é o mesmo**: pincel grande, **elipse VIVA** (arraste sem soltar, não solte
entre janelas) por ~3 s, depois pare e deixe passar mais uma janela **assistindo**. O par
*pintando × assistindo* é o que separa TRABALHO de CONTENÇÃO.

| # | meio | o que este gesto julga |
|---|---|---|
| **S1** | **Digital** | o piso: a rota do carimbo sem física por cima |
| **S2** | **Impasto** | o carimbo + o fold do relevo (as 4 fases do re-stamp) |
| **S3** | **Watercolor** | o carimbo + o warp (56% do custo do meio, doc 28 §5.11) |
| **S4** | **Wet Paint** | o carimbo + a água off-thread (as linhas `agua:`/`worker:`/`poca:`) |

**No S4, aperte Enter** ao fim da elipse: o esboço chapado tem de **derreter** num traço molhado que
escorre. É o `wetpaint_commit`, corrigido nesta linha (`18c9b1f47`/`dd129009e`) e não smokado.

### §2.1 A segunda corrida (regressão, não perf)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
env PH2D_MASK_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

O passo 4 é a estrela: pintar atravessando a zona protegida **muitas vezes, LENTO e depois RÁPIDO** —
a fronteira tem de ficar igual nas duas velocidades. É a lei *"a proteção é aplicada UMA vez por
texel"*, e ela cruza o caminho do carimbo em banda que esta jornada mexeu.

## §3 Como se lê o log

Oito linhas por janela. O que cada uma responde:

```
[frame] total=X ms (~N fps) | cpu-encode(raw)=X | acquire(medido)=X | fora-do-encode=X
        | painter-dispatch(cpu)=X (M px publicados em N quadros) | hero-paint=X
```
**A partição do quadro.** `fora-do-encode` é CPU que o quadro paga e que a linha antiga escondia sob
o nome errado. `painter-dispatch` é o lado da shell — ele foi medido em **6,90 ms** e está NOMEADO
como a próxima fronteira depois do carimbo (doc 28 §5.50).

```
[frame]   tool-tick: media X pico X em N/120 frames
        | stamps: media X pico X em N/120 (N entregas, X ms cada)
```
⚠️ **O divisor é a linha inteira.** `stamps: media 105 ms` admite *um re-stamp de forma inteira a
105 ms* **ou** *cinquenta entregas incrementais a 2 ms* — **curas opostas**. É por isso que
`(N entregas, X ms cada)` existe: leia o **por-entrega**, nunca a média sozinha.

```
[frame]   deposito DEVICE: N lotes, N dabs, X M visitas (X ns/visita)
[frame]   deposito CPU:   N em BANDA + M serial(is), N dabs, X M visitas (X ns/visita)
```
**A rota, e o custo por unidade de TRABALHO.** As visitas são a soma das pegadas dos dabs — a régua
certa, porque dabs se sobrepõem ~10× e a bbox mentiria. O `ns/visita` divide o próprio tempo pelo
próprio trabalho, então ele é **comparável entre janelas de tamanhos diferentes**.

```
[frame]   re-stamp por entrega: restore X | relevo X | save X | CARIMBO X (xN entregas)
```
**As quatro fases de um quadro de re-stamp.** O CARIMBO é a que esta jornada moveu. Se ele deixou de
ser o maior, a fronteira **mudou de lugar** e a próxima wave é de outro dono.

```
[frame]   agua: sim media X | composite media X | ESPERA media X (total X ms)
[frame]   worker: busy X% away X% sleep X% | TAXA DA AGUA X Hz (N passos em X s)
[frame]   poca: X M celulas | X ns/celula
```
**Só no S4.** Os três baldes **particionam** a janela do worker e têm de somar **~100%** — se não
somarem, o instrumento está quebrado (aconteceu **três vezes** nesta linha: `sleep 909%`,
`away 161%`, e a taxa a 392 Hz). A `TAXA DA AGUA` tem de ficar perto de **40 Hz e nunca acima**
(o nominal da SPEC), com `sleep > 0`.

## §4 O que cada leitura DECIDE

| se o log disser | significa | o que abre |
|---|---|---|
| `deposito CPU: … + M serial(is)` com **M > 0** no gesto vivo | o piso ainda recusa trabalho que pagaria | re-conferir `SPAWN_EQUIV_VISITS` **nesta máquina** pela sonda |
| `DEVICE: 0 lotes` numa figura **compacta** | o piso de redundância está alto, ou o predicado recusou | ler qual cláusula recusou (Shape/Grain/blend/**cap**) |
| `ns/visita` do DEVICE **muito abaixo** do da CPU, e a CPU levando os lotes | a fronteira não é o gargalo; a **lei do cap** é que barra o device | **a frente 1 do §5** |
| `CARIMBO` **não é mais** a maior das quatro fases | a fronteira mudou de lugar | a wave seguinte é da fase que subiu (`relevo` → o AA do filme) |
| `ns/visita` **constante** entre *pintando* e *assistindo* | o custo é TRABALHO | otimizar; a estrutura não está no caminho |
| `ns/visita` **subindo** de *assistindo* para *pintando* | é **CONTENÇÃO** | nenhuma otimização de kernel ajuda; é agendamento |
| `painter-dispatch` > `CARIMBO` | o gargalo saiu do tool e entrou na **shell** | frente de outro dono, fora desta linha |
| `stamps` alto com **poucas entregas** | um re-stamp de forma inteira | atacar o re-stamp |
| `stamps` alto com **muitas entregas** | custo por dab | atacar o kernel |

## §5 As frentes candidatas, com o número que cada uma já tem

Nenhuma destas se abre por simetria — cada uma espera o log dizer que ela é a maior.

1. **O cap de Accumulate no WGSL.** A rota do DEVICE exige `!accumulate_cap`, porque o kernel não
   transcreve a lei do cap. O device mede **6,6×** a CPU na fronteira medida (S2 do doc 33), e
   `strength < 1` é ajuste **comum** — então hoje o caminho rápido está desligado no caso normal.
   ⚠️ A CPU deixou de ser a rota lenta (a wave de 04/08), então esta frente **custa menos** do que
   custava; o log é quem diz se ainda paga.
2. **O AA do filme do impasto** (~17 ms medidos). A única cura conhecida é **aproximação** ⇒ oráculo
   de APARÊNCIA e ordem do Enio. Não é dívida de engenharia, é decisão de look.
3. **Os quatro sítios fora da porta** — `stamp_color_cache`, `compositor::compose`,
   `selection_overlay`, `warp/transform_float` ainda chamam `available_parallelism()` direto. Eles são
   por-OPERAÇÃO (não por-dab-por-banda), então o syscall ali é ruído — mas carregam **o mesmo cliff de
   contagem constante** que a wave de 04/08 curou, e **nenhum foi medido**.
4. **`painter-dispatch` a 6,90 ms** — a shell, não o tool. Nomeada, não medida por fase.

---

# PARTE B — a cerca (só depois da Parte A)

## §6 O achado, medido

O módulo tem **~180 mil linhas** em seis crates; `ph2d-tool-painter` sozinho tem 255 arquivos e
109.680 linhas (60,0k de produção). `tool/paint.rs` tem **222 linhas e declara 116 módulos** — e
**48 dessas linhas trazem *"split from … (LOC cap)"* escrito ao lado**. `PainterTool` tem **877
métodos em 103 blocos `impl` espalhados por 100 arquivos**.

⚠️ **E o diagnóstico óbvio ("god-object, decomponha") NÃO se sustenta na medição.** Os campos mais
tocados de `PaintState` são `self.paint.sculpt` (162 acessos), `.deform` (167), `.wetpaint` (97),
`.relief` (89), `.warp` (74) — **a decomposição por subsistema JÁ ACONTECEU no estado**, e 13
subsistemas já são árvores de módulo de verdade (`sculpt/`, `warp/`, `wetpaint/`, `media/`…).

O que não aconteceu foi o **cercamento**:

- **162 dos 164 campos de `PaintState` são `pub(super)`** — visíveis aos 116 módulos.
- **93 arquivos continuam irmãos planos por convenção de NOME** (`impasto_ceiling.rs` ao lado de
  `impasto.rs`) em vez de `impasto/ceiling.rs`. Enquanto o nome é convenção e não árvore,
  `pub(in …)` **não é exprimível**.

**O módulo está a ~70% de uma boa decomposição e parou exatamente onde o compilador começaria a
ajudar.** Um arquivo que é `impl PainterTool { … }` não é um módulo: é uma fatia da tabela de métodos
de um tipo só. O teto de LOC forçou *arquivos*, nunca *fronteiras*.

**O que isso custa, medido:** `PARALLEL_MIN_AREA` existia em **seis cópias**, uma delas um literal
re-declarado dentro de um bloco. Toda wave do registro termina com *"porta única"* como **conquista** —
ela é conquista porque nada estrutural a produz. O modo de falha recorrente deste módulo (*"duas
portas para a mesma pergunta divergem"*) é hoje **disciplina**, quando poderia ser **erro de
compilação**.

⚠️ **O que isso NÃO custa: otimização.** Nenhum ganho do último mês foi bloqueado pela estrutura — a
água 52→11 ms saiu de um solver independente de ordem, a secagem de trocar `libm` por tabela, o fold
e o `advect` de rayon, o composite de fatoração row-invariant, a contagem de bandas de uma fórmula.
**Todos vieram de MEDIR pela porta do produto.** É por isso que a Parte B vem **depois**.

## §7 As waves

### W1 — o impasto como piloto (a wave que decide as outras)

`impasto*.rs` são **13 irmãos planos** e o estado deles já está agregado em `self.paint.relief`.

1. `impasto_ceiling.rs` → `impasto/ceiling.rs` (e os outros 12), com `impasto.rs` → `impasto/mod.rs`.
   ⚠️ **Nenhuma linha de lógica muda**, e nenhum caminho de chamada muda se o `mod.rs` re-exportar —
   é o precedente do `paint_text.rs` e do `tool_rail/tests.rs`.
2. Estreitar os campos do subsistema de `pub(super)` para `pub(in crate::tool::paint::impasto)`.
3. **O compilador então LISTA toda travessia de fronteira.** Cada uma é uma decisão consciente: vira
   porta pública (com nome) ou some.

**O critério de continuar:** se a lista sair **curta** (dezenas), o padrão vale e W2..Wn seguem. Se
sair **longa** (centenas), o subsistema não é o eixo certo de corte e a Parte B **para aqui** — o que
teria custado uma wave em vez de meses.

### W2..Wn — os outros, um por wave, na ordem do retorno

| ordem | subsistema | irmãos planos | por que nesta posição |
|---|---|---:|---|
| 2 | `sculpt` | 9 | já tem `sculpt/`; a árvore está meio-feita |
| 3 | `watercolor` | 16 | 3 já são árvores (`_field`, `_render`, `_settings`) |
| 4 | `selection` | 12 | isolado do resto; poucas travessias esperadas |
| 5 | `curve` / `line` | 11 / 6 | 4 já são árvores; editores de shape |
| 6 | `stamp` | 9 | ⚠️ **por último**: é o caminho quente, e ele é o mais atravessado |

### W-Rota — nomear a rota do carimbo (uma tarde, e ajuda a Parte A)

`stamp_dabs` é hoje uma **cadeia de saídas antecipadas** (`if watercolor_render_active() { … return }`,
depois impasto, depois as quatro rotas). Ele passa a **publicar qual rota correu**, em vez de sair
pelo primeiro `return` que casar.

⚠️ É o único item da Parte B que ajuda otimização **diretamente**: hoje eu atribuo custo por
**ablação de knob de UI** porque não há rótulo para atribuir. Se a Parte A abrir uma frente de
atribuição, esta wave sobe para o topo.

## §8 O que NÃO fazer

- ⛔ **A refatoração abrangente.** São **2.400 sítios** de acesso direto ao estado sobre 60 mil linhas
  de produção, apostados contra um ganho que a medição diz não existir. A cerca compra a mesma
  propriedade — *o compilador impede a segunda porta* — em incrementos que param a qualquer momento
  sem deixar a árvore pela metade.
- ⛔ **Unificar as 22 `stamp_dabs_*` sob um trait.** Elas parecem duplicação e não são: são quatro
  meios × oito modos, e a física diverge de verdade (19 arquivos de watercolor, 13 de impasto, 11 de
  sculpt). O trait sairia com um método e oito flags — [[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]
  pelo avesso.
- ⛔ **Mexer no contrato congelado.** Ele não bloqueia nada disto (`CanvasPaintTool=1` cerca o lado de
  FORA, e a cerca proposta é interior) — e é justamente por isso que não há razão para tocá-lo.

## §9 O critério de parada

A rede é de **1.743 gates** nas quatro crates, com fingerprints byte-a-byte no solver da água e
paridade CPU×GPU no impasto. Ela prende **comportamento**, não estrutura — ou seja, torna a mudança
**verificável**, nunca barata.

⇒ **Toda wave da Parte B fecha com a suíte inteira verde em debug E release, e byte-identidade onde
há fingerprint.** Uma wave que precise mexer num fingerprint **não é uma wave de cerca** — ela mudou
comportamento, e aí parou de ser refatoração.
