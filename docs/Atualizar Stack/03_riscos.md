# 03 — O que vai ficar vermelho, e como ler

## §1 — A regra que organiza este documento

> **Quase tudo nesta atualização é pego pelo compilador.** Um renome, um campo que virou `Option`,
> um tipo de retorno diferente — tudo isso **para a build** e aponta a linha. Esse trabalho é grande
> e é seguro.
>
> **O risco mora no complemento:** o que **compila perfeitamente e muda o resultado.** Nenhum portão
> deste repo o vê, e é por isso que dois dos seis blocos terminam no Enio, não num teste verde.

Abaixo, os itens dessa segunda categoria — **todos eles**, com o sintoma.

## §2 — Bloco C: as quatro coisas que mudam a imagem

| # | o que mudou | onde aparece | sintoma se estiver ERRADO |
|---|---|---|---|
| 1 | **`parley` inverteu `Glyph::y`** de Y-up para Y-down (0.8) | todo texto do app | texto de cabeça para baixo, ou deslocado por uma altura de linha |
| 2 | **`vello` corrigiu meio pixel** no desenho de imagem (0.9) | 45 sítios de `draw_image` | nada errado — a posição de **antes** é que estava |
| 3 | **`ImageQuality::High` virou bicúbico** (era bilinear) (0.9) | `ph2d-host/src/filter.rs` mapeia o nosso «Linear» nele | pré-visualização mais suave; halo em borda de alto contraste |
| 4 | **gradiente interpola em alfa não-pré-multiplicado** (0.10) | seletor de cores, `Gradient` do chrome | cor no meio do gradiente diferente onde há transparência |

⚠️ **Os itens 2, 3 e 4 vão deixar goldens vermelhos, e o vermelho É a correção.**
A árvore tem **61** ficheiros com golden e **179** com asserção de pixel.

### Como separar «melhorou» de «quebrou» — o protocolo

1. **Compare com `_pixels_antes/`** (tarefa **C1**), não com a memória.
2. Para cada golden vermelho, pergunte **qual dos quatro** mecanismos o explica. Se nenhum explicar,
   **não regrave o golden** — é defeito nosso.
3. ⛔ **Nunca regrave um lote de goldens de uma vez.** Um regravado em bloco enterra o único que
   estava a dizer a verdade.
4. Um golden regravado precisa de **uma linha no commit dizendo qual dos quatro** o moveu.

⚠️ **Precedente deste repo:** o `Multiply` do renderer não desobedecia à alfa — ele a **invertia** — e
um golden de outra linha mudou de valor, *e a mudança era a cura* (§5, Motion Nodes). *Um golden
vermelho não é sempre uma regressão, e um golden verde não é sempre uma prova.*

## §3 — Bloco E: as cinco coisas que mudam o tato da física

| # | o que mudou | número | sintoma |
|---|---|---|---|
| 1 | **teto de velocidade** novo | 400 unidades/s por omissão | projétil/explosão/queda longa fica mais lenta do que era |
| 2 | **adormecer** por ilha, limiar de linear | 0,4 → **0,05** | corpo que tremia agora só dorme muito mais tarde |
| 3 | **tempo até dormir** | 2,0 s → **0,5 s** | corpo congela cedo demais; empilhamento «gruda» |
| 4 | **distância de predição de contato** | 0,002 → **0,02** (10×) | contato detectado mais longe; peças parecem flutuar |
| 5 | **velocidade corretiva máxima** | 10,0 → **3,0** | penetração resolve mais devagar; pode «afundar» visivelmente |

⚠️ **`pixels_per_meter` do projeto decide se «400 unidades/s» é muito ou pouco.** Meça antes de
assumir que o teto não morde.

⚠️ **O `additional_solver_iterations` mudou de significado** (E9): o mesmo número passa a valer mais.
Todo lugar que o afinou está agora afinado para outra coisa.

### O que o `physics_ecs_c9` prova — e o que ele NÃO prova

✅ **Prova:** que a nossa simulação é **reprodutível** — mesma entrada, mesma saída, no mesmo binário.
Rodar 2× e obter o mesmo hash é o teste local.

