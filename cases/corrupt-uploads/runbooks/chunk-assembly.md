# Chunk Assembly Runbook

Multipart uploads are split into numeric chunk indexes. The assembler must
order chunks by byte offset, not by object key text. Retry attempts may leave
multiple records for the same offset; the latest successful write should win.

Checksum failures that begin at chunk 10 often indicate string sorting:
`0,1,10,11,2` is not valid archive order.
