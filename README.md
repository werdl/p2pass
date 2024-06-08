# p2pass
> A low-level p2p message passing library for Rust
## Simplicity for the end user
- All that is needed is either an IP and a port, or a base64 encoded p2pass id

## Overall Process
```mermaid
sequenceDiagram
    participant Alice
    participant Bob

    Alice->>Alice: Create ID
    Bob->>Bob: Create ID
    Alice->>Bob: Talks to Bob
    Bob->>Alice: Talks to Alice
```

## POV: Alice

```mermaid
graph TD
    A[Start] --> B[Generate IpAddr]
    B --> C[/Send WAKEUP/]
    C --> D{ACK received?}
    D -->|Y| E[/Send message/]
    D -->|N| C
    E --> F{ACK with hash received?}
    F -->|Y| G{Hash correct?}
    G -->|N| H[/Send ERR/]
    G -->|Y| I[/Send GOODBYE/]

    I --> J([End])
    H --> K[Wait for an arbitrary amount of time]
    K --> C

    F -->|N| C
```

## POV: Bob

```mermaid
graph TD
    A([Start]) --> B{WAKEUP received?}
    B --> |Y| C[Spawn and switch to new thread]
    B --> |N| B

    C --> D[/Send ACK/]

    D --> E{Message received?}
    E --> |Y| F[/Send ACK with hash/]
    E --> |N| E

    F --> G{GOODBYE received?}
    G --> |Y| L[Kill thread] 
    L --> H([End])

    G --> |N| I{Received ERR?}
    I --> |Y| N[Prepare for further messages]
    N --> H
    I --> |N| L
    L --> H

```