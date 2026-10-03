# Handoff — `line/3DModeling`: o BRILHO e o ESTILO no render por malha (2026-10-02)

> **Wave, não integração.** Continua [O_RENDER_POR_MALHA](HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md)
> (§8 item 1 da ordem aprovada pelo dono) e a decisão [ADR-0176](../../architecture/decisions/0176-o-render-do-modelador-e-uma-malha-de-jogo.md).
> Base da linha `1ad60a1ce` (main não andou). Commits: `4c37f0907` brilho · `86ad3ac0c` tom de câmara
> gateado + `PH2D_FIELD_BLOOM=1` + sonda da 2.ª cena · `1ab5d5712` clippy · `cda791319` estilo.

## §1 — O que existe

**BRILHO** (`crates/ph2d-mesh-forward/src/gpu_brilho.rs`, `brilho.wgsl`, `ecra.wgsl`)

- O passe da cena escreve a cena-LINEAR num 2.º alvo MRT `Rgba16Float` **só com o brilho ligado**; o
  alvo 0 continua já no olhar ⇒ AA das bordas igual ao de antes.
- Cadeia em passes de **DESENHO** (WebGL2 não tem compute): desce 13-tap com o corte no 1.º degrau
  (`bl_le` com `bl_corte`); sobe a tenda um degrau de cada vez (nível somado por `textureLoad`,
  ping-pong sem blend); o último degrau + `bl_cor` + `vt_to_display` + `bl_compoe` vão no passe do ecrã.
- Lei = `ph2d_bloom::wgsl` (a do Motion e a do Render traçado), bilinear à mão. Pipelines **6 → 10** onde
  a placa desenha `Rgba16Float` com 4× (`Forward::tem_brilho`), compilados uma vez.
- API: `Cena.brilho` · `Forward::tem_brilho` · `Forward::brilho_sobre` (porta de paridade).
- `ph2d_bloom::wgsl::COMPOE` (`bl_compoe`): a composição por pixel numa porta só; o `field-gpu` passou a
  usá-la (era cópia local). `ph2d-bloom` segue com **ZERO dependências de produção** ⇒ a composição CPU
  vive em `ph2d_field_render::soma_halo`, agora `pub` e re-exportada.
- **Não** se reaproveitou o `bloom.wgsl` do Motion (`ph2d-render`): amostra pelo sampler da placa (sem
  paridade CPU) e compõe ANTES do olhar. Reaproveitou-se a LEI, não o ficheiro.

**TOM DE CÂMARA**: já existia (`vt_to_display` no `fs_objeto`; chips de vista/exposição sempre
publicados e na assinatura do quadro). Ganhou gate (§4).

**ESTILO**
- O `fs` segue a ordem do traçado: `mx_indirect` → `st_saturate_indirect` → lâmpadas →
  `st_apply(luz+emissão, |N·V|, k_estilo·raio)`. O uniforme do quadro leva `ph2d_style::wgsl::pack`.
- Subsuperfície maciça lê `|k_material|` (`com_a_curvatura`). ⚠️ ANTES deste porte o canal ia a zero:
  defeito **pré-existente**, curado.
- As duas curvaturas `[material, estilo]` vivem num 2.º buffer de vértices (`Float32x2`,
  `Forward::sobe_curvatura`). `Cena.estilo`, `Cena.raio_da_peca`.
- `ph2d_field_render::curvatura::curvaturas_por` = a mesma conta com o avaliador do chamador;
  `curvaturas` delega, bit a bit.
- `field3d`: `ObjetoRender.campo` (`Arc<FieldDoc>` do grupo). `malha_render_estado` assa as curvaturas
  **noutra thread**, chaveadas por `ChaveCurv (geracao, eps_material, eps_estilo)`; passos =
  `eps_para(raio)` e `Presentation::curvature_eps()`; raio = `bounding_ball(doc)` (a do traçado). O
  quadro espera só a 1.ª curvatura; durante o arrasto da suavidade mostra a anterior; **mudar o estilo
  NÃO re-extrai a malha**.
