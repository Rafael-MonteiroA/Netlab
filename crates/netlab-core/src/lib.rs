//! `netlab-core` — Motor de simulação determinístico do NetLab.
//!
//! Este crate é o núcleo agnóstico de apresentação do NetLab. Ele será
//! responsável por:
//!
//! - **Tempo virtual** (`SimulationTime`): um relógio próprio, desacoplado do
//!   relógio real do sistema operacional, permitindo execução determinística,
//!   fast-forward e step-by-step.
//!
//! - **Eventos** (`Event`, `EventType`): representação fortemente tipada de
//!   tudo que acontece na simulação (pacote criado, enviado, recebido,
//!   dispositivo online/offline, etc.).
//!
//! - **Fila de eventos** (`EventQueue`): estrutura de dados que mantém os
//!   eventos ordenados por timestamp virtual.
//!
//! - **Scheduler** (`Scheduler`): responsável por agendar eventos futuros e
//!   avançar o tempo virtual.
//!
//! - **Configuração** (`SimulationConfig`): seed determinística, modo de
//!   execução e parâmetros globais da simulação.
//!
//! - **Erros** (`NetlabError`): tipos de erro explícitos, sem `unwrap()`
//!   indiscriminado.
//!
//! - **Ciclo da simulação** (`Simulation`): loop principal que consome eventos
//!   e distribui para os handlers registrados.
//!
//! # Separação de responsabilidades
//!
//! Este crate **não** deve conhecer detalhes de:
//!
//! - `netlab-cli` (formatação de saída, argumentos de linha de comando);
//! - `netlab-network` (topologia, dispositivos, links);
//! - `netlab-packets` (estrutura de pacotes Ethernet/IPv4/ICMP);
//! - GUI, TCP, HTTP ou ESP32.
//!
//! Interfaces externas apenas comandam a simulação e observam seus resultados
//! como dados estruturados.
//!
//! O crate permanece agnóstico de topologia, pacotes e interface de usuário.

pub mod config;
pub mod error;
pub mod event;
pub mod scheduler;
pub mod simulation;
pub mod time;

pub use config::{SimulationConfig, SimulationMode};
pub use error::NetlabError;
pub use event::{Event, EventId, EventType, SimulationEvent};
pub use scheduler::{EventQueue, Scheduler};
pub use simulation::Simulation;
pub use time::SimulationTime;
