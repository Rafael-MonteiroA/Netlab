# Changelog

## [0.2.0] - 2026-09-13

### Added

- Switches Ethernet com portas e tabela de MACs aprendida em tempo de execução.
- Forwarding unicast e flooding de destinos desconhecidos/broadcast.
- Topologias com múltiplos links e ping através de switch.
- Cenário `scenarios/switched_network.toml`.

## [0.1.0] - 2026-09-12

### Added

- Cargo Workspace com resolver 2.
- Crates iniciais para core, rede, pacotes, protocolos e CLI.
- CLI mínima com `--help` e `version`.
- Documentação arquitetural inicial.

### Implemented

- Tempo virtual, eventos e scheduler determinístico.
- Hosts, interfaces, links, endereços MAC/IPv4 e topologia.
- Pacotes Ethernet/IPv4/ICMP e ciclo de vida.
- Entrega direta Host → Link → Host com latência virtual.
- ICMP Echo Request/Reply e ping com RTT derivado dos eventos.
- CLI para executar cenários TOML e inspecionar pacotes.
- Testes automatizados e documentação da implementação.
