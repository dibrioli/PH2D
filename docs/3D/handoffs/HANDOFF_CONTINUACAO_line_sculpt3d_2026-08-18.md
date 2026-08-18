# HANDOFF DE CONTINUAÇÃO — `line/sculpt3d` (2026-08-18)

> Para o agente que **assume a linha e implementa**. O bloco de retomada que o Enio
> cola é o [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md);
> este documento é o item **5 da FASE 2** dele — *o que já foi decidido, medido e
> REPROVADO*.

**Estado:** a linha **INTEGROU** em 2026-08-17 (39 commits + o handoff mestre). Ela
está **reaberta e limpa**, rebasada no `main`, com **1 commit** de higiene de
reabertura (`651e4bdce`).

---

## 1. Antes de ler qualquer código

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d
pwd && git branch --show-current
```

⚠️ **`pwd` TEM de terminar em `/Worktrees/line-sculpt3d` e a branch TEM de ser
`line/sculpt3d`.** A janela abre na raiz do repo primário, que está em `main`, e **o
mesmo path relativo existe nas duas árvores**: abrir `crates/ph2d-sculpt3d/src/…`
daqui edita a árvore errada, **compila e commita sem um único erro**, e ninguém
descobre até a integração.

Depois: `git rebase main` (obrigatório no início de **cada** jornada) e
`cargo check -p ph2d-sculpt3d -p ph2d-panel-sculpt3d`.

---

## 2. A linha, hoje

| | |
|---|---|
| branch · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d/` |
| HEAD | `651e4bdce` — **1 commit** à frente do `main`, 0 atrás |
| árvore | limpa |
| última integração | **2026-08-17** — [`HANDOFF_INTEGRACAO_line_sculpt3d_MESTRE_2026-08-17.md`](HANDOFF_INTEGRACAO_line_sculpt3d_MESTRE_2026-08-17.md) |
| verbos | **23** (`Verb::ALL.len()`, que é a fonte) · alvo honesto **29** |
| cenas de smoke | **1..33 sem duplicata** ⇒ ⚠️ **próxima livre: 34** |
| `PROJECT_SCHEMA` | **84** · tripla `(84, 13, 14)` |
| próximo ADR livre | **0160** |

⚠️ **O `main` ganhou um fix NO SEU MÓDULO que não veio desta linha** —
`04e10e74c fix(sculpt3d): o teclado da cena 3D só é dela enquanto ela está EM USO`.
A porta do 3D perguntava *"existe uma cena?"* e **sair do modo nunca destrói a
cena**, então o primeiro clique no pill armava o portão para o resto da sessão e o
Motion Nodes perdia os atalhos. Ele toca `input_dispatch.rs` ·
`input_dispatch/keyboard.rs` · `sculpt3d_keys.rs` — **leia-o antes de encostar em
teclado**.

---

## 3. O estado do módulo — leia nesta ordem

1. [`docs/3D/00-INDEX.md`](../00-INDEX.md) — o cofre.
2. [`docs/3D/21_plano_modos_e_ferramentas.md`](../21_plano_modos_e_ferramentas.md)
   **§7.25 (o placar)** — é ele que decide qual é o próximo item. ⚠️ Ele foi
   **reconferido em 18/08**: a W7 estava listada como *"por abrir"* e está **fechada**,
   e o cabeçalho divergia da própria tabela. *Um placar que envelhece manda construir
   o construído.*
3. [`06.1-Waves-riscos-e-alvos.md`](../06-Plano/06.1-Waves-riscos-e-alvos.md) — o
   roteiro das dez waves, **todas fechadas**; do roteiro sobra o **marching cubes**.
4. [`BUGS_sculpt3d.md`](../BUGS_sculpt3d.md) — os quatro cuja **causa enganava**.
5. O handoff mestre de 17/08 (acima) — **§8, a lista aberta**, e o **§4.1**, o ponto
   de merge sensível.
6. [`DIRETIVA_IMPLEMENTACAO.md`](../../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)
   — inteira, e **releia a cada passo**, como ela manda.

---

## 4. ⭐ A FILA, na ordem, com o preço

> ⚠️ **Ordem do Enio (18/08):** *"coloque o que está fora da fila no fim da fila."*
> Nada fica de fora — os itens **5-10** são os que antes estavam nomeados como
> *decisão dele* ou como *recusa medida*, e agora têm lugar. **A distinção entre as
> duas espécies continua escrita, porque ela é o que impede o próximo agente de
> desfazer uma medição achando que está fechando uma tarefa.**

### Trabalho de engenharia (1-4)

