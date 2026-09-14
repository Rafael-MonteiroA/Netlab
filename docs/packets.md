# Pacotes no NetLab

Os pacotes da v0.1 são dados estruturados, divididos em cabeçalho Ethernet,
 cabeçalho IPv4 e payload. O payload ICMP possui tipo, identificador e número
 de sequência; não é representado por uma string.

```text
Packet
├── PacketId
├── EthernetFrame { src MAC, dst MAC, ethertype }
├── Ipv4Header { src IPv4, dst IPv4, ttl, protocol }
├── Payload::Icmp(IcmpMessage)
└── PacketStatus + trace
```

O ciclo de vida observado pelo engine é:

```text
Created → Queued → Sent → Transmitting → Received
                                      ↘ Dropped
```

O `PacketId` e o trace são preservados no resultado do ping, permitindo que a
CLI e testes inspecionem a execução sem depender de texto gerado pelo engine.

## Limite da v0.1

Esta versão implementa somente Ethernet/IPv4/ICMP em uma entrega direta entre
dois hosts. ARP, switches, roteamento, UDP e demais protocolos são etapas
futuras.
