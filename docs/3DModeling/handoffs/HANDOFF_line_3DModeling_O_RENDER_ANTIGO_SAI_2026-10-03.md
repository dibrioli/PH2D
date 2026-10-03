# Handoff — `line/3DModeling`: O RENDER ANTIGO SAI (2026-10-03)

> **Wave, não integração.** Item 1 da ordem do dono de 03/10 (§8 do
> [O_CHAO_QUE_TAPA](HANDOFF_line_3DModeling_O_CHAO_QUE_TAPA_2026-10-03.md)): *«lembre-se que buscamos o
> padrão Unreal/Fortnite ou Plants vs Zombies. Pode apagar o render antigo»*. Base da linha `1ad60a1ce`
> (main não andou). Commits: `9fc7a6bdc` o produto deixa de chegar ao traçado + gate red-first ·
> `43cb2692a` a sonda «Matcap intacto» · `2c829e8b9` o corte (CPU, placa, testes) · `4ba89cb24` o
> `Cargo.lock` · o commit de docs desta wave. **Smoke do dono: PENDENTE.**

## ⛔ As premissas que a medição derrubou

| Premissa (do briefing ou dos mapas) | O que se mediu |
|---|---|
| «o produto ainda chega ao traçado pela env var» | também **sem placa**: o `Feito::SemAparelho` do Render caía no sombreamento traçado da CPU (57 995 de 120 000 px diferentes do Matcap) |
| «o `studio_wgsl` é do Render traçado» (mapa de explorador) | é o céu do **desenhista de malha** — fica |
| «o `probe_doors.rs` da CPU é do Render» (mapa de explorador) | são as portas das sondas da **MARCHA** (`trace_cached_for_test`, `linhas_por_ladrilho_for_test`…) — reposto |
| «o Matcap paga a luz/céu na marcha partilhada» | não paga: já corria o kernel `centro_so`; a luz/céu/chão só serviam o pintor de material e a leitura de volta |
| «a `ph2d-field-render` sai inteira?» | não: fica o Matcap da CPU, a marcha, a cache de fitas, `lowest_point`, `soma_halo`, `PointLamp`/`Surfaces`, `curvatura` |

⚠️ **Os três mapas de explorador erraram em pontos decisivos** (acima, e classificaram como «intactas»
sondas que chamavam o pintor dezenas de vezes). O método que valeu: **cortar pela raiz, deixar o
compilador apontar cada dependente, e provar o que fica com uma impressão digital ao byte**.

## §1 — O que mudou

**Produto (`ph2d-app-field3d`):**
- `smoke_draw.rs`: o Render desenha por malha; com `SemAparelho` o pedido de traçado é o do **Matcap**.
  A thread (`smoke_draw_thread.rs`) só pinta o Matcap: o `Pedido` perdeu `shading/style/bloom/materials/
  lights/ground/refinar`; o `Ready` perdeu `passagem/mais` (o refinamento era só do Render).
- Saiu `malha_render_estado::ligado()` (a porta `PH2D_FIELD_RENDER_TRACADO`) e os seus três leitores
  (`smoke_draw`, `scene.rs` gizmo, `scene_panel.rs`). As fileiras de textura e céu aparecem no Render;
  as do brilho no Render quando a placa tem brilho.
- `gpu_frame.rs`: ficam `shared/para_o_quadro/enabled/takes_the_frame`, `march` (a leitura de volta
  para as PARIDADES: `-> Gbuffer`, sem luzes/chão) e o `pedido` do Matcap; `Sonda` = `escalonar ·
  bordas · longe · entrega · fita_interpretada`; `a_caixa_do_pedido` saiu (era o céu na grade).
  Saíram `paint/paint_com/lamps_that_fit/binding_limit/packed/tests_lampada` e `gpu_frame_chao.rs`.