**1 — W9 · MESH FILTER (9 tipos).** 🔄 **7 dos 9 hoje** — a **W9a** deu a fiação e os
4 tipos que reusam verbo, a **W9b** deu os 3 que **não têm verbo nenhum** (Scale ·
Sphere · Random) e o **selector** que os torna alcançáveis. ⚠️ **As duas metades da
W9b não podiam shipar separadas:** enquanto a lei era derivada do verbo em mãos, um
kernel novo sem a row de escolha seria código que nenhum gesto percorre. **Falta a
W9c** (Sharpen de dois passes · Enhance Details) — e a referência já está lida: o
`sculpt_filter_mesh.cc` precomputa um `sharpen_factor` por vértice **mais** as
`detail_directions` (o deslocamento laplaciano), e o *Enhance Details* é o segundo
passo sozinho. ⭐ **O mais barato da lista inteira, e há
precedente executável:** o *Filter Layer* do Painter mostrou que **não há kernel
novo** — o filtro preenche o `amount` **uniforme** e chama o MESMO render dos verbos,
o que faz a máscara, a simetria, o `pre` congelado e o undo virem de graça. Depende
da W4, **fechada**.
⚠️ **A lição do irmão, que vale aqui inteira:** ele **recusa** os verbos cujo alvo é
ajustado à **PEGADA do dab** (uma camada não tem pegada), e recusou o `Layer` porque
filtrar com ele é **translação uniforme** — e a luz lê `∇h`, que uma constante não
muda ⇒ **um knob que não move um pixel**. Faça o censo por verbo **antes** de escrever
o botão.

**2 — W11 · HANDLES** (Pose · Boundary · Nudge · Thumb). Depende da W5, **fechada**
(os Kelvinlets já estão lá).

**3 — W10 · CLOTH** (XPBD) **+ Cloth Filter** (5 tipos). Depende da W9.
⚠️ **Wave cara e com risco próprio:** solver iterativo dentro do laço de dab.

**4 — W12 · A GEODÉSICA** (Heat Method na pegada) → `l-mode` de falloff para a
família inteira. Depende da W6, fechada.

### Decisões de produto do Enio, agora NA fila (5-8)

⚠️ **Estas quatro estão medidas e o número está na mão dele.** O que falta em cada uma
não é engenharia: é **de onde vem o número**. Um agente que as pegar **começa
perguntando ao Enio**, não escrevendo código.

**5 — A LEI DO ZOOM.** Medido: a esbeltez `altura/raio` vai de **0,135** com a câmera
longe a **5,857** a 4× de zoom. ⚠️ **É a lei do Blender, em três fatos do fonte**
(`rna_brush.cc:3230` declara `height` como `PROP_DISTANCE` · `layer.cc:101` a
multiplica **crua** · o `cache.radius` de lá **também** sai dos pixels), e o default
dele é **0.5** contra o nosso **0.1** — no mesmo zoom o Layer dele espiga **cinco
vezes mais**. A cura que achata a curva é a do SculptGL (`Brush.js:62`, o
deslocamento escalando com o raio) e **DIVERGE da referência**, contra a ordem
permanente *"idêntico ao Blender"*.
⚠️ **A cerca é EXECUTÁVEL** (`the_coat_height_is_a_world_distance_and_does_not_follow_the_radius`,
mutação sangra) — mudá-la exige **duas** edições, de propósito.
⚠️ **E a SEGUNDA causa não é de lei nenhuma:** a 4× de zoom o pincel cobre **1,45
arestas medianas**. *Nenhuma lei de deslocamento conserta um pincel mais estreito que
a malha* — quem conserta é **subdividir**, e o gesto já shipa. **Confundir as duas faz
trocar a lei e continuar sem resolução.**

**6 — W1 · OS DEFAULTS DO `B`.** ⚠️ **Sem cura em código, e a razão está medida
(§7.0):** os defaults por-tool do Blender moram num **`.blend` BINÁRIO**, não no
fonte. Ou o Enio dá a fonte do número, ou ele decide os nossos.

**7 — O DRAW SHARP.** ⚠️ **É o item da W1, não um verbo a mais** (§7.18): o que o
nome promete mora na **CURVA**, e a curva de fábrica por-tool está no mesmo `.blend`
binário do item 6. *Construí-lo antes do 6 produz um chip que desenha o que já
existe.*

