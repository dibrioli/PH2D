# HANDOFF de integração — `line/Painter` (2026-07-12)

> **Para o agente integrador.** A linha está **FECHADA**. Não integrei, não pushei, não rodei ship —
> isso é ordem explícita do Enio (CLAUDE.md §0.7 · DIRETRIZ §1.5.9).
> Worktree: `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` · branch `line/Painter`
> **33 commits** sobre `main`, 74 arquivos, +9855/−2550.

---

## 0. ADENDO (mesmo dia, dono novo) — Fase 4: o CORPO (`482c0696` · `2c3492ab`)

Após o veredito do Enio (*"não sei se melhorou ou piorou; ficou mais difícil de ajustar"*), a linha
trocou de dono, fez **pesquisa nova** (5 varreduras de fontes primárias —
[`docs/Painter/17_impasto_deposito_pesquisa2.md`](Painter/17_impasto_deposito_pesquisa2.md)) e
**redesenhou o modelo de depósito** (decisões + divergências medidas: plano 16 §10/§10.1):

- **Curva de corpo** no kernel (`body_profile`, `height.rs`): platô + parede DENTRO da tinta
  pigmentada (o bevel *inner* do PS); o véu translúcido não carrega relevo. O domo morreu.
- **Inclinação física** (`DEPTH_UNIT_PX = 16`): morrem `SLOPE_GAIN` e o mute quadrático por
  cobertura; o glint ganhou curva própria (`gloss_body` — specular só no filme).
- **O knob `Amount` (canvas) MORREU** — era o gêmeo acoplado do Depth. Superfície final: brush
  `Enable/Depth/Smoothing/Source/Draw To` · canvas `Show/Angle/Elevation/Shine`. O id
  `PAINTER_IMPASTO_LIGHT_AMOUNT` saiu de `PAINTER_IMPASTO_FIELDS` (6→5) — **desmonte, não append**:
  se outra linha referenciar esse id, é colisão real (nenhuma deveria).
- **Teto de vidro** no commit de traços (`H_CEIL = 2.0`, "pressed against glass" do Painter).
- **Dívida herdada fechada:** o gate de LOC do workspace já estava **vermelho no HEAD desta linha**
  (`paint.rs` 712/700) — `union_region` movido para o irmão `region.rs` (697/700).
- Gates: **584** `ph2d-tool-painter` (2 novas com RED por mutação: body-with-an-edge + glass
  ceiling; halo/does-not-shade/corrugação re-derivadas — corduroy ViewPlane 1.0→0.70, atenuado) +
  **239** brush; clippy `--all-targets` 0; perf **1.79 ms/move** (alvo ≤4).
- **Fase 4.1 (pós-smoke, mesmo dia):** o Enio aprovou o corpo mas apontou a perda do arredondado —
  a curva esmagava todo falloff. Nasceu o dial **`Body`** (`impasto_body`, 0..1, default 1.0 = look
  da Fase 4; **0 = o relevo obedece a silhueta por inteiro**, o domo redondo sob a luz nova; meio =
  família mesa). É o `Technique` do PS como contínuo. `PAINTER_IMPASTO_BODY` entrou em
  `PAINTER_IMPASTO_FIELDS` (5→6, append). Gate com RED por mutação:
  `impasto_body_zero_obeys_the_falloff`. Perf 1.87 ms/move. Plano §10.2.
- **Fase 4.2 (2ª ordem do smoke): TODO parâmetro do relevo é VIVO.** O traço deixou de guardar a
  ALTURA e passou a guardar os **ingredientes** (`stroke_paint` f32 + `stroke_grain` u8); o relevo é
  sempre `derive_height(spec, paint, grain)` — uma função pura, a MESMA no depósito e na edição. Logo
  **Depth · Body · Depth Source · Smoothing** editam o último traço ao vivo, sem caso especial.
  Envelope passa a ser na TINTA (grandeza que nenhum setting muda) e o perfil roda na tinta
  (`w × dinâmica`), alinhando geometria e luz. `Draw To` segue não-vivo **de propósito** (é roteamento
  de canal, não propriedade da tinta — a metade da cor é irreversível). Gate
  `impasto_every_body_knob_edits_the_last_stroke_live` (RED em 2 mutações) + o unit de pureza no
  kernel. `height.rs` 761→511 (testes p/ `height_tests.rs` — **arquivo NOVO**); `paint.rs` 700/700.
  Perf 1.66 ms/move. Plano §10.3.
