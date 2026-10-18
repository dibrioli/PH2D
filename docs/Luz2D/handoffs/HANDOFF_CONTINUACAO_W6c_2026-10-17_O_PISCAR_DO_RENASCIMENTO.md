# Handoff de continuação — Luz 2D, W6c: o renascimento sem o quadro sem luz (2026-10-17)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `ad5f8fe39c`, por cima da [W6b](HANDOFF_CONTINUACAO_W6b_2026-10-17_A_VISTA_RENASCE.md) (HEAD `3d89ab67ae`, não
> integrada; smoke do dono OK 17/10). Item 1 do §4 da W6b. Placa RTX 5060 Ti, `wgpu` 29.0.4. Commits: `7fa05d5c77`
> (as duas curas por variável — o binário da régua) · o deste handoff (**a (b) PASSA e fica o único caminho**; a (a) e
> a C1b saem do código; o solver volta byte a byte ao da W6b).

## 1. O mecanismo e o kill-criterion (escrito ANTES de medir)

**Quem segura a fluência velha.** Os três receptores — o passe de sprite (`SpriteRenderer::receber_luz`), a colagem
(`BandBlit::receber_luz`) e o blit do Flip (`FlipCompose::receber_luz`) — guardam um `GrupoDeLuz::aceso`
(`ph2d-render/src/luz_recebida.rs`): um bind group com a vista da fluência do `ligar` ANTERIOR. Na C1b (hoje) a vista
larga-se no fim do `gravar_com` do quadro N e o `ligar` de N vê a luz apagada: o bind group velho cai aí, a textura
morre com os envios de N, e o quadro N+1 monta nos mesmos blocos (12 → 12) — mas N sai sem luz. Montar no início de
N+1 (`67c2e1bdac`, revertido) mantém N aceso, mas a fluência velha ainda está presa pelos bind groups de `ligar(N)`
quando a nova se aloca: 12 → 13 blocos.

**As duas curas** (no binário da régua, `7fa05d5c77`: `PH2D_LIGHT2D_RENASCE_SE_DEGRADA=adota|solta|0`; sem a variável, a C1b):

- **(a) `adota`** — no início de N+1 a vista velha larga-se MENOS a textura da fluência; a placa espera
  (`poll(wait)`) destruir o resto e a nova monta-se com a MESMA textura (`Solver::novo_adotando`), zerada por um
  passe de computação (uma textura nova do wgpu nasce zerada: os bits da nova). Os bind groups velhos apontam para
  ela: nada fica preso. Não toca na shell.
- **(b) `solta`** — no início de N+1, ANTES do `gravar_com`, a shell solta os três receptores
  (`LuzDoQuadro::soltar_se_renasce`, uma linha em `present_luz.rs`); depois larga-se, espera-se e monta-se como na
  C1b, mas no mesmo quadro que desenha.

**Régua** (a da W6b §2–3): 2560×1080 High, cena 5, 6 000 quadros, `CUSTO=1 SERIE=1`, sem `RENASCE`; (a), (b) e OFF,
6 limpos de cada, intercalados em blocos rodados; c2 = mediana do último terço; os blocos do alocador antes → depois do
renascimento. Limpo = a tabela INTEIRA do `nvidia-smi` (C e G) só com kwin, Xwayland, plasmashell, portal, VSCode e o
app, durante todo o processo, e 60 s de placa livre antes; o Chrome do dono fechado; quem tiver alheio é descartado.

**Passa** (cada cura, sozinha): 6/6 ≤ 11,65 · blocos 12 → 12 em 6/6 · OFF com o evento (≥ 1 processo ≥ 12,2; senão a
rodada não vale e repete) · ≤ 2 renascimentos por processo · os bits iguais **e nenhum quadro sem luz** (o portão
`a_vista_renascida_da_os_mesmos_bits_que_a_nova`: `recebida()` presente depois de TODO `gravar`, o do renascimento
incluído; controlo: a C1b reprova essa asserção).
**Morre**: algum ≥ 12,2 com a cura · renasce > 2× · a imagem muda · um quadro sem luz.
**Nem passa nem morre** (ex.: 12 → 13 blocos, ou 11,65 < c2 < 12,2): não substitui a C1b — o piscar fica, e a medida
entra nas recusas.
**As duas passam**: fica a de menor mediana dos 6; diferença < 0,03 ms ⇒ fica a (a) (sem linha na shell nem
obrigação para quem chama). A perdedora sai do código.

## 2. Rodada 1 — NÃO VALE (o detector deixava passar OUTRA linha)

