# HANDOFF de continuação — line/UIUX, onda F36 (o que nasce sob a placa cheia renasce; arranque espera a placa; pegada por rótulo), 27/10

> Linha NÃO integrada, sem push. Base: `main` c3662cd8e4 (não andou). Ponta: `git log -1`; a F36 são os
> commits de `git log --oneline 9c00f9084e..HEAD`. **Smoke do dono 27/10: APROVADO («esperou e abriu. smoke ok»).**
> Formato inalterado (`PROJECT_SCHEMA` 197). A shell não cresceu (`the_shell_only_shrinks`: o arranque troca UMA
> chamada, `GpuContext::new` → `new_esperando_memoria`, mesma contagem de linhas).
> Mecanismo, tabelas e fotos: [`F36_plano.md`](../F36_plano.md) — leia §1 (censo e cura), §2 (pegada), §3 (arranque).
> Antecessor: [`HANDOFF_CONTINUACAO_line_UIUX_2026-10-26_F35.md`](HANDOFF_CONTINUACAO_line_UIUX_2026-10-26_F35.md)
> (a F36 fecha o seu §7 itens 1 e 3 e mede o item 2).

## 1 — Commits da F36

| commit | o quê |
|---|---|
| `7447250c24` | diagnóstico: pegada na placa por rótulo — `PH2D_GPU_PEGADA=<s>[,<s>…]` imprime o relatório do alocador somado por rótulo (`crates/ph2d-gpu/src/pegada.rs`) |
| `7a4bc2a1d3` | cura: o que a placa recusou ao NASCER renasce no fim do episódio (átlas, texturas individuais, preview da pintura); gates G17–G20 |
| `121d911b94` | `cargo fmt` nos `lib.rs` da onda |
| `3d29a96ddc` | plano F36 (censo, cura no app, pegada, conclusão do item 2) + fotos |
| `a6dcd5c8ff` | smoke do dono 27/10: PANIC no arranque com a placa cheia — curado; gate G21 |

## 2 — A cura (item 1 de F35 §7): recusa vista no NASCIMENTO

Premissa MEDIDA (plano §1): as 19 falhas de `Queue::write_texture` foram todas em texturas JÁ inválidas; nenhuma
escrita numa textura sã falhou por falta de memória ⇒ basta guardar a fonte quando a textura nasce recusada.

- **`TextureAtlas`** (`crates/ph2d-render/src/atlas/renasce.rs`): nasce sob `sob_escopo_de_memoria`; recusado,
  guarda os bytes de cada região viva (custo zero no caminho normal) e `refaz_se_recusada` refaz a textura com as
  MESMAS regiões (as UVs guardadas pelos chamadores seguem certas). `regrow_inplace` recusado re-insere pelo funil
  `sobe`, que também guarda.
- **`IndividualTextureStore`** (`individual_renasce.rs`, `individual_entry.rs`): `FonteRecusada` por entrada
  (8 ou 16 bits; `replace_pixels` atualiza) + `refaz_recusadas`, com o MESMO id e refcount. `acquire_empty` (só
  o preview da pintura) renasce transparente.
- **Quem chama**: `renasce_do_episodio` (`renderer_renasce.rs`), invocado no início de `draw_scratch`
  (`renderer_draw.rs`) quando `ph2d_gpu::memoria::geracao()` muda (padrão do `FxLive`, F35). Refeito o átlas,
  refaz `material_bind_group` e os grupos por amostragem.
- **`PainterGpuPreview`** (`ph2d-app-painter`): no fim do episódio larga sessão e slot e recompõe a tela inteira
  (a fonte são as camadas em memória comum).
- Gates: G17 `an_atlas_refused_at_birth_comes_back_with_its_sprites_after_the_episode` · G18
  `an_individual_texture_refused_at_birth_comes_back_with_the_same_id` · G19
  `the_sprite_renderer_rebuilds_a_refused_atlas_and_its_material_group` (todos `ph2d-render`, testes em
  `renasce_tests.rs`) · G20 `the_end_of_an_out_of_memory_episode_recomposes_the_gpu_preview` (`ph2d-app-painter`).
- **Mutações M1–M14** (subagente `mutacao`): as 14 sangraram, cada uma só no seu gate (M1 em G17 e G19). Plano §1.
- ⚠️ A detecção REAL da recusa (`erro.is_some()` do escopo) não é alcançável por teste sem encher a placa; os
  gates forçam a recusa destruindo a textura. A prova da detecção é a corrida no app.
