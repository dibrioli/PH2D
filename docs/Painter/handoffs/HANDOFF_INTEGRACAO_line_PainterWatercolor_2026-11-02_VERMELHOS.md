# HANDOFF DE INTEGRAÇÃO — line/PainterWatercolor, os vermelhos da rodada de 02/11 curados pela causa

> Complementa [`…_2026-11-01_REFATORACAO.md`](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-01_REFATORACAO.md) (escopo da refatoração do Wet Paint: leis, painel E10, crates novas, superfície de colisão de 738 ficheiros). **Aqui só o que mudou depois dele** e o que o integrador vai ver. Nada integrado, sem push.

## 0. Contexto

- A rodada de integração de 02/11 entrou no `main` **sem o Painter**: a árvore combinada mostrou ~49 vermelhos, **latentes na própria linha**.
- O integrador escreveu 12 curas nos ramos `integ/2026-11-02-pintura`, `-pintura2`, `-com-painter`; foram cherry-pickadas para a linha (commits `fix(integ 02/11 pintura*)`). **Não apague esses ramos.**
- A linha foi rebaseada em `main` `3df5894489` (merge-base = `main` hoje). Antes: `backup/line-PainterWatercolor-pre-rebase-2026-11-02`.
- Rebase verificado: 297 commits = 300 − 3 junções (r1/r2/r3, linearizadas) · `git range-diff`: 294 `=` e 2 `!` (resolução de conflito dobrada no commit) · todo ficheiro que difere do backup é um que o `main` também mudou · as linhas da linha nos 7 ficheiros partilhados estão intactas (contadas).
- Desde o fim do handoff anterior (`aec5e62af1`): **42 commits, 64 ficheiros, +820/−514** (45 em `ph2d-tool-painter`, 11 em `ph2d-wet-paint`, 5 PNG de miniaturas, 1 `ph2d-app-painter`, `.config/nextest.toml`, 1 doc). Contra o `main`: 767 ficheiros, +56 986/−2 514 (343 `docs`, 152 `ph2d-tool-painter`, 130 `ph2d-wet-paint`, 43 `assets`).

## 1. Curas do MOTOR (achadas por bisseção; SHAs de hoje)

| | cura | culpado (SHA pré-rebase) | efeito medido |
|---|---|---|---|
| a | `032cda64a5` — `trail/espalha.rs` `Espalha::deposita`: o piso do espalhar vale sobre o ACUMULADO do traço (`erguido`), não por dab | `0f32fb6e75` (Bingham) | óleo sobre óleo deixa de empilhar: Thick Impasto p99 24,1→110,9 vira 24,1→39,7; Filbert 1/3/6/10 passes 1,49/3,77/6,64/10,95 vira 1,49/2,39/2,65/3,16. Curou também `a_fenda_forte…` (ponto preto 68 < 70) |
| b | `0e6c9ecf75` — `lote::sub_passo_do_peso` nunca divide por menos de uma célula | `70869bf673` | ponta desfiada 2,14 → 1,00 |
| c | `cca667c3ec` — `LeiDaLargura::de_tabela` guarda o FIM de cada patamar; `7e9bd5ebc1` extrapola pelo último ponto que CRESCE | `7ad2967e2c` regerou tabela com patamar | Gouache Flat, traço fino 0,41 → 0,94. Regerados Colored 4B e Graphite HB (`f211ae8753`) |
| d | `5ef65fa494` — receita sem ficheiro passa pelas migrações E4+E10 | linha (E4/E10) | fios da guache seca 6,25× → 9,33× (como no merge-base) |
| e | `d53dcbb117` — Transmission Blender: o sol não chega ao papel SOB o filme (caminho cáustico; MNEE ligado/desligado dá igual); `21e80feaed` — a luz cruza a superfície DUAS vezes, peso w² | linha | Clear glass bit-idêntico (`claro = 1`) |
| f | `22df3ca22b` — o registo do undo guarda o cursor do passo em voo (`PassoEmVoo`, `grid/snapshot.rs`) e `Engine::undo/redo` repõe-no; a regra é do motor, não do `undo_vivo`. Undo que não se aplica já não drena | linha | mutações vistas sangrar |
| g | `5c84d0e8c3` + `dc66557b6f` (regerado) — Graphite 8B e Colored 4B marcam no toque leve: curva da CARGA 3→2,5 (8B) e 3→2 (4B). Dono 02/11 «Sim, marcar mais» | pedido do dono | escuro a p 0,25: 0,063→0,122 e 0,024→0,094; p 1 e largura iguais. ⛔ o piso de tamanho NÃO foi tocado (o dono recusou em 18/10) |