**8 — K1/K2 do [ADR-0150](../../architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md).**
Os dois kill-criteria **disparam** no regime extremo, ⚠️ **e o K2 aponta para o lugar
errado** — ele manda migrar as normais para a GPU, que custam **1,66 de 13,1 ms**; o
custo real é **descobrir a vizinhança** (**11,5 ms = 88%**). Três caminhos, e a
escolha é **de produto**: paralelizar a descoberta (os 32 threads já renderam 6,4× nas
normais) · migrar o kernel para a GPU mirando o alvo **certo** · ou **aceitar**,
porque é exatamente onde a multiresolução existe para pôr o artista num nível mais
baixo. ⚠️ **No pincel de detalhe (2% do modelo) o mesmo dab custa 0,566 ms, 14× sob o
teto** — o K1 só dispara com um pincel que cobre **19% da malha inteira**.

### Recusas MEDIDAS, no fim da fila (9-10)

⚠️ **Estas duas não são tarefas, e pô-las na fila não as torna tarefas.** Elas são
**cercas com número atrás**. *Reabrir uma custa um NÚMERO NOVO — uma medição que
mostre que a minha estava errada —, nunca uma opinião sobre a foto.* Se o próximo
agente as "fechar" sem isso, ele terá desfeito uma medição e o produto ficará
**diferente da referência**, que é a única coisa que este módulo não pode ser.

**9 — ⛔ O PENTE do platô da demão.** Medido: o platô ondula **0,0093 de UMA aresta**.
Não é caráter de kernel — é a **parede a escadear pela grade**, e ⚠️ **a referência
escadeia igual**. O que o reabre: uma medição mostrando ondulação **maior que uma
aresta**, ou o Blender a não ter.

**10 — ⛔ A DUREZA ALTA do Layer.** ⚠️ **A do Blender TAMBÉM é feia — o alvo é a
violência dele.** *Se o resultado ficar bonito e diferente, ele está errado.* O que o
reabre: uma medição contra o `layer.cc` mostrando que **nós** divergimos, não que o
resultado é feio.

### O item que sobra do roteiro

**11 — O MARCHING CUBES** (W7 do roteiro das dez). ⚠️ **Ele é *deliberadamente* o
segundo:** o Surface Nets veio antes porque devolve **um vértice por célula e valência
4**, a topologia que um escultor quer receber, enquanto o MC devolve triângulos finos
que **subdividem mal**.

### Caudas nomeadas (não são waves)

- O **falloff, a referência e o raio no atalho de teclado** — os três precisam do
  `Sculpt3dUi`, e trazê-los para o motor exigiria dar à crate um tipo de painel.
  **Divergência real, escrita nos dois lados.**
- O **`space attenuation`** não conferido — ⚠️ ele é **TAXA e não forma** (muda em
  quantos dabs a demão fecha, nunca a espessura final).
- A lacuna do **`Hit::normal`** (quad "gravata" → `[0,0,0]`) — gatilho declarado: *o
  primeiro leitor de produto*. ⚠️ **O cursor NÃO a adotou, de propósito** (ele lê
  `Mesh::normals()`).
- A **resolução do remesh não é autorável** (o botão usa o default 150) · *fundir não
  solda* · o **W9.3 (colapso) pendente de smoke**.
- ⚠️ **A tabela de cronometragem contra ZBrush/Blender NÃO EXISTE.** Enquanto ela não
  existir, a frase de performance fica **entre aspas no pedido, nunca no nosso
  relatório**.

### ⛔ UM VERMELHO DO `main`, achado pelo `--ignored` da W9b-b (2026-08-18)

`sculpt3d::bake::light_measure::the_two_lights_agree_where_the_form_turns_away`
(`sculpt3d_bake_light.rs:509`) mede **0,3370** contra uma barra de `0,01` — *"o aro
divergiu no balde 0"*. Ele afirma que **a luz do BARRO e a da TINTA concordam onde a
forma vira**, e é a carta da `ph2d-light` em forma executável.

⚠️ **NÃO é desta linha, e não é opinião — são QUATRO testemunhas medidas:** a cadeia
inteira da luz assada tem **diff VAZIO** contra o `main` (o gate · `baked_form_planes.rs`
· `baked_form.rs` · `sculpt3d_bake*.rs` · `ph2d-render` · `ph2d-painter-brush` ·
`ph2d-gpu` · `ph2d-light` · `ph2d-mesh-render`) · o `baked_form_planes.rs` **não depende
da `ph2d-sculpt3d`**, a única crate que esta linha mudou e que a shell consome · ele
falha no **HEAD** com o número **idêntico a quatro casas** · e falha no **`main` limpo**,
rodado na árvore primária. ⚠️ **E falha ISOLADO em 0,25 s**, logo não é a classe de
flake de carga desta workstation.