- Saiu `render_light.rs` (`lights::opening_place` lê o rig por `ph2d_light::resolve`). O
  `StudioSky` vive em `studio.rs` sob `#[cfg(test)]` (os gates de material medem a lei sob o céu
  da malha). `preview.rs` perdeu `a_fita_sai_do_pintor`, `o_campo_do_chao_e_reaproveitado`,
  `o_passo_do_ceu_a_mexer`, `o_ceu_vive_no_tempo`, `PASSO_DO_CEU_A_MEXER`, `refines_occlusion`.
- Env vars mortas: `PH2D_FIELD_RENDER_TRACADO`, `PH2D_FIELD_CHAO_CACHE`, `PH2D_FIELD_FITA_INERTE`,
  `PH2D_FIELD_CEU_PASSO`, `PH2D_FIELD_CEU_TEMPO`, `PH2D_FIELD_AO`, `PH2D_FIELD_LUZ_SEPARADA`.

**`ph2d-field-gpu`:** saíram `paint*` (8), `ceu_tempo*` (3), `sondas_na_placa`, `brilho`,
`trace_lampadas`, `tests/brilho_wgsl_valido.rs`, `examples/{ao_grain,frame_budget}`. O shader da marcha
ficou com `centro_so` + `bordas` + `bordas_marcha`; `Setup` = `w,h,budget,longe · 8 f32 · alvo+edge_cos ·
right · up · fwd` (112 B); **o armazém `luz` (binding 3) saiu ⇒ o Matcap liga 7 armazéns** (era 8; piso
WebGPU 8). `MarchSetup` sem `lamps/n_lamps/ball_*/ao_*/ceu_*/ground/mole`; `Longe` sem
`so_ceu/grade_caixa` (cabeçalho `16 → 15`); `DeviceGbuffer` = `t/normal/edges`; `Alvos` mudou-se
para `trace.rs`. Deps: `ph2d-bloom`, `ph2d-style`, `naga` (dev) saíram.

**`ph2d-field-render`:** saíram `shade_render(+_luz)`, `refine`, `bounce`, `occlusion`, `shadow`,
`sss_shadow`, `probes`, `ground_shade`, `ground_bounce`, `banda`, `march_visibilidade`; o chão traçado
(`ground_sky/catcher_surface/ground_at/ground_points/Ground::hit/UP/LUMA`); `brilho::campo_de_cena`;
`curvatura::{alguem_le,assar_canais,do_gbuffer}` e os canais `curvature/curvature_style` do
`Gbuffer`. Novos: `lampada.rs` (`PointLamp`, `POINT_LAMP_MIN_DISTANCE`) e `superficies.rs`
(`Surfaces`, sem `of/mix_of`).

**Testes:** 40 ficheiros de teste/sonda do modelador cujo sujeito era o Render traçado (ricochete,
luz, SSS, céu no tempo, sondas, pintor, banda, rebordo, brilho/estilo traçados, paridades
CPU↔pintor…) e 9 da CPU do campo (`tests/{banda,chao_*,cornell,luz_encostada,shade_render,shadow}`,
mais as medições de sombra/oclusão do `tests.rs`). **Reapontados ao Matcap** (a maquinaria é a
mesma marcha): `borda_tests` (+ o censo da porta da borda: `3 → 2` caminhos), `borda_sondas`,
`premultiplicado_sondas`, `device_probes_w9_{omissao,torno,placa}`, `preview_device_tests` (as duas
réguas placa-contra-CPU, incl. `na_faixa_do_produto_a_placa_ganha_com_margem`),
`smoke_gpu_sculpt_tests::a_escultura_pinta_se_no_dispositivo`, `matcap_parity_tests` (kernel magro:
nenhuma entrada de luz/céu compila). `smoke_gpu_tests::o_gbuffer…` compara só `t`, normal e bordas.
**Mudaram de casa:** o censo `QUEM_SEMEIA` e `o_material_da_cena_chega_as_esferas` →
`smoke_scenes_materiais_tests.rs`; `vestido()` → `estilo_lei_parity_tests`; a fixtura de três folhas
→ `matcap_parity_tests`; `o_laco_da_resolucao_desconta_a_compilacao` → `preview_device_w9_tests`
(conta `2 → 1`). Ajudante novo: `gpu_frame::matcap_liso` (`#[cfg(test)]`).

