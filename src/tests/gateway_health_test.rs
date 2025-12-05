use reqwest;

#[tokio::test]
async fn gateway_health_works() {
    let resp = reqwest::get("http://localhost:3000/users/health")
        .await
        .unwrap()
        .status();

    assert!(resp.is_success());
}
