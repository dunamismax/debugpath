# Green CI, Bad Prod

The 16:40 UTC deploy passed CI and rolled out cleanly, but production traffic is
returning 502 from the edge. The service owners report that the container image
digest in production matches the image CI tested. The first edge logs show all
upstreams marked unhealthy within two minutes of the deploy.

Determine why production rejects a build that CI accepted, cite the evidence,
and choose the smallest fix that restores correct routing without removing a
safety check.
