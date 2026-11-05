# HANDOFF DE INTEGRAÇÃO — `line/NovaUI`: todo o app em cartões (05/11)

> Leitor: o integrador `ph2d-44` e as LLMs seguintes. Linha `line/NovaUI` (worktree `Worktrees/line-NovaUI`), `TIP` = o commit que traz este handoff (a mensagem a `ph2d-44` diz o hash); último de código `6c61e8514c`.
> A linha NÃO integra, NÃO faz push, NÃO faz ship. Antecessor: [integração de 03/11](HANDOFF_INTEGRACAO_NovaUI_2026-11-03.md)
> (já no main). Registo de cada bloco (M/V/q/R/H/G/F1–F17), decisões e fotos: [10 tracker](../10_TODO_O_APP_TRACKER.md).
> Ponto de partida da janela que fechou: [HANDOFF 04/11](HANDOFF_line_NovaUI_2026-11-04_PARA_NOVA_JANELA.md).

## 1. Base e tamanho

- Base: `effe04a1c9` (`main`, GEF 22). `main` não andou desde o rebase (`HEAD..main` = 0). Até o último commit de código: **303** commits,
  **1 286 ficheiros, +85 393 −7 254**, mais o commit deste handoff.
- Backup pré-rebase: ramo local `backup/NovaUI-pre-rebase-2026-11-05` (**apagar depois da integração**).
- Conflitos do rebase, resolvidos: `crates/ph2d-i18n/src/inspector.rs` (`x_remove_animation` = «Remove»,
  `pp_rev` = «Ping-pong Reverse»: lado do main), `crates/ph2d-panel-inspector/src/sections/anim_rows.rs` (lado do main),
  `project-memory/MEMORY.md`.

## 2. O que entrou (por família; detalhe no tracker)

- **Chrome M1–M9, varredura V1/V2, capricho q1–q4, rodadas R1–R6:** barra de ferramentas, moldura das áreas de trabalho
  (Timeline, Motion Graph, Audio Editor, Pixel Timeline/Preview, Code, Authored, Widget Lab), nomes inteiros das ferramentas
  de imagem («Remove Background», «Make Square», «Color Equalization»; «Reshape» virou Transform), chip que alarga e `⋯`.
- **Vector G1–G8:** painel em cartões (abas Drawing/Interface no cabeçalho, Snap 2×2, Fill/Stroke/Effects/Advanced).
- **Painter H1–H12:** Layers, Wet Paint, Mask, presets, em cartões. O pintor antigo só com `PH2D_CARDS=0`.
- **Inspector:** todo tipo de objecto; alfinete (Object, Appearance, Fill, Stroke, Ordering, Material, Transform,
  componentes, sprite Render/Color/Slice/Sampling), dois presos, girar/espelhar.
- **Painéis em cartões:** Physics, Tokens, Grid, Macros, Tags, Assets, Audio Mixer, Padding, Upscale, Equalize,
  BG Removal, Color Eq, Flip, Pixel, Hierarchy.
- **Fecho F1–F17 (04–05/11):** F1 os vermelhos e a revisão dos 10 testes da R6 (nenhuma regressão escondida; o `⋯` sobre o chip
  era regressão real, curada) · F2/F7 alças de escala e painel do traço · F3 voo do preset, On switch, grade do Tilt · F4 defeitos
  pequenos · F5 provas de undo e custo dos presos · F6/F12 a linha da multisseleção (§9) · F9 fmt, 55 avisos de clippy, tectos ·
  F8 passada em janela real (Xvfb + xdotool, sem arnês) · F10 campo numérico (pega abre a escrita,
  duplo clique repõe e abre) · F11 alfinete dos cartões de sprite · F13 trava do Scale por chave (`travas.rs`),
  foco da seta · F14 piso das teclas ↑↓ e do arrasto (`number_scrub_floor`) · F15/F16 clamp seguro, tectos por corte,
  chaves da legenda do rail · F17 a Galeria mostra `Tool Bar` e `Floating List`.
- **Sprites/Luz 2D:** o bilinear lia a célula vizinha; `crates/ph2d-gpu/src/recorte_uv.wgsl` é a porta única
  ([22](../../Sprite_projeto/22_o_recorte_nao_le_o_vizinho.md)). Medido em RADV e RTX 5060 Ti.