16 processos (binário de `7fa05d5c77`), 21:54–22:57. O detector aceitava pelo NOME `ph2d-host-desktop` («o próprio
app») e `kwin_wayland`: o smoke de outra linha (`line-PainterWatercolor/target/smoke/ph2d-host-desktop`, no seu
`kwin --virtual`) passava por limpo. Visto em 14–16: a c2 das leituras 5–60 (antes de qualquer pico; o MESMO código
nas três variantes) a ~13,0 contra ~11,5, e o processo alheio na tabela, aceite. Antes disso, 09_adota (12,38, que
pela letra mataria a (a)) tinha essa nascença a 12,56, com a carga a 66 e os dois vizinhos sujos de testes de placa
(`it-*`) — os processos curtos escapavam à amostragem de 2 s. Nenhum «limpo» da rodada se prova: só indicativa —
(a) 11,52 · 11,54; (b) 11,47 · 11,59 · 11,52 · 11,49; OFF 12,40 · 12,41 · 12,89; 12 → 12 blocos em todos com cura
(fora o 10_adota sujo, 12 → 13).

**Detector da rodada 2:** o app pela worktree (`cwd` = `line-Luz2D`) e os seus ANTEPASSADOS (o kwin virtual, a
sessão) e os filhos deles (o Xwayland virtual); da área de trabalho, o kwin do `wayland-0`, o Xwayland `:0`,
plasmashell, portal e VSCode; amostragem a 0,5 s. Controlo: com o smoke do Painter na placa, acusa-o
(`4087818:ph2d-host-deskt:line-PainterWatercolor`). Critério de §1 inalterado. Corrigido a meio da rodada 2 (o 3.º
processo): o kwin virtual desta medição está na placa ANTES de o app nascer e DEPOIS de sair, e lia-se alheio (2
amostras, no 1.º e no último segundo) — passou a valer tudo o que DESCENDE do roteiro (`um_processo.sh`); conferido
com o kwin sozinho e com o app na placa. Esse processo (03_0, 12,40) fica fora das contas.

## 3. Rodada 2 — o veredito (binário de `7fa05d5c77`, 17/10 22:56 → 18/10 07:56)

14 processos medidos, 13 limpos (o 03_0 é o falso sujo do §2). Parada 23:39–07:50: o «Monitor do Sistema» do dono
(`plasma-systemmonitor`, G) na placa — o detector esperou a noite inteira (certo pela régua). A (a) e o OFF não
correram até 6: o veredito já estava dado (abaixo); a (b) foi até 6.

| | c2 do último terço (ms) | blocos antes → depois |
|---|---|---|
| **(b) solta** | 11,553 · 11,516 · 11,539 · 11,497 · 11,506 · 11,476 (med. 11,511) | 12 → 12 (6/6) |
| (a) adota | 11,557 · 11,491 · 11,615 · 11,603 | 12 → 12 · 12 → 12 · **12 → 13** · **12 → 13** |
| OFF | 12,462 · 12,539 · 12,446 | 12 → 12 |

- **(b) PASSA:** 6/6 ≤ 11,65 · 12 → 12 em 6/6 · OFF 3/3 ≥ 12,2 (o evento) · 1 renascimento em cada · os bits iguais e
  nenhum quadro sem luz (o portão).
- **(a) não substitui** (nem passa nem morre): o 13.º bloco nasce na leitura do renascimento, com os mesmos 2 230 MiB
  (09: leitura 87; 10: leitura 85). Adotar a fluência não basta: outra coisa da vista velha ainda está viva quando a
  nova se aloca — os receptores seguram a vista, não só a textura. Não investigado (a (b) resolve).
- **Prova da lei «nenhum quadro sem luz»** (mutações, restauradas): **M1** o renascimento de volta ao FIM do quadro
  (a C1b) ⇒ `um quadro saiu sem luz` e o portão dos receptores reprovam; **M2** `soltar_se_renasce` sem soltar ⇒
  `o pedido não soltou o receptor`. A asserção antiga (`acesa()`) era cega ao quadro sem luz (não olha a vista).
- Instrumentos novos da onda (fora do repo, scratchpad da sessão): `alheios.py` (o detector do §2), `um_processo.sh`
  (60 s livres, amostragem 0,5 s, o roteiro do repo), `ronda2.sh` (as 6 permutações; sujo volta à fila; esperar não
  conta) e `analisa.py` (c2 = mediana do último terço; blocos antes = a última leitura antes do renascimento).

## 4. O que ficou no código

