export function useUpload() {
  const config = useRuntimeConfig()

  const isLocalhost = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'
  const uploadUrl = isLocalhost
    ? (config.public.uploadUrl as string)
    : `${window.location.origin}/api`
  const cdnUrl = config.public.cdnUrl as string

  /**
   * Upload a file to the CDN
   * @param file - File to upload
   * @param type - 'avatar' | 'server-icon' | 'emoji' | 'attachment'
   * @param context - userId, serverId, or channelId for path organization
   * @returns { hash, key, url, type, size, name }
   */
  async function upload(file: File, type: string, context: string = 'general') {
    const formData = new FormData()
    formData.append('file', file)

    const headers: Record<string, string> = {}
    const apiKey = config.public.uploadApiKey as string
    if (apiKey) {
      headers['Authorization'] = `Bearer ${apiKey}`
    }

    const res = await fetch(`${uploadUrl}/upload?type=${type}&context=${context}`, {
      method: 'POST',
      body: formData,
      headers,
    })

    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: 'Upload failed' }))
      throw new Error(err.error || 'Upload failed')
    }

    return await res.json()
  }

  /**
   * Get the full CDN URL from a hash
   */
  function getCdnUrl(type: string, context: string, hash: string): string {
    return `${cdnUrl}/${type}s/${context}/${hash}`
  }

  /**
   * Get attachment URL from hash stored in attachment table
   */
  function getAttachmentUrl(hash: string, messageId: string, channelId: string): string {
    return `${cdnUrl}/attachments/${channelId}/${messageId}/${hash}`
  }

  return { upload, getCdnUrl, getAttachmentUrl, cdnUrl }
}
