# Build Customizable DStack Implementations

Abstraction-based codebase that leverages commands from [flashbox](https://github.com/flashbots/flashbox) to create [dstack](https://github.com/amiller/dstack-vm) implementations app-specific requirements, the goal is to provide a well-defined typed
path for creating working dstack networks but allowing the implementors to fill the building blocks with their own implementations in
order to decide:
- which encryption mechanisms to use (e.g our opinionated implementation uses diffie_hellman x25519-dalek). 
- what data to actually post on chain.
- whether to use an extra pubsub service or not (e.g our is fully on-chain for availability for now).
- which chain to use for the coordination.
- how to pull data from the chain (e.g using light clients vs cloud rpc for availability and safety).
- how to handle migrations.
- and more depending on how the dstack standard pans out.

> Not complete!! Really heavy prototyping phase, everything will probably change. 

I think we'll also want to provide semi-opinionated packed components in `dstack-core` mainly on the tdx side. 

## Components

### Core

The home for the dstack standard paths to be followed and implemented. 

## New york

The opinionated implementation for running myrtle-wyckoff-dstack (hence the new-york naming until we find a better one).
