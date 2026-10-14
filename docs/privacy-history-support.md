# NetLab 0.7 — histórico Bluetooth, privacidade e suporte

## Atualizar

No Android, instale `NetLab-Bluetooth-0.7.apk` sobre a versão anterior, sem desinstalar.
A assinatura de desenvolvimento foi mantida. No Windows, encerre a versão anterior
com `Parar-NetLab.cmd`, extraia todo o novo ZIP e abra `Iniciar-NetLab.cmd`.

## Histórico permanente e recuperação

As conversas Bluetooth, rascunhos e mensagens pendentes agora ficam em SQLite,
com os conteúdos criptografados. Fechar o aplicativo ou reiniciar o aparelho não
apaga esses registros. A tela carrega até 200 mensagens por página; use Mais antigas
e Mais recentes. Não há limite de 200 mensagens para o histórico armazenado.

Android: abra **Aparelhos → Conversas salvas** mesmo sem conexão.
Windows: use **Bluetooth sem Wi-Fi → Conversas salvas neste PC**.
Uma reconexão com o mesmo aparelho reenvia as mensagens ainda pendentes.
Ao reabrir, conecte novamente para retomar o envio. Trocar de conversa encerra a
conexão anterior. A fila aceita até 50 mensagens por aparelho.

Cada mensagem é gravada antes da confirmação de recebimento. Uma falha de gravação
não confirma a mensagem e preserva a possibilidade de reenvio. Identificadores de
mensagens recebidas evitam duplicatas mesmo depois de apagar uma conversa.

Filas da versão anterior são migradas quando disponíveis. Mensagens antigas que
existiam somente em memória e já foram perdidas não podem ser recuperadas.
No Windows, abra a atualização na mesma aba antes de fechá-la se quiser migrar uma
fila antiga ainda guardada na sessão do navegador.

O histórico permanente desta versão se aplica ao **modo Bluetooth**. Salas do
modo Rede IP continuam temporárias e desaparecem quando o servidor reinicia.

## Privacidade e segurança

- Android: conteúdo protegido com AES-256-GCM e chave do Android Keystore.
- Windows: conteúdo protegido com DPAPI da conta do usuário, em
  `%LOCALAPPDATA%\NetLab\history-v1.sqlite`. Funciona pela interface em localhost.
- Não há alternativa de gravação de mensagens em texto puro quando a proteção falha.
- O aplicativo Android continua sem permissão de internet e sem backup automático.
- Bloqueio de aparelhos no NetLab, controle de confirmação de conversa exibida e,
  no Android, opção de ocultar capturas de tela e a prévia de aplicativos recentes.
- A confirmação “Exibida” indica que a conversa estava aberta; não prova leitura humana.

Essa proteção é do armazenamento local. A conexão Bluetooth usa o pareamento do
sistema; **não foi acrescentada criptografia de ponta a ponta ao protocolo**.
Quem controla uma sessão desbloqueada pode abrir o aplicativo e ver as conversas.
Bloquear no NetLab não remove o pareamento do sistema.

Excluir remove conteúdo, rascunho e fila locais, após confirmação. Não apaga a
cópia do destinatário. Marcadores mínimos contra duplicatas e preferências de
bloqueio permanecem; não contêm textos das conversas. A exclusão lógica não é uma
garantia de destruição forense em todos os sistemas de arquivos e backups.

Exportar gera um arquivo de texto **sem criptografia**, escolhido pelo usuário.
Esse arquivo é para consulta; não há importação de backup nesta versão. A recuperação
automática é local: desinstalar, limpar os dados ou perder as chaves/conta pode tornar
o histórico inacessível. Copiar apenas o banco para outro aparelho não transfere as chaves.

## Acabamento e acessibilidade

Textos e botões maiores, foco visível pelo teclado, páginas de configurações com
rolagem, estados e rótulos acessíveis e respeito à preferência de reduzir animações.
O Android acompanha a escala de fonte do sistema. No Windows, **Privacidade e suporte**
oferece Texto ampliado; o zoom do navegador também funciona.

## Diagnóstico para suporte

Android: menu → Diagnóstico para suporte. Windows: Privacidade e suporte.
Gere, revise e, se desejar, copie, salve ou compartilhe o relatório.
Não há envio automático. O relatório gerado contém apenas versão, permissões e
estados técnicos, sem mensagens, nomes, endereços, convites ou identificadores pessoais.
Se você adicionar dados ao editar o relatório, eles serão incluídos ao compartilhar.

## Validação desta entrega

- Testes Rust: comunicação entre clientes, isolamento, reconexão e confirmação;
  persistência DPAPI após reabertura, adulteração, transações e acesso local.
- Testes React: 11 casos, incluindo rascunhos, bloqueio, menu e compartilhamento revisado.
- Testes do cliente Windows: recuperar fila/rascunho, confirmação após salvar,
  prevenção de repetição após exclusão e conexão QR.
- APK de produção executado em emulador Android 14: abertura, interface React,
  Keystore, histórico, rascunho, fila, exclusão, falha de gravação e câmera do scanner.

O usuário confirmou que realizou os testes da versão 0.7 nos aparelhos reais e
que o funcionamento está correto. Essa validação complementa os testes automatizados
e do emulador; não constitui auditoria independente de segurança ou certificação
de acessibilidade.
