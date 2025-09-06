use hodeauxledger_core::Rhex;
use hodeauxledger_core::crypto::b64::to_base64;
use owo_colors::OwoColorize;

pub fn pretty_print_rhex(rhex: &Rhex) -> Result<(), anyhow::Error> {
    println!("{{");
    println!("  {}: {:?}", "🪄", rhex.magic.yellow());
    println!("  {}: {{", "🎯");
    println!(
        "    {}: {}",
        "⬅️🧬",
        to_base64(&rhex.intent.previous_hash).green().bold()
    );
    println!("    {}: {}", "🌐", rhex.intent.scope.green().bold());
    println!("    {}: {}", "🎲", rhex.intent.nonce.green().bold());
    println!(
        "    {}: {}",
        "✍️🔓",
        to_base64(&rhex.intent.author_public_key).green().bold()
    );
    println!(
        "    {}: {}",
        "📣🔓",
        to_base64(&rhex.intent.usher_public_key).green().bold()
    );
    println!("    {}: {}", "📄", rhex.intent.record_type.green().bold());
    println!(
        "    {}: {}",
        "📊",
        serde_json::to_string_pretty(&rhex.intent.data)?
    );
    println!("  }}");
    println!("  {}: {{", "🖼️");
    println!("    {}: {}", "⏱️", rhex.context.at.yellow());
    println!("  }}");
    println!("  🖊️🖊️🖊️: [");
    for s in &rhex.signatures {
        println!("    {{");
        println!("      {}: {}", "🤘", sig_type_to_icon(s.sig_type));
        println!(
            "      {}: {}",
            "🔓",
            to_base64(&s.public_key).green().bold()
        );
        println!("      {}: {}", "🖊️", to_base64(&s.sig).green().bold());
        println!("    }},");
    }
    println!("  ]");
    println!(
        "  {}: {}",
        "⬇️🧬",
        to_base64(&rhex.current_hash.unwrap()).green().bold()
    );
    println!("}}");
    Ok(())
}

fn sig_type_to_icon(sig_type: u8) -> String {
    match sig_type {
        0 => "✍️".to_string(),
        1 => "📣".to_string(),
        2 => "🤝".to_string(),
        _ => "?".to_string(),
    }
}