⚠️ **O número NOMEIA o mecanismo, e o próprio doc do gate traz a tabela:** as duas
mutações que ele documenta medem **0,3002** (*o `resolved_lamps` larga a `dir`*) e
**0,3514** (*o `build_input` manda `form: None`*). O medido, **0,3370**, cai entre as
duas e muito longe do verde de **0,0020** — ou seja, a assinatura é *o bake perdeu a
tradução do rig ou da forma*, não uma deriva numérica. ⚠️ **A premissa `DEFAULT_ENV = 0`
foi conferida e está de pé** (o plano avisa que levantá-la separa as duas luzes **por
desenho**), então não é o slider de ambiente.

⚠️ **Ele é invisível entre integrações:** mora num `--ignored`, e nem o `ship.sh` nem um
fechamento por `cargo test -p` o alcançam. **Wave própria, de outro dono** — não o
conserte dentro de uma wave de filtro.

---

## 5. O que já existe e NÃO se reconstrói

- **A memória por-verbo.** `VerbSlot { brush, radius_px }`, um por `Verb::ALL`.
  ⚠️ **O `arm_verb_defaults` MORREU** (`1e03095b1`): trocar de verbo é **salvar o slot
  que sai e carregar o que entra**, e o estado de fábrica sai de `VerbSlot::for_verb`.
  *O slot SABE* — não escreva heurística de *"o artista mexeu?"*.
- **`reconcile_mode`** — uma lei, **dois chamadores** (o chip, sobre o pincel VIVO; o
  *apply to all*, sobre o pincel de **cada slot**). ⚠️ Escrever o modo **sem**
  re-resolver o `falloff` deixa aquele slot com a curva da referência ANTERIOR.
- **O cursor conformado.** `ring_on_surface` desenha no plano tangente; ⚠️ a **0° ele
  coincide com o círculo de tela ao centésimo de pixel** (é por isso que não pisca), e
  o círculo **fica** como recuo para *"não sei a orientação"*.
- **A incidência já É a do Blender** — `base_nrm[s]` (normal congelada no pen-down) é
  o `orig_normals[i]` do `layer.cc:101`, e a pegada é uma **esfera de mundo**.
  ⚠️ **Não "melhore" isto:** o vazamento a 85-90° é o que a fonte faz (lá o front-face
  é **opt-in** e nenhuma linha do Blender o liga).
- **A demão (Layer)** portada do `layer.cc`, com as **onze leis conferidas uma a uma**
  no handoff de 16/08 — e a **divergência do front-face** curada como **flag de
  pincel**, não lei de modo.
- **W4 · W6 · W7 · W8** fechadas (o alisamento · os dabs que não são discos · o plano
  MLS · a demão).

---

## 6. As armadilhas deste módulo (as que custam tempo)

- ⚠️ **Os gates de GPU são `#[ignore]` e precisam de adapter.** Sem ele fazem *skip
  gracioso*, **que não é verde**: `cargo test -p ph2d-mesh-render --release -- --ignored`.
  **Meça, não cite** — o número que circula (54) é de uma árvore anterior.
- ⚠️ **Rode a suíte em DEBUG também.** Precedente do repo: o `ph2d-flip-colorize`
  panicava só ali (um `wrapping_sub`), e a nota sobreviveu ao fato por três
  integrações.
- ⚠️ **O roteador de cenas do sculpt3d NÃO é um `match`** — cada arquivo testa a env
  var por conta (`== Some("33")`) ⇒ um número repetido **não** é `unreachable pattern`
  do compilador: é uma cena inalcançável **em silêncio**. Hoje **1..33** sem duplicata;
  **conte a próxima lendo os arquivos, nunca uma nota.**
- ⚠️ **Três cenas imprimem o número que as torna válidas.** *Se a linha não aparecer, o
  resto do smoke não diz nada.* E **rode uma vez SEM a env var** — é a metade que prova
  a inércia do frame 2D.
- ⚠️ **Os três gates de LOC medem coisas diferentes, e um verde pode ser do gate
  errado:** o `architecture_workspace_file_loc_cap` (700) **EXCLUI `ph2d-panel-*`**;
  quem é dono deles é o `architecture_panel_loc_cap` (**600**, e só `src/**`); a shell
  tem o `shells/desktop/tests/file_loc_caps.rs` (600). *Foi assim que o `state.rs`
  chegou a 727 com um teto verde ao lado.*
- ⚠️ **O `state.rs` do painel foi PARTIDO** (`slots.rs`), com **re-export** mantendo
  todo caminho — porque um arch-gate lê o fonte atrás de `VerbSlot::for_verb`. *Quem
  editar o `state.rs` procurando o `VerbSlot` funde limpo contra um arquivo de onde ele
  saiu.*
