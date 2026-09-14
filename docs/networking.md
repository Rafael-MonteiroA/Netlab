# Conceitos de Rede no NetLab

Este documento explica os conceitos de rede real representados no NetLab:
o que cada um é, por que existe, como funciona e como o NetLab o modela.

> **Estado atual:** hosts e switches Ethernet estão implementados na v0.2.0;
> ARP e roteamento IP continuam no roadmap.

---

## MAC Address

### O que é?

O endereço MAC (Media Access Control) é um identificador único de 48 bits
atribuído a uma interface de rede. É o endereço da camada 2 (Ethernet).

### Por que existe?

Permite que dispositivos se identifiquem em uma rede local (LAN) antes de
conhecerem endereços IP.

### Como funciona?

```
PC1 envia um frame Ethernet:
  src MAC: 02:00:00:00:00:01
  dst MAC: 02:00:00:00:00:02
```

O switch usa tabelas de MACs para decidir por qual porta encaminhar o frame.

### Como o NetLab representa?

Tipo próprio `MacAddress` — 48 bits como `[u8; 6]`. Não é uma `String`.

```
02:00:00:00:00:01  →  PC1
02:00:00:00:00:02  →  PC2
FF:FF:FF:FF:FF:FF  →  broadcast
```

Possui parsing, validação, `Display`, `Debug`, `Clone`, `Copy`, `PartialEq`,
`Eq`, `Hash` e testes.

---

## Endereço IPv4

### O que é?

O endereço IPv4 é um identificador de 32 bits (4 octetos) atribuído a um
dispositivo em uma rede IP. É o endereço da camada 3 (Internet).

### Por que existe?

Permite o roteamento de pacotes entre redes diferentes. O MAC endereça
dentro de uma LAN; o IP endereça na Internet.

### Como funciona?

```
PC1: 10.0.0.1
PC2: 10.0.0.2
```

Pacotes IP levam src e dst IPs. Roteadores usam a dst IP para decidir
o próximo salto.

### Como o NetLab representa?

Tipo próprio `Ipv4Address` — 32 bits como `[u8; 4]`. Não é uma `String`.

Possui parsing, validação, `Display` e suporte futuro a prefixo CIDR.

---

## Ethernet

### O que é?

Ethernet é o protocolo de camada 2 (enlace de dados) mais usado em LANs.
Define como bits são transmitidos em um link físico e como frames são
endereçados por MAC.

### Como o NetLab representa?

`EthernetFrame` com `src: MacAddress`, `dst: MacAddress` e `ethertype`.

Na v0.1, toda comunicação é dentro da mesma rede (sem roteamento), então
o frame Ethernet é suficiente para identificar origem e destino no enlace.

---

## Switch Ethernet

Na v0.2.0, uma LAN pode conter um switch com várias portas. O switch aprende
o MAC de origem na porta de entrada e encaminha frames conhecidos apenas para
a porta correspondente. Enquanto o destino ainda não foi aprendido, o frame é
inundado para as demais portas conectadas; frames broadcast seguem a mesma
regra, sem retornar pela porta de entrada.

O aprendizado é determinístico e faz parte do estado da topologia clonada para
cada execução da simulação. Assim, uma execução não contamina a tabela MAC da
seguinte e o mesmo cenário continua reproduzível.

## ICMP

### O que é?

ICMP (Internet Control Message Protocol) é um protocolo de camada 3 usado
para diagnósticos de rede. Não transporta dados de aplicação.

### Por que existe?

Permite verificar conectividade (`ping`) e diagnosticar problemas de rede
(`traceroute`, mensagens de erro de roteamento).

### Como funciona?

```
PC1 → ICMP Echo Request → PC2
PC2 → ICMP Echo Reply   → PC1
```

O `Echo Request` tem tipo=8, código=0.
O `Echo Reply` tem tipo=0, código=0.
Ambos carregam `identifier` e `sequence_number` para correlacionar pares.

### Como o NetLab representa?

`IcmpMessage` com `icmp_type: IcmpType`, `identifier: u16`,
`sequence_number: u16`. Fortemente tipado, não baseado em strings.

---

## Ping

### O que é?

`ping` é uma ferramenta de diagnóstico que envia ICMP Echo Requests e mede
o tempo de ida e volta (RTT — Round Trip Time).

### Como funciona?

```
PC1> ping 10.0.0.2

PING 10.0.0.2
Reply from 10.0.0.2: time=20ms
Reply from 10.0.0.2: time=20ms
Reply from 10.0.0.2: time=20ms

--- statistics ---
sent     = 3
received = 3
lost     = 0
```

O RTT de 20ms vem de: 10ms (ida) + 10ms (volta) com link de latência 10ms.

### Como o NetLab representa?

O ping é uma operação real sobre o motor de simulação. O resultado vem dos
timestamps virtuais dos eventos `PacketSent` e `PacketReceived` — não de
valores fixos.

---

## LAN (Local Area Network)

### O que é?

Uma LAN é uma rede local que conecta dispositivos em um mesmo espaço físico
ou lógico.

### Como o NetLab representa?

`Network` com um nome e um prefixo IPv4 (ex.: `10.0.0.0/24`). Na v0.1,
uma única rede simples conecta os dispositivos.

---

## Link

### O que é?

Um link é a conexão física (ou virtual) entre duas interfaces de rede.

### Por que é importante?

O link define as características físicas da conexão: latência, largura de
banda, taxa de perda de pacotes.

### Como o NetLab representa?

`Link` com:
- `latency_ms: u64` — latência em milissegundos virtuais;
- `enabled: bool` — permite simular links desativados.

Futuro: `bandwidth`, `packet_loss`, `jitter`.

---

## Pacote

### O que é?

Um pacote é a unidade de dados transmitida em uma rede IP.

### Estrutura no NetLab

```
Packet #1
  id:         PacketId(1)
  src_ip:     10.0.0.1
  dst_ip:     10.0.0.2
  protocol:   ICMP
  payload:    IcmpMessage { type: EchoRequest, seq: 1 }
  created_at: SimulationTime(0)
  status:     Received
```

Cada pacote tem um ID único e um ciclo de vida rastreável.

---

## Referências

- [architecture.md](architecture.md) — separação de crates
- [simulation.md](simulation.md) — modelo de tempo virtual e eventos
- [PROJECT_SPEC.md](../PROJECT_SPEC.md) — especificação técnica completa