`quadro_renasce.rs`: o pedido do vigia (fim do quadro N) renasce no INÍCIO do `gravar_com` de N+1 (`renascer`:
largar, `poll(wait)`, montar no mesmo quadro). `LuzDoQuadro::soltar_se_renasce(gpu, renderer, colagem) -> bool`, que
a shell chama ANTES do `gravar_com` (`present_luz.rs`, +3 linhas: o `true` solta também o `FlipCompose`, que a crate
não conhece fora dos testes). Quem não a chama (sondas, testes) renasce igual e com os bits iguais, mas a 13 blocos.
Portões: `a_vista_renascida_da_os_mesmos_bits_que_a_nova` (agora exige `recebida()` depois de TODO `gravar`) ·
`o_pedido_solta_os_receptores_e_o_ligar_do_mesmo_quadro_acende_os` (o pedido solta só uma vez; o `ligar` acende).

## 5. ABERTO, por ordem

1. Itens 2–3 do §4 da W5z (o berço e o nascer 2× só servem o caminho em buffer — retirar é decisão de produto) ·
   «bloco inteiro × páginas» (W5x §5.2, só noutra placa) · W5w §8 itens 2–4.
2. O piso do vigia inclui a 1.ª leitura (W6b §4.3) — registado, não mexido.
3. Os outros solvers (o entorno) não renascem (W6a §2).
4. Processo: a régua espera 60 s de placa livre; com a máquina só, é ⅓ do tempo (o dono perguntou, 17/10). Proposta
   para a próxima rodada: placa livre ≥ 5 s quando o detector não vê ninguém desde o fim do processo anterior.

## 6. ⛔ Recusas MEDIDAS (somam às da W6b §5, W6a §4, W5z §5, W5y §5 e W5x §8)

| Recusa | Medida |
|---|---|
| (a) a vista nova ADOTA a textura da fluência da velha (zerada) | 12 → 13 blocos em 2/4 (§3); c2 igual à (b). |
| A C1b (renascer no FIM do quadro do pedido) | Um quadro sem luz por renascimento (M1); substituída pela (b). |
| Um detector de «limpo» que aceita o app e o `kwin` pelo NOME | Deixava passar o smoke de outra linha (§2): rodada 1 inteira sem valor. |
| Um detector que aceita o app pelo `cwd` e o kwin por ser ANTEPASSADO dele | Acusa o próprio kwin virtual no 1.º e no último segundo (§2). |

## 7. Para o integrador

Sem contrato congelado, sem schema. API nova: `LuzDoQuadro::soltar_se_renasce` (pública, chamada pela shell). A
`ph2d-light2d-gpu` sai da onda sem diff. Shell: `present_luz.rs` +3 linhas (composição; `the_shell_only_shrinks`
verde). LOC: `quadro.rs` 697/700 (igual à W6b) · `quadro_renasce.rs` 234. A variável `RENASCE_SE_DEGRADA` só aceita
`0` (desliga). CLAUDE.md §5.1 Luz 2D: «Último:» para este handoff quando integrar.

## 8. Fecho da onda e smoke

`cargo fmt --all` · `clippy -D warnings` (`PH2D_SUITE=onda`) de `ph2d-app-light2d`, `ph2d-light2d-gpu` e
`ph2d-host-desktop`: limpos · os testes da onda por nome (`renasce`: 8 verdes, na placa; `the_shell_only_shrinks`,
`workspace_file_loc_cap`: verdes) · a suíte recusou pela cadência (4 commits; o passo é 10). Binário do smoke, 2.ª
corrida:
```
▸ linha line_luz2d · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.36s
```
**Smoke do dono: OK (18/10).** Os passos (o da W6b §7, agora sem o piscar):

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
   no LG (2560×1080), janela maximizada.
2. Settings ▸ 2D Light: Quality High, automático desligado. Na Hierarchy clicar «Lâmpada»; ler no Inspector o
   «NN ms of GPU per frame». Olhar para as sombras dos blocos e esperar 1–2 min.
3. Esperado: o terminal escreve UMA vez «[luz2d] a placa degradou a vista … renasce (1/2)» e, nesse instante, as
   sombras NÃO somem (antes piscavam um quadro); a luz refletida dá um pequeno ajuste em ~0,2 s, como antes. Depois
   disso o número fica ~1 ms abaixo do mesmo teste com `PH2D_LIGHT2D_RENASCE_SE_DEGRADA=0` à frente do comando.
4. Deu errado se: as sombras piscam quando aparece a linha «renasce», a imagem fica diferente depois dela, «renasce»
   aparece com o PC parado mais de 1×, ou o número não desce em relação ao `=0`.
