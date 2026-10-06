# HANDOFF DE INTEGRAÇÃO — `line/components`: O DESENHO E O CORPO, E O ABERTO DA W15 NUM CICLO (navegação W16) — 2026-10-05

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §25 (os kill-criteria §25.1, as quatro
> rodadas de medição §25.2–§25.4, os vereditos §25.5, as recusas §25.6, a prova §25.7, o aberto §25.8). A linha
> ainda leva a W15 inteira, por integrar: o handoff dela,
> [`O_TIQUE_DEPOIS_DA_PORTA`](HANDOFF_INTEGRACAO_line_components_O_TIQUE_DEPOIS_DA_PORTA_2026-10-05.md), continua
> válido para a parte dela (§1–§4b) — este acrescenta a W16 por cima.

## §0 — O `--ff-only` deve passar limpo

- Rebaseada no `main` `a46c4c200` (a guarda R4 — `.claude/hooks/`, `docs/DevOps`; nenhum ficheiro desta linha) no
  início da sessão, limpa. NADA integrado desde `5d596eaaf`: a W15 (`c3395b0a7..5ab72773f`) e a W16 por cima —
  `baf14765b` (A: o disco, o censo, o roteador da arma) · `42914d83a` (A: as peças reservadas juntas) · `f85ddc637`
  (B, C2 e E) · `091eee8b2` (o plano §25, o ADR-0180, o motor da mutação) — e o deste handoff.
- ⚠️ A `line-Components2` que o handoff da W15 avisava já não existe (nem a worktree, nem o ramo): sem colisão.

## §1 — Superfície de colisão (a W16)

| ficheiro | o quê |
|---|---|
| `ph2d-render/src/atlas/mod.rs` · `atlas/tests.rs` · `lib.rs` | **API nova:** `DISC_TILE_KEY` (`u32::MAX − 1`), `DISC_TILE_PX`, `disc_tile_pixels`, `TextureAtlas::insert_reserved_tiles`; **⚠️ `insert_white_tile` deixou de ser `pub`** — quem a chamava passa a `insert_reserved_tiles` (mesmo retorno: o UV do branco) |
| `shells/desktop/src/init_subsystems.rs` | uma linha: `insert_white_tile` → `insert_reserved_tiles` (a shell não cresce) |
| `ph2d-app-components`: `smoke_desenho.rs` **NOVO** · `smoke_desenho_e_corpo_tests.rs` **NOVO** · `lib.rs` · 18 cenas `*_smoke.rs` | os sprites das cenas (nenhum `Collider` mudou); `FAMILY` ganha `PH2D_WEAPON_SMOKE` (a shell já o lia) |
| `ph2d-nav/src/agent.rs` · `agent_tests.rs` | **API nova:** `agent::a_vista`; `Vez` ganha `a_vista` (quem a constrói por campos põe-no; `Vez::sem_tecto` = `false`); `AMeio` ganha `persegue` |
| `ph2d-physics-ecs/src/bridge/nav.rs` (`694 / 700`) · `nav_desvio.rs` · `nav_fila.rs` · `nav_tests.rs` | `nav_desvio::contorna` (a tangente); as alavancas da sonda juntam-se em `fila::Sonda`; **API nova:** `set_nav_detour`, `set_nav_sight` |
| `ph2d-physics-ecs/examples/medir_replaneio.rs` | o pool de uma thread, os perseguidores, o trabalho por porta |
| testes | `ph2d-physics-ecs` `tests/it/nav_desvio_largo.rs` (reescrito), `nav_fatias.rs`, `nav_mundo.rs`, `nav_nascer.rs` |
| docs | plano 30 §25; ADR-0180 (adenda D); `ferramentas/mutacao_navegacao_w16_2026-10-05.py` |

Zero crate nova, zero pacote novo no `Cargo.lock`, zero i18n, zero componente registado, `PROJECT_SCHEMA` intocado.

### ⚠️ O que um merge pode partir

1. **O número do ADR SOMA entre linhas:** `0180` recontado no fecho desta sessão contra o `main` e as seis worktrees
   (máximo `0179` em todas) — reconte-o na integração e corra `bash scripts/adr-index.sh`.
2. Quem chamar `TextureAtlas::insert_white_tile` de fora da `ph2d-render` não compila: use `insert_reserved_tiles`.
3. Quem construir `ph2d_nav::agent::Vez` ou `AMeio` por campos tem campos novos (`a_vista`, `persegue`).
4. Uma cena de smoke NOVA desta família tem de desenhar dentro do corpo (o censo reprova) e, se a shell ler o
   roteador dela, tem de estar em `FAMILY` (o gate `todo_roteador_que_a_shell_le_esta_na_familia` reprova).
5. Tecto: a ponte `nav.rs` em `694 / 700` — a cura de quem crescer é MOVER (como a `Sonda` foi para o `nav_fila.rs`).

## §2 — O que a W16 traz

