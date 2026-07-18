# Briefing — linha `nesting` (a 2ª mensagem, depois de "linha pronta")

> **Como usar:** o Enio cola primeiro o bloco de [`MODELO_ABERTURA_LINHA.md`](IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md)
> (fonte única, DIRETRIZ §1.5.8). O agente cria a worktree, valida, lê §1.5 + DIRETIVA e reporta
> **"linha pronta"**. ESTE documento é a mensagem seguinte — a tarefa.
>
> **Forke de `main` DEPOIS que `line/anim-fixes` integrar** — ela mexe em `ph2d-timeline` e
> `ph2d-panel-timeline`, que é exatamente onde o nesting vai morar. (A `line/anim-ajustes`, maior,
> já integrou.) O nome `line/nesting` está **livre**: o bloco do `MODELO_ABERTURA_LINHA` roda
> literal, sem o fallback de "a branch já existe".

---

## §0 — A tarefa em uma frase

Desenhar o **nesting** — um objeto que por dentro é uma cena animada inteira e por fora é um
objeto só, com **relógio próprio** —, e o primeiro entregável é um **ADR**, não código.

## §1 — Por que esta é a próxima fase (não re-derive isto)

O [ADR-0115](architecture/decisions/0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md)
já fez a pesquisa e concluiu: **no 2D o idioma de reuso é ANINHAR, não empilhar-e-blendar.**
Animate, Harmony e AE não têm blend de animação; eles têm símbolo / group / precomp. O §5 daquele
plano ([`Timeline/02_plano_composicao_clips.md`](Timeline/02_plano_composicao_clips.md)) nomeia o
nesting como **fora de escopo e PRÓXIMO ADR**, textualmente: *"é o idioma 2D de reuso e nós temos
zero"*.

A composição de clips que acabou de landar (faixas, crossfade por sobreposição, canais esparsos)
cobre *"transição entre dois estados"*. O nesting cobre *"esta peça de animação é uma coisa, e eu
a uso onde quiser"*. É o **multiplicador**: os outros itens da fila somam features, este muda
quanto trabalho uma animação custa.

## §2 — As 3 perguntas que o ADR precisa responder

Estas não são hipóteses: são armadilhas já expostas pelo código atual.

1. **De quem é o relógio, sob duas camadas?**
   O `remapped_time` lê `doc.active_clip()`, e o ADR-0115 **já marcou isso como indefinido sob
   pilha** (*o strip mapeia timeline→clip; o TimeRemap do clip mapeia clip→fonte*). O nesting
   acrescenta uma terceira camada. Responda ANTES de codar: quem responde *"que segundo é agora"*
   para um objeto dentro de um container dentro de um strip.

2. **Um container aninhado é ENTIDADE ECS com filhos, ou é um CLIP que se referencia?**
   Os dois já existem no projeto e puxam pra lados opostos. A hierarquia é única e ECS
   ([ADR-0110](architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md)); os
   clips são do DOCUMENTO e as bindings viajam por `wire_id` (hash do `Name`). Um clip que se
   referencia abre **recursão** — e recursão precisa de detecção de ciclo antes de existir.

3. **O que a UI mostra quando você "entra" num container?**
   AE troca a aba; Animate troca o contexto de edição com uma breadcrumb; Harmony idem. Nós **já
   temos a resposta parcial**: a aba Keys mede o relógio do clip e a aba Arrange o da timeline
   (`ph2d-panel-timeline::tab`). O nesting é a mesma pergunta um nível abaixo — e provavelmente é
   uma breadcrumb, não uma terceira aba.

## §3 — O que JÁ está construído e deve ser reaproveitado (não reinvente)

| Peça | Onde | Por que serve |
|---|---|---|
| **Dois relógios independentes** | `App.playhead` + `App.clip_playhead`, `TimelineState.keys_mode` | O precomp do AE já está meio implementado: a aba Keys roda o clip SOLADO no relógio dele |
| **Fonte ≠ cozido** | [ADR-0121](architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md) (vetor) e o Blend Object do [ADR-0128](architecture/decisions/0128-vector-blend-object-live-virtual-steps-editable-spine.md) | A costura *"o documento guarda o autorado, o mundo consome o derivado"* é EXATAMENTE a forma de um container |
| **Escopos de tempo** | `motion.time_remap` + `Cook::cook_scoped` (Motion Nodes) | Já existe um sistema neste repo que dá **escopo de tempo a uma sub-árvore**. Leia antes de projetar o seu |
| **`PropKind::TimeRemap`** | `ph2d-timeline` | O relógio por-objeto. Um container é a generalização disso pra uma cena |
| **Composição de clips** | `stack.rs` / `stack_eval.rs` / `stack_edit.rs` | Instância de clip numa faixa é meio-caminho de "instância de container" |

## §4 — Entregáveis, em ordem

1. **Wave de pesquisa** — como Rive (artboard aninhado), Animate (symbol + timeline própria),
   Harmony (group), AE (precomp) e Cavalry resolvem as 3 perguntas do §2. ⚠️ **Limite a wave**
   ([[feedback_a_research_fanout_recurses_bound_it]]): dê prioridade, verifique você o fato
   decisivo, mate quando decidir.
2. **ADR** (o último ocupado é **0129**; cheque `docs/architecture/decisions/` antes de escolher — o gate
   `architecture_adr_numbers_are_unique` não tem allowlist). Deve conter as 3 respostas do §2,
   o que fica **fora de escopo nomeado**, e um **kill-criterion medido**.
3. **PARE e reporte ao Enio.** O ADR é aceito por ele antes de virar código — foi assim que o
   0115 evitou implementar o strip-stack do Blender inteiro pra descobrir que era o modelo errado.
4. Só depois: plano em fatias (dados headless → UI, cada fatia com aceitação), no molde do
   `02_plano_composicao_clips.md`.

## §5 — Regras da casa que mais mordem aqui

- **A pesquisa pode DERRUBAR o plano.** É o resultado mais valioso, não um fracasso — o 0115
  descartou o porte do Blender e economizou a implementação inteira.
- **Contrato congelado** (§6 do CLAUDE.md): se o nesting exigir mexer em `NodeOp`/`NodeManifest`
  ou no `Tool`, **PARE e reporte** — exige ADR próprio. O canal de **text param** dos Motion
  Nodes é o precedente de como estender sem tocar o congelado.
- **Postcard é posicional:** todo campo novo em `ClipStrip`/`NamedClip`/`TimelineDoc` é
  **apendado** + bump de `DOC_VERSION` (hoje **7**), e o load REJEITA versão desconhecida.
- **Você fecha, escreve o handoff (§1.5.9) e PARA.** Não integra, não pusha, não roda `ship.sh`.

## §6 — Cenas de smoke que já existem

```
PH2D_STACK_SMOKE=1 cargo run -p ph2d-host-desktop   # composição de clips (L → aba Arrange)
```
Feature nova = **exemplo pronto pra smoke**, auto-montado
([[feedback_ready_to_smoke_example]]). Não peça ao Enio pra montar a cena.
