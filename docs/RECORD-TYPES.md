# Record Types

## Scope (🌐)

-   🌐:💡 = scope:genesis - Beginning of a scope
-   🌐:📩 = scope:request - Requesting a new scope to be created
-   🌐:🟢 = scope:create - Creating a new scope as a child from this one
-   🌐:🔴 = scope:seal - Sealing a child scope for appending

## Policy (📜)

-   📜:🟢 = policy:set - Setting of a scope policy

## Key (🔑)

-   🔑:🟢 = key:grant - Granting of a key to a role
-   🔑:🔴 = key:revoke - Revoking of a key from a role

## Authority (👑)

-   👑:🟢 = authority:grant - Assigns an authority for the scope
-   👑:🔴 = authority:revoke - Revokes an authority for the scope

## Alias (🅰️)

-   🅰️:🟢 = alias:grant - Assigns an alias for the scope
-   🅰️:🔴 = alias:revoke - Revokes an alias for the scope

## Record (📦)

-   📦:📊 = record:data - Generic storage record
-   📦:📝 = record:text - Stores a text node in the chain
-   📦:🖼️ = record:image - Stores an image node in the chain
-   📦:🎵 = record:audio - Stores an audio node in the chain
-   📦:🎥 = record:video - Stores a video node in the chain
-   📦:📄 = record:document - Stores a document node in the chain
-   📦:🔗 = record:link - Stores a link node in the chain
-   📦:📦 = record:package - Header for a chain of records that compile into a package
-   📦:🧩 = record:piece - Chain member for a package
-   📦:🚫 = record:ban - Bans a record from being returned in the scope

## Request (📩)

-   📩:R⬢ = request:rhex - Generic request record
-   📩:📜 = request:policy - Gets the policy for the requested scope
-   📩:🅰️ = request:alias - Gets all the alias records for the scope
-   request:_record_type_ - Convert record type to \_ instead of : and submit, will return all records of that type.

## Steward (💩)

-   💩:🔷 = steward:info
-   💩:❌ = steward:error
-   💩:⚠️ = steward:warning