| item | o que mudou | medido |
|---|---|---|
| **A** o desenho maior que o corpo | a bola desenha-se como o DISCO dela (o atlas ganha o disco reservado); quem RODA para encarar o movimento (heróis `ToMovement`, projécteis `face_velocity`) fica bola — uma caixa que roda no sítio entra na parede sem teste — e o herói ganha o filho «Rumo» dentro do disco; a moeda usa a constante do corpo | o censo: `58` desenhos fora em `41` cenas antes → `0` em `42` cenas (`139` corpos); nenhum corpo mudou: nenhum trajecto mudou |
| **B** a barreira larga e lenta | o troço até ao próximo canto que passa por um corpo que ANDA aponta à tangente do polígono engordado pela folga do desvio, do lado mais curto (no empate, o do peso de lado) | `3 m` a `0,3 m/s`: `578 → 200` tiques (CONTROLO parado `198`; era `2,9×`); os três casos `194 · 200 · 207` |
| **C1** o tique com lama | nada — é a repartição: B gasta MENOS por porta que A e o CONTROLO de B fica abaixo do de A | `4,07` contra `7,66 M` por porta a `200` agentes |
| **C2** o perseguidor | o alvo À VISTA é a resposta da procura: instala-se sem procura nem a vez | ao ar livre, com o orçamento esgotado, `195 → 0` tiques-agente à espera; o atraso real na lama cerrada é `≤ 2,2` tiques em média |
| **D** a web numa thread | nada no código — nenhuma cura cumpre (§25.6); a adenda do ADR-0180 escreve o custo e a exigência da shell web (o `rayon` sobre Web Workers) | uma thread, `200` agentes no stress: `40,6 ms` (todos os núcleos `10,5`; a procura inteira `49,6`) |
| **E** a mutação `S5` | a fixtura que a separa: duas portas, a do caminho fecha no 1.º tique depois do âncora, a outra diferente no fim, um orçamento de duas procuras | `S5` sangra |

## §3 — ⏳ O que fica ABERTO

- **A web numa thread** paga as fatias do passo em paralelo — a cura é da shell web que ainda não existe (ADR-0180).
- **Quem persegue sem o alvo à vista** na lama cerrada: máx `30` tiques a `200` agentes. A vez primeiro encurta-o
  para `4` mas traz o pico de volta pela DOBRA da fatia num recomeço (crítico `40 → 160 mil`); uma cura teria de mudar
  essa dobra só para elas — não medida.
- A dominância da procura ponderada (§22.4), como estava.

## §4 — A prova de fecho

- **As medições** (`medir_replaneio`, a régua do dono de 05/10: as versões no MESMO processo, o pool de uma thread ao
  lado do de todos, 7 rodadas intercaladas com a ordem rodada, o mínimo, perfil `smoke`): quatro rodadas, em série
  porque cada uma mudou a seguinte (§25.4) — `target/prova/w16/medir_replaneio_rodada{1,2,3,4}.txt`. A 4.ª correu a
  load `56–62`: o relógio dela não vale; as colunas de trabalho e de espera não dependem dele.
- **Gates novos** (9): ver o plano §25.7.
- **Mutação `18 / 18`**, zero defeitos de arnês, árvore igual antes e depois (`target/prova/w16/mutacao_w16.txt`).
  A 1.ª corrida deu `15 / 18` — dois gates ligavam à mão a alavanca que o produto tem por omissão, e a folga da
  tangente não tinha régua (agora: os tiques em que o desvio corta o pedido, `0 · 0 · 7` contra `51 · 58 · 95`).
- **Gate batched** (sobre o merge-base, a W15 e a W16 juntas): `nextest-impacted` **`15 779 / 15 779`** · clippy
  `--workspace --all-targets -D warnings` limpo. A 1.ª volta apanhou dois: `arch_color_space_typed` (o
  `disc_tile_pixels` devolvia `Vec<u8>` — cor crua numa fronteira pública; passou a `Vec<SrgbRgba>`) e quatro
  `redundant_closure` no censo — curados em `551a96582`, e as mutações `A1`/`A5` re-corridas (sangram). Load
  `15–51` (outras linhas a compilar).

## §5 — O smoke

Nenhuma cena nova; as de sempre, agora com os bonecos redondos desenhados como discos (o que se vê é o que toca):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
```

As 20 cenas tocadas fotografadas no ecrã virtual (`target/prova/w16/fotos/*.png`, `59–60 fps`): os discos coincidem
com o contorno do corpo; o herói da arena é o disco azul com o «Rumo» dentro (na 1.ª foto a `9 s` ele piscava depois
de uma mordida — refotografado a `5` e `7 s`). A barreira lenta (B) e o alvo à vista (C2) não têm cena: a condução de
uma cena pequena não muda, e quem os mostra é a bateria (`nav_desvio_largo`, `nav_fatias`).

### O smoke compilado (a 2.ª corrida, colada)

Depois de `rm -rf target/*/incremental` (`5,5 G` do `debug` + `5,4 G` do `smoke`), a 2.ª corrida de
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` — zero linhas `Compiling`:

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.26s
```

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (8% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                239   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                749 : 213   alvo: <= 1,0  razao 3.5x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  34%   alvo: >= 80%  (1824 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         462 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
