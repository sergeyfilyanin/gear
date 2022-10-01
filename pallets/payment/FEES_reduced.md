A solution to the problem based exclusively on economic measures could be to increase transaction fees based on the size of the message queue thus raising the barrier for new transactions to enter the system and relieving the pressure on the processing engine.
This is a modification of the standard approach taken in Substrate to adjust fees based on the overall network congestion level. It's not that straightforward, though, due to the nature of transactions in Gear.

Anyway, here's some rationale behind the proposed solution.

### Fees in Substrate

In Substrate, fees usually consist of three terms:
- `base_fee` represents the necessary work validator should do to pull a transaction from the tx pool (regardless of anything),
- `length_fee` which is proportional to the size of an encoded transaction, and
- `weight_fee` which is a function of the transaction weight as meausre of computational load (as discussed earlier).

```
                                            block time
     _______________________________________________________________________________________
    |                     | + + + + + + + + |                                               |
    |                     | +  tx weight  + |                                               |
    |_____________________|_+_+_+_+_+_+_+_+_|_______________________________________________|
                          \________ ________/
                                   V
                           tx processing time
```

`Fee = base_fee + P⋅tx.len + F(tx.weight)`, where `P` - per byte cost, `F(.)` - weight to fee conversion function.

Fees are charged upfront; in fact, if a user doesn’t have enough balance to pay fees, a transaction will be invalidated and dropped even before it gets to be dispathced. Any “leftover” from pre-collected fees is refunded to the user upon the transaction processing completion. This is achieved through keeping track of the already paid fees and weight that was actually spent.

### Accounting for peaks in the network

If the number transactions in the network exceeds some "target" block capacity, Substrate increases the `weight_fee` part by multiplying it by a factor that reflects the level of network congestion:

`Fee = base_fee + P⋅tx.len + 𝛂⋅F(tx.weight)`, where `𝛂 ≥ 1` - dynamically adjusted multiplier.

**Note**: adjusting the `weight-to-fee` conversion function itself is not recommended because it does not have to be linear with respect to weight (even though it normally is), therefore calibrating it in such a way that the resulting fee is proportional to the network congestion can be hard.

### Adding WASM execution to the picture

In Substrate's own `contracts` pallet, adding WASM code execution to extrinsic processing does not change the general way of how fees are charged.

An additional concept of `gas` is introduced in order to
- limit the amount of computations to prevent DoS attacks;
- have an idea of a contract computational complexity to be able to apply fees.

Within the `contracts` pallet, gas is essentially weight: it is measured in the same units. So to calculate the `weight_fee` part, the `gas_limit` is added to the extrinsic weight. This makes sense due to the fact WASM execution is syncronous in `contracts` pallet: the entire transaction is processed in one go (even if there is a chain of nested calls to other contracts) so upon completion the tx executor knows exactly how much gas has been spent and whether there has been changes to the extrinsic weight.

```
                                            block time
     _______________________________________________________________________________________
    |                     | + + + + + + + +  x x x x x x x x x x x x x x |                  |
    |                     | +  tx weight  +  x x       gas_limit     x x |                  |
    |_____________________|_+_+_+_+_+_+_+_+__x_x_x_x_x_x_x_x_x_x_x_x_x_x_|__________________|
                          \_______________________ ______________________/
                                                  V
                                           tx processing time
```

`Fee = base_fee + P⋅tx.len +  𝛂⋅F(tx.weight + tx.gas_limit)`

### What about *Gear*?

In *Gear* a transaction life cycle is more complex: *Gear* is built on top of the *Actor model* with asyncronous messages between the actors. When a transaction is pulled from the tx pool, it may (or may not) result into a `message` being created and pushed to the `MessageQueue`. Furthermore, other outgoing messages can be enqueued as a result of a message processing. The overall time spent on a message is limited by the `gas_limit`, which must be accounted for when calculating fees a user pays for a transaction.

The entire block time is split into two uneven parts: incoming extrinsics processing and messages execution. Each transaction can contribute to both:

