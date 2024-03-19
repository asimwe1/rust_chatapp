use std::net::{SocketAddr, Ipv4Addr};

use rocket::config::Config;
use rocket::fairing::AdHoc;
use rocket::futures::channel::oneshot;

#[rocket::async_test]
async fn on_ignite_fairing_can_inspect_port() {
    let (tx, rx) = oneshot::channel();
    let rocket = rocket::custom(Config::debug_default())
        .attach(AdHoc::on_liftoff("Send Port -> Channel", move |rocket| {
            Box::pin(async move {
                let tcp = rocket.endpoints().find_map(|v| v.tcp());
                tx.send(tcp.unwrap().port()).expect("send okay");
            })
        }));

    rocket::tokio::spawn(rocket.launch_on(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))));
    assert_ne!(rx.await.unwrap(), 0);
}
