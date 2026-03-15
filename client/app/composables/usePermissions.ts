// Permission flags matching server/src/permissions.rs
const ADMINISTRATOR = 1n << 0n
const MANAGE_SERVER = 1n << 1n
const MANAGE_CHANNELS = 1n << 2n
const MANAGE_ROLES = 1n << 3n
const MANAGE_MESSAGES = 1n << 4n
const SEND_MESSAGES = 1n << 5n
const READ_MESSAGES = 1n << 6n
const CREATE_THREADS = 1n << 7n
const MANAGE_THREADS = 1n << 8n
const KICK_MEMBERS = 1n << 9n
const BAN_MEMBERS = 1n << 10n

export const PERMS = {
  ADMINISTRATOR,
  MANAGE_SERVER,
  MANAGE_CHANNELS,
  MANAGE_ROLES,
  MANAGE_MESSAGES,
  SEND_MESSAGES,
  READ_MESSAGES,
  CREATE_THREADS,
  MANAGE_THREADS,
  KICK_MEMBERS,
  BAN_MEMBERS,
}

export function usePermissions() {
  const conn = useSpacetimeDB()
  const { identity } = useAuth()

  function isOwner(serverId: bigint): boolean {
    if (!identity.value || !conn?.db?.server) return false
    for (const s of conn.db.server.iter()) {
      if (s.id === serverId && s.ownerIdentity.isEqual(identity.value)) return true
    }
    return false
  }

  function getMemberPermissions(serverId: bigint): bigint {
    if (!identity.value) return 0n
    if (isOwner(serverId)) return ~0n // All bits set

    // Find member
    let memberId: bigint | null = null
    for (const m of conn.db.server_member.iter()) {
      if (m.serverId === serverId && m.identity.isEqual(identity.value)) {
        memberId = m.id
        break
      }
    }
    if (!memberId) return 0n

    // Aggregate role permissions
    let perms = SEND_MESSAGES | READ_MESSAGES | CREATE_THREADS // defaults
    for (const mr of conn.db.member_role.iter()) {
      if (mr.serverMemberId === memberId) {
        for (const r of conn.db.role.iter()) {
          if (r.id === mr.roleId) {
            perms |= BigInt(r.permissions)
            break
          }
        }
      }
    }
    return perms
  }

  function hasPermission(serverId: bigint, perm: bigint): boolean {
    const perms = getMemberPermissions(serverId)
    return (perms & ADMINISTRATOR) !== 0n || (perms & perm) !== 0n
  }

  return { hasPermission, isOwner, PERMS }
}
