# HANDOFF — `line/sculpt3d` · **O PINCEL DE TECIDO passa a ser o do alvo** (2026-09-06)

> Worktree `Worktrees/line-sculpt3d`, ramo `line/sculpt3d`. ⛔ Nada integrado, nada pushado.
> Clean-room sob [`SKILL_Cleanroom_Reimplementacao.md`](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md):
> a espec é [`SPEC_cloth_brush.md`](../cleanroom/SPEC_cloth_brush.md), o ledger e as 54 fixtures
> vivem ao lado dela. **Esta janela nunca abriu o fonte do alvo.**

## §1 — O que mudou, numa frase

A **lei da referência** deixou de ser um experimento atrás de uma variável de ambiente e passou a
ser **o caminho de omissão** do pincel de tecido, com **29 dos 54 traços do oráculo dentro da barra
de paridade** (8 deles ao bit), os **oito modos de deformação e as três áreas alcançáveis pela
tela**, e **sete gates** que medem tudo isso — três deles a existirem porque a jornada descobriu que
o que os substituía media outra coisa.

## §2 — Como reproduzir (o smoke do dono)

```
cargo run -p ph2d-host-desktop --release
```
Pill **SCULPT** → pincel **Cloth** → as fileiras **Deformation** (8) e **Simulation Area** (3)
aparecem só com ele na mão. `PH2D_CLOTH_LAW=vbd` volta à lei anterior, para bissecar.

As medições: `cargo test --release -p ph2d-cloth --test oraculo_do_pincel <sonda> -- --ignored --nocapture`
com `sonda_da_paridade_com_o_oraculo` (o corpus inteiro), `sonda_passo_a_passo` (`PH2D_TRACO=<nome>`,
passo a passo com o anel imediato, o erro do passo, o pico e o vector) e
`sonda_dos_artefatos_do_oraculo` (as três grandezas de artefato, as faces invertidas e a compressão,
**nos dois lados**).

## §3 — As TRÊS leis que faltavam, e nenhuma era o que eu procurava

| # | o que eu procurava | o que era | como se vê |
|---|---|---|---|
| **Q8** | uma força em falta, ou mais varreduras no *Local* | a **construção da lista de restrições corre `passagens + 1` vezes** no *Local*, e o registo de duplicados vive UMA construção ⇒ cada restrição está lá em dobro | 27 dos 38 traços *Local* melhoram, os 12 não-*Local* ficam **byte-idênticos**, e a contagem de vértices movidos passa a bater **exacta** em oito traços |
| **Q9** | a amplitude do Snake Hook | o **centro da queda está UM PASSO atrasado** — a queda mede-se de onde o pincel *estava* | o pico passa a cair no vértice do oráculo (`0,86R` contra `0,86R`); toca em 7 traços e melhora os 7 |
| **Q11** | a lei que falta no aperto | **não falta nenhuma**: a força do aperto não decresce com a proximidade, o vértice ultrapassa o cursor, faces invertem e a partir daí **a ORDEM da lista decide** | a fixtura do mesmo traço com força `0,2` sai com erro **`0,000` nos doze passos** |

⭐⭐⭐ **A aritmética que fecha o Q8:** a lista dobrada é `[c₁..c_N, c₁..c_N]`, logo cinco varreduras
sobre ela são, **na ordem**, exactamente dez sobre a simples. O botão que eu media e o mecanismo que
o especificador achou são a mesma coisa — e é por isso que as duas leituras coincidiram ao número.
⚠️ **Mas só para os modos de FORÇA:** repetir apenas os pares deixa as âncoras no meio da lista e
piora o Grab de `0,050` para `0,415`. *Onde uma restrição está na lista é tão load-bearing quanto
quantas vezes ela lá está.*

## §4 — ⛔ Recusas MEDIDAS desta jornada (não as reconstrua)

| o que foi tentado | o número que o matou |
|---|---|
| o anel-1 pela **triangulação** | acerta o Arrastar *Local* e derruba o *Global* de `0,6457` para `0,2699`; a triangulação é da MALHA e não pode separar dois ramos que a partilham |
| **`PH2D_VARREDURAS=10`** como lei do *Local* | é bit-idêntico à construção dupla nos modos de força e **diverge** nos de âncora — o mecanismo é a lista, não o laço |
| o filtro de raio na criação de restrições da área *Dynamic* | `0,181 → 0,182`, e as outras nove inalteradas: quem segura os vértices longe é a banda |
| o peso da **normal** por vértice (área contra uniforme) | 19 traços, 18 inalterados |
| a **direcção do aperto medida no repouso** | melhora o plano (`1,380 → 1,012`) e piora a esfera (`0,542 → 0,939`) |
| a **trava** que impede o vértice de ultrapassar o alvo | **não é inerte**: parte os traços de um passo que hoje saem ao bit (`0,000 → 0,465` e `0,811`) |
| o **plano de queda pelo centro da área** (a leitura literal da §4.4) | `empurrar 0,944 → 1,250`, `arrastar 0,233 → 0,716` |
| **faces invertidas** como régua de classificação | não discrimina: o arrasto tem `41`–`57` e bate a `0,071` |
| a **compressão** do par mais apertado como régua | explica a família do aperto e nada mais (`empurrar_plano_local` lê `0,89` sem compressão e erra `0,944`) |

## §5 — ⚠️ Seis coisas que uma leitura rápida do diff entende ao contrário

1. **A lei da referência ser a omissão não é «ligar o que estava pronto»** — ela expôs TRÊS leis
   transversais que o adaptador não honrava (o alpha, a simetria, a declaração de inversão), e
   nenhuma era visível enquanto ele estava desligado.