- Painel: STYLE e BLOOM no Render por malha (BLOOM só se `tem_brilho`).

**Chaves novas:** `PH2D_FIELD_BLOOM=1` (abre com o brilho de fábrica ligado, para fotografar) · sondas
`PH2D_SONDA_BRILHO=1`, `PH2D_SONDA_ESTILO=1`.

**Sonda curada:** o mundo novo de cada cena dava à raiz o MESMO `Entity` e o estado `thread_local` não
recomeçava ⇒ a 2.ª cena era medida com as malhas da 1.ª (44 640 tri, «entrar 0 ms»). Agora limpa com
`sync(sim,false,false)`. ⚠️ Medições de sonda anteriores a `86ad3ac0c` na 2.ª cena não valem.

## §2 — Medições

Release, 1920×1080 a girar, **load 5–13** (⚠️ acima de ~5 o relógio absoluto não vale; os números abaixo
são INTERCALADOS 2× e estáveis, o que vale é o custo relativo).

| cena | sem brilho | com brilho | custo |
|---|---|---|---|
| 36 | 1,15 ms | 1,82 ms | +0,67 |
| 28 | 1,34–1,39 ms | 2,02 ms | +0,65 |

Estilo, cena 35: 1,66 → 1,81 ms (**+0,15**). A 35 tem **262 600** triângulos; entrar 813–887 ms.

**Paridade do brilho vs CPU** (`ph2d_bloom::halo` + `soma_halo`): 5 baterias (fábrica, raio 8, tingido,
tecto 2, anamórfico) × 2 olhares (Standard+0, Neutral+1): **pior 1 byte**. O traçado continua a 1 byte
depois da mudança da `COMPOE`.

**Curvatura interpolada por vértice × campo no centro de cada triângulo** (cena 35, passo de fábrica), em
bytes de TINTA: p50 0 · p99 1 · p99,9 2 · máx 11 (em H·R: p99 0,048 · máx 0,28, mas onde a tinta já
saturou). ⇒ as bordas «retas» da mancha fria na junção caixa/esfera da foto são a curvatura REAL, não
artefacto.

## §3 — ⛔ Recusas MEDIDAS (não reconstrua)

| Recusado | Por quê |
|---|---|
| Portar o `bloom.wgsl` do Motion | sampler de hardware (paridade impossível) e composição antes do olhar |
| MRT sempre ligado | banda e memória pagas com o brilho desligado — só com brilho |
| Blend aditivo nos níveis | exige float-blend que `Features::empty()` não garante — ping-pong |
| Assar a curvatura do estilo dentro do `prepara` | a suavidade é um botão ⇒ re-extrair a cada arrasto; ficou trabalho à parte + buffer à parte |
| Raio = fusão das bolas das unidades | pode diferir da bola do documento do traçado; usa-se `bounding_ball(doc)` |
| Fixtura cinzenta no gate do estilo | a mutação «sem saturação do indirecto» SOBREVIVEU (cinzento sem croma); trocou-se por material colorido |

## §4 — Gates (novos/alterados)