❌ **Não prova:** que a física ficou **certa**, nem que ficou **igual à de antes**. O valor **vai
mudar** com o rapier novo, e isso é esperado.

⚠️ **E ele não compara contra um número guardado** — o `spike.yml` compara os **três sistemas
operacionais entre si** (§5, Componentes). Então o risco real é eles **discordarem**, e **só o CI o
mede**. Localmente, o melhor que se consegue é «roda e é estável».

## §4 — Bloco D: os dois itens do `bevy_ecs` que não são renome

**D4 — consultas largas passaram a conflitar com recursos.** Como recursos agora são entidades,
`Query<Entity>` e `Query<()>` podem passar a **iterar sobre entidades-recurso**. O sintoma não é erro:
é um sistema que de repente processa objetos que não existem na cena.
**Cura:** `Without<IsResource>` no filtro. **Achar:** toda `Query<Entity` e `Query<()` na árvore.

**D11 — `Ref<T>` virou `Copy`.** `ref.clone()` agora devolve `Ref<T>`, não o `T` clonado. Compila.
O sintoma é um valor que deixa de ser independente do mundo.

## §5 — Falso vermelho: a família de flakes de carga

⚠️ **Antes de culpar qualquer bloco deste plano, leia o §5.0 do `CLAUDE.md`.**

O sinal de que um ✗ é carga, não código:
- o mesmo teste passa **sozinho**, 3–5 de 3–5;
- o diff não tem uma linha no módulo dele;
- num grupo, **o conjunto de reprovadas MUDA entre corridas do mesmo binário**.

Membros confirmados que este plano vai encostar: `a_round_live_offset_costs_like_the_other_joins`
(**F5**), a família `flip_smooth::…::orcamento`, `the_region_refresh_is_bound_by_the_footprint_not_by_the_mesh`,
`measure_brush_kernel`, e as **duas de alocação** (`apply_from_doc_is_zero_alloc_steady_state`,
`the_trusted_len_collect_allocates_once`).

⚠️ **A lista nunca estará completa** — a forma é o **mecanismo**: todo gate que compara duas medianas
de um recurso partilhado é candidato.

⚠️ **Nenhuma leitura de relógio desta máquina vale acima de `load ~5`.**

## §6 — Vermelhos PRÉ-EXISTENTES: registre-os no T3

Alguns gates já estavam vermelhos antes desta jornada. **Se você não os anotar no `T3`, vai passar
meio dia a caçar um que não é seu.** O `CLAUDE.md` §5 nomeia alguns por módulo — e a lista dele
também envelhece, então **a fonte é a corrida do T3**, não a seção.

## §7 — Recuo, por bloco

| bloco | como voltar | custo |
|---|---|---|
| **T** | `git checkout scripts/`; `chattr -C` | segundos |
| **A** | reverter `rust-toolchain.toml`, `Cargo.toml`, 4 linhas do `spike.yml` | 1 rebuild |
| **B** | `git checkout Cargo.lock` | 1 rebuild |
| **C** | reverter os 6 `Cargo.toml` + o commit de migração | 1 rebuild (grande) |
| **D** | reverter as 8 declarações + o commit | 1 rebuild (grande) |
| **E** | reverter `ph2d-physics/Cargo.toml` + o commit | 1 rebuild |
| **F** | **por tarefa** — é por isso que cada uma é um commit | minutos |

**A rede de todos:** `git tag stack-upgrade-base` (tarefa **T4**).

⚠️ **O recuo do C e do E é caro em relógio, não em risco.** Numa `workstation` com o `target/` frio,
uma build limpa da workspace é a unidade de tempo desta jornada — meça-a no **T3** para saber o que
está a apostar.

## §8 — ⛔ Onde parar e reportar, em vez de decidir

1. Uma tarefa levar a **contrato congelado** (§6 do `CLAUDE.md`).
2. Um teto **mudar de dono** — o `stack-audit --tetos` mostrar uma amarra que o `01` não lista.
3. O `physics_ecs_c9` **não ser estável** entre duas corridas locais no mesmo binário.
4. Um golden vermelho que **nenhum** dos quatro mecanismos do §2 explica.
5. Qualquer coisa querer **reabrir WASM** como caminho de produto (**F15** / [ADR-0075](../architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md)).
