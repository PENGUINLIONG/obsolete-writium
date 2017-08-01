# Safety

## Relative Path

Resource locating is strictly controlled by Writus. For relative paths, any
requests targeting resources out of a published directory will be responded with
404 without trying accessing the resources. Even two published directory cannot
access to each other through rel-path. Use absolute path instead.

## Cache Safety

Writus need all directories, except `cache`, to be created ahead of time by the
users. The `cache` directory will be created in-time and removed during
termination, or re-cache. It is important to let the `cache` directory not to be
accessed by other programs. Writus will actively generate cache only once. If
`cache`, or its content, was removed, Writus will try generating requested
article in-time rather than re-caching the requested file. If the cache file was modified, temporarily, Writus will respond with the modified version of cache. To re-cache, input
`recache` in Writus CUI.