| Crate | Gates |
|---|---|
| `ph2d-mesh-forward` | `o_brilho_acende_fora_da_peca` (mutação «nunca corre» → vermelho) · `nada_compila_ao_editar` e `quadro_pronto_na_hora` com brilho · `a_cor_e_a_lei_da_casa` com 4 olhares (Standard/Neutral, −1..+2) · `o_estilo_e_a_lei_da_casa` (±2 B vs `Style::apply`; mutações sem-saturação 28 B e curvatura-do-material 33 B → vermelho) · `a_subsuperficie_le_a_curvatura_do_material` (mutação → vermelho) · `o_shader_valida_sem_capacidades` inclui `ecra` e `brilho` · `nenhuma_ranhura` inclui `{ESTILO}` |
| `ph2d-app-field3d` | `malha_render_quadro::tests::o_brilho_do_desenhista_de_jogo_e_a_lei_da_cpu` (GPU; mutações sem-soma-do-nível 238 B e sem-tinta 166 B → vermelho) · costura `in_the_mesh_render_the_panel_offers_the_bloom_and_switching_it_on_lights_the_background` (vermelho antes de ligar o painel) · costura `in_the_mesh_render_the_style_tint_and_its_softness_reach_the_frame` (cena 28; mutação «só re-assa com geração nova» → 0 canais) · `malha_render::tests::the_vertex_curvature_interpolates_to_the_fields` (p99,9 ≤ 2 B) |

**Suítes:** `mesh-forward` 11 testes (9 GPU `ignored` por omissão) verdes · `field3d` lib 499 · `field-render`
120 · `bloom` · `style` verdes. clippy `-D warnings` limpo em `mesh-forward`, `bloom`, `field-gpu`,
`field-render`, `field3d`.

## §5 — ABERTO

Herda o §5 **b..h** do [handoff anterior](HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md#5--aberto)
(o item **a** FECHOU: Style e Bloom portados). Acrescenta:

- GLES real / celular real continuam sem medir (sem adaptador GL aqui).
- O halo do brilho **TAPA a grelha do canvas** (a cobertura que o `soma_halo` acrescenta): decisão
  anterior do dono, mantida por paridade; nota em `ph2d_field_render::brilho::soma_halo`.
- A curvatura é medida no campo do **GRUPO**, não da peça inteira: grupos que não se tocam não influem,
  salvo a menos de um passo (≤ 0,15·R) um do outro — **não medido**.
- Relógio absoluto com `load < 5` continua por fazer (os números do §2 são intercalados a load 5–13).

## §6 — Smoke (aprovado pelo dono, 3×)

Brilho (cena 36):
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=36 cargo run -p ph2d-host-desktop --profile smoke`
2. **Model** → **Shading** → **Render** → **BLOOM** On.
3. Deve acontecer: as partes brilhantes ganham halo que sai para fora da peça.
4. Deu errado se: nada muda ao ligar, ou o fundo inteiro fica lavado.

Estilo (cena 35): o mesmo comando com `PH2D_FIELD_SMOKE=35`; no painel **STYLE**: Edge Tint · Cavity
Tint · Curvature Softness · Edge Sharpness · Cavity Sharpness.
⚠️ Os nomes na tela são ESTES; o 1.º roteiro dizia «Convex/Concave Tint» (nomes do código) e o dono não
os achou.

## §7 — Reports do dono

- ✅ Smoke do brilho OK.
- ✅ Smoke do estilo OK **depois da correcção dos nomes**.
- Lição: os nomes do roteiro tiram-se do i18n (`crates/ph2d-i18n/src/model3d_render.rs`), **nunca** do código.

## §8 — ⭐ A PRÓXIMA ONDA

Item 2 da ordem aprovada: **Céu HDRI** — ambiente fotográfico pré-filtrado (irradiância em SH/cubemap
pequeno + radiância por rugosidade), refletido nas peças. As DUAS partes do céu (`ceu_*_sem_caixa` /
`_da_caixa`) continuam a porta: o HDRI dá a parte **sem caixa** e o sol a **da caixa**. Depois: 3 sol CSM ·
4 texturas triplanares · 5 contacto + chão em paridade (§5 d do handoff anterior).
Inegociáveis: `Features::empty()` + limites WebGL2; nada acumulado; pipelines compilados uma vez.

Estado ao fechar a janela: HEAD `cda791319` (+ o commit deste handoff), árvore limpa, binário `smoke`
compilado nesta worktree; **NÃO integrado, NÃO enviado**; `incremental/` NÃO reclamado (a linha continua).
