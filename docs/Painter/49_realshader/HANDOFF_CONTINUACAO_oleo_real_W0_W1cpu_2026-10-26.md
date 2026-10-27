# Handoff de continuação — Óleo Real, W0 + W1 em CPU (line/PainterWatercolor, 26/10)

> Continuação (NÃO é handoff de integração: a linha segue). Plano: [`plano.md`](plano.md) · medições: [`README.md`](README.md) §2b ·
> pesquisa: [`pesquisa_tecnologia.md`](pesquisa_tecnologia.md) · próximo prompt: [`PROMPT_NOVA_JANELA_W1b.md`](PROMPT_NOVA_JANELA_W1b.md).
> Veredito do smoke do dono: **«Resultado interessante»** com três problemas maiores (abaixo). A onda está feita mas **rejeitada no olho**.

## 1. Commits da onda (ordem)

| Commit | O quê |
|---|---|
| `71acb7ae40` | W0: medições, sondas, ferramentas (custo na placa, sombra × altitude, escala de altura, folha das 10 espátulas) |
| `53162c5948` | Lei em CPU (`oleo_real.rs`) + ligação ao Wet Paint + sombra no papel por plano à parte |
| `e389e7af4d` | Sonda: folha em PNG (hoje × Óleo Real, afinação `PH2D_OLEO_AF`) + candidatas a espátula |
| `40f84cf5e6` | Switch `PH2D_OLEO_FACA_H3=1` (facas da folha com a lâmina H3) |
| `fd8890c2e0` | Penumbra ANGULAR + calibração `px_por_massa` 0,0015 / penumbra 0,6 |
| `f8c2b8eed7` | Cartão: Real Oil + Depth/Shadow/Gloss/Light height/Light angle; gate `seam_wetpaint_oleo_real` |
| `cf17bb0b4a` | Sonda `medida_oleo_real_acumulo` |
| `5f16f4982f` | fmt/clippy da onda + `pesquisa_tecnologia.md` |

Base: `5f49969d0e..HEAD` (`git log --oneline 5f49969d0e..HEAD`).

## 2. W0 — números (README §2b)

- Custo na placa: RTX 5060 Ti, K=32 N=8 a 4K = **6,6 ms** (tela inteira); a Radeon onboard é **20–25× mais lenta** ⇒ K/N são knobs por placa, o tecto é da Radeon.
- Rebelle: sombra mínima a 42°; **15° derruba o app** (W0(b) = não existe). Luz 84° vs 42° (M10): mais baixa = sombra mais longa.
- 10 espátulas medidas na folha `F42` (`ferramentas/tracos_facas.py`, `regua_facas.py`). O Wet Paint **não tem espátula**.

## 3. A lei (`crates/ph2d-wet-paint/src/oleo_real.rs`)

- **FORMA** = massa suavizada (raio 3) → cavidade e sombra. **DETALHE** = grão 0,15 + fios → só a normal.
- Cavidade HBAO acima do plano tangente, faixa 0,15..0,35; **lei da fenda M2** em linear: `s=0,25`, `k=2,1`.
- Sombra por horizonte com penumbra **ANGULAR** 0,6 (a forma `k·folga/t` limitava a luz no papel livre: bug corrigido em `fd8890c2e0`).
- GGX, F0 0,04. `px_por_massa` = **0,0015** pela régua M6 contra o Rebelle (`regua_sombra.py`): 42·18·0,8 vs 51·12·3.
- Produto: `PainterTool::set_wet_oleo_real` (`wetpaint_settings.rs`); o composite cresce a região por `alcance`; a sombra no papel vive em `WetSession::sombra`, aplicada FORA da tinta com as mesmas gates; o alfa do pigmento continua só cobertura.
- **Desligado = ao byte** (gates verdes por nome).

## 4. Espátula H3 (W0b, só na sonda — não é preset)

Dabber + jitter 0, hardness 1, Constant, sem fios, `dab_quina` 1, `dab_flatten` 0,8 a 90° do traço, PaperGate 0, Leveling 0 ⇒ faixa 98 px, borda 2 px, ruído 1,9 %. **Falta o REBORDO** (a lâmina empurra tinta para os lados: `transfer.rs`, junto de Pickup/Drag). Largura por faca do Rebelle = flatten.

## 5. SMOKE DO DONO 26/10 — veredito

«Resultado interessante». Três problemas maiores:

- (a) **pixels brancos nas bordas** (especular sem cobertura/AA);
- (b) **sombra irreal, parece drop shadow**: não há sombra projectada DENTRO da tinta, só nas bordas;
- (c) **o relevo não se eleva conforme o acúmulo de tinta**.

Pediu pesquisar de onde o Rebelle tirou a tecnologia ⇒ [`pesquisa_tecnologia.md`](pesquisa_tecnologia.md). Prioridade dele: «o mais importante é relevo realista, shader realista, luz e sombras realistas».

