# Upload Memory Runbook

Upload-api should stream request bodies. Metadata parsing is allowed to read
only the first MiB before handing the body to object storage. Workers have a
2 GiB memory limit and should not retain full archive bodies in heap memory.

If RSS climbs with large uploads while request rate is normal, inspect recent
body-handling changes before raising memory limits. Restarting workers may
clear pressure temporarily but drops in-flight uploads.
