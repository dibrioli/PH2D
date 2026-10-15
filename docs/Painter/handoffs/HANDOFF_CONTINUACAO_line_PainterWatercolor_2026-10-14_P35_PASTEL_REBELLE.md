# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 35: o pastel seco contra o Rebelle (2026-10-14)

## §1 — Coordenadas

Linha sobre o `main` `0aab2c1cb6` (14/10). Sem push. Prompt:
[35](../NewFeaturesWetPaint/35_PROMPT_janela_pastel_contra_o_rebelle.md) (a secção «Resultado (14/10…)» tem a
tabela do Soft Pastel; não se copia aqui). Commits, por ordem:

1. `4a20af5409` — «Use Texture Colors» esmaecido recusa o clique.
2. `f55d9a83c1` — lei da ponta seca + Soft Pastel + bancada + fixtures.
3. `f0bfa63391` — as linhas «Layer N Color» esmaecem e recusam o clique (26 C16).
4. `2193eb5afb` — a família seca contra o Rebelle (Graphite 8B, Charcoal, Oil Pastel).
5. `1e5f79a5a9` — o arrasto volta a ser do PRESET; a família seca passa `midias_de_fabrica`.

## §2 — Painel: o clique recusado

`inercia::Natureza { DependeDeOutro, MeioNaoLe }`, `Dependente::natureza/cobre`, `Inercias::recusa_clique`. A recusa
vive na porta de eventos do painel da ferramenta (`tool/trait_impls.rs`, `handle_panel_event_corpo`); a dica
(tooltip) fica. A Watercolor nem oferece a linha. Portões (`ph2d-panel-painter-layers`, `tests/it/seam_texture_colors.rs`):
`a_texture_colour_line_the_medium_never_reads_refuses_the_click` e
`the_layer_colour_rows_refuse_the_click_where_the_medium_never_reads_them`.

## §3 — A bancada (Fase 0, no tempo)

`NewFeaturesWetPaint/ferramentas/`: `pastel_eventos.py`; `pastel_tela.sh` (tela virtual; zoom 100 % pelo campo do
Navigator em 1851,236; espera duas fotos iguais para o «up»; um `SET_BRUSH` isolado primeiro, porque o Rebelle pinta
o 1.º gesto com o tamanho antigo depois de trocar de preset); `pastel_regua.py` (textura 2–8 px, buracos, mosqueado
grosso 8–32 px, ondulação da borda, subida 10→90 %, miolo, pó, deriva no tempo; controlo = Rebelle desfocado σ 1,5).
Sonda `wetpaint/sonda_pastel_tempo.rs::sonda_pastel_no_tempo` (envs `PH2D_SONDA_PRESET/KNOBS/MAPEAMENTO/GRAO/
GRAO_TIPO/DUREZA/RECTO/SHAPE/TAMANHO/FALLOFF/JITTER/ORDEM`). Fixtures `fixtures/rebelle/{pastel_dry_thin_29,
pastel_dry_thin_70,graphite_8b_9,graphite_8b_27,charcoal_64,oil_pastel_2_100}` + `pastel_dry_thin_README.md`.
`size_px` do Rebelle ≠ largura visível (Dry Thin 29 → 13 px; 60 → 25). O Rebelle move 0 px de 0,5 s a 6 s depois do
levantar nos quatro meios.

## §4 — Mecanismos medidos

1. **Ruído no pen-up:** a tinta de Water 0 pousou SUSPENSA e o 4.º/5.º passo de água depois do levantar a advectou
   (1 854 px; só o 1.º traço da sessão).
2. **Borrão (smear):** o arrasto da tinta (Drag 0,16) puxa do âncora anterior.
3. **Sem craquelado / largura instável:** grão Noise 0,4 em mapeamento Tiled (base 256 px) = uma mancha de ~640 px: o
   mesmo traço 15 px em y=40, 34 em y=450.
4. **Shape Noise pálido:** Size 1 de fábrica põe UM ladrilho de ruído sobre a ponta inteira: 0,43 da tinta; Size ≥ 4 →
   0,74–0,78 (não mudado, por desenho).

## §5 — Lei do motor e presets

`ph2d-wet-paint/src/trail/transfer.rs`: `ponta_seca` = a janela da ponta sem água (`sum_water <= 0`, NÃO por texel — por
texel mudou a borda do Express Oil); em célula seca (`film <= FILM_TRACO`) o pigmento pousa em `sett` por `pousa_seco`.
O arrasto da tinta continua knob do PRESET (a 1.ª versão zerou-o; refutado pelo portão antigo: o arrasto só suaviza).
Muda o Express Oil (Water 0,1): a ponta seca no fim do traço.

