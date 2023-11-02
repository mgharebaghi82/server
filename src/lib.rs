use axum::Router;
use axum_server::tls_openssl::OpenSSLConfig;
use routers::create_routes;
use std::net::SocketAddr;

mod routers;

pub async fn run() {
    let app: Router = create_routes();

    let config = OpenSSLConfig::from_pem_file(
        "/etc/letsencrypt/live/centichain.org/fullchain.pem",
        "/etc/letsencrypt/live/centichain.org/privkey.pem",
    )
    .unwrap();

    let addr = SocketAddr::from(([0,0,0,0], 3002));
    println!("listening on: {}", addr);

    axum_server::bind_openssl(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
