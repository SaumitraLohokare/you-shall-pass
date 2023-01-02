use anyhow::{anyhow, Result};
use clap::{arg, command, Command};

mod store;

#[tokio::main]
async fn main() -> Result<()> {
    let store = store::Store::new("file://data.db").await?;

    let flags = command!()
        .subcommand(
            Command::new("save")
                .about("Save a new password.")
                .arg(arg!(-u --username <VALUE> "Username"))
                .arg(arg!(-p --password <VALUE> "Password")),
        )
        .subcommand(
            Command::new("get-pass")
                .about("Get the password for a username.")
                .arg(arg!(-u --username <VALUE> "Username")),
        )
        .subcommand(
            Command::new("update-pass")
                .about("Update the password for a username.")
                .arg(arg!(-u --username <VALUE> "Username"))
                .arg(arg!(-p --password <VALUE> "New Password")),
        )
        .subcommand(
            Command::new("delete")
                .about("Delete entry for username.")
                .arg(arg!(-u --username <VALUE> "Username")),
        )
        .get_matches();

    if let Some(flags) = flags.subcommand_matches("save") {
        let u = flags
            .get_one::<String>("username")
            .ok_or_else(|| anyhow!("No valid username entered."))?
            .to_owned();
        let p = flags
            .get_one::<String>("password")
            .ok_or_else(|| anyhow!("No valid password entered"))?
            .to_owned();

        let _ = store.store_password(u, p).await?;
        println!("Successfully stored password.");
    } else if let Some(flags) = flags.subcommand_matches("get-pass") {
        let u = flags
            .get_one::<String>("username")
            .ok_or_else(|| anyhow!("No valid username entered."))?
            .to_owned();

        let pass = store.get_password_for(u).await?;
        println!("Password is: {pass}");
    } else if let Some(flags) = flags.subcommand_matches("update-pass") {
        let u = flags
            .get_one::<String>("username")
            .ok_or_else(|| anyhow!("No valid username entered."))?
            .to_owned();
        let p = flags
            .get_one::<String>("password")
            .ok_or_else(|| anyhow!("No valid new password entered"))?
            .to_owned();

        let _ = store.update_password_for(u, p).await?;
        println!("Successfully updated password.");
    } else if let Some(flags) = flags.subcommand_matches("delete") {
        let u = flags
            .get_one::<String>("username")
            .ok_or_else(|| anyhow!("No valid username entered."))?
            .to_owned();

        let _ = store.delete_entry(u).await?;
        println!("Successfully deleted password");
    }

    Ok(())
}
