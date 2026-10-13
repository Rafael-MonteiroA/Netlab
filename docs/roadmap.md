# Roadmap do NetLab

Este documento descreve o roadmap de desenvolvimento do NetLab, do estado
atual até a visão de longo prazo.

## Atualização — v0.4.0 entregue para teste

Reconexão automática, confirmações, deduplicação e QR Code foram implementados.
Há APK Android e ponte Windows para mensagens por Bluetooth entre um par de
aparelhos. Compilação e testes de software concluídos; homologação física ainda
pendente por falta de rádio/telefones no ambiente. Consulte
[offline-bluetooth.md](offline-bluetooth.md). Grupos Bluetooth, arquivos e
criptografia própria continuam fora desta versão.

## Entrega anterior — v0.3.0

A prioridade desta versão passou a ser uma interface utilizável e comunicação
entre clientes reais. Foram implementados: GUI no navegador, laboratório em
estrela, inspetor de pacotes, servidor WebSocket com salas/convites, mensagens,
presença e ping de aplicação. Consulte [real-network.md](real-network.md).

Próximos passos: homologação em aparelhos físicos e hospedagem HTTPS; depois
controle administrativo de salas, transferência de arquivos e, se necessário,
criptografia de ponta a ponta. ARP/roteadores continuam como evolução do motor.
As versões abaixo preservam o planejamento original, não as datas de entrega.

---

## Estado atual — v0.1.0 concluída

- [x] Cargo Workspace com resolver 2
- [x] Cinco crates separados por responsabilidade
- [x] CLI (`--help`, `version`, `run`, `ping`, `inspect packet`)
- [x] Documentação arquitetural e de pacotes
- [x] Tempo virtual, eventos, scheduler
- [x] Host, Interface, Link, MAC, IPv4
- [x] Pacotes estruturados
- [x] Ping sobre simulação real

---

## v0.1.0 — Fundação

**Objetivo:** provar que o motor consegue simular corretamente uma comunicação
básica entre dois computadores virtuais.

### Etapas

| Etapa | Conteúdo |
|-------|----------|
| 1 | Workspace Rust (✅ concluída) |
| 2 | `SimulationTime`, `Event`, `EventQueue`, `Simulation` (✅ concluída) |
| 3 | `Device`, `Host`, `Network`, `Interface`, `Link`, `MacAddress`, `Ipv4Address` (✅ concluída) |
| 4 | `Packet`, `EthernetFrame`, `Ipv4Header`, `IcmpMessage`, `Payload` (✅ concluída) |
| 5 | Entrega de pacotes Host → Link → Host (✅ concluída) |
| 6 | ICMP Echo Request e Echo Reply (✅ concluída) |
| 7 | Ping com RTT real da simulação (✅ concluída) |
| 8 | CLI: `run`, `ping`, `inspect` (✅ concluída) |
| 9 | Cenários em TOML (✅ concluída para o formato básico) |
| 10 | Documentação (✅ concluída) |
| 11 | Testes finais e qualidade (✅ concluída) |

### Critério de conclusão da v0.1.0

```bash
cargo test --workspace   # todos os testes passam
cargo clippy --workspace --all-targets --all-features  # sem erros críticos
cargo build --release    # compila
cargo run -p netlab-cli -- run scenarios/basic_network.toml  # executa
```

---

## v0.2.0 — Ethernet & Switch

- [x] Switch com MAC address table
- [x] Forwarding unicast e flooding/broadcast
- [x] Topologia com switch e portas múltiplas
- [x] Ping atravessando uma LAN com switch

---

## Planejado — ARP (versão a redefinir)

- ARP Request e Reply
- ARP cache nos hosts
- Integração com Ethernet

---

## v0.4.0 — IPv4 & Roteamento

- IPv4 completo com TTL
- Router com tabela de roteamento
- Subnetting (CIDR)
- Comunicação entre sub-redes

---

## v0.5.0 — ICMP & Traceroute

- ICMP Time Exceeded
- Traceroute

---

## v0.6.0 — UDP

- Sockets UDP virtuais
- Comunicação cliente-servidor

---

## v0.7.0 — TCP

- Three-way handshake (SYN, SYN-ACK, ACK)
- Números de sequência e acknowledgement
- FIN, RST

---

## v0.8.0 — DNS

- Servidores DNS virtuais
- Resolução de nomes

---

## v0.9.0 — HTTP

- Servidores HTTP virtuais
- GET/Response

---

## v1.0.0 — Internet Virtual

- Topologia completa: PC → Switch → Router → DNS → HTTP Server
- NAT
- Firewall

---

## Futuro (sem versão definida)

- **GUI:** edição de links arbitrários e roteadores além do editor web em estrela já entregue
- **Packet Capture:** modo semelhante ao Wireshark
- **ESP32:** integração com hardware físico
- **Redes híbridas:** dispositivos virtuais + ESP32

---

## Referências

- [architecture.md](architecture.md)
- [PROJECT_SPEC.md](../PROJECT_SPEC.md)
