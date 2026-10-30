# Bloqueios — o que ficou por fazer e o que falta para destravar

| Onda | Item | O que falta |
|---|---|---|
| W10 | JSON real do Trello do dono | o dono exporta um quadro dele (Menu → Export as JSON) e ele vira a fixture e gate |
| W5 | Kanban ao vivo: furo do NAT entre DUAS máquinas reais | provado só em 127.0.0.1 (dois processos, LAN e QUIC sem ponto de encontro: `ph2d-kanban-sync` `loopback::two_processes_converge_*`); falta a corrida em duas redes de casa — a mesma pendência do Quadro (ADR-0187, «furo real por medir») |
