# Komple Framework

**WARNING: THIS FRAMEWORK IS IN ALPHA STAGE AND STILL IN DEVELOPMENT. IF YOU WANT TO USE IT IN PRODUCTION APPLICATIONS, USE WITH YOUR OWN RISK.**

More info about Komple Framework can be found in [our documentation](https://docs.komple.io/komple-framework/overview).

Komple Framework is a smart contract framework that provides the tools for creating NFT based applications using [CosmWasm](https://cosmwasm.com).

## Building

### Smart Contracts

Build using Intel optimizer:

```bash
./scripts/optimize -i
```

Build using Arm optimizer:

```bash
./scripts/optimize -a
```

Keep in mind that Arm optimizer should not be used for production builds. 

**Always** use intel optimizer for production builds.

### Contract Schemas

Generate contract schemas using:

```bash
./scripts/generate-schemas
```

### TS Codegen

First install the dependencies in scripts folder:

```bash
yarn install
```

Generate contract schemas with the previous command.

After generating the contract schemas, you can generate the TS code for the contracts using:

```bash
./scripts/ts-codegen
```

## License

Contents of this repository are open source under [Apache License v2](https://www.apache.org/licenses/LICENSE-2.0) or later.
