# HANDOFF de continuação — line/SfxLab, ondas W0 e W1 (2026-11-05)

Leitor: a próxima LLM da linha (ou o integrador). Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab`, branch `line/SfxLab`, base `c61f228b05`. **Nada integrado, nada enviado.** Estado: W0 e W1 feitas; **à espera do smoke do dono** (nenhum som foi ouvido por ninguém). Impactados e suíte completa ficam para a integração.

Commits: `git log --oneline c61f228b05..HEAD` (11, do `997883c719` plano ao `696586d262` espelhos do registo).

## 1. Superfície de colisão (saída de `bash scripts/collision-surface.sh`, colada)

```
SUPERFÍCIE DE COLISÃO — line/SfxLab contra main
  merge-base c61f228b05   ·   11 commit(s)   ·   113 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        216   (base: 216)
      └ tripla do gate               (216, 13, 29)   (base: (216, 13, 29))
    VEC_SCENE_SCHEMA 29 (base 29) · FLIP_SCHEMA 13 (base 13) · DOC_VERSION (timeline) 25 (base 25)
▸ REGISTRO DE COMPONENTES
  ⚠ ph2d-ecs                              118   (base: 116)
  ⚠ ph2d-render (espelho)                 119   (base: 117)
  ⚠ ph2d-script (espelho)                 119   (base: 117)
▸ CONTRATO CONGELADO (§6): node.rs intocado · tool.rs intocado
▸ ADR: último no disco 0198, próximo livre 0199
  ⚠ esta linha cria ADR: 0198   — reconte contra o main do dia
