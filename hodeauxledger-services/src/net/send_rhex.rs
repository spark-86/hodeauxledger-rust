use hodeauxledger_core::rhex::rhex::Rhex;
use hodeauxledger_io::net::Transport;

pub async fn send(
    host: &str,
    port: &str,
    rhex_vec: Vec<Rhex>,
) -> anyhow::Result<Vec<Rhex>, anyhow::Error> {
    let mut trans = Transport::new();
    let mut master_vec = Vec::new();
    trans.connect(host, port).await?;
    for rhex in rhex_vec {
        trans.send_rhex(&rhex).await?;
        while let Some(rhex) = trans.recv_next().await? {
            master_vec.push(rhex);
        }
    }
    Ok(master_vec)
}
