# taidan_proc_macros

Also known as `rdms_proc_macros`. This crate contains proc macros used by either or both [Readymade] and [Taidan].

The code was originally extracted from Taidan itself. Specifically, the generation of language list during compile-time
was handled in this crate and used by Taidan, and later Readymade borrowed the same macro.

We choose to not publish this crate and instead make it used as a submodule in RDMS to speed up development.

[Readymade]: https://github.com/FyraLabs/readymade
[Taidan]: https://github.com/Ultramarine-Linux/taidan
