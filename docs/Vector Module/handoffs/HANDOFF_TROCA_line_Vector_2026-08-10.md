# HANDOFF DE TROCA — `line/Vector` (2026-08-10)

> Para o agente que **assume** esta linha. O bloco de abertura da sessão é o
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md);
> este documento é o item 5 da FASE 2 dele — *o estado que o agente anterior deixou*.

---

## §0 — O fato que decide como ler o resto

**Esta linha tem ZERO commits.** Ela foi aberta hoje, o setup rodou, a tarefa nunca veio.
Não existe trabalho em voo, não existe decisão pendente minha, não existe nada a resgatar.

Se você veio procurar código, **pare de procurar** — não há. O que este handoff carrega é
(a) o que o setup já pagou e você não deve refazer, (b) onde o MÓDULO está, e (c) as
armadilhas que esperam a primeira linha de código que você escrever.

---

## §1 — Identidade

| | |
|---|---|
| Branch | `line/Vector` |
| Worktree | `Worktrees/line-Vector/` |
| HEAD | `76788440a` — **idêntico ao `main`** |
| Commits próprios | **0** (`git rev-list --count main..HEAD`) |
| Árvore | limpa |
| Base do fork | o próprio `main` de hoje |

⚠️ **É linha NOVA, não reaberta.** A `line/Vector` anterior morreu na integração de
2026-08-10 (a branch não existia quando esta worktree foi criada). Consequência prática:
**a FASE 1 do bloco de troca (`git rebase main`) é no-op HOJE** — `HEAD == main`. Ela passa
a valer normalmente na próxima jornada, e aí é obrigatória.

---

## §2 — O que o setup já pagou (não refaça)

- **Worktree criada** + **Mergiraf registrado** (`scripts/mergiraf-setup.sh`, 0.17.0, 5 globs,
  1 gramática Rust). É idempotente, mas já está feito.
- ⚠️ **`chattr +C` no `target/` da worktree, ANTES do primeiro build.** O `target/` do
  primário é symlink para `/dev/shm`; **o da worktree não é** — ele nasce no btrfs, com CoW
  e `compress=zstd`, que é o caminho patológico para um diretório de build
  ([memória](../../../project-memory/project_modo_l_speed_hole_worktree_targets_slow_path.md):
  foi assim que uma jornada somou 836 GB de target). Se você apagar `target/` e ele voltar
  a nascer sem o `C`, re-aplique **antes** de compilar; depois de povoado, o atributo não
  alcança o que já está lá.
- **sccache quente:** `cargo check -p ph2d-core` custou **0,62 s** nesta worktree fria.
  ⚠️ *Essa é a métrica de velocidade a vigiar no Modo L — hit rate do sccache e onde o
  target mora, nunca a RAM.* Se o seu primeiro check demorar minutos, o cache está frio ou
  em thrash; não é build "normal de worktree nova".

---

## §3 — Onde o MÓDULO está (a fonte, e o espelho)

A última jornada da `line/Vector` **integrou em 2026-08-10** (48 commits, 100 arquivos):
*o painel autorado fica VIVO, e o sistema de widgets é auditado*.

**A fonte** (leia esta, nesta ordem, se a sua tarefa tocar o que ela mexeu):

1. [`Estudos/HANDOFF_INTEGRACAO_line_Vector_painel_vivo_2026-08-09.md`](../Estudos/HANDOFF_INTEGRACAO_line_Vector_painel_vivo_2026-08-09.md)
   — a última entrega, com as leis (§2) e o aberto (§5).
2. [`Estudos/HANDOFF_INTEGRACAO_line_Vector_MESTRE_2026-08-08.md`](../Estudos/HANDOFF_INTEGRACAO_line_Vector_MESTRE_2026-08-08.md)
   — a jornada da UI/UX (tokens · estados · painel autorado), onze waves.
3. [`Estudos/AUDITORIA_widgets_achados_2026-08-09.md`](../Estudos/AUDITORIA_widgets_achados_2026-08-09.md)
   — os dezoito achados e o mecanismo único deles (*um fato com duas cópias que discordam*).
4. [`Estudos/LEVANTAMENTO_vector_para_a_UI_do_app_2026-08-08.md`](../Estudos/LEVANTAMENTO_vector_para_a_UI_do_app_2026-08-08.md)
   — a medição que justifica as outras (438 widgets, cobertura 67,1%).

⚠️ **O `CLAUDE.md` §5 é ESPELHO, não fonte.** Ele resume as jornadas do módulo e já esteve
errado — a linha do `PROJECT_SCHEMA` sozinha esteve falsa **cinco vezes** naquele arquivo.
Quando o número decide alguma coisa, meça na árvore.

⚠️ **Os handoffs da última jornada vivem em `Estudos/`, não aqui.** A regra
(DIRETRIZ §1.5.9) é `docs/<Módulo>/handoffs/` e foi escrita **hoje** — os anteriores ficaram
onde estavam. **O próximo é aqui.**

### Números MEDIDOS hoje nesta árvore (não copiados)

