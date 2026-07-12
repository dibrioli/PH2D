# HANDOFF — Linha FLIP, próximo agente (COMECE AQUI)

> Você vai continuar a implementação da linha **FLIP** (o 4º meio do PH2D: animação
> quadro-a-quadro, port 2D clean-room do Grease Pencil do Blender — ADR-0113).
> Leia este arquivo INTEIRO antes de tocar em código. O tracker exaustivo do que já
> landou é [`HANDOFF_flip_impl.md`](HANDOFF_flip_impl.md) — leia depois deste.
> **Sua PRIMEIRA TAREFA é o §3 (bug da rasterização do traço).**

---

## 1. Como funciona o Modo L (leia com atenção — é o seu contrato)

Você opera em **Modo L** (linhas paralelas por worktree, [ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)/[ADR-0107](architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md); guia do operador: [`GUIA_JORNADA_MODO_L.md`](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md), protocolo: [`DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md) §1.5). O essencial:

- **Você tem um worktree próprio:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP`, branch `line/FLIP`. Índice e HEAD próprios → **colisão de commit não existe**. NÃO precisa de `git add -- <paths>` defensivo nem `git status` antes de stage (isso é do Modo C). Pode commitar à vontade.
- **Fast mode o dia todo:** `git commit --no-verify -m "..."` (instantâneo). **ZERO `git push`, ZERO CI, ZERO monitorar run.** O inner loop é **só `cargo check -p <crate>`** (ou `scripts/cargo-check-narrow.sh <crate>`). Teste/clippy/auditoria só **1× no fechamento do módulo**.
- **Isolamento:** edite a pasta do seu módulo. **Foundational (crates compartilhadas) você PODE e DEVE tocar** com cuidado — a integração roda um gate da árvore combinada + Mergiraf funde o resíduo textual. Ao criar foundational novo, projete pra isolamento (módulo irmão / extensão append-only). **PARE e reporte ao Enio** só em 2 casos: (a) **contrato congelado** (CLAUDE.md §6 — nodes/tools/vector data-model; exige ADR) ou (b) **rebase conflitando de mesmo-símbolo fora dos seus arquivos** (DIRETRIZ §1.5.5).
- **Você NÃO integra e NÃO faz push. NUNCA.** Ao terminar, você **fecha a linha, escreve o handoff de integração ([DIRETRIZ §1.5.9](IntegracaoMultiAgente/DIRETRIZ.md)) e PARA.** A integração + ship é **Enio-only**, por ordem EXPLÍCITA, via um agente integrador dedicado. Integrar/pushar por conta própria = violação do protocolo.
- **Padrão-ouro sem adiamentos:** a melhor opção técnica vence custo de build/cronograma; gaps in-scope fecham na sessão. Mas veja o §3 abaixo — a lição desta linha é **iterar às cegas queima**; use os testes GPU como oráculo antes de declarar vitória.
- **Memória viva:** leia `project-memory/MEMORY.md` (índice) antes de agir; em especial [`project_flip_module_grease_pencil_2d.md`](../project-memory/project_flip_module_grease_pencil_2d.md) e [`project_flip_stroke_analytic_coverage_gp.md`](../project-memory/project_flip_stroke_analytic_coverage_gp.md).

**Gates ao fechar o módulo** (1× sobre o diff acumulado): `rustup run 1.95 cargo fmt -p <crates> --check`; `cargo clippy -p <crates> --all-targets`; os testes das crates + os arch-gates do shell (`file_loc_caps` = 600 LOC/arquivo, `no_tofu_glyphs` = zero `→` em string literal, `node_id_collisions`, `architecture_no_downcast_to_concrete_tool_in_shell`, `architecture_no_per_tool_branch_in_render_loop`). NÃO rode `ship.sh` nem push.

---

## 2. Estado da linha FLIP (resumo; detalhe em `HANDOFF_flip_impl.md`)

Já landou e funciona (com o bug do §3 no render do traço):
- **W0** — modelo de documento: `ph2d-flip` (`FlipDoc → objects → layers → frames(BTreeMap) → FlipDrawing(refcount) → FlipStroke` SoA pos/width/opacity/color por-ponto). Cada objeto é uma **entidade ECS** (`FlipObjectRef`, ponte em `shells/desktop/src/flip_entities.rs`).
- **W1** — render GPU: `ph2d-flip-render` (rasteriza o traço) + `shells/desktop/src/render_loop/flip_pass.rs` (compõe camada-a-camada pelo compositor 22-modos do Painter, cache de tesselação em `flip_pass_cache.rs`).
- **W2** — desenho/painel/borracha: `ph2d-tool-flip` + `ph2d-panel-flip` + `shells/desktop/src/flip_draw.rs`/`flip_erase.rs`/`flip_layers.rs`. Painel docado (Brush/Layers), 14 controles, blend por-camada (dropdown), borracha 3 modos.
- **Select/gizmo (ADR-0111 parity):** o objeto Flip é selecionável (Hierarquia ou canvas) e movido/girado/escalado pelo **gizmo de sprite** — geometria LOCAL + `Transform`, `flip_transform.rs` (settle do pivô) + `flip_gizmo_view.rs` (picking) + `flip_pass::fold_model` (render aplica o model). Draw/erase localizam na fronteira world→local.
- **Brush ABSOLUTO:** a largura é px de TELA (não escala com o zoom); `camera_raw` passa escala de espessura 1.0, `fold_model` × `mean_scale` do objeto.
- **Aberto (fora do §3):** W3 (Frames/Ghost Frames/Tween); **Edit Mode / seleção de traço individual** (o "select do traço" que o Enio pediu — é um pacote próprio, tipo GP Edit Mode); `vec_save` não serializa a pose Flip.

---

## 3. PRIMEIRA TAREFA — rasterização do traço (o bug a resolver)

### O sintoma atual
O traço não está **liso e uniforme como o Grease Pencil**. Especificamente (última smoke do Enio): **com hardness baixo, as QUINAS afiadas acumulam cor** (spikes/estrelas radiando da bissetriz). Um agente anterior (eu) tentou 6 abordagens e cada uma consertava um artefato e criava outro. **A linha está revertida para o estado "fita conectada por miter" — liso nas curvas, mas com spike nas quinas afiadas.** Reproduza desenhando um zigzag com hardness < 1.

### A matriz de trade-offs já explorada (NÃO repita estes becos)
Arquivo: `crates/ph2d-flip-render/src/shaders/flip.wgsl` (vertex + fragment) + o depth-state em `pipeline.rs`.

| Abordagem | Curva lisa? | Spike na quina | Bead (junção macia) | Acúmulo de cor | Cruzamento |
|---|---|---|---|---|---|
| `v_perp` por-quad (v1) | ~ | sim (miter) | — | — | notch duro |
| analítica + GEQUAL + **stadium** | sim | algum | **BEAD** | sim | novo por cima |
| analítica + GEQUAL + **fita conectada** (ESTADO ATUAL) | **sim** | **SIM (bowtie)** | não | no bowtie | novo por cima |
| analítica + **GREATER estrito** + **stadium** | **NÃO (escamado!)** | não | não | não | velho por cima |

**Lições cruciais (verificadas em GPU):**
1. **Cobertura ANALÍTICA (distância do pixel à linha-de-centro no fragment), não `v_perp` por-quad.** O `v_perp` distorce nas junções. Isto está CERTO no código atual e deve permanecer. Ref: `gpencil_stroke_segment_mask` em [`draw_grease_pencil_lib.glsl`](file:///home/enio/Downloads/blender-5.2-grease-pencil-ref/source/blender/draw/intern/shaders/draw_grease_pencil_lib.glsl).
2. **O spike na quina = a fita conectada por miter DOBRA sobre si numa quina afiada** (bowtie = triângulo invertido/auto-sobreposto) e o premult-over acumula ao longo da dobra. (O Enio suspeitou das "normais da face" — era isso.)
3. **`GREATER estrito` + `stadium` (quads independentes) fica ESCAMADO** (corrente de ovais) porque o descarte por depth isola cada segmento — cada um mostra sua tampa redonda. **NÃO use stadium com GREATER estrito.**
4. **O estado de depth do GP 2D é `WRITE_DEPTH | BLEND_ALPHA_PREMUL | DEPTH_GREATER` (estrito), depth por-stroke crescente com o sid** ([`gpencil_cache_utils.cc:449`](file:///home/enio/Downloads/blender-5.2-grease-pencil-ref/source/blender/draw/engines/gpencil/gpencil_cache_utils.cc)). O GREATER estrito **descarta** a 2ª face no mesmo pixel → sem acúmulo. **Mas só funciona sem escamar se a geometria for uma FITA CONECTADA** (segmentos adjacentes NÃO se sobrepõem; só cruzamentos reais se sobrepõem).

### A abordagem que NINGUÉM tentou (o caminho recomendado)
Combine o que cada round acertou, faltando UMA peça (o **bevel** nas quinas afiadas):

> **Fita CONECTADA + BEVEL nas quinas afiadas (miter_break) + `DEPTH_GREATER` estrito + write-depth + fragment analítico.**

- **Fita conectada** (já no código): segmentos adjacentes compartilham o vértice de junção → sem escama, sem bead.
- **`DEPTH_GREATER` ESTRITO** (troque o `GreaterEqual` atual em `pipeline.rs`): nos cruzamentos reais (não-adjacentes) descarta a 2ª face → sem acúmulo. Como a fita é conectada, adjacentes NÃO se sobrepõem → NÃO escama (essa é a diferença crucial do beco #3, que usava stadium).
- **BEVEL nas quinas afiadas (a peça que falta):** quando o ângulo excede o limite, NÃO mitre — **quebre a quina** (bevel) para a geometria não dobrar (fim do bowtie/spike). É o `miter_break` do Blender (`gpencil_vertex` em `draw_grease_pencil_lib.glsl`, ~linha 705: `bool miter_break = cos_angle_adj > miter_limit; miter_tan = miter_break ? line : ...; screen_ofs += line * x`). Porte-o fielmente.
- **Fragment com p0/p3 (refino):** o `gpencil_stroke_segment_mask` do Blender passa os VIZINHOS p0/p3 ao fragment pra tratar a distância na quina (round/bevel/miter por tipo). Hoje o fragment usa só p1/p2. Adicionar p0/p3 dá a quina exata e evita costura entre segmentos que o GREATER estrito pode deixar. Porte a função inteira.

**Não confie na minha prescrição cegamente** — foi o que me queimou 6×. **Use os testes GPU como oráculo** (rodam nesta máquina!) e valide CADA mudança antes de declarar. A regra-mãe da linha: *verde-de-compilação vale zero; só o pixel no teste GPU conta*.

### O harness de teste GPU (sua maior alavanca — rode SEMPRE)
Os testes rasterizam num alvo offscreen 64×64, leem os pixels e afirmam o comportamento. **Rodam nesta máquina Linux** (tem adapter Vulkan; não precisa de Mac/Metal):
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && cargo test -p ph2d-flip-render --test gpu_render -- --ignored --nocapture
```
Arquivo: `crates/ph2d-flip-render/tests/gpu_render.rs`. Já tem: banda reta, falloff de hardness, cruzamento, fill-sob-traço, junção redonda, **sem-bead numa reta**. **Escreva testes novos pro seu fix** (ex.: `a_sharp_corner_does_not_accumulate_color` — um pixel na sobreposição da quina ≈ um pixel single-coverage na reta; e um teste que uma CURVA lisa não escama — alpha uniforme ao longo da linha-de-centro). Composição: `--test composite_blend`.

**Smoke visual do Enio** (o teste final; você não consegue GUI aqui, então entregue e peça):
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && cargo run -p ph2d-host-desktop --release
```
Desenhe zigzags/curvas com hardness alto E baixo; cruze o traço consigo mesmo. O alvo é: **liso como o GP, sem escama, sem bead, sem spike, sem acúmulo**, em qualquer hardness.

### Arquivos que você vai tocar
- `crates/ph2d-flip-render/src/shaders/flip.wgsl` — vertex (geometria da fita + bevel) + fragment (cobertura analítica + p0/p3).
- `crates/ph2d-flip-render/src/pipeline.rs` — o depth-state (`fn depth_greater_equal` → GREATER estrito; renomeie).
- `crates/ph2d-flip-render/tests/gpu_render.rs` — testes novos.
- (talvez) `crates/ph2d-flip-render/src/pack.rs` — se precisar passar mais atributos por-ponto (tipo de quina).

### Referência do Blender (LEIA — é a fonte da verdade, GPL → só comportamento, clean-room)
`/home/enio/Downloads/blender-5.2-grease-pencil-ref/source/blender/draw/`:
- `intern/shaders/draw_grease_pencil_lib.glsl` — `gpencil_vertex` (geometria + miter_break/bevel) e `gpencil_stroke_segment_mask` (cobertura analítica com p0/p3 + tipos de quina round/bevel/miter) e `gpencil_stroke_hardess_mask` (o perfil `pow`+smoothstep).
- `engines/gpencil/gpencil_cache_utils.cc:447-449` — o DRWState do passe de traço (WRITE_DEPTH + BLEND_ALPHA_PREMUL + DEPTH_GREATER, depth por-stroke crescente).

---

## 4. Como fechar e entregar

1. Rode os gates (§1, "ao fechar"). Corrija todo `✗`. LOC: se `flip.wgsl`/`pipeline.rs` crescerem, WGSL não tem cap; os arquivos Rust do shell têm 600 (split em módulo-irmão, não allowlist).
2. Atualize `HANDOFF_flip_impl.md` (o tracker) com o que fez + resultados dos testes GPU.
3. Se o Enio pedir, escreva o **handoff de integração** (DIRETRIZ §1.5.9): símbolos novos grep-áveis, foundational tocado, contratos encostados, ordem de dependências, o que só o `ship.sh` pega. Veja a §3 do `HANDOFF_flip_impl.md` como modelo (já lista os símbolos das crates Flip).
4. **PARE.** Não integre, não pushe, não rode `ship.sh`. Reporte o commit local pronto + o link do smoke e espere ordem do Enio.

> Docs de planejamento (`docs/Flip/`, ADR-0113, memórias) estão **untracked na árvore primária** — NÃO os commite nesta linha (quebra o `merge --ff-only` da integração). O Enio os commita por fora.