## 6. Causa de (c): medida (sonda `medida_oleo_real_acumulo`)

A massa **para na 2.ª passagem** (p50 com 1/2/4/8 passagens): Dabber 3845 → 6226 → 6371 → 6419; Oil Bristle parecido. Mecanismo: **tecto duro de 3000 por célula** em `crates/ph2d-wet-paint/src/trail/transfer.rs` `:268–275` (molhado), `:285–288` (assentado), `:178–181` (pincel seco); a secagem liberta mais um 3000, uma vez. Pickup/leveling/drag **não** são a causa. Auto-sombra interior a 42°: **0 %** (Dabber) / **~2 %** (Oil Bristle) ⇒ sem altura que se acumule não há sombra dentro da tinta.

## 7. Pesquisa (conclusões de `pesquisa_tecnologia.md`)

1. A altura do Rebelle acumula **linearmente sem tecto** (exporta «physical depth» EXR16).
2. Sombra = marcha de raio sobre o campo de altura em shader GL (há fallback em CPU).
3. A oclusão das fendas é um termo PRÓPRIO.
4. RealShader = IBL de uma foto de esfera cinza.

## 8. Vermelhos PRÉ-EXISTENTES (medidos, não são desta onda)

- (i) `ph2d-panel-painter-layers` `censo_dos_controlos::a_linha_esmaecida_e_a_que_nao_age`: 4 linhas do meio Digital (Accumulate/Shape Follow); igual em `f8c2b8eed7~1`.
- (ii) `ph2d-tool-painter` `a_lei_da_tinta::sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes`: Wet Wash (`0x94c4915c9762415f`→`0x47ddb5430c301ebb`), Wet Liner false (`0xc8c30d93e4c6ffd3`→`0xa212ee358c43676e`) e true (`0x88a2105dbd6a9777`→`0x06a9cff290a04135`); vermelho igual em HEAD, `5f49969d0e` e `8f26f1c222` — anterior à regeneração de presets da janela 15.
- (iii) `ph2d-host-code-scenes` `a_cena_3` falhou uma vez sob loadavg 14–57; verde sozinho.
- Ainda abertos da janela 15 (intocados): `a_largura_e_a_prometida` vermelho (Smear Shattered, Smear Wet, Blender raio 120 / Textured 300); smoke da janela 15 adiado pelo dono.

## 9. PRÓXIMO (ordem; é a onda «W1b», ver `plano.md`)

1. **`Grid::espessura`**: espessura por célula escrita no depósito ANTES do tecto de 3000 (`transfer.rs` `:260–283`, `:178`), nunca aparada; levada pelo arrasto (`:296–306`), pelo empurrão do molhado (`molhado.rs:593`) e pelas ferramentas de levantar; a secagem deixa-a. Modelo: `grid/altura.rs` (incluindo patch de undo e gravação). `CampoAltura` lê a espessura (recuo: massa). Recalibrar `px_por_massa` com `regua_sombra.py`.
2. Altura para sombra/oclusão **sem suavizar** (ou raio muito menor); amplitude da textura ∝ espessura; acúmulo na borda (rebordo da faca).
3. Altura e normais **pesadas pela cobertura** na borda + especular × cobertura + AA/oclusão do especular ⇒ cura dos pixels brancos.
4. Cavidade tipo GTAO com rebote dependente do albedo (casa com M2).
5. Sombra por horizonte com **pirâmide de máximos** da altura (Snyder 2008 / Timonen 2010) para sombras longas dentro da tinta baratas; depois o passe de placa (W1 GPU em `ph2d-render`, paridade com a referência CPU; K/N por cartão a partir de W0(a)).
6. Facas: H3 → presets de fábrica Knife Smooth/Textured (+ larguras da F42), depois a lei do rebordo.
7. IBL de foto de ambiente NOSSA / CC0.

## 10. Smoke (para a próxima onda — igual ao de hoje + empilhamento)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run -p ph2d-host-desktop --profile smoke`
2. Meio **Wet Paint**; pincéis Oil → Dabber e Oil → Bristle → Oil Bristle; três pinceladas grossas (clara, média, escura).
3. No cartão do Wet Paint, abaixo de Smooth Edges, marque **Real Oil**.
4. Mexa Light angle / Light height / Shadow / Gloss / Depth.
5. Compare com `capturas/A_base.jpg`.
6. **Novo:** passe 4 vezes sobre a MESMA pincelada: a tinta tem de subir à vista e projectar sombra DENTRO da tinta.
Deu errado se: bordas com pixels brancos; sombra só fora da tinta; 4 passagens iguais a 1.

## 11. Ficou ABERTO

Tudo da §9; os vermelhos da §8; sem preset de espátula; sem passe de placa; handoff de INTEGRAÇÃO por escrever (a linha não fechou).
