import { ref, readonly, computed } from 'vue'
import { useDefra } from './useDefra'
import { useNotifications } from './useNotifications'

/**
 * A relationship tuple: actor has a relation on a resource.
 */
export interface Relationship {
  collection: string
  docId: string
  relation: string
  actor: string
}

/**
 * ACP state and operations for DefraDB access control.
 *
 * This composable is active only when DefraDB is connected and ACP is
 * enabled. When inactive, all operations are no-ops and the UI should
 * hide the access control sections.
 *
 * Uses the same optional-chaining pattern as useDefra: gracefully
 * handles the case where ACP IPC handlers are not available.
 */

const acpEnabled = ref(false)
const acpIdentity = ref('')
const relationships = ref<Relationship[]>([])
const loading = ref(false)

export function useAccessControl() {
  const { healthy } = useDefra()
  const { notify } = useNotifications()

  const available = computed(() => healthy.value && acpEnabled.value)

  /**
   * Enable ACP with the given secp256k1 identity (hex private key).
   */
  function enable(identity: string) {
    acpIdentity.value = identity
    acpEnabled.value = true
  }

  /**
   * Disable ACP. All documents become public.
   */
  function disable() {
    acpIdentity.value = ''
    acpEnabled.value = false
    relationships.value = []
  }

  /**
   * Grant an actor a relation on a document.
   *
   * @param collection - DefraDB collection name (e.g., "ClaspRouterConfig")
   * @param docId - DefraDB document ID
   * @param relation - Relation name (e.g., "operator")
   * @param actor - Actor DID or "*" for public access
   */
  async function grantAccess(
    collection: string,
    docId: string,
    relation: string,
    actor: string,
  ): Promise<boolean> {
    if (!available.value) return false
    loading.value = true
    try {
      const api = (window as any).clasp
      if (api?.acpAddRelationship) {
        const result = await api.acpAddRelationship(collection, docId, relation, actor, acpIdentity.value)
        if (result.success) {
          relationships.value.push({ collection, docId, relation, actor })
          notify('Access granted', 'success')
          return true
        }
      }
      return false
    } catch (e: any) {
      notify(`Failed to grant access: ${e.message}`, 'error')
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * Revoke an actor's relation on a document.
   */
  async function revokeAccess(
    collection: string,
    docId: string,
    relation: string,
    actor: string,
  ): Promise<boolean> {
    if (!available.value) return false
    loading.value = true
    try {
      const api = (window as any).clasp
      if (api?.acpDeleteRelationship) {
        const result = await api.acpDeleteRelationship(collection, docId, relation, actor, acpIdentity.value)
        if (result.success) {
          relationships.value = relationships.value.filter(
            r => !(r.collection === collection && r.docId === docId && r.relation === relation && r.actor === actor)
          )
          notify('Access revoked', 'success')
          return true
        }
      }
      return false
    } catch (e: any) {
      notify(`Failed to revoke access: ${e.message}`, 'error')
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * Check if an actor has a specific relation on a document.
   * This is a local check against cached relationships, not a DefraDB query.
   */
  function hasAccess(collection: string, docId: string, relation: string, actor: string): boolean {
    return relationships.value.some(
      r => r.collection === collection && r.docId === docId && r.relation === relation && (r.actor === actor || r.actor === '*')
    )
  }

  return {
    available: readonly(available),
    enabled: readonly(acpEnabled),
    identity: readonly(acpIdentity),
    relationships: readonly(relationships),
    loading: readonly(loading),
    enable,
    disable,
    grantAccess,
    revokeAccess,
    hasAccess,
  }
}
