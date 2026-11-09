# HANDOFF DE INTEGRAÇÃO — linha Pixel Art (P10c fix, P11 Slices, P12 Ergonomia) — 2026-11-08

> Leitor: o agente integrador. A linha **fecha e PARA**: não integra, não faz push. Planos: [P11_plano.md](../P11_plano.md) · [P12_plano.md](../P12_plano.md) · oráculos [P12_oraculos.md](../P12_oraculos.md) · teclas [P12_teclas.md](../P12_teclas.md).
> Mecanismo por onda: [P11](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-08_P11.md) e os planos acima (o mecanismo da P12 vive no `P12_plano.md`, não aqui).
> Antecessor: [P10](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-06_P10.md) (já no `main`, base desta linha).

## 0. Leia primeiro

- Entregas, **integram como UMA**: fix da P10c (`79989e9e0b`: `G` é a grade do canvas, balde → `P`) · **P11 Slices** · **P12 Ergonomia**.
- **Smoke do dono:** P11 OK em 08/11; P12 smoke 1 (defeitos achados e curados), smokes 2 e 3 OK em 08/11 («Restore OK. Tudo OK»). **Integrar não é aprovar.**
- **Não correram nesta linha:** suíte completa e `nextest-impacted` (regra do dono 23/10: são do integrador).
- ⚠️ Commit `a5d6e2c0ec` está **vermelho sozinho** (cura em `44ef3cf432`, o tecto de 500 dos widgets). Se cada commit tem de compilar/passar: **squash** os dois.

## 1. Identidade

