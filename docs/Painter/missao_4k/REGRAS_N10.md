# Regras da rodada n.º 10 (14/11) — somam-se às [REGRAS_DO_AGENTE.md](REGRAS_DO_AGENTE.md)

Ordem do dono 14/11: atacar TUDO o que está aberto no plano de desempenho, por ordem, sem parar, sem perguntar.
Dúvida ⇒ o estado da arte (padrão-ouro), sem pensar em custo. No fim da rodada a janela principal integra e faz ship.

1. **Leia antes do primeiro comando:** `REGRAS_DO_AGENTE.md` (inteiro), o seu bloco no `BACKLOG.md` (`grep -n '^### B<nn>'`
   + Read com offset/limit — nunca o ficheiro inteiro) e as Recusas MEDIDAS de `PLANO_DESEMPENHO_TOTAL_2026-11-12.md` e do
   handoff `../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_N8_N9_DESEMPENHO.md` §4. Não reconstrua recusa.
2. **NÃO edite `BACKLOG.md`** (quatro agentes em paralelo; um `commit -- BACKLOG.md` levaria as linhas dos outros). Escreva o
   estado de cada item seu em `medicoes/N10_<familia>.md` (uma linha por id: estado, antes → depois, sha, porquê) e
   commite ESSE ficheiro com a cura. A janela principal dobra-o no BACKLOG no fecho.
3. **Confira o código antes de acreditar no BACKLOG:** um item «aberto» pode já estar feito no `main` (`git log --oneline -F
   --grep=<id>`). Se estiver, marque `feito (sha)` no seu N10 e siga.
4. **A máquina tem outros agentes a compilar:** grave `uptime` com cada medição; A×B só no MESMO processo, intercalado, o
   MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`); chave em execução (`PH2D_<X>=0/1` lida uma vez), nunca `const` recompilada.
   Binário base do `main` (36a312482a) para A×B no app: `target/ph2d-base-36a312` (não o apague).
5. **Prazo 45 min.** Antes dos 40: commite o que está verde e devolva «parcial» com o PRÓXIMO passo exato (ficheiro, função,
   número). A janela principal relança-o. Árvore sempre a compilar entre os seus commits.
6. **Nunca `git reset`, `checkout --`, `restore` de ficheiro que não é seu**; nunca `rm -rf target` (só o que você criou).