`ph2d-vector`: `StableImage::rgba()` (só acrescenta) — o gate lê o quadro.

## §2 — Medições

| o quê | resultado |
|---|---|
| Gate `o_render_sem_aparelho_mostra_o_matcap_e_nao_o_tracado`, ANTES | **VERMELHO**: `57 995` de `120 000` px diferem |
| Gate `ninguem_le_a_porta_do_render_tracado`, ANTES | **VERMELHO**: 2 leitores (`smoke_draw.rs`, `malha_render_estado.rs`) |
| Os dois, DEPOIS | verdes (`0` px, `0` leitores) |
| **Matcap intacto** (`matcap_intacto_sondas`, 27 cenas × 2 câmaras, CPU + placa + placa ampliada `2×`; antes = cópia limpa de `43cb2692a` com alvo próprio) | **54 de 54 impressões iguais ao byte**, e as bordas iguais; a CPU também igual à de antes de qualquer corte |
| `cargo check --workspace --all-targets` | limpo (0 erros, 0 avisos) |
| `cargo machete` (as 4 crates) | limpo |
| Diff | 147 ficheiros, `+559 −40 589` |

## §3 — ⛔ Recusas (não reconstrua)

| Recusado | Por quê |
|---|---|
| Confiar nos mapas «só o Render usa» de explorador | erraram `studio_wgsl` e `probe_doors`; ⇒ corte guiado pelo compilador |
| Manter a luz/céu no shader da marcha «para a paridade» | a leitura de volta só precisa de forma e normal; a luz era do pintor |
| Re-escrever os gates de sombra/oclusão/ricochete contra a malha nesta wave | o sujeito deles (o motor traçado) saiu; a malha mede-se contra o Cycles nos gates dela |
| Apagar as leis partilhadas que perderam o último chamador de fora (§5) | são leis com testes nas crates delas; fora do âmbito «Render traçado» |

## §4 — Gates e prova de mutação

`render_sem_tracado_tests` (filho do `smoke_draw`, porque os campos do viewport são `pub(super)`):
`o_render_sem_aparelho_mostra_o_matcap_e_nao_o_tracado` (gancho de teste
`malha_render_quadro::SEM_APARELHO`) e `ninguem_le_a_porta_do_render_tracado` (varre `crates/` e
`shells/`, piso `> 1000` ficheiros). Re-ancorados: `o_modo_de_omissao_e_pintado_no_dispositivo` (o
ramo da placa passa pela porta, pinta, e o recuo da CPU vem depois) e o controlo de
`a_borda_que_a_mao_arrasta_e_a_de_parar` (era «as cores diferem» — o ricochete do assente; passou a
ser a silhueta ter cobertura parcial).

**Mutações (agente `mutacao`, controlos verdes antes, árvore restaurada): 4 de 5 sangram.**

| mutação | gate | resultado |
|---|---|---|
| `SemAparelho => return` (o Render sem placa não desenha) | G1 | VERMELHO (o quadro não assenta, 120 s) |
| leitor `std::env::var("PH2D_FIELD_RENDER_TRACADO")` em `malha_render_estado` | G2 | VERMELHO |
| o mesmo montado por `concat!` | G2 | sobrevive — limite declarado de uma régua textual |
| `SEM_APARELHO` ignorado (com placa) | G1 | VERMELHO (o quadro não assenta) |
| `if false && takes_the_frame(…)` na thread | G3 | VERMELHO |
| `None.or_else(\|\| pinta_matcap(…))` | G3 | sobrevive — equivalente semântico |

⚠️ As duas do G1 sangram por **tempo** (o quadro deixa de assentar) e não por pixels; a
sensibilidade aos pixels está provada pelo vermelho inicial (`57 995` px).