- ⚠️ **Ids novos são `hash_node_id`** ⇒ fora de todo gate de contagem, cobertos pelo
  `node_id_collisions`. `SCULPT3D_VERB` tem o tamanho do `Verb::ALL` (**23**) e **há
  gate que os compara**. O painel já tem o **scrollbar id 840**.
- ⚠️ **A posição do pill SCULPT na topbar é load-bearing, não gosto:** os **sete
  primeiros** clusters são o grupo da ESQUERDA; ele entra **depois do FLIP**, e um
  merge que o mova quebra o layout **sem nenhum gate reclamar**.
- ⚠️ **O `ph2d-i18n/src/lib.rs` foi PARTIDO** — as chaves `panel.sculpt3d.*` moram no
  irmão `sculpt3d.rs` e os irmãos são consultados **em CADEIA**
  (`vector::tr(k).or_else(sculpt3d::tr)`). **Um irmão novo entra nessa cadeia, nunca
  num segundo `match`.**
- ⚠️ **Se você bumpar `PROJECT_SCHEMA`** (hoje **84**): o valor se **CONTA** contra o
  `main` do dia, a conferência é na **família `project*` inteira** (o arquivo foi
  partido: a escada e a constante vivem no `project_schema.rs`), e **escreva o degrau
  na escada** — quem conta o próximo lê a escada, não o literal. ⚠️ Esta colisão passa
  **MUDA** quando duas linhas escrevem o mesmo literal: o git não sabe o que o número
  significa.
- ⚠️ **O registro do `ph2d-ecs` tem TRÊS casas** (o registro + os espelhos em
  `ph2d-render` e `ph2d-script`), cada uma rodando só na suíte da própria crate. Este
  módulo não o toca hoje; **um componente novo move as três**.
- ⚠️ **`rayon` novo exige ADR novo** (a cerca do ADR-0109 mora no `Cargo.toml` de cada
  crate que o paga). O módulo já tem o **[ADR-0156](../../architecture/decisions/0156-sculpt3d-ao-trace-is-a-per-vertex-gather-rayon-exception.md)**
  (o traço de AO) e o **[ADR-0159](../../architecture/decisions/0159-sculpt3d-the-dab-vertex-loop-is-a-row-disjoint-map-rayon-exception.md)**
  (o laço de vértices do dab).
- ⚠️ **`measure_brush_kernel` é kill de RELÓGIO** e já reprovou sob `load average 26`,
  passando isolado. *Nenhuma leitura de relógio desta workstation significa coisa
  nenhuma acima de `load ~5`.*
- ⛔ **A referência do Blender (`/home/enio/Documentos/Recursos/BlenderSculpt`) é
  GPL — COMPORTAMENTO apenas, nunca código**, e não se edita o clone. O SculptGL
  (`/home/enio/Documentos/Recursos/SculptGL`) é MIT.

---

## 7. O que você NÃO faz

Fecha a wave, roda o **gate batched 1× sobre o diff acumulado** (DIRETRIZ §6.6.A.2 +
DIRETIVA §3-§5), escreve o **handoff de integração** (DIRETRIZ §1.5.9, **nesta
pasta**), reclama o `target/*/incremental` da worktree — e **PARA**.

⛔ Você **não** integra, **não** roda `scripts/foundational-integrate.sh` e **não**
faz `git push`. Integração e ship são do Enio, por **ordem explícita**, via agente
integrador dedicado (CLAUDE.md §0.7).

⛔ **Contrato congelado** (CLAUDE.md §6) e **`ph2d-expr`** (ADR-0039): se a tarefa
pedir, **PARE e reporte**.

---

## 8. Ao começar a trabalhar

O primeiro output é a **TRIAGEM** (DIRETRIZ §2). Inner loop = **só `cargo check -p`**;
teste, clippy e auditoria **uma vez**, no fechamento.

⚠️ **E a primeira coisa de toda wave deste módulo é RECONFERIR a célula do plano que
manda fazê-la.** Só nesta linha, quatro notas envelheceram antes de alguém voltar a
elas — a **W7** listada como *"por abrir"* já fechada · o cabeçalho do placar
divergindo da **tabela logo abaixo dele** · o **`arm_verb_defaults`** nomeado como
porta viva em **cinco** passagens depois de morto · e a frase *"import/export, objeto
misto e merge/isolate não foram tocados"*, que sobreviveu ao fato por **duas
integrações** (os arquivos existiam). *O que se perde ao não reconferir não é tempo: é
construir o que já existe.*
