# NebulaDB (🦀❤️)
> "In the cosmic dance of data, like the intricate patterns of the Crab Nebula, lies the beauty of discovery. Rust and renewal are but facets of the same process, revealing that in the depths of the digital universe, every byte holds the potential to transform the unknown into the known. This is the essence of each endeavor, with every visit being a voyage through the stellar mists of information." ~ GPT-4 (after a bit of back and forth)


NebulaDB is a Vectordatabase written in (mostly) safe Rust (except the HNSW-Index,
which is implemented using raw pointers due to performance.
Also this does not really matter, the reason beeing that pointers are only used inside the insert function
without any kind of possible interaction with other code or mutable state (only *const is used)).

## Why use NebulaDB
To be honest, you shouldn't consider using NebulaDB in any kind of system at this point in time.
This is only a toy project of mine, which is currently and was never intended to be a production ready database.
Especially aspects like Memory consumption or mmap storage for large indices may never be completed. 

## State of development (sorted by priority)
- [x] Implemented basic vector operations
- [x] Implemented Annoy index and HNSW index
- [x] Basic key value payload storage and basic equality payload query (effectively done)
- [ ] Support multithreaded query executor (:construction:)
- [ ] File storage
- [ ] Implemented sparse Vector support
- [ ] Provide GRPC API to be useable as a single server
- [ ] Add async api
- [ ] Other (idk yet)?

## Usage
:construction: This section is currently under construction :construction:

## Installation
:construction: This section is currently under construction :construction:

## Running as a Docker container
:construction: This section is currently under construction :construction:
