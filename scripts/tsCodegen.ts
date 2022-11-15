import codegen from '@cosmwasm/ts-codegen';

codegen({
  contracts: [
    // MODULES
    {
      name: 'Fee Module',
      dir: '../contracts/modules/fee'
    },
    {
      name: 'Hub Module',
      dir: '../contracts/modules/hub'
    },
    {
      name: 'Marketplace Module',
      dir: '../contracts/modules/marketplace'
    },
    {
      name: 'Merge Module',
      dir: '../contracts/modules/merge'
    },
    {
      name: 'Metadata Module',
      dir: '../contracts/modules/metadata'
    },
    {
      name: 'Mint Module',
      dir: '../contracts/modules/mint'
    },
    {
      name: 'Permission Module',
      dir: '../contracts/modules/permission'
    },
    {
      name: 'Token Module',
      dir: '../contracts/modules/token'
    },
    {
      name: 'Whitelist Module',
      dir: '../contracts/modules/whitelist'
    },
    // PERMISSION
    {
      name: 'Attribute Permission',
      dir: '../contracts/permissions/attribute'
    },
    {
      name: 'Link Permission',
      dir: '../contracts/permissions/link'
    },
    {
      name: 'Ownership Permission',
      dir: '../contracts/permissions/ownership'
    },
  ],
  outPath: '../ts-types',

  // options are completely optional ;)
  options: {
    bundle: {
      enabled: true,
      bundleFile: 'index.ts',
      scope: 'contracts'
    },
    types: {
      enabled: true
    },
    client: {
      enabled: true,
    },
    messageComposer: {
      enabled: false
    }
  }
}).then(() => {
  console.log('✨ all done!');
});