import { DbConnection } from '../client/module_bindings/index.js'

const conn = DbConnection.builder()
  .withUri('ws://127.0.0.1:3020')
  .withDatabaseName('discord-clone')
  .onConnect((c, identity, token) => {
    console.log('Connected as', identity.toHexString())
    
    c.subscriptionBuilder()
      .onApplied(() => {
        // Check if we have a profile
        let hasProfile = false
        for (const p of c.db.user_profile.iter()) {
          if (p.identity.isEqual(identity)) { hasProfile = true; break }
        }
        
        if (!hasProfile) {
          console.log('Setting up temp profile...')
          c.reducers.setProfile({ username: '_bot_creator_tmp', displayName: 'Bot Creator' })
        }
        
        setTimeout(() => {
          console.log('Creating FlaviBot...')
          c.reducers.createBot({ name: 'FlaviBot' })
          
          setTimeout(() => {
            console.log('Done!')
            process.exit(0)
          }, 2000)
        }, 1500)
      })
      .subscribe(['SELECT * FROM user_profile'])
  })
  .onConnectError((_c: any, err: any) => { console.error('Error:', err); process.exit(1) })
  .build()

setInterval(() => {}, 1000)
