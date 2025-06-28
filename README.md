# quic-mpc

To connect the boostrap node:

```bash
cargo run -- --bootstrap /ip4/<public_ip>/udp/8081/quic-v1/p2p/<key>
```

For example,

```bash
cargo run -- --bootstrap /ip4/13.238.141.54/udp/8081/quic-v1/p2p/12D3KooWMqaRrDRrVbJrWUw15KJRVAmibKaygBdaNMy7EZdF8YLm
```

Bootstrap table

| IPv4          | Key                                                  |
| ------------- | ---------------------------------------------------- |
| 13.238.141.54 | 12D3KooWMqaRrDRrVbJrWUw15KJRVAmibKaygBdaNMy7EZdF8YLm |
| master-1      | TBD                                                  |
| master-2      | TBD                                                  |

## Init & Update new version

### Init

```bash
git clone -b release https://github.com/tuphan-dn/quic-mpc.git
```

### Update

```bash
git fetch origin
git checkout release
git reset --hard origin/release
```

## Developer Mode

```bash
RUST_LOG=debug cargo run -- ...
```