▸ Cargo.lock — ⚠ 2 pacote(s) '+name' novo(s): "ph2d-sfx-scene", "ph2d-sound-set"
▸ MARCADORES DE CONFLITO: nenhum
▸ TETOS DE LOC: nenhum arquivo da linha passa do teto
```

Contadores como DELTA para o integrador contar contra o main do dia: **registo de componentes ecs +2 (116→118), espelhos render e script +2 (117→119)**; **PROJECT_SCHEMA delta 0** (216; `SpriteAnimMarkers` é componente irmão, o postcard é posicional, sem degrau); **ADR +1 (0198, provisório)**; Cargo.lock +2 crates internas.

Shell desktop (`git diff --stat c61f228b05..HEAD -- shells/desktop`): 5 arquivos, +20 −1: `Cargo.toml` (+1), `src/anim_smoke.rs` (+12), `src/janela_do_jogo.rs` (±1), `src/render_loop/fase_sprite_inspector_smokes.rs` (+4), e o gate `…the_anchor_section_reaches_every_anchor_the_model_allows.rs` (+2).

## 2. O que um merge pode partir

- **`ph2d-app-components` está a 80866/80869 (folga de 3 linhas).** W2+ NÃO deve crescer ali; qualquer coisa nova vai para crate nova. Outros contra teto: ph2d-app-audio 10514/10547, ph2d-app-game 2708/2728, ph2d-app-sprite-anim 8345/8352.
- Registo de componentes: três contadores, cada um roda só na suíte da própria crate; se o main andou, reconte os três.
- `InspectorAnimRow` ganhou `markers` e `steps`; o gate da âncora da shell (commit `eddc2d5e27`) foi ajustado para isso: outra linha que construa `InspectorAnimRow` literal quebra a compilar.
- Comportamento mudado de propósito, que um teste de outra linha pode ter fixado: (a) **Autoplay só dispara no Play** (era ao abrir o projeto); a cena `PH2D_AUDIO_2D_SMOKE` agora pede Play para o zumbido; (b) **`Shuffle` no Audio Editor passou de «não repete» a baralho**, nas duas pontas (mesma palavra, mesma lei); o manifesto de texto do editor está intacto.
- Movimento por catraca: nova folha `crates/ph2d-sfx-scene`; testes do caminho do som em `shells/game-scenes/tests/it/` (`sound_set.rs`, `a_lista_de_sons.rs`, `sprite_marcadores.rs`, `jogo_arte.rs`); `registry_audio.rs` irmão de `registry.rs`.
- Memória `project-memory/project_sfx_module_no_ai_2026_11_05.md` está sem commit no primário: entra com a integração.

## 3. O que W1 entregou (mecanismo)

- **`crates/ph2d-sound-set`** (folha, sem FFT): `SoundSet` (nomes de `ProjectSounds`), `PickStrategy` (Random, Sequence, Shuffle = baralho), `no_repeat`, `Picker`, `sanitize`. O `VariationPicker` do Audio Editor é uma vista: mesma lei.
- **Sorteio numa porta só:** `SceneAudio::play`; gate `o_sorteio_tem_uma_porta_so`. Semente por objeto, recomeça no Stop (logo o sorteio começa igual em cada Play).
- **Autoplay:** o gate `autoplay_espera_o_play` nasceu VERMELHO («o Autoplay tocou sem Play»): tocava ao abrir, marcava como nascido e não disparava no Play. Cura: só nasce com `RunOnCopy`; `run_edge` no Stop cala tudo e recomeça.
- **Variação de volume** sobrevive ao update por quadro (`jitter_gain`; gate `o_volume_do_disparo_sobrevive_ao_quadro_seguinte`).
- **Marcador em quadro:** `SpriteAnimMarkers` em `crates/ph2d-ecs/src/sprite_anim_markers.rs`, componente irmão, sem degrau de schema. Começo reconhecido sem bit de estado. Gates `marcador_dispara_uma_vez_por_ciclo` (laço que cruza exatamente na volta) e `um_tique_atrasado_…`. Num ping-pong o passo do meio grita duas vezes por ciclo, de propósito.
- `SoundSet` e `SpriteAnimMarkers` são `D::machinery` no catálogo: fora da paleta, escritos pelos cartões. Portas: `AudioSource2D` e `SoundSet` Recusado(DO_PROJECTO); `SpriteAnimMarkers` Gravado. `property_laws`: `SoundSet` com `sanitize`.
- **Cartão Sound** (lista, Pick, Mode, No repeat, variações) e «Frame markers» em Advanced da animação. Consts com recurso nomeado (tabela de ids compilada): `MAX_SOUND_SET_ROWS=64`, `MAX_SOUND_OFFER=256`, `MAX_ANIM_MARKER_ROWS=32`; medição ao lado das consts: 64 linhas 1557 µs, 32 marcadores 873 µs; **a lista cheia de 256 da oferta NÃO foi pintada**. Edições: `AudioFieldEdit::Set*`, `AnimFieldEdit::Marker*`.

## 4. W0 (resumo; mecanismo em [02_W0_medicao.md](../02_W0_medicao.md), [ADR-0198](../../architecture/decisions/0198-o-motor-do-sfx-lab-e-kernels-proprios-sobre-det-math.md), sonda em `docs/SfxLab/ferramentas/sonda_w0/`)

- Decisão: kernels próprios sobre `ph2d-det-math`; FunDSP fora (o envelope muda com o vizinho mesmo fixando seed e phase; blocos mudam de bytes com AVX2).
- Receita típica de 2 s: 9,6 ms de kernels, 11,9 blocos, 165 amostra a amostra. Pior caso 38,8 ms > limite proposto de 30 ms ⇒ **cache por camada antes de W3**.
- Hash da receita típica sem efeito, igual em x86-64, x86-64-v3 e wasm32: `e262c4b2a017544c`. ⛔ **Intel e ARM: NÃO PROVADO** (comando para o Mac no doc W0).
- 32 de 38 efeitos do rack usam matemática de plataforma: sem garantia entre máquinas para receita com efeito.
- Porta de volume `integrated_lufs`: viés −0,25 LU; sem portão de silêncio (−10,44 LU com 9 s de silêncio); `ebur128` devolve −70 abaixo de 400 ms. Pendências de W6.

## 5. Prova de fecho

- **Mutação: 10 de 10 sangraram** — M1 `no_repeat`, M2 lei antiga de 8 refazimentos, M3 baralho, M4 `jitter_gain`, M5 corrida, M6 `run_edge`, M7 `has_sounds`, M8 `repeat_count`, M9 começo, M10 tique atrasado. Valem para a lei; **os seam tests do cartão NÃO tiveram prova de mutação**.
- Contratos: os dois greps da §3 deram 0 e 0 no fecho. Clippy das crates tocadas limpo; fmt limpo.
- **Sonda** — mil disparos, classificando o que saiu do MIXER contra cada som sozinho, sem variação:

| Modo | Contagem por som | Repetidos seguidos |
|---|---|---|
| Shuffle + No repeat | 334 · 333 · 333 | 0 |
| Random + No repeat | 327 · 340 · 333 | 0 |
| Random | 327 · 341 · 332 | 328 |
| Sequence | 334 · 333 · 333 | 0 |

## 6. Smokes (comando inteiro)

- Sonda offline: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && bash scripts/ph2d-run.sh cargo test -p ph2d-host-game-scenes --profile smoke sonda_mil_disparos -- --nocapture`
- Cena do dono: `PH2D_SFX_SMOKE=1` (`ph2d-sfx-scene`): nível 2, Coin 1–5 apontando para o objeto «Coin Sounds» (três sons sintetizados em `docs/SfxLab/arte/sons`, gerados por `docs/SfxLab/ferramentas/gera_sons_smoke.py`), Listener no Player, «Start Jingle» com Autoplay. Fotos conferidas: `target/foto-sfx/final.png` (app aberto com a cena) e `target/foto-sfx/marcadores.png` (via `PH2D_ANIM_SMOKE=2`: animação com um marcador e Advanced aberto).
- Gates por nome: `o_sorteio_tem_uma_porta_so`, `autoplay_espera_o_play`, `marcador_dispara_uma_vez_por_ciclo`, `o_volume_do_disparo_sobrevive_ao_quadro_seguinte` (todos em `shells/game-scenes/tests/it/sound_set.rs` ou `sprite_marcadores.rs`; confira com `git grep -n`).

## 7. ABERTO, em ordem

1. **Smoke do dono:** ninguém ouviu som; a sonda não passa pela coluna. O sorteio começa igual em cada Play (semente por objeto, recomeça no Stop): decidir no ouvido se incomoda.
2. **W2** (receita, kernels, render): crates `ph2d-sfx-recipe` e `ph2d-sfx-dsp`; entrar no gate `o_motor_nao_chama_a_libm_do_sistema`; degrau de schema pelas receitas no projeto, com `python3 scripts/schema-recount.py`. Cache por camada antes de W3. Não crescer `ph2d-app-components` (folga 3).
3. Provar Intel/ARM no Mac depois da integração.
4. Limitações sabidas: `ImageUseMemory` não guarda marcadores (formato posicional; marcador cuja tag sumiu fica inerte); nenhum teste afirma o texto pintado dos avisos do cartão; a lista de 256 da oferta não foi pintada; Preview vai por `play_target`, rota lida e não clicada em teste; a crítica pergunta se o canal do objeto chega ao Mixer: não testado.
5. Pendências do dono: biblioteca gravada e profissional para W4. A memória `project_sfx_module_no_ai_2026_11_05.md` fica para a integração.

## Smoke do dono

05/11: **smoke OK**, sem ressalva dita. Ele não comentou a ordem igual a cada Play nem a qualidade dos sons de teste. W1 não foi integrada: integrar é só por ordem dele. Próximo item: W2.