- **Prova no app** (plano §1, tabela «A cura no app»): cena 1 (`PH2D_OBJECT_MODE_SMOKE=1`) com 282 MiB livres:
  543 erros do átlas + 3 098 desenhos com o grupo por amostragem antes; **0 depois e o Canvas branco voltou**.
  Fotos: [`cena1_placa_cheia_282MiB.png`](img_F36/cena1_placa_cheia_282MiB.png) ·
  [`cena1_volta_atlas_renascido.png`](img_F36/cena1_volta_atlas_renascido.png) ·
  [`cena41_placa_cheia_aviso.png`](img_F36/cena41_placa_cheia_aviso.png) ·
  [`cena41_volta_depois_do_G16.png`](img_F36/cena41_volta_depois_do_G16.png) (fecha F35 §7 item 3: a Nuvem volta
  desenhada).

## 3 — O arranque espera a placa (smoke do dono 27/10)

O 1.º smoke caiu antes de abrir: PANIC em `shells/desktop/src/init.rs:110`, `GpuContext::new: DeviceRequest(…
OutOfMemory)` — o pedido do DISPOSITIVO era a primeira coisa recusável e só se fazia uma vez.

- `GpuContext::new_esperando_memoria` (`crates/ph2d-gpu/src/context.rs`) + `memoria::tenta_enquanto_falta_memoria`:
  recusa por memória ⇒ avisa 1× no terminal («a placa recusou memória ao abrir … À espera»), anota o episódio
  (`anota_falta_de_memoria`: título e aviso de volta da F35 aparecem ao abrir), repete a cada
  `PAUSA_NO_ARRANQUE` 1 s até `ESPERA_MAXIMA_NO_ARRANQUE` 600 s. Outro erro falha na hora.
  `GpuContext::new` (testes, ferramentas) não muda.
- Gate G21 `the_editor_waits_for_the_graphics_card_instead_of_crashing` (CPU, `ph2d-gpu`); mutações sem
  reconhecer a falta, sem nova tentativa e sem teto: 3/3 sangraram.
- No app (`ocupa_a_placa 0 20`, 189 MiB livres): esperou ~14 s (12 recusas), abriu, 0 erros —
  [`arranque_com_189MiB_livres_abre.png`](img_F36/arranque_com_189MiB_livres_abre.png).

## 4 — Item 2 (modo leve): MEDIDO, não cabe sem mudar o desenho

Plano §2. `nvidia-smi` por processo: dispositivo vazio 198 MiB · editor sem cena e cena 1 **831 MiB** · cena 41
**1 343 MiB**. No caso do dono (946 MiB livres) a cena 41 não cabe (faltam ~397). Por rótulo (alocado 570 MiB
sem cena): `ph2d-render atlas` **341 MiB** (8192² com mips, `ATLAS_DEFAULT_SIZE_PX`, pré-alocado) e `vello.*`
**~166 MiB por `vello::Renderer`** (alocadores fixos do Vello 0.10; a cena 41 acorda 3 ⇒ ~500).

- Desenhar MENOS não é a alavanca: são dois custos FIXOS. Tirá-los não muda um pixel.
- **Proposta F37**: (a) átlas sob demanda — nascer 2048² (~21 MiB), crescer por `regrow_inplace` (−~320 MiB);
  (b) UM `vello::Renderer` partilhado por placa (−~332 MiB; ≥ 8 donos de `VelloPass`; `VelloPass::refaz` passa a
  reaplicar os `registos` de todos). Resultado esperado cena 41: **~690 MiB**.
- **Pré-condição de (a)**: os consumidores que GUARDAM a UV do ladrilho branco (`motion_default_uv`:
  `default_uv_rect` do Motion, `p.uv` das partículas, `b.uv` dos bandos em `shells/desktop/src/init.rs`;
  `ph2d-app-motion/src/macro_track/inicial.rs`) têm de reler a UV a cada quadro, senão amostram o texel errado
  quando o átlas cresce (o defeito «vermelho→grená» do `init_subsystems.rs`). As sprites já leem por quadro.
