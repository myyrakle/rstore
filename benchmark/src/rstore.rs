use reqwest::blocking::Client;

pub fn create_rstore_client() -> anyhow::Result<Client> {
    let client = Client::new();

    client.get("http://localhost:13535/").send()?;

    Ok(client)
}

pub fn set_key_value(client: &Client, key: &str, value: &str) -> anyhow::Result<()> {
    let request_body = format!("{{\"key\": \"{}\", \"value\": \"{}\"}}", key, value);
    println!("Request body: {}", request_body);

    client
        .post("http://localhost:13535/value")
        .header("Content-Type", "application/json")
        .body(request_body)
        .send()?;

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct RStoreGetResponse {
    pub value: String,
}

pub fn get_key_value(client: &Client, key: &str) -> anyhow::Result<String> {
    let response = client
        .get(format!("http://localhost:13535/value?key={key}"))
        .send()?;

    let response_body = response.text()?;
    println!("Response body: {}", response_body);

    let response: RStoreGetResponse = serde_json::from_str(&response_body)?;
    let value = response.value;

    Ok(value)
}

pub fn clear_all(client: &Client) -> anyhow::Result<()> {
    client.delete("http://localhost:13535/clear").send()?;

    Ok(())
}