| Constante | Valor | Onde |
|---|---|---|
| `PROJECT_SCHEMA` | **70** | `shells/desktop/src/project.rs:379` |
| `VEC_SCENE_SCHEMA_VERSION` | **14** | `crates/ph2d-vec-scene/src/lib.rs:434` |
| Registro `ph2d-ecs` | **55** | `crates/ph2d-ecs/src/scene/registry.rs:442` |
| Espelhos (`ph2d-render` · `ph2d-script`) | **56** cada | `*/src/registry.rs:34` |
| Próximo scrollbar id livre | **842** | `widget/scrollbar.rs:221` |

⚠️ **O contador de componentes é TRÊS**, e cada um roda só na suíte da própria crate — é a
família que já ficou vermelho-latente **três vezes** nesta linha. Componente novo = três
edições, não uma.

⚠️ **Um bump de schema se CONTA contra o `main` do dia, nunca se escolhe** — e a colisão
passa **MUDA** quando duas linhas escrevem o mesmo literal (o git não sabe o que o número
significa; o `project.rs` funde limpo e o bump da segunda evapora com a suíte verde). Se a
sua tarefa bumpar, confira **os dois** arquivos: `project.rs` **e** `project_schema_tests.rs`.

---

## §4 — Aberto no módulo, com o preço ao lado

Da última entrega (§5 do handoff do painel vivo):

- ⚠️ **A guarda do popover VAZIO não tem gate, e isso está MEDIDO** — a mutação que a remove
  **sobreviveu** à suíte inteira: um popover sem opções não regista opção nenhuma, logo não
  come cliques. O efeito é **só visual**, e o harness de painel deste repo lê retângulos de
  hit e nunca a cena. Está declarada no `paint.rs` como defesa em camada, não como dívida.
- **Duas molduras autoradas** ⇒ o painel vivo mostra a **primeira**. Escolher pela SELEÇÃO é
  decisão de produto — um desempate que o artista não vê seria pior.
- **Mutar um GRUPO de opções** (esconder uma aba) não existe: um filho de controle de lista
  é um **rótulo**, não um controle.
- **`AuthoredIntent::Value`/`Flag`/`Text`** são drenados e descartados (o store é a
  autoridade). Consumidor futuro entra na mesma ponte.

Anterior, ainda de pé:

- ~~**`align_content`** não é exposto no auto layout — numa moldura `Wrap` com folga o `taffy`
  *distribui* as faixas (medido: a 2ª faixa pousou em **54,5** em vez de 19).~~ ✅ **FECHADO
  pela wave 5b (2026-08-10):** o `align` espelha para o `align_content` nas **quatro** direções,
  não só na grade. O item não era *"falta expor um controlo"* — era **dois contentores a
  responder ao contrário sob o mesmo controlo**, e só a grade o tornou observável. Um controlo
  separado está agora **recusado com motivo** (ele venceria o `align` num wrap de faixa única —
  medido). Ver o estudo §7, *"o que a wave 5b entregou"*.
- A **caixa do gizmo é aproximada** com filho ROTACIONADO *e* pose de escala NÃO-UNIFORME
  (deixa de ser retângulo orientado — é geometria, não descuido).
- ~~⚠️ **O hit-test só recebe o produtor de OFFSET.** Os outros seis produtores de
  `LiveGeometry` não chegam ao pick; a cura geral é o pick ler o mapa **FUNDIDO** que o
  renderer desenhou — **wave própria**, não conserto de passagem.~~ ✅ **FECHADO
  (2026-08-10):** eram **oito** e não seis (a fusão tem nove produtores), e o defeito estava
  MEDIDO em *3 de 3* pontos de uma metade espelhada desenhada e não-clicável. A cura foi a
  que o item previa — o frame guarda a fusão tal como o `dispatch` a consumiu
  (`App::vec_live_drawn`) e o pick lê-a —, e ela **não era arquitetural**: as quatro funções
  de pick já ACEITAVAM o mapa, só recebiam o parcial. ⚠️ O mapa é do frame **ANTERIOR**, e
  isso é a semântica certa: o artista clica no que VÊ. Arch-gate
  `the_pick_reads_the_map_that_was_drawn`, 4 mutações.
- O **caminho do tablet** (a fonte `Pen` é oferecida e **não chega**: a shell não recebe
  pressão de dispositivo — é INPUT de shell, custa uma função, e afeta o Flip igual) · o
  **lasso** · **X/Y numérico do nó** · ~~**editar nós de VÁRIAS formas** (ausência *por
  construção*: `selected_verts` pertence a um `selected` único)~~ ✅ **FECHADO (2026-08-10):**
  `selected_verts` virou `Vec<(VecPathId, usize)>` — o dono entrou no par, e com ele morreram os
  **três** casos especiais que a ausência exigia (a soma que trocava de alvo · o marquee que
  elegia um caminho · o overlay que só acendia o primário, com *"selected path only"* escrito no
  próprio comentário). Medido antes: uma caixa sobre duas formas apanhava **4 de 8** nós.
  ⚠️ **A metade que carrega a wave é o ESPAÇO, não a contagem** — o arrasto em grupo e o Average
  cruzavam frames locais diferentes, e um único `delta_to_local` deforma **em silêncio** com a
  contagem certa. ⚠️ E isto era o **pré-requisito do lasso**, que segue aberto: um laço que varre
  os nós de duas formas não significa nada enquanto a seleção só souber guardar os de uma.
  Detalhe: plano 25 §6, *"W6.4 — a seleção de nós ganha DONO"*. Smoke `PH2D_BUILD_SMOKE=70`.

