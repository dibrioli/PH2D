# CONTINUAÇÃO — `line/motion-value`, o `Collide` da forma corre na placa (11/10)

> As regras R1–R9 do topo do
> [`…_SO_O_ESTADO.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_SO_O_ESTADO.md) valem iguais.
> Espec desta onda: [doc 130](../130_o_passe_na_placa.md).

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, base `8f20c60de` (= `main`).
  **Nada integrado nem enviado.** Onda anterior: [`…_SO_O_ESTADO.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_SO_O_ESTADO.md).
- Commits desta onda: `ef52ffe7e` (kernel + rota + gates) · `26195b539` (duas fixturas pedidas pelas
  mutações) · o deste handoff.

## O que mudou

1. **`ph2d-gpu-cook/src/passe_na_placa.rs`** + cinco `.wgsl` irmãos: a lei do
   `ph2d_contact::passe` porta a porta (colisor declarado, giro, disco/caixa com o TRECHO de dois
   pontos, filtro do repouso, Jacobi com média, `falloff` no fim). Grelha por balde com célula
   `2·alcance_max` (lida da placa), cada balde ordenado por índice e os 9 fundidos em ordem
   CRESCENTE — a ordem de soma da CPU. A paragem (`andou == 0` ou `< 1e-5·alcance_max`) é uma
   bandeira na placa; varreduras depois dela saem logo. Sem leitura de volta.
2. **`saidas.rs`**: a corrente de cada sink passa pelo passe antes do lowering e do passe de formas.
   As varreduras chegam por `GpuCook::set_passe` (uma por sink, como o `set_driven`); vazio = nenhum
   passe ⇒ todo chamador antigo fica igual.
3. **A ponte** (`motion_bridge_gpu.rs` + `_colisor.rs`): `peca_declara_colisor` deixa de recusar e
   arma a **cerca do fim** — se alguma peça declara e nenhuma corrente chegou à placa com coluna de
   colisor, cai para a CPU (`RECUSA_PASSE`, frase nova). O passe conta como estágio que despacha
   (`estagios_que_despacham`, porta única com o censo de rota) ⇒ a `=114` (tudo a montante é
   `sim.step`) vai à rota HÍBRIDA em vez de cair por «fronteira sem estágio».
4. `=122`: `PH2D_PASSE_COLIDE=1` nasce com o `Collide` ligado — só para a FOTO (o clique sintético
   não chega à janela virtual). Sem a variável, desarmada como sempre.

## Medido

| régua | resultado |
|---|---|
| paridade por passo, 9 fixturas × `1, 2, 4, 8` varreduras | pior `2,84e-6·alcance_max`, `1,07e-4°`; varreduras IGUAIS em todas |
| cenas `=114`/`=121`/`=122`/`=123` com `Collide`, `TIQUE 6` | as quatro HÍBRIDAS e aceites; pior `5,96e-8` contra a CPU |
| controlo «o passe mexe» | `=121` `2,65e-2` · `=122` `1,68` · `=123` `1,05e-1` · `=114` `7e-9` (o rapier já separou) |
| custo a 8 varreduras, release, **load 44** (relógio sujo) | `1 000`: CPU `0,85` × placa `2,38 ms` · `10 000`: `6,47` × `2,43` · `32 761`: `22,3` × `4,45 ms` (`5,0×`) |
| app, `=122` armada, `fotografa_cena.sh` | rota `device: HIBRIDO`, `32 copias … pela PLACA`, HUD `61 fps`; foto: fileira de cima separada, a de baixo (campo `falloff 0`) atravessada |

⚠️ A coluna «placa» da sonda inclui o envio e a leitura de volta (o produto não lê); o piso de
`~2,4 ms` é esse ida-e-volta. A `1 000` peças a CPU ainda ganha na sonda — no produto o passe
entra no MESMO `submit` do cozimento.

## Gates e provas

- `ph2d-gpu-cook/tests/it/passe_na_placa.rs`: paridade por passo (9 fixturas), controlo «a placa
  afasta», sem colisor ⇒ sem passe, os nove módulos validam no `naga` sem placa, sonda de custo.
- `motion_bridge_gpu_passe_tests.rs` (`#[ignore]`, placa): as quatro cenas aceites e iguais à CPU.
- `motion_bridge_gpu_collider_tests.rs`: o fio textual (pergunta antes do plano, `set_passe`, a
  cerca nas DUAS rotas) + o irmão `#[ignore]` com a placa (objecto com `Collide` separado na placa).
- **Mutações 12/13 sangram** (quota, sinal, filtro do repouso, `falloff`, paragem, célula, `size`,
  inércia da coluna, trecho, cerca, estágio do passe, varreduras a zero). **M10 sobrevive** (empate
  da face incidente pelo ÚLTIMO mínimo): com a caixa de quina as duas faces empatadas partilham a
  única quina que penetra ⇒ o mesmo contacto. Equivalente na prática, não provado em geral.
- Portão de fecho: ver o fim deste ficheiro.
- ⚠️ **A cerca conta uma saída VAZIA como vista**: um objecto sem imagem não põe coluna de zero
  linhas (`com_colisor`), e a CPU também não separa nada — sem isto a cerca mandava para a CPU toda
  cena com um objecto por carregar (apanhado pelo irmão `#[ignore]` da ponte no portão de fecho).

## ABERTO (nomeado)

- O `Relatorio` (varreduras · peças · vizinhos do perfilador do quadro) não é lido da placa: sob a
  rota da placa o perfilador mostra o último da CPU. Ler de volta é uma leitura assíncrona por medir.
- Cada varredura são ~6 despachos + o `scan`; a `4096` (o tecto do cartão) são ~30 mil despachos por
  quadro, a maioria vazios depois da paragem — o custo de CPU do encode a esse tecto está por medir.
  Os grupos de ligação e o uniform do passe recriam-se por cozimento (não usam as `Ligacoes`).
- A grelha é de UMA camada (`2·alcance_max`): uma peça grande infla a célula de todas, como o
  caminho de omissão da CPU (as duas camadas da CPU estão desligadas por omissão).
- `RECUSA_COLISOR` (o colisor escrito PELO NOME num text param) continua na CPU — fora desta ordem.
- Os abertos anteriores continuam (ver o handoff anterior).

## Próxima janela

Por decidir pelo Enio (o smoke desta onda é dele). Candidatos: o `Relatorio` lido da placa; medir o
encode a `4096` varreduras; o `Boundaries` híbrido do handoff anterior.

## Portão de fecho (`verificador`, sobre `0e75bf12e..HEAD`)

- `nextest-impacted` `3 642/3 643` — a falha era o censo `todo_leitor_do_colisor_declarado_se_regista`
  (a `ph2d-gpu-cook` passou a chamar `ph2d_contact::colisores` no gate): registada com `None` e a
  razão. Depois da cura: `ph2d-app-motion` `1 481/1 481`, `ph2d-gpu-cook` `146/146`.
- clippy `--all-targets -D warnings` e `fmt --check` limpos nas duas crates; gate-fonte da shell
  `the_gpu_cook_recusal_placement` 1/1; censos da árvore combinada `117/117`; tectos de LOC ok
  (`ph2d-gpu-cook/src/lib.rs` `695/700` — o próximo campo da `GpuCook` vai para o `estado.rs`).
- Na placa: `passe_na_placa` 4/4, `passe_tests` 1/1, `collider_tests` com os ignorados 8/8.
- `collision-surface.sh`: schemas, registos e contratos intocados; nenhum pacote externo novo; sem ADR.