## 3. Foundational tocado (superfície de colisão)

- `ph2d-editor-core`, portas NOVAS: `Panel::pinta_com_a_aba_atras` · `HeroLayout::fila_que_cobre_o_topo` ·
  `WidgetStore::publish_own_list` / `take_own_list_dismissed` · `paint_dropdown_list_reordering` / `dropdown_list_bands` /
  `DropdownRowInHand` · `HitIndex::desde` / `alvos_desde` · `number_scrub_floor` · gizmo de escala:
  `partida_do_grupo`, `escala_do_primario`, `fator_do_grupo`, `uniforme_do_grupo`, `grip_pivot_world`.
- `ph2d-vector`: `scene::colar_cena` (o único `append`), `append_scene`, `VectorScene::is_recording`, `extend_record`.
- `ph2d-ecs`: `Transform::de_autoria` (acrescento). `ph2d-render` passou a depender de `ph2d-vector` (era dev-dep;
  `Cargo.lock` igual). Shader: `crates/ph2d-gpu/src/recorte_uv.wgsl`.
- **Crate nova:** `shells/presos` (`ph2d-host-presos`; a shell troca uma linha por `ph2d_host_presos::chaves_do_quadro`).
- **Contratos §6: nenhum tocado.** `git diff --stat effe04a1c9 HEAD` vazio em `ph2d-nodegraph`, `ph2d-vector-doc`,
  `ph2d-imageio` e em `tool.rs`/`tool/` do `ph2d-editor-core`. `PROJECT_SCHEMA` **intacto (216)**, tripla (216, 13, 29) de 03/11.
- Próximo ADR: nenhum criado (0198 segue livre). Cenas: `PH2D_VEC_CARTOES_SMOKE=1..13` · `PH2D_VEC_BONE_SMOKE=9` ·
  `PH2D_GALLERY_SMOKE=1`.

## 4. Catracas (antes → depois → porquê)

| Catraca | Antes → depois | Porquê |
|---|---|---|
| altura de abertura `vector` | 250 → 260 | cartões do Vector com a mesma folga do Inspector |
| altura `painter_layers` | 1 618 → 1 670 | cartões do Painter; o nome quebra em vez de cortar |
| `onde_comeca_o_valor` | `COLUNA_UNICA_PISO` 11 → 3; declarações e Galeria colunas 3 → 2 | a Galeria passou a duas colunas; piso mede o que sobrou |
| `PISO_DE_PONTEIROS` (memória) | 115 → 95 | 101 medidos, 0 órfãos; o índice encolheu |
| `nenhuma_escolha_do_app_e_montada_a_mao` | piso 31 → 63 | a varredura estava POBRE (em cartões o corpo recua 12–20 px e o teste de `x` perdia 9 painéis), não houve redução; 4 exceções novas em `FORA` com censo de obsolescência: abas do Vector, abas da Galeria, matriz de camadas da Física, grelha de bits da Galeria |
| `TELAS_FORA_DO_CENSO` (Painter) | novo, 69 | telas sem cartão declaradas e contadas |
| Galeria | ritmo 16 → 0, grudadas 1 → 0, colunas 3 → 2, irmãos 7 → 3 | F10/F11 |
| `POPULACAO_RITMO` | novo, por painel | o zero tem população |
| `nenhuma_colagem_de_cena_fora_da_porta` | novo; pisos 7 000 ficheiros (medido 7 163) e 29 chamadas da porta (medido 29) | F12 |
| `WIDGET_OPT_OUT` | +3 | `camada_das_listas`, `card_censo`, `censo_cartao` (lido o código: nenhum tem rosto) |

- `allow(too_many_arguments)`: saldo **+42 sobre 1 192 do main**, em funções de pintura. `#[ignore]` novos: **19**, todos com motivo
  (foto, medição).
- Testes movidos pela R6 e o veredito (teste atrasado × produto regredido): tabela de F1 no tracker.

## 5. i18n e ids

