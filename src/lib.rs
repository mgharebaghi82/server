use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use routers::create_routes;
use std::net::SocketAddr;

mod routers;

pub async fn run() {
    let app: Router = create_routes();

    let config = RustlsConfig::from_pem_file(
        "/etc/letsencrypt/live/centichain.org/fullchain.pem",
        "/etc/letsencrypt/live/centichain.org/privkey.pem",
    )
    .await
    .unwrap();

    let addr = SocketAddr::from(([0,0,0,0], 3002));
    println!("listening on: {}", addr);

    axum_server::bind_rustls(addr, config)
    .serve(app.into_make_service())
    .await
    .unwrap();

    // axum::Server::bind(&"0.0.0.0:3002".parse().unwrap())
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();
}
