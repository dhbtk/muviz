to do:

- textura para pista
- guard rails luminosos
- pilastras de concreto para pista
- file picker: filtrar por arquivos suportados, scroll na lista

hoje:

- bevy_asset_loader
- melhorar ui: fonte nova, algum design, cena 3d da pista no fundo
- materials decentes para a pista e o concreto
- gerar uns metadados na hora de analisar a música: retornar um hue e saturation para a música
- implementar o guard rail luminoso (ou faixa da pista luminosa?)
- destrinchar como implementar o veículo do player
- explorar a geração de terreno em volta da pista:
    - gerar cadeia de ilhas, explorar múltiplas texturas de terreno (areia da praia, grama, rochas, pedras nas praias
      etc)
- normalizar áudio de alguma forma: aplicar um compressor no som antes e assumir que o pico da música fica em -9 dB
- na geração do track, trazer um bias para voltar o yaw para 0 para que o track siga em direção ao por do sol

notas:

veículo vai ser um carro futurista. light streaks dos postes (usar luz amarela de sódio). guard rails luminosos. olhos
de gato na pista luminosos.
explorar deixar o pôr do sol mais escuro
trajetória vertical do carro é meio que pré calculada. colocar pontos (?) quando o carro estiver pulando

bugs:

- meshes saem com a normal errada caso nao estejam proximas da horizontal ou sla
- a mesh do track ainda sai muito tremida, precisa dar um smoothing nela melhorado
- precisa melhorar a interpolação do track quando muda de equi-temp para equi-dist, de linear pra curvas de bezier ou
  algo assim?
- bpm detectado ou em half-time às vezes
- bpm detectado corretamente mas fora de fase
