# Simulação no NetLab

Este documento explica o modelo de simulação do NetLab: tempo virtual,
eventos, scheduler, determinismo, latência, ciclo de vida de pacotes e seeds.

> **Estado atual:** os recursos descritos abaixo estão implementados na
> fundação da v0.1.0.

---

## Tempo virtual

O NetLab usa um **relógio virtual** (`SimulationTime`), representado em
milissegundos virtuais. Ele é completamente desacoplado do relógio real do
sistema operacional.

### Por que tempo virtual?

- **Determinismo:** o mesmo cenário com a mesma seed sempre produz os
  mesmos timestamps, independentemente do hardware host.
- **Fast-forward:** `1 segundo de simulação` pode ser executado em
  `< 1 ms real`. Não há `sleep()` no engine.
- **Step-by-step:** no futuro, o usuário poderá avançar um evento por vez
  para fins educacionais.
- **Replay:** eventos com timestamps virtuais podem ser gravados e
  reproduzidos fielmente.

### Representação

```
SimulationTime(0)    →  t = 0 ms
SimulationTime(10)   →  t = 10 ms
SimulationTime(1000) →  t = 1000 ms = 1 s
```

O tempo começa em `t = 0 ms` a cada simulação e avança conforme eventos
são processados.

---

## Eventos

A simulação é **event-driven**. Tudo que acontece na rede é representado
por um evento fortemente tipado.

### Estrutura de um evento

```rust
struct SimulationEvent {
    id: EventId,
    timestamp: SimulationTime,
    event_type: EventType,
}
```

### Tipos de evento planejados

```
PacketCreated(PacketId)
PacketSent { packet_id, from_device, to_link }
PacketTransmitting { packet_id, link_id, from_device }
PacketReceived { packet_id, at_device, ingress_link }
PacketDropped { packet_id, reason }
DeviceOnline(DeviceId)
DeviceOffline(DeviceId)
```

### Exemplo de sequência de eventos

```
t=0ms   PacketCreated(#1)
t=0ms   PacketSent { #1, pc1, link-1 }
t=10ms  PacketReceived { #1, pc2 }        ← latency=10ms
t=10ms  PacketCreated(#2)                 ← Echo Reply
t=20ms  PacketReceived { #2, pc1 }
```

---

## Scheduler

O `Scheduler` é a peça central do motor. Ele:

1. Mantém uma **fila de eventos futuros** ordenada por `SimulationTime`;
2. Fornece o método `schedule(event)` para adicionar eventos;
3. Avança o relógio virtual para o próximo evento com `advance()`;
4. Suporta os modos `run` (processa todos os eventos) e `step` (um por vez).

### Invariante

O scheduler **nunca** retrocede o tempo. Uma vez que `t = 10ms`, nenhum
evento com `timestamp < 10ms` pode ser inserido.

### Determinismo

A fila é uma min-heap por timestamp. Em caso de empate, a ordem é
determinada pelo `EventId` (monotonicamente crescente), garantindo que a
simulação seja 100% reproduzível.

---

## Latência

Cada `Link` possui `latency_ms`. Quando um pacote é transmitido pelo link:

```
arrival_time = send_time + link.latency_ms
```

O scheduler agenda `PacketReceived` em `arrival_time`.

### Modelo de RTT

Se o link tem `latency_ms = 10`:

```
Request:  t=0ms  →  t=10ms  (PC2 recebe)
Reply:    t=10ms →  t=20ms  (PC1 recebe)
RTT = 20ms
```

---

## Ciclo de vida de um pacote

```
CREATED
   |
   v
QUEUED       ← pacote aguarda transmissão
   |
   v
SENT         ← pacote entrou no link
   |
   v
TRANSMITTING ← em trânsito (latência em curso)
   |
   v
RECEIVED     ← chegou ao destino
```

Alternativa em caso de falha:

```
SENT
   |
   v
DROPPED      ← link desabilitado, host offline, packet_loss, etc.
```

Cada transição de estado é registrada como um evento, possibilitando trace
completo do caminho do pacote.

---

## Seeds e determinismo

Para garantir reprodutibilidade em simulações com elementos probabilísticos
(ex.: `packet_loss = 0.1`), a simulação usa uma **seed** fornecida na
configuração:

```toml
simulation_seed = 12345
```

Com a mesma seed e o mesmo cenário, a sequência de eventos aleatórios é
idêntica. Isso permite:

- reproduzir bugs;
- comparar algoritmos;
- gravar e fazer replay de simulações;
- escrever testes determinísticos mesmo com aleatoriedade.

Na v0.1, não há aleatoriedade, mas a infraestrutura de seed está planejada.

---

## Modos de execução

| Modo | Descrição | Status |
|------|-----------|--------|
| `run` | Executa todos os eventos até o fim | Implementado |
| `step` | Avança um evento por vez | Implementado |
| `realtime` | Sincroniza com o relógio real | Futuro |
| `fast-forward` | Executa mais rápido que o tempo real | Futuro |
| `paused` | Suspende a simulação | Futuro |

---

## Observabilidade

Cada etapa importante da simulação produz um log de evento estruturado:

```
[0000ms] PC1 → criando ICMP Echo Request
[0000ms] Packet #1 criado
[0000ms] Packet #1 enviado por PC1 via link-1
[0010ms] Packet #1 recebido por PC2
[0010ms] PC2 → gerando Echo Reply
[0010ms] Packet #2 criado
[0020ms] Packet #2 recebido por PC1
```

Esses logs são produzidos pelo engine como dados estruturados. A CLI (ou GUI)
decide como exibi-los. Não há `println!` espalhados pelo engine.

---

## Referências

- [architecture.md](architecture.md) — separação de crates e responsabilidades
- [PROJECT_SPEC.md](../PROJECT_SPEC.md) — especificação técnica completa
