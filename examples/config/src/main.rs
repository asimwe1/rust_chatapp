#[cfg(test)] mod tests;

use rocket::fairing::AdHoc;

// This example's illustration is the Rocket.toml file. Running this server will
// print the config, however.
#[rocket::launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(AdHoc::on_attach("Config Reader", |rocket| async {
            let value = rocket.figment().find_value("").unwrap();
            println!("{:#?}", value);
            Ok(rocket)
        }))
}
