# Memory Tide

At 17:35 UTC, upload-api memory began climbing from a stable 420 MiB per
worker to more than 1.8 GiB. The increase is gradual and repeats after worker
replacement. Error rate is still low, but large archive uploads are starting
to fail when workers are evicted.

The on-call note says a deploy landed ten minutes earlier to "simplify body
inspection for upload metadata." Garbage collection warnings appear in the
logs, but CPU is not saturated and request volume is close to baseline.

Find the root cause, show the evidence, and choose a fix that prevents the
next large upload burst from pushing workers out of memory.
