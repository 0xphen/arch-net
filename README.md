# arch-net

A basic peer-to-peer network application, utilizing the gossip protocol for communication. At startup, each node connects to a designated boot node, serving as a central point of contact for new entrants to the network. Upon successful connection, nodes receive a subset of peers within the network from the boot node. This initial exchange primes the node for further interaction within the network.

# Usage
Run the boot node in a terminal window:

```
$ cargo run --bin boot_node
```

In another terminal, run the node:
```
$ cargo run --bin boot_node
```

You can run as many nodes as you want. The ports are generated dynamically to avoid port-in-use errors.
