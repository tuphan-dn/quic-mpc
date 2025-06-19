# quic-mpc

To connect the boostrap node:

```
cargo run -- --bootstrap /ip4/<public_ip>/udp/8081/quic-v1/p2p/<key> --silent
```

For example,

```
cargo run -- --bootstrap /ip4/13.238.141.54/udp/8081/quic-v1/p2p/12D3KooWMqaRrDRrVbJrWUw15KJRVAmibKaygBdaNMy7EZdF8YLm --silent
```

Bootstrap table

| IPv4          | Key                                                  |
| ------------- | ---------------------------------------------------- |
| 13.238.141.54 | 12D3KooWMqaRrDRrVbJrWUw15KJRVAmibKaygBdaNMy7EZdF8YLm |
| master-1      | TBD                                                  |
| master-2      | TBD                                                  |
