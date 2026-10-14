# internet_oracle — o que a rede de uma casa permite (W22)

Sonda **só de leitura**: pergunta ao router se fala UPnP-IGD (multicast e unicast), NAT-PMP e PCP — sem pedir
mapeamento nenhum — e a servidores STUN públicos (v4 e v6) que endereço vêem, a partir de UMA porta local, para
classificar o mapeamento (independente do destino ⇒ furar o NAT funciona). Análise e tabela das casas:
[`../../11_ao_vivo_pela_internet.md`](../../11_ao_vivo_pela_internet.md).

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone
python3 -I docs/MiroClone/ferramentas/internet_oracle/sonda_rede.py <rótulo-da-casa> \
  > docs/MiroClone/ferramentas/internet_oracle/saidas/<rótulo-da-casa>.json
```

Os endereços públicos saem mascarados (`191.34.x.x`, `2804:7f7::x`) — a saída é versionada; `--sem-mascara` só
para olhar no terminal. Só Python 3 da biblioteca padrão e o comando `ip` (Linux).