- Branch `line/PixelArt`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt`, HEAD de código `6597c82542`.
- Merge-base = `main` `652b506b58`. `git log HEAD..main` **vazio no fecho** (o `main` não andou): rebase nulo hoje; reconfira.
- `git diff --stat 652b506b58..HEAD`: **95 commits, 494 ficheiros, +41 246 / −2 552** (docs, fotos e fixturas incluídas).

## 2. O que entrega

- **P11 Slices:** `Slice` no documento (formato, desfazer, `.ase`, `meta.slices`), ferramenta, 9-slice e pivot, keys por quadro, janela SLICES; `PixelTool::Slice` no PLAIN. Cena **24**.
- **P12 atalhos editáveis:** crates `ph2d-pixel-keys` + `ph2d-panel-pixel-keys`; clique-captura; delta do usuário em `~/.ph2d/pixel-keys.txt`. Lei de conflito `RESERVED_GLOBAL`/`SAME_MEANING`/`SHADOWED`/`OWNER_EXCEPTIONS`, com o censo `the_app_keys_census` lendo os braços de tecla da shell; o painel sob o ponteiro mantém as suas teclas (`PanelUnderPointer::keeps`).
- **Run Command** = paleta global `Ctrl+K` com grupo Pixel. ⚠️ **A busca passou a casar por subsequência (fuzzy) e reordena, em `ph2d-editor-core` `busca.rs`: afeta TODAS as paletas do app.**
- **Undo History** (`ph2d-panel-pixel-history`): passos rotulados, `go_to`, tecto de bytes, `History::revision`.
- **Recuperação de projeto** (`ph2d-recovery`, vale para o APP inteiro): cópia a cada N min (Edit ▸ Preferences ▸ Recovery, fábrica 2 min, pref `recovery.minutes`); gatilho por pausa do artista + prazo duro 2×; relógio em `about_to_wait` (independe de quadros); pulso de mudança com `CaptureCache::take_changed` **pegajoso**; vivacidade por trava do SO (`File::try_lock`); escrita atómica; cartão Restore/Discard/Later; **pergunta ao sair Save/Don't Save/Cancel para o app todo**; `PH2D_RECOVERY_DIR`.
- **User Data** (cor/texto/props em sprite/layer/cel/tag/tileset/slice): formato nativo **MINOR 4→6**, chunk `.ase` `0x2020`.
- Prefs por documento (Grid/Background/Onion) + *Set as Defaults* (`~/.ph2d/pixel-prefs.txt`); Sprite Properties.
- Regra de colocação de janela flutuante `borda::coloca`/`assenta_ao_abrir` (`cards_borda.rs`); `Widget::Search` com × (`cards_busca.rs`, `TextInput::reserva_direita`); linhas no menu Window para as janelas do Pixel (check == na tela, `JanelaDaMao`).
- Auditoria de fecho (2 lentes) achou e curou: R1–R15 (segurança de dados: Restore/Open só adotam depois de carregar; estado desconhecido pergunta; fechar sem decisão guarda a cópia; vivacidade; retry+toast na falha de escrita; cópia sem meta usa mtime; ida e volta da tecla Comma; user data de cel no `.ase`; chaves de mapa desconhecidas voltam; bytes de UserData; textos de recuperação em `ph2d-i18n/src/recovery.rs`; `codec.rs` partido), U1–U8, F1–F3, 5 vermelhos e a corrida do pulso.

## 3. Superfície de colisão

Aditivo salvo onde marcado. Confira com `git diff --stat 652b506b58..HEAD -- <caminho>`.

| Onde | O quê |
|---|---|
| `ph2d-editor-core` (foundational) | `busca.rs` (**fuzzy + reordena: todas as paletas**), `panel/cards_borda.rs`, `cards_busca.rs`, `Widget::Search`, `TextInput::reserva_direita`; ids/tabelas de menu e hero; `PixelTool::Slice` no PLAIN (**`SHAPES_AT` 16→17**); **Esc com menu aberto só fecha o menu**; **campo que perde o foco reseta o visual** |
| `ph2d-label-census` (**crate compartilhada**) | `keys.rs` `modulo_inteiro`: o censo agora lê os submódulos da tabela; qualquer linha que use o censo vê a nova leitura |
| `ph2d-i18n` | `recovery.rs` NOVO + chaves de keys/history/slices/prefs/props — foundational, append |
| `ph2d-ecs` | `CaptureCache::take_changed` (bit pegajoso) + captura incremental |
| `ph2d-pixel-core`/`-io`, `ph2d-aseprite` | Slice, UserData, `PixelPrefs`, `History::revision`, `.ase` `0x2020`/`0x2008`; **codec MINOR 6** |
| `ph2d-app-pixel` | teclas editáveis, histórico, cena 24/25; tecto da catraca **baixou** |
| `shells/desktop` | a shell agora depende de `ph2d-pixel-keys`; pulso/relógio em `about_to_wait`; pergunta ao sair; braços de tecla (lidos pelo censo); `Cargo.toml` (features) |
| `ph2d-panel-registry-init` | +4 painéis no registro tipado |
| `shells/pixel-scenes` | cenas 24, 25, testes novos |

**Crates NOVAS:** `ph2d-panel-pixel-{history,keys,prefs,slices}`, `ph2d-pixel-keys`, `ph2d-pixel-slices`, `ph2d-pixel-cenas`, `ph2d-recovery`. `Cargo.lock`: REGENERE, nunca à mão.

**API pública que muda:** `PixelTool` ganhou `Slice`; `History::revision` novo; `CaptureCache::take_changed` novo. `match` exaustivo noutra linha sobre `PixelTool` parte.

## 4. Contratos e schemas

Contratos §6 **não tocados:** `git diff --name-only 652b506b58..HEAD` sem ficheiro de `ph2d-nodegraph*`, registros de node/tool, `ph2d-imageio`, `ph2d-vector*`; nenhuma linha ± nomeia `NodeOp`/`OpResolver`/`NodeManifest`/`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`/`VectorOp`/`ColorProfile`. `PROJECT_SCHEMA` **não** mexido. Blob do Pixel: MINOR 4→6 (campos appendados; fixtura MINOR antiga nos testes do `ph2d-pixel-io`).

## 5. Números que o integrador RECONTA (DELTA, nunca à mão)

| Item | Delta |
|---|---|
| `MODULE_TRUTHS` | 24 → **29** |
| Registro tipado de painéis | **+4** |
| Features default de painel da shell | **+4** |
| `the_shell_only_shrinks`, `ph2d-app-pixel` | tecto **11 415 → 11 278** (baixou; soma entre linhas, use `python3 scripts/recontar.py`) |
| `PH2D_PIXEL_SMOKE` | `max_level` → **25** (24 = P11, 25 = P12); se outra linha tomou, RECONTE |
| `SHAPES_AT` | 16 → 17 |
| chrome-sync | conferir colisão do **z=132** |
| `Cargo.lock` | 8 pacotes novos |

## 6. Prova de fecho (todos verdes em `e08b05570f`/`6597c82542`; por nome, suíte completa é do integrador)

- fmt + clippy `-D warnings` em 26 pacotes tocados + `ph2d-ecs`/`ph2d-label-census`.
- `ph2d-editor-core`: `arch_*`, `the_shell_only_shrinks`, `workspace_src_files_under_loc_cap`, `architecture_*_contract_surface` (nodegraph 3, vector-doc 11, imageio 2, tool), `architecture_claude_md_cabe_no_orcamento`.
- `registry-init` 64 nomeados · `host-desktop` `file_loc_caps` etc. 8 · `pixel-scenes` 341 + mapa 2 · `panel-pixel-tools` 75 · `-timeline` 32 · `ph2d-pixel-keys` 22 · `ph2d-recovery` 27 · `ph2d-ecs` incremental 12 · conjuntos das crates novas.
- **Medido** (`--profile smoke`, mesmo processo, intercalado, mínimo; Ryzen 9 9950X): parte do Pixel na cópia, no fio da janela: **1,12 ms** (projeto de 128 MiB, um documento editado; antes 76,1 ms; a impressão sai do fio: 34,1 ms → 43 µs — [P12_plano.md §4.3](../P12_plano.md)); `wants_pulse` **17,2 ns/volta** (n=9, load ~29); `capture_incremental` 10k entidades parado **0,613 ms**, 1 % mudado **0,777 ms**, 1 entidade 0,0044 ms; `print_of` 1 MiB 0,118 ms. **Não medido:** a captura do projeto inteiro da shell dentro do pulso.

## 7. O que um merge pode partir

1. `match` exaustivo sobre `PixelTool` (variante `Slice`).
2. **Paletas de outras linhas:** testes que assumem a ordem/casamento exato da busca (fuzzy + reordena).
3. Testes que contam `Esc` com menu aberto, ou foco de campo, em outras linhas.
4. Linhas que usam o censo de `ph2d-label-census`: agora leem os submódulos.
5. Quem estenda o prompt de saída (agora para o app todo) ou `CaptureCache` (bit pegajoso).
6. `Cargo.lock`, features de `shells/desktop` e do `ph2d-panel-registry-init`, `MODULE_TRUTHS`, cena de smoke 24/25, tecto de `ph2d-app-pixel`, tecto de 700 linhas por ficheiro, `the_app_keys_census` se outra linha acrescentou braços de tecla na shell.

## 8. Smoke do dono (reconstruir o binário no `main`/worktree onde for abrir)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && cargo build --profile smoke -p ph2d-host-desktop && PH2D_PIXEL_SMOKE=25 ./target/smoke/ph2d-host-desktop` (a cena 24, Slices, é igual com `PH2D_PIXEL_SMOKE=24`).
2. Cena 25 abre um boneco com sete passos no histórico: abra **Undo History** (menu Window), clique num passo antigo, depois **Ctrl+Z/Ctrl+Y**. *Errado:* o clique não volta o desenho.
3. `Ctrl+K`, digite `flp`: acha Flip. Abra **Keyboard Shortcuts**, clique numa tecla e aperte outra: tecla ocupada pergunta Replace. *Errado:* rouba calado.
4. **Edit ▸ Preferences ▸ Recovery** = 10 seconds, pinte, espere, feche o terminal; reabra sem a cena: o cartão *Recover project* oferece Restore. *Errado:* nada oferecido.
5. Fechar com trabalho: pergunta Save / Don't Save / Cancel.

Fotos: `handoffs/P12_smoke_*.png` e `P12_fecho_*.png` (e `P11_f2_real_propriedades.png`).

## 9. ABERTO (honesto)

- `Ctrl+D` na Hierarchy duplica e passa a Object mode (comportamento antigo do app, não da linha).
- Em 1024 px, 5 janelas se sobrepõem um pouco (o × sempre clica).
- Linha vazia de Slices fora da aba Pixel diz «Pick the Slice tool».
- Recuperação: produtores só cobertos por entrada (cena vetor, Flip, guias, doc da timeline, doc do Motion, Painter sem macro, som/SFX/mixer); nome/câmera/comentários do Quadro não entram em `BoardDoc::rev`; teclas do usuário só checadas contra `RESERVED_GLOBAL`; borda «restaurado mas adoção falhou»; nome vazio no cartão quando falta o meta; painel HIERARCHY sem ×; relógio ainda em `about_to_wait` mas a captura do projeto da shell no pulso **não medida**.
- Residuais de [P12_plano.md §11](../P12_plano.md) que continuam verdadeiros: a pergunta sobre o clique de `10 seconds` no Wayland nativo (XTest não chega) e a janela que deixa de receber quadros.