**Suítes** (verificador, load `4–8`): `ph2d-field-render` lib `53` ✓, `tests/it` `24` ✓ em série (`8`
de orçamento reprovam só em `cargo test` paralelo — contadores globais do processo; sob o `nextest`,
um processo por teste, não se tocam; não vêm desta onda) · `ph2d-field-gpu` `10` + `4` com placa ✓ ·
`ph2d-mesh-forward` `6` + `27` com placa ✓ · `ph2d-viewport3d` `27` ✓ · `ph2d-material` `22` ✓ ·
`ph2d-app-field3d` lib `480` ✓ (+ os 2 re-ancorados, verdes), `tests/it` `4` ✓ (as duas frases de erro
das texturas da onda AS_TEXTURAS estavam fora da tabela — corrigido), gates de placa reapontados ✓.

## §5 — ABERTO

- **Sem placa, o painel do Render mostra estilo/textura/céu sobre um quadro que é o Matcap** —
  controlos mortos só nesse caso (o brilho já se esconde: `tem_brilho`). A cura (esconder pelo
  aparelho) muda o que os gates de painel veem sem placa; decidir com o dono.
- **A lei da lâmpada pontual da malha** (`forward.wgsl`, piso `PISO_LUZ`, queda `1/r²`) não tem gate
  próprio — os que a mediam (`a_light_falls_off_with_the_square_of_the_distance`,
  `a_light_object_lights_the_side_it_is_on`) mediam o desenho traçado. Já era lacuna da malha.
- Os testes de composição do brilho que andavam no pintor traçado (`brilho_tests`) saíram; ficam os
  de `soma_halo` puros e o gate da malha contra o gémeo de CPU (`malha_render_quadro_tests`).
- **Itens partilhados sem chamador fora da crate** (pub ⇒ sem `dead_code`; o integrador confere na
  árvore combinada): `ph2d_field_eval::owners::wgsl::{OwnersWgsl, sem_donos}`,
  `ph2d_field_eval::interp::REGISTOS_DA_MARCHA`, `ph2d_material::bsdf::grazing_dielectric`,
  `ph2d_material::subsurface::integrate_burley`, `ph2d_style::Style::curvature_tinted`.
- `docs/Render3d/` fica como história (aviso no índice); o código que cita já não existe.
- Herdados: §5 do [O_CHAO_QUE_TAPA](HANDOFF_line_3DModeling_O_CHAO_QUE_TAPA_2026-10-03.md#5--aberto).

## §6 — Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=41 cargo run -p ph2d-host-desktop --profile smoke`
2. **MODEL** → **Shading** → **Matcap**: gire a peça; depois **Render**.
3. Deve acontecer: o Matcap igual ao de sempre; o Render igual ao de hoje (as bolas, o cromo, o chão
   escuro por baixo). Nada pisca ao trocar de modo.
4. Deu errado se: o Render ficar vazio ou granulado a assentar (era o traçado antigo), ou o Matcap
   mudar de aspecto.

## §7 — Para o integrador

ids/consts: `CABECALHO` da grade de longe `16 → 15`; armazéns do grupo `0` `6 → 5` (Matcap `8 → 7`);
`smoke_scenes.rs` ganhou `mod materiais_tests` (cenas `CENAS = 41` inalteradas). `PROJECT_SCHEMA`
intacto; shell: `0` linhas nesta wave. Contratos congelados: nenhum. Catracas baixadas: nenhuma
lista numérica — os censos que contavam caminhos foram re-contados (§1). O censo `QUEM_SEMEIA` mora
agora em `smoke_scenes_materiais_tests.rs` (o briefing ainda diz `cor_da_profundidade_tests.rs`).

## §8 — A PRÓXIMA ONDA

Item 2 da ordem do dono: **CAPTURAS DE REFLEXO** (§8 do
[O_CHAO_QUE_TAPA](HANDOFF_line_3DModeling_O_CHAO_QUE_TAPA_2026-10-03.md#8--a-próxima-onda--ordem-do-dono-0310)),
em janela nova, depois do smoke do dono desta.
