# List of Icons and Their Record Types

## Scope

-   💡 = scope:genesis - Beginning of a scope
-   📜 = policy:\* - Setting of a scope policy
    -   ⛓️ = Append Rules
        -   📄 = record_type
        -   🤝☝️ = quorum k required
        -   ↔️ = rate per mark
-   🔑🟢 = key:grant - Granting of a key to a role
-   🔑🔴 = key:revoke - Revoking of a key from a role
-   👑🟢 = authority:grant
-   👑🔴 = authority:revoke
-   🌐🟢 = scope:create - Creating a new subscope
-   🌐🔴 = scope:seal - Sealing a subscope from append
-   📩 = request: - All request record types
-   📦 = record: - All record storage types
-   💩 = steward: - Steward alert types, can be followed with others for more detail
    -   💩🔷 = steward:info
    -   💩❌ = steward:error
    -   💩⚠️ = steward:warning
-   🧠 = VeroNeko observations
-   🧐 = VeroNeko thoughts

## R⬢ (Rhex) Record Fields

-   🪄 [magic] - Magic is the control bytes. First 4 are always "RHEX", the next is 8 flags, and the final byte is the version number.
    -   ✨ [version] - version number. Incremental.
-   🎯 [intent] - The intent is the initial information provided by the Author.
    -   ⬅️🧬 [previous_hash] - Hash of the previous record
    -   🌐 [scope] - Scope name
    -   🎲 [nonce] - Nonce to prevent replay attacks
    -   ✍️🔓 [author_public_key] - Author's public key
    -   📣🔓 [usher_public_key] - Usher's public key
    -   📄 [record_type] - Record type
    -   📊 [data] - Record data in JSON format
-   🖼️ [context] - The context is the information provided by the Usher at the time of submission.
    -   ⏱️ [at] - Time in micromarks. Time is monotonic and starts at 0 so no need for i128 here
-   🖊️🖊️🖊️ [signatures] - Here lies the Author, Usher, and Quorum. Three friends that bind each other in honesty.
    -   🤘 [sig_type] - 0 = Author, 1 = Usher, 2 = Quorum
    -   🔓 [public_key] - Public key of the signer
    -   🖊️ [signature] - Ed25519 signature of the hash so it's always 64 bytes
-   ⬇️🧬 [current_hash] - Hash of the complete record

## Misc

-   📐 = schema
-   ⛓️ = allowed rhex rules
-   ⛓️‍💥 = invalid rhex
-   🥐 = roles
-   ↔️ = rate
-   🤝☝️ = quorum count required