---

## §5 — As três armadilhas que esperam a sua primeira edição

**1. `ph2d-i18n/src/lib.rs` foi PARTIDO, e a saída dele é uma CADEIA.**
As 186 chaves `panel.vector.*` mudaram-se para o irmão `vector.rs` (701 → 520 LOC) e a
`line/sculpt3d` fez o mesmo com as dela na mesma janela. O `tr` consulta os irmãos **em
cadeia** antes do vazamento:

```rust
vector::tr(k).or_else(|| sculpt3d::tr(k)).unwrap_or_else(|| leak_key(k))
```

⚠️ Uma chave `panel.vector.*` acrescentada ao **`lib.rs`** funde **limpa** contra um arquivo
de onde a tabela saiu — e evapora. **Irmão novo entra nesta cadeia, nunca num segundo
`match`** (um segundo `match` apaga a família inteira do outro painel, que passa a pintar os
próprios identificadores na tela, com a suíte verde).

**2. Dois "não reverta", os dois com o motivo escrito no produto.**
- `PANEL_HEADER_CLOSE_RESERVE` **42** (era 40): ela era 2 px curta na escala de **fábrica**,
  e o botão de fechar de um dos dezasseis painéis **arrastava em vez de fechar**. O pin
  `the_reserve_is_the_pad_plus_the_icon_at_factory_scale` existe para essa reversão custar
  duas edições.
- Os quatro literais de spacing cuja **derivação** foi corrigida (`SECTION_LABEL_TO_CONTROL_PX
  = 4.0` dizia `Xxs`, que vale 2): **os números estão certos, os comentários é que mentiam**.
  Trocar o literal pelo token nomeado encolhe o gap de **toda seção do app**.

**3. Adjacência de NOME, viva e não resolvida: `ph2d-runtime`.**
O R0 do `line/runtime` integrou em 08/08 uma crate-folha de sinais com esse nome; o
[`PLANO_UI_UX_padrao_figma.md`](../Estudos/PLANO_UI_UX_padrao_figma.md) aponta o **mesmo
nome** para o runtime de UI que a W8a vai procurar. **Hoje não há conflito** — nenhuma crate
do Vector menciona a do runtime, e a última jornada **consumiu** a folha em vez de disputar
o nome (o gate `the_event_core_is_a_leaf` segue de pé). O que existe é uma **expectativa**, e
a decisão é do Enio; a recomendação registrada do R0 é **crate irmã**.
⚠️ E o **envelope por SEÇÕES (F1.W0) NÃO existe no `main`**, embora a W8a dependa dele: ele
vivia na `line/runtime` antiga, destruída sem integrar. O commit `37ff53467` sobrevive, mas
**o desenho volta e o diff não** — ele reescrevia o `project.rs`, que desde então andou de
`PROJECT_SCHEMA` 48 para **70**.

---

## §6 — Regras da sessão (ponteiro, não cópia)

As **A–H** do [`MODELO_ABERTURA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md)
valem iguais para você, e **não estão copiadas aqui de propósito** — duas cópias da mesma
regra divergem. As três que esta linha mais paga:

- Todo comando começa com o `cd` da worktree. ⚠️ A cwd do Bash **volta ao primário** entre
  chamadas, e o mesmo path relativo existe nas duas árvores: editar a errada **compila e
  commita sem erro**. Já aconteceu com várias linhas, inclusive mandando metade de um commit
  para o `main`.
- **Você fecha a linha, escreve o handoff (DIRETRIZ §1.5.9) e PARA.** Integrar e fazer ship
  são de um agente dedicado, só por ordem EXPLÍCITA do Enio.
- **DIRETIVA_IMPLEMENTACAO a cada passo** — verde-de-compilação é velocidade; no audit vale
  zero.

Duas notas operacionais herdadas, das que custam uma hora quando se descobre sozinho:

- ⚠️ **Rode a suíte em DEBUG *e* em RELEASE.** Há precedente registrado neste repo de pânico
  que só aparece em debug (`wrapping_sub`) e de kill de relógio que só reprova em debug.
- ⚠️ **`-- --ignored` pede `--test-threads=1` e máquina calma.** Kills de wall-clock dão
  vermelho sob carga sem uma linha de código mudar (medido noutra linha: 11,36 ms sob
  `load 41` contra 5,50 sob `load 0,6`).

---

## §7 — O que falta

**A tarefa.** Ela vem do Enio, na mensagem seguinte ao bloco de troca. Este handoff não a
escolhe, e a §4 acima é *inventário do que está aberto*, **não** uma fila priorizada.
