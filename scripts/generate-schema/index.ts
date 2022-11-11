import codegen from '@cosmwasm/ts-codegen';

codegen({
  contracts: [
    // MODULES
    {
      name: 'Fee Module',
      dir: '../../contracts/modules/fee'
    },
    {
      name: 'HubModule',
      dir: '../../contracts/modules/hub'
    },
    {
      name: 'MarketplaceModule',
      dir: '../../contracts/modules/marketplace'
    },
    {
      name: 'MergeModule',
      dir: '../../contracts/modules/merge'
    },
    {
      name: 'MetadataModule',
      dir: '../../contracts/modules/metadata'
    },
    {
      name: 'MintModule',
      dir: '../../contracts/modules/mint'
    },
    {
      name: 'PermissionModule',
      dir: '../../contracts/modules/permission'
    },
    {
      name: 'TokenModule',
      dir: '../../contracts/modules/token'
    },
    {
      name: 'WhitelistModule',
      dir: '../../contracts/modules/whitelist'
    },
    // PERMISSION
    {
      name: 'AttributePermission',
      dir: '../../contracts/permissions/attribute'
    },
    {
      name: 'LinkPermission',
      dir: '../../contracts/permissions/link'
    },
    {
      name: 'OwnershipPermission',
      dir: '../../contracts/permissions/ownership'
    },
  ],
  outPath: '../../js-types',

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