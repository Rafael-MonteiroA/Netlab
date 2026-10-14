# Android e PC por QR Code — NetLab 0.7

O QR Code inicia a conexão Bluetooth; não exige Wi-Fi, dados móveis ou internet.
Ele não substitui o pareamento Bluetooth do sistema nem aumenta o alcance do rádio.
O computador precisa de um adaptador Bluetooth clássico ativo.

## Para testar agora

1. Extraia **todo** o ZIP Windows. Abra `Iniciar-NetLab.cmd` na pasta extraída.
   Se a versão antiga estiver aberta, encerre-a com `Parar-NetLab.cmd` antes.
2. Instale `NetLab-Bluetooth-0.7.apk` no Android como atualização; não desinstale.
3. Ligue o Bluetooth no PC e no celular. Pareie os dois nas configurações do sistema
   e confira o código de pareamento nos aparelhos.
4. No Windows, abra **Bluetooth sem Wi-Fi**, clique em **Atualizar aparelhos**,
   selecione o celular e clique em **Conectar por QR Code**.
5. No app Android: **Aparelhos → Escanear QR Code**. Permita a câmera, aponte para
   o código e confirme **Conectar** ao PC identificado pelo nome do pareamento.
6. Mantenha a aba do PC aberta e aguarde a conexão. O PC tenta novamente se a
   primeira tentativa acontecer antes da leitura; pode levar alguns segundos.
7. Quando os dois indicarem conexão Bluetooth, envie texto nas duas direções.
   Wi-Fi e dados móveis podem ficar desligados durante todo esse fluxo.

Para cancelar: **Desconectar** no PC; **Desconectar e parar** no Android.

## Qual QR Code usar

Use o código da aba **Bluetooth sem Wi-Fi**. O QR da aba **Conexões reais** é um
convite para rede IP e não é aceito pelo leitor Bluetooth deste aplicativo.

Se o PC não aparecer como pareado no celular, o aplicativo explica como parear
e pede nova leitura. Negar a câmera mantém a conexão manual disponível.
Sem adaptador Bluetooth, o Windows mostra um aviso e não inventa um QR funcional.

## Implementação e privacidade

O Windows gera `netlab://bluetooth/v1?pc=AA:BB:CC:DD:EE:FF` usando o endereço real
do rádio. Essa rota só funciona via localhost com a verificação de origem existente.
O código é um identificador, não uma senha. O Android aceita apenas o formato
NetLab v1 e exige que o PC já esteja pareado. Após confirmar, recebe somente desse
endereço; o PC inicia a conexão RFCOMM ao celular selecionado. Não há busca de
sites, upload da câmera ou permissão INTERNET no aplicativo Android.

ZXing Core 3.5.3 faz a leitura local. A biblioteca Apache-2.0 e sua licença ficam
em `android/libs`; a licença também acompanha o APK. Origem:
https://repo.maven.apache.org/maven2/com/google/zxing/core/3.5.3/

SHA-256 do JAR: `8d8064c1636fdaef7189dd9055c7d59950a8940a12f2293956446ec3c109fd82`.
O build verifica esse hash. Android usa Camera API e SurfaceView com liberação
da câmera ao sair da tela; o leitor não depende do Google Play Services.

## Limites de teste

Testes automatizados cobrem o botão React, geração do convite, restrição localhost,
codificação/leitura QR em luminância e rejeição de URLs ou formatos inválidos.
O usuário confirmou o funcionamento nos aparelhos reais após testar a versão 0.7.
O modo Bluetooth continua limitado a um par de aparelhos por vez.
