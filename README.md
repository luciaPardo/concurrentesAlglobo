# tp-concurrentes

- Intervienen 4 entidades: AlGlobo.com, el banco, la aerolínea y el hotel.
- Existe una cola de pagos a procesar que se lee desde un archivo.
- Cada entidad puede aleatoriamente procesar correctamente el cobro o no.
- Si alguna falla, se debe mantener la transaccionalidad y por lo tanto revertir o cancelar apropiadamente.

- AlGlobo.com debe coordinar el pago informando el monto a cobrar a cada entidad de forma concurrente.
  pagos = [{
  "cliente": "lucho",
  "precios": {
  "aerolinea": 20,
  "hotel": 10,
  }
  }, {
  "cliente": "viole",
  "precios": {
  "aerolinea": 500,
  "hotel": 20
  }
  }]

banco = {
"lucho": -30,
"aerolinea": 520,
"hotel": 30,
"viole": -520
}

- La aplicación de AlGlobo.com esta basada internamente en el modelo de Actores, utilizando el framework Actix.
  Todas las aplicaciones deben escribir un archivo de log con las operaciones que se realizan y sus resultados.
- El sistema mantendrá estadísticas operacionales. Para ello debe calcular el tiempo medio que toma un pago en procesarse desde que ingresa el pedido hasta que es finalmente procesado por todas las entidades.
- El sistema de AlGlobo.com es de misión crítica y por lo tanto debe mantener varias réplicas en línea listas para continuar el proceso, aunque solo una de ellas se encuentra activa al mismo tiempo. Para ello utiliza un algoritmo de elección de líder y mantiene sincronizado entre las réplicas la información del la transacción actual.
- Las fallas se guardan en un archivo de fallas para su posterior procesamiento manual. Debe implementarse una utilidad que permita reintentar manualmente cada pedido fallado.

## Hipótesis

- Es posible que las entidades banco, aerolínea y hotel dejen de funcionar ya que son
  independientes del sistema Alglobo con sus réplicas.
- En caso de que el hotel, el banco o la aerolínea dejen de funcionar las transacciones fallarán pero el sistema deberá terminar de forma correcta.
- Todas las transacciones tiene aerolínea y hotel
- Puede haber saldos negativos
- Solamente la etapa de prepare de las transacciones puede fallar. Se asume que la
  etapa de commit nunca falla.

## Flujo

### AlGlobo

- AlGlobo carga pagos desde `pagos.csv`
- Para cada pago:
- - Prepara una transacción en el banco para transferir a la aerolinea
- - Prepara una transacción en el banco para transferir al hotel
- - Prepara una transacción en la aerolínea para confirmar el vuelo
- - Prepara una transacción en el hotel para confirmar la reserva
- - Si falla alguna de estas operaciones: Abort a las restantes
- - Si no: commit(\*) a todo el mundo (que no debería fallar)
- Si falla el pago:
- - Lo guarda en `fallas.csv`
- - Si se procesaron todas las entidades, se manda el tiempo a Stats

### Banco

- Para cada transacción recibida (prepare):
- - Si la transacción debe fallar: Abort. # Random
- - Sino:
- - - lockear el saldo del que paga (en este punto: le sacaste la plata al cliente pero no se la diste al hotel)
- - - responder Ack
- - Si AlGlobo manda Commit: transferir saldo lockeado al destinatario
- - Si AlGlobo manda Abort: devolver saldo lockeado.

### Aerolínea

- Para cada transacción recibida (Prepare):
- - Si la transacción debe fallar: Abort # Random
- - Sino:
- - - responder Ack
- - Si AlGlobo manda Commit
- - Si AlGlobo manda Abort

### Hotel

- Para cada transacción recibida (Prepare):
- - Si la transacción debe fallar: Abort # Random
- - Sino:
- - - "lockear la habitación en el hotel"
- - - responder Ack
- - Si AlGlobo manda Commit
- - Si AlGlobo manda Abort
