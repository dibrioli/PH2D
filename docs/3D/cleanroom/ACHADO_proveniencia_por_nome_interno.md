# ⛔ ACHADO — a proveniência do repo inteiro está escrita em NOME INTERNO do alvo

> Encontrado em 2026-08-24 pelo papel **E**, com `scripts/cleanroom-sweep.sh`, ao varrer os
> artefatos da triagem do quad remesh. ⚠️ **O achado NÃO é do quad remesh** — ele atravessa
> Painter, Sculpt3D, Flip e Components, e é **anterior** a esta linha e à própria skill.
>
> ⛔ **Este documento DESCREVE e nunca REPRODUZ** (SKILL_Cleanroom §6.1). Todos os endereços
> abaixo são **nossos**. Nenhum identificador do alvo aparece aqui — é por isso que ele passa
> o próprio sweep.

---

## §1 — O número, e o que ele NÃO diz

| medida | valor | como |
|---|---|---|
| ⭐ **arquivos de fonte C/C++ de alvo rastreados no repo** | **ZERO** | `git ls-files` |
| linhas que citam um arquivo de fonte **interno** de um alvo (`nome.cc`, muitas com `:linha`) | **~460**, em **133** arquivos | detector de citação sobre `git ls-files -z` |
| ⛔ **destas, as que carregam TRANSCRIÇÃO junto** (sintaxe C, ou símbolo interno de casa alheia) | **25**, em **20** arquivos | detector apertado, calibrado à mão sobre 14 amostras |
| por família do alvo | **~382 + ~85 não classificadas ⇒ quase tudo Blender (GPL-2.0-or-later)** · **13** da família do quad remesh (GPL-3.0) · **6** de alvos **permissivos** (lícito) | classificador por nome |
| ⚠️ **onde dói mais** | **2** em `project-memory/` | — |
| distribuição | **214** em `crates/` · **192** em `docs/` · **18** em `shells/` | — |

⭐⭐ **A propriedade que segurava tudo, segurou:** **nenhum fonte de alvo está na árvore.**
A parede do §0 — *quem escreve o produto nunca teve a expressão original no contexto* — não
foi furada pelo canal que importa. O que vazou foi a **FORMA da nota de proveniência**.

⚠️ **E a maior parte das ~460 é matemática.** O §4.1.2 da skill permite **toda** a
matemática, sem limite de profundidade; o §4.1.3 permite **constante e default como FATO,
com proveniência**. ⇒ *o fato é lícito; o endereço interno como forma de o citar é que não é.*

---

## §2 — As duas classes, e só a segunda é violação de expressão

### Classe A — citação de endereço interno (~435 linhas) · ⚠️ higiene do §4.2

A nota diz um **fato funcional** (uma fórmula, um default, uma ordem de operações) e cita a
proveniência como *arquivo interno do alvo, linha N*.

- ⛔ O §4.2 proíbe **nome interno** (função, variável, **arquivo**) em artefato.
- ⭐ Mas o **conteúdo** dessas notas é exactamente o que o §4.1.2/§4.1.3 **manda** guardar.
- ⇒ **A cura preserva o fato e troca o endereço**: *«observado na referência, no verbo de
  camada, no passo de frente-de-face»* em vez do nome do arquivo. Perde-se re-checagem
  automática; ganha-se um artefato que passa o sweep.
- ⚠️ **Não é uma edição em massa cega.** O endereço interno é hoje o que permite a um agente
  futuro **reconferir o fato contra o oráculo**. Trocar 435 notas custa essa rastreabilidade,
  e a troca tem de ser **medida antes de ordenada**.

### Classe B — transcrição (25 linhas) · ⛔ violação real do §4.2

A mesma linha carrega, além da citação, **texto de código**: um predicado com o índice e o
nome de variável do alvo, uma **assinatura com argumentos por omissão**, um `if` com uma
flag de enum da casa alheia, uma chamada com os nomes de parâmetro dela.

- ⛔ *«Texto de código, trechos, diffs — nem uma linha»* (§4.2, 1º item).
- ⚠️ Pela régua do **§6.2** cada uma é **«assinatura/nome isolado» ⇒ RELANCE**, não
  «substancial»: nenhuma é corpo de função nem bloco de ~10+ linhas. ⇒ **nada é queimado, e
  nenhuma janela precisa de ser refeita.**
- ⇒ **Curam-se uma a uma**, re-expressas em vocabulário do domínio. São **25**.

### Os 20 endereços NOSSOS da Classe B

`crates/ph2d-flip/src/stroke.rs:15` · `crates/ph2d-painter-brush/src/spec.rs:61` ·
`crates/ph2d-sculpt3d/src/brush_magnitudes.rs:171,272` ·
`crates/ph2d-sculpt3d/src/brush_scale.rs:63` ·
`crates/ph2d-sculpt3d/src/brush_verb.rs:177,203,214,252,295` ·
`crates/ph2d-sculpt3d/src/ref_profiles.rs:305` ·
`crates/ph2d-sculpt3d/src/stroke_dab_core.rs:302` ·
`crates/ph2d-sculpt3d/tests/measure_layer_front_face.rs:31` ·
`docs/3D/20_divergencias_tools.md:26` · `docs/3D/21_plano_modos_e_ferramentas.md:437` ·
`docs/3D/handoffs/HANDOFF_CONTINUACAO_line_sculpt3d_2026-08-18.md:235` ·
`docs/3D/handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_MESTRE_2026-08-17.md:189` ·
`docs/Components/pesquisa/instancias_2026-08-21/enderecamento_override_aninhado.md:45` ·
`docs/Components/pesquisa/instancias_2026-08-21/flecs_bevy_internals.md:96` ·
`docs/Flip/02_referencia_algoritmos_blender_5.2.md:26` ·
`docs/Painter/03_algoritmos_referencia_blender.md:118` ·
`docs/archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md:151` ·
`docs/archive/estado-2026-08-18/sculpt3d.md:10` ·
`docs/archive/estado-2026-08-18/timeline.md:10` ·
`project-memory/project_blender_texture_paint_reference.md:12`