Seis chaves de `crates/ph2d-i18n/src/tool_rail/caption.rs` renomeadas em `65c4d97dea` (a legenda do rail acha a chave pela sigla).
Texto idêntico antes e depois; `a_legenda_em_cartoes_e_em_caixa_normal` fixa as seis. «Remove» da lista de animação fica (do main).

## 6. Risco de colisão

- Pontos quentes: `ph2d-editor-core` (panel, widget, gizmo, interaction), `ph2d-panel-inspector`, `shells/desktop`, `ph2d-i18n`.
- **Luz 2D e `sprite`** pelo `recorte_uv.wgsl` (porta única do recorte de UV): qualquer linha que mexa na amostragem de sprite deve
  passar por ele.
- **GEF** em `crates/ph2d-panel-inspector/src/sections/anim_rows.rs` (resolvido a favor do main).
- Quem pintar com `vello::Encoding::append` fora de `colar_cena` reprova o portão novo.

## 7. O que só a integração apanha

- A **suíte completa e o `nextest-impacted` nunca rodaram nesta linha** (regra do dono): os testes fora da lista da §11 só a suíte os vê.
  Foi a bateria por nome que apanhou, já no fecho, 8 tectos de LOC e 4 ciclos do DAG: espere ainda alguns vermelhos deste tipo.
- As provas de janela real da F8 são em Xvfb + xdotool + Radeon (`MESA_VK_WSI_DEBUG=sw`), não Wayland/KWin nem NVIDIA.
- Os cortes da F16 (8 tectos) não têm mutantes, só testes por nome.
- 7 testes pesados reprovaram sob carga na jornada (não rastreados um a um): trate pela [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md).
  `h5_`/`h6_os_cartoes::every_click…` estouravam 180 s; curados na F11 com `em_paralelo`: 29,7 s e 38,6 s sob load ~100.
- `no_magic_numeric` alternou uma vez sob edições alheias, na mesma árvore.
- `panel_files_under_loc_cap` reprovou 1× na F3 sem causa achada; verde depois (`tracks.rs` estava acima na altura).
- `a_repetida_continua_a_costurar` leu zeros 1× em lote pesado, sem causa; verde em 20+ corridas.

## 8. ABERTOS técnicos

- `c3_mascara.wgsl` sem portão; custo por fragmento do recorte não medido; bits na NVIDIA não medidos.
- A arte que encosta na borda perde orla macia ao minificar (cabelo a 0,5×: 0,39 → 0,58, 9 px); decisão do dono.
- Ctrl+Z em janela real sem prova ampla (Scale: prova com `snapshot_to_world`, não com `ProjectState`).
- Custo dos presos: ~1,2 ms/quadro a 5 objectos (`--profile smoke`, load ~21), sem cura; 1.º quadro com caches sujas ~9 ms.
- Arrasto de secção real não exercido em janela (só `a_pega_reordena`); `vello_fundo` sem teste que corra.
- `pilula`/`segmento` de `Barra` não estão na Galeria (pintá-los pede layout próprio).
- Trava de escala: «bits reciclados não herdam» foi provado com mundo novo; `despawn`+`spawn` no mesmo mundo não foi testado.
- Auto Key no duplo clique do ícone provado por pedido igual, não ponta a ponta.
- Rolagem no desfazer que tira cartão: sem prova.

## 9. O bug do dono (a linha da multisseleção)

Sintoma: arrastar no canvas não desenhava o retângulo. Causa: `Encoding::append` do vello faz `flags = other.flags` e apaga
`FORCE_NEXT_TRANSFORM` deixado pelo texto; o retângulo herdava o transform do glifo. Origem `c0b7cb8506` (R4, só nesta linha). Cura
`0b2351c621` (F4); família (12 colagens cruas, 6 alcançáveis, nenhuma disparada em janela real) em `e4bc6dd7e8` + portão
`nenhuma_colagem_de_cena_fora_da_porta`. Contraprova: repor `flags = 0` antes do retângulo tira-o da janela com o arrasto vivo
(`maquete/f6_sem_a_cura_o_retangulo_sai.png`). O binário das 23:31 que o dono usou foi sobrescrito: a ligação com o sintoma dele é
por mecanismo, não por reprodução nesse binário; por isso é o item (a) do smoke.