Também no motor: `0fb37300f8` (o registo do undo abre a meio do passo da água, sem drená-lo) e `Grid::sett_do_traco`, `material`/`oleo_luz` (foundational DESTA linha; `ph2d-wet-paint` não é partilhada).

## 2. Réguas e fixtures (medidas, não afrouxadas)

- Miolo da cena do papel texturizado = 0,633 × a mediana da massa pousada, e asserta não-vazio (`1368c97b01`). Mecanismo: a E10 dá Medium 1 aos óleos (decisão do dono) ⇒ líquido livre 0 ⇒ 3 passes levam −40 % de massa. **Não é defeito de migração** (`7a1d39ca13`).
- Tecto dos vãos do Dabber 3 no Estúdio 42 → 57 %, contra o Cycles (CPU) rodado de novo na malha de hoje (`2c90295ad6`).
- Hash do Clear glass regravado pela bisseção, 15 degraus, todos commits de lei nova da tinta (`1af935d66c`).
- `o_chato_pequeno_esmaece` piso 0,75 → 0,69 = meio do vão entre produto 0,741 e mutação 0,642 (`66857a4e58`). A queda é do Bingham `0f32fb6e75` (aprovado) + 0,001 da E10.
- Tabelas de largura do Blender Hard/Soft regeradas sob a E10 (`3ba165dcc1`). **O gerador pica em 33–35 GB: use `PH2D_MEM_MAX=64G`** (OOM em 24G).
- Régua da mão (pressão): o decil 2 mudo agora REPROVA (antes dava 99), com controlo de pressão presa a 1 (`2e39ca457b`); o piso da coluna é relativo ao topo do PRÓPRIO traço, 0,08 × topo (`68273a8e77`). O Wet Wash ficou mais claro na R1 (`99d1d5806b`, `0046ce2414`) mas responde (2,90×).
- Escada do relevo dos fios volta com tecto NOSSO (`3078a5adec`, `regua_escada`), mutação sangra em 6 regiões.
- **Gate novo:** o Wet Liner NÃO tem fio escuro de borda — orla ≤ 1,07, hoje 0,956 (`c426c54b8a`, dono 02/11). Nenhum botão reproduz o fio, logo o controlo é sintético (1,177).
- Impressões 1–4 do Shape Color bisseccionadas: um só degrau, a E4 (`6ffe01653d`).

**Réguas APOSENTADAS** (só paridade com o Rebelle; dono 31/10; registo em [`BUGS_painter.md`](../BUGS_painter.md), tabela «o que saiu»): `o_liner_tem_o_fio_escuro_da_borda` (`8235252138`, virou o gate novo) · Square Hard «como a foto» · a metade do contraste de `o_relevo_nao_tem_escada` · `cada_preset_parece_a_sua_midia` (`026736e919`) · o tecto Rebelle de `o_relevo_nao_cresce_com_o_pincel` (`2d0b889390`) · do integrador: paridade da pressão e Broad Nib.

## 3. Contadores e contratos

- `PROJECT_SCHEMA`: nenhuma linha +/− (`git diff 3df5894489..HEAD -- crates | grep PROJECT_SCHEMA` vazio) ⇒ sem recontagem. ADR: nenhum novo nesta etapa (o 0197 é do handoff anterior).
- **Contratos §6 intactos:** `git diff --stat 3df5894489..HEAD -- crates/ph2d-nodegraph crates/ph2d-vec*` ⇒ vazio. As linhas +/− com `PanelEvent`/`CanvasPaintTool`/`RasterEditTool`/`Tool` em `crates/` são só `use` e chamadas em testes de `ph2d-tool-painter`; nenhuma definição. `ph2d-editor-core`: 4 ficheiros, +30/−3 (rótulos), igual ao handoff anterior.
- O que um merge pode partir: `.config/nextest.toml` (filtros por nome: rode `scripts/nextest-filtros-casam.py` se uma régua renomeada estiver lá); PNG de miniaturas (binário: em conflito fica o da linha; gate `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`); hashes gravados do motor se outra linha mexer no depósito/solver. Ficheiros de teste > 700 linhas: testes isentos.

## 4. Prova de fecho (a suíte completa e o `nextest-impacted` NÃO correram: são da integração, ordem do dono 23/10)

