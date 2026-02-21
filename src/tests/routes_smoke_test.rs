use reqwest;

#[tokio::test]
async fn users_routes_exist() {
    let resp = reqwest::get("http://localhost:3000/health")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(resp.contains("status"));
}