⚠️ **Duas delas citam alvo PERMISSIVO** (a pesquisa de Components cita ECS sob MIT/Apache) —
ali citar é **lícito** e a cura é desnecessária. *Um detector que não pergunta a licença do
alvo conta a mais.*

---

## §2-bis — ⭐ ESTADO em 2026-08-24 (fim da jornada): a família do quad remesh está a ZERO

| família do alvo | antes | ⭐ agora | como |
|---|---|---|---|
| **quad remesh** (GPL-3.0) | 13 citações, **4 com transcrição** | ⭐⭐ **ZERO na árvore rastreada** | as duas em código (`ph2d-quantize`, `ph2d-quadfill`) e a tabela do `PLAN.md` foram **re-expressas em comportamento por fase**; ⛔ e a atribuição de licença da tabela estava **errada** — a fase de quantização por retalhos **não é MIT**, herda GPL-3.0 |
| **Blender** (GPL-2.0-or-later) | ~420 citações, ~21 com transcrição | ⚠️ **inalterado** | Classe A é higiene com custo de rastreabilidade (§3.3); a Classe B dele continua na lista do §2 |
| **memória** (`project-memory/`) | 2 | ⭐ **1** — e o que resta é de alvo **MIT**, onde citar é lícito | a nota do Painter foi curada |

⛔⛔ **Não leia «zero» como «o repo está limpo».** A vassoura cobre **uma** família — a do
quad remesh. *Um sweep verde vale exactamente o que a vassoura contém.* A cura da família
Blender exige a vassoura dela, e essa ainda não existe.

⚠️ **E a vassoura encolheu de 24 para 21 entradas, de propósito:** três eram **interface
pública** (opção de build, nomes de binário de linha de comando) que o **nosso próprio
runbook tem de falar** — mantê-las fazia o instrumento reprovar sobre documentação correcta.
*Um detector que não pergunta o estatuto do nome conta a mais.*

## §3 — ⛔ A prioridade que não é a maior contagem

| # | alvo da cura | quantidade | por quê primeiro |
|---|---|---|---|
| **1** | ⛔ os **2 de `project-memory/`** | 2 | ⚠️ **o symlink da memória é partilhado e injectado em TODA janela futura desta máquina** (§3.E) — inclusive numa janela I substituta. É o único sítio onde uma nota **contamina para a frente**, sozinha |
| **2** | ⛔ as **25** da Classe B | 25 | é a violação de **expressão**; são poucas e curam-se uma a uma |
| **3** | ⚠️ as ~435 da Classe A | ~435 | higiene; **meça o custo de rastreabilidade antes de ordenar a troca** |
| **4** | ⚠️ o **histórico** (`--git-history` deu 2 achados) | 2 | ⛔ reescrever histórico é caro e destrutivo; a decisão é do Enio, e **não** é urgente |

---

## §4 — O que este achado **não** é

- ⛔ **Não é o algoritmo copiado.** O código foi escrito de papers e de comportamento
  observado; é isso que o [ADR-0162](../../architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md) declara e é isso que a árvore mostra (zero C++ do alvo).
- ⛔ **Não é acesso escondido.** O `CLAUDE.md` §5 **declara** a referência do Painter/Sculpt
  abertamente. Acesso declarado + criação independente é a configuração do §1.3, não a
  acusação.
- ⛔ **Não queima janela nenhuma** — a régua do §6.2 classifica as 25 como *relance*.
- ⚠️ **É** o instrumento a mostrar que a casa vivia sob uma regra que **não tinha instrumento**
  — exactamente a doença que o `CLAUDE.md` §2 mede noutros sítios: *regra sem instrumento é
  nota que envelhece.*

---

## §5 — Recomendação ao dono

1. ⭐ **Adopte o sweep como portão**, senão a contagem volta: uma linha no `ship.sh` com uma
   vassoura por alvo declarado. ⚠️ **O sweep de hoje não pergunta a licença do alvo** — ele
   precisa de uma vassoura **por família**, senão marca vermelho sobre citação lícita de MIT.
2. Cure **os 2 da memória** e **as 25**, nesta ordem.
3. ⚠️ **Meça** o custo de rastreabilidade da Classe A antes de mandar trocar ~435 notas.
4. ⛔ **Não reescreva o histórico** sem uma razão que não seja arrumação.

---

## §6 — ⛔ Recusas MEDIDAS

| recusa | mecanismo | onde |
|---|---|---|
| ⛔ **Não curar as ~435 da Classe A em massa agora** | o endereço interno é hoje o único caminho para reconferir o fato contra o oráculo; a troca cega **apaga rastreabilidade paga** | §2.A |
| ⛔ **Não tratar as 25 como incidente do §6** | a régua do §6.2 classifica assinatura/nome isolado como **relance**; nenhuma é corpo de função | §2.B |
| ⛔ **Não reescrever o histórico do git** | 2 achados, custo destrutivo, zero urgência | §3.4 |
| ⛔ **Não confiar no detector sem a licença do alvo** | 6 das citações são de alvos **permissivos**, onde citar é lícito — contá-las infla o número | §2 |