```
                                          block time
     _________________________________________/\___________________________________________
    /                                                                                      \
          extrinsics procesing                           messages execution
     _____________________________    ______________________________________________________
    |    | + + + + +|             |  |              | x x x x x x x x x x x |               |
    |    | + weight |             |  |              | x x   gas_limit   x x |               |
    |____|_+_+_+_+_+|_____________|  |______________|_x_x_x_x_x_x_x_x_x_x_x_|_______________|
               \____________                      __________/
                            \                    /
                           two-fold tx contribution
```

### Key features that make *Gear* stand out:

- in *Gear* we have two distinct queues: the tx pool and the `MessageQueue`. They act as a system of communicating vessels, however, the level of fullness of each pool changes independently. This, in particular, means that, gauging the network congestion solely based on the transaction pool size and not taking into account how full or empty the `MessageQueue` is, can be inaccurate;

- messages processing in *Gear* is async, which means a message created by an extrinsic is processed at later time (can easily be even a different block) with respect to the extrinsic itself. As a connsequence, by the time we post-process a dispatched call and calculate the amount to rebate to the user the exact amount of gas that will have been consumed by WASM execution is still unknown;

- in *Gear*, unlike the `contracts` pallet, `Gas` is not an ephemeral entity but it can actually store value and must be expilictly exchanged with currency units. On the other hand, as a computational complexity measure, `Gas` unit always equals `Weight` unit. This means we have to consider a "triangle" diagram of different value representations, which should always commute in order not to let malicious players take advantage of the system:

```
                                Weight
                                /    \
                               /      \
                              /        \
                             /          \
                           Gas ------ Balance
```

In particular, the async nature of the actor model expalins why we shouldn't manipulate the `gas_price()` function: when a message goes for processing, gas is converted to currency and reserved in `Balances` pallet. Upon completion, the unused gas is released back to the user. If the gas price has changed in the meantime, the released amount may differ from what was due, representing another attack vector based on gas price manipulation.

### Proposed solution

The most reasonable idea seems to follow the `contracts` pallet, with a couple of nuances:

- we do not directly add weight and `gas_limit` because they are measured in different units;
- we use two fee multipliers: one for the tx pool, the other for the `MessageQueue`.

The formula will roughly look like so:

`Fee = base_fee + P⋅tx.len +  𝛂⋅F(tx.weight) + 𝛃⋅G(tx.gas_limit), s.t. 𝛂 ≥ 1, 𝛃 ≥ 1`,

where `P` - per byte cost, `F(.)` - weight to fee conversion function, `G(.)` - gas-to-fee conversion function (`gas_price`), `𝛃` - fee multiplier reflecting the message queue congestion level.

**Note**: in current implementation gas is bought and released upon message processing inside the `pallet-gear`, hence it shouldn't be charged in `pallet-transaction-payment`; only the additional term responsible for the message queue size must be dealt with here.

Therefore the fees we'd need to collect in `transaction-payment` pallet will shrink to

`Fee = base_fee + P⋅tx.len +  𝛂⋅F(tx.weight) + (𝛃 - 1)⋅G(tx.gas_limit), 𝛂 ≥ 1, 𝛃 ≥ 1`.

In the UI though, the full fee including the amount to be reserved later should be displayed.

**Pros**:
- the resulting fee is a function of the `gas_limit`, that is encompasses all potential nested calls (outgoing messages).

**Cons**:
- after an extrinsic has been dispatched, the total actual spent gas is not yet known so the leftover can't be fully refunded.

Hence a need for an approximation.

In current implementation we suggest the following:

`Fee = base_fee + P⋅tx.len +  𝝲⋅𝛂⋅F(tx.weight), 𝛂 ≥ 1, 𝝲 ≥ 1`, where `𝝲` is some empirical factor reflecting the fullness of the message queue (generally, different from `𝛃` in the equation above). The `G(tx.gas_limit)` term is still implcitly present (and should be displayed in the UI) but it is not charged until a message is created for an extrinsic and pushed into the message queue.
