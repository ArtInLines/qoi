# TODOs:

## Remove manual indexing

Currently, all accesses to buffer and pixel slices is done with manual indexing. This can easily be done wrong without noticing and is definitely not a way that is encouraged by Rust.

Solution: Create some kind of wrapper type around these slices, that automates the indexing. This feels very much like a job for Iterators, but the current API for iterators doesn't seem very useful in this case (though maybe I'm just too dumb to find the right methods). Making such a wrapper type should be relatively easy.