2. **Duas das três barras do gate de artefatos foram RETIRADAS, e não afrouxadas.** Elas reprovavam
   a saída do PRÓPRIO alvo: medido nele, espinho até `0,900` e estica até `3,72×` nos traços de
   arrasto, contra o defeito de 05/09 que leu `0,690` e `2,98×`. *O alvo deforma mais do que o
   defeito deformava em duas das três colunas.* Sobra o **rasgo**, que discrimina por mecanismo.
3. **O gate `the_panel_offers_every_falloff_the_engine_has` era NOMEADO em dois doc-comments e não
   existia.** Uma promessa de gate lê-se exactamente como um gate.
4. **O gate 20 da espec foi corrigido pela medição, não implementado à letra.** «A nossa assimetria
   nunca passa a do oráculo» não é propriedade de nenhum dos dois lados num regime caótico: nós
   ficamos acima em 3 passos e abaixo em 9. Ele mede **dois regimes**, com barras da tabela.
5. **`PH2D_CLOTH_LAW=vbd` não é «o antigo», é o REPROVADO** — a lei VBD foi recusada pelo dono três
   vezes com foto e nunca teve paridade medida contra o alvo em modo nenhum.
6. **Três tectos de LOC foram curados por CORTE, e dois estavam vermelhos ANTES desta jornada**,
   invisíveis a todos os portões da linha: o `cargo test --bins` não alcança os gates que vivem em
   `tests/`. É a **sexta** ocorrência registada desta cegueira.

## §6 — ⛔⛔ A DECISÃO que é do dono, e as duas frases que a põem

A espec ([§5.2-ter](../cleanroom/SPEC_cloth_brush.md)) devolve-a explicitamente, e **não há terceira
saída** — a inversão nasce no PRIMEIRO passo, antes de a relaxação correr, logo nenhuma afinação do
solver a evita:

- **(a) reproduzir** — apertar com força alta vira o retalho debaixo do cursor do avesso, as faces
  atravessam-se e a superfície fica com um nó que nada desfaz depois. É o que o alvo faz hoje, e
  apertar com força baixa continua limpo. **É o que shipa agora.**
- **(b) limitar** — o aperto nunca ultrapassa o ponto para onde puxa, o nó não aparece em força
  nenhuma, e a nossa saída deixa de casar com a do alvo exactamente nos traços fortes.

⚠️ **O alvo sabe que (a) é defeito dele:** são duas das entradas ABERTAS do tracker que a §9 nº 23
lista. Reproduzir é reproduzir um defeito conhecido.

## §7 — O que fica ABERTO, com o número de cada um

⛔ **Não é uma família** — as duas réguas que tentei para o agrupar estão no §4 como recusas.

| família | traços e `err_max / max_oráculo` | o que já se sabe |
|---|---|---|
| **Push** | `plano_empurrar_plano_local` **`0,944`** · `_radial_local` `0,329` · esfera `0,303` | o de um passo sai **ao bit**; sem inversão e sem compressão ⇒ a suspeita é a **normal da área** |
| **Inflate** | `0,253` (plano) · `0,378` (esfera) | idem |
| **Expand** | `0,192` · `_1passo` `0,560` · esfera `0,557` | ⚠️ o `0,560` é uma razão com denominador minúsculo — o erro ABSOLUTO é `0,0011`, `2,3 %` de uma aresta |
| **Snake Hook** | `_2passos` `0,388`–`0,420` | o Q9 curou o passo 2; sobra o 3 (`max` `0,4460` contra `0,3439`) |
| **a esfera** | os sete não-arrasto, `0,27`–`0,59` | o arrasto na esfera BATE (`0,092`) ⇒ não é a malha nem a área |

⏳ **A pergunta Q12 já está escrita no INBOX** com estes números, e um subagente-E foi despachado
com ela.

## §8 — O diff

- **`ph2d-cloth`** — `verlet.rs` (a construção reaberta) · `verlet_gesto.rs` (as passagens, o centro
  atrasado do gancho, o zeramento da força por passo das âncoras, o campo `passagens`) ·
  `verlet_gesto_tests.rs` (**novo**) · `tests/oraculo_do_pincel.rs` (os gates 15-21 e as três sondas).
- **`ph2d-sculpt3d`** — `cloth_mode.rs` (**novo**: os dois enums) · `brush.rs` (dois campos) ·
  `stroke_cloth_ref.rs` (a lei de omissão, o alpha, as passagens) · `brush_verb.rs` +
  `brush_verb_predicados.rs` (**novo**, corte) · `stroke_cloth_num.rs` (**novo**, corte) ·
  `stroke_cloth_tests.rs` + `stroke_cloth_mode_tests.rs` + `stroke_cloth_artefatos_tests.rs`
  (**dois novos**, corte).
- **`ph2d-panel-sculpt3d`** — `paint/brush.rs` (as duas fileiras) · `event.rs` (os dois braços) ·
  `censo_das_fileiras_tests.rs` (**novo**).
- **`ph2d-editor-core`** — os dois arrays de ids. **`ph2d-i18n`** — duas chaves.
- **`shells/desktop`** — `sculpt3d_input.rs` (a pergunta com o pincel na mão) ·
  `sculpt3d_undo_tests.rs` (o gate percorre o braço que o gesto percorre).
- **`docs/3D/cleanroom/`** — a espec emendada quatro vezes (Q8·Q9·Q10·Q11, todas atestadas), o
  ledger, o INBOX e **6 fixtures novas**.

## §9 — Portões corridos no fecho desta fatia

`ph2d-cloth` 33 · `ph2d-sculpt3d` 390 · `ph2d-panel-sculpt3d` 77 · `ph2d-host-desktop` **5192** ·
o gate de undo do tecido **com GPU** · `architecture_workspace_file_loc_cap` · clippy `--all-targets`
limpo nas cinco crates tocadas · `cargo fmt`.
