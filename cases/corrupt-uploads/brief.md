# Corrupt Uploads

Large archive uploads began failing checksum validation at 18:25 UTC. Small
uploads complete normally. Failed archives usually have at least eleven chunks
and one chunk write retry. Object storage latency is mildly elevated, but the
storage service reports successful writes.

The deploy immediately before the incident refactored multipart assembly to
use object keys directly when ordering chunks. Determine whether storage,
network retries, or assembly logic is corrupting the final archive.
