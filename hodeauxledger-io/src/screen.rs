use hodeauxledger_core::Rhex;
use hodeauxledger_core::crypto::b64::to_base64;
use owo_colors::OwoColorize;

pub fn pretty_print_rhex(rhex: &Rhex) {
    println!("{{");
    println!("  magic: {:?}", rhex.magic.yellow());
    println!("  intent: {{");
    println!(
        "    {}: {}",
        "previous_hash",
        to_base64(&rhex.intent.previous_hash).green().bold()
    );
    println!("    {}: {}", "scope", rhex.intent.scope.green().bold());
    println!("    {}: {}", "nonce", rhex.intent.nonce.green().bold());
    println!(
        "    {}: {}",
        "author_public_key",
        to_base64(&rhex.intent.author_public_key).green().bold()
    );
    println!(
        "    {}: {}",
        "usher_public_key",
        to_base64(&rhex.intent.usher_public_key).green().bold()
    );
    println!(
        "    {}: {}",
        "record_type",
        rhex.intent.record_type.green().bold()
    );
    println!("    {}: {}", "data", rhex.intent.data.green().bold());
    println!("  }}");
    println!("  context: {{");
    println!("    {}: {}", "at", rhex.context.at.yellow());
    println!("  }}");
    println!("  signatures: [");
    for s in &rhex.signatures {
        println!("    {{");
        println!("      {}: {}", "sig_type", s.sig_type.to_string().yellow());
        println!(
            "      {}: {}",
            "public_key",
            to_base64(&s.public_key).green().bold()
        );
        println!("      {}: {}", "sig", to_base64(&s.sig).green().bold());
        println!("    }},");
    }
    println!("  ]");
    println!(
        "  current_hash: {}",
        to_base64(&rhex.current_hash.unwrap()).green().bold()
    );
    println!("}}");
}
