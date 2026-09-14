# Arquitetura do NetLab

Este documento descreve a arquitetura do NetLab, as responsabilidades de cada
crate e as decisões de design que guiam o desenvolvimento.

---

## Visão geral

O NetLab é organizado como um **Cargo Workspace Rust** com separação explícita
entre o motor de simulação e suas interfaces. A separação garante que:

1. O mesmo cenário possa ser executado pela CLI, por testes automatizados e,
   futuramente, por uma GUI ou adaptador de hardware — sem duplicar a lógica.
2. A GUI (quando existir) possa mudar completamente sem exigir reescrever o
   engine.
3. Testes automatizados possam verificar a simulação sem depender de saída
   de terminal.

```
                         NETLAB ENGINE
                               |
              +----------------+----------------+
              |                |                |
           netlab-cli      (GUI futura)       Tests
              |
              |
      netlab-core  ←── netlab-network
           |                  |
      netlab-protocols ←── netlab-packets
```

---

## Crates

### `netlab-core`

**Responsabilidade:** motor de simulação, agnóstico de apresentação.

Conterá:

- `SimulationTime` — relógio virtual em milissegundos, desacoplado do
  relógio real do sistema operacional;
- `Event` / `EventType` — representação fortemente tipada de tudo que
  acontece na simulação;
- `EventQueue` — fila ordenada por timestamp virtual;
- `Scheduler` — avança o tempo virtual e despacha eventos;
- `Simulation` — ciclo principal: `run()` e `step()`;
- `SimulationConfig` — seed determinística e parâmetros globais;
- `NetlabError` — tipos de erro explícitos.

**Restrições:**

- NÃO deve conhecer detalhes de CLI, GUI, TCP, HTTP ou ESP32.
- NÃO deve produzir saída de terminal.
- NÃO deve usar `unwrap()` indiscriminadamente em código de produção.

---

### `netlab-network`

**Responsabilidade:** modelo de topologia da rede simulada.

Conterá:

- `DeviceId`, `InterfaceId`, `LinkId` — IDs estáveis de entidades;
- `Device` / `DeviceState` — abstração base de dispositivos;
- `Host` — primeiro dispositivo real (nome, MAC, IPv4, interfaces);
- `Interface` — ponto de conexão entre dispositivo e rede;
- `Link` — conecta duas interfaces; possui `latency_ms` e `enabled`;
- `Network` — rede lógica (ex.: `10.0.0.0/24`);
- `MacAddress` — 48 bits, sem `String`, com parsing e validação;
- `Ipv4Address` — 32 bits, sem `String`, com parsing e validação;
- `Topology` — conjunto de dispositivos e links de um cenário.

**Restrições:**

- Representa a estrutura da rede, mas NÃO executa a simulação.
- O `netlab-core` é quem agenda e processa eventos; `netlab-network` é o
  modelo de dados.

---

### `netlab-packets`

**Responsabilidade:** tipos estruturados para pacotes de rede.

Conterá:

- `PacketId` — identificador único de pacote na simulação;
- `PacketStatus` — ciclo de vida: `Created → Queued → Sent → Received / Dropped`;
- `Packet` — estrutura principal com ID, origem, destino, protocolo,
  payload e timestamp;
- `EthernetFrame` — cabeçalho Ethernet com MACs de origem e destino;
- `Ipv4Header` — cabeçalho IPv4 com IPs, TTL e protocolo;
- `IcmpMessage` — tipos ICMP (Echo Request, Echo Reply);
- `Payload` — enum tipado de payloads suportados.

**Princípio:** pacotes são dados estruturados, não strings. Isso permite
inspeção, serialização futura, replay e visualização na GUI sem perda de
fidelidade.

---

### `netlab-protocols`

**Responsabilidade:** comportamento dos protocolos sobre os pacotes.

Conterá (a partir da Etapa 6):

- `IcmpHandler` — processa Echo Requests recebidos e gera Echo Replies;
- `PingSession` — rastreia estado de um ping (enviados, recebidos, RTTs).

