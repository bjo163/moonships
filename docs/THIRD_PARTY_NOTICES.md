# Third-party notices

## diff 9.0.0

The text diff uses `diff` by Kevin Decker and contributors, pinned to version 9.0.0. The upstream package describes an implementation of Myers's diff algorithm, ships TypeScript declarations, supports a maximum edit-distance bound and has no runtime dependencies.

- Source: [kpdecker/jsdiff](https://github.com/kpdecker/jsdiff)
- License: BSD-3-Clause
- Purpose: bounded local unified text diff (M3-021)

BSD 3-Clause License

Copyright (c) 2009-2015, Kevin Decker <kpdecker@gmail.com>
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

## sha2 0.10.9

The native SHA-256 DevTools use the exact-pinned RustCrypto `sha2` crate and its incremental `Digest` API to hash file streams in bounded memory.

- Source: [RustCrypto/hashes](https://github.com/RustCrypto/hashes)
- Crate documentation: [sha2 on docs.rs](https://docs.rs/sha2/0.10.9/sha2/)
- License: Apache-2.0 or MIT, at the user's option
- Purpose: local SHA-256 text and selected-file hashing (M3-020)

## yaml 2.9.1

The YAML formatter uses `yaml` by Eemeli Aro, pinned to version 2.9.1. The upstream project documents YAML 1.1/1.2 support, browser compatibility, no external runtime dependencies and the ISC license:

- Source: [eemeli/yaml](https://github.com/eemeli/yaml)
- Documentation: [eemeli.org/yaml](https://eemeli.org/yaml/)
- License: ISC
- Purpose: local YAML 1.2 parsing and formatting in DevTools (M3-016)

Copyright Eemeli Aro <eemeli@gmail.com>

Permission to use, copy, modify, and/or distribute this software for any purpose
with or without fee is hereby granted, provided that the above copyright notice
and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS
OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER
TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF
THIS SOFTWARE.