- (b) cruza famílias de outras linhas (Motion, Luz 2D, Quadro, filtros) ⇒ avise o Enio antes de tocar.
- **Primeiro passo da F37: MEDIR** com `PH2D_GPU_PEGADA` na cena 41 e por placa (NVIDIA × Radeon onboard), antes de
  escrever. Plano §2 tem a tabela de partida.

## 5 — Superfície de colisão (DELTA) e o que um merge pode partir

- `git diff --stat c3662cd8e4..HEAD` (conferir): F36 sozinha = 52 ficheiros, +2 305 −176 (`git diff --stat
  9c00f9084e..HEAD`). Alterados: `ph2d-gpu` (`pegada.rs`, `memoria.rs`, `context.rs`), `ph2d-render` (`atlas/`,
  `individual*.rs`, `renderer_draw.rs`, `renderer_renasce.rs`, `renasce_tests.rs`), `ph2d-app-painter`
  (`PainterGpuPreview`), `shells/desktop/src/init.rs` (UMA chamada).
- Um merge parte: (a) novo cache de textura que nasça sem o escopo de memória — não renasce e volta a ser o
  defeito do §2; (b) quem alterar o funil `sobe` do átlas sem manter a guarda da fonte; (c) quem criar outro
  chamador de `GpuContext::new` no arranque do editor (volta o PANIC); (d) `renderer_draw.rs`: a chamada de
  `renasce_do_episodio` no início do `draw_scratch` é fácil de perder num conflito.
- Contador de cena: `PH2D_OBJECT_MODE_SMOKE` segue até **41** (a F36 não criou cena). `PROJECT_SCHEMA` 197.

## 6 — Prova de fecho

- `cargo fmt` limpo. Clippy `-p ph2d-gpu -p ph2d-render -p ph2d-app-painter`: **clippy da onda: verde**.
- Gates G17–G21 e as mutações M1–M14 + as 3 do G21: verdes/sangraram como acima.
- Suíte inteira NÃO corrida (regra do dono: só antes da integração; quem a corre é o integrador).
- Comandos CPU: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && bash scripts/cargo-test-narrow.sh ph2d-gpu the_editor_waits_for_the_graphics_card`
  e `bash scripts/cargo-test-narrow.sh ph2d-render an_atlas_refused_at_birth` (G17; G18/G19 idem pelo nome).
  G20: `bash scripts/cargo-test-narrow.sh ph2d-app-painter the_end_of_an_out_of_memory_episode`. Se algum for
  `#[ignore]` (placa), use `bash scripts/ph2d-run.sh` com `PH2D_GPU=1` e `-- --ignored` (confira com `git grep`).

## 7 — Smoke do dono (APROVADO) — passos

1. Terminal 1: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/examples/ocupa_a_placa 50 60` (enche a placa; ao «seguro 60 s» ela está cheia).
2. Terminal 2: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=1 ./target/smoke/ph2d-host-desktop`.
3. O terminal diz «a placa recusou memória ao abrir … À espera». Quando o terminal 1 acaba, a janela abre com o
   quadrado branco «Canvas».
4. Errado: o editor cai com mensagem de memória (PANIC), ou abre sem o quadrado branco.

Truques de medição (para a F37): pré-construir ANTES de encher a placa (o `fotografa_cena.sh` recompila);
toda a rodada dentro de `PH2D_GPU=1 bash scripts/ph2d-run.sh`; `ocupa_a_placa` enche em blocos de 256 MiB (alvo
50 → ~282 MiB livres; alvo 0 → ~189).

## 8 — ABERTO, por ordem

1. **F37**: átlas sob demanda + UM `vello::Renderer` partilhado (§4). Medir primeiro; (a) exige mover o UV do
   ladrilho branco para leitura por quadro; (b) cruza famílias de outras linhas.
2. Views do mipgen: não auditadas (F35 §7 item 1, resto) — o censo da F36 não as viu recusadas depois da volta.
3. O episódio OSCILA enquanto outro programa segura a placa (a cada 3 s tenta e recusa de novo): desenho, não
   defeito (plano §1); só avise se o dono estranhar o piscar.
4. `CLAUDE.md` §5 (UIUX): o link «Último» troca só na integração; `PH2D_OBJECT_MODE_SMOKE` segue até 41.
5. Herdado: F34 §6 itens 4–5; F33 §6: 2, 3; F32 §6: 2, 3, 4.
6. **Integrar e push só por ordem EXPLÍCITA do Enio**; a suíte completa é do integrador.