⚠️ O tracker cita hashes de antes do rebase em F1–F7 (ex.: `91d95e4c23` = `0b2351c621`, `615113d5b2` = `c0b7cb8506`): acham-se em
`backup/NovaUI-pre-rebase-2026-11-05`; o título do commit é igual.

## 10. Decisões tomadas sem o dono (lista D)

Mantidas, para ele confirmar no smoke:
- Regra 6 (Per-corner sem código) · «Behind Parent» · legendas sobre os chips da barra · Inspector vazio ao lado do Quadro ·
  Size/Wrap width do texto em metros · proposta da Galeria (17) · título «Background Removal» · campo Filter Clip · arte na borda.
- Nomes crípticos em dúvida: `Jetty`, `+M`, `DTCG`, `Offset (1/n)`, `Cross`, `Pick Shapes`, `Smooth wash body (off · on)`.
- De F1–F17 (tracker): cartões de contador com «Now: 42» · custo dos presos 1,2 ms · eixo a zero em grupo não abre pelo gizmo
  do grupo · arrastar além do oposto pára no piso (a Figma vira) · véu do Add Object · Harmony «Comp»/«Anlg» · ícone Transform
  no 3.º chip (parece o de Move) · «Remove» do main · arrastar um campo pára no piso da faixa mas passa do tecto; setas e teclas param no tecto (F14; `number_input_body_drag_uses_registered_rate_unbounded` exige-o livre).

## 11. Prova de fecho e binário

Provas por nome, mutantes e fotos: no tracker, bloco a bloco. Bateria final sobre `adc81fc44f` (`PH2D_SUITE=fecho`):

| Grupo | Resultado |
|---|---|
| `ship.sh --so-estatico` (fmt, clippy, deny, audit, typos, standalone, workflows, recontagens, índices) | 0 ✗; audit só avisou RUSTSEC-2026-0186 em `memmap2` 0.5.10, já permitido |
| `censos-da-arvore-combinada.sh` | 132/132; 12 de 12 censos correram |
| `arch_*`/`architecture_*`, todas as crates | 196/196 |
| Contratos, `ph2d-vector`, Inspector, Timeline, Painter (`h5`, `h6`), `ph2d-host-desktop`, pincéis | 153/153 + 26 da 3.ª passagem; alarmes de pincel verdes, nada a aprovar |
| `ph2d-editor-core` e `ph2d-panel-registry-init` | 178/179; o TIMEOUT era o censo do scrub, curado na F18 |
| `clippy --workspace --all-targets --keep-going -D warnings` | 0 avisos |
| `cargo fmt --check`, `check-standalone-optional`, `check-workflow-packages`, `cargo machete`, `recontar.py` | verdes |

Depois dessa bateria só entrou a F18 (`6c61e8514c`, só teste, `ph2d-panel-registry-init`): nextest por nome 5,7–6,0 s em 4 corridas, 132/132
nos módulos de `pinta`, clippy e fmt da crate verdes; `cartoes_mao.rs` é partilhado por outros testes de `pinta`, e esses correram.
Binário `target/smoke/ph2d-host-desktop`, 2.ª saída do build, sobre `6c61e8514c` (só docs depois dele):

```
▸ linha line_novaui · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.30s
```

Perfil do agente (`agent-loop-profile.sh`, 20 sessões): `cargo test : cargo check` 5,2× (alvo ≤ 1) e contexto 265 mil por passo (alvo ≤ 250 mil), ambos fora; os outros 4 dentro.

## Smoke do dono

O texto do app é em inglês. Só o que você ainda não viu. Em todos, o comando é o mesmo; sem variável abre o app normal:

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-NovaUI && ./target/smoke/ph2d-host-desktop`

(Para o Inspector com um retângulo: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-NovaUI && PH2D_VEC_CARTOES_SMOKE=3 ./target/smoke/ph2d-host-desktop`.)

**a. A linha da multisseleção voltou**
1. Crie duas ou três formas. Com a seta de seleção, aperte o botão esquerdo num ponto vazio do canvas e arraste por cima delas, sem soltar.
2. Um retângulo azul segue o rato enquanto arrasta; ao soltar, as formas tocadas ficam selecionadas.
3. Deu errado se não aparecer retângulo nenhum. (Com a cena a correr, com moldura azul em volta, o clique é do jogo e não há retângulo: isso é esperado. A aba Nodes liga isso sozinha.)