- Corrida final por nome em `ph2d-tool-painter`, `ph2d-app-painter`, `ph2d-panel-painter-layers`, `ph2d-wet-paint`: **217 testes, 214 verdes, 3 vermelhos conhecidos** (§5 1–3; o 4.º, lento, não entrou).
- Nível lento corrido sozinho, 22/22 verdes: `a_miniatura_de_fabrica…`, `cada_preset_de_fabrica_e_a_sua_receita`, `cada_preset_de_fabrica_volta_ao_byte`, `leis_sistemicas::*`, `regua_escada::*`, `familias::grafite::*`, `aguarela::portoes::*`. `corpo_da_agua…` e `a_fenda_forte…` verdes sozinhos (load 16).
- `cargo fmt` e `clippy --all-targets -D warnings` limpos em `ph2d-wet-paint`, `ph2d-tool-painter`, `ph2d-app-painter`, `ph2d-panel-painter-layers`.
- Gates do editor-core (LOC/arquitetura, 14) e da shell (`file_loc_caps`/`fn_loc_caps`/`the_shell_only_shrinks`, 6): verdes.
- Auditoria com 2 lentes (motor e réguas): 12 achados, todos curados ou medidos.

## 5. ⚠️ Vermelhos CONHECIDOS (o integrador VAI vê-los; não afrouxe nenhum)

1. `macro_track::roteiro::tests::o_splash_tocado_pinta_a_tela_e_o_inicio_e_branco` — aberto por decisão do dono 02/11 («deixe como está»).
2. `transparencia::o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes` — o modo Blender deixa passar luz demais contra o Cycles na malha de hoje (L÷papel 0,5: nosso 0,152 × Cycles 0,124; a 1: 0,115 × 0,076). Em rampas sintéticas casa. Hipótese: a estrutura 2D fechada da malha real (TIR / luz que escapa na borda). Dados TEMPORÁRIOS (entradas e saídas do Cycles) em `/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/2adf13e5-96b7-44ec-b545-84d986a49c44/scratchpad/{tr,tr0,rampa,rampag}`; copie antes de perder. Próximo passo: malha 2D sintética.
3. `transparencia::a_transmissao_e_o_relevo_do_papel_mostram_o_grao` — Paper Relief 1 lê 0,490 contra a âncora 0,67 (faixa 0,55..0,8). A âncora é a calibração velha `papel_na_tinta = 1,5` contra a tela de antes de `ca26a7738a`, irreproduzível hoje. A/B: o piso do espalhar tira 0,026 (sem ele 0,516). Reancorar seria escolher o número, não medi-lo: ABERTO.
4. **Lento (~20–25 min):** `a_largura_e_a_prometida` falha só em Smear Wet r7 (17,0 para 14). Mantida a tabela ANTIGA (`a5e1c21b37`): sob a E10 a largura bruta do Smear Wet não é monótona abaixo de r≈7 e a régua do gerador `largura_do_rasto` tem defeito em r < 1. Próximo: `sonda_monotonia_da_largura` com `PH2D_SONDA_PRESET="Smear Wet"`, alinhar `raio_do_motor_da_tabela` (`tests/it/presets_de_fabrica.rs`) a `de_tabela`, regerar.

## 6. ABERTO

- Os 4 vermelhos acima. FPS com o Real Oil e o retângulo velho na shell (do handoff de 01/11, §6).
- A cena `cerda_e_marca` força Medium 0 (lei de geometria, margem ~2,7 %).
- Restante do handoff anterior §6.4 (E11 GPU com meio e chão, E12, outros modos, ajuda ilustrada).

## 7. Smoke do Enio (a interface do app é em inglês)

1. `cd /home/enio/Documentos/Projetos/PH2D && VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json cargo run -p ph2d-host-desktop --profile smoke` (máquina calma).
2. Painter, modo **Wet Paint**. Escolha **Graphite 8B** e risque com toque LEVE: tem de deixar marca visível. Repita com **Colored 4B**. Errado: continua quase invisível.
3. Pincel **Filbert** e depois **Thick Impasto** (Oil): passe 6 a 8 vezes sobre o mesmo traço. O relevo cresce um pouco e para; não vira caroços. Errado: grumos altos e duros.
4. **Gouache Flat**: um traço fino e um grosso. O fino tem de ficar fino e firme, o grosso cheio. Errado: o fino sai fraco/apagado.
5. **Wet Liner**: um traço com água. A borda é suave, sem linha escura no contorno. Errado: contorno escuro.
6. Desfazer/Refazer (Ctrl+Z / Ctrl+Shift+Z) no meio de uma pincelada com água: o traço volta limpo, sem sobra.

## Para o integrador

- Os **4 vermelhos conhecidos** (§5) são esperados; a cura de cada um está descrita, nenhum se afrouxa.
- **Não apague** `integ/2026-11-02-pintura`, `-pintura2`, `-com-painter` nem `backup/line-PainterWatercolor-pre-rebase-2026-11-02`.
- Gerador das tabelas de largura: `PH2D_MEM_MAX=64G` (pica em 33–35 GB).
- **A suíte completa e o `nextest-impacted` são seus** (a linha só correu por nome).
- O smoke é do Enio; integrar e shipar só por ordem dele.
