use hodeauxledger_core::Rhex;

use crate::process::scope;
pub fn process_rhex(rhex: &Rhex, first_time: bool) -> Vec<Rhex> {
    let exploded_record_type = rhex.intent.record_type.split(":").collect::<Vec<&str>>();
    let record_major = exploded_record_type[0];

    let result = match record_major {
        "🌐" | "scope" => scope::process_scope_rhex(rhex, first_time),
        //"🔑" | "key" => {}
        //"👑" | "authority" => {}
        //"📩" | "request" => {}
        //"📦" | "record" => {}
        _ => Ok(Vec::new()),
    };
    if result.is_err() {}
    let returned_rhex = result.unwrap();
    returned_rhex
}
