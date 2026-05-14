# PotLock Core Contracts

PotLock Core contains the NEAR smart contracts that power the PotLock open funding stack. The contracts provide primitives for public goods funding flows such as donations, project registries, funding rounds, sybil checks, and pot deployment.

## Contracts

The contract sources live in [`contracts/`](contracts):

- [`donation`](contracts/donation): donate NEAR or fungible tokens to accounts.
- [`lists`](contracts/lists): manage lists and registrations.
- [`pot`](contracts/pot): manage a configurable funding round.
- [`pot_factory`](contracts/pot_factory): deploy and track Pot contracts.
- [`registry`](contracts/registry): register projects that can apply to funding rounds.
- [`sybil`](contracts/sybil): aggregate sybil resistance providers and human verification stamps.
- [`sybil_provider_simulator`](contracts/sybil_provider_simulator): local simulator for third-party sybil providers.

For higher-level contract documentation, start with the [PotLock contracts overview](https://docs.potlock.io/contracts/contracts-overview).

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/)
- [Yarn](https://yarnpkg.com/)
- [NEAR CLI](https://docs.near.org/tools/near-cli)
- A NEAR testnet account for deployment workflows

## Install

```bash
cd contracts
yarn install
```

## Build

Build one contract at a time from the `contracts` directory:

```bash
yarn build:donation
yarn build:lists
yarn build:pot
yarn build:potfactory
yarn build:registry
yarn build:sybil
yarn build:sybilprovider
```

Each build script enters the corresponding contract directory and runs its local `scripts/build.sh`.

## Tests

The repository includes legacy near-api-js integration tests in [`contracts/test`](contracts/test). These tests are no longer maintained, but the existing commands are:

```bash
cd contracts
yarn test:all
```

See [`contracts/README.md`](contracts/README.md) for the current testing caveats and known issues.

## AI Agent Entry Points

Machine-readable navigation for coding agents is available in [`llms.txt`](llms.txt). It points agents to the contract overview, build commands, test caveats, and the most relevant source directories.

## Contributing

Contributions are welcome. Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) before opening an issue or pull request.

## License

This project is licensed under the [MIT License](LICENSE).

## Links

- Website: [potlock.io](https://potlock.io)
- Docs: [docs.potlock.io](https://docs.potlock.io)
- Telegram: [NEAReFi.org/telegram](https://NEAReFi.org/telegram)
- Twitter: [@PotLock_](https://twitter.com/PotLock_)
