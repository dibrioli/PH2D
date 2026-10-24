# ADR-0191 — O canal de qualquer propriedade é o catálogo de componentes, e a linha liga-se a ele pelo RÓTULO

- **Status:** Accepted — o produto («o Auto Key grava ABSOLUTAMENTE TUDO», 19/10) é ordem do DONO; o desenho
  técnico é delegado.
- **Data:** 2026-10-24
- **Linha:** `line/UIUX` (onda F27, blocos B–D)
- **Amparo:** [plano do bloco B](../../UI_New_and_Simple/F27_bloco_B_plano.md) (§1 censo, §2 desenho, §4 medição,
  §5 decisões da B2) · [handoff B3+C](../../UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_B3_C.md)

> ⚠️ **Número PROVISÓRIO** (§5.0): contado em 24/10 contra o `main` (`0189`) e todas as worktrees
> (`line/Macros-Automação` tem `0190` por integrar). Quem integrar reconta-o.

## Contexto

Antes da F27 só as famílias com vocabulário à mão (vetor, objecto, Flip) tinham canal: o Inspector dos
componentes (Câmera, Player, Vida, Script…) pintava **108 PORTAS + 19 SEM_COLUNA com zero canais** (censo de
23/10). Um id à mão por campo seriam ~600 constantes e ~68 funis de leitura/escrita. O catálogo
(`ph2d-component-desc`) já descreve cada campo — `field_id` append-only, `kind`, a chave serde, a `View` — e o
registo dá a cada componente um id estável (`stable_type_id`, travado por `stable_type_ids_are_locked_in`).

## Decisão

1. **Uma família genérica, `COMPONENTE = 6`** (`ph2d-ecs/src/canal.rs`). A `FieldKey` leva
   `campo = field_id << 2 | faixa` (faixa 0–3 = x/y/z/w de Vec2/Vec4/Color; `field_id < 16384`, gate) e, em
   `i`/`j`, os 32 bits altos do `stable_type_id` (`canal_componente::id32`; colisão zero sobre o catálogo, gate).
   Grava-se só o estável: `field_id`, faixa, id do componente.
2. **`canal_componente::info` resolve pelo catálogo**: o rótulo é o `label_key`; discreto = Int/Toggle/Enum/Seed;
   `FieldKind::Text` (e o `Named` de texto, o alvo da câmera) é canal discreto pela `TextosDosCanais`, que já
   viajava no documento; `View::Not` (o `hidden` do «Visible») é canal do que se VÊ (`1 = visível`).
   Curve/Gradient/refs não são canal (NaoAnimavel com razão). O id vem da chave — nunca um `blake3` por chamada.
3. **O grupo** (o losango único de x/y, a cor inteira) é o índice DENSO de `(componente, campo)` no catálogo, só
   em sessão, nunca gravado.
4. **A linha do Inspector liga-se ao campo pelo RÓTULO do catálogo, não pelo id do controlo.** Ao PINTAR, a linha
   regista os seus ids sob o rótulo (`key_mark::Rotulo` / `paint_decorator_campo`; o `TextKey` do catálogo é a
   chave); as marcas e o clique resolvem o rótulo no quadro seguinte (`canal_componente::chaves_do_rotulo`).
   Rótulo partilhado (só os `LIMITS_FIELDS` da câmera × scroll) ⇒ as chaves dos dois; o objecto só publica o que
   tem. Sem tabela à mão: o tipo obriga.
5. **O estado do losango lê as ligações POR GRUPO**: `ph2d_timeline::Ligacoes::de(doc, entidades, inst)` constrói
   uma vez por quadro o índice `(objecto, grupo)` e o relógio de cada objecto; `estado(reps, valor)` responde por
   grupo. Os três pintores (Inspector, vetor, camadas do Flip) usam-no.
6. **O dope-sheet mostra UMA linha por grupo** (`linhas.rs`, `GrupoChave`), e o menu do botão direito nessa linha
   (`ContextMenuKind::TimelineTrackGroup`, a tabela do menu de faixa) aplica Delete Track e a extrapolação a
   TODOS os membros, num passo de undo.

## ⛔ Recusas MEDIDAS (o preço)

| alternativa | medição | porquê não |
|---|---|---|
| vocabulário à mão por componente (o caminho do vetor) | ~600 constantes, ~68 funis | cada componente novo pagaria a sua tabela; o catálogo já é a fonte |
| ligar controlo → campo pelo NOME do id (`INSP_*` ↔ `key`/`label_key`, regra mecânica) | **136 / 1975 (6,9 %)** | os ids nasceram antes do catálogo e nomeiam outra coisa |
| GERAR a tabela id → campo do fonte (chamadas com id E rótulo na mesma chamada) | **65 / 188 (35 %)** | a maioria das linhas recebe o rótulo por variável/helper; a tabela gerada envelhece |
| a tabela `COMPONENTES` da câmera, à mão (B1) | substituída na B2 | uma segunda lista é a resposta que envelhece (`a_mascara_da_camera_resolve_no_catalogo`) |
| `estado_do_grupo` a varrer todas as ligações por grupo | **51,5 ms** por quadro com 462 canais (plano §4) | curado por 2 (sem hash, 17,4 ms) e 5 (o índice): **0,23 ms**; nenhum tecto |

Os rótulos do catálogo que o Inspector já pinta: **391 / 438 (89 %)** — a ligação pelo rótulo cobre quase tudo
sem tocar nos ids.

## Consequências

- Componente novo no catálogo ⇒ os seus campos são canais sem código de canal; o rótulo pintado é a ligação.
- O gate `architecture_every_editable_field_is_a_channel` obriga cada PORTA e cada primitivo SEM_COLUNA a
  escolher lado (Canal · NaoAnimavel com razão · Divida); a lista de isenções só diminui.
- **Dívida declarada:** itens de LISTA (Timers, Tweens, StateMachine, CounterWatch, SignalActions, VecAnchors,
  ShakeEmitter, resistências) — a chave endereça campos, não «o item n»; o endereço por item é desenho próprio.
- O `field_id` é append-only também por isto: renumerar um campo muda a chave gravada das animações dele.

## Adendo F27 E (2026-10-24)

Desenho e medições: [F27_E_plano.md](../../UI_New_and_Simple/F27_E_plano.md).

- Família de canais `SCRIPT = 7`, por NOME da prop (`i:j` = índice na `TextosDosCanais`; `campo = tipo << 2 | faixa`).
- Forma de chave nova `top:*` = o payload da variante em que o enum está (`any_variant`, partilhado por script, ledger e canal).
- `Target::Script` (`ph2d-component-desc`): o ficheiro do script é um canal discreto pelo `ScriptId`.
- Opções de enum com dados comparam-se POR VARIANTE (`same_option`): cura de NavAgent «Target» e Factory «Where», que liam `None` com payload ≠ molde.
- `TimelineDoc` v22 → v23 (mesmos bytes; a `retabela` remapeia as chaves SCRIPT).
- **Dívida declarada:** as três dívidas do catálogo (fonte do rótulo do HUD, ficheiro do script, props do script) estão FECHADAS;
  ficam os itens de LISTA.

## Adendo F27 F (2026-10-24)

Desenho e medições: [F27_F_plano.md](../../UI_New_and_Simple/F27_F_plano.md).

- `CampoInfo.especie: Especie` (Continuo · Numero · Ligado · Opcoes · PorIrmao · Catalogo · Camada · Script · Texto) substitui
  `discreto`/`texto`; todo discreto de família declara a espécie (o helper exige-a).
- Porta única da palavra de um valor discreto: `ph2d_ecs::canal_palavra::palavra` (chave `ph2d-i18n` ou nome do artista);
  a coluna do valor e o gráfico da timeline leem-na pelo `TrackValues`.
- Cada opção do catálogo declara a chave do rótulo do cartão (`options!["x" => X @ "panel…"]`, `Opcao { nome, valor, rotulo }`).
- Um canal, um losango: o ficheiro do script leva-o no seletor.
- O corte de script recomeça o script (lei do Godot; os valores próprios ficam).
- Sem formato novo (`TimelineDoc` v23).