- **Fase 4.3 — `Shine` estava MORTO** (3ª rodada do smoke). Causa **minha, da Fase 4**: o relevo só tem
  declive na faixa `W_TAIL..W_SOLID` (a parede) e eu tinha gateado o glint **acima** de `W_SOLID` — só
  no platô, que é plano. Medido: 94% dos pixels com declive ficavam fora do gate; Shine 0→1 movia
  **1 nível**. Fix duplo: o glint usa a **mesma curva de corpo do difuso** (1 → **160 níveis**) e o
  highlight vira **screen** (`lit + add·(1−lit)`) em vez de aditivo puro — o aditivo plano ressuscitava
  o halo pelo canal saturado (gate do halo vermelha em 19%). Gate novo com **3 claims, 3 vermelhos por
  mutação**: `impasto_shine_glints_on_the_wall_without_bleaching_the_rim`. Perf 1.67 ms/move. Plano §10.4.
- **Defaults do artista + BUG do relevo por documento (Fase 4.4).** Defaults (ordem do Enio): brush
  `Depth 1` · `Body 0` · `Smoothing 1`; canvas `Angle 230°` · `Elevation 30°` · `Shine 0.7` — nos 4
  sites espelhados; o smoke passou a NÃO re-armar Depth/Smoothing (mostra o pincel default de verdade).
  **Bug meu, da Fase 1, achado no caminho:** `StashedDoc` não levava `heights`/`covers`, e os
  `RtLayerId` colidem entre documentos ⇒ trocar de sprite **perdia** a escultura, e voltar a um sprite
  cacheado fazia o relevo do anterior **iluminar** o novo (espécie do #13.c). Fix + gate
  `relief_travels_with_its_document_and_is_never_lent_to_another` (duas barreiras, defesa em
  profundidade — provado por mutação). **Gate do halo reescrita** para a propriedade indefensável
  (tinta sem corpo não recebe luz **nenhuma**, em qualquer setting) — a métrica de croma antiga não
  distinguia luz honesta de halo sob `Body 0`; e o screen ganhou **gate algébrico exato**
  (`screen(R)−screen(G) = (R−G)(1−add)`), sem limiar de imagem. Perf 1.94 ms/move. Plano §10.5.
- **PERSISTÊNCIA do documento pintado (Fase 5) — a pintura sobrevive ao `Ctrl+S`.** Antes: sprite
  pintado = `Individual{texture_id}` (id de runtime da GPU) ⇒ pintar+salvar+reabrir devolvia o quadro
  **em branco**. Agora: componente **NOVA `ph2d_ecs::PaintedDoc(u32)`** (módulo irmão `painted_doc.rs`,
  registrada no `ComponentRegistry` 26→**27**) dá identidade estável; `PaintedDocument`
  (`ph2d-tool-painter/src/tool/persist.rs`) leva **camadas + pixels + relevo + cobertura** para o
  arquivo (`LayerImage` ganhou serde); o load re-instala, compõe **pelo caminho normal do preview**
  (sem segundo bake) e re-materializa a textura individual. `PROJECT_SCHEMA` 2→**3**. Shell:
  `project_painter.rs` (NOVO) + 3 linhas em `project.rs` + `next_painted_doc` no `AppGfx`. Gate
  `a_painted_document_survives_the_disk_with_its_relief` (postcard REAL, entidade com bits diferentes).
  **⚠️ Foundational tocado:** `ph2d-ecs` (componente nova, append; `registry.rs` +1 linha e o gate de
  contagem 26→27) — anotado para o integrador. Plano §10.6.
- **PLOW — a espátula (Fase 6).** O Smear arrastava a cor e deixava o corpo: tinta grossa era imexível
  depois de pousada. `plow_dab_height` é o **mesmo lift-and-drag** do smear de cor, sobre `h` **e**
  `cover`, consumindo o **mesmo `last_smear_pos`** (roda ANTES de a cor avançar a cadeia) — pigmento e
  corpo movem-se como uma coisa só. Deslocamento, nunca depósito: não usa Depth nem o master switch;
  default `Plow=0` (Smear byte-idêntico). UI: no Smear o card Body **some** e entra o card **Knife** com
  1 linha — não Body esmaecido (o que não se aplica não é pintado). `PAINTER_IMPASTO_PLOW` em
  `PAINTER_IMPASTO_FIELDS` (6→**7**, append). Gates com RED por mutação: relevo arrastado + **cobertura
  junto** + a matriz virou exclusividade (Paint⊕Smear). Perf 1.99 ms/move. Plano §10.7.
- **COMPOSITE DEPTH POR CAMADA (Fase 7) — o relevo vira parâmetro de composição.** O Depth do pincel é
  **assado em cada traço** quando ele pousa e o re-derive vivo alcança **só o último**: na 2ª pincelada a
  espessura da 1ª estava congelada, e **nada no produto voltava a tocá-la**. O Depth de CAMADA alcança —
  composita, não re-esculpe, então age sobre **tudo que já foi esculpido ali, para sempre** (`0` muda, não
  apaga; o gate sobe de volta e exige a escultura **bit a bit**). Mora na **linha 3 da row da camada**, ao
  lado da opacidade, no mesmo formato — e **só em camadas que têm relevo** (`Layer::has_relief`, projetado
  pelo tool do mapa de alturas; documento não-esculpido não mostra chrome de impasto em lugar nenhum).
  **Os 4 modos do plano viraram 2:** `Add`/`Subtract`/`Ignore` são três leituras de **um número com sinal**
  (`+` empilha, `0` muda, `−` cava) — enum que duplica slider é o segundo-ganho que esta seção já matou uma
  vez (o antigo "Amount"). Sobra o único que a escala não diz: **`Level`** (a tinta opaca desta camada
  **soterra** a textura debaixo, pesada pela própria cobertura — o *"composite, don't add"* da pesquisa).
  E `Level` **não comuta**: o fold da altura passa a caminhar a **ordem-z** (`z_order_bottom_up`), com o
  traço vivo **no slot da camada ativa**. Enquanto era soma pura ele iterava o mapa em ordem de **chave** e
  ninguém via. **5 mutações provadas vermelhas** (fold sem `depth` · `Level`=`Add` · flag não publicado ·
  fold em ordem de chave · **revisão não bumpada** — o flag certo e o painel nunca sabendo, porque pincelada
  é edit de *pixel*, não bump de revisão: um teste que só lê o flag fica **verde** com esse bug).
  `PainterLayerWidget::ALL` 23→**25** (append). `PROJECT_SCHEMA` 3→**4** (postcard é posicional; o `Layer`
  ganhou 3 campos). Perf **2.35 ms/move** (alvo 4, kill 8). Plano §10.8.
- **⚠️ VERMELHO LATENTE MEU, PAGO AQUI:** os gates de contagem de componentes ECS de **`ph2d-render`** e
  **`ph2d-script`** estavam **quebrados desde `0a90ed31`** (a persistência registrou `PaintedDoc`: 27→28) e
  **a linha reportou verde**. `nextest-impacted` não os toca — só `cargo test --workspace` pega. Ambos
  corrigidos neste commit; a lição já estava catalogada ([`feedback_ship_parity_gaps_ci_only`]), e desta
  vez ela cobrou. **Integrador: rode o workspace inteiro, não o impacted.**
- **FIX do smoke: "smoothing nem sempre se aplica no fim do traço" (Fase 8).** A palavra era ***nem
  sempre***. O settle roda num único lugar e **incondicionalmente** — a aritmética nunca foi a suspeita; o
  que varia é **se o commit roda**. As cinco tools de **FORMA** (Line·Arc·Ellipse·Polygon·**Free Hand**)
  mantêm o traço **ABERTO** no pen-up de propósito (a forma segue editável até o Apply), então
  `close_stroke` — e com ele `commit_stroke_height` — **nunca disparava para elas**. Três consequências, e
  o Smoothing era só a visível: (1) Smoothing morto nas 5 formas (a luz lia o **envelope cru**); (2) o card
  Body **inteiro** morto nelas (os ingredientes nunca eram entregues); (3) **pior, e medido:** o relevo
  ficava em `stroke_height` **sem dono** e o próximo pen-down o **apagava** — aplique uma curva, comece
  outro traço, e a espessura da primeira **evaporava** (o pigmento ficava, o corpo não). Fix em **dois
  chokepoints, não cinco call-sites**: `commit_drag_preview()` (onde um desenho vira canvas) passa a comitar
  o relevo — um ponto, todos os métodos, Apply **e** Apply & Keep; e `cancel_open_shape()`/
  `discard_open_shape()` **largam o envelope** (Esc devolve os pixels ao pristino ⇒ crista sem tinta é o
  fantasma que o gate da borracha já recusa, entrando pela tecla Esc). Gate principal = **tabela sobre os 10
  métodos de traço** — o bug nunca esteve no código escrito, esteve nos **caminhos que ninguém conectou**;
  uma 6ª forma sem commit fica vermelha. 2 mutações provadas vermelhas. Plano §10.9.
- **Smoke do Enio: PENDENTE** (validar: os defaults novos · **Shine** acende a crista · girar
  Depth/Body/Source/Smoothing DEPOIS do traço e ver o relevo mudar ao vivo · pintar relevo, trocar de
  sprite e voltar — a escultura tem de estar lá · **pintar, Ctrl+S, fechar, reabrir, Ctrl+O — a pintura
  E o relevo têm de voltar, ainda editáveis** · **Smear com Plow: passar a faca num traço grosso e ver o
  relevo ser ARRASTADO junto com a cor** · **Depth de camada: esculpir DOIS traços, e no painel de Layers
  arrastar o Depth da linha 3 da camada — os DOIS relevos têm de responder ao vivo (o pincel só alcançava o
  último); no negativo a crista vira sulco; o chip `Add`/`Level` numa camada de cima tem de soterrar a
  textura debaixo**). Comando do §5 inalterado; card Lighting 4 linhas, card Body 5, card Knife (só no
  Smear) 1; a linha de Depth só aparece em camadas COM relevo.

Os §§ 1–8 abaixo descrevem as entregas anteriores da linha e continuam válidos; números de gates
ficam superseded pelos desta seção.

---

## 1. O que a linha entregou, em uma frase cada

1. **Varredura do Painter** (9 achados, 8 corrigidos) — nasceu de um SIGSEGV e virou a descoberta de que o
   bug era uma **espécie**, não um caso: todo *guard* de reúso perguntava *"já inicializei?"* em vez de
   *"este dado ainda pertence às entradas que o produziram?"*.
2. **Impasto (#16)** — traço com aspecto 3D: canal de altura + passe de luz, integrado a **tudo** que o
   Enio listou (Shape, Grain, ramps, Stroke, shapes dinâmicas, Tiling, Mirror, Jitter, Per-Layer Color) e
   **escondido** onde não se aplica. **Watercolor não foi tocado.**

---

## 2. Estado dos gates (rodados no fechamento, worktree limpa)

| Gate | Resultado |
|---|---|
| `ph2d-painter-brush` | **239** passed |
| `ph2d-tool-painter` | **575** passed (+17 ignored: GPU/perf) |
| `ph2d-panel-painter-layers` | **40** + **20** seam |
| `ph2d-editor-core` (33 suítes de arquitetura) | verde — incl. `no_magic_numeric`, LOC caps |
| `cargo clippy --all-targets` (3 crates + shell) | **0 warnings** |
| `cargo fmt --all --check` | limpo |
| `cargo build --release -p ph2d-host-desktop` | ok |

**NÃO rodei `ship.sh`.** A memória [`project_integrator_ship_catches_latents_budget_iterations`] avisa: o
gate por-linha **não roda** fmt-workspace / clippy-all / machete / deny / typos — **orce 2-4 iterações** de
ship vermelha. Os prováveis: `machete` (a shell ganhou uso novo de `ph2d-tool-painter`), `typos` (os
commits e docs têm pt-BR).

---

## 3. Superfície tocada (para prever conflito de merge)

**Foundational tocado** (Modo L permite; anotado conforme
[`feedback_foundational_editable_design_for_isolation`]):

- **`ph2d-editor-core`** — **só append**: `ids/chrome/painter_impasto.rs` (arquivo NOVO) + 2 linhas em
  `ids/chrome/mod.rs`. Nenhum id existente mudou. Colisão só se outra linha inserir no mesmo ponto do
  `mod.rs` — resíduo textual trivial.
- **`ph2d-painter-brush`** — `height.rs` (NOVO) · `spec.rs` (+campos `impasto_*`, **append**; testes inline
  extraídos p/ `spec_tests.rs` pelo teto de LOC) · `dab.rs` (silhueta extraída p/ `silhouette_at`, chamada
  pelo kernel de cor — **comportamento byte-idêntico**, 239 testes provam) · `texture.rs`
  (`rotate_by_degrees` virou `pub`).
- **`shells/desktop`** — `impasto_smoke.rs` (NOVO) + 1 guard em `painter_gpu_preview.rs` + 3 linhas em
  `painter_bridge.rs` + 1 campo em `app_state.rs`.

**Contratos congelados (§6): NENHUM tocado.** `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`
intactos — o Impasto anda pelo `PanelEvent` existente.

---

## 4. ⚠️ O único ponto que exige olho no merge

`crates/ph2d-panel-painter-layers/src/paint_watercolor.rs` teve `card_frame`/`card_row` **extraídos** para
`card.rs` (**movimento puro**: −92/+1, só remoções + um import). Se outra linha editou esse arquivo, o
Mergiraf pode se confundir. **A óptica da aquarela não foi tocada** — nenhuma linha de física/render.
Confira com: `git diff main -- crates/ph2d-panel-painter-layers/src/paint_watercolor.rs`.

---

## 5. O que o Enio ainda não viu (smoke pendente)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```
Canvas branco 1024² **já selecionado**, pincel **já armado** (Depth 0.7, source Grain). **Pegue o Painter e
arraste.** Card **Lighting** move a luz ao vivo; **Depth negativo CAVA**.

Também pendentes de smoke (da varredura): os 8 fixes — em especial **Rake de Shape com Per-Layer Color +
traço freehand**, que era o SIGSEGV original.

---

## 6. Aberto e DELIBERADO (não é esquecimento)

| Item | Por quê |
|---|---|
| **Watercolor OFF→ON no meio do traço** apaga tinta | **Não consegui construir um RED** — o dab plano não chega a pintar no harness. Escrevi o fix e **revertí**: sem vermelho refutável não se mexe, menos ainda na aquarela que o Enio mandou não ferir. BUGS #13. |
| Gates de paridade banda-vs-serial são **dependentes de máquina** | Num runner de 1 core comparam serial com serial. |
| ~~`Plow`~~ · ~~Composite Depth por camada~~ · ~~persistência do `h`~~ | **FECHADOS** (§0, Fases 5–7). |
| Luz na GPU · conservação de volume real da faca · múltiplas luzes / IBL | Fora do 1º corte, **nomeados** (plano §10.8). A perf não pede a GPU: 2.35 ms/move contra um alvo de 4. |
| Relevo do PAPEL | **Acopla impasto↔aquarela ⇒ exige ordem NOVA do Enio** (plano §2). |

---

## 7. Docs vivos

- [`docs/Painter/16_impasto_plano_implementacao.md`](Painter/16_impasto_plano_implementacao.md) — plano +
  **§9: onde a implementação divergiu do plano, e por quê** (5 decisões, cada uma com gate).
- [`docs/Painter/15_impasto_pesquisa_e_design.md`](Painter/15_impasto_pesquisa_e_design.md) — a pesquisa.
- [`docs/Painter/BUGS_painter.md`](Painter/BUGS_painter.md) — **#12** (o SIGSEGV) e **#13** (a varredura).
- [`docs/Painter/13_fila_integracao_watercolor_secoes.md`](Painter/13_fila_integracao_watercolor_secoes.md)
  — **#19** (semântica da aquarela fixada).
- Memória nova: [`feedback_stale_comment_and_dead_code_lie`](../project-memory/feedback_stale_comment_and_dead_code_lie.md).

---

## 8. A lição que vale além desta linha

**Cinco vezes** um teste meu ficou **verde pelo motivo errado**, e sempre do mesmo jeito: a fixture não
continha o fenômeno. Cache testado em dois tools (frio nas duas vezes → re-assa sempre). Opacidade com a
camada de baixo cheia (o over-composite satura → a de cima não podia mudar nada). Smear dentro de região
uniforme (girar a elipse não muda nada). Disco duro chamado de "crista" (platô de parede vertical não tem
flanco). Papel branco usado como "tinta plana". **Em todos, foi DESLIGAR o fix que expôs o teste falso** —
e num deles o próprio assert de sanidade me pegou antes.

É por isso que cada gate desta linha tem **vermelho verificado por mutação**, não só verde.