**Princípio:** protocolos não produzem saída de terminal. Eles retornam
eventos ou resultados que a camada de apresentação decide como exibir.

Roadmap de protocolos: ICMP → ARP → IPv4 routing → UDP → TCP → DNS → HTTP.

---

### `netlab-cli`

**Responsabilidade:** interface de linha de comando — adaptador sobre o engine.

Comandos planejados:

```
netlab --help
netlab version
netlab run scenarios/basic_network.toml
netlab ping <host> <ip>
netlab inspect packet <id>
```

**Princípio:** a CLI NÃO contém lógica de simulação. Ela apenas comanda o
engine e exibe os resultados estruturados que ele retorna.

---

## Por que o engine é independente da interface?

**Motivo 1 — Testabilidade.** Testes automatizados precisam verificar o
comportamento da simulação sem produzir saída de terminal. Se o engine
dependesse de `println!`, seria impossível testar resultados de forma
estruturada.

**Motivo 2 — Múltiplas interfaces.** O mesmo cenário deve poder ser executado
por CLI, testes, GUI e, futuramente, adaptadores de hardware — sem duplicar
nenhuma regra de simulação.

**Motivo 3 — Determinismo.** Dado o mesmo cenário, a mesma configuração e a
mesma seed, o resultado deve ser idêntico. Isso só é possível se a simulação
for uma função pura do estado de entrada, sem efeitos colaterais acidentais
dependentes de interface.

**Motivo 4 — Extensibilidade.** A GUI pode mudar completamente (de egui para
Tauri, por exemplo) sem exigir modificar o engine.

---

## Modelo de tempo virtual

O NetLab usa um **relógio virtual** (`SimulationTime`), desacoplado do relógio
real do sistema. Isso significa que:

- `1 segundo de simulação` pode ser executado em `< 1 ms real` (fast-forward);
- O tempo avança apenas quando há eventos para processar;
- Não existe `sleep()` ou dependência de `std::time::Instant` no engine.

```
t=0ms   PC1 envia pacote
t=10ms  pacote atravessa link (latency=10ms)
t=10ms  PC2 recebe pacote
t=10ms  PC2 gera Echo Reply
t=20ms  PC1 recebe Echo Reply  →  RTT = 20ms
```

---

## Modelo de eventos

A simulação é **event-driven**:

```
EventQueue
  0ms  → PacketCreated
  0ms  → PacketSent(pc1 → link-1)
 10ms  → PacketReceived(link-1 → pc2)
 10ms  → PacketCreated (Echo Reply)
 20ms  → PacketReceived(link-1 → pc1)
```

O `Scheduler` mantém a fila ordenada por timestamp virtual e avança o
relógio para o próximo evento.

---

## Determinismo

Dado o mesmo cenário + mesma configuração + mesma seed:

- A ordem dos eventos é sempre idêntica;
- Os timestamps virtuais são sempre os mesmos;
- O resultado do ping é sempre o mesmo.

Isso é essencial para testes, debugging, reprodução de bugs e futura
funcionalidade de replay.

---

## Escopo implementado (v0.2.0)

O workspace agora contém a fundação funcional da v0.1.0: relógio virtual,
eventos, scheduler, topologia de dois hosts, pacotes estruturados, ICMP Echo e
uma execução de ping acionada pela CLI. O `netlab-core` continua sem conhecer
topologia ou apresentação; o crate de protocolos orquestra o fluxo usando as
interfaces públicas do core, rede e pacotes.

O NetLab agora também modela switches Ethernet, portas múltiplas, aprendizado
de MAC e forwarding/flooding em uma LAN. ARP, roteamento e os demais
protocolos permanecem no roadmap.

---

## Referências

- [PROJECT_SPEC.md](../PROJECT_SPEC.md) — especificação técnica completa
- [CHANGELOG.md](../CHANGELOG.md) — histórico de versões
- [README.md](../README.md) — visão geral e instruções