Receitas em `tests/it/receitas_de_fabrica.rs` (campo novo `pincel: &[...]`). O gerador `presets_de_fabrica.rs::parametros`
PARTE do ficheiro existente: desde o plano 27 W2 as cerdas vivem no Shape e a captura nova perdia-as. Regenerar:
`PH2D_MEM_MAX=48G`, pico 25,9 G, ~13 min.

NOTA = Σ|log(nosso/Rebelle)| sobre textura, buracos, miolo, ondulação, subida, mosqueado (menor = mais perto):

| Preset | Receita | NOTA |
|---|---|---|
| Soft Pastel | Noise 100 · papel 0,4 · carga 100 · dureza 0,2 · arrasto 0 | 4,76 → 1,40 |
| Graphite 8B | Noise 32 · papel 0,65 · carga 125 · falloff Linear · jitter 0,1 · arrasto 0,05 | 4,21 → 0,64 |
| Charcoal | Noise 48 · papel 0 · carga 24 · dureza 0,1 · Sharp · jitter 0,2 · arrasto 0,02 | 12,03 → 0,88 |
| Oil Pastel | Noise 100 · papel 0,8 · carga 100 · jitter 0,1 · arrasto 0,05 | 5,06 → 1,04 |

Largura independente da posição (y=40 vs 450 dentro de ±1 px). Todos passam `midias_de_fabrica::cada_preset_parece_a_sua_midia`.

## §6 — Portões e mutações

Novos: `wetpaint/pastel_seco_tests.rs` — `o_pastel_seco_nao_se_mexe_depois_do_traco` (Wet Round é o controlo) e
`a_largura_do_pastel_nao_depende_da_altura_na_tela`; mais os dois do §2.
**Mutações 5/5 sangram:** `seco=false` → 2 394 px movidos · PNG antigo do Soft Pastel → a largura varia · recusa
apagada · `natureza` DependeDeOutro · `cobre` sem o ramo das camadas → sangra só o portão «Layer N» (como desenhado).

## §7 — ABERTO

- **Oil Pastel** estria ao longo do traço (nenhum knob varrido as produz).
- **Charcoal:** o zigzag junta mais que o do Rebelle a jitter 0,2 (0,1 separa os traços, NOTA 1,01).
- **Graphite** a 6,5 px reais é pálido (pico 0,59 vs 0,98).
- **Soft Pastel:** falta o halo de pó (0,011 vs 0); miolo mais cheio (0,974 vs 0,914); borda mais firme (subida 5,5 vs 9);
  o 2.º traço a cruzar o 1.º é mais transparente no Rebelle.
- **Decisão do dono (26 B7):** o Soft Pastel de fábrica Size 29 desenha 29 px; o Rebelle Dry Thin «29» desenha 13 px.
  Manter 29?
- A sessão virtual do Rebelle tem prazo de 1 h.

### Incidente (para a bancada)

No início da janela o Rebelle PRÓPRIO do dono estava aberto no ecrã real (`DISPLAY=:0`) com a porta WebSocket. O meu
`gotas_inicia.sh` entregou-se a ele (instância única do Wine) e dois cliques injectados (1567,1049 e 1355,1049) podem
ter chegado à janela dele. Cura sugerida: antes de arrancar, verificar que não há `Rebelle 8.exe` com `DISPLAY=:0` e
RECUSAR (hoje o `gotas_inicia.sh` não verifica; 26 D2).

## §8 — Fecho

Sobre `1e5f79a5a9`: `cargo fmt --check` (3 crates) ✓ · clippy `-D warnings --all-targets` (3 crates) ✓ ·
`nextest-impacted` (merge-base, `PH2D_SUITE=fecho`) **18 447/18 449** em 498 s — os 2 vermelhos são relógio da
máscara (`mask_gate_tests::the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`,
`mask_tests::the_mask_stroke_cost_does_not_follow_the_canvas`) a `loadavg` **140**; isolados 3/3 verdes cada, a `load`
143; o módulo da máscara não foi tocado ⇒ família [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) (candidatos a
membro). A mutação `seco = false` re-sangrou depois da última mudança da lei (3 508 px mexem).

## §9 — Smoke ao dono

Passos numerados na resposta da janela (Wet Paint: Soft Pastel / Graphite / Charcoal / Oil Pastel; linha «Use Texture
Colors» e «Layer N Color» esmaecidas não reagem ao clique).