**b. Campo numérico**
1. No Inspector, em Scale ou Position, clique no ícone do campo e digite um número; depois dê duplo clique no ícone.
2. Clicar abre a escrita; o duplo clique repõe o valor de fábrica e deixa escrever. Na Timeline, clique no campo «Dur (s)» e aperte a tecla ↓ várias vezes: não passa de zero. Arraste o ícone de «Dur (s)» para a esquerda: também pára em zero.
3. Deu errado se o que digitou não entrar no campo, ou se Dur mostrar um número negativo.

**c. Scale com a corrente fechada**
1. Num objecto, feche a corrente do Scale, mude um eixo, Ctrl+Z. Depois apague o objecto e desfaça.
2. A proporção volta e a corrente continua fechada.
3. Deu errado se a proporção virar 1:1 ou a corrente abrir.

**d. Eixo a zero**
1. Ponha Scale X em 0 e puxe a alça da forma no canvas.
2. O eixo abre de novo.
3. Deu errado se ficar preso em zero.

**e. Stroke e listas com o Inspector à frente**
1. Abra o Stroke de uma forma e uma lista de opções; role o painel do traço.
2. Abrem por cima, inteiros, com barra de rolagem.
3. Deu errado se ficarem atrás do Inspector ou cortados.

**f. «+ Track» e menu `⋯` com a fila embaixo**
1. Na Timeline, clique em «+ Track» e no `⋯` da barra, com a fila de ferramentas na parte de baixo.
2. As listas abrem para o lado certo, sem sair da janela.
3. Deu errado se cortarem ou cobrirem o botão.

**g. Remove Background inteiro**
1. Selecione uma imagem e olhe a fila de Edit.
2. O nome aparece completo: «Remove Background».
3. Deu errado se ler «Cutout» ou vier cortado.

**h. Painter**
1. No painel Brush, abra a lista «Preset» e arraste um nome para outro lugar. Depois, ache a lista «On switch» e escolha «Keep volumes and mode». Olhe o visor do «Tilt» (seção Pen).
2. O nome voa com a mão, os outros abrem espaço e uma linha rosa mostra onde cai; «Keep volumes and mode» aparece inteiro; a grade do visor do Tilt se vê.
3. Deu errado se o preset pular, o texto cortar ou a grade sumir.

**i. Véu do Add Object**
1. Abra Add Object.
2. A janela inteira escurece, sem faixa clara.
3. Deu errado se sobrar uma faixa.

**j. Bones**
1. Abra o painel Bones com um osso selecionado e procure «Reach» e «Handles».
2. Ficam na mesma linha, e a escolha de Handles diz «Manual | Chain» inteiro.
3. Deu errado se ficarem em alturas diferentes ou com texto cortado.

**k. Sprite com cartão preso**
1. Selecione um sprite, solte e prenda os cartões.
2. 4 cartões têm alfinete (Render, Color, Slice, Sampling); Imagem continua sem.
3. Deu errado se faltar o alfinete ou se escrever noutro objecto.

**l. Widget Gallery**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-NovaUI && PH2D_GALLERY_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. Procure os cartões **Tool Bar** e **Floating List**.
3. Deu errado se nome e atalho ficarem abaixo do ícone, ou se faltarem.

**m. Imagens**
1. Ponha um sprite pequeno e aumente o zoom; depois Equalize e Text on Path.
2. Sem risco de 1 px na borda; Equalize com Upscale e Rasterize legíveis; Text on Path com Offset e Side visíveis.
3. Deu errado se aparecer risco na borda ou texto cortado.

**Já curados, ainda não vistos por você:** Scale travado, Weight preso, pulos no Ctrl+Z, estrela do Preset, capricho.
Se algum voltar, diga qual.

**Decida quando puder** (mantive como está): a lista D, §10 acima, em palavras: nomes estranhos (`Jetty`, `+M`, `DTCG`…),
o véu do Add Object, o «Remove» da lista de animação, a arte que encosta na borda e o campo Filter Clip.
